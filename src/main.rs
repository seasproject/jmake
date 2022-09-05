use std::{fs::File, path::PathBuf, process};

use clap::{Parser, Subcommand};
use flate2::{write::GzEncoder, Compression};
use tokio::fs::{self};
use tracing::info;

mod format;
use format::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Hello => {
            info!("Hello!");
        }
        Command::Make => {
            // Search for common package types
            // Cargo project
            if PathBuf::from("Cargo.toml").exists() {
                info!("Detected Cargo project");

                // Load Cargo.toml
                let path = PathBuf::from("Cargo.toml");
                let cargo_package: cargo::CargoProject =
                    toml::from_str(fs::read_to_string(path).await.unwrap().as_str())
                        .expect("Failed to parse Cargo.toml");

                // Build package
                info!("Building package");
                let status = process::Command::new("cargo")
                    .args(["build", "--release"])
                    .status()
                    .expect("Failed to execute cargo");

                if !status.success() {
                    panic!("failed to build cargo project");
                }

                let mut bin_paths: Vec<PathBuf> = vec![];

                let exe_path = PathBuf::from("target/release/")
                    .join(format!("{}.exe", cargo_package.package.name));
                if exe_path.exists() {
                    bin_paths.push(exe_path);
                }

                let lib_path = PathBuf::from("target/release/")
                    .join(format!("lib{}.rlib", cargo_package.package.name));
                if lib_path.exists() {
                    bin_paths.push(lib_path);
                }

                let bin_folder = PathBuf::from("bin");
                info!("Compressing!");
                let tar_gz = File::create(format!("{}.jellyfish", cargo_package.package.name))
                    .expect("Failed to create archive");
                let enc = GzEncoder::new(tar_gz, Compression::best());
                let mut tar = tar::Builder::new(enc);
                for bin in &bin_paths {
                    tar.append_file(
                        bin_folder.join(bin.file_name().unwrap()),
                        &mut File::open(bin).unwrap(),
                    )
                    .unwrap();
                }
            }
        }
    }
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Hello,
    Make,
}
