mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use commands::{
    add_model, compare, import_pytorch, inspect, latency, new_project, record, replay, run, test,
    validate_model, visualize,
};

const BRAND: &str = "clankeRS";

#[derive(Parser)]
#[command(name = "clankers", about = "clankeRS — Rust robotics SDK CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

pub fn print_banner(msg: &str) {
    println!("{BRAND} {msg}");
}
