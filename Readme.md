# bsr Project Management Tool

`bsr is a Rust command-line tool designed for efficiently managing Bisheng projects and their sub-repository statuses.

## Installation

### macOS (One-click Install)

You can install `bsr` easily using the installation script:

```bash
curl -fsSL https://raw.githubusercontent.com/cyhndaugust/bsr/main/install.sh | bash
```

The script will:
1. Automatically detect your architecture (Intel or Apple Silicon).
2. Download the latest binary release.
3. Install it to `~/.local/bin`.
4. Automatically add `~/.local/bin` to your `PATH` if it's not already there (supports Zsh and Bash).

### Upgrade

Once installed, you can upgrade `bsr` to the latest version at any time using:

```bash
bsr upgrade
```

## Implemented Commands

### 1. Add Project (`add`)
Add a Bisheng project from a specified directory to the management list.

```bash
bsr add <directory>
```

- **Arguments**: `<directory>` - The directory path to scan.
- **Function**: Recursively searches for all eligible Bisheng projects under that directory and records them in the local configuration for quick access later.

### 2. List Projects (`list` / `ls`)
View and select added Bisheng projects.

```bash
bsr list
# Or use the alias
bsr ls
```

- **Function**:
  - Displays a list of all recorded projects.
  - Provides an interactive selection interface (using `inquire`).
  - Shows the full path of the project upon selection.

### 3. View Sub-repository Status (`status`)
Check the Git status of components under the `originSource` directory in a project.

```bash
bsr status [directory] [options]
```

- **Arguments**: `[directory]` - Target project directory, defaults to the current directory `.`.
- **Options**:
  - `-a, --all`: Show status for all repositories. By default, only repositories with uncommitted changes are shown.
- **Function**:
  - Recursively scans the `originSource` directory.
  - Displays the current branch of each sub-repository.
  - Marks as `clean` (no changes) or `modified` (has changes).
  - Lists specific modified files.
  - Shows the number of Stashes (if any).

### 4. Directory Comparison (`compare`)
Compare differences between two directories, optimized for Bisheng project structures.

```bash
bsr compare [directory] [options]
```

- **Arguments**: `[directory]` - Directory path to add or compare, defaults to the current directory `.`.
- **Options**:
  - `-C, --context <n>`: Set the number of context lines shown in Diff output (default is 3).
- **Function**:
  - **Waiting Area Management**:
    - The first run adds the specified directory to the "Waiting Area".
    - Running it again (specifying another directory) prompts the user to:
      - Start comparison
      - Replace waiting area directory
      - Cancel
  - **Smart Comparison**:
    - **Structure Check**: Automatically stops comparison if the directory structures are too different.
    - **originSource Optimization**: For component repositories under `originSource`, prioritizes checking Git Tags and status. If versions match and there are no uncommitted changes, it's considered identical, skipping time-consuming file content comparison.
    - **Visual Output**:
      - Uses colors to distinguish additions (Green), deletions (Red), and modifications.
      - Supports inline difference highlighting.
      - Automatically ignores `.git`, hidden files, and files listed in `.gitignore`.
  - **Report Generation**:
    - Comparison results are automatically saved as text files, typically at `~/.config/bsr/reports/diff_YYYYMMDD_HHMMSS.txt`.
    - ANSI color codes are stripped from the report for easy viewing.
    - Upon completion, offers an option (Open / Cancel) to open the generated report file directly in VS Code.

## Planned Commands

The following commands are not yet implemented and will be released in future versions:

- `bsr start`: Start the project.
