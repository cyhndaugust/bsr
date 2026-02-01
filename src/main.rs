//! BSR (Bisheng Rust) CLI 入口程序

use bsr::cli::Args;
use bsr::commands::handle_subcommands;
use clap::Parser;

fn main() {
    // 解析命令行参数
    let config = Args::parse();

    // 执行子命令逻辑
    if let Err(err) = handle_subcommands(config) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
