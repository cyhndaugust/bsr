use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;

use crate::types::DirectoryNode;

const FILE_NAME: &str = "projects.json";

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

/// 从文件中读取数据
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

/// 存储数据到文件中
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
