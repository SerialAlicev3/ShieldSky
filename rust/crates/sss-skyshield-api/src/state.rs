use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use sss_response::PolicyEvaluation;
use sss_response::ResponseDispatch;
use sss_site_registry::{ProtectedAsset, ProtectedZone, Site, SiteExposureProfile};
use sss_skyshield::{
    AerialTrack, AuthorizationGate, GateStatus, IncidentStatus, ResponseAction, ResponsePolicy,
    SiteIncident, SiteOperationalDecision, SkySignal, SkyThreatAssessment,
};

#[derive(Debug, Clone)]
pub struct AppState {
    data: Arc<RwLock<StateData>>,
    path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum StateError {
    Io(std::io::Error),
    Serde(serde_json::Error),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for StateError {}

impl From<std::io::Error> for StateError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for StateError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentCoreMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranked_event_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct StateData {
    sites: Vec<Site>,
    zones: Vec<ProtectedZone>,
    assets: Vec<ProtectedAsset>,
    exposure_profiles: Vec<SiteExposureProfile>,
    sky_signals: Vec<SkySignal>,
    aerial_tracks: Vec<AerialTrack>,
    incidents: Vec<SiteIncident>,
    #[serde(default)]
    incident_assessments: Vec<IncidentAssessmentRecord>,
    authorization_gates: Vec<AuthorizationGate>,
    response_dispatches: Vec<ResponseDispatch>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncidentAssessmentRecord {
    pub incident_id: Uuid,
    pub threat_assessment: SkyThreatAssessment,
    pub operational_decision: SiteOperationalDecision,
    pub response_policy: ResponsePolicy,
    pub policy_evaluation: PolicyEvaluation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranked_event_id: Option<String>,
}

pub type IncidentDroneDefenseArtifacts = IncidentAssessmentRecord;

impl IncidentAssessmentRecord {
    #[must_use]
    pub fn core_metadata(&self) -> IncidentCoreMetadata {
        IncidentCoreMetadata {
            bundle_hash: self.bundle_hash.clone(),
            manifest_hash: self.manifest_hash.clone(),
            ranked_event_id: self.ranked_event_id.clone(),
        }
    }

    #[must_use]
    pub fn with_core_metadata(mut self, core_metadata: IncidentCoreMetadata) -> Self {
        self.bundle_hash = core_metadata.bundle_hash;
        self.manifest_hash = core_metadata.manifest_hash;
        self.ranked_event_id = core_metadata.ranked_event_id;
        self
    }

    fn merge_core_metadata(&mut self, core_metadata: IncidentCoreMetadata) {
        if let Some(bundle_hash) = core_metadata.bundle_hash {
            self.bundle_hash = Some(bundle_hash);
        }
        if let Some(manifest_hash) = core_metadata.manifest_hash {
            self.manifest_hash = Some(manifest_hash);
        }
        if let Some(ranked_event_id) = core_metadata.ranked_event_id {
            self.ranked_event_id = Some(ranked_event_id);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteOverview {
    pub site: Site,
    pub zones: Vec<ProtectedZone>,
    pub assets: Vec<ProtectedAsset>,
    pub exposure_profile: Option<SiteExposureProfile>,
    pub recent_signals: Vec<SkySignal>,
    pub active_tracks: Vec<AerialTrack>,
    pub open_incidents: Vec<SiteIncident>,
    pub authorization_gates: Vec<AuthorizationGate>,
    pub response_dispatches: Vec<ResponseDispatch>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorizeResponseRequest {
    pub action_type: ResponseAction,
    pub approved_by: String,
    pub approval_basis: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::in_memory()
    }
}

impl AppState {
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            data: Arc::new(RwLock::new(StateData::default())),
            path: None,
        }
    }

    pub fn file_backed(path: impl Into<PathBuf>) -> Result<Self, StateError> {
        let path = path.into();
        let data = if path.exists() {
            let payload = std::fs::read_to_string(&path)?;
            serde_json::from_str(&payload)?
        } else {
            StateData::default()
        };
        Ok(Self {
            data: Arc::new(RwLock::new(data)),
            path: Some(path),
        })
    }

    pub fn create_site(&self, site: Site) -> Result<Site, StateError> {
        self.mutate(|data| {
            data.sites.retain(|item| item.site_id != site.site_id);
            data.sites.push(site.clone());
            site
        })
    }

    #[must_use]
    pub fn sites(&self) -> Vec<Site> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .sites
            .clone()
    }

    #[must_use]
    pub fn site(&self, site_id: Uuid) -> Option<Site> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .sites
            .iter()
            .find(|site| site.site_id == site_id)
            .cloned()
    }

    pub fn create_zone(
        &self,
        site_id: Uuid,
        mut zone: ProtectedZone,
    ) -> Result<ProtectedZone, StateError> {
        zone.site_id = site_id;
        self.mutate(|data| {
            data.zones.retain(|item| item.zone_id != zone.zone_id);
            data.zones.push(zone.clone());
            zone
        })
    }

    pub fn create_asset(
        &self,
        site_id: Uuid,
        mut asset: ProtectedAsset,
    ) -> Result<ProtectedAsset, StateError> {
        asset.site_id = site_id;
        self.mutate(|data| {
            data.assets.retain(|item| item.asset_id != asset.asset_id);
            data.assets.push(asset.clone());
            asset
        })
    }

    pub fn upsert_exposure_profile(
        &self,
        site_id: Uuid,
        mut profile: SiteExposureProfile,
    ) -> Result<SiteExposureProfile, StateError> {
        profile.site_id = site_id;
        self.mutate(|data| {
            data.exposure_profiles
                .retain(|item| item.site_id != profile.site_id);
            data.exposure_profiles.push(profile.clone());
            profile
        })
    }

    #[must_use]
    pub fn site_overview(&self, site_id: Uuid) -> Option<SiteOverview> {
        let data = self.data.read().expect("state lock should not be poisoned");
        let site = data
            .sites
            .iter()
            .find(|site| site.site_id == site_id)
            .cloned()?;
        Some(SiteOverview {
            site,
            zones: data
                .zones
                .iter()
                .filter(|zone| zone.site_id == site_id)
                .cloned()
                .collect(),
            assets: data
                .assets
                .iter()
                .filter(|asset| asset.site_id == site_id)
                .cloned()
                .collect(),
            exposure_profile: data
                .exposure_profiles
                .iter()
                .find(|profile| profile.site_id == site_id)
                .cloned(),
            recent_signals: data
                .sky_signals
                .iter()
                .filter(|signal| signal.site_id == site_id)
                .cloned()
                .collect(),
            active_tracks: data
                .aerial_tracks
                .iter()
                .filter(|track| track.site_id == site_id)
                .cloned()
                .collect(),
            open_incidents: data
                .incidents
                .iter()
                .filter(|incident| {
                    incident.site_id == site_id && incident.status != IncidentStatus::Closed
                })
                .cloned()
                .collect(),
            authorization_gates: data
                .authorization_gates
                .iter()
                .filter(|gate| gate.site_id == site_id)
                .cloned()
                .collect(),
            response_dispatches: data
                .response_dispatches
                .iter()
                .filter(|dispatch| {
                    data.incidents.iter().any(|incident| {
                        incident.incident_id == dispatch.incident_id && incident.site_id == site_id
                    })
                })
                .cloned()
                .collect(),
        })
    }

    pub fn create_signal(
        &self,
        site_id: Uuid,
        mut signal: SkySignal,
    ) -> Result<SkySignal, StateError> {
        signal.site_id = site_id;
        self.mutate(|data| {
            data.sky_signals
                .retain(|item| item.signal_id != signal.signal_id);
            data.sky_signals.push(signal.clone());
            signal
        })
    }

    #[must_use]
    pub fn site_signals(&self, site_id: Uuid) -> Vec<SkySignal> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .sky_signals
            .iter()
            .filter(|signal| signal.site_id == site_id)
            .cloned()
            .collect()
    }

    pub fn create_track(
        &self,
        site_id: Uuid,
        mut track: AerialTrack,
    ) -> Result<AerialTrack, StateError> {
        track.site_id = site_id;
        self.mutate(|data| {
            data.aerial_tracks
                .retain(|item| item.track_id != track.track_id);
            data.aerial_tracks.push(track.clone());
            track
        })
    }

    #[must_use]
    pub fn site_tracks(&self, site_id: Uuid) -> Vec<AerialTrack> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .aerial_tracks
            .iter()
            .filter(|track| track.site_id == site_id)
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn track(&self, site_id: Uuid, track_id: Uuid) -> Option<AerialTrack> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .aerial_tracks
            .iter()
            .find(|track| track.site_id == site_id && track.track_id == track_id)
            .cloned()
    }

    pub fn create_incident(&self, incident: SiteIncident) -> Result<SiteIncident, StateError> {
        self.mutate(|data| {
            data.incidents
                .retain(|item| item.incident_id != incident.incident_id);
            data.incidents.push(incident.clone());
            incident
        })
    }

    pub fn store_incident_assessment(
        &self,
        record: IncidentAssessmentRecord,
    ) -> Result<IncidentAssessmentRecord, StateError> {
        self.mutate(|data| {
            data.incident_assessments
                .retain(|item| item.incident_id != record.incident_id);
            data.incident_assessments.push(record.clone());
            record
        })
    }

    pub fn store_incident_drone_defense_artifacts(
        &self,
        artifacts: IncidentDroneDefenseArtifacts,
    ) -> Result<IncidentDroneDefenseArtifacts, StateError> {
        self.store_incident_assessment(artifacts)
    }

    #[must_use]
    pub fn site_incidents(&self, site_id: Uuid) -> Vec<SiteIncident> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .incidents
            .iter()
            .filter(|incident| incident.site_id == site_id)
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn incident(&self, incident_id: Uuid) -> Option<SiteIncident> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .incidents
            .iter()
            .find(|incident| incident.incident_id == incident_id)
            .cloned()
    }

    #[must_use]
    pub fn incident_assessment(&self, incident_id: Uuid) -> Option<IncidentAssessmentRecord> {
        self.data
            .read()
            .expect("state lock should not be poisoned")
            .incident_assessments
            .iter()
            .find(|record| record.incident_id == incident_id)
            .cloned()
    }

    #[must_use]
    pub fn incident_drone_defense_artifacts(
        &self,
        incident_id: Uuid,
    ) -> Option<IncidentDroneDefenseArtifacts> {
        self.incident_assessment(incident_id)
    }

    pub fn update_incident_drone_defense_core_metadata(
        &self,
        incident_id: Uuid,
        core_metadata: IncidentCoreMetadata,
    ) -> Result<Option<IncidentDroneDefenseArtifacts>, StateError> {
        self.mutate(|data| {
            let record = data
                .incident_assessments
                .iter_mut()
                .find(|record| record.incident_id == incident_id)?;
            record.merge_core_metadata(core_metadata);
            Some(record.clone())
        })
    }

    pub fn update_incident_status(
        &self,
        incident_id: Uuid,
        status: IncidentStatus,
    ) -> Result<Option<SiteIncident>, StateError> {
        self.mutate(|data| {
            let incident = data
                .incidents
                .iter_mut()
                .find(|incident| incident.incident_id == incident_id)?;
            incident.status = status;
            incident.updated_at_unix_seconds = now_unix_seconds();
            Some(incident.clone())
        })
    }

    pub fn authorize_response(
        &self,
        incident_id: Uuid,
        request: AuthorizeResponseRequest,
    ) -> Result<Option<AuthorizationGate>, StateError> {
        let Some(incident) = self.incident(incident_id) else {
            return Ok(None);
        };
        let gate = AuthorizationGate {
            gate_id: Uuid::new_v4(),
            incident_id,
            site_id: incident.site_id,
            action_type: request.action_type,
            approved_by: request.approved_by,
            approved_at_unix_seconds: now_unix_seconds(),
            approval_basis: request.approval_basis,
            status: GateStatus::Approved,
        };
        self.mutate(|data| {
            data.authorization_gates.push(gate.clone());
            if let Some(incident) = data
                .incidents
                .iter_mut()
                .find(|incident| incident.incident_id == incident_id)
            {
                incident.status = IncidentStatus::ResponseAuthorized;
                incident.updated_at_unix_seconds = now_unix_seconds();
            }
            gate
        })
        .map(Some)
    }

    pub fn record_response_dispatch(
        &self,
        dispatch: ResponseDispatch,
    ) -> Result<ResponseDispatch, StateError> {
        self.mutate(|data| {
            data.response_dispatches
                .retain(|item| item.dispatch_id != dispatch.dispatch_id);
            data.response_dispatches.push(dispatch.clone());
            dispatch
        })
    }

    #[must_use]
    pub fn response_dispatches(&self, incident_id: Uuid) -> Option<Vec<ResponseDispatch>> {
        let _incident = self.incident(incident_id)?;
        let data = self.data.read().expect("state lock should not be poisoned");
        Some(
            data.response_dispatches
                .iter()
                .filter(|dispatch| dispatch.incident_id == incident_id)
                .cloned()
                .collect(),
        )
    }

    #[must_use]
    pub fn response_audit(&self, incident_id: Uuid) -> Option<Vec<AuthorizationGate>> {
        let _incident = self.incident(incident_id)?;
        let data = self.data.read().expect("state lock should not be poisoned");
        Some(
            data.authorization_gates
                .iter()
                .filter(|gate| gate.incident_id == incident_id)
                .cloned()
                .collect(),
        )
    }

    fn mutate<T>(&self, f: impl FnOnce(&mut StateData) -> T) -> Result<T, StateError> {
        let mut data = self
            .data
            .write()
            .expect("state lock should not be poisoned");
        let result = f(&mut data);
        self.persist(&data)?;
        Ok(result)
    }

    fn persist(&self, data: &StateData) -> Result<(), StateError> {
        let Some(path) = self.path.as_ref() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            ensure_dir(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(data)?)?;
        Ok(())
    }
}

fn ensure_dir(path: &Path) -> Result<(), StateError> {
    if !path.as_os_str().is_empty() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{AppState, IncidentAssessmentRecord, IncidentCoreMetadata, StateData};
    use uuid::Uuid;

    use sss_response::PolicyEvaluation;
    use sss_skyshield::{
        EscalationPolicy, RecipientRole, ResponseAction, ResponseOption, ResponsePolicy, Severity,
        SiteOperationalDecision, SkyThreatAssessment, ThreatType,
    };

    #[test]
    fn legacy_state_without_assessments_still_deserializes() {
        let payload = r#"{
            "sites": [],
            "zones": [],
            "assets": [],
            "exposure_profiles": [],
            "sky_signals": [],
            "aerial_tracks": [],
            "incidents": [],
            "authorization_gates": [],
            "response_dispatches": []
        }"#;

        let data: StateData = serde_json::from_str(payload).expect("legacy state should load");

        assert!(data.incident_assessments.is_empty());
    }

    #[test]
    fn stores_and_reads_drone_defense_artifacts_by_incident_id() {
        let state = AppState::in_memory();
        let record = sample_assessment_record();

        state
            .store_incident_drone_defense_artifacts(record.clone())
            .expect("artifacts should store");

        assert_eq!(
            state
                .incident_drone_defense_artifacts(record.incident_id)
                .expect("artifacts should exist"),
            record
        );
    }

    #[test]
    fn updates_core_metadata_incrementally() {
        let state = AppState::in_memory();
        let record = sample_assessment_record().with_core_metadata(IncidentCoreMetadata {
            bundle_hash: Some("bundle-1".to_string()),
            manifest_hash: None,
            ranked_event_id: None,
        });

        state
            .store_incident_drone_defense_artifacts(record.clone())
            .expect("artifacts should store");

        let updated = state
            .update_incident_drone_defense_core_metadata(
                record.incident_id,
                IncidentCoreMetadata {
                    bundle_hash: None,
                    manifest_hash: Some("manifest-1".to_string()),
                    ranked_event_id: Some("ranked-event-1".to_string()),
                },
            )
            .expect("metadata update should succeed")
            .expect("artifacts should exist");

        assert_eq!(
            updated.core_metadata(),
            IncidentCoreMetadata {
                bundle_hash: Some("bundle-1".to_string()),
                manifest_hash: Some("manifest-1".to_string()),
                ranked_event_id: Some("ranked-event-1".to_string()),
            }
        );
    }

    fn sample_assessment_record() -> IncidentAssessmentRecord {
        let site_id = Uuid::new_v4();
        IncidentAssessmentRecord {
            incident_id: Uuid::new_v4(),
            threat_assessment: SkyThreatAssessment {
                assessment_id: Uuid::new_v4(),
                site_id,
                threat_type: ThreatType::UnauthorizedOverflight,
                confidence: 0.94,
                impact_probability: 0.83,
                impact_window_seconds: 180,
                affected_zones: vec![Uuid::new_v4()],
                affected_assets: vec![Uuid::new_v4()],
                explanation: "high-confidence drone track near perimeter".to_string(),
                predicted_path_summary: "north-east vector toward critical zone".to_string(),
            },
            operational_decision: SiteOperationalDecision {
                decision_id: Uuid::new_v4(),
                site_id,
                severity: Severity::High,
                priority: 9,
                recommendation: "Escalate and require human authorization".to_string(),
                recipients: vec![RecipientRole::DroneDefenseCoordinator],
                escalation_policy: EscalationPolicy::HumanAuthorizationRequired,
                authorized_response_options: vec![ResponseOption::DispatchDroneIntercept],
            },
            response_policy: ResponsePolicy {
                policy_id: Uuid::new_v4(),
                site_id,
                threat_type: ThreatType::UnauthorizedOverflight,
                minimum_confidence: 0.8,
                minimum_severity: Severity::Medium,
                allowed_actions: vec![ResponseAction::DispatchDroneIntercept],
                requires_human_authorization: true,
                requires_external_authority: false,
            },
            policy_evaluation: PolicyEvaluation {
                allowed: true,
                requires_human_authorization: true,
                requires_external_authority: false,
                reason: "policy thresholds satisfied".to_string(),
            },
            bundle_hash: None,
            manifest_hash: None,
            ranked_event_id: None,
        }
    }
}
