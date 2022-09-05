use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CargoProject {
    pub package: InnerPackage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InnerPackage {
    pub name: String,
    pub version: String,
}
