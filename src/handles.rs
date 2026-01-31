use crate::args::{Args, Subcommands};
use crate::types::DirectoryNode;
use crate::utils::{is_bisheng_project, is_hidden_entry, path_to_dir_string};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

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
                    handle_cmd_add(dir)?;
                }
            }

            Ok(())
        }
    }
}

/// 子命令 Add
fn handle_cmd_add(dir: PathBuf) -> anyhow::Result<()> {
    let path = fs::canonicalize(&dir)?;
    // println!("Add directory: {:?}, {:?}", dir, path);

    // 直接调用递归函数，由内部统一判断
    if let Some(directory_node) = get_bs_dir(path)? {
        println!("{:#?}", directory_node);
    } else {
        println!("No Bisheng projects found in the specified directory.");
    }

    Ok(())
}

/// 获取Bisheng目录
fn get_bs_dir(path: PathBuf) -> anyhow::Result<Option<DirectoryNode>> {
    let name = path_to_dir_string(&path);
    let matched = is_bisheng_project(&path);

    // 1. 如果当前目录匹配成功，直接返回该节点，不再递归子目录
    if matched {
        return Ok(Some(DirectoryNode {
            path,
            name,
            matched,
            child_dirs: vec![],
        }));
    }

    // 2. 如果当前不匹配，尝试读取子目录
    let mut child_dirs = vec![];
    let entries = fs::read_dir(&path)?;

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        // 忽略非目录、隐藏目录
        if !file_type.is_dir() || is_hidden_entry(&entry) {
            continue;
        }

        if let Some(child_node) = get_bs_dir(entry.path())? {
            child_dirs.push(child_node);
        }
    }

    // 3. 关键逻辑：如果当前目录不匹配，且没有任何子目录包含项目，则返回 None
    if child_dirs.is_empty() {
        return Ok(None);
    }

    // 4. 当前目录不匹配，但有子目录包含项目，返回当前节点以维持树结构
    Ok(Some(DirectoryNode {
        path,
        name,
        matched: false,
        child_dirs,
    }))
}
