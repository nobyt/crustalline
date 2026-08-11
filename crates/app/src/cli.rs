use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "crustalline")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Render a molecule to a PNG file with no visible window (see M3).
    Render(RenderArgs),
}

#[derive(clap::Args)]
pub struct RenderArgs {
    /// SMILES string to render.
    pub smiles: String,
    /// Output PNG path.
    pub out_path: String,
    #[arg(long, default_value_t = 800)]
    pub width: u32,
    #[arg(long, default_value_t = 600)]
    pub height: u32,
}
