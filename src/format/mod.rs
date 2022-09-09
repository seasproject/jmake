pub mod cargo;
pub mod install;
pub mod out;

pub enum Format {
    Cargo(cargo::CargoProject),
    None,
}
