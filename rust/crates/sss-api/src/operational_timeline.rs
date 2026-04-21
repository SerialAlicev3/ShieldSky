use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sss_storage::{PassiveRegionRunStatus, PassiveSourceHealthSample};

use crate::map_views::{build_region_map_response, MapPriority};
use crate::state::{now_unix_seconds, AppError, AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct OperationalTimelineQuery {
    pub window_hours: Option<u64>,
    pub bucket_hours: Option<u64>,
    pub include_current_snapshot: Option<bool>,
    pub include_empty_buckets: Option<bool>,
    pub min_priority: Option<MapPriority>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalTimelineResponse {
    pub generated_at_unix_seconds: i64,
    pub region_id: String,
    pub window_hours: u64,
    pub bucket_hours: u64,
    pub bucket_count: usize,
    pub current_snapshot: Option<OperationalPressureSnapshot>,
    pub buckets: Vec<OperationalPressureTimelineBucket>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalPressureSnapshot {
    pub generated_at_unix_seconds: i64,
    pub pressure_score: f64,
    pub pressure_priority: MapPriority,
    pub active_lease_count: usize,
    pub stale_lease_count: usize,
    pub active_worker_count: usize,
    pub stale_worker_count: usize,
    pub latest_run_status: Option<String>,
    pub latest_run_finished_at_unix_seconds: Option<i64>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalPressureTimelineBucket {
    pub label: String,
    pub start_unix_seconds: i64,
    pub end_unix_seconds: i64,
    pub pressure_score: f64,
    pub pressure_priority: MapPriority,
    pub run_count: usize,
    pub failed_run_count: usize,
    pub partial_run_count: usize,
    pub lease_loss_count: usize,
    pub source_sample_count: usize,
    pub source_failure_count: usize,
    pub dominant_sources: Vec<String>,
    pub summary: String,
    pub suggested_read_paths: Vec<String>,
}

#[derive(Debug, Default)]
struct BucketAccumulator {
    run_count: usize,
    failed_run_count: usize,
    partial_run_count: usize,
    lease_loss_count: usize,
    source_sample_count: usize,
    source_failure_count: usize,
    source_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Copy)]
struct CurrentPressureInputs {
    active_lease_count: usize,
    stale_lease_count: usize,
    active_worker_count: usize,
    stale_worker_count: usize,
    has_latest_run: bool,
    failed_run_count: usize,
    partial_run_count: usize,
    lease_loss_count: usize,
}

type BucketRow = (usize, i64, i64, BucketAccumulator);

pub fn build_region_operational_timeline(
    state: &AppState,
    region_id: &str,
    query: &OperationalTimelineQuery,
) -> Result<Option<OperationalTimelineResponse>, AppError> {
    if state.passive_region_target(region_id)?.is_none() {
        return Ok(None);
    }

    let generated_at_unix_seconds = now_unix_seconds();
    let window_hours = query.window_hours.unwrap_or(72).clamp(6, 720);
    let bucket_hours = query
        .bucket_hours
        .unwrap_or(6)
        .clamp(1, 24)
        .min(window_hours);
    let bucket_seconds = hours_to_seconds(bucket_hours);
    let window_start = generated_at_unix_seconds.saturating_sub(hours_to_seconds(window_hours));
    let bucket_count = usize::try_from(window_hours.div_ceil(bucket_hours)).unwrap_or(usize::MAX);

    let mut buckets = initialize_buckets(
        bucket_count,
        window_start,
        bucket_seconds,
        generated_at_unix_seconds,
    );
    fill_buckets_from_runs(state, region_id, window_start, &mut buckets)?;
    fill_buckets_from_source_health(state, region_id, window_start, &mut buckets)?;

    let current_snapshot = if query.include_current_snapshot.unwrap_or(true) {
        Some(build_current_snapshot(
            state,
            region_id,
            generated_at_unix_seconds,
        )?)
    } else {
        None
    };

    let min_priority = query.min_priority.unwrap_or(MapPriority::Low);
    let include_empty_buckets = query.include_empty_buckets.unwrap_or(true);
    let buckets = buckets
        .into_iter()
        .map(|(index, start, end, accumulator)| {
            build_bucket(
                region_id,
                index,
                start,
                end,
                generated_at_unix_seconds,
                accumulator,
            )
        })
        .filter(|bucket| bucket.pressure_priority >= min_priority)
        .filter(|bucket| include_empty_buckets || bucket_has_activity(bucket))
        .collect::<Vec<_>>();

    Ok(Some(OperationalTimelineResponse {
        generated_at_unix_seconds,
        region_id: region_id.to_string(),
        window_hours,
        bucket_hours,
        bucket_count: buckets.len(),
        current_snapshot,
        buckets,
    }))
}

fn initialize_buckets(
    bucket_count: usize,
    window_start: i64,
    bucket_seconds: i64,
    generated_at_unix_seconds: i64,
) -> Vec<BucketRow> {
    (0..bucket_count)
        .map(|index| {
            let start = window_start.saturating_add(
                i64::try_from(index)
                    .unwrap_or(i64::MAX)
                    .saturating_mul(bucket_seconds),
            );
            let end = if index + 1 == bucket_count {
                generated_at_unix_seconds
            } else {
                start.saturating_add(bucket_seconds)
            };
            (index, start, end, BucketAccumulator::default())
        })
        .collect()
}

fn fill_buckets_from_runs(
    state: &AppState,
    region_id: &str,
    window_start: i64,
    buckets: &mut [BucketRow],
) -> Result<(), AppError> {
    let run_logs = state.passive_region_run_logs(500, Some(region_id))?;
    for run in run_logs
        .iter()
        .filter(|run| run.finished_at_unix_seconds >= window_start)
    {
        if let Some((_, _, _, bucket)) = bucket_for_timestamp(buckets, run.finished_at_unix_seconds)
        {
            bucket.run_count += 1;
            match run.status {
                PassiveRegionRunStatus::Failed => bucket.failed_run_count += 1,
                PassiveRegionRunStatus::Partial => bucket.partial_run_count += 1,
                PassiveRegionRunStatus::Completed
                | PassiveRegionRunStatus::CompletedWithNoDueRegions
                | PassiveRegionRunStatus::DryRun => {}
            }
            if run
                .source_errors
                .iter()
                .any(|error| error.starts_with("lease_lost:"))
            {
                bucket.lease_loss_count += 1;
            }
        }
    }
    Ok(())
}

fn fill_buckets_from_source_health(
    state: &AppState,
    region_id: &str,
    window_start: i64,
    buckets: &mut [BucketRow],
) -> Result<(), AppError> {
    let source_samples = state.passive_source_health_samples(500, None, Some(region_id))?;
    for sample in source_samples.iter().filter(|sample| {
        sample.region_id.as_deref() == Some(region_id)
            && sample.generated_at_unix_seconds >= window_start
    }) {
        if let Some((_, _, _, bucket)) =
            bucket_for_timestamp(buckets, sample.generated_at_unix_seconds)
        {
            bucket.source_sample_count += 1;
            if !sample.fetched {
                bucket.source_failure_count += 1;
            }
            let source = source_name(sample);
            *bucket.source_counts.entry(source).or_default() += 1;
        }
    }
    Ok(())
}

fn build_current_snapshot(
    state: &AppState,
    region_id: &str,
    generated_at_unix_seconds: i64,
) -> Result<OperationalPressureSnapshot, AppError> {
    let diagnostics = state.passive_worker_diagnostics(12, 180, Some(region_id))?;
    let latest_run = state
        .passive_region_run_logs(1, Some(region_id))?
        .into_iter()
        .next();
    let map_region = build_region_map_response(state)?
        .regions
        .into_iter()
        .find(|region| region.region_id == region_id);
    let pressure_score = current_pressure_score(CurrentPressureInputs {
        active_lease_count: diagnostics.active_region_lease_count,
        stale_lease_count: diagnostics.stale_region_lease_count,
        active_worker_count: diagnostics.active_worker_heartbeat_count,
        stale_worker_count: diagnostics.stale_worker_heartbeat_count,
        has_latest_run: latest_run.is_some(),
        failed_run_count: map_region
            .as_ref()
            .map_or(0, |region| region.recent_failed_run_count),
        partial_run_count: map_region
            .as_ref()
            .map_or(0, |region| region.recent_partial_run_count),
        lease_loss_count: map_region
            .as_ref()
            .map_or(0, |region| region.recent_lease_loss_count),
    });
    let pressure_priority = priority_from_score(pressure_score);
    let latest_run_status = latest_run.as_ref().map(|run| format!("{:?}", run.status));
    let latest_run_finished_at_unix_seconds =
        latest_run.as_ref().map(|run| run.finished_at_unix_seconds);

    Ok(OperationalPressureSnapshot {
        generated_at_unix_seconds,
        pressure_score,
        pressure_priority,
        active_lease_count: diagnostics.active_region_lease_count,
        stale_lease_count: diagnostics.stale_region_lease_count,
        active_worker_count: diagnostics.active_worker_heartbeat_count,
        stale_worker_count: diagnostics.stale_worker_heartbeat_count,
        latest_run_status,
        latest_run_finished_at_unix_seconds,
        summary: snapshot_summary(pressure_priority, &diagnostics.recommendation),
    })
}

fn build_bucket(
    region_id: &str,
    index: usize,
    start: i64,
    end: i64,
    generated_at_unix_seconds: i64,
    accumulator: BucketAccumulator,
) -> OperationalPressureTimelineBucket {
    let pressure_score = bucket_pressure_score(&accumulator);
    let pressure_priority = priority_from_score(pressure_score);
    let summary = bucket_summary(index, pressure_priority, &accumulator);
    let dominant_sources = dominant_sources(accumulator.source_counts);
    let hours_ago_start = generated_at_unix_seconds
        .saturating_sub(start)
        .checked_div(3_600)
        .unwrap_or_default();
    let hours_ago_end = generated_at_unix_seconds
        .saturating_sub(end)
        .checked_div(3_600)
        .unwrap_or_default();

    OperationalPressureTimelineBucket {
        label: format!("t-{hours_ago_start}h..t-{hours_ago_end}h"),
        start_unix_seconds: start,
        end_unix_seconds: end,
        pressure_score,
        pressure_priority,
        run_count: accumulator.run_count,
        failed_run_count: accumulator.failed_run_count,
        partial_run_count: accumulator.partial_run_count,
        lease_loss_count: accumulator.lease_loss_count,
        source_sample_count: accumulator.source_sample_count,
        source_failure_count: accumulator.source_failure_count,
        dominant_sources,
        summary,
        suggested_read_paths: bucket_read_paths(region_id),
    }
}

fn bucket_read_paths(region_id: &str) -> Vec<String> {
    vec![
        format!("/v1/passive/regions/runs?region_id={region_id}"),
        format!("/v1/passive/worker/diagnostics?region_id={region_id}"),
        format!("/v1/passive/source-health/samples?limit=20&region_id={region_id}"),
        format!("/v1/passive/regions/{region_id}/remediation"),
    ]
}

fn bucket_for_timestamp(buckets: &mut [BucketRow], timestamp: i64) -> Option<&mut BucketRow> {
    buckets
        .iter_mut()
        .find(|(_, start, end, _)| timestamp >= *start && timestamp <= *end)
}

fn source_name(sample: &PassiveSourceHealthSample) -> String {
    format!("{:?}", sample.source_kind)
}

fn dominant_sources(source_counts: BTreeMap<String, usize>) -> Vec<String> {
    let mut sources = source_counts.into_iter().collect::<Vec<_>>();
    sources.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    sources
        .into_iter()
        .take(3)
        .map(|(source, _)| source)
        .collect()
}

fn bucket_pressure_score(bucket: &BucketAccumulator) -> f64 {
    let mut pressure: f64 = 0.0;
    if bucket.failed_run_count > 0 {
        pressure += 0.35;
    }
    if bucket.partial_run_count > 0 {
        pressure += 0.20;
    }
    if bucket.lease_loss_count > 0 {
        pressure += 0.45;
    }
    if bucket.source_failure_count > 0 {
        pressure += 0.15;
    }
    if bucket.run_count > 0 && bucket.source_failure_count > 0 {
        pressure += 0.05;
    }
    pressure.clamp(0.0, 1.0)
}

fn bucket_has_activity(bucket: &OperationalPressureTimelineBucket) -> bool {
    bucket.run_count > 0
        || bucket.failed_run_count > 0
        || bucket.partial_run_count > 0
        || bucket.lease_loss_count > 0
        || bucket.source_sample_count > 0
        || bucket.source_failure_count > 0
}

fn current_pressure_score(inputs: CurrentPressureInputs) -> f64 {
    let mut pressure: f64 = 0.0;
    if inputs.stale_lease_count > 0 {
        pressure += 0.35;
    }
    if inputs.stale_worker_count > 0 {
        pressure += 0.25;
    }
    if inputs.failed_run_count > 0 {
        pressure += 0.35;
    }
    if inputs.partial_run_count > 0 {
        pressure += 0.20;
    }
    if inputs.lease_loss_count > 0 {
        pressure += 0.45;
    }
    if inputs.active_lease_count == 0 && inputs.active_worker_count == 0 && inputs.has_latest_run {
        pressure += 0.15;
    }
    pressure.clamp(0.0, 1.0)
}

fn priority_from_score(score: f64) -> MapPriority {
    if score >= 0.85 {
        MapPriority::Critical
    } else if score >= 0.65 {
        MapPriority::High
    } else if score >= 0.35 {
        MapPriority::Medium
    } else {
        MapPriority::Low
    }
}

fn bucket_summary(index: usize, priority: MapPriority, bucket: &BucketAccumulator) -> String {
    if bucket.run_count == 0 && bucket.source_sample_count == 0 {
        return format!("Bucket {} has no recorded worker activity.", index + 1);
    }
    format!(
        "Bucket {} is {:?}: {} runs, {} failed, {} partial, {} lease-loss signals, {} source failures.",
        index + 1,
        priority,
        bucket.run_count,
        bucket.failed_run_count,
        bucket.partial_run_count,
        bucket.lease_loss_count,
        bucket.source_failure_count
    )
}

fn snapshot_summary(priority: MapPriority, recommendation: &str) -> String {
    format!("Current region posture is {priority:?}. {recommendation}")
}

fn hours_to_seconds(hours: u64) -> i64 {
    i64::try_from(hours)
        .unwrap_or(i64::MAX)
        .saturating_mul(3_600)
}
