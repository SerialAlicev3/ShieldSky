use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

static COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Uuid(u128);

impl Uuid {
    #[must_use]
    pub fn new_v4() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        let counter = u128::from(COUNTER.fetch_add(1, Ordering::Relaxed));
        Self(nanos ^ counter)
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            ((self.0 >> 96) & 0xffff_ffff) as u32,
            ((self.0 >> 80) & 0xffff) as u16,
            ((self.0 >> 64) & 0xffff) as u16,
            ((self.0 >> 48) & 0xffff) as u16,
            self.0 & 0xffff_ffff_ffff
        )
    }
}

impl FromStr for Uuid {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.replace('-', "");
        u128::from_str_radix(&normalized, 16)
            .map(Self)
            .map_err(|_| "invalid uuid string")
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}
