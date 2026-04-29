use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sss_api::state::{AppError, AppState};
use sss_storage::{PassiveRegionTarget, PassiveRunOrigin, PassiveWorkerHeartbeat};
use tokio::sync::oneshot;
use tokio::task::JoinSet;

use crate::config::WorkerConfig;
use crate::execution::{ensure_cycle_success, region_run_id, run_region, RegionExecutionReport};
use crate::logging::WorkerRunReason;
use crate::scheduler::should_run;

#[derive(Debug, Clone)]
pub struct PassiveWorker {
    state: AppState,
    config: WorkerConfig,
    region_locks: Arc<Mutex<HashSet<String>>>,
    region_failures: Arc<Mutex<RegionFailureTracker>>,
    started_at_unix_seconds: i64,
}

impl PassiveWorker {
    #[must_use]
    pub fn new(state: AppState, config: WorkerConfig) -> Self {
        Self {
            state,
            config,
            region_locks: Arc::new(Mutex::new(HashSet::new())),
            region_failures: Arc::new(Mutex::new(RegionFailureTracker::default())),
            started_at_unix_seconds: unix_seconds_now(),
        }
    }

    pub async fn run_forever(&self) -> Result<(), AppError> {
        loop {
            match self.run_once().await {
                Ok(reports) => {
                    tracing::info!(
                        regions_run = reports.len(),
                        "passive worker cycle completed"
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        retry_seconds = self.config.retry_interval_seconds,
                        error = %error,
                        "passive worker cycle failed"
                    );
                    tokio::time::sleep(Duration::from_secs(self.config.retry_interval_seconds))
                        .await;
                    continue;
                }
            }

            tokio::time::sleep(Duration::from_secs(self.config.poll_interval_seconds)).await;
        }
    }

    pub async fn run_once(&self) -> Result<Vec<RegionExecutionReport>, AppError> {
        let now = unix_seconds_now();
        let purged = self.state.purge_expired_passive_region_leases(now)?;
        if purged > 0 {
            tracing::info!(purged, "passive worker purged expired region leases");
        }
        let purged_heartbeats =
            self.state
                .purge_stale_passive_worker_heartbeats(now.saturating_sub(
                    heartbeat_retention_seconds_i64(self.config.heartbeat_retention_seconds),
                ))?;
        if purged_heartbeats > 0 {
            tracing::info!(
                purged_heartbeats,
                "passive worker purged stale worker heartbeats"
            );
        }
        self.record_heartbeat("running", None, "cycle_start", None)?;
        if !self.config.enable_discovery && !self.config.enable_scan {
            self.record_heartbeat(
                "degraded",
                None,
                "cycle_complete",
                Some("worker idle: discovery and scan both disabled"),
            )?;
            return Ok(Vec::new());
        }
        let regions = self.state.passive_region_targets(10_000, true)?;
        let mut due_regions = Vec::with_capacity(self.config.max_regions_per_cycle);
        for region in regions.into_iter().filter(|region| should_run(region, now)) {
            if !self.region_ready_for_run(&region.region_id, now)? {
                continue;
            }
            if due_regions.len() >= self.config.max_regions_per_cycle {
                break;
            }
            due_regions.push(region);
        }

        let mut reports = Vec::new();
        let mut pending_regions = due_regions.into_iter();
        loop {
            let mut region_tasks = JoinSet::new();
            for region in pending_regions
                .by_ref()
                .take(self.config.max_parallel_regions)
            {
                let worker = self.clone();
                region_tasks.spawn(async move { worker.process_due_region(region, now).await });
            }
            if region_tasks.is_empty() {
                break;
            }

            while let Some(result) = region_tasks.join_next().await {
                match result {
                    Ok(Ok(Some(report))) => reports.push(report),
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => return Err(error),
                    Err(error) => {
                        return Err(AppError::SourceUnavailable(format!(
                            "passive worker region task failed to join: {error}"
                        )));
                    }
                }
            }
        }

        if let Err(error) = ensure_cycle_success(&reports) {
            self.record_heartbeat("degraded", None, "cycle_complete", Some(&error.to_string()))?;
            return Err(error);
        }
        self.record_heartbeat("idle", None, "cycle_complete", None)?;
        Ok(reports)
    }

    async fn process_due_region(
        &self,
        region: PassiveRegionTarget,
        now: i64,
    ) -> Result<Option<RegionExecutionReport>, AppError> {
        if let Some(report) = self.preflight_region_guardrail_report(&region, now) {
            self.record_heartbeat(
                report.log.heartbeat_status(),
                Some(&region.region_id),
                "region_guardrail",
                report.log.heartbeat_detail().as_deref(),
            )?;
            self.state
                .store_passive_region_run_log_entry(&report.log.as_storage_log())?;
            return Ok(Some(report));
        }
        if !self.try_lock_region(&region.region_id) {
            tracing::debug!(
                region_id = %region.region_id,
                "passive worker skipped locked region"
            );
            return Ok(None);
        }

        let Some(run_id) = self.acquire_region_lease(&region, now)? else {
            self.unlock_region(&region.region_id);
            return Ok(None);
        };

        self.record_heartbeat("running", Some(&region.region_id), "region_run", None)?;
        let (stop_renewer, renewer_rx) = oneshot::channel();
        let (lease_loss_tx, mut lease_loss_rx) = oneshot::channel();
        let renewer = self.spawn_region_renewer(
            region.region_id.clone(),
            run_id.clone(),
            renewer_rx,
            lease_loss_tx,
        );
        let state = self.state.clone();
        let config = self.config.clone();
        let region_for_run = region.clone();
        let run_id_for_run = run_id.clone();
        let mut region_run = tokio::spawn(async move {
            run_region(&state, &config, &region_for_run, &run_id_for_run, now).await
        });
        let mut report = tokio::select! {
            result = &mut region_run => match result {
                Ok(report) => report,
                Err(error) => RegionExecutionReport::failed(
                    &run_id,
                    &region.region_id,
                    now,
                    unix_seconds_now(),
                    format!("region worker task failed: {error}"),
                ),
            },
            loss = &mut lease_loss_rx => {
                region_run.abort();
                let detail = loss
                    .unwrap_or_else(|_| "region lease lost during execution".to_string());
                RegionExecutionReport::failed_with_reason(
                    &run_id,
                    &region.region_id,
                    now,
                    unix_seconds_now(),
                    WorkerRunReason::LeaseLost,
                    detail,
                )
            }
        };
        let _ = stop_renewer.send(());
        if let Err(error) = renewer.await {
            tracing::warn!(
                region_id = %region.region_id,
                error = %error,
                "passive worker region renewer task ended unexpectedly"
            );
        }
        if let Some(cooldown_until_unix_seconds) = self.record_region_outcome(&report) {
            report.log.next_run_at = Some(cooldown_until_unix_seconds);
        }
        self.finalize_region_report(&region, &run_id, &mut report)?;
        Ok(Some(report))
    }

    fn acquire_region_lease(
        &self,
        region: &PassiveRegionTarget,
        now: i64,
    ) -> Result<Option<String>, AppError> {
        let run_id = region_run_id(&region.region_id, now);
        let lease = self.state.acquire_passive_region_lease(
            &region.region_id,
            &self.config.worker_id,
            &run_id,
            unix_seconds_now(),
            lease_ttl_seconds_i64(self.config.lease_ttl_seconds),
        )?;
        if lease.is_none() {
            tracing::debug!(
                region_id = %region.region_id,
                worker_id = %self.config.worker_id,
                "passive worker skipped region owned by another lease"
            );
            return Ok(None);
        }
        Ok(Some(run_id))
    }

    fn finalize_region_report(
        &self,
        region: &PassiveRegionTarget,
        run_id: &str,
        report: &mut RegionExecutionReport,
    ) -> Result<(), AppError> {
        self.unlock_region(&region.region_id);
        let released = self.state.release_passive_region_lease(
            &region.region_id,
            &self.config.worker_id,
            run_id,
        )?;
        if !released {
            tracing::warn!(
                region_id = %region.region_id,
                run_id = %run_id,
                worker_id = %self.config.worker_id,
                "passive worker could not release region lease"
            );
            report
                .log
                .mark_lease_release_rejected("region lease release rejected");
        }
        self.record_heartbeat(
            report.log.heartbeat_status(),
            Some(&region.region_id),
            "region_complete",
            report.log.heartbeat_detail().as_deref(),
        )?;
        self.state
            .store_passive_region_run_log_entry(&report.log.as_storage_log())?;
        if report.log.status == crate::logging::WorkerRunStatus::Failed {
            tracing::warn!(
                region_id = %report.log.region_id,
                reason = report.log.reason_code().unwrap_or("unspecified"),
                errors = ?report.log.source_errors,
                "passive worker region run failed; continuing cycle"
            );
        } else {
            tracing::info!(
                region_id = %report.log.region_id,
                status = ?report.log.status,
                reason = report.log.reason_code().unwrap_or("none"),
                seeds_discovered = report.log.seeds_discovered,
                seeds_scanned = report.log.seeds_scanned,
                events_created = report.log.events_created,
                sources_used = ?report.log.sources_used,
                source_errors = ?report.log.source_errors,
                "passive worker region run completed"
            );
        }
        Ok(())
    }

    fn spawn_region_renewer(
        &self,
        region_id: String,
        run_id: String,
        mut stop: oneshot::Receiver<()>,
        lease_loss: oneshot::Sender<String>,
    ) -> tokio::task::JoinHandle<()> {
        let state = self.state.clone();
        let worker_id = self.config.worker_id.clone();
        let lease_ttl_seconds = lease_ttl_seconds_i64(self.config.lease_ttl_seconds);
        let started_at_unix_seconds = self.started_at_unix_seconds;
        let interval = lease_refresh_interval(self.config.lease_ttl_seconds);

        tokio::spawn(async move {
            let mut lease_loss = Some(lease_loss);
            loop {
                tokio::select! {
                    _ = &mut stop => break,
                    () = tokio::time::sleep(interval) => {}
                }

                let now = unix_seconds_now();
                match state.refresh_passive_region_lease(
                    &region_id,
                    &worker_id,
                    &run_id,
                    now,
                    lease_ttl_seconds,
                ) {
                    Ok(true) => {
                        let _ = store_worker_heartbeat(
                            &state,
                            &worker_id,
                            started_at_unix_seconds,
                            "running",
                            Some(&region_id),
                            "lease_renewal",
                            None,
                        );
                    }
                    Ok(false) => {
                        tracing::warn!(
                            region_id = %region_id,
                            run_id = %run_id,
                            worker_id = %worker_id,
                            "passive worker lost region lease during renewal"
                        );
                        if let Some(sender) = lease_loss.take() {
                            let _ = sender.send("region lease could not be renewed".to_string());
                        }
                        let _ = store_worker_heartbeat(
                            &state,
                            &worker_id,
                            started_at_unix_seconds,
                            "degraded",
                            Some(&region_id),
                            "lease_lost",
                            Some("region lease could not be renewed"),
                        );
                        break;
                    }
                    Err(error) => {
                        tracing::warn!(
                            region_id = %region_id,
                            run_id = %run_id,
                            worker_id = %worker_id,
                            error = %error,
                            "passive worker failed to renew region lease"
                        );
                        if let Some(sender) = lease_loss.take() {
                            let _ = sender.send(format!("region lease renewal error: {error}"));
                        }
                        let _ = store_worker_heartbeat(
                            &state,
                            &worker_id,
                            started_at_unix_seconds,
                            "degraded",
                            Some(&region_id),
                            "lease_renewal_error",
                            Some(&error.to_string()),
                        );
                        break;
                    }
                }
            }
        })
    }

    fn record_heartbeat(
        &self,
        status: &str,
        current_region_id: Option<&str>,
        current_phase: &str,
        last_error: Option<&str>,
    ) -> Result<(), AppError> {
        store_worker_heartbeat(
            &self.state,
            &self.config.worker_id,
            self.started_at_unix_seconds,
            status,
            current_region_id,
            current_phase,
            last_error,
        )
    }

    fn try_lock_region(&self, region_id: &str) -> bool {
        let Ok(mut locks) = self.region_locks.lock() else {
            return false;
        };
        locks.insert(region_id.to_string())
    }

    fn unlock_region(&self, region_id: &str) {
        if let Ok(mut locks) = self.region_locks.lock() {
            locks.remove(region_id);
        }
    }

    fn region_ready_for_run(
        &self,
        region_id: &str,
        now_unix_seconds: i64,
    ) -> Result<bool, AppError> {
        if let Some(blocked_until_unix_seconds) =
            self.persisted_blocked_until_unix_seconds(region_id, now_unix_seconds)?
        {
            tracing::info!(
                region_id,
                blocked_until_unix_seconds,
                "passive worker skipped region due to persisted failure backoff"
            );
            return Ok(false);
        }
        let Ok(tracker) = self.region_failures.lock() else {
            return Ok(true);
        };
        let Some(blocked_until_unix_seconds) =
            tracker.blocked_until_unix_seconds(region_id, now_unix_seconds)
        else {
            return Ok(true);
        };
        tracing::info!(
            region_id,
            blocked_until_unix_seconds,
            "passive worker skipped region due to failure backoff"
        );
        Ok(false)
    }

    fn persisted_blocked_until_unix_seconds(
        &self,
        region_id: &str,
        now_unix_seconds: i64,
    ) -> Result<Option<i64>, AppError> {
        let latest_run = self
            .state
            .passive_region_run_logs(1, Some(region_id))?
            .into_iter()
            .find(|log| log.origin == PassiveRunOrigin::Worker);
        Ok(latest_run.and_then(|log| {
            (log.status == sss_storage::PassiveRegionRunStatus::Failed)
                .then_some(log.next_run_at_unix_seconds)
                .flatten()
                .filter(|blocked_until_unix_seconds| *blocked_until_unix_seconds > now_unix_seconds)
        }))
    }

    fn record_region_outcome(&self, report: &RegionExecutionReport) -> Option<i64> {
        let Ok(mut tracker) = self.region_failures.lock() else {
            return None;
        };
        let cooldown_base_seconds = self.config.retry_interval_seconds;
        let cooldown_cap_seconds = self.config.retry_interval_seconds.saturating_mul(8);
        if let Some(cooldown_until_unix_seconds) =
            tracker.record_outcome(report, cooldown_base_seconds, cooldown_cap_seconds)
        {
            tracing::warn!(
                region_id = %report.log.region_id,
                reason = report.log.reason_code().unwrap_or("unspecified"),
                cooldown_until_unix_seconds,
                error = ?report.log.source_errors.first(),
                "passive worker applied regional failure backoff"
            );
            return Some(cooldown_until_unix_seconds);
        }
        None
    }

    fn preflight_region_guardrail_report(
        &self,
        region: &PassiveRegionTarget,
        now_unix_seconds: i64,
    ) -> Option<RegionExecutionReport> {
        let radius = region.observation_radius_km.unwrap_or(25.0);
        if region.scan_limit > self.config.max_scan_limit_per_region {
            return Some(RegionExecutionReport::failed(
                &region_run_id(&region.region_id, now_unix_seconds),
                &region.region_id,
                now_unix_seconds,
                unix_seconds_now(),
                format!(
                    "region scan_limit {} exceeds worker guardrail {}",
                    region.scan_limit, self.config.max_scan_limit_per_region
                ),
            ));
        }
        if radius > self.config.max_observation_radius_km {
            return Some(RegionExecutionReport::failed(
                &region_run_id(&region.region_id, now_unix_seconds),
                &region.region_id,
                now_unix_seconds,
                unix_seconds_now(),
                format!(
                    "region observation radius {:.1}km exceeds worker guardrail {:.1}km",
                    radius, self.config.max_observation_radius_km
                ),
            ));
        }
        None
    }
}

fn lease_ttl_seconds_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn lease_refresh_interval(lease_ttl_seconds: u64) -> Duration {
    Duration::from_secs((lease_ttl_seconds / 3).clamp(5, 60))
}

fn heartbeat_retention_seconds_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn store_worker_heartbeat(
    state: &AppState,
    worker_id: &str,
    started_at_unix_seconds: i64,
    status: &str,
    current_region_id: Option<&str>,
    current_phase: &str,
    last_error: Option<&str>,
) -> Result<(), AppError> {
    let heartbeat = PassiveWorkerHeartbeat {
        worker_id: worker_id.to_string(),
        started_at_unix_seconds,
        last_heartbeat_unix_seconds: unix_seconds_now(),
        status: status.to_string(),
        current_region_id: current_region_id.map(ToOwned::to_owned),
        current_phase: current_phase.to_string(),
        last_error: last_error.map(ToOwned::to_owned),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    state.store_passive_worker_heartbeat(&heartbeat)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegionFailureState {
    consecutive_failures: u32,
    blocked_until_unix_seconds: i64,
}

#[derive(Debug, Default)]
struct RegionFailureTracker {
    regions: HashMap<String, RegionFailureState>,
}

impl RegionFailureTracker {
    fn blocked_until_unix_seconds(&self, region_id: &str, now_unix_seconds: i64) -> Option<i64> {
        self.regions.get(region_id).and_then(|state| {
            (state.blocked_until_unix_seconds > now_unix_seconds)
                .then_some(state.blocked_until_unix_seconds)
        })
    }

    fn record_outcome(
        &mut self,
        report: &RegionExecutionReport,
        cooldown_base_seconds: u64,
        cooldown_cap_seconds: u64,
    ) -> Option<i64> {
        if report.log.status != crate::logging::WorkerRunStatus::Failed {
            self.regions.remove(&report.log.region_id);
            return None;
        }

        let finished_at_unix_seconds = report
            .log
            .finished_at_unix_seconds
            .unwrap_or(report.log.started_at_unix_seconds);
        let state =
            self.regions
                .entry(report.log.region_id.clone())
                .or_insert(RegionFailureState {
                    consecutive_failures: 0,
                    blocked_until_unix_seconds: finished_at_unix_seconds,
                });
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        let backoff_seconds = region_failure_backoff_seconds(
            state.consecutive_failures,
            cooldown_base_seconds,
            cooldown_cap_seconds,
        );
        state.blocked_until_unix_seconds = finished_at_unix_seconds.saturating_add(backoff_seconds);
        Some(state.blocked_until_unix_seconds)
    }
}

fn region_failure_backoff_seconds(
    consecutive_failures: u32,
    cooldown_base_seconds: u64,
    cooldown_cap_seconds: u64,
) -> i64 {
    let shift = consecutive_failures.saturating_sub(1).min(20);
    let multiplier = 1_u64.checked_shl(shift).unwrap_or(u64::MAX);
    let backoff_seconds = cooldown_base_seconds
        .max(1)
        .saturating_mul(multiplier)
        .min(cooldown_cap_seconds.max(cooldown_base_seconds.max(1)));
    i64::try_from(backoff_seconds).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorkerFeedConfig;
    use crate::execution::RegionExecutionReport;
    use sss_api::state::PassiveRegionTargetRequest;
    use sss_storage::{PassiveRegionRunLog, PassiveRegionRunStatus, SqliteStore};
    use std::path::PathBuf;

    #[test]
    fn failed_regions_receive_exponential_backoff_with_cap() {
        let mut tracker = RegionFailureTracker::default();
        let first = tracker
            .record_outcome(&failed_report("region-a", 1_000), 30, 120)
            .expect("first cooldown");
        let second = tracker
            .record_outcome(&failed_report("region-a", 1_040), 30, 120)
            .expect("second cooldown");
        let third = tracker
            .record_outcome(&failed_report("region-a", 1_120), 30, 120)
            .expect("third cooldown");
        let fourth = tracker
            .record_outcome(&failed_report("region-a", 1_240), 30, 120)
            .expect("fourth cooldown");

        assert_eq!(first, 1_030);
        assert_eq!(second, 1_100);
        assert_eq!(third, 1_240);
        assert_eq!(fourth, 1_360);
    }

    #[test]
    fn successful_region_run_clears_existing_backoff() {
        let mut tracker = RegionFailureTracker::default();
        tracker.record_outcome(&failed_report("region-a", 1_000), 30, 120);
        assert_eq!(
            tracker.blocked_until_unix_seconds("region-a", 1_010),
            Some(1_030)
        );

        assert_eq!(
            tracker.record_outcome(&successful_report("region-a", 1_020), 30, 120),
            None
        );
        assert_eq!(tracker.blocked_until_unix_seconds("region-a", 1_021), None);
    }

    #[test]
    fn region_becomes_runnable_again_after_backoff_window() {
        let mut tracker = RegionFailureTracker::default();
        tracker.record_outcome(&failed_report("region-a", 1_000), 30, 120);

        assert_eq!(
            tracker.blocked_until_unix_seconds("region-a", 1_010),
            Some(1_030)
        );
        assert_eq!(tracker.blocked_until_unix_seconds("region-a", 1_030), None);
        assert_eq!(tracker.blocked_until_unix_seconds("region-a", 1_031), None);
    }

    #[test]
    fn worker_respects_persisted_failed_region_cooldown() {
        let state = test_state();
        state
            .store_passive_region_run_log_entry(&storage_run_log(
                "run-worker-failed",
                "region-a",
                PassiveRegionRunStatus::Failed,
                Some(1_200),
                PassiveRunOrigin::Worker,
            ))
            .expect("run log stored");
        let worker = PassiveWorker::new(state, test_config());

        assert!(!worker
            .region_ready_for_run("region-a", 1_100)
            .expect("backoff lookup"));
        assert!(worker
            .region_ready_for_run("region-a", 1_201)
            .expect("backoff expired"));
    }

    #[test]
    fn worker_ignores_persisted_manual_failures_for_backoff() {
        let state = test_state();
        state
            .store_passive_region_run_log_entry(&storage_run_log(
                "run-manual-failed",
                "region-a",
                PassiveRegionRunStatus::Failed,
                Some(1_200),
                PassiveRunOrigin::Api,
            ))
            .expect("manual run log stored");
        let worker = PassiveWorker::new(state, test_config());

        assert!(worker
            .region_ready_for_run("region-a", 1_100)
            .expect("manual failures ignored"));
    }

    #[tokio::test]
    async fn run_once_respects_max_parallel_regions() {
        let state = test_state();
        state
            .upsert_passive_region_target(&test_region_request("region-a"))
            .expect("region-a stored");
        state
            .upsert_passive_region_target(&test_region_request("region-b"))
            .expect("region-b stored");

        let mut config = test_config();
        config.enable_discovery = false;
        config.enable_scan = false;
        config.dry_run = true;
        config.max_parallel_regions = 1;
        config.max_regions_per_cycle = 1;

        let worker = PassiveWorker::new(state.clone(), config.clone());
        let reports = worker.run_once().await.expect("worker run");
        assert!(reports.is_empty());

        config.max_parallel_regions = 2;
        config.max_regions_per_cycle = 2;
        let worker = PassiveWorker::new(state, config);
        let reports = worker.run_once().await.expect("worker run");
        assert!(reports.is_empty());
    }

    #[tokio::test]
    async fn run_once_limits_regions_per_cycle_even_when_more_are_due() {
        let state = test_state();
        state
            .upsert_passive_region_target(&test_region_request("region-a"))
            .expect("region-a stored");
        state
            .upsert_passive_region_target(&test_region_request("region-b"))
            .expect("region-b stored");
        state
            .upsert_passive_region_target(&test_region_request("region-c"))
            .expect("region-c stored");

        let mut config = test_config();
        config.enable_discovery = false;
        config.enable_scan = false;
        config.dry_run = false;
        config.max_parallel_regions = 2;
        config.max_regions_per_cycle = 2;

        let worker = PassiveWorker::new(state, config);
        let reports = worker.run_once().await.expect("worker run");
        assert!(reports.is_empty());
    }

    #[tokio::test]
    async fn region_guardrail_rejects_excessive_scan_limit() {
        let state = test_state();
        let mut request = test_region_request("region-a");
        request.scan_limit = Some(150);
        state
            .upsert_passive_region_target(&request)
            .expect("region stored");

        let mut config = test_config();
        config.max_scan_limit_per_region = 50;
        let worker = PassiveWorker::new(state.clone(), config);
        let error = worker.run_once().await.expect_err("guardrail failure");
        assert!(error
            .to_string()
            .contains("scan_limit 150 exceeds worker guardrail 50"));
        let run_log = state
            .passive_region_run_logs(1, Some("region-a"))
            .expect("run logs")
            .into_iter()
            .next()
            .expect("run log");
        assert_eq!(run_log.status, PassiveRegionRunStatus::Failed);
        assert!(run_log
            .source_errors
            .iter()
            .any(|error| error.contains("scan_limit 150 exceeds worker guardrail 50")));
    }

    #[test]
    fn region_complete_heartbeat_persists_degraded_summary_for_partial_runs() {
        let state = test_state();
        state
            .upsert_passive_region_target(&test_region_request("region-a"))
            .expect("region stored");
        let worker = PassiveWorker::new(state.clone(), test_config());
        let region = state
            .passive_region_targets(1, true)
            .expect("regions")
            .into_iter()
            .next()
            .expect("region target");
        let run_id = worker
            .acquire_region_lease(&region, 1_000)
            .expect("lease lookup")
            .expect("lease acquired");
        let mut report = RegionExecutionReport::failed("run-1", "region-a", 995, 1_000, "ignored");
        report.log.status = crate::logging::WorkerRunStatus::Partial;
        report.log.sources_used = vec!["Adsb".to_string()];
        report.log.source_errors = vec![
            "Weather: upstream timeout".to_string(),
            "FireSmoke: partial ingest (3 observations)".to_string(),
        ];

        worker
            .finalize_region_report(&region, &run_id, &mut report)
            .expect("report finalized");

        let heartbeat = state
            .passive_worker_heartbeats(1)
            .expect("heartbeats")
            .into_iter()
            .next()
            .expect("heartbeat");
        assert_eq!(heartbeat.status, "degraded");
        assert_eq!(heartbeat.current_phase, "region_complete");
        assert_eq!(heartbeat.current_region_id.as_deref(), Some("region-a"));
        assert_eq!(
            heartbeat.last_error.as_deref(),
            Some(
                "source health degraded: Weather: upstream timeout; FireSmoke: partial ingest (3 observations)"
            )
        );
    }

    #[test]
    fn lease_release_rejection_persists_partial_cleanup_diagnostic() {
        let state = test_state();
        state
            .upsert_passive_region_target(&test_region_request("region-a"))
            .expect("region stored");
        let worker = PassiveWorker::new(state.clone(), test_config());
        let region = state
            .passive_region_targets(1, true)
            .expect("regions")
            .into_iter()
            .next()
            .expect("region target");
        let _lease_run_id = worker
            .acquire_region_lease(&region, 1_000)
            .expect("lease lookup")
            .expect("lease acquired");
        let mut report = successful_report("region-a", 1_000);

        worker
            .finalize_region_report(&region, "wrong-run-id", &mut report)
            .expect("report finalized");

        assert_eq!(report.log.status, crate::logging::WorkerRunStatus::Partial);
        assert_eq!(report.log.reason_code(), Some("lease_release_rejected"));
        assert_eq!(
            report.log.source_errors,
            vec!["lease_release_rejected: region lease release rejected".to_string()]
        );

        let heartbeat = state
            .passive_worker_heartbeats(1)
            .expect("heartbeats")
            .into_iter()
            .next()
            .expect("heartbeat");
        assert_eq!(heartbeat.status, "degraded");
        assert_eq!(heartbeat.current_phase, "region_complete");
        assert_eq!(heartbeat.current_region_id.as_deref(), Some("region-a"));
        assert_eq!(
            heartbeat.last_error.as_deref(),
            Some("lease cleanup rejected: region lease release rejected")
        );

        let run_log = state
            .passive_region_run_logs(1, Some("region-a"))
            .expect("run logs")
            .into_iter()
            .next()
            .expect("run log");
        assert_eq!(run_log.status, PassiveRegionRunStatus::Partial);
        assert_eq!(
            run_log.source_errors,
            vec!["lease_release_rejected: region lease release rejected".to_string()]
        );
    }

    fn failed_report(region_id: &str, finished_at_unix_seconds: i64) -> RegionExecutionReport {
        RegionExecutionReport::failed(
            "run-1",
            region_id,
            finished_at_unix_seconds.saturating_sub(5),
            finished_at_unix_seconds,
            "boom",
        )
    }

    fn successful_report(region_id: &str, finished_at_unix_seconds: i64) -> RegionExecutionReport {
        let mut report = RegionExecutionReport::failed(
            "run-2",
            region_id,
            finished_at_unix_seconds.saturating_sub(5),
            finished_at_unix_seconds,
            "temporary",
        );
        report.log.status = crate::logging::WorkerRunStatus::Completed;
        report.log.source_errors.clear();
        report
    }

    fn test_state() -> AppState {
        AppState::from_storage(SqliteStore::in_memory().expect("store")).expect("state")
    }

    fn test_config() -> WorkerConfig {
        WorkerConfig {
            storage_path: PathBuf::from(":memory:"),
            worker_id: "worker-test".to_string(),
            poll_interval_seconds: 30,
            retry_interval_seconds: 30,
            max_parallel_regions: 4,
            max_regions_per_cycle: 8,
            max_runtime_seconds: 20,
            lease_ttl_seconds: 120,
            heartbeat_retention_seconds: 86_400,
            window_hours: 24,
            max_scan_limit_per_region: 100,
            max_observation_radius_km: 75.0,
            max_unfetched_feeds_per_region: 1,
            enable_discovery: true,
            enable_scan: true,
            feeds: WorkerFeedConfig {
                include_adsb: true,
                include_weather: true,
                include_fire_smoke: true,
            },
            required_feeds: WorkerFeedConfig {
                include_adsb: false,
                include_weather: true,
                include_fire_smoke: false,
            },
            dry_run: false,
        }
    }

    fn storage_run_log(
        run_id: &str,
        region_id: &str,
        status: PassiveRegionRunStatus,
        next_run_at_unix_seconds: Option<i64>,
        origin: PassiveRunOrigin,
    ) -> PassiveRegionRunLog {
        PassiveRegionRunLog {
            run_id: run_id.to_string(),
            request_id: run_id.to_string(),
            origin,
            started_at_unix_seconds: 1_000,
            finished_at_unix_seconds: 1_010,
            duration_ms: 10_000,
            status,
            region_ids: vec![region_id.to_string()],
            discovery_triggered: false,
            evaluated_region_count: 1,
            discovered_region_count: 0,
            skipped_region_count: 0,
            discovered_seed_count: 0,
            selected_seed_count: 0,
            event_count: 0,
            sources_used: Vec::new(),
            source_errors: vec!["boom".to_string()],
            next_run_at_unix_seconds,
        }
    }

    fn test_region_request(region_id: &str) -> PassiveRegionTargetRequest {
        PassiveRegionTargetRequest {
            region_id: Some(region_id.to_string()),
            name: format!("Region {region_id}"),
            south: 38.0,
            west: -9.0,
            north: 39.0,
            east: -8.0,
            site_types: None,
            timezone: Some("Europe/Lisbon".to_string()),
            country_code: Some("PT".to_string()),
            default_operator_name: Some("SSS Passive".to_string()),
            default_criticality: None,
            observation_radius_km: Some(25.0),
            discovery_cadence_seconds: Some(3_600),
            scan_limit: Some(10),
            minimum_priority: Some(0.0),
            enabled: Some(true),
        }
    }
}
