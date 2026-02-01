//! 命令处理模块
//!
//! 包含所有子命令的具体实现逻辑。

pub mod add;
pub mod list;
pub mod status;

use crate::cli::{Args, Subcommands};

/// 处理命令行输入的入口函数
///
/// # Arguments
/// * `args` - 解析后的命令行参数
pub fn handle_subcommands(args: Args) -> anyhow::Result<()> {
    match args.command {
        None => Ok(()),
        Some(command) => match command {
            Subcommands::Add { dir } => add::handle(dir),
            Subcommands::List => list::handle(),
            Subcommands::Status { dir, all } => status::handle(dir, all),
        },
    }
}
