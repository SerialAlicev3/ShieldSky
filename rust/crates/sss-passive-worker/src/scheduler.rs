use sss_storage::PassiveRegionTarget;

#[must_use]
pub fn should_run(region: &PassiveRegionTarget, now_unix_seconds: i64) -> bool {
    if !region.enabled {
        return false;
    }
    next_run_at(region).is_none_or(|timestamp| now_unix_seconds >= timestamp)
}

#[must_use]
pub fn next_run_at(region: &PassiveRegionTarget) -> Option<i64> {
    region
        .last_scheduler_run_at_unix_seconds
        .or(region.last_discovered_at_unix_seconds)
        .map(|timestamp| timestamp.saturating_add(region.discovery_cadence_seconds.max(1)))
}

#[must_use]
pub fn compute_next_run(region: &PassiveRegionTarget, now_unix_seconds: i64) -> i64 {
    now_unix_seconds.saturating_add(region.discovery_cadence_seconds.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sss_site_registry::SiteType;

    #[test]
    fn should_run_when_no_prior_schedule_exists() {
        let region = target(None, None, true);
        assert!(should_run(&region, 1_000));
    }

    #[test]
    fn should_not_run_before_next_due_time() {
        let region = target(Some(1_000), None, true);
        assert!(!should_run(&region, 1_100));
    }

    #[test]
    fn should_run_after_next_due_time() {
        let region = target(Some(1_000), None, true);
        assert!(should_run(&region, 4_700));
    }

    fn target(
        last_scheduler_run_at_unix_seconds: Option<i64>,
        last_discovered_at_unix_seconds: Option<i64>,
        enabled: bool,
    ) -> PassiveRegionTarget {
        PassiveRegionTarget {
            region_id: "region-test".to_string(),
            name: "Region Test".to_string(),
            south: 38.0,
            west: -9.0,
            north: 39.0,
            east: -8.0,
            site_types: Some(vec![SiteType::SolarPlant]),
            timezone: Some("Europe/Lisbon".to_string()),
            country_code: Some("PT".to_string()),
            default_operator_name: Some("SSS Passive".to_string()),
            default_criticality: None,
            observation_radius_km: Some(25.0),
            discovery_cadence_seconds: 3_600,
            scan_limit: 10,
            minimum_priority: 0.0,
            enabled,
            created_at_unix_seconds: 0,
            updated_at_unix_seconds: 0,
            last_discovered_at_unix_seconds,
            last_scheduler_run_at_unix_seconds,
        }
    }
}
