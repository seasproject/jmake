pub mod cargo;
pub mod out;

pub enum Format {
    Cargo(cargo::CargoProject),
}
