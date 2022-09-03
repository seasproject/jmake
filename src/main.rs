use std::{env, process, str::FromStr};

use clap::{Args, Parser, Subcommand};
use tokio::fs;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Hello => {
            println!("Hello!");
        }
        Command::Make(args) => {
            let current_dir = env::current_dir().expect("failed to load current directory");

            match args.format {
                ProjectType::Rust => {
                    // Expect a Cargo.toml
                    let cargo_toml = current_dir.join("Cargo.toml");
                    if !cargo_toml.exists() {
                        panic!("this rust project does not contain a Cargo.toml file");
                    }

                    // Run `cargo build --release`
                    let status = process::Command::new("cargo")
                        .args(["build", "--release"])
                        .status()
                        .expect("Failed to run `cargo build --release`");
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
    Make(MakeArgs),
}

#[derive(Args)]
struct MakeArgs {
    #[clap(short = 't', long = "type")]
    format: ProjectType,
}

#[derive(Debug)]
enum ProjectType {
    Rust,
}

impl FromStr for ProjectType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Rust" => Ok(ProjectType::Rust),
            _ => Err(String::from("Invalid project type")),
        }
    }
}
