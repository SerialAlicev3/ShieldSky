//! Durable persistence for Serial Alice Sky intelligence artifacts.

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sss_core::{
    replay_manifest_for, AlertSeverity, EvidenceBundle, IntelligenceAssessment,
    IntelligenceRecipient, Observation, Prediction, RankedEvent, ReplayManifest, SpaceObject,
};
use sss_passive_scanner::{
    PassiveCoreEnvelope, PassiveEvent, PassiveObservation, PassiveRiskRecord, PassiveScanOutput,
    PassiveSiteProfile, PassiveSourceKind, PassiveThreatType, PatternSignal,
};
use sss_site_registry::{Criticality, SiteType};

#[derive(Debug, Clone)]
pub struct SqliteStore {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestResponseLog {
    pub request_id: String,
    pub endpoint: String,
    pub object_id: Option<String>,
    pub timestamp_unix_seconds: i64,
    pub request_json: String,
    pub response_json: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestBatchLog {
    pub request_id: String,
    pub source: String,
    pub timestamp_unix_seconds: i64,
    pub records_received: usize,
    pub observations_created: usize,
    pub object_ids: Vec<String>,
    pub raw_payload: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationDeliveryLog {
    pub delivery_id: String,
    pub request_id: String,
    pub object_id: String,
    pub recipient: IntelligenceRecipient,
    pub channel: String,
    pub target: String,
    pub status: String,
    pub status_code: Option<u16>,
    pub timestamp_unix_seconds: i64,
    pub attempt_number: u32,
    pub max_attempts: u32,
    pub notification_reason: String,
    pub payload_json: String,
    pub response_body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionSnapshot {
    pub snapshot_id: String,
    pub request_id: String,
    pub endpoint: String,
    pub object_id: String,
    pub generated_at_unix_seconds: i64,
    pub horizon_hours: f64,
    pub base_epoch_unix_seconds: Option<i64>,
    pub propagation_model: String,
    pub model_version: String,
    pub evidence_bundle_hash: Option<String>,
    pub predictions: Vec<Prediction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassiveSeedClassificationStatus {
    Discovered,
    Observed,
    Elevated,
    Strategic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassiveSeedStatus {
    New,
    Monitoring,
    Elevated,
    Dormant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveSeedRecord {
    pub seed_key: String,
    pub site_id: Option<String>,
    pub seed: sss_passive_scanner::OpenInfraSiteSeed,
    pub discovered_at_unix_seconds: i64,
    pub last_scanned_at_unix_seconds: Option<i64>,
    pub last_event_at_unix_seconds: Option<i64>,
    pub confidence: f64,
    pub source_count: u32,
    pub classification_status: PassiveSeedClassificationStatus,
    pub seed_status: PassiveSeedStatus,
    pub scan_priority: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveRegionTarget {
    pub region_id: String,
    pub name: String,
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
    pub site_types: Option<Vec<SiteType>>,
    pub timezone: Option<String>,
    pub country_code: Option<String>,
    pub default_operator_name: Option<String>,
    pub default_criticality: Option<Criticality>,
    pub observation_radius_km: Option<f64>,
    pub discovery_cadence_seconds: i64,
    pub scan_limit: usize,
    pub minimum_priority: f64,
    pub enabled: bool,
    pub created_at_unix_seconds: i64,
    pub updated_at_unix_seconds: i64,
    pub last_discovered_at_unix_seconds: Option<i64>,
    pub last_scheduler_run_at_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassiveRegionRunStatus {
    Completed,
    CompletedWithNoDueRegions,
    DryRun,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassiveRunOrigin {
    Api,
    Worker,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveRegionRunLog {
    pub run_id: String,
    pub request_id: String,
    pub origin: PassiveRunOrigin,
    pub started_at_unix_seconds: i64,
    pub finished_at_unix_seconds: i64,
    pub duration_ms: u64,
    pub status: PassiveRegionRunStatus,
    pub region_ids: Vec<String>,
    pub discovery_triggered: bool,
    pub evaluated_region_count: usize,
    pub discovered_region_count: usize,
    pub skipped_region_count: usize,
    pub discovered_seed_count: usize,
    pub selected_seed_count: usize,
    pub event_count: usize,
    pub sources_used: Vec<String>,
    pub source_errors: Vec<String>,
    pub next_run_at_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveRegionLease {
    pub region_id: String,
    pub worker_id: String,
    pub run_id: String,
    pub acquired_at_unix_seconds: i64,
    pub heartbeat_at_unix_seconds: i64,
    pub expires_at_unix_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveWorkerHeartbeat {
    pub worker_id: String,
    pub started_at_unix_seconds: i64,
    pub last_heartbeat_unix_seconds: i64,
    pub status: String,
    pub current_region_id: Option<String>,
    pub current_phase: String,
    pub last_error: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassiveSourceHealthSample {
    pub sample_id: String,
    pub request_id: String,
    pub region_id: Option<String>,
    pub source_kind: PassiveSourceKind,
    pub fetched: bool,
    pub observations_collected: usize,
    pub detail: String,
    pub generated_at_unix_seconds: i64,
    pub window_start_unix_seconds: i64,
    pub window_end_unix_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPassiveEvent {
    pub canonical_id: String,
    pub signature: String,
    pub site_id: String,
    pub site_name: String,
    pub threat_type: PassiveThreatType,
    pub source_kinds: Vec<PassiveSourceKind>,
    pub first_seen_at_unix_seconds: i64,
    pub last_seen_at_unix_seconds: i64,
    pub occurrence_count: u32,
    pub max_risk_score: f64,
    pub avg_confidence: f64,
    pub severity_rank: i64,
    pub summary: String,
    pub latitude: f64,
    pub longitude: f64,
    pub related_event_ids: Vec<String>,
    pub evidence_bundle_hashes: Vec<String>,
    pub replay_manifest_hashes: Vec<String>,
}

#[derive(Debug)]
pub enum StorageError {
    Sqlite(rusqlite::Error),
    Serde(serde_json::Error),
    PoisonedLock,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "sqlite error: {error}"),
            Self::Serde(error) => write!(f, "json error: {error}"),
            Self::PoisonedLock => write!(f, "sqlite store lock was poisoned"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let store = Self {
            connection: Arc::new(Mutex::new(Connection::open(path)?)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let store = Self {
            connection: Arc::new(Mutex::new(Connection::open_in_memory()?)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn store_evidence_bundle(&self, bundle: &EvidenceBundle) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO evidence_bundles
                 (bundle_hash, object_id, window_start, window_end, payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    bundle.bundle_hash,
                    bundle.object_id,
                    bundle.window_start_unix_seconds,
                    bundle.window_end_unix_seconds,
                    to_json(bundle)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn store_space_object(&self, object: &SpaceObject) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO space_objects (object_id, payload_json)
                 VALUES (?1, ?2)",
                params![object.id, to_json(object)?],
            )?;
            Ok(())
        })
    }

    pub fn space_objects(&self) -> Result<Vec<SpaceObject>, StorageError> {
        self.with_connection(|connection| {
            let mut statement =
                connection.prepare("SELECT payload_json FROM space_objects ORDER BY object_id")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn store_observation(&self, observation: &Observation) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO observations (object_id, timestamp_unix_seconds, payload_json)
                 VALUES (?1, ?2, ?3)",
                params![
                    observation.object_id,
                    observation.observed_state.epoch_unix_seconds,
                    to_json(observation)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn latest_observations(&self) -> Result<Vec<Observation>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM observations
                 ORDER BY object_id ASC, timestamp_unix_seconds ASC, id ASC",
            )?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            let mut latest_by_object = std::collections::BTreeMap::new();
            for row in rows {
                let observation = from_json::<Observation>(&row?)?;
                latest_by_object.insert(observation.object_id.clone(), observation);
            }
            Ok(latest_by_object.into_values().collect())
        })
    }

    pub fn evidence_bundle(
        &self,
        bundle_hash: &str,
    ) -> Result<Option<EvidenceBundle>, StorageError> {
        self.with_connection(|connection| {
            let payload = connection
                .query_row(
                    "SELECT payload_json FROM evidence_bundles WHERE bundle_hash = ?1",
                    [bundle_hash],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            payload.map(|json| from_json(&json)).transpose()
        })
    }

    pub fn evidence_bundle_for_manifest(
        &self,
        manifest_hash: &str,
    ) -> Result<Option<EvidenceBundle>, StorageError> {
        let Some(manifest) = self.replay_manifest(manifest_hash)? else {
            return Ok(None);
        };
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM evidence_bundles
                 WHERE object_id = ?1
                 ORDER BY window_end DESC",
            )?;
            let rows = statement.query_map([manifest.object_id], |row| row.get::<_, String>(0))?;
            for row in rows {
                let bundle = from_json::<EvidenceBundle>(&row?)?;
                if replay_manifest_for(&bundle).manifest_hash == manifest_hash {
                    return Ok(Some(bundle));
                }
            }
            Ok(None)
        })
    }

    pub fn store_replay_manifest(&self, manifest: &ReplayManifest) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO replay_manifests
                 (manifest_hash, object_id, created_at, payload_json)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    manifest.manifest_hash,
                    manifest.object_id,
                    manifest.created_at_unix_seconds,
                    to_json(manifest)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn replay_manifest(
        &self,
        manifest_hash: &str,
    ) -> Result<Option<ReplayManifest>, StorageError> {
        self.with_connection(|connection| {
            let payload = connection
                .query_row(
                    "SELECT payload_json FROM replay_manifests WHERE manifest_hash = ?1",
                    [manifest_hash],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            payload.map(|json| from_json(&json)).transpose()
        })
    }

    pub fn store_assessment(
        &self,
        object_id: &str,
        timestamp_unix_seconds: Option<i64>,
        assessment: &IntelligenceAssessment,
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO assessments (object_id, timestamp_unix_seconds, payload_json)
                 VALUES (?1, ?2, ?3)",
                params![object_id, timestamp_unix_seconds, to_json(assessment)?],
            )?;
            Ok(())
        })
    }

    pub fn assessment_history(
        &self,
        object_id: &str,
    ) -> Result<Vec<IntelligenceAssessment>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM assessments
                 WHERE object_id = ?1
                 ORDER BY timestamp_unix_seconds ASC, id ASC",
            )?;
            let rows = statement.query_map([object_id], |row| row.get::<_, String>(0))?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn store_ranked_events(&self, events: &[RankedEvent]) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for event in events {
                connection.execute(
                    "INSERT OR REPLACE INTO ranked_events
                     (event_id, object_id, risk, severity_rank, timestamp_unix_seconds, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        event.event_id,
                        event.object_id,
                        event.risk,
                        severity_rank_from_risk(event.risk),
                        event.target_epoch_unix_seconds,
                        to_json(event)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn store_passive_scan_output(
        &self,
        output: &PassiveScanOutput,
    ) -> Result<(), StorageError> {
        self.store_passive_site_profiles(&output.sites)?;
        self.store_passive_observations(&output.observations)?;
        self.store_passive_events(&output.events)?;
        self.store_passive_risk_history(&output.risk_history)?;
        self.store_pattern_signals(&output.patterns)?;
        self.store_ranked_events(
            &output
                .core_envelopes
                .iter()
                .map(|envelope| envelope.ranked_event.clone())
                .collect::<Vec<_>>(),
        )?;
        for envelope in &output.core_envelopes {
            self.store_evidence_bundle(&envelope.evidence_bundle)?;
            self.store_replay_manifest(&envelope.replay_manifest)?;
            self.store_assessment(
                &envelope.finding.object_id,
                envelope.finding.observed_at_unix_seconds,
                &envelope.assessment,
            )?;
        }
        self.store_or_merge_canonical_passive_events(&output.events, &output.core_envelopes)?;
        Ok(())
    }

    pub fn store_passive_site_profiles(
        &self,
        sites: &[PassiveSiteProfile],
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for site in sites {
                connection.execute(
                    "INSERT OR REPLACE INTO passive_site_profiles
                     (site_id, site_name, site_type, criticality, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        site.site.site_id.to_string(),
                        site.site.name,
                        format!("{:?}", site.site.site_type),
                        format!("{:?}", site.site.criticality),
                        to_json(site)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn store_passive_seed_records(
        &self,
        records: &[PassiveSeedRecord],
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for record in records {
                connection.execute(
                    "INSERT OR REPLACE INTO passive_seed_registry
                     (seed_key, site_id, discovered_at_unix_seconds, last_scanned_at_unix_seconds,
                      last_event_at_unix_seconds, confidence, source_count,
                      classification_status, seed_status, scan_priority, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        record.seed_key,
                        record.site_id,
                        record.discovered_at_unix_seconds,
                        record.last_scanned_at_unix_seconds,
                        record.last_event_at_unix_seconds,
                        record.confidence,
                        i64::from(record.source_count),
                        format!("{:?}", record.classification_status),
                        format!("{:?}", record.seed_status),
                        record.scan_priority,
                        to_json(record)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn passive_seed_records(
        &self,
        limit: usize,
    ) -> Result<Vec<PassiveSeedRecord>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_seed_registry
                 ORDER BY scan_priority DESC, COALESCE(last_scanned_at_unix_seconds, 0) DESC,
                          discovered_at_unix_seconds DESC, seed_key ASC
                 LIMIT ?1",
            )?;
            let rows = statement.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                row.get::<_, String>(0)
            })?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn passive_seed_record(
        &self,
        seed_key: &str,
    ) -> Result<Option<PassiveSeedRecord>, StorageError> {
        self.with_connection(|connection| {
            let payload = connection
                .query_row(
                    "SELECT payload_json FROM passive_seed_registry WHERE seed_key = ?1",
                    [seed_key],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            payload.map(|json| from_json(&json)).transpose()
        })
    }

    pub fn store_passive_region_target(
        &self,
        target: &PassiveRegionTarget,
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO passive_region_targets
                 (region_id, name, enabled, updated_at_unix_seconds,
                  last_discovered_at_unix_seconds, last_scheduler_run_at_unix_seconds,
                  payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    target.region_id,
                    target.name,
                    target.enabled,
                    target.updated_at_unix_seconds,
                    target.last_discovered_at_unix_seconds,
                    target.last_scheduler_run_at_unix_seconds,
                    to_json(target)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn passive_region_targets(
        &self,
        limit: usize,
        enabled_only: bool,
    ) -> Result<Vec<PassiveRegionTarget>, StorageError> {
        self.with_connection(|connection| {
            let sql = if enabled_only {
                "SELECT payload_json FROM passive_region_targets
                 WHERE enabled = 1
                 ORDER BY COALESCE(last_discovered_at_unix_seconds, 0) ASC,
                          updated_at_unix_seconds DESC, region_id ASC
                 LIMIT ?1"
            } else {
                "SELECT payload_json FROM passive_region_targets
                 ORDER BY enabled DESC, updated_at_unix_seconds DESC, region_id ASC
                 LIMIT ?1"
            };
            let mut statement = connection.prepare(sql)?;
            let rows = statement.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                row.get::<_, String>(0)
            })?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn passive_region_target(
        &self,
        region_id: &str,
    ) -> Result<Option<PassiveRegionTarget>, StorageError> {
        self.with_connection(|connection| {
            let payload = connection
                .query_row(
                    "SELECT payload_json FROM passive_region_targets WHERE region_id = ?1",
                    [region_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            payload.map(|json| from_json(&json)).transpose()
        })
    }

    pub fn store_passive_region_run_log(
        &self,
        log: &PassiveRegionRunLog,
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO passive_region_run_logs
                 (run_id, request_id, origin, started_at_unix_seconds, finished_at_unix_seconds,
                  status, next_run_at_unix_seconds, region_ids_json, payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    log.run_id,
                    log.request_id,
                    format!("{:?}", log.origin),
                    log.started_at_unix_seconds,
                    log.finished_at_unix_seconds,
                    format!("{:?}", log.status),
                    log.next_run_at_unix_seconds,
                    to_json(&log.region_ids)?,
                    to_json(log)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn passive_region_run_logs(
        &self,
        limit: usize,
        region_id: Option<&str>,
    ) -> Result<Vec<PassiveRegionRunLog>, StorageError> {
        self.with_connection(|connection| {
            let limit = i64::try_from(limit).unwrap_or(i64::MAX);
            let payloads = if let Some(region_id) = region_id {
                let mut statement = connection.prepare(
                    "SELECT payload_json FROM passive_region_run_logs
                     WHERE region_ids_json LIKE ?1
                     ORDER BY finished_at_unix_seconds DESC, run_id DESC
                     LIMIT ?2",
                )?;
                let pattern = format!("%\"{region_id}\"%");
                let rows =
                    statement.query_map(params![pattern, limit], |row| row.get::<_, String>(0))?;
                rows.collect::<Result<Vec<_>, _>>()?
            } else {
                let mut statement = connection.prepare(
                    "SELECT payload_json FROM passive_region_run_logs
                     ORDER BY finished_at_unix_seconds DESC, run_id DESC
                     LIMIT ?1",
                )?;
                let rows = statement.query_map([limit], |row| row.get::<_, String>(0))?;
                rows.collect::<Result<Vec<_>, _>>()?
            };
            payloads.into_iter().map(|json| from_json(&json)).collect()
        })
    }

    pub fn passive_region_run_log(
        &self,
        run_id: &str,
    ) -> Result<Option<PassiveRegionRunLog>, StorageError> {
        self.with_connection(|connection| {
            let payload = connection
                .query_row(
                    "SELECT payload_json FROM passive_region_run_logs WHERE run_id = ?1",
                    [run_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            payload
                .map(|payload| serde_json::from_str(&payload).map_err(StorageError::from))
                .transpose()
        })
    }

    pub fn acquire_passive_region_lease(
        &self,
        region_id: &str,
        worker_id: &str,
        run_id: &str,
        now_unix_seconds: i64,
        ttl_seconds: i64,
    ) -> Result<Option<PassiveRegionLease>, StorageError> {
        let ttl_seconds = ttl_seconds.max(1);
        let lease = PassiveRegionLease {
            region_id: region_id.to_string(),
            worker_id: worker_id.to_string(),
            run_id: run_id.to_string(),
            acquired_at_unix_seconds: now_unix_seconds,
            heartbeat_at_unix_seconds: now_unix_seconds,
            expires_at_unix_seconds: now_unix_seconds.saturating_add(ttl_seconds),
        };

        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO passive_region_leases
                 (region_id, worker_id, run_id, acquired_at_unix_seconds,
                  heartbeat_at_unix_seconds, expires_at_unix_seconds, payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(region_id) DO UPDATE SET
                    worker_id = excluded.worker_id,
                    run_id = excluded.run_id,
                    acquired_at_unix_seconds = excluded.acquired_at_unix_seconds,
                    heartbeat_at_unix_seconds = excluded.heartbeat_at_unix_seconds,
                    expires_at_unix_seconds = excluded.expires_at_unix_seconds,
                    payload_json = excluded.payload_json
                 WHERE passive_region_leases.expires_at_unix_seconds <= ?8
                    OR passive_region_leases.worker_id = ?2",
                params![
                    lease.region_id,
                    lease.worker_id,
                    lease.run_id,
                    lease.acquired_at_unix_seconds,
                    lease.heartbeat_at_unix_seconds,
                    lease.expires_at_unix_seconds,
                    to_json(&lease)?,
                    now_unix_seconds,
                ],
            )?;

            let payload = connection
                .query_row(
                    "SELECT payload_json FROM passive_region_leases WHERE region_id = ?1",
                    [region_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let Some(payload) = payload else {
                return Ok(None);
            };
            let stored = from_json::<PassiveRegionLease>(&payload)?;
            if stored.worker_id == worker_id && stored.run_id == run_id {
                Ok(Some(stored))
            } else {
                Ok(None)
            }
        })
    }

    pub fn refresh_passive_region_lease(
        &self,
        region_id: &str,
        worker_id: &str,
        run_id: &str,
        now_unix_seconds: i64,
        ttl_seconds: i64,
    ) -> Result<bool, StorageError> {
        let ttl_seconds = ttl_seconds.max(1);
        self.with_connection(|connection| {
            let existing = connection
                .query_row(
                    "SELECT payload_json FROM passive_region_leases
                     WHERE region_id = ?1 AND worker_id = ?2 AND run_id = ?3",
                    params![region_id, worker_id, run_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let Some(payload) = existing else {
                return Ok(false);
            };
            let mut lease = from_json::<PassiveRegionLease>(&payload)?;
            lease.heartbeat_at_unix_seconds = now_unix_seconds;
            lease.expires_at_unix_seconds = now_unix_seconds.saturating_add(ttl_seconds);
            let updated = connection.execute(
                "UPDATE passive_region_leases
                 SET heartbeat_at_unix_seconds = ?3,
                     expires_at_unix_seconds = ?4,
                     payload_json = ?5
                 WHERE region_id = ?1 AND worker_id = ?2 AND run_id = ?6",
                params![
                    region_id,
                    worker_id,
                    lease.heartbeat_at_unix_seconds,
                    lease.expires_at_unix_seconds,
                    to_json(&lease)?,
                    run_id,
                ],
            )?;
            Ok(updated > 0)
        })
    }

    pub fn release_passive_region_lease(
        &self,
        region_id: &str,
        worker_id: &str,
        run_id: &str,
    ) -> Result<bool, StorageError> {
        self.with_connection(|connection| {
            let deleted = connection.execute(
                "DELETE FROM passive_region_leases
                 WHERE region_id = ?1 AND worker_id = ?2 AND run_id = ?3",
                params![region_id, worker_id, run_id],
            )?;
            Ok(deleted > 0)
        })
    }

    pub fn passive_region_leases(
        &self,
        limit: usize,
    ) -> Result<Vec<PassiveRegionLease>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_region_leases
                 ORDER BY expires_at_unix_seconds DESC, region_id ASC
                 LIMIT ?1",
            )?;
            let rows = statement.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                row.get::<_, String>(0)
            })?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn purge_expired_passive_region_leases(
        &self,
        now_unix_seconds: i64,
    ) -> Result<usize, StorageError> {
        self.with_connection(|connection| {
            connection
                .execute(
                    "DELETE FROM passive_region_leases WHERE expires_at_unix_seconds <= ?1",
                    [now_unix_seconds],
                )
                .map_err(StorageError::from)
        })
    }

    pub fn store_passive_worker_heartbeat(
        &self,
        heartbeat: &PassiveWorkerHeartbeat,
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO passive_worker_heartbeats
                 (worker_id, started_at_unix_seconds, last_heartbeat_unix_seconds,
                  status, current_region_id, current_phase, payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    heartbeat.worker_id,
                    heartbeat.started_at_unix_seconds,
                    heartbeat.last_heartbeat_unix_seconds,
                    heartbeat.status,
                    heartbeat.current_region_id,
                    heartbeat.current_phase,
                    to_json(heartbeat)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn passive_worker_heartbeats(
        &self,
        limit: usize,
    ) -> Result<Vec<PassiveWorkerHeartbeat>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_worker_heartbeats
                 ORDER BY last_heartbeat_unix_seconds DESC, worker_id ASC
                 LIMIT ?1",
            )?;
            let rows = statement.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                row.get::<_, String>(0)
            })?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn stale_passive_worker_heartbeats(
        &self,
        limit: usize,
        stale_before_unix_seconds: i64,
    ) -> Result<Vec<PassiveWorkerHeartbeat>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_worker_heartbeats
                 WHERE last_heartbeat_unix_seconds < ?1
                 ORDER BY last_heartbeat_unix_seconds ASC, worker_id ASC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(
                params![
                    stale_before_unix_seconds,
                    i64::try_from(limit).unwrap_or(i64::MAX)
                ],
                |row| row.get::<_, String>(0),
            )?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn count_stale_passive_worker_heartbeats(
        &self,
        stale_before_unix_seconds: i64,
    ) -> Result<usize, StorageError> {
        self.with_connection(|connection| {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM passive_worker_heartbeats
                     WHERE last_heartbeat_unix_seconds < ?1",
                    [stale_before_unix_seconds],
                    |row| row.get::<_, i64>(0),
                )
                .map(|count| usize::try_from(count).unwrap_or(usize::MAX))
                .map_err(StorageError::from)
        })
    }

    pub fn purge_stale_passive_worker_heartbeats(
        &self,
        stale_before_unix_seconds: i64,
    ) -> Result<usize, StorageError> {
        self.with_connection(|connection| {
            connection
                .execute(
                    "DELETE FROM passive_worker_heartbeats
                     WHERE last_heartbeat_unix_seconds < ?1",
                    [stale_before_unix_seconds],
                )
                .map_err(StorageError::from)
        })
    }

    pub fn store_passive_source_health_samples(
        &self,
        samples: &[PassiveSourceHealthSample],
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for sample in samples {
                connection.execute(
                    "INSERT OR REPLACE INTO passive_source_health_samples
                     (sample_id, request_id, region_id, source_kind, fetched,
                      generated_at_unix_seconds, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        sample.sample_id,
                        sample.request_id,
                        sample.region_id,
                        format!("{:?}", sample.source_kind),
                        sample.fetched,
                        sample.generated_at_unix_seconds,
                        to_json(sample)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn passive_source_health_samples(
        &self,
        limit: usize,
        source_kind: Option<PassiveSourceKind>,
        region_id: Option<&str>,
    ) -> Result<Vec<PassiveSourceHealthSample>, StorageError> {
        self.with_connection(|connection| {
            let limit = i64::try_from(limit).unwrap_or(i64::MAX);
            let payloads = match (source_kind, region_id) {
                (Some(source_kind), Some(region_id)) => {
                    let mut statement = connection.prepare(
                        "SELECT payload_json FROM passive_source_health_samples
                         WHERE source_kind = ?1 AND region_id = ?2
                         ORDER BY generated_at_unix_seconds DESC, sample_id DESC
                         LIMIT ?3",
                    )?;
                    let rows = statement.query_map(
                        params![format!("{:?}", source_kind), region_id, limit],
                        |row| row.get::<_, String>(0),
                    )?;
                    rows.collect::<Result<Vec<_>, _>>()?
                }
                (Some(source_kind), None) => {
                    let mut statement = connection.prepare(
                        "SELECT payload_json FROM passive_source_health_samples
                         WHERE source_kind = ?1
                         ORDER BY generated_at_unix_seconds DESC, sample_id DESC
                         LIMIT ?2",
                    )?;
                    let rows = statement
                        .query_map(params![format!("{:?}", source_kind), limit], |row| {
                            row.get::<_, String>(0)
                        })?;
                    rows.collect::<Result<Vec<_>, _>>()?
                }
                (None, Some(region_id)) => {
                    let mut statement = connection.prepare(
                        "SELECT payload_json FROM passive_source_health_samples
                         WHERE region_id = ?1
                         ORDER BY generated_at_unix_seconds DESC, sample_id DESC
                         LIMIT ?2",
                    )?;
                    let rows = statement
                        .query_map(params![region_id, limit], |row| row.get::<_, String>(0))?;
                    rows.collect::<Result<Vec<_>, _>>()?
                }
                (None, None) => {
                    let mut statement = connection.prepare(
                        "SELECT payload_json FROM passive_source_health_samples
                         ORDER BY generated_at_unix_seconds DESC, sample_id DESC
                         LIMIT ?1",
                    )?;
                    let rows = statement.query_map([limit], |row| row.get::<_, String>(0))?;
                    rows.collect::<Result<Vec<_>, _>>()?
                }
            };
            payloads.into_iter().map(|json| from_json(&json)).collect()
        })
    }

    pub fn purge_passive_source_health_samples(
        &self,
        older_than_unix_seconds: i64,
        source_kind: Option<PassiveSourceKind>,
        region_id: Option<&str>,
    ) -> Result<usize, StorageError> {
        self.with_connection(|connection| {
            match (source_kind, region_id) {
                (Some(source_kind), Some(region_id)) => connection.execute(
                    "DELETE FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1
                       AND source_kind = ?2
                       AND region_id = ?3",
                    params![
                        older_than_unix_seconds,
                        format!("{:?}", source_kind),
                        region_id
                    ],
                ),
                (Some(source_kind), None) => connection.execute(
                    "DELETE FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1
                       AND source_kind = ?2",
                    params![older_than_unix_seconds, format!("{:?}", source_kind)],
                ),
                (None, Some(region_id)) => connection.execute(
                    "DELETE FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1
                       AND region_id = ?2",
                    params![older_than_unix_seconds, region_id],
                ),
                (None, None) => connection.execute(
                    "DELETE FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1",
                    [older_than_unix_seconds],
                ),
            }
            .map_err(StorageError::from)
        })
    }

    pub fn count_passive_source_health_samples_before(
        &self,
        older_than_unix_seconds: i64,
        source_kind: Option<PassiveSourceKind>,
        region_id: Option<&str>,
    ) -> Result<usize, StorageError> {
        self.with_connection(|connection| {
            let count = match (source_kind, region_id) {
                (Some(source_kind), Some(region_id)) => connection.query_row(
                    "SELECT COUNT(*) FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1
                       AND source_kind = ?2
                       AND region_id = ?3",
                    params![
                        older_than_unix_seconds,
                        format!("{:?}", source_kind),
                        region_id
                    ],
                    |row| row.get::<_, i64>(0),
                ),
                (Some(source_kind), None) => connection.query_row(
                    "SELECT COUNT(*) FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1
                       AND source_kind = ?2",
                    params![older_than_unix_seconds, format!("{:?}", source_kind)],
                    |row| row.get::<_, i64>(0),
                ),
                (None, Some(region_id)) => connection.query_row(
                    "SELECT COUNT(*) FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1
                       AND region_id = ?2",
                    params![older_than_unix_seconds, region_id],
                    |row| row.get::<_, i64>(0),
                ),
                (None, None) => connection.query_row(
                    "SELECT COUNT(*) FROM passive_source_health_samples
                     WHERE generated_at_unix_seconds < ?1",
                    [older_than_unix_seconds],
                    |row| row.get::<_, i64>(0),
                ),
            }?;
            Ok(usize::try_from(count).unwrap_or(usize::MAX))
        })
    }

    pub fn canonical_passive_events(
        &self,
        limit: usize,
        site_id: Option<&str>,
    ) -> Result<Vec<CanonicalPassiveEvent>, StorageError> {
        self.with_connection(|connection| {
            let limit = i64::try_from(limit).unwrap_or(i64::MAX);
            let payloads = if let Some(site_id) = site_id {
                let mut statement = connection.prepare(
                    "SELECT payload_json FROM passive_canonical_events
                     WHERE site_id = ?1
                     ORDER BY max_risk_score DESC, last_seen_at_unix_seconds DESC
                     LIMIT ?2",
                )?;
                let rows =
                    statement.query_map(params![site_id, limit], |row| row.get::<_, String>(0))?;
                rows.collect::<Result<Vec<_>, _>>()?
            } else {
                let mut statement = connection.prepare(
                    "SELECT payload_json FROM passive_canonical_events
                     ORDER BY max_risk_score DESC, last_seen_at_unix_seconds DESC
                     LIMIT ?1",
                )?;
                let rows = statement.query_map([limit], |row| row.get::<_, String>(0))?;
                rows.collect::<Result<Vec<_>, _>>()?
            };
            payloads.into_iter().map(|json| from_json(&json)).collect()
        })
    }

    pub fn canonical_passive_event(
        &self,
        canonical_id: &str,
    ) -> Result<Option<CanonicalPassiveEvent>, StorageError> {
        self.with_connection(|connection| {
            let payload = connection
                .query_row(
                    "SELECT payload_json FROM passive_canonical_events WHERE canonical_id = ?1",
                    [canonical_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            payload.map(|json| from_json(&json)).transpose()
        })
    }

    fn store_or_merge_canonical_passive_events(
        &self,
        events: &[PassiveEvent],
        core_envelopes: &[PassiveCoreEnvelope],
    ) -> Result<(), StorageError> {
        for event in events {
            let canonical_id = canonical_event_signature(event);
            let existing = self.canonical_passive_event(&canonical_id)?;
            let next = merge_canonical_passive_event(existing, event, core_envelopes);
            self.with_connection(|connection| {
                connection.execute(
                    "INSERT OR REPLACE INTO passive_canonical_events
                     (canonical_id, site_id, threat_type, last_seen_at_unix_seconds,
                      max_risk_score, severity_rank, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        next.canonical_id,
                        next.site_id,
                        format!("{:?}", next.threat_type),
                        next.last_seen_at_unix_seconds,
                        next.max_risk_score,
                        next.severity_rank,
                        to_json(&next)?,
                    ],
                )?;
                Ok(())
            })?;
        }
        Ok(())
    }

    pub fn passive_site_profiles(&self) -> Result<Vec<PassiveSiteProfile>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_site_profiles
                 ORDER BY site_name ASC, site_id ASC",
            )?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn store_passive_observations(
        &self,
        observations: &[PassiveObservation],
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for observation in observations {
                connection.execute(
                    "INSERT OR REPLACE INTO passive_observations
                     (observation_id, source_kind, observed_at_unix_seconds, latitude, longitude,
                      altitude_m, severity_hint, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        observation.observation_id.to_string(),
                        format!("{:?}", observation.source_kind),
                        observation.observed_at_unix_seconds,
                        observation.latitude,
                        observation.longitude,
                        observation.altitude_m,
                        observation.severity_hint,
                        to_json(observation)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn passive_observations(
        &self,
        limit: usize,
        source_kind: Option<PassiveSourceKind>,
    ) -> Result<Vec<PassiveObservation>, StorageError> {
        self.with_connection(|connection| {
            let limit = i64::try_from(limit).unwrap_or(i64::MAX);
            let mut observations = Vec::new();
            if let Some(source_kind) = source_kind {
                let mut statement = connection.prepare(
                    "SELECT payload_json FROM passive_observations
                     WHERE source_kind = ?1
                     ORDER BY observed_at_unix_seconds DESC, observation_id DESC
                     LIMIT ?2",
                )?;
                let rows = statement
                    .query_map(params![format!("{:?}", source_kind), limit], |row| {
                        row.get::<_, String>(0)
                    })?;
                for row in rows {
                    observations.push(from_json(&row?)?);
                }
            } else {
                let mut statement = connection.prepare(
                    "SELECT payload_json FROM passive_observations
                     ORDER BY observed_at_unix_seconds DESC, observation_id DESC
                     LIMIT ?1",
                )?;
                let rows = statement.query_map(params![limit], |row| row.get::<_, String>(0))?;
                for row in rows {
                    observations.push(from_json(&row?)?);
                }
            }
            Ok(observations)
        })
    }

    pub fn store_passive_events(&self, events: &[PassiveEvent]) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for event in events {
                connection.execute(
                    "INSERT OR REPLACE INTO passive_events
                     (event_id, site_id, threat_type, source_kind, risk_score,
                      observed_at_unix_seconds, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        event.event_id.to_string(),
                        event.site_id.to_string(),
                        format!("{:?}", event.threat_type),
                        format!("{:?}", event.source_kind),
                        event.risk_score,
                        event.observed_at_unix_seconds,
                        to_json(event)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn passive_events_for_site(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PassiveEvent>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_events
                 WHERE site_id = ?1
                 ORDER BY risk_score DESC, observed_at_unix_seconds DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(
                params![site_id, i64::try_from(limit).unwrap_or(i64::MAX)],
                |row| row.get::<_, String>(0),
            )?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn store_passive_risk_history(
        &self,
        risk_history: &[PassiveRiskRecord],
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for record in risk_history {
                connection.execute(
                    "INSERT INTO passive_risk_history
                     (site_id, window_start_unix_seconds, window_end_unix_seconds,
                      cumulative_risk, peak_risk, event_count, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        record.site_id.to_string(),
                        record.window_start_unix_seconds,
                        record.window_end_unix_seconds,
                        record.cumulative_risk,
                        record.peak_risk,
                        i64::try_from(record.event_count).unwrap_or(i64::MAX),
                        to_json(record)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn passive_risk_history(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PassiveRiskRecord>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_risk_history
                 WHERE site_id = ?1
                 ORDER BY window_end_unix_seconds DESC, id DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(
                params![site_id, i64::try_from(limit).unwrap_or(i64::MAX)],
                |row| row.get::<_, String>(0),
            )?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn store_pattern_signals(&self, patterns: &[PatternSignal]) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            for pattern in patterns {
                connection.execute(
                    "INSERT INTO passive_pattern_signals
                     (site_id, threat_type, recurring_events, average_risk, payload_json)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        pattern.site_id.to_string(),
                        format!("{:?}", pattern.threat_type),
                        i64::try_from(pattern.recurring_events).unwrap_or(i64::MAX),
                        pattern.average_risk,
                        to_json(pattern)?,
                    ],
                )?;
            }
            Ok(())
        })
    }

    pub fn passive_patterns(
        &self,
        site_id: &str,
        limit: usize,
    ) -> Result<Vec<PatternSignal>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM passive_pattern_signals
                 WHERE site_id = ?1
                 ORDER BY average_risk DESC, recurring_events DESC, id DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(
                params![site_id, i64::try_from(limit).unwrap_or(i64::MAX)],
                |row| row.get::<_, String>(0),
            )?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn top_ranked_events(&self, limit: usize) -> Result<Vec<RankedEvent>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM ranked_events
                 ORDER BY risk DESC, severity_rank DESC, timestamp_unix_seconds DESC
                 LIMIT ?1",
            )?;
            let rows = statement.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                row.get::<_, String>(0)
            })?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn ranked_event(&self, event_id: &str) -> Result<Option<RankedEvent>, StorageError> {
        self.with_connection(|connection| {
            let payload = connection
                .query_row(
                    "SELECT payload_json FROM ranked_events WHERE event_id = ?1",
                    [event_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            payload.map(|json| from_json(&json)).transpose()
        })
    }

    pub fn log_exchange(&self, log: &RequestResponseLog) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO request_response_logs
                 (request_id, endpoint, object_id, timestamp_unix_seconds, request_json, response_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    log.request_id,
                    log.endpoint,
                    log.object_id,
                    log.timestamp_unix_seconds,
                    log.request_json,
                    log.response_json,
                ],
            )?;
            Ok(())
        })
    }

    pub fn log_ingest_batch(&self, log: &IngestBatchLog) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO ingest_batches
                 (request_id, source, timestamp_unix_seconds, records_received,
                  observations_created, object_ids_json, raw_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    log.request_id,
                    log.source,
                    log.timestamp_unix_seconds,
                    i64::try_from(log.records_received).unwrap_or(i64::MAX),
                    i64::try_from(log.observations_created).unwrap_or(i64::MAX),
                    to_json(&log.object_ids)?,
                    log.raw_payload,
                ],
            )?;
            Ok(())
        })
    }

    pub fn log_notification_delivery(
        &self,
        log: &NotificationDeliveryLog,
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO notification_deliveries
                 (delivery_id, request_id, object_id, recipient_json, channel, target,
                  status, status_code, timestamp_unix_seconds, attempt_number, max_attempts,
                  notification_reason, payload_json, response_body)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    log.delivery_id,
                    log.request_id,
                    log.object_id,
                    to_json(&log.recipient)?,
                    log.channel,
                    log.target,
                    log.status,
                    log.status_code.map(i64::from),
                    log.timestamp_unix_seconds,
                    i64::from(log.attempt_number),
                    i64::from(log.max_attempts),
                    log.notification_reason,
                    log.payload_json,
                    log.response_body,
                ],
            )?;
            Ok(())
        })
    }

    pub fn latest_ingest_batches(&self, limit: usize) -> Result<Vec<IngestBatchLog>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT request_id, source, timestamp_unix_seconds, records_received,
                        observations_created, object_ids_json, raw_payload
                 FROM ingest_batches
                 ORDER BY timestamp_unix_seconds DESC, id DESC
                 LIMIT ?1",
            )?;
            let rows = statement.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?;
            rows.map(|row| {
                let (
                    request_id,
                    source,
                    timestamp_unix_seconds,
                    records_received,
                    observations_created,
                    object_ids_json,
                    raw_payload,
                ) = row.map_err(StorageError::from)?;
                Ok(IngestBatchLog {
                    request_id,
                    source,
                    timestamp_unix_seconds,
                    records_received: usize::try_from(records_received).unwrap_or(usize::MAX),
                    observations_created: usize::try_from(observations_created)
                        .unwrap_or(usize::MAX),
                    object_ids: from_json(&object_ids_json)?,
                    raw_payload,
                })
            })
            .collect()
        })
    }

    pub fn latest_ingest_batch_for_source(
        &self,
        source: &str,
    ) -> Result<Option<IngestBatchLog>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT request_id, source, timestamp_unix_seconds, records_received,
                        observations_created, object_ids_json, raw_payload
                 FROM ingest_batches
                 WHERE source = ?1
                 ORDER BY timestamp_unix_seconds DESC, id DESC
                 LIMIT 1",
            )?;
            let row = statement
                .query_row([source], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                })
                .optional()?;
            row.map(
                |(
                    request_id,
                    source,
                    timestamp_unix_seconds,
                    records_received,
                    observations_created,
                    object_ids_json,
                    raw_payload,
                )| {
                    Ok(IngestBatchLog {
                        request_id,
                        source,
                        timestamp_unix_seconds,
                        records_received: usize::try_from(records_received).unwrap_or(usize::MAX),
                        observations_created: usize::try_from(observations_created)
                            .unwrap_or(usize::MAX),
                        object_ids: from_json(&object_ids_json)?,
                        raw_payload,
                    })
                },
            )
            .transpose()
        })
    }

    pub fn notification_deliveries(
        &self,
        limit: usize,
        object_id: Option<&str>,
    ) -> Result<Vec<NotificationDeliveryLog>, StorageError> {
        self.with_connection(|connection| {
            let limit_value = i64::try_from(limit).unwrap_or(i64::MAX);
            if let Some(object_id) = object_id {
                let mut statement = connection.prepare(
                    "SELECT delivery_id, request_id, object_id, recipient_json, channel, target,
                            status, status_code, timestamp_unix_seconds, attempt_number,
                            max_attempts, notification_reason, payload_json, response_body
                     FROM notification_deliveries
                     WHERE object_id = ?1
                     ORDER BY timestamp_unix_seconds DESC, id DESC
                     LIMIT ?2",
                )?;
                let rows = statement.query_map(params![object_id, limit_value], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<i64>>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, i64>(9)?,
                        row.get::<_, i64>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, String>(12)?,
                        row.get::<_, Option<String>>(13)?,
                    ))
                })?;
                rows.map(|row| notification_delivery_from_row(row.map_err(StorageError::from)?))
                    .collect()
            } else {
                let mut statement = connection.prepare(
                    "SELECT delivery_id, request_id, object_id, recipient_json, channel, target,
                            status, status_code, timestamp_unix_seconds, attempt_number,
                            max_attempts, notification_reason, payload_json, response_body
                     FROM notification_deliveries
                     ORDER BY timestamp_unix_seconds DESC, id DESC
                     LIMIT ?1",
                )?;
                let rows = statement.query_map([limit_value], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<i64>>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, i64>(9)?,
                        row.get::<_, i64>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, String>(12)?,
                        row.get::<_, Option<String>>(13)?,
                    ))
                })?;
                rows.map(|row| notification_delivery_from_row(row.map_err(StorageError::from)?))
                    .collect()
            }
        })
    }

    pub fn store_prediction_snapshot(
        &self,
        snapshot: &PredictionSnapshot,
    ) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT OR REPLACE INTO prediction_snapshots
                 (snapshot_id, request_id, endpoint, object_id, generated_at_unix_seconds,
                  horizon_hours, base_epoch_unix_seconds, propagation_model, model_version,
                  evidence_bundle_hash, payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    snapshot.snapshot_id,
                    snapshot.request_id,
                    snapshot.endpoint,
                    snapshot.object_id,
                    snapshot.generated_at_unix_seconds,
                    snapshot.horizon_hours,
                    snapshot.base_epoch_unix_seconds,
                    snapshot.propagation_model,
                    snapshot.model_version,
                    snapshot.evidence_bundle_hash,
                    to_json(snapshot)?,
                ],
            )?;
            Ok(())
        })
    }

    pub fn prediction_snapshots(
        &self,
        object_id: &str,
        limit: usize,
    ) -> Result<Vec<PredictionSnapshot>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT payload_json FROM prediction_snapshots
                 WHERE object_id = ?1
                 ORDER BY generated_at_unix_seconds DESC, id DESC
                 LIMIT ?2",
            )?;
            let rows = statement.query_map(
                params![object_id, i64::try_from(limit).unwrap_or(i64::MAX)],
                |row| row.get::<_, String>(0),
            )?;
            rows.map(|row| {
                row.map_err(StorageError::from)
                    .and_then(|json| from_json(&json))
            })
            .collect()
        })
    }

    pub fn request_history(
        &self,
        object_id: &str,
    ) -> Result<Vec<RequestResponseLog>, StorageError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT request_id, endpoint, object_id, timestamp_unix_seconds, request_json, response_json
                 FROM request_response_logs
                 WHERE object_id = ?1
                 ORDER BY timestamp_unix_seconds ASC, id ASC",
            )?;
            let rows = statement.query_map([object_id], |row| {
                Ok(RequestResponseLog {
                    request_id: row.get(0)?,
                    endpoint: row.get(1)?,
                    object_id: row.get(2)?,
                    timestamp_unix_seconds: row.get(3)?,
                    request_json: row.get(4)?,
                    response_json: row.get(5)?,
                })
            })?;
            rows.map(|row| row.map_err(StorageError::from)).collect()
        })
    }

    pub fn recent_request_logs(
        &self,
        limit: usize,
        endpoint: Option<&str>,
    ) -> Result<Vec<RequestResponseLog>, StorageError> {
        let limit = i64::try_from(limit.max(1)).unwrap_or(i64::MAX);
        self.with_connection(|connection| {
            if let Some(endpoint) = endpoint {
                let mut statement = connection.prepare(
                    "SELECT request_id, endpoint, object_id, timestamp_unix_seconds, request_json, response_json
                     FROM request_response_logs
                     WHERE endpoint = ?1
                     ORDER BY timestamp_unix_seconds DESC, id DESC
                     LIMIT ?2",
                )?;
                let rows = statement.query_map(rusqlite::params![endpoint, limit], |row| {
                    Ok(RequestResponseLog {
                        request_id: row.get(0)?,
                        endpoint: row.get(1)?,
                        object_id: row.get(2)?,
                        timestamp_unix_seconds: row.get(3)?,
                        request_json: row.get(4)?,
                        response_json: row.get(5)?,
                    })
                })?;
                rows.map(|row| row.map_err(StorageError::from)).collect()
            } else {
                let mut statement = connection.prepare(
                    "SELECT request_id, endpoint, object_id, timestamp_unix_seconds, request_json, response_json
                     FROM request_response_logs
                     ORDER BY timestamp_unix_seconds DESC, id DESC
                     LIMIT ?1",
                )?;
                let rows = statement.query_map(rusqlite::params![limit], |row| {
                    Ok(RequestResponseLog {
                        request_id: row.get(0)?,
                        endpoint: row.get(1)?,
                        object_id: row.get(2)?,
                        timestamp_unix_seconds: row.get(3)?,
                        request_json: row.get(4)?,
                        response_json: row.get(5)?,
                    })
                })?;
                rows.map(|row| row.map_err(StorageError::from)).collect()
            }
        })
    }

    #[allow(clippy::too_many_lines)]
    fn init(&self) -> Result<(), StorageError> {
        self.with_connection(|connection| {
            connection.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS evidence_bundles (
                    bundle_hash TEXT PRIMARY KEY,
                    object_id TEXT NOT NULL,
                    window_start INTEGER,
                    window_end INTEGER,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_evidence_object_time
                    ON evidence_bundles(object_id, window_end);

                CREATE TABLE IF NOT EXISTS space_objects (
                    object_id TEXT PRIMARY KEY,
                    payload_json TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS observations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    object_id TEXT NOT NULL,
                    timestamp_unix_seconds INTEGER NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_observations_object_time
                    ON observations(object_id, timestamp_unix_seconds);

                CREATE TABLE IF NOT EXISTS replay_manifests (
                    manifest_hash TEXT PRIMARY KEY,
                    object_id TEXT NOT NULL,
                    created_at INTEGER,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_replay_object_time
                    ON replay_manifests(object_id, created_at);

                CREATE TABLE IF NOT EXISTS assessments (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    object_id TEXT NOT NULL,
                    timestamp_unix_seconds INTEGER,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_assessments_object_time
                    ON assessments(object_id, timestamp_unix_seconds);

                CREATE TABLE IF NOT EXISTS ranked_events (
                    event_id TEXT PRIMARY KEY,
                    object_id TEXT NOT NULL,
                    risk REAL NOT NULL,
                    severity_rank INTEGER NOT NULL,
                    timestamp_unix_seconds INTEGER,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_ranked_events_priority
                    ON ranked_events(risk DESC, severity_rank DESC, timestamp_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS request_response_logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    request_id TEXT NOT NULL,
                    endpoint TEXT NOT NULL,
                    object_id TEXT,
                    timestamp_unix_seconds INTEGER NOT NULL,
                    request_json TEXT NOT NULL,
                    response_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_request_logs_object_time
                    ON request_response_logs(object_id, timestamp_unix_seconds);

                CREATE TABLE IF NOT EXISTS ingest_batches (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    request_id TEXT NOT NULL,
                    source TEXT NOT NULL,
                    timestamp_unix_seconds INTEGER NOT NULL,
                    records_received INTEGER NOT NULL,
                    observations_created INTEGER NOT NULL,
                    object_ids_json TEXT NOT NULL,
                    raw_payload TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_ingest_batches_source_time
                    ON ingest_batches(source, timestamp_unix_seconds);

                CREATE TABLE IF NOT EXISTS notification_deliveries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    delivery_id TEXT NOT NULL UNIQUE,
                    request_id TEXT NOT NULL,
                    object_id TEXT NOT NULL,
                    recipient_json TEXT NOT NULL,
                    channel TEXT NOT NULL,
                    target TEXT NOT NULL,
                    status TEXT NOT NULL,
                    status_code INTEGER,
                    timestamp_unix_seconds INTEGER NOT NULL,
                    attempt_number INTEGER NOT NULL DEFAULT 1,
                    max_attempts INTEGER NOT NULL DEFAULT 1,
                    notification_reason TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    response_body TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_notification_deliveries_object_time
                    ON notification_deliveries(object_id, timestamp_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS prediction_snapshots (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    snapshot_id TEXT NOT NULL UNIQUE,
                    request_id TEXT NOT NULL,
                    endpoint TEXT NOT NULL,
                    object_id TEXT NOT NULL,
                    generated_at_unix_seconds INTEGER NOT NULL,
                    horizon_hours REAL NOT NULL,
                    base_epoch_unix_seconds INTEGER,
                    propagation_model TEXT NOT NULL,
                    model_version TEXT NOT NULL,
                    evidence_bundle_hash TEXT,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_prediction_snapshots_object_time
                    ON prediction_snapshots(object_id, generated_at_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS passive_site_profiles (
                    site_id TEXT PRIMARY KEY,
                    site_name TEXT NOT NULL,
                    site_type TEXT NOT NULL,
                    criticality TEXT NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_site_profiles_name
                    ON passive_site_profiles(site_name, site_type);

                CREATE TABLE IF NOT EXISTS passive_seed_registry (
                    seed_key TEXT PRIMARY KEY,
                    site_id TEXT,
                    discovered_at_unix_seconds INTEGER NOT NULL,
                    last_scanned_at_unix_seconds INTEGER,
                    last_event_at_unix_seconds INTEGER,
                    confidence REAL NOT NULL,
                    source_count INTEGER NOT NULL,
                    classification_status TEXT NOT NULL,
                    seed_status TEXT NOT NULL,
                    scan_priority REAL NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_seed_registry_priority
                    ON passive_seed_registry(scan_priority DESC, last_scanned_at_unix_seconds DESC);
                CREATE INDEX IF NOT EXISTS idx_passive_seed_registry_site
                    ON passive_seed_registry(site_id, classification_status);

                CREATE TABLE IF NOT EXISTS passive_region_targets (
                    region_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    enabled INTEGER NOT NULL,
                    updated_at_unix_seconds INTEGER NOT NULL,
                    last_discovered_at_unix_seconds INTEGER,
                    last_scheduler_run_at_unix_seconds INTEGER,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_region_targets_due
                    ON passive_region_targets(enabled, last_discovered_at_unix_seconds, updated_at_unix_seconds);

                CREATE TABLE IF NOT EXISTS passive_region_run_logs (
                    run_id TEXT PRIMARY KEY,
                    request_id TEXT NOT NULL,
                    origin TEXT NOT NULL,
                    started_at_unix_seconds INTEGER NOT NULL,
                    finished_at_unix_seconds INTEGER NOT NULL,
                    status TEXT NOT NULL,
                    next_run_at_unix_seconds INTEGER,
                    region_ids_json TEXT NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_region_run_logs_time
                    ON passive_region_run_logs(finished_at_unix_seconds DESC, status);

                CREATE TABLE IF NOT EXISTS passive_region_leases (
                    region_id TEXT PRIMARY KEY,
                    worker_id TEXT NOT NULL,
                    run_id TEXT NOT NULL,
                    acquired_at_unix_seconds INTEGER NOT NULL,
                    heartbeat_at_unix_seconds INTEGER NOT NULL,
                    expires_at_unix_seconds INTEGER NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_region_leases_expiry
                    ON passive_region_leases(expires_at_unix_seconds, worker_id);

                CREATE TABLE IF NOT EXISTS passive_worker_heartbeats (
                    worker_id TEXT PRIMARY KEY,
                    started_at_unix_seconds INTEGER NOT NULL,
                    last_heartbeat_unix_seconds INTEGER NOT NULL,
                    status TEXT NOT NULL,
                    current_region_id TEXT,
                    current_phase TEXT NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_worker_heartbeats_last
                    ON passive_worker_heartbeats(last_heartbeat_unix_seconds DESC, status);

                CREATE TABLE IF NOT EXISTS passive_source_health_samples (
                    sample_id TEXT PRIMARY KEY,
                    request_id TEXT NOT NULL,
                    region_id TEXT,
                    source_kind TEXT NOT NULL,
                    fetched INTEGER NOT NULL,
                    generated_at_unix_seconds INTEGER NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_source_health_samples_time
                    ON passive_source_health_samples(source_kind, generated_at_unix_seconds DESC);
                CREATE INDEX IF NOT EXISTS idx_passive_source_health_samples_region
                    ON passive_source_health_samples(region_id, generated_at_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS passive_canonical_events (
                    canonical_id TEXT PRIMARY KEY,
                    site_id TEXT NOT NULL,
                    threat_type TEXT NOT NULL,
                    last_seen_at_unix_seconds INTEGER NOT NULL,
                    max_risk_score REAL NOT NULL,
                    severity_rank INTEGER NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_canonical_events_site_time
                    ON passive_canonical_events(site_id, last_seen_at_unix_seconds DESC);
                CREATE INDEX IF NOT EXISTS idx_passive_canonical_events_priority
                    ON passive_canonical_events(max_risk_score DESC, severity_rank DESC, last_seen_at_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS passive_observations (
                    observation_id TEXT PRIMARY KEY,
                    source_kind TEXT NOT NULL,
                    observed_at_unix_seconds INTEGER NOT NULL,
                    latitude REAL NOT NULL,
                    longitude REAL NOT NULL,
                    altitude_m REAL NOT NULL,
                    severity_hint REAL NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_observations_source_time
                    ON passive_observations(source_kind, observed_at_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS passive_events (
                    event_id TEXT PRIMARY KEY,
                    site_id TEXT NOT NULL,
                    threat_type TEXT NOT NULL,
                    source_kind TEXT NOT NULL,
                    risk_score REAL NOT NULL,
                    observed_at_unix_seconds INTEGER NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_events_site_priority
                    ON passive_events(site_id, risk_score DESC, observed_at_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS passive_risk_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    site_id TEXT NOT NULL,
                    window_start_unix_seconds INTEGER NOT NULL,
                    window_end_unix_seconds INTEGER NOT NULL,
                    cumulative_risk REAL NOT NULL,
                    peak_risk REAL NOT NULL,
                    event_count INTEGER NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_risk_history_site_time
                    ON passive_risk_history(site_id, window_end_unix_seconds DESC);

                CREATE TABLE IF NOT EXISTS passive_pattern_signals (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    site_id TEXT NOT NULL,
                    threat_type TEXT NOT NULL,
                    recurring_events INTEGER NOT NULL,
                    average_risk REAL NOT NULL,
                    payload_json TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_passive_pattern_signals_site_priority
                    ON passive_pattern_signals(site_id, average_risk DESC, recurring_events DESC);
                ",
            )?;
            ensure_column(
                connection,
                "passive_region_run_logs",
                "origin",
                "TEXT NOT NULL DEFAULT 'Api'",
            )?;
            ensure_column(
                connection,
                "passive_region_run_logs",
                "next_run_at_unix_seconds",
                "INTEGER",
            )?;
            Ok(())
        })
    }

    fn with_connection<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| StorageError::PoisonedLock)?;
        f(&connection)
    }
}

fn ensure_column(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<(), StorageError> {
    let pragma = format!("PRAGMA table_info({table_name})");
    let mut statement = connection.prepare(&pragma)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    let columns = rows.collect::<Result<Vec<_>, _>>()?;
    if !columns.iter().any(|column| column == column_name) {
        connection.execute(
            &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
            [],
        )?;
    }
    Ok(())
}

fn to_json(value: &impl Serialize) -> Result<String, StorageError> {
    serde_json::to_string(value).map_err(StorageError::from)
}

fn from_json<T: DeserializeOwned>(json: &str) -> Result<T, StorageError> {
    serde_json::from_str(json).map_err(StorageError::from)
}

fn canonical_event_signature(event: &PassiveEvent) -> String {
    let phenomenon_key = event.phenomenon_key.trim();
    if !phenomenon_key.is_empty() {
        return phenomenon_key.to_string();
    }

    let time_bucket = event
        .payload
        .get("phenomenon_time_bucket")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_else(|| {
            event
                .observed_at_unix_seconds
                .div_euclid(canonical_window_seconds(event.source_kind))
        });
    let anchor = event
        .payload
        .get("phenomenon_anchor")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| canonical_anchor_from_payload(event))
        .unwrap_or_else(|| {
            canonical_geo_anchor(event, canonical_geo_bucket_degrees(event.source_kind))
        });

    format!(
        "{}:{:?}:{:?}:{anchor}:{time_bucket}",
        event.site_id, event.source_kind, event.threat_type
    )
}

fn canonical_anchor_from_payload(event: &PassiveEvent) -> Option<String> {
    match event.source_kind {
        PassiveSourceKind::Adsb => canonical_payload_string(&event.payload, "flight_id", "flight"),
        PassiveSourceKind::Notam => canonical_payload_string(&event.payload, "notam_id", "notam"),
        PassiveSourceKind::Orbital => {
            canonical_payload_string(&event.payload, "object_id", "object")
        }
        PassiveSourceKind::Satellite | PassiveSourceKind::InfraMap => {
            canonical_payload_string(&event.payload, "scene_id", "scene")
        }
        PassiveSourceKind::Weather | PassiveSourceKind::FireSmoke => None,
    }
}

fn canonical_payload_string(
    payload: &serde_json::Value,
    key: &str,
    prefix: &str,
) -> Option<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(|value| format!("{prefix}:{value}"))
}

fn canonical_window_seconds(source_kind: PassiveSourceKind) -> i64 {
    match source_kind {
        PassiveSourceKind::Adsb | PassiveSourceKind::Weather | PassiveSourceKind::FireSmoke => {
            6 * 3_600
        }
        PassiveSourceKind::Notam
        | PassiveSourceKind::Orbital
        | PassiveSourceKind::Satellite
        | PassiveSourceKind::InfraMap => 24 * 3_600,
    }
}

fn canonical_geo_bucket_degrees(source_kind: PassiveSourceKind) -> f64 {
    match source_kind {
        PassiveSourceKind::Weather | PassiveSourceKind::FireSmoke | PassiveSourceKind::Orbital => {
            0.5
        }
        PassiveSourceKind::Adsb
        | PassiveSourceKind::Satellite
        | PassiveSourceKind::Notam
        | PassiveSourceKind::InfraMap => 0.25,
    }
}

fn canonical_geo_anchor(event: &PassiveEvent, bucket_degrees: f64) -> String {
    format!(
        "geo:{}:{}",
        canonical_coordinate_bucket(event_latitude(event), bucket_degrees),
        canonical_coordinate_bucket(event_longitude(event), bucket_degrees)
    )
}

#[allow(clippy::cast_possible_truncation)]
fn canonical_coordinate_bucket(value: f64, bucket_degrees: f64) -> i64 {
    (value / bucket_degrees).floor() as i64
}

fn envelope_confidence(
    event_id: &str,
    core_envelopes: &[PassiveCoreEnvelope],
    fallback_risk: f64,
) -> f64 {
    core_envelopes
        .iter()
        .find(|envelope| envelope.ranked_event.event_id == event_id)
        .map_or(fallback_risk, |envelope| envelope.assessment.confidence)
}

fn envelope_bundle_hashes(event_id: &str, core_envelopes: &[PassiveCoreEnvelope]) -> Vec<String> {
    core_envelopes
        .iter()
        .filter(|envelope| envelope.ranked_event.event_id == event_id)
        .map(|envelope| envelope.evidence_bundle.bundle_hash.clone())
        .collect()
}

fn envelope_manifest_hashes(event_id: &str, core_envelopes: &[PassiveCoreEnvelope]) -> Vec<String> {
    core_envelopes
        .iter()
        .filter(|envelope| envelope.ranked_event.event_id == event_id)
        .map(|envelope| envelope.replay_manifest.manifest_hash.clone())
        .collect()
}

fn event_latitude(event: &PassiveEvent) -> f64 {
    event.payload["latitude"].as_f64().unwrap_or_default()
}

fn event_longitude(event: &PassiveEvent) -> f64 {
    event.payload["longitude"].as_f64().unwrap_or_default()
}

fn merge_canonical_passive_event(
    existing: Option<CanonicalPassiveEvent>,
    event: &PassiveEvent,
    core_envelopes: &[PassiveCoreEnvelope],
) -> CanonicalPassiveEvent {
    let event_id = event.event_id.to_string();
    let confidence = envelope_confidence(&event_id, core_envelopes, event.risk_score);
    let bundle_hashes = envelope_bundle_hashes(&event_id, core_envelopes);
    let manifest_hashes = envelope_manifest_hashes(&event_id, core_envelopes);
    let severity_rank = severity_rank_from_risk(event.risk_score);

    if let Some(mut canonical) = existing {
        canonical.first_seen_at_unix_seconds = canonical
            .first_seen_at_unix_seconds
            .min(event.observed_at_unix_seconds);
        canonical.last_seen_at_unix_seconds = canonical
            .last_seen_at_unix_seconds
            .max(event.observed_at_unix_seconds);

        if !canonical.source_kinds.contains(&event.source_kind) {
            canonical.source_kinds.push(event.source_kind);
        }

        let is_new_event = !canonical.related_event_ids.contains(&event_id);
        if is_new_event {
            let previous_occurrences = canonical.occurrence_count;
            let next_occurrences = previous_occurrences.saturating_add(1);
            canonical.avg_confidence =
                ((canonical.avg_confidence * f64::from(previous_occurrences)) + confidence)
                    / f64::from(next_occurrences);
            canonical.occurrence_count = next_occurrences;
            canonical.related_event_ids.push(event_id);
        }

        for bundle_hash in bundle_hashes {
            if !canonical.evidence_bundle_hashes.contains(&bundle_hash) {
                canonical.evidence_bundle_hashes.push(bundle_hash);
            }
        }

        for manifest_hash in manifest_hashes {
            if !canonical.replay_manifest_hashes.contains(&manifest_hash) {
                canonical.replay_manifest_hashes.push(manifest_hash);
            }
        }

        if event.risk_score >= canonical.max_risk_score {
            canonical.max_risk_score = event.risk_score;
            canonical.severity_rank = severity_rank;
            canonical.summary.clone_from(&event.summary);
            canonical.latitude = event_latitude(event);
            canonical.longitude = event_longitude(event);
        }

        canonical
    } else {
        CanonicalPassiveEvent {
            canonical_id: canonical_event_signature(event),
            signature: canonical_event_signature(event),
            site_id: event.site_id.to_string(),
            site_name: event.site_name.clone(),
            threat_type: event.threat_type,
            source_kinds: vec![event.source_kind],
            first_seen_at_unix_seconds: event.observed_at_unix_seconds,
            last_seen_at_unix_seconds: event.observed_at_unix_seconds,
            occurrence_count: 1,
            max_risk_score: event.risk_score,
            avg_confidence: confidence,
            severity_rank,
            summary: event.summary.clone(),
            latitude: event_latitude(event),
            longitude: event_longitude(event),
            related_event_ids: vec![event_id],
            evidence_bundle_hashes: bundle_hashes,
            replay_manifest_hashes: manifest_hashes,
        }
    }
}

type NotificationDeliveryRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<i64>,
    i64,
    i64,
    i64,
    String,
    String,
    Option<String>,
);

fn notification_delivery_from_row(
    row: NotificationDeliveryRow,
) -> Result<NotificationDeliveryLog, StorageError> {
    let (
        delivery_id,
        request_id,
        object_id,
        recipient_json,
        channel,
        target,
        status,
        status_code,
        timestamp_unix_seconds,
        attempt_number,
        max_attempts,
        notification_reason,
        payload_json,
        response_body,
    ) = row;
    Ok(NotificationDeliveryLog {
        delivery_id,
        request_id,
        object_id,
        recipient: from_json(&recipient_json)?,
        channel,
        target,
        status,
        status_code: status_code.and_then(|value| u16::try_from(value).ok()),
        timestamp_unix_seconds,
        attempt_number: u32::try_from(attempt_number).unwrap_or(1),
        max_attempts: u32::try_from(max_attempts).unwrap_or(1),
        notification_reason,
        payload_json,
        response_body,
    })
}

fn severity_rank_from_risk(risk: f64) -> i64 {
    let severity = if risk >= 0.85 {
        AlertSeverity::Critical
    } else if risk >= 0.65 {
        AlertSeverity::Warning
    } else if risk >= 0.35 {
        AlertSeverity::Watch
    } else {
        AlertSeverity::Info
    };

    match severity {
        AlertSeverity::Info => 1,
        AlertSeverity::Watch => 2,
        AlertSeverity::Warning => 3,
        AlertSeverity::Critical => 4,
    }
}

#[cfg(test)]
mod tests {
    use sss_core::{
        replay_manifest_for, AnalysisStatus, AnomalySignal, BehaviorSignature, DerivedFeatures,
        EvidenceBundle, IntelligenceAssessment, IntelligenceRecipient, MissionClass, Observation,
        ObservationSource, OrbitRegime, OrbitalState, PredictedEvent, Prediction, RankedEvent,
        RankedEventType, RiskProfile, SpaceObject, Vector3,
    };
    use sss_passive_scanner::{
        AdsbFlightObservation, FireSmokeObservation, OpenInfraSiteSeed, PassiveScanner,
        SatelliteSceneObservation, WeatherObservation,
    };
    use sss_site_registry::{Criticality, SiteType};

    use super::*;

    #[test]
    fn stores_and_loads_evidence_and_replay_by_hash() {
        let store = SqliteStore::in_memory().expect("store");
        let bundle = sample_bundle();
        let manifest = replay_manifest_for(&bundle);

        store.store_evidence_bundle(&bundle).expect("bundle stored");
        store
            .store_replay_manifest(&manifest)
            .expect("manifest stored");

        assert_eq!(
            store
                .evidence_bundle(&bundle.bundle_hash)
                .expect("bundle lookup"),
            Some(bundle)
        );
        assert_eq!(
            store
                .replay_manifest(&manifest.manifest_hash)
                .expect("manifest lookup"),
            Some(manifest)
        );
    }

    #[test]
    fn stores_history_by_object_and_priority_events() {
        let store = SqliteStore::in_memory().expect("store");
        store
            .store_assessment("25544", Some(1_704_067_200), &sample_assessment())
            .expect("assessment");
        store
            .store_prediction_snapshot(&sample_prediction_snapshot())
            .expect("prediction snapshot");
        store
            .store_ranked_events(&[
                sample_ranked_event("low", 0.2),
                sample_ranked_event("high", 0.9),
            ])
            .expect("events");
        store
            .log_ingest_batch(&IngestBatchLog {
                request_id: "ingest-1".to_string(),
                source: "celestrak".to_string(),
                timestamp_unix_seconds: 1_704_067_200,
                records_received: 1,
                observations_created: 1,
                object_ids: vec!["25544".to_string()],
                raw_payload: "ISS".to_string(),
            })
            .expect("ingest batch");
        store
            .log_notification_delivery(&NotificationDeliveryLog {
                delivery_id: "delivery-1".to_string(),
                request_id: "notify-1".to_string(),
                object_id: "25544".to_string(),
                recipient: IntelligenceRecipient::Operator,
                channel: "webhook".to_string(),
                target: "http://example.test/hook".to_string(),
                status: "delivered".to_string(),
                status_code: Some(200),
                timestamp_unix_seconds: 1_704_067_260,
                attempt_number: 1,
                max_attempts: 3,
                notification_reason: "operator owns immediate triage".to_string(),
                payload_json: "{\"hello\":\"world\"}".to_string(),
                response_body: Some("ok".to_string()),
            })
            .expect("notification delivery");

        assert_eq!(store.assessment_history("25544").expect("history").len(), 1);
        assert_eq!(
            store.top_ranked_events(1).expect("events")[0].event_id,
            "high"
        );
        assert_eq!(
            store.top_ranked_events(1).expect("events")[0].target_epoch_unix_seconds,
            Some(1_704_070_800)
        );
        assert_eq!(
            store
                .notification_deliveries(1, Some("25544"))
                .expect("deliveries")[0]
                .status,
            "delivered"
        );
        assert_eq!(
            store
                .prediction_snapshots("25544", 1)
                .expect("prediction snapshots")[0]
                .propagation_model,
            "sgp4"
        );
    }

    #[test]
    fn stores_objects_and_latest_observations_for_hydration() {
        let store = SqliteStore::in_memory().expect("store");
        let object = sample_object();
        let mut later_observation = sample_observation();
        later_observation.observed_state.epoch_unix_seconds += 60;

        store.store_space_object(&object).expect("object");
        store
            .store_observation(&sample_observation())
            .expect("observation");
        store
            .store_observation(&later_observation)
            .expect("later observation");

        assert_eq!(store.space_objects().expect("objects"), vec![object]);
        assert_eq!(
            store.latest_observations().expect("observations"),
            vec![later_observation]
        );
    }

    #[test]
    fn stores_passive_scan_corpus_and_core_artifacts() {
        let store = SqliteStore::in_memory().expect("store");
        let scanner = PassiveScanner::new();
        let output = scanner.scan(&sample_passive_scan_input());
        let first_site_id = output.sites[0].site.site_id.to_string();
        let first_bundle_hash = output.core_envelopes[0].evidence_bundle.bundle_hash.clone();

        store
            .store_passive_scan_output(&output)
            .expect("passive scan stored");

        assert_eq!(store.passive_site_profiles().expect("sites").len(), 1);
        assert!(!store
            .passive_observations(10, Some(PassiveSourceKind::Weather))
            .expect("observations")
            .is_empty());
        assert!(!store
            .passive_events_for_site(&first_site_id, 10)
            .expect("events")
            .is_empty());
        assert_eq!(
            store
                .passive_risk_history(&first_site_id, 10)
                .expect("risk history")
                .len(),
            1
        );
        assert!(!store
            .passive_patterns(&first_site_id, 10)
            .expect("patterns")
            .is_empty());
        assert!(store
            .evidence_bundle(&first_bundle_hash)
            .expect("bundle")
            .is_some());
        assert!(!store
            .assessment_history(&output.core_envelopes[0].finding.object_id)
            .expect("assessment history")
            .is_empty());
        assert!(!store
            .canonical_passive_events(10, Some(&first_site_id))
            .expect("canonical events")
            .is_empty());
    }

    #[test]
    fn merges_canonical_passive_events_by_signature() {
        let store = SqliteStore::in_memory().expect("store");
        let scanner = PassiveScanner::new();
        let first_output = scanner.scan(&sample_passive_scan_input());
        let second_output = scanner.scan(&sample_passive_scan_input());
        let site_id = first_output.sites[0].site.site_id.to_string();

        store
            .store_passive_scan_output(&first_output)
            .expect("first passive scan stored");
        store
            .store_passive_scan_output(&second_output)
            .expect("second passive scan stored");

        let canonical_events = store
            .canonical_passive_events(50, Some(&site_id))
            .expect("canonical events");
        assert!(!canonical_events.is_empty());
        assert!(canonical_events
            .iter()
            .any(|event| event.occurrence_count >= 2));
        assert!(canonical_events
            .iter()
            .any(|event| event.related_event_ids.len() >= 2));
    }

    #[test]
    fn canonical_passive_merges_are_idempotent_for_replayed_event_ids() {
        let store = SqliteStore::in_memory().expect("store");
        let scanner = PassiveScanner::new();
        let output = scanner.scan(&sample_passive_scan_input());
        let site_id = output.sites[0].site.site_id.to_string();

        store
            .store_passive_scan_output(&output)
            .expect("first passive scan stored");
        store
            .store_passive_scan_output(&output)
            .expect("second passive scan stored");

        let canonical_events = store
            .canonical_passive_events(50, Some(&site_id))
            .expect("canonical events");
        assert!(!canonical_events.is_empty());
        assert!(canonical_events
            .iter()
            .all(|event| event.occurrence_count == 1));
        assert!(canonical_events
            .iter()
            .all(|event| event.related_event_ids.len() == 1));
    }

    #[test]
    fn canonical_signature_rebuilds_from_persisted_anchor_metadata() {
        let scanner = PassiveScanner::new();
        let output = scanner.scan(&sample_passive_scan_input());
        let adsb_event = output
            .events
            .iter()
            .find(|event| event.source_kind == PassiveSourceKind::Adsb)
            .expect("adsb event");
        let weather_event = output
            .events
            .iter()
            .find(|event| event.source_kind == PassiveSourceKind::Weather)
            .expect("weather event");

        let mut adsb_without_key = adsb_event.clone();
        adsb_without_key.phenomenon_key.clear();
        let mut weather_without_key = weather_event.clone();
        weather_without_key.phenomenon_key.clear();

        assert_eq!(
            canonical_event_signature(&adsb_without_key),
            adsb_event.phenomenon_key
        );
        assert_eq!(
            canonical_event_signature(&weather_without_key),
            weather_event.phenomenon_key
        );
    }

    #[test]
    fn stores_passive_seed_lifecycle_records() {
        let store = SqliteStore::in_memory().expect("store");
        let seed_record = sample_passive_seed_record();

        store
            .store_passive_seed_records(std::slice::from_ref(&seed_record))
            .expect("seed stored");

        assert_eq!(
            store
                .passive_seed_record(&seed_record.seed_key)
                .expect("seed lookup"),
            Some(seed_record.clone())
        );
        assert_eq!(
            store
                .passive_seed_records(10)
                .expect("seed list")
                .first()
                .cloned(),
            Some(seed_record)
        );
    }

    #[test]
    fn stores_passive_region_targets() {
        let store = SqliteStore::in_memory().expect("store");
        let target = sample_passive_region_target();

        store
            .store_passive_region_target(&target)
            .expect("region stored");

        assert_eq!(
            store
                .passive_region_target(&target.region_id)
                .expect("region lookup"),
            Some(target.clone())
        );
        assert_eq!(
            store
                .passive_region_targets(10, true)
                .expect("region list")
                .first()
                .cloned(),
            Some(target)
        );
    }

    #[test]
    fn stores_passive_region_run_logs() {
        let store = SqliteStore::in_memory().expect("store");
        let log = sample_passive_region_run_log();

        store
            .store_passive_region_run_log(&log)
            .expect("run log stored");

        assert_eq!(
            store
                .passive_region_run_logs(10, None)
                .expect("run logs")
                .first()
                .cloned(),
            Some(log.clone())
        );
        assert_eq!(
            store
                .passive_region_run_logs(10, Some("andalusia-critical-infra"))
                .expect("filtered run logs")
                .first()
                .cloned(),
            Some(log.clone())
        );
        assert_eq!(
            store
                .passive_region_run_log(&log.run_id)
                .expect("run log detail"),
            Some(log)
        );
        assert!(store
            .passive_region_run_log("missing-run")
            .expect("missing run log lookup")
            .is_none());
    }

    #[test]
    fn stores_worker_heartbeat_and_coordinates_region_leases() {
        let store = SqliteStore::in_memory().expect("store");
        let region_id = "andalusia-critical-infra";

        let first = store
            .acquire_passive_region_lease(region_id, "worker-a", "run-a", 100, 30)
            .expect("lease acquired")
            .expect("worker-a owns lease");
        assert_eq!(first.expires_at_unix_seconds, 130);

        let blocked = store
            .acquire_passive_region_lease(region_id, "worker-b", "run-b", 110, 30)
            .expect("lease contention checked");
        assert!(blocked.is_none());

        assert!(store
            .refresh_passive_region_lease(region_id, "worker-a", "run-a", 120, 45)
            .expect("lease refreshed"));
        assert!(!store
            .refresh_passive_region_lease(region_id, "worker-a", "stale-run", 120, 45)
            .expect("stale run cannot refresh"));
        let leases = store.passive_region_leases(10).expect("leases");
        assert_eq!(leases[0].worker_id, "worker-a");
        assert_eq!(leases[0].expires_at_unix_seconds, 165);

        let expired_takeover = store
            .acquire_passive_region_lease(region_id, "worker-b", "run-b", 166, 30)
            .expect("expired lease acquired")
            .expect("worker-b owns expired lease");
        assert_eq!(expired_takeover.worker_id, "worker-b");
        assert!(!store
            .release_passive_region_lease(region_id, "worker-a", "run-a")
            .expect("stale release rejected"));
        assert!(store
            .release_passive_region_lease(region_id, "worker-b", "run-b")
            .expect("active release accepted"));
        assert!(store.passive_region_leases(10).expect("leases").is_empty());

        let heartbeat = PassiveWorkerHeartbeat {
            worker_id: "worker-a".to_string(),
            started_at_unix_seconds: 100,
            last_heartbeat_unix_seconds: 130,
            status: "running".to_string(),
            current_region_id: Some(region_id.to_string()),
            current_phase: "region_run".to_string(),
            last_error: None,
            version: "test".to_string(),
        };
        store
            .store_passive_worker_heartbeat(&heartbeat)
            .expect("heartbeat stored");
        assert_eq!(
            store
                .passive_worker_heartbeats(10)
                .expect("worker heartbeats"),
            vec![heartbeat]
        );
        assert_eq!(
            store
                .purge_expired_passive_region_leases(200)
                .expect("expired leases purged"),
            0
        );
    }

    #[test]
    fn purges_expired_passive_region_leases() {
        let store = SqliteStore::in_memory().expect("store");
        store
            .acquire_passive_region_lease("region-a", "worker-a", "run-a", 100, 10)
            .expect("lease acquired");
        store
            .acquire_passive_region_lease("region-b", "worker-b", "run-b", 100, 40)
            .expect("lease acquired");

        assert_eq!(
            store
                .purge_expired_passive_region_leases(111)
                .expect("expired leases purged"),
            1
        );
        let leases = store.passive_region_leases(10).expect("leases");
        assert_eq!(leases.len(), 1);
        assert_eq!(leases[0].region_id, "region-b");
    }

    #[test]
    fn purges_stale_passive_worker_heartbeats() {
        let store = SqliteStore::in_memory().expect("store");
        let stale = PassiveWorkerHeartbeat {
            worker_id: "worker-stale".to_string(),
            started_at_unix_seconds: 100,
            last_heartbeat_unix_seconds: 120,
            status: "idle".to_string(),
            current_region_id: None,
            current_phase: "cycle_complete".to_string(),
            last_error: None,
            version: "test".to_string(),
        };
        let fresh = PassiveWorkerHeartbeat {
            worker_id: "worker-fresh".to_string(),
            started_at_unix_seconds: 200,
            last_heartbeat_unix_seconds: 260,
            status: "running".to_string(),
            current_region_id: Some("region-a".to_string()),
            current_phase: "region_run".to_string(),
            last_error: None,
            version: "test".to_string(),
        };

        store
            .store_passive_worker_heartbeat(&stale)
            .expect("stale heartbeat stored");
        store
            .store_passive_worker_heartbeat(&fresh)
            .expect("fresh heartbeat stored");

        assert_eq!(
            store
                .count_stale_passive_worker_heartbeats(200)
                .expect("stale heartbeat count"),
            1
        );
        assert_eq!(
            store
                .stale_passive_worker_heartbeats(10, 200)
                .expect("stale heartbeat list"),
            vec![stale.clone()]
        );
        assert_eq!(
            store
                .purge_stale_passive_worker_heartbeats(200)
                .expect("stale heartbeats purged"),
            1
        );
        assert_eq!(
            store
                .passive_worker_heartbeats(10)
                .expect("worker heartbeats"),
            vec![fresh]
        );
    }

    #[test]
    fn stores_passive_source_health_samples() {
        let store = SqliteStore::in_memory().expect("store");
        let samples = vec![
            PassiveSourceHealthSample {
                sample_id: "sample-weather".to_string(),
                request_id: "scan-1".to_string(),
                region_id: Some("andalusia-critical-infra".to_string()),
                source_kind: PassiveSourceKind::Weather,
                fetched: true,
                observations_collected: 3,
                detail: "Open-Meteo current weather pulled per site.".to_string(),
                generated_at_unix_seconds: 1_776_384_000,
                window_start_unix_seconds: 1_776_297_600,
                window_end_unix_seconds: 1_776_384_000,
            },
            PassiveSourceHealthSample {
                sample_id: "sample-adsb".to_string(),
                request_id: "scan-1".to_string(),
                region_id: Some("andalusia-critical-infra".to_string()),
                source_kind: PassiveSourceKind::Adsb,
                fetched: false,
                observations_collected: 0,
                detail: "OpenSky bearer token not configured.".to_string(),
                generated_at_unix_seconds: 1_776_384_001,
                window_start_unix_seconds: 1_776_297_600,
                window_end_unix_seconds: 1_776_384_000,
            },
            PassiveSourceHealthSample {
                sample_id: "sample-weather-other-region".to_string(),
                request_id: "scan-2".to_string(),
                region_id: Some("lisbon-critical-infra".to_string()),
                source_kind: PassiveSourceKind::Weather,
                fetched: true,
                observations_collected: 1,
                detail: "Open-Meteo current weather pulled for Lisbon.".to_string(),
                generated_at_unix_seconds: 1_776_384_002,
                window_start_unix_seconds: 1_776_297_600,
                window_end_unix_seconds: 1_776_384_000,
            },
        ];

        store
            .store_passive_source_health_samples(&samples)
            .expect("source health samples stored");

        assert_eq!(
            store
                .passive_source_health_samples(10, Some(PassiveSourceKind::Weather), None)
                .expect("weather source samples"),
            vec![samples[2].clone(), samples[0].clone()]
        );
        assert_eq!(
            store
                .passive_source_health_samples(
                    10,
                    Some(PassiveSourceKind::Weather),
                    Some("andalusia-critical-infra"),
                )
                .expect("region weather source samples"),
            vec![samples[0].clone()]
        );
        assert_eq!(
            store
                .passive_source_health_samples(10, None, None)
                .expect("all source samples")
                .len(),
            3
        );
        assert_eq!(
            store
                .count_passive_source_health_samples_before(
                    1_776_384_002,
                    Some(PassiveSourceKind::Weather),
                    Some("andalusia-critical-infra"),
                )
                .expect("old source health count"),
            1
        );
        assert_eq!(
            store
                .passive_source_health_samples(10, None, None)
                .expect("source samples before dry run")
                .len(),
            3
        );
        assert_eq!(
            store
                .purge_passive_source_health_samples(
                    1_776_384_002,
                    Some(PassiveSourceKind::Weather),
                    Some("andalusia-critical-infra"),
                )
                .expect("old source health purged"),
            1
        );
        assert_eq!(
            store
                .passive_source_health_samples(10, None, None)
                .expect("remaining source samples"),
            vec![samples[2].clone(), samples[1].clone()]
        );
    }

    fn sample_bundle() -> EvidenceBundle {
        EvidenceBundle {
            object_id: "25544".to_string(),
            window_start_unix_seconds: Some(1_704_067_200),
            window_end_unix_seconds: Some(1_704_067_200),
            observations: vec![Observation {
                object_id: "25544".to_string(),
                observed_state: OrbitalState {
                    epoch_unix_seconds: 1_704_067_200,
                    position_km: Vector3 {
                        x: 6_790.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    velocity_km_s: Vector3 {
                        x: 0.0,
                        y: 7.6,
                        z: 0.0,
                    },
                },
                rf_mhz: None,
                maneuver_detected: false,
                source: ObservationSource::Catalog,
            }],
            sources: vec![ObservationSource::Catalog],
            events: Vec::new(),
            anomaly_score: 0.0,
            signals: Vec::new(),
            derived_features: DerivedFeatures {
                position_delta_km: 0.0,
                velocity_delta_km_s: 0.0,
                rf_delta_mhz: None,
                maneuver_detected: false,
            },
            confidence_contributors: Vec::new(),
            bundle_hash: "bundle-1".to_string(),
        }
    }

    fn sample_assessment() -> IntelligenceAssessment {
        IntelligenceAssessment {
            status: AnalysisStatus::Nominal,
            behavior: "baseline behavior".to_string(),
            confidence: 0.44,
            prediction: Prediction {
                object_id: "25544".to_string(),
                horizon_hours: 24.0,
                event: PredictedEvent::ContinuedAnomalousBehavior,
                probability: 0.08,
                explanation: "baseline".to_string(),
                propagation_model: "baseline".to_string(),
                based_on_observation_epoch: None,
                predicted_state: None,
                position_delta_km: None,
                velocity_delta_km_s: None,
            },
            explanation: "baseline".to_string(),
            decision_version: "sss-decision-v1".to_string(),
            model_version: "sss-behavioral-mvp-v1".to_string(),
            rule_hits: Vec::new(),
            signal_summary: vec![format!("{:?}", AnomalySignal::PositionDrift)],
        }
    }

    fn sample_ranked_event(id: &str, risk: f64) -> RankedEvent {
        RankedEvent {
            event_id: id.to_string(),
            object_id: "25544".to_string(),
            object_name: "ISS (ZARYA)".to_string(),
            risk,
            target_epoch_unix_seconds: Some(1_704_067_200 + if id == "high" { 3_600 } else { 0 }),
            event_type: RankedEventType::BehaviorShiftDetected,
            prediction: None,
            summary: format!("risk {risk}"),
        }
    }

    fn sample_prediction_snapshot() -> PredictionSnapshot {
        PredictionSnapshot {
            snapshot_id: "snapshot-1".to_string(),
            request_id: "predict-1".to_string(),
            endpoint: "/v1/predict".to_string(),
            object_id: "25544".to_string(),
            generated_at_unix_seconds: 1_704_067_240,
            horizon_hours: 24.0,
            base_epoch_unix_seconds: Some(1_704_067_200),
            propagation_model: "sgp4".to_string(),
            model_version: "sss-behavioral-mvp-v1".to_string(),
            evidence_bundle_hash: Some("bundle-1".to_string()),
            predictions: vec![sample_assessment().prediction],
        }
    }

    fn sample_object() -> SpaceObject {
        SpaceObject {
            id: "25544".to_string(),
            name: "ISS (ZARYA)".to_string(),
            regime: OrbitRegime::Leo,
            state: sample_observation().observed_state,
            tle: None,
            behavior: BehaviorSignature {
                nominal_rf_mhz: None,
                historical_maneuvers_per_week: 0.0,
                station_keeping_expected: false,
            },
            risk: RiskProfile {
                operator: None,
                mission_class: MissionClass::Unknown,
                criticality: 0.4,
            },
        }
    }

    fn sample_observation() -> Observation {
        Observation {
            object_id: "25544".to_string(),
            observed_state: OrbitalState {
                epoch_unix_seconds: 1_704_067_200,
                position_km: Vector3 {
                    x: 6_790.0,
                    y: 0.0,
                    z: 0.0,
                },
                velocity_km_s: Vector3 {
                    x: 0.0,
                    y: 7.6,
                    z: 0.0,
                },
            },
            rf_mhz: None,
            maneuver_detected: false,
            source: ObservationSource::Catalog,
        }
    }

    fn sample_passive_scan_input() -> sss_passive_scanner::PassiveScanInput {
        sss_passive_scanner::PassiveScanInput {
            window_start_unix_seconds: 1_776_384_000,
            window_end_unix_seconds: 1_776_470_400,
            infra_sites: vec![OpenInfraSiteSeed {
                name: "Seville Solar South".to_string(),
                site_type: SiteType::SolarPlant,
                latitude: 37.3891,
                longitude: -5.9845,
                elevation_m: 12.0,
                timezone: "Europe/Madrid".to_string(),
                country_code: "ES".to_string(),
                criticality: Criticality::High,
                operator_name: "Open Corpus Energy".to_string(),
                observation_radius_km: 18.0,
                source_reference: "openinframap:substation:sev-001".to_string(),
            }],
            adsb_observations: vec![
                AdsbFlightObservation {
                    flight_id: "ADSBX-ES001".to_string(),
                    observed_at_unix_seconds: 1_776_390_000,
                    latitude: 37.40,
                    longitude: -5.99,
                    altitude_m: 620.0,
                    speed_kts: 92.0,
                    vertical_rate_m_s: -1.0,
                },
                AdsbFlightObservation {
                    flight_id: "ADSBX-ES002".to_string(),
                    observed_at_unix_seconds: 1_776_391_800,
                    latitude: 37.395,
                    longitude: -5.986,
                    altitude_m: 540.0,
                    speed_kts: 88.0,
                    vertical_rate_m_s: -0.4,
                },
            ],
            weather_observations: vec![WeatherObservation {
                observed_at_unix_seconds: 1_776_392_000,
                latitude: 37.42,
                longitude: -5.97,
                wind_kph: 58.0,
                hail_probability: 0.62,
                irradiance_drop_ratio: 0.41,
            }],
            fire_smoke_observations: vec![FireSmokeObservation {
                observed_at_unix_seconds: 1_776_394_000,
                latitude: 37.44,
                longitude: -6.00,
                fire_radiative_power: 78.0,
                smoke_density_index: 61.0,
            }],
            orbital_observations: Vec::new(),
            satellite_observations: vec![SatelliteSceneObservation {
                scene_id: "sentinel-2a-t32".to_string(),
                observed_at_unix_seconds: 1_776_398_000,
                latitude: 37.39,
                longitude: -5.985,
                change_score: 0.73,
                cloud_cover_ratio: 0.12,
            }],
            notam_observations: Vec::new(),
        }
    }

    fn sample_passive_seed_record() -> PassiveSeedRecord {
        PassiveSeedRecord {
            seed_key: "osm:way:sev-001".to_string(),
            site_id: Some("site-001".to_string()),
            seed: OpenInfraSiteSeed {
                name: "Seville Solar South".to_string(),
                site_type: SiteType::SolarPlant,
                latitude: 37.3891,
                longitude: -5.9845,
                elevation_m: 12.0,
                timezone: "Europe/Madrid".to_string(),
                country_code: "ES".to_string(),
                criticality: Criticality::High,
                operator_name: "Open Corpus Energy".to_string(),
                observation_radius_km: 18.0,
                source_reference: "osm:way:sev-001".to_string(),
            },
            discovered_at_unix_seconds: 1_776_384_000,
            last_scanned_at_unix_seconds: Some(1_776_470_400),
            last_event_at_unix_seconds: Some(1_776_470_300),
            confidence: 0.78,
            source_count: 3,
            classification_status: PassiveSeedClassificationStatus::Elevated,
            seed_status: PassiveSeedStatus::Monitoring,
            scan_priority: 0.82,
        }
    }

    fn sample_passive_region_target() -> PassiveRegionTarget {
        PassiveRegionTarget {
            region_id: "andalusia-critical-infra".to_string(),
            name: "Andalusia Critical Infra".to_string(),
            south: 37.30,
            west: -6.10,
            north: 37.50,
            east: -5.80,
            site_types: Some(vec![SiteType::SolarPlant, SiteType::Substation]),
            timezone: Some("Europe/Madrid".to_string()),
            country_code: Some("ES".to_string()),
            default_operator_name: Some("Open Corpus Energy".to_string()),
            default_criticality: Some(Criticality::High),
            observation_radius_km: Some(20.0),
            discovery_cadence_seconds: 86_400,
            scan_limit: 25,
            minimum_priority: 0.25,
            enabled: true,
            created_at_unix_seconds: 1_776_384_000,
            updated_at_unix_seconds: 1_776_384_000,
            last_discovered_at_unix_seconds: None,
            last_scheduler_run_at_unix_seconds: None,
        }
    }

    fn sample_passive_region_run_log() -> PassiveRegionRunLog {
        PassiveRegionRunLog {
            run_id: "region-run-1".to_string(),
            request_id: "request-1".to_string(),
            origin: PassiveRunOrigin::Worker,
            started_at_unix_seconds: 1_776_384_000,
            finished_at_unix_seconds: 1_776_384_003,
            duration_ms: 3_000,
            status: PassiveRegionRunStatus::Completed,
            region_ids: vec!["andalusia-critical-infra".to_string()],
            discovery_triggered: true,
            evaluated_region_count: 1,
            discovered_region_count: 1,
            skipped_region_count: 0,
            discovered_seed_count: 3,
            selected_seed_count: 3,
            event_count: 4,
            sources_used: vec!["Weather".to_string(), "FireSmoke".to_string()],
            source_errors: Vec::new(),
            next_run_at_unix_seconds: Some(1_776_387_600),
        }
    }
}
