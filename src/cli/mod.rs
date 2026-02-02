//! CLI 参数解析模块
//!
//! 该模块负责定义和解析命令行参数，使用 `clap` 库。

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// 命令行顶级参数结构体
#[derive(Debug, Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Bisheng Rust CLI")]
pub struct Args {
    /// 子命令
    #[command(subcommand)]
    pub command: Option<Subcommands>,
}

/// 支持的子命令列表
#[derive(Debug, Subcommand)]
pub enum Subcommands {
    /// 扫描并添加指定目录下的所有毕昇项目
    #[command(about = "Add a directory to retrieve all Bisheng project.")]
    Add {
        /// 待扫描的目录路径
        #[arg(help = "A directory.")]
        #[arg(value_parser = validate_dir)]
        dir: PathBuf,
    },
    /// 列出所有已关联的毕昇项目
    #[command(about = "List all added directories.")]
    #[command(alias = "ls")]
    List,
    /// 检查 originSource 下所有组件的 Git 状态
    #[command(about = "Show status of all originSource repo.")]
    Status {
        /// 毕昇根目录（默认为当前目录）
        #[arg(help = "A directory.")]
        #[arg(value_parser = validate_dir)]
        #[arg(default_value = ".")]
        dir: PathBuf,

        /// 是否显示所有仓库（包括干净的）
        #[arg(
            short,
            long,
            help = "Show all repositories, including clean ones.",
            default_value_t = false
        )]
        all: bool,
    },
    /// 对比两个目录的差异
    #[command(about = "Compare two directories.")]
    Compare {
        /// 要对比的目录路径（可选，默认为空）
        #[arg(help = "A directory to compare.")]
        #[arg(value_parser = validate_dir)]
        dir: Option<PathBuf>,

        /// 显示上下文行数
        #[arg(
            short = 'C',
            long = "context",
            help = "Lines of context to show.",
            default_value_t = 3
        )]
        context: usize,

        /// 指定对比 originSource 下的特定子目录
        #[arg(
            long = "originSource",
            visible_alias = "os",
            help = "Compare a specific subdirectory relative to originSource."
        )]
        origin_source: Option<PathBuf>,
    },
    /// 升级 bsr 到最新版本
    #[command(about = "Upgrade bsr to the latest version.")]
    Upgrade,
}

/// 验证给定的路径字符串是否为存在的目录
///
/// # Arguments
/// * `dir` - 路径字符串
///
/// # Returns
/// * `Result<PathBuf>` - 验证成功返回 PathBuf，否则返回错误
fn validate_dir(dir: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(dir);
    if !path.exists() {
        return Err(anyhow!("Invalid directory: {}", dir));
    }
    if !path.is_dir() {
        return Err(anyhow!("Path is not a directory: {}", dir));
    }
    Ok(path.to_path_buf())
}
