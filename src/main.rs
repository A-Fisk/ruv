use clap::{Parser, Subcommand};
use std::time::Instant;

mod cache;
mod config;
mod crandb;
mod index;
mod installer;
mod lockfile;
mod r_version;
mod resolver;
mod version;

use config::{
    AddDependencyResult, RemoveDependencyResult, add_dependency, init_config, parse_dep,
    parse_dep_name, read_config, remove_dependency,
};
use index::fetch_cran_index;
use installer::{build_urls, build_urls_from_pairs, download_and_install};
use lockfile::{lockfile_is_fresh, read_lockfile, write_lockfile};
use resolver::{resolve, resolve_all};

const LIB_DIR: &str = ".ruv/library";

#[derive(Parser)]
#[command(name = "ruv", about = "A fast R package manager")]
struct Cli {
    /// Print extra debug information
    #[arg(long, short, global = true)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new ruv.toml in the current directory
    Init,
    /// Install an R package and its dependencies
    Install {
        /// Name of the package to install
        package: String,
    },
    /// Install from ruv.lock (error if lockfile missing or stale)
    Sync,
    /// Resolve dependencies from ruv.toml and write ruv.lock
    Lock,
    /// Add a package to ruv.toml and re-lock
    Add {
        /// Name of the package to add (optionally with version constraint, e.g. ggplot2>=3.4)
        package: String,
    },
    /// Remove a package from ruv.toml and re-lock
    Remove {
        /// Name of the package to remove
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

/// Reads ruv.toml, resolves all dependencies, writes ruv.lock, and prints progress.
/// Exits the process on error.
fn run_lock(verbose: bool) {
    let config = read_config().unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
    let root_deps: Vec<_> = config
        .project
        .dependencies
        .iter()
        .map(|d| parse_dep(d))
        .collect();
    let root_names: Vec<String> = root_deps.iter().map(|d| d.name.clone()).collect();

    let t = Instant::now();
    let index = fetch_cran_index();
    let resolved = resolve_all(&root_deps, &index, verbose).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
    println!(
        "Resolved {} packages in {}",
        resolved.len(),
        fmt_duration(t.elapsed().as_millis())
    );

    write_lockfile(&root_names, &resolved, &index);
}

fn main() {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        Commands::Init => {
            let project_name = std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "my-project".to_string());

            init_config(&project_name).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });
            println!("wrote ruv.toml");
            println!("next: run `ruv lock` then `ruv sync`");
        }

        Commands::Install { package } => {
            let t = Instant::now();
            let index = fetch_cran_index();
            let resolved = resolve(&package, &index, verbose).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });
            let dep_names: Vec<String> = resolved
                .keys()
                .filter(|n| *n != &package)
                .cloned()
                .collect();
            let packages = build_urls(&dep_names, &index);
            println!(
                "Resolved {} packages in {}",
                resolved.len(),
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
            run_lock(verbose);
        }

        Commands::Sync => {
            let config = read_config().unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            });
            let roots: Vec<String> = config
                .project
                .dependencies
                .iter()
                .map(|d| parse_dep_name(d))
                .collect();

            if !lockfile_is_fresh(&roots) {
                eprintln!("error: ruv.lock is missing or out of date — run `ruv lock` first");
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

            if let Some(constraint) = &config.project.r_version {
                let installation = r_version::select_r(constraint).or_else(|_| {
                    println!("  R {} not found locally, downloading...", constraint);
                    r_version::auto_install_r(constraint)
                });
                match installation {
                    Ok(inst) => {
                        match r_version::setup_r_symlinks(&inst) {
                            Ok(()) => {
                                println!("Using R {} ({})", inst.version, inst.bin_dir.display())
                            }
                            Err(e) => eprintln!("warning: {}", e),
                        }
                        if let Err(e) = r_version::setup_r_profile() {
                            eprintln!("warning: {}", e);
                        }
                        if let Err(e) = r_version::setup_rproj(&config.project.name) {
                            eprintln!("warning: {}", e);
                        }
                    }
                    Err(e) => eprintln!("warning: failed to set up R: {}", e),
                }
            }
        }

        Commands::Add { package } => {
            match add_dependency(&package).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            }) {
                AddDependencyResult::Added => {
                    println!("added \"{}\" to ruv.toml", package);
                    run_lock(verbose);
                    println!("next: run `ruv sync` to install");
                }
                AddDependencyResult::AlreadyPresent => {
                    println!("\"{}\" is already in ruv.toml — nothing to do", package);
                }
            }
        }

        Commands::Remove { package } => {
            match remove_dependency(&package).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                std::process::exit(1);
            }) {
                RemoveDependencyResult::Removed => {
                    println!("removed \"{}\" from ruv.toml", package);
                    run_lock(verbose);
                    println!("next: run `ruv sync` to apply");
                }
                RemoveDependencyResult::NotFound => {
                    eprintln!("error: \"{}\" is not in ruv.toml", package);
                    std::process::exit(1);
                }
            }
        }

        Commands::Run { args } => {
            let lib_dir = std::fs::canonicalize(LIB_DIR)
                .expect("no project library found — run `ruv lock && ruv sync` first");

            // Route to managed R/Rscript if symlinks exist, else fall back to system.
            // `ruv run R ...`       → R binary (interactive REPL, workspace via .Rprofile)
            // `ruv run Rscript ...` → Rscript binary
            // `ruv run script.R`    → Rscript script.R (backward compat)
            let (program, run_args): (std::ffi::OsString, Vec<String>) =
                match args.first().map(String::as_str) {
                    Some("R") => {
                        let bin = r_version::project_r()
                            .map(|p| p.into_os_string())
                            .unwrap_or_else(|| "R".into());
                        (bin, args[1..].to_vec())
                    }
                    Some("Rscript") => {
                        let bin = r_version::project_rscript()
                            .map(|p| p.into_os_string())
                            .unwrap_or_else(|| "Rscript".into());
                        (bin, args[1..].to_vec())
                    }
                    _ => {
                        let bin = r_version::project_rscript()
                            .map(|p| p.into_os_string())
                            .unwrap_or_else(|| "Rscript".into());
                        (bin, args)
                    }
                };

            std::process::Command::new(program)
                .args(&run_args)
                .env("R_LIBS", lib_dir)
                .status()
                .unwrap();
        }
    }
}
