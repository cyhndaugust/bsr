use std::fs;
use crate::args::{Args, Subcommands};
use clap::Parser;

/// 运行程序.
pub fn run() {
    let config = Args::parse();

    if let Err(err) = handle_subcommands(config) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    };
}

/// 处理子命令逻辑
fn handle_subcommands(args: Args) -> anyhow::Result<()> {
    match args.command {
        None => Ok(()),
        Some(command) => {
            match command {
                Subcommands::Add { dir } => {
                    let path = fs::canonicalize(&dir)?;
                    println!("Add directory: {:?}, {:?}", dir, path);
                }
            }

            Ok(())
        }
    }
}
