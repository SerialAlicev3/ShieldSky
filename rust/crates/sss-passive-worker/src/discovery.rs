use sss_api::state::PassiveRegionRunRequest;

use crate::config::WorkerConfig;

#[must_use]
pub fn discovery_enabled(config: &WorkerConfig) -> bool {
    config.enable_discovery
}

#[must_use]
pub fn build_region_request(
    region_id: String,
    config: &WorkerConfig,
    force_discovery: bool,
) -> PassiveRegionRunRequest {
    PassiveRegionRunRequest {
        region_ids: Some(vec![region_id]),
        force_discovery: Some(force_discovery),
        dry_run: Some(config.dry_run || !config.scan_effectively_enabled()),
        window_hours: Some(config.window_hours),
        include_adsb: Some(config.feeds.include_adsb),
        include_weather: Some(config.feeds.include_weather),
        include_fire_smoke: Some(config.feeds.include_fire_smoke),
    }
}
