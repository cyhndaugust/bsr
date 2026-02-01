use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use git2::{Repository, StatusOptions};

/// Git 状态信息
#[derive(Debug)]
pub struct GitStatusInfo {
    pub branch: String,
    pub is_clean: bool,
    pub modified_files: Vec<String>,
    pub stash_count: usize,
}

/// 获取 git 仓库的状态信息
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
                    if let Some(tag_id) = obj
                        .as_tag()
                        .map(|t| t.target_id())
                        .or_else(|| Some(obj.id()))
                    {
                        if Some(tag_id) == head_id {
                            tag_name = Some(name.to_string());
                            break;
                        }
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
