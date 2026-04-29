use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct WorkerConfig {
    pub storage_path: PathBuf,
    pub worker_id: String,
    pub poll_interval_seconds: u64,
    pub retry_interval_seconds: u64,
    pub max_parallel_regions: usize,
    pub max_regions_per_cycle: usize,
    pub max_runtime_seconds: u64,
    pub lease_ttl_seconds: u64,
    pub heartbeat_retention_seconds: u64,
    pub window_hours: u64,
    pub max_scan_limit_per_region: usize,
    pub max_observation_radius_km: f64,
    pub enable_discovery: bool,
    pub enable_scan: bool,
    pub feeds: WorkerFeedConfig,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerFeedConfig {
    pub include_adsb: bool,
    pub include_weather: bool,
    pub include_fire_smoke: bool,
}

impl WorkerConfig {
    #[must_use]
    pub fn from_env() -> Self {
        let max_runtime_seconds = u64_env("SSS_WORKER_MAX_RUNTIME", 20).max(1);
        let max_parallel_regions = usize_env("SSS_WORKER_MAX_PARALLEL", 4).max(1);
        Self {
            storage_path: path_env("SSS_STORAGE_PATH", "data/sss-api.sqlite"),
            worker_id: string_env("SSS_WORKER_ID").unwrap_or_else(default_worker_id),
            poll_interval_seconds: u64_env("SSS_WORKER_POLL_SECONDS", 30).max(1),
            retry_interval_seconds: u64_env("SSS_WORKER_RETRY_SECONDS", 300).max(1),
            max_parallel_regions,
            max_regions_per_cycle: usize_env(
                "SSS_WORKER_MAX_REGIONS_PER_CYCLE",
                max_parallel_regions.saturating_mul(2),
            )
            .max(max_parallel_regions),
            max_runtime_seconds,
            lease_ttl_seconds: u64_env("SSS_WORKER_LEASE_TTL_SECONDS", 120)
                .max(max_runtime_seconds.saturating_add(1)),
            heartbeat_retention_seconds: u64_env("SSS_WORKER_HEARTBEAT_RETENTION_SECONDS", 86_400)
                .max(1),
            window_hours: u64_env("SSS_WORKER_WINDOW_HOURS", 24).clamp(1, 168),
            max_scan_limit_per_region: usize_env("SSS_WORKER_MAX_SCAN_LIMIT_PER_REGION", 100)
                .clamp(1, 200),
            max_observation_radius_km: f64_env("SSS_WORKER_MAX_OBSERVATION_RADIUS_KM", 75.0)
                .clamp(5.0, 500.0),
            enable_discovery: bool_env("SSS_WORKER_ENABLE_DISCOVERY", true),
            enable_scan: bool_env("SSS_WORKER_ENABLE_SCAN", true),
            feeds: WorkerFeedConfig {
                include_adsb: bool_env("SSS_WORKER_INCLUDE_ADSB", true),
                include_weather: bool_env("SSS_WORKER_INCLUDE_WEATHER", true),
                include_fire_smoke: bool_env("SSS_WORKER_INCLUDE_FIRE_SMOKE", true),
            },
            dry_run: bool_env("SSS_WORKER_DRY_RUN", false),
        }
    }

    #[must_use]
    pub fn has_live_feeds(&self) -> bool {
        self.feeds.include_adsb || self.feeds.include_weather || self.feeds.include_fire_smoke
    }

    #[must_use]
    pub fn scan_effectively_enabled(&self) -> bool {
        self.enable_scan && self.has_live_feeds()
    }
}

fn path_env(key: &str, default: &str) -> PathBuf {
    env::var_os(key).map_or_else(|| PathBuf::from(default), PathBuf::from)
}

fn string_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn default_worker_id() -> String {
    format!("sss-passive-worker-{}", std::process::id())
}

fn u64_env(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn usize_env(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn f64_env(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
}

fn bool_env(key: &str, default: bool) -> bool {
    env::var(key).ok().map_or(default, |value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_env_accepts_common_true_values() {
        std::env::set_var("SSS_WORKER_BOOL_TEST", "yes");
        assert!(bool_env("SSS_WORKER_BOOL_TEST", false));
        std::env::remove_var("SSS_WORKER_BOOL_TEST");
    }

    #[test]
    fn numeric_env_falls_back_when_invalid() {
        std::env::set_var("SSS_WORKER_NUM_TEST", "nope");
        assert_eq!(u64_env("SSS_WORKER_NUM_TEST", 42), 42);
        std::env::remove_var("SSS_WORKER_NUM_TEST");
    }

    #[test]
    fn config_reads_heartbeat_retention() {
        std::env::set_var("SSS_WORKER_HEARTBEAT_RETENTION_SECONDS", "120");
        let config = WorkerConfig::from_env();
        assert_eq!(config.heartbeat_retention_seconds, 120);
        std::env::remove_var("SSS_WORKER_HEARTBEAT_RETENTION_SECONDS");
    }

    #[test]
    fn config_defaults_cycle_guardrails_from_parallelism() {
        std::env::set_var("SSS_WORKER_MAX_PARALLEL", "3");
        std::env::remove_var("SSS_WORKER_MAX_REGIONS_PER_CYCLE");
        let config = WorkerConfig::from_env();
        assert_eq!(config.max_parallel_regions, 3);
        assert_eq!(config.max_regions_per_cycle, 6);
        std::env::remove_var("SSS_WORKER_MAX_PARALLEL");
    }
}
