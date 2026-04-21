use serde::{Deserialize, Serialize};

use crate::canonical_views::{
    project_canonical_event, CanonicalEventStatus, RiskDelta, RiskDeltaType, TemporalPhase,
};
use crate::state::{now_unix_seconds, AppError, AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct SemanticTimelineQuery {
    pub limit: Option<usize>,
    pub window_hours: Option<u64>,
    pub status: Option<CanonicalEventStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTimelineScope {
    Dashboard,
    Site,
    Region,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticTimelineEntry {
    pub canonical_event_id: String,
    pub site_id: String,
    pub site_name: String,
    pub event_type: String,
    pub status: CanonicalEventStatus,
    pub status_summary: String,
    pub temporal_phase: TemporalPhase,
    pub operational_readout: String,
    pub risk_score: f64,
    pub confidence: f64,
    pub risk_delta: RiskDelta,
    pub support_count: usize,
    pub first_observed_at_unix_seconds: i64,
    pub last_observed_at_unix_seconds: i64,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticTimelineResponse {
    pub generated_at_unix_seconds: i64,
    pub scope: SemanticTimelineScope,
    pub scope_id: String,
    pub window_hours: u64,
    pub entry_count: usize,
    pub dominant_status: Option<CanonicalEventStatus>,
    pub entries: Vec<SemanticTimelineEntry>,
}

pub fn build_site_semantic_timeline(
    state: &AppState,
    site_id: &str,
    query: &SemanticTimelineQuery,
) -> Result<SemanticTimelineResponse, AppError> {
    let generated_at = now_unix_seconds();
    let window_hours = query.window_hours.unwrap_or(24 * 30).clamp(1, 24 * 365);
    let after_unix_seconds =
        generated_at.saturating_sub(i64::try_from(window_hours).unwrap_or(i64::MAX) * 3_600);
    let limit = query.limit.unwrap_or(20).clamp(1, 200);

    let mut history = state.canonical_passive_events(limit.max(200), None, Some(site_id))?;
    history.retain(|event| event.last_seen_at_unix_seconds >= after_unix_seconds);

    let entries = semantic_entries(&history, generated_at, limit, query.status);
    Ok(SemanticTimelineResponse {
        generated_at_unix_seconds: generated_at,
        scope: SemanticTimelineScope::Site,
        scope_id: site_id.to_string(),
        window_hours,
        entry_count: entries.len(),
        dominant_status: dominant_status(&entries),
        entries,
    })
}

pub fn build_region_semantic_timeline(
    state: &AppState,
    region_id: &str,
    query: &SemanticTimelineQuery,
) -> Result<SemanticTimelineResponse, AppError> {
    let generated_at = now_unix_seconds();
    let window_hours = query.window_hours.unwrap_or(24 * 30).clamp(1, 24 * 365);
    let after_unix_seconds =
        generated_at.saturating_sub(i64::try_from(window_hours).unwrap_or(i64::MAX) * 3_600);
    let limit = query.limit.unwrap_or(30).clamp(1, 300);

    let mut history = state.canonical_passive_events(limit.max(300), Some(region_id), None)?;
    history.retain(|event| event.last_seen_at_unix_seconds >= after_unix_seconds);

    let entries = semantic_entries(&history, generated_at, limit, query.status);
    Ok(SemanticTimelineResponse {
        generated_at_unix_seconds: generated_at,
        scope: SemanticTimelineScope::Region,
        scope_id: region_id.to_string(),
        window_hours,
        entry_count: entries.len(),
        dominant_status: dominant_status(&entries),
        entries,
    })
}

pub fn build_dashboard_semantic_timeline(
    state: &AppState,
    region_id: Option<&str>,
    query: &SemanticTimelineQuery,
) -> Result<SemanticTimelineResponse, AppError> {
    let generated_at = now_unix_seconds();
    let window_hours = query.window_hours.unwrap_or(24 * 14).clamp(1, 24 * 365);
    let after_unix_seconds =
        generated_at.saturating_sub(i64::try_from(window_hours).unwrap_or(i64::MAX) * 3_600);
    let limit = query.limit.unwrap_or(12).clamp(1, 200);

    let mut history = state.canonical_passive_events(limit.max(300), region_id, None)?;
    history.retain(|event| event.last_seen_at_unix_seconds >= after_unix_seconds);

    let entries = semantic_entries(&history, generated_at, limit, query.status);
    Ok(SemanticTimelineResponse {
        generated_at_unix_seconds: generated_at,
        scope: SemanticTimelineScope::Dashboard,
        scope_id: region_id.unwrap_or("global").to_string(),
        window_hours,
        entry_count: entries.len(),
        dominant_status: dominant_status(&entries),
        entries,
    })
}

fn semantic_entries(
    history: &[sss_storage::CanonicalPassiveEvent],
    generated_at: i64,
    limit: usize,
    status_filter: Option<CanonicalEventStatus>,
) -> Vec<SemanticTimelineEntry> {
    let mut entries = history
        .iter()
        .map(|event| {
            let projection = project_canonical_event(history, event, generated_at);
            SemanticTimelineEntry {
                canonical_event_id: event.canonical_id.clone(),
                site_id: event.site_id.clone(),
                site_name: event.site_name.clone(),
                event_type: threat_type_name(event.threat_type),
                status: projection.status,
                status_summary: projection.status_summary,
                temporal_phase: projection.temporal_phase,
                operational_readout: projection.operational_readout,
                risk_score: event.max_risk_score,
                confidence: event.avg_confidence,
                risk_delta: projection.risk_delta,
                support_count: usize::try_from(event.occurrence_count).unwrap_or(usize::MAX),
                first_observed_at_unix_seconds: event.first_seen_at_unix_seconds,
                last_observed_at_unix_seconds: event.last_seen_at_unix_seconds,
                summary: event.summary.clone(),
            }
        })
        .collect::<Vec<_>>();

    if let Some(status_filter) = status_filter {
        entries.retain(|entry| entry.status == status_filter);
    }

    entries.sort_by(|left, right| {
        right
            .last_observed_at_unix_seconds
            .cmp(&left.last_observed_at_unix_seconds)
            .then_with(|| status_rank(right.status).cmp(&status_rank(left.status)))
            .then_with(|| right.risk_score.total_cmp(&left.risk_score))
    });
    entries.truncate(limit);
    entries
}

fn dominant_status(entries: &[SemanticTimelineEntry]) -> Option<CanonicalEventStatus> {
    entries
        .iter()
        .max_by(|left, right| {
            status_rank(left.status)
                .cmp(&status_rank(right.status))
                .then_with(|| left.risk_score.total_cmp(&right.risk_score))
                .then_with(|| {
                    left.last_observed_at_unix_seconds
                        .cmp(&right.last_observed_at_unix_seconds)
                })
        })
        .map(|entry| entry.status)
}

fn status_rank(status: CanonicalEventStatus) -> u8 {
    match status {
        CanonicalEventStatus::Escalating => 4,
        CanonicalEventStatus::New => 3,
        CanonicalEventStatus::Recurring => 2,
        CanonicalEventStatus::Cooling => 1,
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

#[allow(dead_code)]
fn _delta_classification_name(classification: RiskDeltaType) -> &'static str {
    match classification {
        RiskDeltaType::Spike => "spike",
        RiskDeltaType::Increase => "increase",
        RiskDeltaType::Decrease => "decrease",
        RiskDeltaType::Stable => "stable",
    }
}
