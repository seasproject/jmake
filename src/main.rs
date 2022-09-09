use std::{fs::File, path::PathBuf, process};

use clap::{Args, Parser, Subcommand};
use flate2::{write::GzEncoder, Compression};
use tokio::fs::{self};
use tracing::{error, info};

mod format;
use format::{install::InstallFormat, out::PackageType, *};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let _: () = match cli.command {
        Command::Hello => {
            info!("Hello!");
        }
        Command::Make => {
            let (install_format, format) = info()
                .await
                .expect("Failed to find information about package");
            match format {
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

                    if let InstallFormat::Wharf = install_format {
                        for rope in glob::glob("*.rope").unwrap() {
                            let rope = rope.unwrap();
                            tar.append_file(
                                &rope.file_name().unwrap(),
                                &mut File::open(&rope).unwrap(),
                            )
                            .unwrap();
                        }
                    }
                }
                Format::None => {
                    error!("Unknown project type");
                    panic!()
                }
            }
        }
        Command::Info(args) => {
            let (install_format, format) = info().await.expect("Failed to load package info");
            let mut package: format::out::Package = match format {
                Format::Cargo(cargo_project) => format::out::Package {
                    name: cargo_project.package.name.clone(),
                    friendly_name: cargo_project.package.name,
                    version: cargo_project.package.version,
                    install: format::out::InstallInfo {
                        url: args
                            .download_url
                            .unwrap_or("<INSERT DOWNLOAD URL>".to_string()),
                        type_: format::out::PackageType::JellyFish,
                    },
                },
                Format::None => {
                    error!("Invalid format");
                    panic!()
                }
            };

            match install_format {
                InstallFormat::JellyFish => {}
                InstallFormat::Wharf => {
                    package.install.type_ = PackageType::Wharf;
                }
            }

            if args.out.is_none() {
                println!("{}", toml::to_string(&package).unwrap());
            } else {
                let out_file = args.out.unwrap();
                fs::write(out_file, toml::to_string(&package).unwrap())
                    .await
                    .expect("Failed to write to output file");
            }
        }
    };
}

async fn info() -> Result<(InstallFormat, Format), failure::Error> {
    // Search for common package types
    let mut format: Format = Format::None;
    let mut install_format: InstallFormat = InstallFormat::JellyFish;

    // Cargo project
    if PathBuf::from("Cargo.toml").exists() {
        info!("Detected Cargo project");

        // Load Cargo.toml
        let path = PathBuf::from("Cargo.toml");
        let cargo_package: cargo::CargoProject =
            toml::from_str(fs::read_to_string(path).await?.as_str())
                .expect("Failed to parse Cargo.toml");

        format = Format::Cargo(cargo_package);
    }

    // Wharf
    if PathBuf::from("build.rope").exists() {
        info!("Detected Wharf ship");

        install_format = InstallFormat::Wharf;
    }

    Ok((install_format, format))
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
