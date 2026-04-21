use serde::{Deserialize, Serialize};

use sss_site_registry::{ProtectedAsset, ProtectedZone, Site, SiteExposureProfile};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteContext {
    pub site: Site,
    pub zones: Vec<ProtectedZone>,
    pub assets: Vec<ProtectedAsset>,
    pub exposure_profile: SiteExposureProfile,
}
