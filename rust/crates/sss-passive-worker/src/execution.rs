use std::time::Duration;

use sss_api::state::{AppError, AppState};
use sss_storage::PassiveRegionTarget;

use crate::config::WorkerConfig;
use crate::discovery::{build_region_request, discovery_enabled};
use crate::logging::{PassiveWorkerRunLog, WorkerRunReason};
use crate::scheduler::compute_next_run;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub scanned: usize,
    pub events: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionExecutionReport {
    pub log: PassiveWorkerRunLog,
    pub scan_result: ScanResult,
}

pub async fn run_region(
    state: &AppState,
    config: &WorkerConfig,
    region: &PassiveRegionTarget,
    run_id: &str,
    now_unix_seconds: i64,
) -> RegionExecutionReport {
    let mut log = PassiveWorkerRunLog::start(
        run_id.to_string(),
        region.region_id.clone(),
        now_unix_seconds,
    );
    let force_discovery = discovery_enabled(config);
    let request = build_region_request(region.region_id.clone(), config, force_discovery);
    let run = tokio::time::timeout(
        Duration::from_secs(config.max_runtime_seconds),
        state.run_passive_regions(run_id, &request),
    )
    .await;
    let finished_at = unix_seconds_now();

    match run {
        Ok(Ok(response)) => {
            let next_run_at = compute_next_run(region, finished_at);
            log.finish_success(finished_at, &response, next_run_at, config);
        }
        Ok(Err(error)) => log.finish_failed(finished_at, error.to_string()),
        Err(_) => log.finish_timed_out(finished_at, config.max_runtime_seconds),
    }

    RegionExecutionReport {
        scan_result: ScanResult {
            scanned: log.seeds_scanned,
            events: log.events_created,
        },
        log,
    }
}

impl RegionExecutionReport {
    #[must_use]
    pub fn failed(
        run_id: &str,
        region_id: &str,
        started_at_unix_seconds: i64,
        finished_at_unix_seconds: i64,
        error: impl AsRef<str>,
    ) -> Self {
        Self::failed_with_reason(
            run_id,
            region_id,
            started_at_unix_seconds,
            finished_at_unix_seconds,
            WorkerRunReason::ExecutionFailed,
            error,
        )
    }

    #[must_use]
    pub(crate) fn failed_with_reason(
        run_id: &str,
        region_id: &str,
        started_at_unix_seconds: i64,
        finished_at_unix_seconds: i64,
        reason: WorkerRunReason,
        error: impl AsRef<str>,
    ) -> Self {
        let mut log = PassiveWorkerRunLog::start(
            run_id.to_string(),
            region_id.to_string(),
            started_at_unix_seconds,
        );
        log.finish_failed_with_reason(finished_at_unix_seconds, reason, error);
        Self {
            log,
            scan_result: ScanResult {
                scanned: 0,
                events: 0,
            },
        }
    }
}

#[must_use]
pub fn region_run_id(region_id: &str, now_unix_seconds: i64) -> String {
    format!("passive-worker-{region_id}-{now_unix_seconds}")
}

pub fn ensure_success(report: &RegionExecutionReport) -> Result<(), AppError> {
    if report.log.status == crate::logging::WorkerRunStatus::Failed {
        return Err(AppError::SourceUnavailable(
            report
                .log
                .source_errors
                .first()
                .cloned()
                .unwrap_or_else(|| "passive worker region run failed".to_string()),
        ));
    }
    Ok(())
}

pub fn ensure_cycle_success(reports: &[RegionExecutionReport]) -> Result<(), AppError> {
    if reports.is_empty() {
        return Ok(());
    }

    if reports.iter().all(|report| ensure_success(report).is_err()) {
        return Err(AppError::SourceUnavailable(
            reports
                .iter()
                .flat_map(|report| report.log.source_errors.iter())
                .next()
                .cloned()
                .unwrap_or_else(|| "all passive worker region runs failed".to_string()),
        ));
    }

    Ok(())
}

fn unix_seconds_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .try_into()
        .unwrap_or(i64::MAX)
}
