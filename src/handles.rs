use crate::args::{Args, Subcommands};
use crate::storage::{read_project_file, write_project_file};
use crate::types::{DirectoryNode, ProjectOption};
use crate::utils::{
    get_git_status, get_tilde_path, is_bisheng_project, is_git_repo, is_hidden_entry,
    path_to_dir_string,
};
use anyhow::{Ok, anyhow};
use clap::Parser;
use colored::Colorize;
use inquire::Select;
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
                Subcommands::List => {
                    handle_cmd_list()?;
                }
                Subcommands::Status { dir, all } => {
                    handle_cmd_status(dir, all)?;
                }
            }

            Ok(())
        }
    }
}

/// 子命令 Add
fn handle_cmd_add(dir: PathBuf) -> anyhow::Result<()> {
    let path = fs::canonicalize(&dir)?;

    if let Some(directory_node) = get_bs_dir(path)? {
        if let Err(err) = write_project_file(directory_node) {
            eprintln!("Error writing project file: {}", err);
        }
    } else {
        eprintln!("No Bisheng projects found in the specified directory.");
    }

    Ok(())
}

/// 子命令 List
/// 列出所有已经关联的Bisheng项目
fn handle_cmd_list() -> anyhow::Result<()> {
    if let Some(saved_data) = read_project_file()? {
        let mut projects = vec![];
        flatten_projects(&saved_data, &mut projects);

        if projects.is_empty() {
            eprintln!("No Bisheng projects found in the specified directory.");
            return Ok(());
        }

        let selected = Select::new("Select a project to add:", projects).prompt()?;
        println!(
            "Selected project path: {:?}",
            get_tilde_path(&selected.path)
        );
    };

    Ok(())
}

/// 子命令 Status
/// 列出所有originSource下的组件的git状态
fn handle_cmd_status(dir: PathBuf, all: bool) -> anyhow::Result<()> {
    let origin_source = dir.join("originSource");

    if !origin_source.exists() {
        return Err(anyhow!("Not found Bisheng Dependencies"));
    }

    println!(
        "\n{}",
        format!(
            "Checking git status in: {:?}",
            get_tilde_path(&origin_source)
        )
        .bold()
    );

    let mut repos = Vec::new();
    find_git_repos(&origin_source, &mut repos)?;

    if repos.is_empty() {
        println!("{}", "No git repositories found in originSource.".yellow());
    } else {
        let mut shown_count = 0;
        for path in repos {
            let repo_name = path_to_dir_string(&path);

            match get_git_status(&path) {
                anyhow::Result::Ok(status) => {
                    if !all && status.is_clean {
                        continue;
                    }

                    shown_count += 1;
                    let status_str = if status.is_clean {
                        "clean".green()
                    } else {
                        "modified".red()
                    };

                    println!(
                        "\n{} [{}] ({})",
                        repo_name.bold().cyan(),
                        status.branch.blue(),
                        status_str
                    );

                    if status.stash_count > 0 {
                        println!("  {} {} stashes", "→".yellow(), status.stash_count);
                    }

                    if !status.is_clean {
                        for file in status.modified_files {
                            println!("  {} {}", "-", file);
                        }
                    }
                }
                Err(e) => {
                    println!("{} {}: {}", "Error".red(), repo_name, e);
                }
            }
        }

        if shown_count == 0 && !all {
            println!(
                "\n{}",
                "All repositories are clean. Use --all to show them.".green()
            );
        }
    }

    println!();
    Ok(())
}

/// 递归查找 git 仓库
fn find_git_repos(path: &PathBuf, repos: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if is_git_repo(path) {
        repos.push(path.clone());
        return Ok(());
    }

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let child_path = entry.path();

        if child_path.is_dir() && !is_hidden_entry(&entry) {
            find_git_repos(&child_path, repos)?;
        }
    }

    Ok(())
}

/// 递归遍历目录树，将匹配的项目拉齐到一个向量中
fn flatten_projects(node: &DirectoryNode, projects: &mut Vec<ProjectOption>) {
    if node.matched {
        let parent_path = node
            .path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"));
        projects.push(ProjectOption {
            name: node.name.clone(),
            path: node.path.clone(),
            parent_path: get_tilde_path(&parent_path),
        });
    }

    for child in &node.child_dirs {
        flatten_projects(child, projects);
    }
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
