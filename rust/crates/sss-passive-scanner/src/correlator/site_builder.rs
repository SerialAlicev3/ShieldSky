use crate::domain::{OpenInfraSiteSeed, PassiveSiteProfile};
use crate::sources::infra_mapper::site_profile_from_seed;

#[must_use]
pub fn build_sites(seeds: &[OpenInfraSiteSeed]) -> Vec<PassiveSiteProfile> {
    seeds.iter().map(site_profile_from_seed).collect()
}
