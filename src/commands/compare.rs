//! 对比命令实现
//!
//! 处理 `bsr compare` 命令，负责目录对比逻辑。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use colored::Colorize;
use ignore::WalkBuilder;
use inquire::Select;
use similar::{ChangeTag, TextDiff};

use crate::storage;

/// 处理 compare 命令
pub fn handle(target_dir: PathBuf, context: usize) -> Result<()> {
    let abs_target_dir = fs::canonicalize(&target_dir)
        .with_context(|| format!("Failed to canonicalize path: {:?}", target_dir))?;

    let pending_dir_opt = storage::read_pending_compare()?;

    if let Some(pending_dir) = pending_dir_opt {
        // 待比区已有目录
        if pending_dir == abs_target_dir {
            println!(
                "{}",
                "The directory is already in the waiting area.".yellow()
            );
            return Ok(());
        }

        println!(
            "Waiting area directory: {}",
            pending_dir.display().to_string().cyan()
        );
        println!(
            "Current directory:      {}",
            abs_target_dir.display().to_string().cyan()
        );

        let options = vec![
            "Start comparison",
            "Replace waiting area with current directory",
            "Cancel",
        ];

        let ans = Select::new(
            "A directory is already in the waiting area. What do you want to do?",
            options,
        )
        .prompt()?;

        match ans {
            "Start comparison" => {
                // 开始对比
                println!("{}", "\nStarting comparison...\n".green().bold());
                perform_comparison(&pending_dir, &abs_target_dir, context)?;
            }
            "Replace waiting area with current directory" => {
                storage::write_pending_compare(&abs_target_dir)?;
                println!("{}", "Updated waiting area with current directory.".green());
            }
            _ => {
                println!("Cancelled.");
            }
        }
    } else {
        // 待比区为空，直接添加
        storage::write_pending_compare(&abs_target_dir)?;
        println!(
            "Added {} to the waiting area.",
            abs_target_dir.display().to_string().green()
        );
    }

    Ok(())
}

/// 执行对比逻辑
fn perform_comparison(dir_a: &Path, dir_b: &Path, context: usize) -> Result<()> {
    // 1. 获取文件列表
    let files_a = list_files(dir_a)?;
    let files_b = list_files(dir_b)?;

    // 2. 结构对比
    if !check_structure_similarity(&files_a, &files_b)? {
        println!(
            "{}",
            "The directory structures are too different. Comparison aborted."
                .red()
                .bold()
        );
        return Ok(());
    }

    // 3. 详细对比
    compare_directories(dir_a, dir_b, files_a, files_b, context)?;

    Ok(())
}

/// 检查目录结构相似度
fn check_structure_similarity(files_a: &[PathBuf], files_b: &[PathBuf]) -> Result<bool> {
    let set_a: HashSet<_> = files_a.iter().collect();
    let set_b: HashSet<_> = files_b.iter().collect();

    let intersection_count = set_a.intersection(&set_b).count();
    let union_count = set_a.union(&set_b).count();

    if union_count == 0 {
        return Ok(true); // 都是空目录，视为相似
    }

    let similarity = intersection_count as f64 / union_count as f64;

    // 阈值设为 0.3 (30%)，避免过于严格
    Ok(similarity >= 0.3)
}

/// 列出目录下所有文件（相对路径），遵循 gitignore
fn list_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .hidden(true) // 忽略隐藏文件
        .git_ignore(true) // 遵循 gitignore
        .filter_entry(|e| {
            // 过滤 .git 目录
            if e.file_name() == ".git" {
                return false;
            }
            true
        })
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    // 获取相对路径
                    if let Ok(rel_path) = entry.path().strip_prefix(root) {
                        files.push(rel_path.to_path_buf());
                    }
                }
            }
            Err(err) => eprintln!("Error walking directory: {}", err),
        }
    }
    Ok(files)
}

struct CompareContext {
    git_version_cache: HashMap<PathBuf, GitVersion>,
    git_root_cache: HashMap<PathBuf, Option<PathBuf>>,
}

impl CompareContext {
    fn new() -> Self {
        Self {
            git_version_cache: HashMap::new(),
            git_root_cache: HashMap::new(),
        }
    }
}

/// 详细对比两个目录
fn compare_directories(
    root_a: &Path,
    root_b: &Path,
    files_a: Vec<PathBuf>,
    files_b: Vec<PathBuf>,
    context: usize,
) -> Result<()> {
    let mut all_files: HashSet<PathBuf> = HashSet::new();
    for f in &files_a {
        all_files.insert(f.clone());
    }
    for f in &files_b {
        all_files.insert(f.clone());
    }

    let mut sorted_files: Vec<_> = all_files.into_iter().collect();
    sorted_files.sort();

    // 上下文缓存
    let mut ctx_a = CompareContext::new();
    let mut ctx_b = CompareContext::new();

    // 按目录分组结果
    let mut results: HashMap<PathBuf, Vec<String>> = HashMap::new();

    for rel_path in sorted_files {
        let path_a = root_a.join(&rel_path);
        let path_b = root_b.join(&rel_path);

        let exists_a = files_a.contains(&rel_path);
        let exists_b = files_b.contains(&rel_path);

        let parent = rel_path.parent().unwrap_or(Path::new("")).to_path_buf();

        if exists_a && !exists_b {
            results.entry(parent.clone()).or_default().push(format!(
                "{} {}",
                "Deleted:".red(),
                rel_path.display()
            ));
        } else if !exists_a && exists_b {
            results.entry(parent.clone()).or_default().push(format!(
                "{} {}",
                "Added:  ".green(),
                rel_path.display()
            ));
        } else {
            // Both exist, compare content
            if is_origin_source(&rel_path) {
                // originSource 特殊对比
                if let Some(diff) =
                    compare_origin_source(&path_a, &path_b, &mut ctx_a, &mut ctx_b, context)?
                {
                    results.entry(parent.clone()).or_default().push(format!(
                        "{} (originSource)\n{}",
                        rel_path.display().to_string().yellow(),
                        diff
                    ));
                }
            } else {
                // 普通文件对比
                if is_binary(&path_a)? || is_binary(&path_b)? {
                    let meta_a = fs::metadata(&path_a)?;
                    let meta_b = fs::metadata(&path_b)?;
                    if meta_a.len() != meta_b.len() {
                        results.entry(parent.clone()).or_default().push(format!(
                            "{} {} (binary files differ)",
                            "M".yellow(),
                            rel_path.display()
                        ));
                    }
                } else {
                    let content_a = fs::read_to_string(&path_a).unwrap_or_default();
                    let content_b = fs::read_to_string(&path_b).unwrap_or_default();

                    if content_a != content_b {
                        let diff_output = generate_diff(
                            &content_a,
                            &content_b,
                            &rel_path.to_string_lossy(),
                            context,
                        );
                        results.entry(parent.clone()).or_default().push(diff_output);
                    }
                }
            }
        }
    }

    // 输出结果
    if results.is_empty() {
        println!("{}", "No differences found.".green());
    } else {
        let mut sorted_dirs: Vec<_> = results.keys().collect();
        sorted_dirs.sort();

        for dir in sorted_dirs {
            println!("{}", format!("Directory: {}", dir.display()).blue().bold());
            for line in &results[dir] {
                println!("{}", line);
            }
            println!();
        }
    }

    Ok(())
}

fn is_origin_source(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "originSource")
}

fn get_cached_git_root(path: &Path, ctx: &mut CompareContext) -> Option<PathBuf> {
    if let Some(root) = ctx.git_root_cache.get(path) {
        return root.clone();
    }
    let root = find_git_root(path);
    ctx.git_root_cache.insert(path.to_path_buf(), root.clone());
    root
}

fn get_cached_git_version(repo_path: &Path, ctx: &mut CompareContext) -> Result<GitVersion> {
    if let Some(ver) = ctx.git_version_cache.get(repo_path) {
        return Ok(ver.clone());
    }
    let ver = get_git_version(repo_path)?;
    ctx.git_version_cache
        .insert(repo_path.to_path_buf(), ver.clone());
    Ok(ver)
}

/// 对比 originSource 目录下的仓库
fn compare_origin_source(
    path_a: &Path,
    path_b: &Path,
    ctx_a: &mut CompareContext,
    ctx_b: &mut CompareContext,
    context: usize,
) -> Result<Option<String>> {
    let repo_root_a = get_cached_git_root(path_a, ctx_a);
    let repo_root_b = get_cached_git_root(path_b, ctx_b);

    if let (Some(root_a), Some(root_b)) = (repo_root_a, repo_root_b) {
        // 检查版本
        let ver_a = get_cached_git_version(&root_a, ctx_a)?;
        let ver_b = get_cached_git_version(&root_b, ctx_b)?;

        // 只有当都是 tag，且 tag 相同，且 clean 时，才跳过
        if let (GitVersion::Tag(t1, clean1), GitVersion::Tag(t2, clean2)) = (&ver_a, &ver_b)
            && t1 == t2
            && *clean1
            && *clean2
        {
            return Ok(None); // 视为相同
        }
    }

    // 否则回退到内容对比
    let content_a = fs::read_to_string(path_a).unwrap_or_default();
    let content_b = fs::read_to_string(path_b).unwrap_or_default();

    if content_a != content_b {
        Ok(Some(generate_diff(
            &content_a,
            &content_b,
            &path_a.file_name().unwrap().to_string_lossy(),
            context,
        )))
    } else {
        Ok(None)
    }
}

fn find_git_root(path: &Path) -> Option<PathBuf> {
    let mut curr = path;
    if curr.is_file() {
        curr = curr.parent()?;
    }

    loop {
        if curr.join(".git").exists() {
            return Some(curr.to_path_buf());
        }
        if let Some(parent) = curr.parent() {
            curr = parent;
        } else {
            break;
        }
    }
    None
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum GitVersion {
    Tag(String, bool),    // tag_name, is_clean
    Branch(String, bool), // branch_name, is_clean
    Commit(String, bool), // commit_hash, is_clean
}

fn get_git_version(repo_path: &Path) -> Result<GitVersion> {
    use git2::Repository;

    let repo = Repository::open(repo_path).map_err(|_| anyhow::anyhow!("Not a git repo"))?;
    let head = repo.head()?;
    let is_clean = repo.statuses(None)?.is_empty();

    let head_name = head.shorthand().unwrap_or("HEAD").to_string();

    // 如果是 detached HEAD，尝试匹配 tag
    if repo.head_detached()? {
        let head_oid = head.target().unwrap();
        // 遍历 tags 看看有没有指向这个 oid 的
        let tags = repo.tag_names(None)?;
        for name in tags.iter().flatten() {
            let is_match = repo
                .revparse_single(name)
                .and_then(|obj| obj.peel_to_commit())
                .map(|commit| commit.id() == head_oid)
                .unwrap_or(false);

            if is_match {
                return Ok(GitVersion::Tag(name.to_string(), is_clean));
            }
        }
        return Ok(GitVersion::Commit(head_oid.to_string(), is_clean));
    }

    Ok(GitVersion::Branch(head_name, is_clean))
}

fn is_binary(path: &Path) -> Result<bool> {
    let mut file = fs::File::open(path)?;
    use std::io::Read;
    let mut buffer = [0; 1024];
    let n = file.read(&mut buffer)?;
    // 检查 null byte
    Ok(buffer[0..n].contains(&0))
}

fn generate_diff(text1: &str, text2: &str, filename: &str, context: usize) -> String {
    let diff = TextDiff::from_lines(text1, text2);
    let mut output = String::new();

    output.push_str(&format!("File: {}\n", filename));

    for (idx, group) in diff.grouped_ops(context).iter().enumerate() {
        if idx > 0 {
            output.push_str(&format!("{:-^1$}\n", "-", 80));
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let s = match change.tag() {
                    ChangeTag::Delete => change.old_index(),
                    ChangeTag::Insert => change.new_index(),
                    ChangeTag::Equal => change.old_index(),
                };
                let line_number = s.map(|i| (i + 1).to_string()).unwrap_or_default();
                output.push_str(&format!("{:>4} | ", line_number.dimmed()));

                for (emphasized, value) in change.values() {
                    if *emphasized {
                        match change.tag() {
                            ChangeTag::Delete => {
                                output.push_str(&format!("{}", value.white().on_red()))
                            }
                            ChangeTag::Insert => {
                                output.push_str(&format!("{}", value.white().on_green()))
                            }
                            ChangeTag::Equal => output.push_str(&format!("{}", value.dimmed())),
                        }
                    } else {
                        match change.tag() {
                            ChangeTag::Delete => output.push_str(&format!("{}", value.red())),
                            ChangeTag::Insert => output.push_str(&format!("{}", value.green())),
                            ChangeTag::Equal => output.push_str(&format!("{}", value.dimmed())),
                        }
                    }
                }

                if change.missing_newline() {
                    output.push('\n');
                }
            }
        }
    }
    output
}
