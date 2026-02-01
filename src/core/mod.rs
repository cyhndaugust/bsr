//! 核心业务逻辑模块
//!
//! 包含项目扫描、Git 仓库查找和数据扁平化处理等核心功能。

use crate::types::{DirectoryNode, ProjectOption};
use crate::utils::{get_tilde_path, is_bisheng_project, is_hidden_entry, path_to_dir_string};
use std::fs;
use std::path::PathBuf;

/// 递归查找指定路径下的所有 Git 仓库
///
/// # Arguments
/// * `path` - 开始查找的路径
/// * `repos` - 用于存储找到的 Git 仓库路径的向量
pub fn find_git_repos(path: &PathBuf, repos: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if crate::utils::is_git_repo(path) {
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

/// 递归遍历目录树，将匹配的项目提取并扁平化到向量中
///
/// # Arguments
/// * `node` - 目录树节点
/// * `projects` - 用于存储扁平化后的项目选项的向量
pub fn flatten_projects(node: &DirectoryNode, projects: &mut Vec<ProjectOption>) {
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

/// 递归扫描目录，构建包含毕昇项目的目录树
///
/// # Arguments
/// * `path` - 扫描的起始路径
///
/// # Returns
/// * `Result<Option<DirectoryNode>>` - 如果找到项目，返回目录树节点，否则返回 None
pub fn scan_bs_projects(path: PathBuf) -> anyhow::Result<Option<DirectoryNode>> {
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

        if let Some(child_node) = scan_bs_projects(entry.path())? {
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
