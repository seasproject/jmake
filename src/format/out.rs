use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub friendly_name: String,
    pub version: String,
    pub install: InstallInfo,
}

#[derive(Serialize, Deserialize)]
pub struct InstallInfo {
    pub url: String,
    pub type_: PackageType,
}

#[derive(Serialize, Deserialize)]
pub enum PackageType {
    JellyFish,
    Wharf,
}
