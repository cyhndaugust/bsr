use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Bisheng Rust CLI")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Subcommands>,
}

#[derive(Debug, Subcommand)]
pub enum Subcommands {
    #[command(about = "Add a directory to the Bisheng project index.")]
    Add {
        #[arg(help = "A directory.")]
        dir: String,
    },
}
