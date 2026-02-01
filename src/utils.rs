use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use git2::Repository;

/// 判断项目是否是Bisheng项目
pub fn is_bisheng_project(path: &Path) -> bool {
    // let condition1 = path.join("srcTemplate");
    let condition2 = path.join("package.json");

    // 满足条件证明是Bisheng项目
    /* condition1.exists() &&  */
    condition2.exists()
}

/// 是否是隐藏文件或目录
pub fn is_hidden_entry(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// 从路径获取当前目录名
pub fn path_to_dir_string(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// 从路径获取当前目录名，若路径以HOME目录开头，则替换为 ~
pub fn get_tilde_path(path: &Path) -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            let mut p = PathBuf::from("~");
            p.push(stripped);
            return p;
        }
    }
    path.to_path_buf()
}

/// 判断路径是否是git仓库
pub fn is_git_repo(path: &Path) -> bool {
    // 非目录，不是git仓库
    if !path.is_dir() {
        return false;
    }

    // 目录下有.git目录，是git仓库
    if path.join(".git").exists() {
        return true;
    }

    // 目录下没有.git目录，尝试打开仓库，若成功则是git仓库
    Repository::open(path).is_ok()
}
