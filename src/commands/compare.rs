//! 对比命令实现
//!
//! 处理 `bsr compare` 命令，负责目录对比逻辑。

use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;
use ignore::WalkBuilder;
use inquire::Select;
use similar::{ChangeTag, TextDiff};

use crate::storage;

/// 处理 compare 命令
pub fn handle(
    target_dir: Option<PathBuf>,
    context: usize,
    origin_source_subpath: Option<PathBuf>,
) -> Result<()> {
    let mut pending_dirs = storage::read_pending_compare()?;

    // 如果指定了 -os 参数，则必须已有两个待比对目录
    if let Some(subpath) = origin_source_subpath {
        if pending_dirs.len() != 2 {
            println!(
                "{}",
                "Error: Comparison with --originSource requires exactly two directories in the waiting area."
                    .red()
            );
            println!(
                "Current waiting area has {} directories.",
                pending_dirs.len()
            );
            return Ok(());
        }

        let dir_a = &pending_dirs[0];
        let dir_b = &pending_dirs[1];
        let path_a = dir_a.join("originSource").join(&subpath);
        let path_b = dir_b.join("originSource").join(&subpath);

        println!(
            "{}",
            format!(
                "\nComparing originSource subdirectory: {}\n",
                subpath.display()
            )
            .green()
            .bold()
        );
        println!("Dir A: {}", path_a.display().to_string().cyan());
        println!("Dir B: {}", path_b.display().to_string().cyan());

        if !path_a.exists() {
            println!("{}", format!("Path not found in Dir A: {:?}", path_a).red());
        }
        if !path_b.exists() {
            println!("{}", format!("Path not found in Dir B: {:?}", path_b).red());
        }

        if path_a.exists() && path_b.exists() {
            // 对子目录进行全量对比（递归）
            perform_comparison(&path_a, &path_b, context, true)?;
        }

        return Ok(());
    }

    // 如果未指定 target_dir，则尝试直接对比待比对区的两个目录
    if target_dir.is_none() {
        if pending_dirs.len() == 2 {
            println!(
                "{}",
                "\nNo directory specified. Comparing waiting area directories...\n"
                    .green()
                    .bold()
            );
            println!("Dir A: {}", pending_dirs[0].display().to_string().cyan());
            println!("Dir B: {}", pending_dirs[1].display().to_string().cyan());

            perform_main_comparison(&pending_dirs[0], &pending_dirs[1], context)?;
            return Ok(());
        } else {
            // 如果待比对区不足2个，提示错误
            println!(
                "{}",
                "Error: No directory specified and waiting area does not have 2 directories.".red()
            );
            println!(
                "Current waiting area has {} directory(s).",
                pending_dirs.len()
            );
            println!("Usage: bsr compare <dir> or add 2 directories to waiting area.");
            return Ok(());
        }
    }

    let target_path = target_dir.unwrap(); // 安全，因为上面已经处理了 None 的情况
    let abs_target_dir = fs::canonicalize(&target_path)
        .with_context(|| format!("Failed to canonicalize path: {:?}", target_path))?;

    // 处理待比对区逻辑
    if pending_dirs.is_empty() {
        // 0 -> 1
        pending_dirs.push(abs_target_dir.clone());
        storage::write_pending_compare(&pending_dirs)?;
        println!(
            "Added {} to the waiting area (1/2).",
            abs_target_dir.display().to_string().green()
        );
    } else if pending_dirs.len() == 1 {
        // 1 -> 2
        let first_dir = &pending_dirs[0];
        if first_dir == &abs_target_dir {
            println!(
                "{}",
                "The directory is already in the waiting area.".yellow()
            );
            return Ok(());
        }

        pending_dirs.push(abs_target_dir.clone());
        storage::write_pending_compare(&pending_dirs)?;
        println!(
            "Added {} to the waiting area (2/2).",
            abs_target_dir.display().to_string().green()
        );

        // 自动提示开始对比
        let ans = Select::new(
            "Two directories are ready. What do you want to do?",
            vec!["Start comparison", "Cancel"],
        )
        .prompt()?;

        if ans == "Start comparison" {
            println!("{}", "\nStarting comparison...\n".green().bold());
            perform_main_comparison(&pending_dirs[0], &pending_dirs[1], context)?;
        }
    } else {
        // 2 -> Check if target is one of them or Replace
        if pending_dirs.contains(&abs_target_dir) {
            println!(
                "{}",
                "The directory is already in the waiting area.".yellow()
            );
            // 即使已经在待比对区，如果总数是2，也提示是否开始对比
            let ans = Select::new(
                "Two directories are ready. What do you want to do?",
                vec!["Start comparison", "Cancel"],
            )
            .prompt()?;

            if ans == "Start comparison" {
                println!("{}", "\nStarting comparison...\n".green().bold());
                perform_main_comparison(&pending_dirs[0], &pending_dirs[1], context)?;
            }
            return Ok(());
        } else {
            println!("Waiting area is full:");
            println!("1: {}", pending_dirs[0].display().to_string().cyan());
            println!("2: {}", pending_dirs[1].display().to_string().cyan());
            println!("Current: {}", abs_target_dir.display().to_string().green());

            let options = vec![
                "Replace 1st directory",
                "Replace 2nd directory",
                "Clear and add as 1st",
                "Start comparison (ignore current)",
                "Cancel",
            ];

            let ans = Select::new("What do you want to do?", options).prompt()?;

            match ans {
                "Replace 1st directory" => {
                    pending_dirs[0] = abs_target_dir;
                    storage::write_pending_compare(&pending_dirs)?;
                    println!("{}", "Updated waiting area.".green());
                }
                "Replace 2nd directory" => {
                    pending_dirs[1] = abs_target_dir;
                    storage::write_pending_compare(&pending_dirs)?;
                    println!("{}", "Updated waiting area.".green());
                }
                "Clear and add as 1st" => {
                    pending_dirs.clear();
                    pending_dirs.push(abs_target_dir);
                    storage::write_pending_compare(&pending_dirs)?;
                    println!("{}", "Cleared and added to waiting area.".green());
                }
                "Start comparison (ignore current)" => {
                    println!("{}", "\nStarting comparison...\n".green().bold());
                    perform_main_comparison(&pending_dirs[0], &pending_dirs[1], context)?;
                }
                _ => println!("Cancelled."),
            }
        }
    }

    Ok(())
}

/// 执行主工程对比（包含 originSource 摘要）
fn perform_main_comparison(dir_a: &Path, dir_b: &Path, context: usize) -> Result<()> {
    // 1. 获取文件列表
    let files_a = list_files(dir_a, false)?;
    let files_b = list_files(dir_b, false)?;

    // 2. 结构对比
    if !check_structure_similarity(&files_a, &files_b)? {
        println!(
            "{}",
            "The directory structures are quite different."
                .yellow()
                .bold()
        );
        let ans = Select::new("Do you want to continue comparison?", vec!["Yes", "No"]).prompt()?;
        if ans == "No" {
            println!("Comparison aborted.");
            return Ok(());
        }
    }

    // 3. 详细对比
    let diff_result = compare_directories(dir_a, dir_b, files_a, files_b, context, false)?;

    // 4. originSource 版本对比摘要
    let origin_diff = compare_origin_source_summary(dir_a, dir_b)?;

    // 5. 保存对比结果并提示打开
    save_and_open_diff(dir_a, dir_b, &diff_result, origin_diff.as_deref())?;

    Ok(())
}

/// 执行通用对比逻辑
/// include_origin_source: 是否包含 originSource 目录（递归）
fn perform_comparison(
    dir_a: &Path,
    dir_b: &Path,
    context: usize,
    include_origin_source: bool,
) -> Result<()> {
    // 1. 获取文件列表
    let files_a = list_files(dir_a, include_origin_source)?;
    let files_b = list_files(dir_b, include_origin_source)?;

    // 2. 结构对比
    if !check_structure_similarity(&files_a, &files_b)? {
        println!(
            "{}",
            "The directory structures are quite different."
                .yellow()
                .bold()
        );
        let ans = Select::new("Do you want to continue comparison?", vec!["Yes", "No"]).prompt()?;
        if ans == "No" {
            println!("Comparison aborted.");
            return Ok(());
        }
    }

    // 3. 详细对比
    let diff_result = compare_directories(
        dir_a,
        dir_b,
        files_a,
        files_b,
        context,
        include_origin_source,
    )?;

    // 4. 保存对比结果并提示打开
    save_and_open_diff(dir_a, dir_b, &diff_result, None)?;

    Ok(())
}

/// 对比 originSource 下的所有仓库版本
fn compare_origin_source_summary(dir_a: &Path, dir_b: &Path) -> Result<Option<String>> {
    let os_a = dir_a.join("originSource");
    let os_b = dir_b.join("originSource");

    if !os_a.exists() && !os_b.exists() {
        return Ok(None);
    }

    let mut output = String::new();
    let title = "\n--- originSource Version Comparison ---";
    println!("{}", title.blue().bold());
    output.push_str(title);
    output.push('\n');

    let repos_a = scan_git_repos(&os_a)?;
    let repos_b = scan_git_repos(&os_b)?;

    let mut all_repos: HashSet<PathBuf> = HashSet::new();
    for r in repos_a.keys() {
        all_repos.insert(r.clone());
    }
    for r in repos_b.keys() {
        all_repos.insert(r.clone());
    }

    let mut sorted_repos: Vec<_> = all_repos.into_iter().collect();
    sorted_repos.sort();

    let mut has_diff = false;

    for rel_path in sorted_repos {
        let ver_a = repos_a.get(&rel_path);
        let ver_b = repos_b.get(&rel_path);

        match (ver_a, ver_b) {
            (Some(va), Some(vb)) => {
                let s_a = format_git_version(va);
                let s_b = format_git_version(vb);
                if s_a != s_b {
                    has_diff = true;
                    println!("{}: {} vs {}", rel_path.display(), s_a.red(), s_b.green());
                    output.push_str(&format!("{}: {} vs {}\n", rel_path.display(), s_a, s_b));
                }
            }
            (Some(_), None) => {
                has_diff = true;
                println!("{}: {} (only in A)", rel_path.display(), "Missing".red());
                output.push_str(&format!("{}: Missing (only in A)\n", rel_path.display()));
            }
            (None, Some(_)) => {
                has_diff = true;
                println!("{}: {} (only in B)", rel_path.display(), "Missing".green());
                output.push_str(&format!("{}: Missing (only in B)\n", rel_path.display()));
            }
            _ => {}
        }
    }

    if !has_diff {
        let msg = "All originSource repositories match.";
        println!("{}", msg.green());
        output.push_str(msg);
        output.push('\n');
    }

    Ok(Some(output))
}

fn scan_git_repos(root: &Path) -> Result<HashMap<PathBuf, GitVersion>> {
    let mut repos = HashMap::new();
    if !root.exists() {
        return Ok(repos);
    }

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(false) // 我们需要找到 .git，所以不能忽略 gitignore? 不，我们需要遍历所有目录找 .git
        .build();

    for result in walker.flatten() {
        if !result.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }
        let path = result.path();
        if !path.join(".git").exists() {
            continue;
        }
        if let (Ok(ver), Ok(rel)) = (get_git_version(path), path.strip_prefix(root)) {
            repos.insert(rel.to_path_buf(), ver);
        }
    }
    Ok(repos)
}

fn format_git_version(v: &GitVersion) -> String {
    match v {
        GitVersion::Tag(t, clean) => format!("Tag: {}{}", t, if *clean { "" } else { "*" }),
        GitVersion::Branch(b, clean) => format!("Branch: {}{}", b, if *clean { "" } else { "*" }),
        GitVersion::Commit(c, clean) => {
            format!("Commit: {:.7}{}", c, if *clean { "" } else { "*" })
        }
    }
}

fn save_and_open_diff(
    dir_a: &Path,
    dir_b: &Path,
    diff_content: &str,
    origin_diff: Option<&str>,
) -> Result<()> {
    let now = Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("diff_{}.md", timestamp);
    let reports_dir = storage::get_reports_dir();
    let file_path = reports_dir.join(&filename);

    let mut full_content = String::new();

    // 1. Header
    full_content.push_str("# Comparison Report\n\n");
    full_content.push_str(&format!(
        "- **Generated at**: {}\n",
        now.format("%Y-%m-%d %H:%M:%S")
    ));
    full_content.push_str(&format!("- **Directory A**: `{}`\n", dir_a.display()));
    full_content.push_str(&format!("- **Directory B**: `{}`\n", dir_b.display()));
    full_content.push_str("\n---\n\n");

    // 2. Main Diff
    full_content.push_str("## File Differences\n\n");
    if diff_content.trim().is_empty() {
        full_content.push_str("*No file differences found.*\n\n");
    } else {
        full_content.push_str("```diff\n");
        full_content.push_str(&strip_ansi_codes(diff_content));
        full_content.push_str("\n```\n\n");
    }

    // 3. OriginSource Diff (if any)
    if let Some(origin) = origin_diff {
        full_content.push_str("## originSource Version Comparison\n\n");
        full_content.push_str("```diff\n");
        full_content.push_str(&strip_ansi_codes(origin));
        full_content.push_str("\n```\n\n");
    }

    let mut file = fs::File::create(&file_path)?;
    file.write_all(full_content.as_bytes())?;

    println!(
        "\nDiff result saved to: {}",
        file_path.display().to_string().green()
    );

    let open_option = Select::new(
        "Do you want to open the diff file in VS Code?",
        vec!["Open", "Cancel"],
    )
    .with_starting_cursor(1) // 默认选中 Cancel
    .prompt()?;

    if open_option == "Open" {
        Command::new("code").arg(&file_path).spawn()?;
    }

    Ok(())
}

fn strip_ansi_codes(input: &str) -> String {
    let regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    regex.replace_all(input, "").to_string()
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

/// 列出目录下所有文件（相对路径），遵循 gitignore，可选择是否强制包含 originSource
fn list_files(root: &Path, include_origin_source: bool) -> Result<Vec<PathBuf>> {
    let mut files = HashSet::new();

    // 1. 主遍历：遵循所有 gitignore 规则
    let mut builder = WalkBuilder::new(root);
    builder.hidden(true);
    builder.git_ignore(true);

    let walker = builder
        .filter_entry(|e| {
            if e.file_name() == ".git" {
                return false;
            }
            true
        })
        .build();

    for result in walker.flatten() {
        if !result.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        if let Ok(rel_path) = result.path().strip_prefix(root) {
            files.insert(rel_path.to_path_buf());
        }
    }

    // 2. originSource 专项遍历：忽略父目录的 gitignore，但遵循内部的 gitignore
    // 只有在 include_origin_source 为 true 时才执行
    if include_origin_source {
        let origin_source_path = root.join("originSource");
        if origin_source_path.exists() {
            let mut os_builder = WalkBuilder::new(&origin_source_path);
            os_builder.hidden(true);
            os_builder.git_ignore(true);
            os_builder.parents(false); // 关键：不读取上层目录的 gitignore

            let os_walker = os_builder
                .filter_entry(|e| {
                    if e.file_name() == ".git" {
                        return false;
                    }
                    true
                })
                .build();

            for result in os_walker.flatten() {
                if !result.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    continue;
                }
                // 注意：这里的 path 需要转换为相对于 root 的路径
                if let Ok(rel_path) = result.path().strip_prefix(root) {
                    files.insert(rel_path.to_path_buf());
                }
            }
        }
    }

    let mut result: Vec<_> = files.into_iter().collect();
    result.sort();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_list_files_includes_ignored_origin_source() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // 1. Create .gitignore ignoring originSource
        let gitignore_path = root.join(".gitignore");
        let mut gitignore = File::create(gitignore_path)?;
        writeln!(gitignore, "originSource")?;

        // 2. Create originSource directory and a file inside
        let origin_source = root.join("originSource");
        fs::create_dir(&origin_source)?;
        let file_path = origin_source.join("test.txt");
        let mut file = File::create(&file_path)?;
        writeln!(file, "content")?;

        // 3. Create a normal file
        let normal_file = root.join("normal.txt");
        File::create(&normal_file)?;

        // 4. Run list_files
        let files = list_files(root, true)?;

        // 5. Assertions
        let files_set: HashSet<_> = files.iter().collect();
        assert!(files_set.contains(&PathBuf::from("normal.txt")));
        assert!(files_set.contains(&PathBuf::from("originSource").join("test.txt")));

        Ok(())
    }

    #[test]
    fn test_list_files_normal_behavior() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Initialize git repo (required for .gitignore to be respected by ignore crate usually)
        Command::new("git").arg("init").current_dir(root).output()?;

        // 1. Create .gitignore
        let gitignore_path = root.join(".gitignore");
        let mut gitignore = File::create(gitignore_path)?;
        writeln!(gitignore, "ignored.txt")?;

        // 2. Create ignored file
        File::create(root.join("ignored.txt"))?;

        // 3. Create normal file
        File::create(root.join("normal.txt"))?;

        // 4. Create sub directory and file
        let subdir = root.join("subdir");
        fs::create_dir(&subdir)?;
        File::create(subdir.join("sub.txt"))?;

        // 5. Run list_files (without originSource force inclusion)
        let files = list_files(root, false)?;
        let files_set: HashSet<_> = files.iter().collect();

        assert!(files_set.contains(&PathBuf::from("normal.txt")));
        assert!(files_set.contains(&PathBuf::from("subdir").join("sub.txt")));
        assert!(!files_set.contains(&PathBuf::from("ignored.txt")));

        Ok(())
    }
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
    include_origin_source: bool,
) -> Result<String> {
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
            if !include_origin_source && is_origin_source(&rel_path) {
                // 如果不包含 originSource，则跳过（理论上 list_files 已经过滤了，但双重保险）
                continue;
            }

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

    let mut full_output = String::new();

    // 输出结果
    if results.is_empty() {
        let msg = "No differences found.";
        println!("{}", msg.green());
        full_output.push_str(msg);
    } else {
        let mut sorted_dirs: Vec<_> = results.keys().collect();
        sorted_dirs.sort();

        for dir in sorted_dirs {
            let dir_header = format!("Directory: {}", dir.display());
            println!("{}", dir_header.blue().bold());
            full_output.push_str(&dir_header);
            full_output.push('\n');

            for line in &results[dir] {
                println!("{}", line);
                full_output.push_str(line);
                full_output.push('\n');
            }
            println!();
            full_output.push('\n');
        }
    }

    Ok(full_output)
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
                let prefix = match change.tag() {
                    ChangeTag::Delete => "-".red(),
                    ChangeTag::Insert => "+".green(),
                    ChangeTag::Equal => " ".dimmed(),
                };
                output.push_str(&format!("{:>4} | {} ", line_number.dimmed(), prefix));

                for (emphasized, value) in change.values() {
                    if *emphasized {
                        match change.tag() {
                            ChangeTag::Delete => {
                                output.push_str(&format!("{}", value.red().bold()))
                            }
                            ChangeTag::Insert => {
                                output.push_str(&format!("{}", value.green().bold()))
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
