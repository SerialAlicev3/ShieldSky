use uuid::Uuid;

use sss_site_registry::{
    AirspacePolicy, AssetType, Criticality, GeoPoint, ProtectedAsset, ProtectedZone, Site,
    SiteExposureProfile, SiteType, ZoneType,
};

use crate::{adapters, assess_site_sky, AssessSiteSkyInput, SiteContext, ThreatType};

#[test]
fn assess_site_sky_maps_site_signal_to_core_envelope() {
    let site_id = Uuid::new_v4();
    let site = Site {
        site_id,
        name: "Solar Shield PT-01".to_string(),
        site_type: SiteType::SolarPlant,
        latitude: 38.0,
        longitude: -9.0,
        elevation_m: 32.0,
        timezone: "Europe/Lisbon".to_string(),
        country_code: "PT".to_string(),
        criticality: Criticality::High,
        operator_name: "Serial Alice Energy".to_string(),
    };
    let zone = ProtectedZone {
        zone_id: Uuid::new_v4(),
        site_id,
        name: "Panel Field A".to_string(),
        zone_type: ZoneType::PanelField,
        polygon: vec![GeoPoint {
            latitude: 38.0,
            longitude: -9.0,
            altitude_m: 32.0,
        }],
        criticality: Criticality::High,
        allowed_airspace_policy: AirspacePolicy::AuthorizedOnly,
    };
    let asset = ProtectedAsset {
        asset_id: Uuid::new_v4(),
        site_id,
        zone_id: zone.zone_id,
        asset_type: AssetType::SolarArray,
        criticality: Criticality::High,
        operational_dependency_score: 0.82,
    };
    let exposure = SiteExposureProfile {
        site_id,
        drone_risk_sensitivity: 0.9,
        rf_sensitivity: 0.6,
        hail_sensitivity: 0.7,
        wind_sensitivity: 0.6,
        smoke_sensitivity: 0.5,
        dust_sensitivity: 0.7,
        irradiance_variability_sensitivity: 0.8,
    };
    let input = AssessSiteSkyInput {
        site: SiteContext {
            site,
            zones: vec![zone],
            assets: vec![asset],
            exposure_profile: exposure,
        },
        signals: vec![
            adapters::drone_radar::radar_contact(site_id, 1_900_000_000),
            adapters::rf_sensor::rf_burst(site_id, 1_900_000_030),
        ],
    };
    let output = assess_site_sky(&input).expect("assessment");

    assert_eq!(
        output.threat_assessment.threat_type,
        ThreatType::UnauthorizedOverflight
    );
    assert_eq!(output.incident.linked_bundle_hashes.len(), 1);
    assert!(output.core.ranked_event.risk > 0.0);
    assert_eq!(
        output.core.assessment.decision_version,
        "skyshield-v1".to_string()
    );
}
