use std::fmt::Write;

use sss_api::state::{PassiveLiveSourceStatus, PassiveRegionRunResponse};
use sss_storage::{PassiveRegionRunLog, PassiveRegionRunStatus, PassiveRunOrigin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkerRunReason {
    SourceHealthDegraded,
    AllSourcesFailed,
    ExecutionFailed,
    ExecutionTimedOut,
    LeaseLost,
    LeaseReleaseRejected,
}

impl WorkerRunReason {
    fn code(self) -> &'static str {
        match self {
            Self::SourceHealthDegraded => "source_health_degraded",
            Self::AllSourcesFailed => "all_sources_failed",
            Self::ExecutionFailed => "execution_failed",
            Self::ExecutionTimedOut => "execution_timed_out",
            Self::LeaseLost => "lease_lost",
            Self::LeaseReleaseRejected => "lease_release_rejected",
        }
    }

    fn partial_heartbeat_label(self) -> &'static str {
        match self {
            Self::LeaseReleaseRejected => "lease cleanup rejected",
            _ => "source health degraded",
        }
    }

    fn failed_heartbeat_label(self) -> Option<&'static str> {
        match self {
            Self::AllSourcesFailed => Some("all passive live sources failed"),
            Self::ExecutionTimedOut => Some("worker runtime exhausted"),
            Self::LeaseLost => Some("lease continuity lost"),
            Self::SourceHealthDegraded | Self::ExecutionFailed | Self::LeaseReleaseRejected => None,
        }
    }

    fn matches_code(candidate: &str) -> bool {
        matches!(
            candidate,
            "source_health_degraded"
                | "all_sources_failed"
                | "execution_failed"
                | "execution_timed_out"
                | "lease_lost"
                | "lease_release_rejected"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerRunStatus {
    Running,
    Partial,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassiveWorkerRunLog {
    pub run_id: String,
    pub region_id: String,
    pub started_at_unix_seconds: i64,
    pub finished_at_unix_seconds: Option<i64>,
    pub status: WorkerRunStatus,
    pub discovery_triggered: bool,
    pub seeds_discovered: usize,
    pub seeds_scanned: usize,
    pub events_created: usize,
    pub sources_used: Vec<String>,
    pub source_errors: Vec<String>,
    pub next_run_at: Option<i64>,
    all_live_sources_failed: bool,
    primary_reason: Option<WorkerRunReason>,
}

impl PassiveWorkerRunLog {
    #[must_use]
    pub fn start(run_id: String, region_id: String, started_at_unix_seconds: i64) -> Self {
        Self {
            run_id,
            region_id,
            started_at_unix_seconds,
            finished_at_unix_seconds: None,
            status: WorkerRunStatus::Running,
            discovery_triggered: false,
            seeds_discovered: 0,
            seeds_scanned: 0,
            events_created: 0,
            sources_used: Vec::new(),
            source_errors: Vec::new(),
            next_run_at: None,
            all_live_sources_failed: false,
            primary_reason: None,
        }
    }

    pub fn finish_success(
        &mut self,
        finished_at_unix_seconds: i64,
        response: &PassiveRegionRunResponse,
        next_run_at: i64,
    ) {
        self.finished_at_unix_seconds = Some(finished_at_unix_seconds);
        self.status = WorkerRunStatus::Completed;
        self.discovery_triggered = response.discovered_region_count > 0;
        self.seeds_discovered = response.discovered_seed_count;
        self.seeds_scanned = response
            .scheduler
            .as_ref()
            .map_or(0, |scheduler| scheduler.selected_seed_count);
        self.events_created = response
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.execution_summary.as_ref())
            .map_or(0, |summary| summary.event_count);
        let source_statuses = response
            .scheduler
            .as_ref()
            .and_then(|scheduler| scheduler.live_scan.as_ref())
            .map(|scan| {
                scan.source_statuses
                    .iter()
                    .map(WorkerSourceHealthSnapshot::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.primary_reason = None;
        self.ingest_source_health(&source_statuses);
        self.next_run_at = Some(next_run_at);
    }

    pub fn finish_failed(&mut self, finished_at_unix_seconds: i64, error: impl AsRef<str>) {
        self.finish_failed_with_reason(
            finished_at_unix_seconds,
            WorkerRunReason::ExecutionFailed,
            error,
        );
    }

    pub(crate) fn finish_failed_with_reason(
        &mut self,
        finished_at_unix_seconds: i64,
        reason: WorkerRunReason,
        error: impl AsRef<str>,
    ) {
        self.finished_at_unix_seconds = Some(finished_at_unix_seconds);
        self.status = WorkerRunStatus::Failed;
        self.all_live_sources_failed = false;
        self.primary_reason = Some(reason);
        self.source_errors
            .push(format_reason_error(reason, error.as_ref()));
    }

    pub fn finish_timed_out(&mut self, finished_at_unix_seconds: i64, max_runtime_seconds: u64) {
        self.finish_failed_with_reason(
            finished_at_unix_seconds,
            WorkerRunReason::ExecutionTimedOut,
            format!("region run exceeded {max_runtime_seconds} seconds"),
        );
    }

    pub fn mark_lease_lost(&mut self, detail: &str) {
        self.status = WorkerRunStatus::Failed;
        self.all_live_sources_failed = false;
        self.primary_reason = Some(WorkerRunReason::LeaseLost);
        self.source_errors
            .push(format_reason_error(WorkerRunReason::LeaseLost, detail));
    }

    pub fn mark_lease_release_rejected(&mut self, detail: &str) {
        if self.status != WorkerRunStatus::Failed {
            self.status = WorkerRunStatus::Partial;
            self.primary_reason = Some(WorkerRunReason::LeaseReleaseRejected);
        }
        self.all_live_sources_failed = false;
        self.source_errors.push(format_reason_error(
            WorkerRunReason::LeaseReleaseRejected,
            detail,
        ));
    }

    #[must_use]
    pub fn heartbeat_status(&self) -> &'static str {
        match self.status {
            WorkerRunStatus::Running | WorkerRunStatus::Completed => "running",
            WorkerRunStatus::Partial | WorkerRunStatus::Failed => "degraded",
        }
    }

    #[must_use]
    pub fn heartbeat_detail(&self) -> Option<String> {
        match self.status {
            WorkerRunStatus::Running | WorkerRunStatus::Completed => None,
            WorkerRunStatus::Partial => {
                let reason = self
                    .primary_reason
                    .unwrap_or(WorkerRunReason::SourceHealthDegraded);
                summarize_source_errors(&self.source_errors)
                    .map(|summary| format!("{}: {summary}", reason.partial_heartbeat_label()))
            }
            WorkerRunStatus::Failed => {
                let reason = self
                    .primary_reason
                    .unwrap_or(if self.all_live_sources_failed {
                        WorkerRunReason::AllSourcesFailed
                    } else {
                        WorkerRunReason::ExecutionFailed
                    });
                if let Some(label) = reason.failed_heartbeat_label() {
                    summarize_source_errors(&self.source_errors)
                        .map(|summary| format!("{label}: {summary}"))
                } else {
                    first_display_error(&self.source_errors)
                }
            }
        }
    }

    #[must_use]
    pub fn reason_code(&self) -> Option<&'static str> {
        match self.status {
            WorkerRunStatus::Running | WorkerRunStatus::Completed => None,
            WorkerRunStatus::Partial | WorkerRunStatus::Failed => {
                self.primary_reason.map(WorkerRunReason::code)
            }
        }
    }

    #[must_use]
    pub fn as_storage_log(&self) -> PassiveRegionRunLog {
        PassiveRegionRunLog {
            run_id: self.run_id.clone(),
            request_id: self.run_id.clone(),
            origin: PassiveRunOrigin::Worker,
            started_at_unix_seconds: self.started_at_unix_seconds,
            finished_at_unix_seconds: self
                .finished_at_unix_seconds
                .unwrap_or(self.started_at_unix_seconds),
            duration_ms: self
                .finished_at_unix_seconds
                .map(|finished| finished.saturating_sub(self.started_at_unix_seconds))
                .and_then(|seconds| u64::try_from(seconds).ok())
                .unwrap_or_default()
                .saturating_mul(1_000),
            status: match self.status {
                WorkerRunStatus::Running | WorkerRunStatus::Partial => {
                    PassiveRegionRunStatus::Partial
                }
                WorkerRunStatus::Completed => PassiveRegionRunStatus::Completed,
                WorkerRunStatus::Failed => PassiveRegionRunStatus::Failed,
            },
            region_ids: vec![self.region_id.clone()],
            discovery_triggered: self.discovery_triggered,
            evaluated_region_count: 1,
            discovered_region_count: usize::from(self.discovery_triggered),
            skipped_region_count: 0,
            discovered_seed_count: self.seeds_discovered,
            selected_seed_count: self.seeds_scanned,
            event_count: self.events_created,
            sources_used: self.sources_used.clone(),
            source_errors: self.source_errors.clone(),
            next_run_at_unix_seconds: self.next_run_at,
        }
    }

    fn ingest_source_health(&mut self, source_statuses: &[WorkerSourceHealthSnapshot]) {
        self.sources_used.clear();
        self.source_errors.clear();
        self.all_live_sources_failed = false;
        self.primary_reason = None;

        if source_statuses.is_empty() {
            return;
        }

        for status in source_statuses {
            if status.fetched {
                self.sources_used.push(status.source_name.clone());
            } else {
                self.source_errors.push(status.error_summary());
            }
        }

        if self.source_errors.is_empty() {
            return;
        }

        if self.sources_used.is_empty() {
            self.status = WorkerRunStatus::Failed;
            self.all_live_sources_failed = true;
            self.primary_reason = Some(WorkerRunReason::AllSourcesFailed);
            prefix_first_error(&mut self.source_errors, WorkerRunReason::AllSourcesFailed);
            return;
        }

        self.status = WorkerRunStatus::Partial;
        self.primary_reason = Some(WorkerRunReason::SourceHealthDegraded);
        prefix_first_error(
            &mut self.source_errors,
            WorkerRunReason::SourceHealthDegraded,
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkerSourceHealthSnapshot {
    source_name: String,
    fetched: bool,
    observations_collected: usize,
    detail: String,
}

impl WorkerSourceHealthSnapshot {
    fn error_summary(&self) -> String {
        if self.observations_collected > 0 {
            format!(
                "{}: {} ({} observations)",
                self.source_name, self.detail, self.observations_collected
            )
        } else {
            format!("{}: {}", self.source_name, self.detail)
        }
    }
}

impl From<&PassiveLiveSourceStatus> for WorkerSourceHealthSnapshot {
    fn from(value: &PassiveLiveSourceStatus) -> Self {
        Self {
            source_name: format!("{:?}", value.source_kind),
            fetched: value.fetched,
            observations_collected: value.observations_collected,
            detail: value.detail.clone(),
        }
    }
}

fn summarize_source_errors(source_errors: &[String]) -> Option<String> {
    if source_errors.is_empty() {
        return None;
    }

    let visible = source_errors
        .iter()
        .take(2)
        .map(|entry| strip_reason_prefix(entry).to_string())
        .collect::<Vec<_>>();
    let remaining = source_errors.len().saturating_sub(visible.len());
    let mut summary = visible.join("; ");
    if remaining > 0 {
        let _ = write!(summary, " (+{remaining} more)");
    }
    Some(summary)
}

fn first_display_error(source_errors: &[String]) -> Option<String> {
    source_errors
        .first()
        .map(|entry| strip_reason_prefix(entry).to_string())
}

fn prefix_first_error(source_errors: &mut [String], reason: WorkerRunReason) {
    if let Some(first) = source_errors.first_mut() {
        let detail = strip_reason_prefix(first).to_string();
        *first = format_reason_error(reason, &detail);
    }
}

fn format_reason_error(reason: WorkerRunReason, detail: &str) -> String {
    format!("{}: {detail}", reason.code())
}

fn strip_reason_prefix(entry: &str) -> &str {
    if let Some((prefix, detail)) = entry.split_once(": ") {
        if WorkerRunReason::matches_code(prefix) {
            return detail;
        }
    }
    entry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_source_health_marks_run_partial_with_source_context() {
        let mut log = PassiveWorkerRunLog::start("run-1".to_string(), "region-a".to_string(), 100);

        log.ingest_source_health(&[
            snapshot("Adsb", true, 12, "ok"),
            snapshot("Weather", false, 0, "upstream timeout"),
            snapshot("FireSmoke", false, 3, "partial ingest"),
        ]);

        assert_eq!(log.status, WorkerRunStatus::Partial);
        assert_eq!(log.sources_used, vec!["Adsb".to_string()]);
        assert_eq!(
            log.source_errors,
            vec![
                "source_health_degraded: Weather: upstream timeout".to_string(),
                "FireSmoke: partial ingest (3 observations)".to_string()
            ]
        );
        assert_eq!(log.reason_code(), Some("source_health_degraded"));
        assert_eq!(log.heartbeat_status(), "degraded");
        assert_eq!(
            log.heartbeat_detail().as_deref(),
            Some(
                "source health degraded: Weather: upstream timeout; FireSmoke: partial ingest (3 observations)"
            )
        );
    }

    #[test]
    fn fully_failed_source_health_marks_run_failed() {
        let mut log = PassiveWorkerRunLog::start("run-2".to_string(), "region-a".to_string(), 100);

        log.ingest_source_health(&[
            snapshot("Adsb", false, 0, "upstream timeout"),
            snapshot("Weather", false, 0, "service unavailable"),
            snapshot("FireSmoke", false, 0, "rate limited"),
        ]);

        assert_eq!(log.status, WorkerRunStatus::Failed);
        assert!(log.sources_used.is_empty());
        assert_eq!(
            log.source_errors,
            vec![
                "all_sources_failed: Adsb: upstream timeout".to_string(),
                "Weather: service unavailable".to_string(),
                "FireSmoke: rate limited".to_string(),
            ]
        );
        assert_eq!(log.reason_code(), Some("all_sources_failed"));
        assert_eq!(log.heartbeat_status(), "degraded");
        assert_eq!(
            log.heartbeat_detail().as_deref(),
            Some(
                "all passive live sources failed: Adsb: upstream timeout; Weather: service unavailable (+1 more)"
            )
        );
    }

    #[test]
    fn timed_out_runs_persist_a_timeout_reason() {
        let mut log = PassiveWorkerRunLog::start("run-3".to_string(), "region-a".to_string(), 100);

        log.finish_timed_out(130, 20);

        assert_eq!(log.status, WorkerRunStatus::Failed);
        assert_eq!(log.reason_code(), Some("execution_timed_out"));
        assert_eq!(
            log.source_errors,
            vec!["execution_timed_out: region run exceeded 20 seconds".to_string()]
        );
        assert_eq!(
            log.heartbeat_detail().as_deref(),
            Some("worker runtime exhausted: region run exceeded 20 seconds")
        );
    }

    #[test]
    fn lease_release_rejection_degrades_completed_runs_without_marking_them_failed() {
        let mut log = PassiveWorkerRunLog::start("run-4".to_string(), "region-a".to_string(), 100);
        log.status = WorkerRunStatus::Completed;
        log.mark_lease_release_rejected("region lease release rejected");

        assert_eq!(log.status, WorkerRunStatus::Partial);
        assert_eq!(log.reason_code(), Some("lease_release_rejected"));
        assert_eq!(
            log.source_errors,
            vec!["lease_release_rejected: region lease release rejected".to_string()]
        );
        assert_eq!(
            log.heartbeat_detail().as_deref(),
            Some("lease cleanup rejected: region lease release rejected")
        );
    }

    fn snapshot(
        source_name: &str,
        fetched: bool,
        observations_collected: usize,
        detail: &str,
    ) -> WorkerSourceHealthSnapshot {
        WorkerSourceHealthSnapshot {
            source_name: source_name.to_string(),
            fetched,
            observations_collected,
            detail: detail.to_string(),
        }
    }
}
