# bsr 项目管理工具

`bsr 是一个用于高效管理 Bisheng 项目及其子仓库状态的 Rust 命令行工具。

## 安装

### macOS (一键安装)

你可以使用安装脚本轻松安装 `bsr`：

```bash
curl -fsSL https://raw.githubusercontent.com/cyhndaugust/bsr/main/install.sh | bash
```

该脚本会：
1. 自动检测您的系统架构（Intel 或 Apple Silicon）。
2. 下载最新的二进制发行版。
3. 将其安装到 `~/.local/bin`。
4. 如果 `~/.local/bin` 不在您的 `PATH` 中，脚本会自动将其添加到您的 Shell 配置文件中（支持 Zsh 和 Bash）。

### 升级

安装完成后，你可以随时使用以下命令将 `bsr` 升级到最新版本：

```bash
bsr upgrade
```

## 已实现命令

### 1. 添加检索项目 (`add`)
将指定目录下的 Bisheng 项目添加到管理列表中。

```bash
bsr add <directory>
```

- **参数**: `<directory>` - 需要扫描的目录路径。
- **功能**: 递归检索该目录下所有符合条件的 Bisheng 项目，并将其记录在本地配置中，方便后续快速访问。

### 2. 列出项目 (`list` / `ls`)
查看并选择已添加的 Bisheng 项目。

```bash
bsr list
# 或者使用别名
bsr ls
```

- **功能**: 
  - 展示所有已记录的项目列表。
  - 提供交互式选择界面（使用 `inquire`）。
  - 选中后会显示项目的完整路径。

### 3. 查看子仓库状态 (`status`)
检查项目中 `originSource` 目录下各组件的 Git 状态。

```bash
bsr status [directory] [options]
```

- **参数**: `[directory]` - 目标项目目录，默认为当前目录 `.`。
- **选项**:
  - `-a, --all`: 显示所有仓库的状态。默认情况下，仅显示有未提交修改的仓库。
- **功能**: 
  - 递归扫描 `originSource` 目录。
  - 显示每个子仓库的当前分支。
  - 标记 `clean`（无修改）或 `modified`（有修改）。
  - 列出具体的修改文件列表。
  - 显示 Stash 的数量（如果有）。

### 4. 目录对比 (`compare`)
对比两个目录之间的差异，专为 Bisheng 项目结构优化。

```bash
bsr compare [directory] [options]
```

- **参数**: `[directory]` - 要添加或对比的目录路径，默认为当前目录 `.`。
- **选项**:
  - `-C, --context <n>`: 设置 Diff 输出显示的上下文行数（默认为 3）。
- **功能**:
  - **待比对区管理**:
    - 首次运行会将指定目录添加到“待比对区”。
    - 再次运行（指定另一个目录）时，会提示用户：
      - 开始对比（Start comparison）
      - 替换待比对区目录（Replace waiting area）
      - 取消
  - **智能对比**:
    - **结构检查**: 如果两个目录结构差异过大，会自动停止对比。
    - **originSource 优化**: 针对 `originSource` 下的组件仓库，会优先检查 Git Tag 和状态。如果版本一致且无未提交修改，则视为相同，跳过耗时的文件内容对比。
    - **可视化输出**:
      - 使用颜色区分新增（绿色）、删除（红色）和修改。
      - 支持行内差异高亮显示。
      - 自动忽略 `.git`、隐藏文件和 `.gitignore` 中列出的文件。
  - **报告生成**:
    - 对比结果会自动保存为文本文件，路径通常为 `~/.config/bsr/reports/diff_YYYYMMDD_HHMMSS.txt`。
    - 报告中去除了 ANSI 颜色代码，方便查看。
    - 任务完成后，提供选项（Open / Cancel）直接在 VS Code 中打开生成的报告文件。

## 规划中的命令

以下命令目前尚未实现，将在后续版本中推出：

- `bsr start`: 启动项目。