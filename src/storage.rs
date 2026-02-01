//! 数据存储模块
//!
//! 负责将扫描到的项目信息持久化到本地文件系统中。

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;

use crate::types::DirectoryNode;

const FILE_NAME: &str = "projects.json";

/// 获取存储文件的绝对路径
///
/// 优先尝试在用户的配置目录下创建 `~/.config/bsr/projects.json`。
/// 如果失败，则回退到当前目录下的 `projects.json`。
pub fn get_file_path() -> PathBuf {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".config");
        path.push("bsr");

        // 尝试创建目录（如果不存在）。如果创建失败（例如权限被拒绝），回退到本地文件。
        if !path.exists() {
            if fs::create_dir_all(&path).is_err() {
                return PathBuf::from(FILE_NAME);
            }
        }
        path.push(FILE_NAME);
        return path;
    }
    PathBuf::from(FILE_NAME)
}

/// 从文件中读取存储的项目数据
///
/// # Returns
/// * `Result<Option<DirectoryNode>>` - 读取成功返回目录树根节点，文件不存在或内容为空返回 None
pub fn read_project_file() -> anyhow::Result<Option<DirectoryNode>> {
    let path = get_file_path();

    if !path.exists() {
        return Ok(None);
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read file: {:?}", path))?;

    if content.trim().is_empty() {
        return Ok(None);
    }

    let projects =
        serde_json::from_str::<DirectoryNode>(&content).with_context(|| "Failed to parse JSON")?;
    Ok(Some(projects))
}

/// 将项目数据写入存储文件
///
/// 使用临时文件进行原子写入，以保证数据的一致性。
///
/// # Arguments
/// * `dir_node` - 要存储的目录树根节点
pub fn write_project_file(dir_node: DirectoryNode) -> anyhow::Result<()> {
    let path = get_file_path();
    let dn_json = serde_json::to_string_pretty(&dir_node)?;

    // 尝试原子写入
    let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    if let Ok(mut temp_file) = tempfile::NamedTempFile::new_in(dir) {
        if temp_file.write_all(dn_json.as_bytes()).is_ok() {
            if let Err(_) = temp_file.persist(&path) {
                // 如果持久化失败（例如跨设备链接错误或权限问题），回退到直接写入
                fs::write(&path, dn_json)?;
            }
            return Ok(());
        }
    }

    // 如果临时文件创建/写入失败，回退到直接写入
    fs::write(&path, dn_json)?;

    Ok(())
}
