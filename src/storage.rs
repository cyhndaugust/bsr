//! 数据存储模块
//!
//! 负责将扫描到的项目信息持久化到本地文件系统中。

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;

use crate::types::DirectoryNode;

const FILE_NAME: &str = "projects.json";
const COMPARE_FILE_NAME: &str = "compare_pending";

/// 获取配置目录
pub fn get_config_dir() -> PathBuf {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".config");
        path.push("bsr");

        // 尝试创建目录（如果不存在）。
        if !path.exists() && fs::create_dir_all(&path).is_err() {
            return PathBuf::from(".");
        }

        // 检查写权限
        let test_file = path.join(".perm_test");
        if fs::write(&test_file, "").is_err() {
            return PathBuf::from(".");
        }
        let _ = fs::remove_file(test_file);

        return path;
    }
    PathBuf::from(".")
}

/// 获取报告存放目录
pub fn get_reports_dir() -> PathBuf {
    let mut path = get_config_dir();
    path.push("reports");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

/// 获取存储文件的绝对路径
pub fn get_file_path() -> PathBuf {
    let mut path = get_config_dir();
    path.push(FILE_NAME);
    path
}

/// 获取待比对目录存储文件的绝对路径
pub fn get_compare_file_path() -> PathBuf {
    let mut path = get_config_dir();
    path.push(COMPARE_FILE_NAME);
    path
}

/// 从文件中读取存储的项目数据
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
pub fn write_project_file(dir_node: DirectoryNode) -> anyhow::Result<()> {
    let path = get_file_path();
    let dn_json = serde_json::to_string_pretty(&dir_node)?;

    // 尝试原子写入
    let dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    if let Ok(mut temp_file) = tempfile::NamedTempFile::new_in(dir)
        && temp_file.write_all(dn_json.as_bytes()).is_ok()
    {
        if temp_file.persist(&path).is_err() {
            fs::write(&path, dn_json)?;
        }
        return Ok(());
    }

    fs::write(&path, dn_json)?;

    Ok(())
}

/// 读取待比对的目录路径列表
pub fn read_pending_compare() -> anyhow::Result<Vec<PathBuf>> {
    let path = get_compare_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    // 尝试解析为 JSON 数组，如果失败（旧格式），则回退到读取单个路径并包装为 Vec
    if let Ok(dirs) = serde_json::from_str::<Vec<PathBuf>>(&content) {
        Ok(dirs)
    } else {
        Ok(vec![PathBuf::from(content.trim())])
    }
}

/// 写入待比对的目录路径列表
pub fn write_pending_compare(dirs: &[PathBuf]) -> anyhow::Result<()> {
    let path = get_compare_file_path();
    // 确保存储的是绝对路径
    let abs_dirs: Vec<PathBuf> = dirs
        .iter()
        .map(|d| fs::canonicalize(d).unwrap_or_else(|_| d.clone()))
        .collect();
    let json = serde_json::to_string(&abs_dirs)?;
    fs::write(&path, json)?;
    Ok(())
}

/// 清除待比对的目录路径
pub fn clear_pending_compare() -> anyhow::Result<()> {
    let path = get_compare_file_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
