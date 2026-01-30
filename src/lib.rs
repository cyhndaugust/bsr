pub mod args;

pub use args::Args;
pub use args::Subcommands;
use clap::Parser;

pub fn run() {
    let args = Args::parse();

    match args.command {
        Some(Subcommands::Add { dir }) => {
            println!("Add directory: {}", dir);
        }
        None => {
            println!("No command specified.");
        }
    }
}
