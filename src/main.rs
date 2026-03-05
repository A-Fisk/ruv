use clap::{Parser, Subcommand};
use std::time::Instant;

mod cache;
mod config;
mod index;
mod installer;
mod lockfile;
mod resolver;

use config::{parse_dep_name, read_config};
use index::fetch_cran_index;
use installer::{build_urls, build_urls_from_pairs, download_and_install};
use lockfile::{lockfile_is_fresh, read_lockfile, write_lockfile};
use resolver::{resolve, resolve_all};

const LIB_DIR: &str = ".arrrv/library";

#[derive(Parser)]
#[command(name = "arrrv", about = "A fast R package manager")]
struct Cli {
    /// Print extra debug information
    #[arg(long, short, global = true)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install an R package and its dependencies
    Install {
        /// Name of the package to install
        package: String,
    },
    /// Install from arrrv.lock (error if lockfile missing or stale)
    Sync,
    /// Resolve dependencies from arrrv.toml and write arrrv.lock
    Lock,
    /// Add a package to arrrv.toml and sync
    Add {
        /// Name of the package to add
        package: String,
    },
    /// Run a script with the project library
    Run {
        /// Arguments to pass to Rscript (e.g. analysis.R or -e "library(ggplot2)")
        args: Vec<String>,
    },
}

fn fmt_duration(ms: u128) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else {
        format!("{:.2}s", ms as f64 / 1000.0)
    }
}

fn main() {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        Commands::Install { package } => {
            let t = Instant::now();
            let index = fetch_cran_index();
            let deps = resolve(&package, &index);
            let packages = build_urls(&deps, &index);
            println!(
                "Resolved {} packages in {}",
                deps.len(),
                fmt_duration(t.elapsed().as_millis())
            );

            let t = Instant::now();
            let (audited, installed) = download_and_install(&packages, LIB_DIR, verbose);
            if audited > 0 {
                println!(
                    "Audited {} packages in {}",
                    audited,
                    fmt_duration(t.elapsed().as_millis())
                );
            }
            if installed > 0 {
                println!(
                    "Installed {} packages in {}",
                    installed,
                    fmt_duration(t.elapsed().as_millis())
                );
            }
        }

        Commands::Lock => {
            let config = read_config();
            let roots: Vec<String> = config
                .project
                .dependencies
                .iter()
                .map(|d| parse_dep_name(d))
                .collect();

            let t = Instant::now();
            let index = fetch_cran_index();
            let all = resolve_all(&roots, &index);
            println!(
                "Resolved {} packages in {}",
                all.len(),
                fmt_duration(t.elapsed().as_millis())
            );

            write_lockfile(&roots, &all, &index);
        }

        Commands::Sync => {
            let config = read_config();
            let roots: Vec<String> = config
                .project
                .dependencies
                .iter()
                .map(|d| parse_dep_name(d))
                .collect();

            if !lockfile_is_fresh(&roots) {
                eprintln!("error: arrrv.lock is missing or out of date — run `arrrv lock` first");
                std::process::exit(1);
            }

            let t = Instant::now();
            let locked = read_lockfile();
            let packages = build_urls_from_pairs(&locked);
            println!(
                "Resolved {} packages in {}",
                locked.len(),
                fmt_duration(t.elapsed().as_millis())
            );

            if verbose {
                println!("  lib_dir:  {}", LIB_DIR);
                println!("  packages: {}", packages.len());
            }

            let t = Instant::now();
            let (audited, installed) = download_and_install(&packages, LIB_DIR, verbose);
            if audited > 0 {
                println!(
                    "Audited {} packages in {}",
                    audited,
                    fmt_duration(t.elapsed().as_millis())
                );
            }
            if installed > 0 {
                println!(
                    "Installed {} packages in {}",
                    installed,
                    fmt_duration(t.elapsed().as_millis())
                );
            }
        }

        Commands::Add { package } => {
            println!(
                "add \"{}\" to your arrrv.toml dependencies, then run `arrrv lock && arrrv sync`",
                package
            );
            println!("  dependencies = [\"{}\"]", package);
        }

        Commands::Run { args } => {
            let lib_dir = std::fs::canonicalize(LIB_DIR)
                .expect("no project library found — run `arrrv lock && arrrv sync` first");

            std::process::Command::new("Rscript")
                .args(&args)
                .env("R_LIBS", lib_dir)
                .status()
                .unwrap();
        }
    }
}
