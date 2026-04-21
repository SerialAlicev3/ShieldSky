use sss_core::{AlertSeverity, AnalysisStatus, RankedEventType};
use sss_site_registry::{Criticality, SiteType};

use crate::{
    AdsbFlightObservation, FireSmokeObservation, OpenInfraSiteSeed, OrbitalPassObservation,
    PassiveScanner, PassiveThreatType, SatelliteSceneObservation, WeatherObservation,
};

fn sample_seed() -> OpenInfraSiteSeed {
    OpenInfraSiteSeed {
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
    }
}

#[test]
fn passive_scanner_builds_site_and_ranks_events_from_public_feeds() {
    let scanner = PassiveScanner::new();
    let output = scanner.scan(&crate::PassiveScanInput {
        window_start_unix_seconds: 1_776_384_000,
        window_end_unix_seconds: 1_776_470_400,
        infra_sites: vec![sample_seed()],
        adsb_observations: vec![AdsbFlightObservation {
            flight_id: "ADSBX-ES001".to_string(),
            observed_at_unix_seconds: 1_776_390_000,
            latitude: 37.40,
            longitude: -5.99,
            altitude_m: 620.0,
            speed_kts: 92.0,
            vertical_rate_m_s: -1.0,
        }],
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
        orbital_observations: vec![OrbitalPassObservation {
            object_id: "deb-2042".to_string(),
            observed_at_unix_seconds: 1_776_396_000,
            latitude: 37.38,
            longitude: -5.98,
            reentry_probability: 0.57,
            footprint_radius_km: 160.0,
        }],
        satellite_observations: vec![SatelliteSceneObservation {
            scene_id: "sentinel-2a-t32".to_string(),
            observed_at_unix_seconds: 1_776_398_000,
            latitude: 37.39,
            longitude: -5.985,
            change_score: 0.73,
            cloud_cover_ratio: 0.12,
        }],
        notam_observations: vec![],
    });

    assert_eq!(output.sites.len(), 1);
    assert!(output.events.len() >= 4);
    assert_eq!(output.risk_history.len(), 1);
    assert!(!output.core_envelopes.is_empty());
    assert!(output
        .events
        .iter()
        .any(|event| event.threat_type == PassiveThreatType::HailExposure));
    assert!(output
        .core_envelopes
        .iter()
        .all(|envelope| !envelope.evidence_bundle.bundle_hash.is_empty()));
}

#[test]
fn passive_scanner_extracts_recurring_patterns_for_same_site() {
    let scanner = PassiveScanner::new();
    let output = scanner.scan(&crate::PassiveScanInput {
        window_start_unix_seconds: 1_776_384_000,
        window_end_unix_seconds: 1_776_470_400,
        infra_sites: vec![sample_seed()],
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
        weather_observations: vec![],
        fire_smoke_observations: vec![],
        orbital_observations: vec![],
        satellite_observations: vec![],
        notam_observations: vec![],
    });

    assert_eq!(output.patterns.len(), 1);
    assert_eq!(
        output.patterns[0].threat_type,
        PassiveThreatType::LowAltitudeOverflight
    );
    assert_eq!(output.patterns[0].recurring_events, 2);
}

#[test]
fn passive_scanner_derives_temporal_phenomenon_keys_from_source_identity() {
    let scanner = PassiveScanner::new();
    let output = scanner.scan(&crate::PassiveScanInput {
        window_start_unix_seconds: 1_776_384_000,
        window_end_unix_seconds: 1_776_470_400,
        infra_sites: vec![sample_seed()],
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
                flight_id: "ADSBX-ES001".to_string(),
                observed_at_unix_seconds: 1_776_396_000,
                latitude: 37.401,
                longitude: -5.988,
                altitude_m: 610.0,
                speed_kts: 90.0,
                vertical_rate_m_s: -0.8,
            },
            AdsbFlightObservation {
                flight_id: "ADSBX-ES777".to_string(),
                observed_at_unix_seconds: 1_776_396_600,
                latitude: 37.402,
                longitude: -5.987,
                altitude_m: 605.0,
                speed_kts: 91.0,
                vertical_rate_m_s: -0.7,
            },
        ],
        weather_observations: vec![],
        fire_smoke_observations: vec![],
        orbital_observations: vec![],
        satellite_observations: vec![],
        notam_observations: vec![],
    });

    let mut first_flight_keys = output
        .events
        .iter()
        .filter(|event| event.payload["flight_id"].as_str() == Some("ADSBX-ES001"))
        .map(|event| event.phenomenon_key.clone())
        .collect::<Vec<_>>();
    first_flight_keys.sort();
    first_flight_keys.dedup();

    let mut second_flight_keys = output
        .events
        .iter()
        .filter(|event| event.payload["flight_id"].as_str() == Some("ADSBX-ES777"))
        .map(|event| event.phenomenon_key.clone())
        .collect::<Vec<_>>();
    second_flight_keys.sort();
    second_flight_keys.dedup();

    assert_eq!(first_flight_keys.len(), 1);
    assert_eq!(second_flight_keys.len(), 1);
    assert_ne!(first_flight_keys[0], second_flight_keys[0]);
}

#[test]
fn passive_scanner_persists_temporal_anchor_metadata_on_events() {
    let scanner = PassiveScanner::new();
    let output = scanner.scan(&crate::PassiveScanInput {
        window_start_unix_seconds: 1_776_384_000,
        window_end_unix_seconds: 1_776_470_400,
        infra_sites: vec![sample_seed()],
        adsb_observations: vec![AdsbFlightObservation {
            flight_id: "ADSBX-ES001".to_string(),
            observed_at_unix_seconds: 1_776_390_000,
            latitude: 37.40,
            longitude: -5.99,
            altitude_m: 620.0,
            speed_kts: 92.0,
            vertical_rate_m_s: -1.0,
        }],
        weather_observations: vec![],
        fire_smoke_observations: vec![],
        orbital_observations: vec![],
        satellite_observations: vec![],
        notam_observations: vec![],
    });

    let event = output.events.first().expect("passive event");
    assert_eq!(event.payload["flight_id"].as_str(), Some("ADSBX-ES001"));
    assert_eq!(
        event.payload["phenomenon_anchor"].as_str(),
        Some("flight:ADSBX-ES001")
    );
    assert_eq!(
        event.payload["phenomenon_window_seconds"].as_i64(),
        Some(21_600)
    );
    assert!(event.payload["phenomenon_time_bucket"].as_i64().is_some());
}

#[test]
fn passive_core_envelope_maps_to_auditable_core_contracts() {
    let scanner = PassiveScanner::new();
    let output = scanner.scan(&crate::PassiveScanInput {
        window_start_unix_seconds: 1_776_384_000,
        window_end_unix_seconds: 1_776_470_400,
        infra_sites: vec![sample_seed()],
        adsb_observations: vec![],
        weather_observations: vec![WeatherObservation {
            observed_at_unix_seconds: 1_776_392_000,
            latitude: 37.42,
            longitude: -5.97,
            wind_kph: 96.0,
            hail_probability: 0.72,
            irradiance_drop_ratio: 0.51,
        }],
        fire_smoke_observations: vec![],
        orbital_observations: vec![],
        satellite_observations: vec![],
        notam_observations: vec![],
    });

    let envelope = &output.core_envelopes[0];
    assert_eq!(envelope.assessment.status, AnalysisStatus::Watch);
    assert_eq!(envelope.decision.severity, AlertSeverity::Watch);
    assert_eq!(
        envelope.ranked_event.event_type,
        RankedEventType::BehaviorShiftDetected
    );
    assert_eq!(
        envelope.assessment.decision_version,
        "sss-passive-decision-v1".to_string()
    );
    assert!(!envelope.replay_manifest.manifest_hash.is_empty());
}
