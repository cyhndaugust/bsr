//! 通用工具函数模块
//!
//! 包含路径处理、Git 状态获取以及文件系统检查等辅助函数。

use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use git2::{Repository, StatusOptions};

/// Git 状态信息结构体
#[derive(Debug)]
pub struct GitStatusInfo {
    /// 当前分支名、Tag 或 Commit ID
    pub branch: String,
    /// 工作区是否干净
    pub is_clean: bool,
    /// 已修改的文件列表
    pub modified_files: Vec<String>,
    /// Stash 的数量
    pub stash_count: usize,
}

/// 获取指定路径下 Git 仓库的状态信息
///
/// # Arguments
/// * `path` - Git 仓库根目录路径
///
/// # Returns
/// * `Result<GitStatusInfo>` - 成功返回状态信息，否则返回错误
pub fn get_git_status(path: &Path) -> anyhow::Result<GitStatusInfo> {
    let mut repo = Repository::open(path)?;

    // 获取当前分支、Tag 或 Commit
    let branch = {
        if repo.head_detached().unwrap_or(false) {
            // 如果是分离头指针状态，尝试查找对应的 Tag
            let mut tag_name = None;
            let tags = repo.tag_names(None)?;
            let head_id = repo.head()?.target();

            for name in tags.iter().flatten() {
                if let Ok(obj) = repo.revparse_single(name) {
                    let tag_id_opt = obj
                        .as_tag()
                        .map(|t| t.target_id())
                        .or_else(|| Some(obj.id()));

                    if tag_id_opt.is_some() && tag_id_opt == head_id {
                        tag_name = Some(name.to_string());
                        break;
                    }
                }
            }

            tag_name.unwrap_or_else(|| {
                // 如果没有对应的 Tag，显示短 Commit ID
                head_id
                    .map(|id| id.to_string()[..7].to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            })
        } else {
            // 正常分支状态
            let head = repo.head();
            match head {
                Ok(ref reference) => reference.shorthand().unwrap_or("HEAD").to_string(),
                Err(_) => "Unknown".to_string(),
            }
        }
    };

    // 获取状态
    let modified_files = {
        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        let statuses = repo.statuses(Some(&mut status_options))?;

        let mut files = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                files.push(path.to_string());
            }
        }
        files
    };

    let is_clean = modified_files.is_empty();

    // 获取 stash 数量
    let mut stash_count = 0;
    repo.stash_foreach(|_, _, _| {
        stash_count += 1;
        true
    })?;

    Ok(GitStatusInfo {
        branch,
        is_clean,
        modified_files,
        stash_count,
    })
}

/// 判断指定路径是否为毕昇项目
///
/// 目前通过检查是否存在 `package.json` 来简单判断。
pub fn is_bisheng_project(path: &Path) -> bool {
    let condition2 = path.join("package.json");
    condition2.exists()
}

/// 检查文件系统条目是否为隐藏文件或目录（以 `.` 开头）
pub fn is_hidden_entry(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// 将路径转换为目录名称字符串
pub fn path_to_dir_string(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// 将路径中的 Home 目录部分替换为 `~`
///
/// # Arguments
/// * `path` - 原始路径
///
/// # Returns
/// * `PathBuf` - 转换后的路径
pub fn get_tilde_path(path: &Path) -> PathBuf {
    if let Some(home) = dirs::home_dir()
        && let Ok(stripped) = path.strip_prefix(&home)
    {
        return Path::new("~").join(stripped);
    }
    path.to_path_buf()
}

/// 判断指定路径是否为 Git 仓库
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
