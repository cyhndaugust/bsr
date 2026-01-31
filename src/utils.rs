use std::fs::DirEntry;
use std::path::Path;

/// 判断项目是否是Bisheng项目
pub fn is_bisheng_project(path: &Path) -> bool {
    let condition1 = path.join("srcTemplate");
    let condition2 = path.join("package.json");

    // 满足条件证明是Bisheng项目
    condition1.exists() && condition2.exists()
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
