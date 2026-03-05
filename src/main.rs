use clap::{Parser, Subcommand};

mod config;
mod index;
mod installer;
mod lockfile;
mod resolver;

use config::{parse_dep_name, read_config};
use index::fetch_cran_index;
use installer::{build_urls, download_and_install};
use lockfile::write_lockfile;
use resolver::{resolve, resolve_all};

#[derive(Parser)]
#[command(name = "arrrv", about = "A fast R package manager")]
struct Cli {
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
    /// Sync project library from arrrv.toml
    Sync,
    /// Add a package to arrrv.toml and sync
    Add {
        /// Name of the package to add
        package: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { package } => {
            let index = fetch_cran_index();
            println!("resolving dependencies for {}...", package);
            let deps = resolve(&package, &index);
            println!("installing {} packages...", deps.len());
            let urls = build_urls(&deps, &index);
            download_and_install(&urls, "./arrrv_lib");
            println!("done! run with: R_LIBS=./arrrv_lib Rscript -e \"library({})\"", package);
        }

        Commands::Sync => {
            let config = read_config();
            let roots: Vec<String> = config.project.dependencies
                .iter()
                .map(|d| parse_dep_name(d))
                .collect();
            println!("resolving {} direct dependencies...", roots.len());
            let index = fetch_cran_index();
            let all = resolve_all(&roots, &index);
            println!("installing {} packages total...", all.len());
            let urls = build_urls(&all, &index);
            download_and_install(&urls, "./arrrv_lib");
            write_lockfile(&all, &index);
            println!("done! run with: R_LIBS=./arrrv_lib Rscript -e \"library(...)\"");
        }

        Commands::Add { package } => {
            println!("add \"{}\" to your arrrv.toml dependencies, then run arrrv sync", package);
            println!("  dependencies = [\"{}\"]", package);
        }
    }
}
