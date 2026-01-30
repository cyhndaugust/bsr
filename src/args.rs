use anyhow::anyhow;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        #[arg(value_parser = validate_dir)]
        dir: PathBuf,
    },
}

/// 验证参数 是否是目录
fn validate_dir(dir: &str) -> anyhow::Result<PathBuf> {
    let path = PathBuf::from(dir);

    if !path.exists() {
        return Err(anyhow!("Invalid directory"));
    }

    Ok(path)
}
