use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

const EXAMPLE_DIR: &str = "examples/react-ssr-stream";
const BUNDLE_PATH: &str = "examples/react-ssr-stream/dist/app.bundle.js";

#[derive(Parser)]
#[command(author, version, about = "Project automation helpers for rsengine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the React sample bundle used for smoke tests.
    Bundle {
        /// Force reinstalling Node dependencies before building.
        #[arg(long)]
        install: bool,
    },
    /// Build the React sample bundle and run cargo tests.
    Test {
        /// Force reinstalling Node dependencies before building.
        #[arg(long)]
        install: bool,
        /// Extra arguments passed to `cargo test` after the bundle is built.
        #[arg(trailing_var_arg = true)]
        cargo_args: Vec<String>,
    },
    /// Build the React sample bundle and run the server with it.
    Serve {
        /// Force reinstalling Node dependencies before building.
        #[arg(long)]
        install: bool,
        /// Extra arguments forwarded to `cargo run -p server` after `--`.
        #[arg(trailing_var_arg = true)]
        server_args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bundle { install } => build_bundle(install).map(|_| ()),
        Commands::Test {
            install,
            cargo_args,
        } => {
            let bundle = build_bundle(install)?;
            run_tests(bundle, cargo_args)
        }
        Commands::Serve {
            install,
            server_args,
        } => {
            let bundle = build_bundle(install)?;
            run_server(bundle, server_args)
        }
    }
}

fn build_bundle(force_install: bool) -> Result<PathBuf> {
    let project_dir = Path::new(EXAMPLE_DIR);
    if !project_dir.exists() {
        return Err(anyhow!(
            "example project directory '{}' is missing",
            EXAMPLE_DIR
        ));
    }

    let node_modules = project_dir.join("node_modules");
    if force_install || !node_modules.exists() {
        let mut install = Command::new("npm");
        install
            .arg("install")
            .current_dir(project_dir)
            .envs(filtered_env());
        run_command(install, "npm install")?;
    }

    let mut bundle = Command::new("npm");
    bundle
        .arg("run")
        .arg("bundle")
        .current_dir(project_dir)
        .envs(filtered_env());
    run_command(bundle, "npm run bundle")?;

    let bundle_path = PathBuf::from(BUNDLE_PATH);
    if !bundle_path.exists() {
        return Err(anyhow!(
            "bundle expected at '{}' was not produced",
            BUNDLE_PATH
        ));
    }

    println!("bundle ready at {}", bundle_path.display());
    Ok(bundle_path)
}

fn run_tests(bundle_path: PathBuf, cargo_args: Vec<String>) -> Result<()> {
    let mut command = Command::new("cargo");
    command.arg("test");
    command.args(cargo_args);
    command.env("RSENGINE_TEST_BUNDLE", bundle_path);

    run_command(command, "cargo test")
}

fn run_server(bundle_path: PathBuf, server_args: Vec<String>) -> Result<()> {
    let bundle_arg = bundle_path.to_string_lossy().to_string();

    let mut command = Command::new("cargo");
    command.arg("run");
    command.arg("-p");
    command.arg("server");
    command.arg("--");
    command.arg("--bundle");
    command.arg(bundle_arg);
    command.args(server_args);

    run_command(command, "cargo run -p server")
}

fn run_command(mut command: Command, label: &str) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("failed to spawn {}", label))?;
    if !status.success() {
        return Err(anyhow!("{} exited with {}", label, status));
    }
    Ok(())
}

fn filtered_env() -> impl Iterator<Item = (String, String)> {
    env::vars().filter(|(key, _)| key != "RSENGINE_TEST_BUNDLE")
}
