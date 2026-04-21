use sss_core::{
    AnticipationEngine, BehaviorSignature, DigitalTwin, MissionClass, Observation,
    ObservationSource, OrbitRegime, OrbitalState, RiskProfile, SpaceObject, Vector3,
};

fn main() {
    let object = SpaceObject {
        id: "SSS-DEMO-001".to_string(),
        name: "Serial Alice Demo Object".to_string(),
        regime: OrbitRegime::Leo,
        state: OrbitalState {
            epoch_unix_seconds: 1_800_000_000,
            position_km: Vector3 {
                x: 6_900.0,
                y: 0.0,
                z: 0.0,
            },
            velocity_km_s: Vector3 {
                x: 0.0,
                y: 7.6,
                z: 0.0,
            },
        },
        tle: None,
        behavior: BehaviorSignature {
            nominal_rf_mhz: Some(2_240.0),
            historical_maneuvers_per_week: 0.2,
            station_keeping_expected: false,
        },
        risk: RiskProfile {
            operator: Some("synthetic".to_string()),
            mission_class: MissionClass::Commercial,
            criticality: 0.55,
        },
    };

    let observation = Observation {
        object_id: object.id.clone(),
        observed_state: OrbitalState {
            epoch_unix_seconds: 1_800_000_060,
            position_km: Vector3 {
                x: 6_940.0,
                y: 470.0,
                z: 0.0,
            },
            velocity_km_s: Vector3 {
                x: 0.05,
                y: 7.63,
                z: 0.0,
            },
        },
        rf_mhz: Some(2_241.1),
        maneuver_detected: true,
        source: ObservationSource::Synthetic,
    };

    let twin = DigitalTwin::new(vec![object]);
    let alert = AnticipationEngine::default()
        .evaluate_observation(&twin, &observation)
        .expect("demo object should exist");

    println!("{alert:#?}");
}
