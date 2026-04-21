use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sss_site_registry::SiteType;

use crate::map_views::{MapCoordinates, MapPriority, MapSeverity, SiteMapType};
use crate::state::{now_unix_seconds, AppError, AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct CanonicalEventsQuery {
    pub region_id: Option<String>,
    pub window_hours: Option<u64>,
    pub bucket_minutes: Option<i64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPassiveEvent {
    pub canonical_event_id: String,
    pub region_id: Option<String>,
    pub site_id: String,
    pub site_name: String,
    pub site_type: SiteMapType,
    pub event_type: String,
    pub severity: MapSeverity,
    pub priority: MapPriority,
    pub risk_score: f64,
    pub confidence: f64,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
    pub bucket_start_unix_seconds: i64,
    pub support_count: usize,
    pub source_event_ids: Vec<String>,
    pub coordinates: MapCoordinates,
    pub summary: String,
    pub status: CanonicalEventStatus,
    pub status_summary: String,
    pub temporal_phase: TemporalPhase,
    pub operational_readout: String,
    pub risk_delta: RiskDelta,
    pub bundle_hashes: Vec<String>,
    pub manifest_hashes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalEventStatus {
    New,
    Recurring,
    Escalating,
    Cooling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalPhase {
    Immediate,
    Active,
    Monitoring,
    CoolingWatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskDeltaType {
    Spike,
    Increase,
    Decrease,
    Stable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskDelta {
    pub previous_score: Option<f64>,
    pub current_score: f64,
    pub delta: f64,
    pub classification: RiskDeltaType,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CanonicalEventProjection {
    pub status: CanonicalEventStatus,
    pub temporal_phase: TemporalPhase,
    pub operational_readout: String,
    pub risk_delta: RiskDelta,
    pub status_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalEventsResponse {
    pub generated_at_unix_seconds: i64,
    pub region_id: Option<String>,
    pub window_hours: u64,
    pub bucket_minutes: i64,
    pub raw_event_count: usize,
    pub canonical_event_count: usize,
    pub compression_ratio: f64,
    pub events: Vec<CanonicalPassiveEvent>,
}

pub fn build_canonical_events_response(
    state: &AppState,
    query: &CanonicalEventsQuery,
) -> Result<CanonicalEventsResponse, AppError> {
    let generated_at = now_unix_seconds();
    let window_hours = query.window_hours.unwrap_or(24).clamp(1, 720);
    let bucket_minutes = query.bucket_minutes.unwrap_or(60).clamp(5, 1_440);
    let after_unix_seconds =
        generated_at.saturating_sub(i64::try_from(window_hours).unwrap_or(720) * 3_600);
    let limit = query.limit.unwrap_or(100).clamp(1, 500);

    let site_types = state
        .passive_sites()?
        .into_iter()
        .map(|site| {
            (
                site.site.site_id.to_string(),
                site_map_type(site.site.site_type),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let site_regions = site_region_map(state)?;

    let mut canonical_history =
        state.canonical_passive_events(limit.max(100), query.region_id.as_deref(), None)?;
    canonical_history.sort_by(|left, right| {
        left.first_seen_at_unix_seconds
            .cmp(&right.first_seen_at_unix_seconds)
            .then_with(|| {
                left.last_seen_at_unix_seconds
                    .cmp(&right.last_seen_at_unix_seconds)
            })
    });
    let mut canonical_events = canonical_history.clone();
    canonical_events.retain(|event| event.last_seen_at_unix_seconds >= after_unix_seconds);

    let raw_event_count = canonical_events
        .iter()
        .map(|event| usize::try_from(event.occurrence_count).unwrap_or(usize::MAX))
        .sum();

    let mut events = canonical_events
        .into_iter()
        .map(|event| {
            let projection = project_canonical_event(&canonical_history, &event, generated_at);
            CanonicalPassiveEvent {
                canonical_event_id: event.canonical_id,
                region_id: site_regions.get(&event.site_id).cloned(),
                site_id: event.site_id.clone(),
                site_name: event.site_name,
                site_type: site_types
                    .get(&event.site_id)
                    .copied()
                    .unwrap_or(SiteMapType::Substation),
                event_type: threat_type_name(event.threat_type),
                severity: severity_from_risk(event.max_risk_score),
                priority: priority_from_score(event.max_risk_score),
                risk_score: event.max_risk_score,
                confidence: event.avg_confidence,
                first_observed_at_unix_seconds: event.first_seen_at_unix_seconds,
                last_observed_at_unix_seconds: event.last_seen_at_unix_seconds,
                bucket_start_unix_seconds: bucket_start(
                    event.first_seen_at_unix_seconds,
                    bucket_minutes,
                ),
                support_count: usize::try_from(event.occurrence_count).unwrap_or(usize::MAX),
                source_event_ids: event.related_event_ids,
                coordinates: MapCoordinates {
                    lat: event.latitude,
                    lon: event.longitude,
                },
                summary: event.summary,
                status: projection.status,
                status_summary: projection.status_summary,
                temporal_phase: projection.temporal_phase,
                operational_readout: projection.operational_readout,
                risk_delta: projection.risk_delta,
                bundle_hashes: event.evidence_bundle_hashes,
                manifest_hashes: event.replay_manifest_hashes,
            }
        })
        .collect::<Vec<_>>();

    events.sort_by(|left, right| {
        right
            .risk_score
            .total_cmp(&left.risk_score)
            .then_with(|| right.support_count.cmp(&left.support_count))
            .then_with(|| {
                right
                    .last_observed_at_unix_seconds
                    .cmp(&left.last_observed_at_unix_seconds)
            })
    });
    events.truncate(limit);

    let canonical_event_count = events.len();
    Ok(CanonicalEventsResponse {
        generated_at_unix_seconds: generated_at,
        region_id: query.region_id.clone(),
        window_hours,
        bucket_minutes,
        raw_event_count,
        canonical_event_count,
        compression_ratio: compression_ratio(raw_event_count, canonical_event_count),
        events,
    })
}

fn bucket_start(observed_at_unix_seconds: i64, bucket_minutes: i64) -> i64 {
    let bucket_seconds = bucket_minutes.saturating_mul(60).max(60);
    observed_at_unix_seconds - observed_at_unix_seconds.rem_euclid(bucket_seconds)
}

fn seed_matches_region(
    record: &sss_storage::PassiveSeedRecord,
    region: &sss_storage::PassiveRegionTarget,
) -> bool {
    let seed = &record.seed;
    let within_bbox = seed.latitude >= region.south
        && seed.latitude <= region.north
        && seed.longitude >= region.west
        && seed.longitude <= region.east;
    let matches_type = region
        .site_types
        .as_ref()
        .is_none_or(|site_types| site_types.contains(&seed.site_type));
    within_bbox && matches_type
}

fn site_region_map(state: &AppState) -> Result<BTreeMap<String, String>, AppError> {
    let seed_records = state.passive_seed_records(10_000)?;
    let regions = state.passive_region_targets(10_000, true)?;
    Ok(seed_records
        .iter()
        .filter_map(|record| {
            let region = regions
                .iter()
                .find(|region| seed_matches_region(record, region))?;
            record
                .site_id
                .as_ref()
                .map(|site_id| (site_id.clone(), region.region_id.clone()))
        })
        .collect())
}

pub(crate) fn project_canonical_event(
    history: &[sss_storage::CanonicalPassiveEvent],
    current: &sss_storage::CanonicalPassiveEvent,
    now_unix_seconds: i64,
) -> CanonicalEventProjection {
    let previous = previous_canonical_event(history, current);
    let risk_delta = build_risk_delta(previous, current);
    let status = canonical_event_status(current, previous, &risk_delta, now_unix_seconds);
    let temporal_phase =
        temporal_phase(status, current.last_seen_at_unix_seconds, now_unix_seconds);
    CanonicalEventProjection {
        status,
        temporal_phase,
        operational_readout: operational_readout(
            status,
            temporal_phase,
            &risk_delta,
            current,
            now_unix_seconds,
        ),
        status_summary: canonical_status_summary(
            status,
            temporal_phase,
            &risk_delta,
            current,
            now_unix_seconds,
        ),
        risk_delta,
    }
}

fn previous_canonical_event<'a>(
    history: &'a [sss_storage::CanonicalPassiveEvent],
    current: &sss_storage::CanonicalPassiveEvent,
) -> Option<&'a sss_storage::CanonicalPassiveEvent> {
    history
        .iter()
        .filter(|candidate| {
            candidate.site_id == current.site_id
                && candidate.threat_type == current.threat_type
                && candidate.first_seen_at_unix_seconds < current.first_seen_at_unix_seconds
        })
        .max_by(|left, right| {
            left.first_seen_at_unix_seconds
                .cmp(&right.first_seen_at_unix_seconds)
                .then_with(|| {
                    left.last_seen_at_unix_seconds
                        .cmp(&right.last_seen_at_unix_seconds)
                })
        })
}

fn build_risk_delta(
    previous: Option<&sss_storage::CanonicalPassiveEvent>,
    current: &sss_storage::CanonicalPassiveEvent,
) -> RiskDelta {
    let previous_score = previous.map(|event| event.max_risk_score);
    let delta = current.max_risk_score - previous_score.unwrap_or(0.0);
    let classification = if (previous.is_none() && current.max_risk_score >= 0.75) || delta >= 0.20
    {
        RiskDeltaType::Spike
    } else if delta >= 0.08 {
        RiskDeltaType::Increase
    } else if delta <= -0.08 {
        RiskDeltaType::Decrease
    } else {
        RiskDeltaType::Stable
    };

    let explanation = match (previous_score, classification) {
        (None, RiskDeltaType::Spike) => format!(
            "First canonical occurrence for this threat class arrived with high risk {:.2}.",
            current.max_risk_score
        ),
        (None, _) => "First canonical occurrence for this threat class at this site.".to_string(),
        (Some(previous_score), RiskDeltaType::Spike) => format!(
            "Risk spiked from {:.2} to {:.2} against the previous canonical window.",
            previous_score, current.max_risk_score
        ),
        (Some(previous_score), RiskDeltaType::Increase) => format!(
            "Risk increased from {:.2} to {:.2} against the previous canonical window.",
            previous_score, current.max_risk_score
        ),
        (Some(previous_score), RiskDeltaType::Decrease) => format!(
            "Risk eased from {:.2} to {:.2} against the previous canonical window.",
            previous_score, current.max_risk_score
        ),
        (Some(previous_score), RiskDeltaType::Stable) => format!(
            "Risk stayed near {:.2} with a current score of {:.2}.",
            previous_score, current.max_risk_score
        ),
    };

    RiskDelta {
        previous_score,
        current_score: current.max_risk_score,
        delta,
        classification,
        explanation,
    }
}

fn canonical_event_status(
    current: &sss_storage::CanonicalPassiveEvent,
    previous: Option<&sss_storage::CanonicalPassiveEvent>,
    risk_delta: &RiskDelta,
    now_unix_seconds: i64,
) -> CanonicalEventStatus {
    let age_seconds = now_unix_seconds.saturating_sub(current.last_seen_at_unix_seconds);

    if previous.is_none() && age_seconds <= 21_600 {
        CanonicalEventStatus::New
    } else if matches!(
        risk_delta.classification,
        RiskDeltaType::Spike | RiskDeltaType::Increase
    ) && age_seconds <= 43_200
    {
        CanonicalEventStatus::Escalating
    } else if matches!(risk_delta.classification, RiskDeltaType::Decrease) {
        CanonicalEventStatus::Cooling
    } else {
        CanonicalEventStatus::Recurring
    }
}

fn canonical_status_summary(
    status: CanonicalEventStatus,
    temporal_phase: TemporalPhase,
    risk_delta: &RiskDelta,
    current: &sss_storage::CanonicalPassiveEvent,
    now_unix_seconds: i64,
) -> String {
    let recency =
        recency_phrase(now_unix_seconds.saturating_sub(current.last_seen_at_unix_seconds));
    let phase = temporal_phase_label(temporal_phase);
    match status {
        CanonicalEventStatus::New => format!(
            "{phase} new phenomenon with {} supporting signals; last seen {recency}.",
            current.occurrence_count,
        ),
        CanonicalEventStatus::Recurring => format!(
            "{phase} recurring phenomenon sustained by {} supporting signals; last seen {recency}. {}",
            current.occurrence_count, risk_delta.explanation
        ),
        CanonicalEventStatus::Escalating => format!(
            "{phase} escalating phenomenon with {} supporting signals; last seen {recency}. {}",
            current.occurrence_count, risk_delta.explanation
        ),
        CanonicalEventStatus::Cooling => format!(
            "{phase} cooling phenomenon after {} supporting signals; last seen {recency}. {}",
            current.occurrence_count, risk_delta.explanation
        ),
    }
}

pub(crate) fn temporal_phase(
    status: CanonicalEventStatus,
    last_observed_at_unix_seconds: i64,
    now_unix_seconds: i64,
) -> TemporalPhase {
    let age_seconds = now_unix_seconds.saturating_sub(last_observed_at_unix_seconds);
    if matches!(status, CanonicalEventStatus::Cooling) {
        TemporalPhase::CoolingWatch
    } else if age_seconds <= 7_200 {
        TemporalPhase::Immediate
    } else if age_seconds <= 43_200 {
        TemporalPhase::Active
    } else {
        TemporalPhase::Monitoring
    }
}

pub(crate) fn operational_readout(
    status: CanonicalEventStatus,
    temporal_phase: TemporalPhase,
    risk_delta: &RiskDelta,
    current: &sss_storage::CanonicalPassiveEvent,
    now_unix_seconds: i64,
) -> String {
    let recency =
        recency_phrase(now_unix_seconds.saturating_sub(current.last_seen_at_unix_seconds));
    format!(
        "{}: {} event at risk {:.2} with {} supporting signals, confidence {:.2}, last seen {}. Delta posture: {}.",
        temporal_phase_label(temporal_phase),
        status_label(status),
        current.max_risk_score,
        current.occurrence_count,
        current.avg_confidence,
        recency,
        risk_delta.explanation,
    )
}

fn temporal_phase_label(temporal_phase: TemporalPhase) -> &'static str {
    match temporal_phase {
        TemporalPhase::Immediate => "Immediate",
        TemporalPhase::Active => "Active",
        TemporalPhase::Monitoring => "Monitoring",
        TemporalPhase::CoolingWatch => "Cooling watch",
    }
}

fn status_label(status: CanonicalEventStatus) -> &'static str {
    match status {
        CanonicalEventStatus::New => "new",
        CanonicalEventStatus::Recurring => "recurring",
        CanonicalEventStatus::Escalating => "escalating",
        CanonicalEventStatus::Cooling => "cooling",
    }
}

fn recency_phrase(age_seconds: i64) -> String {
    if age_seconds < 3_600 {
        format!("{}m ago", age_seconds.max(0) / 60)
    } else if age_seconds < 86_400 {
        format!("{}h ago", age_seconds / 3_600)
    } else {
        format!("{}d ago", age_seconds / 86_400)
    }
}

fn compression_ratio(raw_count: usize, canonical_count: usize) -> f64 {
    if raw_count == 0 {
        1.0
    } else {
        let raw = u32::try_from(raw_count).unwrap_or(u32::MAX);
        let canonical = u32::try_from(canonical_count).unwrap_or(u32::MAX);
        f64::from(canonical) / f64::from(raw)
    }
}

fn priority_from_score(risk_score: f64) -> MapPriority {
    if risk_score >= 0.85 {
        MapPriority::Critical
    } else if risk_score >= 0.65 {
        MapPriority::High
    } else if risk_score >= 0.35 {
        MapPriority::Medium
    } else {
        MapPriority::Low
    }
}

fn severity_from_risk(risk_score: f64) -> MapSeverity {
    match priority_from_score(risk_score) {
        MapPriority::Critical => MapSeverity::Critical,
        MapPriority::High => MapSeverity::High,
        MapPriority::Medium => MapSeverity::Medium,
        MapPriority::Low => MapSeverity::Low,
    }
}

fn threat_type_name(threat_type: sss_passive_scanner::PassiveThreatType) -> String {
    format!("{threat_type:?}").chars().enumerate().fold(
        String::new(),
        |mut output, (index, character)| {
            if character.is_uppercase() && index > 0 {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
            output
        },
    )
}

fn site_map_type(site_type: SiteType) -> SiteMapType {
    match site_type {
        SiteType::SolarPlant => SiteMapType::SolarPlant,
        SiteType::DataCenter => SiteMapType::DataCenter,
        SiteType::Substation => SiteMapType::Substation,
    }
}
