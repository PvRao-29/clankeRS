//! Command-line interface for [clankeRS](https://docs.rs/clankers).
//!
//! Install the `clankers` binary with `cargo install clankers-cli`, then scaffold
//! projects, replay MCAP logs, validate ONNX models, and run bundled demos.
//!
//! ## Commands
//!
//! | Command | Purpose |
//! |---------|---------|
//! | `new` | Scaffold from `basic-node`, `perception-node`, `ml-inference-node`, or `replay-test-node` |
//! | `run` | Run the current project's node |
//! | `test` | Run replay tests and `cargo test` |
//! | `inspect` | Summarize an MCAP file |
//! | `replay` | Replay a log (optionally through a node) |
//! | `latency` | Latency stats from a replay |
//! | `compare` | Diff two MCAP files |
//! | `validate-model` | ONNX vs PyTorch reference outputs |
//! | `bench` | Benchmark inference latency and copy/allocation stats |
//! | `import-pytorch` | Export a checkpoint to ONNX |
//! | `add-model` | Register a model in `clankeRS.toml` |
//! | `visualize` | MCAP summary for Foxglove / Rerun |
//! | `demo` | Run bundled demos (`camera-perception`) |
//! | `record` | MCAP recording (stub — not complete yet) |
//!
//! See the [crate README](https://github.com/PvRao-29/clankeRS/blob/main/crates/clankers-cli/README.md)
//! for install steps and examples.

mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use commands::{
    add_model, bench, compare, import_pytorch, inspect, latency, new_project, record, replay, run,
    test, validate_model, visualize,
};

const BRAND: &str = "clankeRS";

/// `clankers` CLI parsed from `std::env::args`.
#[derive(Parser)]
#[command(name = "clankers", about = "clankeRS — Rust robotics SDK CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Subcommands for the `clankers` binary.
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new clankeRS project from a template
    New {
        name: String,
        #[arg(long, default_value = "basic-node")]
        template: String,
    },
    /// Run the current clankeRS node
    Run,
    /// Run replay tests and cargo test
    Test,
    /// Inspect an MCAP file
    Inspect { file: String },
    /// Replay an MCAP file through a node
    Replay {
        file: String,
        #[arg(long)]
        node: Option<String>,
    },
    /// Report latency statistics from an MCAP replay
    Latency { file: String },
    /// Compare two MCAP files
    Compare { expected: String, actual: String },
    /// Record node I/O to MCAP
    Record {
        #[arg(long, default_value = "logs/run.mcap")]
        output: String,
    },
    /// Benchmark model inference latency (p50/p95/p99) and copy/alloc accounting
    Bench {
        #[arg(long)]
        model: String,
        #[arg(long, default_value = "onnxruntime")]
        backend: String,
        #[arg(long)]
        input: Option<String>,
        #[arg(long, default_value = "50")]
        warmup: u32,
        #[arg(long, default_value = "1000")]
        iters: u32,
    },
    /// Validate ONNX model output against PyTorch reference
    ValidateModel {
        #[arg(long)]
        pytorch: Option<String>,
        #[arg(long)]
        checkpoint: Option<String>,
        #[arg(long)]
        onnx: String,
        #[arg(long, default_value = "sample_data/policy_inputs/")]
        samples: String,
        #[arg(long, default_value = "0.001")]
        tolerance: f32,
    },
    /// Import a PyTorch model and export to ONNX
    ImportPytorch {
        #[arg(long)]
        model: String,
        #[arg(long)]
        checkpoint: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        opset: Option<u32>,
    },
    /// Add a model to the current project
    AddModel { path: String },
    /// Visualization hook — export MCAP summary for Foxglove/Rerun
    Visualize { file: String },
    /// Run the camera perception demo
    Demo {
        #[arg(default_value = "camera-perception")]
        name: String,
    },
}

/// Parse CLI arguments and dispatch the selected subcommand.
pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, template } => new_project::execute(&name, &template)?,
        Commands::Run => run::execute()?,
        Commands::Test => test::execute().await?,
        Commands::Inspect { file } => inspect::execute(&file)?,
        Commands::Replay { file, node } => replay::execute(&file, node.as_deref()).await?,
        Commands::Latency { file } => latency::execute(&file).await?,
        Commands::Compare { expected, actual } => compare::execute(&expected, &actual)?,
        Commands::Bench {
            model,
            backend,
            input,
            warmup,
            iters,
        } => bench::execute(&model, &backend, input.as_deref(), warmup, iters)?,
        Commands::Record { output } => record::execute(&output)?,
        Commands::ValidateModel {
            pytorch,
            checkpoint,
            onnx,
            samples,
            tolerance,
        } => validate_model::execute(
            pytorch.as_deref(),
            checkpoint.as_deref(),
            &onnx,
            &samples,
            tolerance,
        )?,
        Commands::ImportPytorch {
            model,
            checkpoint,
            output,
            opset,
        } => import_pytorch::execute(&model, &checkpoint, &output, opset)?,
        Commands::AddModel { path } => add_model::execute(&path)?,
        Commands::Visualize { file } => visualize::execute(&file)?,
        Commands::Demo { name } => commands::demo::execute(&name).await?,
    }

    Ok(())
}

/// Print a branded status line to stdout.
pub fn print_banner(msg: &str) {
    println!("{BRAND} {msg}");
}
