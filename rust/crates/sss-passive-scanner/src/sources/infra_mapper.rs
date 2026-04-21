use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use uuid::Uuid;

use crate::domain::{OpenInfraSiteSeed, PassiveSiteProfile};
use sss_site_registry::Site;

#[must_use]
pub fn site_profile_from_seed(seed: &OpenInfraSiteSeed) -> PassiveSiteProfile {
    PassiveSiteProfile {
        site: Site {
            site_id: stable_site_uuid(seed),
            name: seed.name.clone(),
            site_type: seed.site_type,
            latitude: seed.latitude,
            longitude: seed.longitude,
            elevation_m: seed.elevation_m,
            timezone: seed.timezone.clone(),
            country_code: seed.country_code.clone(),
            criticality: seed.criticality,
            operator_name: seed.operator_name.clone(),
        },
        observation_radius_km: seed.observation_radius_km,
        source_references: vec![seed.source_reference.clone()],
        passive_tags: vec![
            format!("{:?}", seed.site_type),
            format!("{:?}", seed.criticality),
            "phase0_observed".to_string(),
        ],
    }
}

fn stable_site_uuid(seed: &OpenInfraSiteSeed) -> Uuid {
    let primary = stable_hash(&format!(
        "{}:{}:{:.6}:{:.6}",
        seed.source_reference, seed.name, seed.latitude, seed.longitude
    ));
    let secondary = stable_hash(&format!(
        "{:?}:{:?}:{}:{}",
        seed.site_type, seed.criticality, seed.country_code, seed.operator_name
    ));
    let primary_high = u32::try_from(primary >> 32)
        .expect("stable passive site primary hash head should fit in u32");
    let primary_mid = u16::try_from((primary >> 16) & 0xffff)
        .expect("stable passive site primary hash segment should fit in u16");
    let primary_low = u16::try_from(primary & 0xffff)
        .expect("stable passive site primary hash tail should fit in u16");
    let secondary_high = u16::try_from((secondary >> 48) & 0xffff)
        .expect("stable passive site secondary hash segment should fit in u16");
    let uuid = format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        primary_high,
        primary_mid,
        primary_low,
        secondary_high,
        secondary & 0x0000_ffff_ffff_ffff
    );
    uuid.parse().expect("stable passive site uuid should parse")
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
