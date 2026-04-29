use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sss_passive_scanner::{PassiveEvent, PassiveThreatType};
use sss_site_registry::SiteType;
use sss_storage::{
    CanonicalPassiveEvent as StoredCanonicalPassiveEvent, PassiveRecommendationReview,
    PassiveRecommendationReviewState, PassiveRegionTarget, PassiveSeedClassificationStatus,
    PassiveSeedRecord, PassiveSeedStatus,
};

use crate::canonical_views::{project_canonical_event, CanonicalEventStatus, RiskDeltaType};
use crate::state::{now_unix_seconds, AppError, AppState};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapCoordinates {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapBbox {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionMapStatus {
    Fresh,
    Due,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteMapType {
    SolarPlant,
    Substation,
    DataCenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteMapClassification {
    Candidate,
    Probable,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteMapSeedStatus {
    Discovered,
    Observed,
    Elevated,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTrend {
    Up,
    Flat,
    Down,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionMapItem {
    pub region_id: String,
    pub name: String,
    pub country_code: Option<String>,
    pub timezone: Option<String>,
    pub enabled: bool,
    pub status: RegionMapStatus,
    pub bbox: MapBbox,
    pub site_types: Vec<SiteMapType>,
    pub observation_radius_km: Option<f64>,
    pub seeds_known: usize,
    pub seeds_observed: usize,
    pub seeds_elevated: usize,
    pub recent_events: usize,
    pub critical_events: usize,
    pub max_priority: MapPriority,
    pub average_observation_confidence: f64,
    pub last_discovered_at_unix_seconds: Option<i64>,
    pub last_scheduler_run_at_unix_seconds: Option<i64>,
    pub next_run_at_unix_seconds: Option<i64>,
    pub discovery_due: bool,
    pub dominant_status: Option<CanonicalEventStatus>,
    pub new_event_count: usize,
    pub recurring_event_count: usize,
    pub escalating_event_count: usize,
    pub cooling_event_count: usize,
    pub active_lease_count: usize,
    pub stale_lease_count: usize,
    pub active_worker_count: usize,
    pub stale_worker_count: usize,
    pub recent_failed_run_count: usize,
    pub recent_partial_run_count: usize,
    pub recent_lease_loss_count: usize,
    pub recommendation_pending_review_count: usize,
    pub recommendation_reviewed_count: usize,
    pub recommendation_applied_count: usize,
    pub recommendation_dismissed_count: usize,
    pub latest_run_status: Option<String>,
    pub latest_run_finished_at_unix_seconds: Option<i64>,
    pub operational_pressure_priority: MapPriority,
    pub operational_summary: String,
    pub narrative_summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionMapResponse {
    pub generated_at_unix_seconds: i64,
    pub regions: Vec<RegionMapItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteMapItem {
    pub site_id: Option<String>,
    pub region_id: String,
    pub seed_key: String,
    pub name: String,
    pub site_type: SiteMapType,
    pub classification_status: SiteMapClassification,
    pub seed_status: SiteMapSeedStatus,
    pub scan_priority: MapPriority,
    pub confidence: f64,
    pub source_count: u32,
    pub coordinates: MapCoordinates,
    pub risk_score: f64,
    pub risk_trend: RiskTrend,
    pub top_canonical_status: Option<CanonicalEventStatus>,
    pub risk_delta_classification: Option<RiskDeltaType>,
    pub risk_delta_explanation: Option<String>,
    pub last_scanned_at_unix_seconds: Option<i64>,
    pub last_event_at_unix_seconds: Option<i64>,
    pub top_event_type: Option<String>,
    pub top_event_severity: Option<MapSeverity>,
    pub top_event_target_epoch_unix_seconds: Option<i64>,
    pub priority_reason: String,
    pub status_reason: String,
    pub overview_path: Option<String>,
    pub narrative_path: Option<String>,
    pub has_recommendation: bool,
    pub recommendation_review_state: Option<PassiveRecommendationReviewState>,
    pub recommendation_reviewed_at_unix_seconds: Option<i64>,
    pub observed: bool,
    pub elevated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteMapResponse {
    pub generated_at_unix_seconds: i64,
    pub region_id: String,
    pub sites: Vec<SiteMapItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMapItem {
    pub event_id: String,
    pub region_id: String,
    pub site_id: String,
    pub site_name: String,
    pub site_type: SiteMapType,
    pub event_type: String,
    pub severity: MapSeverity,
    pub priority: MapPriority,
    pub risk_score: f64,
    pub confidence: f64,
    pub observed_at_unix_seconds: i64,
    pub target_epoch_unix_seconds: Option<i64>,
    pub future: bool,
    pub coordinates: MapCoordinates,
    pub summary: String,
    pub bundle_hash: Option<String>,
    pub manifest_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMapResponse {
    pub generated_at_unix_seconds: i64,
    pub region_id: String,
    pub events: Vec<EventMapItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapSitesQuery {
    pub region_id: String,
    pub site_type: Option<SiteMapType>,
    pub min_priority: Option<f64>,
    pub observed_only: Option<bool>,
    pub elevated_only: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapEventsQuery {
    pub region_id: String,
    pub future_only: Option<bool>,
    pub event_type: Option<String>,
    pub severity: Option<MapSeverity>,
    pub after_unix_seconds: Option<i64>,
    pub before_unix_seconds: Option<i64>,
    pub limit: Option<usize>,
}

pub fn build_region_map_response(state: &AppState) -> Result<RegionMapResponse, AppError> {
    let regions = state.passive_region_targets(1_000, false)?;
    let mut items = Vec::new();
    for region in regions {
        let overview = state.passive_region_overview(&region.region_id, 5, 30)?;
        let canonical_events =
            state.canonical_passive_events(1_000, Some(&region.region_id), None)?;
        items.push(region_map_item(
            state,
            region,
            overview.as_ref(),
            &canonical_events,
        ));
    }
    Ok(RegionMapResponse {
        generated_at_unix_seconds: now_unix_seconds(),
        regions: items,
    })
}

pub fn build_site_map_response(
    state: &AppState,
    query: &MapSitesQuery,
) -> Result<SiteMapResponse, AppError> {
    let region = required_region(state, &query.region_id)?;
    let canonical_by_site = canonical_events_by_site(state.canonical_passive_events(
        10_000,
        Some(&query.region_id),
        None,
    )?);
    let mut sites = Vec::new();
    for record in region_seed_records(state, &region)? {
        if !site_record_matches_query(&record, query) {
            continue;
        }
        let top_event = top_event_for_record(state, &record)?;
        let latest_review = record
            .site_id
            .as_deref()
            .map(|site_id| latest_recommendation_review_for_site(state, site_id))
            .transpose()?
            .flatten();
        let has_recommendation = record
            .site_id
            .as_deref()
            .map(|site_id| site_has_recommendation(state, site_id))
            .transpose()?
            .unwrap_or(false);
        let canonical_focus = record
            .site_id
            .as_ref()
            .and_then(|site_id| canonical_by_site.get(site_id))
            .and_then(|history| site_canonical_focus(history, now_unix_seconds()));
        sites.push(site_map_item(
            record,
            query.region_id.clone(),
            top_event.as_ref(),
            canonical_focus.as_ref(),
            latest_review.as_ref(),
            has_recommendation,
        ));
    }
    sites.sort_by(|left, right| {
        right
            .risk_score
            .partial_cmp(&left.risk_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.confidence.total_cmp(&left.confidence))
    });
    Ok(SiteMapResponse {
        generated_at_unix_seconds: now_unix_seconds(),
        region_id: query.region_id.clone(),
        sites,
    })
}

pub fn build_event_map_response(
    state: &AppState,
    query: &MapEventsQuery,
) -> Result<EventMapResponse, AppError> {
    let region = required_region(state, &query.region_id)?;
    let now = now_unix_seconds();
    let mut events = Vec::new();
    for record in region_seed_records(state, &region)? {
        let Some(site_id) = record.site_id.as_ref() else {
            continue;
        };
        for event in state.passive_events_for_site(site_id, 100)? {
            if event_matches_query(&event, query, now) {
                events.push(event_map_item(&record, query.region_id.clone(), event, now));
            }
        }
    }
    events.sort_by(|left, right| {
        right
            .risk_score
            .partial_cmp(&left.risk_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .observed_at_unix_seconds
                    .cmp(&left.observed_at_unix_seconds)
            })
    });
    events.truncate(query.limit.unwrap_or(100).clamp(1, 500));
    Ok(EventMapResponse {
        generated_at_unix_seconds: now,
        region_id: query.region_id.clone(),
        events,
    })
}

fn required_region(state: &AppState, region_id: &str) -> Result<PassiveRegionTarget, AppError> {
    state
        .passive_region_target(region_id)?
        .ok_or_else(|| AppError::SiteNotFound(format!("passive region not found: {region_id}")))
}

fn region_seed_records(
    state: &AppState,
    region: &PassiveRegionTarget,
) -> Result<Vec<PassiveSeedRecord>, AppError> {
    Ok(state
        .passive_seed_records(10_000)?
        .into_iter()
        .filter(|record| seed_in_region(record, region))
        .collect())
}

fn seed_in_region(record: &PassiveSeedRecord, region: &PassiveRegionTarget) -> bool {
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

#[allow(clippy::too_many_lines)]
fn region_map_item(
    state: &AppState,
    region: PassiveRegionTarget,
    overview: Option<&crate::state::PassiveRegionOverview>,
    canonical_events: &[StoredCanonicalPassiveEvent],
) -> RegionMapItem {
    let discovery_due = overview.is_some_and(|overview| overview.discovery_due);
    let now = now_unix_seconds();
    let canonical_summary = region_canonical_summary(canonical_events, now);
    let operational = region_operational_summary(state, &region.region_id, now)
        .unwrap_or_else(|_| empty_region_operational_summary());
    let narrative_summary = match canonical_summary.dominant_status {
        Some(status) => {
            let base = overview.map_or_else(
                || "Region configured; no passive observations yet.".to_string(),
                |overview| overview.narrative.clone(),
            );
            format!(
                "{base} Canonical state: {:?} (new {}, recurring {}, escalating {}, cooling {}).",
                status,
                canonical_summary.new_event_count,
                canonical_summary.recurring_event_count,
                canonical_summary.escalating_event_count,
                canonical_summary.cooling_event_count
            )
        }
        None => overview.map_or_else(
            || "Region configured; no passive observations yet.".to_string(),
            |overview| overview.narrative.clone(),
        ),
    };
    let operational_summary = format!(
        "{} active leases, {} stale leases, {} active workers, {} stale workers, {} failed runs, {} partial runs, {} lease-loss signals, {} pending recommendation reviews, {} applied, {} dismissed.",
        operational.active_lease_count,
        operational.stale_lease_count,
        operational.active_worker_count,
        operational.stale_worker_count,
        operational.recent_failed_run_count,
        operational.recent_partial_run_count,
        operational.recent_lease_loss_count,
        operational.recommendation_pending_review_count,
        operational.recommendation_applied_count,
        operational.recommendation_dismissed_count
    );
    RegionMapItem {
        region_id: region.region_id,
        name: region.name,
        country_code: region.country_code,
        timezone: region.timezone,
        enabled: region.enabled,
        status: region_status(region.enabled, discovery_due, &operational),
        bbox: MapBbox {
            south: region.south,
            west: region.west,
            north: region.north,
            east: region.east,
        },
        site_types: region
            .site_types
            .clone()
            .unwrap_or_else(|| {
                vec![
                    SiteType::SolarPlant,
                    SiteType::DataCenter,
                    SiteType::Substation,
                ]
            })
            .into_iter()
            .map(site_type_label)
            .collect(),
        observation_radius_km: region.observation_radius_km,
        seeds_known: overview.map_or(0, |overview| overview.seed_count),
        seeds_observed: overview.map_or(0, |overview| overview.observed_seed_count),
        seeds_elevated: overview.map_or(0, |overview| overview.elevated_seed_count),
        recent_events: overview.map_or(0, |overview| overview.recent_event_count),
        critical_events: overview.map_or(0, |overview| overview.critical_event_count),
        max_priority: priority_from_score(
            overview.map_or(0.0, |overview| overview.highest_scan_priority),
        ),
        average_observation_confidence: overview
            .map_or(0.0, |overview| overview.average_observation_confidence),
        last_discovered_at_unix_seconds: region.last_discovered_at_unix_seconds,
        last_scheduler_run_at_unix_seconds: region.last_scheduler_run_at_unix_seconds,
        next_run_at_unix_seconds: region
            .last_discovered_at_unix_seconds
            .map(|last| last.saturating_add(region.discovery_cadence_seconds)),
        discovery_due,
        dominant_status: canonical_summary.dominant_status,
        new_event_count: canonical_summary.new_event_count,
        recurring_event_count: canonical_summary.recurring_event_count,
        escalating_event_count: canonical_summary.escalating_event_count,
        cooling_event_count: canonical_summary.cooling_event_count,
        active_lease_count: operational.active_lease_count,
        stale_lease_count: operational.stale_lease_count,
        active_worker_count: operational.active_worker_count,
        stale_worker_count: operational.stale_worker_count,
        recent_failed_run_count: operational.recent_failed_run_count,
        recent_partial_run_count: operational.recent_partial_run_count,
        recent_lease_loss_count: operational.recent_lease_loss_count,
        recommendation_pending_review_count: operational.recommendation_pending_review_count,
        recommendation_reviewed_count: operational.recommendation_reviewed_count,
        recommendation_applied_count: operational.recommendation_applied_count,
        recommendation_dismissed_count: operational.recommendation_dismissed_count,
        latest_run_status: operational.latest_run_status,
        latest_run_finished_at_unix_seconds: operational.latest_run_finished_at_unix_seconds,
        operational_pressure_priority: priority_from_score(operational.operational_pressure_score),
        operational_summary,
        narrative_summary,
    }
}

fn site_record_matches_query(record: &PassiveSeedRecord, query: &MapSitesQuery) -> bool {
    if let Some(site_type) = query.site_type {
        if site_type_label(record.seed.site_type) != site_type {
            return false;
        }
    }
    if record.scan_priority < query.min_priority.unwrap_or(0.0).clamp(0.0, 1.0) {
        return false;
    }
    if query.observed_only.unwrap_or(false) && record.last_scanned_at_unix_seconds.is_none() {
        return false;
    }
    if query.elevated_only.unwrap_or(false) && !seed_is_elevated(record) {
        return false;
    }
    true
}

fn top_event_for_record(
    state: &AppState,
    record: &PassiveSeedRecord,
) -> Result<Option<PassiveEvent>, AppError> {
    let Some(site_id) = record.site_id.as_ref() else {
        return Ok(None);
    };
    Ok(state
        .passive_events_for_site(site_id, 25)?
        .into_iter()
        .max_by(|left, right| {
            left.risk_score
                .partial_cmp(&right.risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }))
}

fn latest_recommendation_review_for_site(
    state: &AppState,
    site_id: &str,
) -> Result<Option<PassiveRecommendationReview>, AppError> {
    Ok(state
        .passive_site_recommendation_reviews(site_id, 1)
        .map_err(AppError::Storage)?
        .into_iter()
        .next())
}

fn site_has_recommendation(state: &AppState, site_id: &str) -> Result<bool, AppError> {
    Ok(!state
        .passive_site_orchestrator_decisions(site_id, 1)
        .map_err(AppError::Storage)?
        .is_empty())
}

fn site_map_item(
    record: PassiveSeedRecord,
    region_id: String,
    top_event: Option<&PassiveEvent>,
    canonical_focus: Option<&SiteCanonicalFocus>,
    latest_review: Option<&PassiveRecommendationReview>,
    has_recommendation: bool,
) -> SiteMapItem {
    let elevated = seed_is_elevated(&record);
    let priority_reason = seed_priority_reason(&record, top_event);
    let status_reason = seed_status_reason(
        &record,
        top_event,
        canonical_focus,
        latest_review,
        has_recommendation,
    );
    let overview_path = record
        .site_id
        .as_ref()
        .map(|site_id| format!("/v1/passive/sites/{site_id}/overview"));
    let narrative_path = record
        .site_id
        .as_ref()
        .map(|site_id| format!("/v1/passive/sites/{site_id}/narrative?days=30"));
    SiteMapItem {
        site_id: record.site_id.clone(),
        region_id,
        seed_key: record.seed_key,
        name: record.seed.name,
        site_type: site_type_label(record.seed.site_type),
        classification_status: classification_label(record.classification_status),
        seed_status: seed_status_label(record.seed_status),
        scan_priority: priority_from_score(record.scan_priority),
        confidence: record.confidence,
        source_count: record.source_count,
        coordinates: MapCoordinates {
            lat: record.seed.latitude,
            lon: record.seed.longitude,
        },
        risk_score: top_event.map_or(0.0, |event| event.risk_score),
        risk_trend: canonical_focus.map_or(RiskTrend::Flat, |focus| focus.risk_trend),
        top_canonical_status: canonical_focus.map(|focus| focus.status),
        risk_delta_classification: canonical_focus.map(|focus| focus.risk_delta_classification),
        risk_delta_explanation: canonical_focus.map(|focus| focus.risk_delta_explanation.clone()),
        last_scanned_at_unix_seconds: record.last_scanned_at_unix_seconds,
        last_event_at_unix_seconds: record.last_event_at_unix_seconds,
        top_event_type: top_event.map(|event| threat_type_name(event.threat_type)),
        top_event_severity: top_event.map(|event| severity_from_risk(event.risk_score)),
        top_event_target_epoch_unix_seconds: None,
        priority_reason,
        status_reason,
        overview_path,
        narrative_path,
        has_recommendation,
        recommendation_review_state: latest_review.map(|review| review.state),
        recommendation_reviewed_at_unix_seconds: latest_review
            .map(|review| review.decided_at_unix_seconds),
        observed: record.last_scanned_at_unix_seconds.is_some(),
        elevated,
    }
}

fn event_matches_query(event: &PassiveEvent, query: &MapEventsQuery, now: i64) -> bool {
    if query.future_only.unwrap_or(false) && event.observed_at_unix_seconds <= now {
        return false;
    }
    if let Some(event_type) = query.event_type.as_ref() {
        if !threat_type_name(event.threat_type).eq_ignore_ascii_case(event_type) {
            return false;
        }
    }
    if let Some(severity) = query.severity {
        if severity_from_risk(event.risk_score) != severity {
            return false;
        }
    }
    if query
        .after_unix_seconds
        .is_some_and(|after| event.observed_at_unix_seconds < after)
    {
        return false;
    }
    if query
        .before_unix_seconds
        .is_some_and(|before| event.observed_at_unix_seconds > before)
    {
        return false;
    }
    true
}

fn event_map_item(
    record: &PassiveSeedRecord,
    region_id: String,
    event: PassiveEvent,
    now: i64,
) -> EventMapItem {
    EventMapItem {
        event_id: event.event_id.to_string(),
        region_id,
        site_id: event.site_id.to_string(),
        site_name: event.site_name,
        site_type: site_type_label(record.seed.site_type),
        event_type: threat_type_name(event.threat_type),
        severity: severity_from_risk(event.risk_score),
        priority: priority_from_score(event.risk_score),
        risk_score: event.risk_score,
        confidence: record
            .confidence
            .max(event.risk_score * 0.8)
            .clamp(0.0, 0.99),
        observed_at_unix_seconds: event.observed_at_unix_seconds,
        target_epoch_unix_seconds: None,
        future: event.observed_at_unix_seconds > now,
        coordinates: MapCoordinates {
            lat: record.seed.latitude,
            lon: record.seed.longitude,
        },
        summary: event.summary,
        bundle_hash: None,
        manifest_hash: None,
    }
}

fn region_status(
    enabled: bool,
    discovery_due: bool,
    operational: &RegionOperationalSummary,
) -> RegionMapStatus {
    if !enabled
        || operational.stale_lease_count > 0
        || operational.stale_worker_count > 0
        || operational.recent_failed_run_count > 0
        || operational.recent_lease_loss_count > 0
    {
        RegionMapStatus::Degraded
    } else if operational.recent_partial_run_count > 0 || discovery_due {
        RegionMapStatus::Due
    } else {
        RegionMapStatus::Fresh
    }
}

fn site_type_label(site_type: SiteType) -> SiteMapType {
    match site_type {
        SiteType::SolarPlant => SiteMapType::SolarPlant,
        SiteType::Substation => SiteMapType::Substation,
        SiteType::DataCenter => SiteMapType::DataCenter,
    }
}

fn classification_label(status: PassiveSeedClassificationStatus) -> SiteMapClassification {
    match status {
        PassiveSeedClassificationStatus::Discovered => SiteMapClassification::Candidate,
        PassiveSeedClassificationStatus::Observed => SiteMapClassification::Probable,
        PassiveSeedClassificationStatus::Elevated | PassiveSeedClassificationStatus::Strategic => {
            SiteMapClassification::Confirmed
        }
    }
}

fn seed_status_label(status: PassiveSeedStatus) -> SiteMapSeedStatus {
    match status {
        PassiveSeedStatus::New => SiteMapSeedStatus::Discovered,
        PassiveSeedStatus::Monitoring => SiteMapSeedStatus::Observed,
        PassiveSeedStatus::Elevated => SiteMapSeedStatus::Elevated,
        PassiveSeedStatus::Dormant => SiteMapSeedStatus::Stale,
    }
}

fn seed_is_elevated(record: &PassiveSeedRecord) -> bool {
    matches!(
        record.classification_status,
        PassiveSeedClassificationStatus::Elevated | PassiveSeedClassificationStatus::Strategic
    ) || matches!(record.seed_status, PassiveSeedStatus::Elevated)
}

#[derive(Debug, Clone)]
struct SiteCanonicalFocus {
    status: CanonicalEventStatus,
    risk_trend: RiskTrend,
    risk_delta_classification: RiskDeltaType,
    risk_delta_explanation: String,
}

#[derive(Debug, Clone, Copy)]
struct RegionCanonicalSummary {
    dominant_status: Option<CanonicalEventStatus>,
    new_event_count: usize,
    recurring_event_count: usize,
    escalating_event_count: usize,
    cooling_event_count: usize,
}

#[derive(Debug, Clone)]
struct RegionOperationalSummary {
    active_lease_count: usize,
    stale_lease_count: usize,
    active_worker_count: usize,
    stale_worker_count: usize,
    recent_failed_run_count: usize,
    recent_partial_run_count: usize,
    recent_lease_loss_count: usize,
    recommendation_pending_review_count: usize,
    recommendation_reviewed_count: usize,
    recommendation_applied_count: usize,
    recommendation_dismissed_count: usize,
    latest_run_status: Option<String>,
    latest_run_finished_at_unix_seconds: Option<i64>,
    operational_pressure_score: f64,
}

fn canonical_events_by_site(
    events: Vec<StoredCanonicalPassiveEvent>,
) -> BTreeMap<String, Vec<StoredCanonicalPassiveEvent>> {
    let mut grouped = BTreeMap::<String, Vec<StoredCanonicalPassiveEvent>>::new();
    for event in events {
        grouped
            .entry(event.site_id.clone())
            .or_default()
            .push(event);
    }
    grouped
}

fn site_canonical_focus(
    events: &[StoredCanonicalPassiveEvent],
    now_unix_seconds: i64,
) -> Option<SiteCanonicalFocus> {
    let primary = events.iter().max_by(|left, right| {
        left.last_seen_at_unix_seconds
            .cmp(&right.last_seen_at_unix_seconds)
            .then_with(|| left.max_risk_score.total_cmp(&right.max_risk_score))
    })?;
    let projection = project_canonical_event(events, primary, now_unix_seconds);
    Some(SiteCanonicalFocus {
        status: projection.status,
        risk_trend: risk_trend_from_delta(projection.risk_delta.classification),
        risk_delta_classification: projection.risk_delta.classification,
        risk_delta_explanation: projection.risk_delta.explanation,
    })
}

fn region_canonical_summary(
    events: &[StoredCanonicalPassiveEvent],
    now_unix_seconds: i64,
) -> RegionCanonicalSummary {
    let mut summary = RegionCanonicalSummary {
        dominant_status: None,
        new_event_count: 0,
        recurring_event_count: 0,
        escalating_event_count: 0,
        cooling_event_count: 0,
    };

    let mut dominant = None::<(CanonicalEventStatus, f64, i64)>;
    for event in events {
        let projection = project_canonical_event(events, event, now_unix_seconds);
        match projection.status {
            CanonicalEventStatus::New => summary.new_event_count += 1,
            CanonicalEventStatus::Recurring => summary.recurring_event_count += 1,
            CanonicalEventStatus::Escalating => summary.escalating_event_count += 1,
            CanonicalEventStatus::Cooling => summary.cooling_event_count += 1,
        }

        let candidate = (
            projection.status,
            event.max_risk_score,
            event.last_seen_at_unix_seconds,
        );
        if dominant.as_ref().is_none_or(|current| {
            status_rank(candidate.0) > status_rank(current.0)
                || (status_rank(candidate.0) == status_rank(current.0)
                    && (candidate.1 > current.1
                        || (candidate.1.total_cmp(&current.1).is_eq() && candidate.2 > current.2)))
        }) {
            dominant = Some(candidate);
        }
    }

    summary.dominant_status = dominant.map(|candidate| candidate.0);
    summary
}

#[allow(clippy::too_many_lines)]
fn region_operational_summary(
    state: &AppState,
    region_id: &str,
    now_unix_seconds: i64,
) -> Result<RegionOperationalSummary, AppError> {
    let stale_cutoff = now_unix_seconds.saturating_sub(180);
    let run_window_start = now_unix_seconds.saturating_sub(24 * 3_600);
    let region = required_region(state, region_id)?;
    let leases = state.passive_region_leases(500)?;
    let heartbeats = state.passive_worker_heartbeats(500)?;
    let run_logs = state.passive_region_run_logs(50, Some(region_id))?;
    let region_records = state
        .passive_seed_records(10_000)?
        .into_iter()
        .filter(|record| record.site_id.is_some() && seed_in_region(record, &region))
        .collect::<Vec<_>>();

    let active_lease_count = leases
        .iter()
        .filter(|lease| {
            lease.region_id == region_id && lease.expires_at_unix_seconds > now_unix_seconds
        })
        .count();
    let stale_lease_count = leases
        .iter()
        .filter(|lease| {
            lease.region_id == region_id && lease.expires_at_unix_seconds <= now_unix_seconds
        })
        .count();
    let active_worker_count = heartbeats
        .iter()
        .filter(|heartbeat| {
            heartbeat.current_region_id.as_deref() == Some(region_id)
                && heartbeat.last_heartbeat_unix_seconds >= stale_cutoff
        })
        .count();
    let stale_worker_count = heartbeats
        .iter()
        .filter(|heartbeat| {
            heartbeat.current_region_id.as_deref() == Some(region_id)
                && heartbeat.last_heartbeat_unix_seconds < stale_cutoff
        })
        .count();

    let recent_logs = run_logs
        .iter()
        .filter(|run| run.finished_at_unix_seconds >= run_window_start)
        .collect::<Vec<_>>();
    let recent_failed_run_count = recent_logs
        .iter()
        .filter(|run| run.status == sss_storage::PassiveRegionRunStatus::Failed)
        .count();
    let recent_partial_run_count = recent_logs
        .iter()
        .filter(|run| run.status == sss_storage::PassiveRegionRunStatus::Partial)
        .count();
    let recent_lease_loss_count = recent_logs
        .iter()
        .filter(|run| {
            run.source_errors
                .iter()
                .any(|error| error.starts_with("lease_lost:"))
        })
        .count();
    let latest_run = run_logs.first();

    let mut pressure: f64 = 0.0;
    if stale_lease_count > 0 {
        pressure += 0.35;
    }
    if stale_worker_count > 0 {
        pressure += 0.25;
    }
    if recent_failed_run_count > 0 {
        pressure += 0.35;
    }
    if recent_partial_run_count > 0 {
        pressure += 0.2;
    }
    if recent_lease_loss_count > 0 {
        pressure += 0.45;
    }
    if active_lease_count == 0 && active_worker_count == 0 && latest_run.is_some() {
        pressure += 0.15;
    }

    let mut recommendation_pending_review_count = 0;
    let mut recommendation_reviewed_count = 0;
    let mut recommendation_applied_count = 0;
    let mut recommendation_dismissed_count = 0;
    for record in &region_records {
        let Some(site_id) = record.site_id.as_deref() else {
            continue;
        };
        let has_recommendation = site_has_recommendation(state, site_id)?;
        let latest_review = latest_recommendation_review_for_site(state, site_id)?;
        match latest_review.map(|review| review.state) {
            Some(PassiveRecommendationReviewState::Reviewed) => recommendation_reviewed_count += 1,
            Some(PassiveRecommendationReviewState::Applied) => recommendation_applied_count += 1,
            Some(PassiveRecommendationReviewState::Dismissed) => {
                recommendation_dismissed_count += 1;
            }
            None if has_recommendation => recommendation_pending_review_count += 1,
            None => {}
        }
    }
    if recommendation_pending_review_count > 0 {
        pressure += 0.15;
    }

    Ok(RegionOperationalSummary {
        active_lease_count,
        stale_lease_count,
        active_worker_count,
        stale_worker_count,
        recent_failed_run_count,
        recent_partial_run_count,
        recent_lease_loss_count,
        recommendation_pending_review_count,
        recommendation_reviewed_count,
        recommendation_applied_count,
        recommendation_dismissed_count,
        latest_run_status: latest_run.map(|run| format!("{:?}", run.status)),
        latest_run_finished_at_unix_seconds: latest_run.map(|run| run.finished_at_unix_seconds),
        operational_pressure_score: pressure.clamp(0.0, 1.0),
    })
}

fn empty_region_operational_summary() -> RegionOperationalSummary {
    RegionOperationalSummary {
        active_lease_count: 0,
        stale_lease_count: 0,
        active_worker_count: 0,
        stale_worker_count: 0,
        recent_failed_run_count: 0,
        recent_partial_run_count: 0,
        recent_lease_loss_count: 0,
        recommendation_pending_review_count: 0,
        recommendation_reviewed_count: 0,
        recommendation_applied_count: 0,
        recommendation_dismissed_count: 0,
        latest_run_status: None,
        latest_run_finished_at_unix_seconds: None,
        operational_pressure_score: 0.0,
    }
}

fn risk_trend_from_delta(classification: RiskDeltaType) -> RiskTrend {
    match classification {
        RiskDeltaType::Spike | RiskDeltaType::Increase => RiskTrend::Up,
        RiskDeltaType::Decrease => RiskTrend::Down,
        RiskDeltaType::Stable => RiskTrend::Flat,
    }
}

fn status_rank(status: CanonicalEventStatus) -> u8 {
    match status {
        CanonicalEventStatus::Escalating => 4,
        CanonicalEventStatus::New => 3,
        CanonicalEventStatus::Recurring => 2,
        CanonicalEventStatus::Cooling => 1,
    }
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

fn severity_from_risk(risk_score: f64) -> MapSeverity {
    match priority_from_score(risk_score) {
        MapPriority::Critical => MapSeverity::Critical,
        MapPriority::High => MapSeverity::High,
        MapPriority::Medium => MapSeverity::Medium,
        MapPriority::Low => MapSeverity::Low,
    }
}

fn seed_priority_reason(record: &PassiveSeedRecord, top_event: Option<&PassiveEvent>) -> String {
    if let Some(event) = top_event {
        format!(
            "Priority elevated by {} risk at {:.0}% over a {:?} asset.",
            threat_type_name(event.threat_type),
            event.risk_score * 100.0,
            record.seed.site_type
        )
    } else if record.last_scanned_at_unix_seconds.is_some() {
        format!(
            "Monitoring priority {:.0}% driven by {:?} criticality {:?} with confidence {:.0}%.",
            record.scan_priority * 100.0,
            record.seed.site_type,
            record.seed.criticality,
            record.confidence * 100.0
        )
    } else {
        format!(
            "First-pass candidate discovered for {:?} criticality {:?}; scan priority {:.0}%.",
            record.seed.site_type,
            record.seed.criticality,
            record.scan_priority * 100.0
        )
    }
}

fn seed_status_reason(
    record: &PassiveSeedRecord,
    top_event: Option<&PassiveEvent>,
    canonical_focus: Option<&SiteCanonicalFocus>,
    latest_review: Option<&PassiveRecommendationReview>,
    has_recommendation: bool,
) -> String {
    let recommendation_state = recommendation_status_reason(latest_review, has_recommendation);
    if let Some(event) = top_event {
        return format!(
            "Latest event: {} with {:?} severity and observational confidence {:.0}%. {}",
            threat_type_name(event.threat_type),
            severity_from_risk(event.risk_score),
            record.confidence * 100.0,
            recommendation_state
        );
    }
    if let Some(focus) = canonical_focus {
        return format!(
            "Canonical state {:?} with {:?} risk trend. {}",
            focus.status, focus.risk_trend, recommendation_state
        );
    }
    if record.last_scanned_at_unix_seconds.is_some() {
        format!(
            "Observed site under continuous monitoring; no active canonical pressure yet. Sources fused: {}. {}",
            record.source_count,
            recommendation_state
        )
    } else {
        format!(
            "Discovered site candidate awaiting first live scan and narrative enrichment. {recommendation_state}"
        )
    }
}

fn recommendation_status_reason(
    latest_review: Option<&PassiveRecommendationReview>,
    has_recommendation: bool,
) -> String {
    match latest_review.map(|review| review.state) {
        Some(PassiveRecommendationReviewState::Reviewed) => {
            "Recommendation reviewed by operator.".to_string()
        }
        Some(PassiveRecommendationReviewState::Applied) => {
            "Recommendation applied by operator.".to_string()
        }
        Some(PassiveRecommendationReviewState::Dismissed) => {
            "Recommendation dismissed by operator.".to_string()
        }
        None if has_recommendation => "Recommendation pending operator review.".to_string(),
        None => "No operator recommendation recorded yet.".to_string(),
    }
}

fn threat_type_name(threat_type: PassiveThreatType) -> String {
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
