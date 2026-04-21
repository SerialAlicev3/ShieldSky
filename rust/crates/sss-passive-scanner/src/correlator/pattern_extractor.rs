use std::collections::BTreeMap;

use crate::domain::{PassiveEvent, PatternSignal};

#[must_use]
pub fn extract_patterns(events: &[PassiveEvent]) -> Vec<PatternSignal> {
    let mut grouped = BTreeMap::new();
    for event in events {
        let key = (event.site_id, event.site_name.clone(), event.threat_type);
        grouped
            .entry(key)
            .or_insert_with(Vec::new)
            .push(event.risk_score);
    }

    let mut patterns = grouped
        .into_iter()
        .filter_map(|((site_id, site_name, threat_type), scores)| {
            if scores.len() < 2 {
                return None;
            }
            let total_risk = scores.iter().sum::<f64>();
            let sample_count = scores.iter().fold(0.0, |count, _| count + 1.0);
            let average_risk = total_risk / sample_count;
            Some(PatternSignal {
                site_id,
                site_name: site_name.clone(),
                threat_type,
                recurring_events: scores.len(),
                average_risk,
                summary: format!(
                    "Recurring {:?} pattern detected for {} across {} passive observations",
                    threat_type,
                    site_name,
                    scores.len()
                ),
            })
        })
        .collect::<Vec<_>>();

    patterns.sort_by(|left, right| {
        right
            .average_risk
            .partial_cmp(&left.average_risk)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    patterns
}
