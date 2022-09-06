use std::{fs::File, path::PathBuf, process};

use clap::{Args, Parser, Subcommand};
use flate2::{write::GzEncoder, Compression};
use tokio::fs::{self};
use tracing::info;

mod format;
use format::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let _: () = match cli.command {
        Command::Hello => {
            info!("Hello!");
        }
        Command::Make => {
            match info()
                .await
                .expect("Failed to find information about package")
            {
                Format::Cargo(cargo_package) => {
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
        Command::Info(args) => {
            let compiled: String;
            match info().await.expect("Failed to load package info") {
                Format::Cargo(cargo_project) => {
                    let out: format::out::Package = format::out::Package {
                        name: cargo_project.package.name.clone(),
                        friendly_name: cargo_project.package.name,
                        version: cargo_project.package.version,
                        install: format::out::InstallInfo {
                            url: args
                                .download_url
                                .unwrap_or("<INSERT DOWNLOAD URL>".to_string()),
                            type_: format::out::PackageType::JellyFish,
                        },
                    };
                    compiled = toml::to_string(&out).expect("Failed to compile Toml");
                }
            }
            if args.out.is_none() {
                println!("{}", compiled);
            } else {
                let out_file = args.out.unwrap();
                fs::write(out_file, compiled)
                    .await
                    .expect("Failed to write to output file");
            }
        }
    };
}

async fn info() -> Result<Format, failure::Error> {
    // Search for common package types

    // Cargo project
    if PathBuf::from("Cargo.toml").exists() {
        info!("Detected Cargo project");

        // Load Cargo.toml
        let path = PathBuf::from("Cargo.toml");
        let cargo_package: cargo::CargoProject =
            toml::from_str(fs::read_to_string(path).await?.as_str())
                .expect("Failed to parse Cargo.toml");

        return Ok(Format::Cargo(cargo_package));
    }

    Err(failure::err_msg("Unknown or invalid project type"))
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Hello,
    /// Builds package then converts it into the JellyFish format
    Make,
    /// Outputs the info of your package in Toml format, ready to go straight into a JellyFish package.toml file.
    Info(InfoArgs),
}

#[derive(Args)]
struct InfoArgs {
    #[clap(short = 'u', long = "url")]
    /// Fill in the download url for the package automatically
    download_url: Option<String>,
    #[clap(short, long)]
    /// Optionally write the generated toml directly to a file
    out: Option<PathBuf>,
}
