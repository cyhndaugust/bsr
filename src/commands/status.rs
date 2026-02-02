//! Status 子命令处理逻辑

use crate::core::find_git_repos;
use crate::utils::{get_git_status, get_tilde_path, path_to_dir_string};
use anyhow::anyhow;
use colored::Colorize;
use std::path::PathBuf;

/// 处理 'status' 命令
///
/// 检查毕昇项目 originSource 目录下所有组件的 Git 状态。
pub fn handle(dir: PathBuf, all: bool) -> anyhow::Result<()> {
    let origin_source = dir.join("originSource");

    if !origin_source.exists() {
        return Err(anyhow!("Not found Bisheng Dependencies (originSource directory missing)"));
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
                Ok(status) => {
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
                            println!("  - {}", file);
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
