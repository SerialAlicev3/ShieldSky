use sss_site_registry::{ProtectedAsset, ProtectedZone};

use crate::ThreatType;

#[must_use]
pub fn impact_summary(
    site_name: &str,
    threat_type: ThreatType,
    zone: &ProtectedZone,
    asset: &ProtectedAsset,
) -> String {
    format!(
        "{site_name} threat {:?} affects zone {} and asset {:?}",
        threat_type, zone.name, asset.asset_type
    )
}
