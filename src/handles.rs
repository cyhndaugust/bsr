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

    let matched = is_bisheng_project(&path);

    // 递归读取目录
    let directory_node = get_bs_dir(path, matched)?;
    println!("{:?}", directory_node);

    Ok(())
}

/// 获取Bisheng目录
fn get_bs_dir(path: PathBuf, matched: bool) -> anyhow::Result<DirectoryNode> {
    let name = path_to_dir_string(&path);
    let mut child_dirs = vec![];

    // 当前目录就是Bisheng项目
    if matched {
        println!("Bisheng Project");
        return Ok(DirectoryNode {
            path,
            name,
            matched,
            child_dirs,
        });
    }

    let entries = fs::read_dir(&path)?;

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        // 忽略非目录、隐藏目录
        if !file_type.is_dir() || is_hidden_entry(&entry) {
            continue;
        }
        // println!("entry={:?}", entry);

        let p = entry.path();
        // 是否匹配到了
        let _matched = is_bisheng_project(&p);

        if _matched {
            let n = path_to_dir_string(&p);
            child_dirs.push(DirectoryNode {
                path: p,
                name: n,
                matched: true,
                child_dirs: vec![],
            });
        } else {
            let child_dir = get_bs_dir(p, false)?;
            child_dirs.push(child_dir);
        }
    }

    Ok(DirectoryNode {
        path,
        name,
        matched,
        child_dirs,
    })
}
