//! Add 子命令处理逻辑

use crate::core::scan_bs_projects;
use crate::storage::write_project_file;
use std::fs;
use std::path::PathBuf;

/// 处理 'add' 命令
///
/// 扫描指定目录并将其中的毕昇项目信息持久化到存储中。
pub fn handle(dir: PathBuf) -> anyhow::Result<()> {
    let path = fs::canonicalize(&dir)?;

    if let Some(directory_node) = scan_bs_projects(path)? {
        if let Err(err) = write_project_file(directory_node) {
            eprintln!("Error writing project file: {}", err);
        } else {
            println!("Successfully added projects from directory.");
        }
    } else {
        eprintln!("No Bisheng projects found in the specified directory.");
    }

    Ok(())
}
