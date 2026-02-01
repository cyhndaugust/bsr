//! List 子命令处理逻辑

use crate::core::flatten_projects;
use crate::storage::read_project_file;
use crate::utils::get_tilde_path;
use inquire::Select;

/// 处理 'list' 命令
///
/// 从存储中读取已关联的毕昇项目并提供交互式选择列表。
pub fn handle() -> anyhow::Result<()> {
    if let Some(saved_data) = read_project_file()? {
        let mut projects = vec![];
        flatten_projects(&saved_data, &mut projects);

        if projects.is_empty() {
            eprintln!("No Bisheng projects found in the storage.");
            return Ok(());
        }

        let selected = Select::new("Select a project to view path:", projects).prompt()?;
        println!(
            "Selected project path: {:?}",
            get_tilde_path(&selected.path)
        );
    } else {
        println!("No projects stored. Use 'add' command first.");
    }

    Ok(())
}
