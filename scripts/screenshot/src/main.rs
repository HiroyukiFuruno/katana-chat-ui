mod executor_harness;
mod fixture;
mod hash;
mod native_capture;
mod request;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "kcu-screenshot",
    about = "Headless screenshot runner for katana-chat-ui"
)]
struct Cli {
    #[arg(long, value_name = "FILE")]
    request: Option<PathBuf>,
    #[arg(long, value_enum)]
    native_host: Option<native_capture::NativeHost>,
    #[arg(long, value_name = "SHA256")]
    native_baseline_sha256: Option<String>,
    #[arg(long, value_name = "DIR")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    std::fs::create_dir_all(&cli.output)
        .with_context(|| format!("cannot create output dir: {}", cli.output.display()))?;
    let output_dir = cli.output.canonicalize()?;

    match (cli.request, cli.native_host) {
        (Some(request), None) => run_headless_request(request, &output_dir)?,
        (None, Some(host)) => {
            native_capture::run(host, &output_dir, cli.native_baseline_sha256.as_deref())?;
        }
        _ => {
            anyhow::bail!("specify exactly one of --request or --native-host");
        }
    }
    println!("[kcu-screenshot] done");
    Ok(())
}

fn run_headless_request(request_path: PathBuf, output_dir: &std::path::Path) -> Result<()> {
    let request_path = request_path
        .canonicalize()
        .with_context(|| format!("request file not found: {}", request_path.display()))?;
    let request = request::load(&request_path)?;

    println!("[kcu-screenshot] request: {}", request.name);
    println!("[kcu-screenshot] output:  {}", output_dir.display());
    executor_harness::run(&request, output_dir)
}
