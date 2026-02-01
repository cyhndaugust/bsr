use anyhow::anyhow;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Bisheng Rust CLI")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Subcommands>,
}

#[derive(Debug, Subcommand)]
pub enum Subcommands {
    #[command(about = "Add a directory to retrieve all Bisheng project.")]
    Add {
        #[arg(help = "A directory.")]
        #[arg(value_parser = validate_dir)]
        dir: PathBuf,
    },
    #[command(about = "List all added directories.")]
    #[command(alias = "ls")]
    List,
    #[command(about = "Show status of all originSource repo.")]
    Status {
        #[arg(help = "A directory.")]
        #[arg(value_parser = validate_dir)]
        #[arg(default_value = ".")]
        dir: PathBuf,

        #[arg(
            short,
            long,
            help = "Show all repositories, including clean ones.",
            default_value_t = false
        )]
        all: bool,
    },
}

/// 验证参数 是否是目录
fn validate_dir(dir: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(dir);
    if !path.exists() {
        return Err(anyhow!("Invalid directory"));
    }
    Ok(path.to_path_buf())
}
