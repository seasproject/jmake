use serde::Serialize;

#[derive(Serialize)]
pub struct Package {
    pub name: String,
    pub friendly_name: String,
    pub version: String,
    pub install: InstallInfo,
}

#[derive(Serialize)]
pub struct InstallInfo {
    pub url: String,
    pub type_: PackageType,
}

#[derive(Serialize)]
pub enum PackageType {
    JellyFish,
}
