use std::collections::BTreeMap;

use crate::domain::{PassiveEvent, PassiveRiskRecord, PassiveSiteProfile};

#[must_use]
pub fn build_risk_history(
    sites: &[PassiveSiteProfile],
    events: &[PassiveEvent],
    window_start_unix_seconds: i64,
    window_end_unix_seconds: i64,
) -> Vec<PassiveRiskRecord> {
    let mut events_by_site = BTreeMap::new();
    for event in events {
        events_by_site
            .entry(event.site_id)
            .or_insert_with(Vec::new)
            .push(event);
    }

    let mut risk_history = Vec::new();
    for site in sites {
        let site_events = events_by_site
            .remove(&site.site.site_id)
            .unwrap_or_default();
        let cumulative_risk = site_events
            .iter()
            .map(|event| event.risk_score)
            .sum::<f64>()
            .min(1.0);
        let peak_risk = site_events
            .iter()
            .map(|event| event.risk_score)
            .fold(0.0, f64::max);

        let mut dominant_threats = site_events
            .iter()
            .map(|event| event.threat_type)
            .collect::<Vec<_>>();
        dominant_threats.sort();
        dominant_threats.dedup();

        risk_history.push(PassiveRiskRecord {
            site_id: site.site.site_id,
            site_name: site.site.name.clone(),
            window_start_unix_seconds,
            window_end_unix_seconds,
            cumulative_risk,
            peak_risk,
            event_count: site_events.len(),
            dominant_threats,
        });
    }

    risk_history.sort_by(|left, right| {
        right
            .cumulative_risk
            .partial_cmp(&left.cumulative_risk)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    risk_history
}
