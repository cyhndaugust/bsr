use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

/// 目录节点（树形结构）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectoryNode {
    pub path: PathBuf,                  // 绝对路径
    pub name: String,                   // 当前目录名称
    pub matched: bool,                  // 是否是匹配的目录，比如匹配到的Bisheng
    pub child_dirs: Vec<DirectoryNode>, // 子目录节点
}

impl DirectoryNode {
    /// 创建目录节点
    pub fn from(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            matched: false,
            child_dirs: vec![],
        }
    }
}

/// 项目选项（用于选择项目）
#[derive(Clone)]
pub struct ProjectOption {
    pub name: String,
    pub path: PathBuf,
    pub parent_path: PathBuf,
}

impl Display for ProjectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.parent_path.to_string_lossy())
    }
}
