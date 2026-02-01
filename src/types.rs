//! 类型定义模块
//!
//! 包含项目中使用的各种数据结构和类型。

use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

/// 目录节点（树形结构）
///
/// 用于表示扫描过程中的目录层级，标记哪些目录是匹配到的毕昇项目。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectoryNode {
    /// 目录的绝对路径
    pub path: PathBuf,
    /// 当前目录名称
    pub name: String,
    /// 是否是匹配到的毕昇项目目录
    pub matched: bool,
    /// 子目录节点列表
    pub child_dirs: Vec<DirectoryNode>,
}

impl DirectoryNode {
    /// 创建一个新的目录节点
    ///
    /// # Arguments
    /// * `path` - 目录路径
    /// * `name` - 目录名称
    pub fn from(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            matched: false,
            child_dirs: vec![],
        }
    }
}

/// 项目选项结构体
///
/// 用于在交互式列表中显示的项目简略信息。
#[derive(Clone)]
pub struct ProjectOption {
    /// 项目名称
    pub name: String,
    /// 项目完整路径
    pub path: PathBuf,
    /// 项目父级目录路径（用于辅助显示）
    pub parent_path: PathBuf,
}

impl Display for ProjectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.parent_path.to_string_lossy())
    }
}
