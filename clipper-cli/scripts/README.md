# Clipper CLI Scripts

This folder contains useful shell integration scripts for clipper-cli.

## Scripts

### clipper-zsh-autosuggestions.zsh

A custom strategy for [zsh-autosuggestions](https://github.com/zsh-users/zsh-autosuggestions) that suggests completions from your Clipper clipboard history.

**Installation:**

```zsh
# In your .zshrc (after zsh-autosuggestions is loaded):
source /path/to/clipper-zsh-autosuggestions.zsh

# Add 'clipper' to your strategy list:
ZSH_AUTOSUGGEST_STRATEGY=(clipper history completion)
```

**Configuration:**

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_CLI_PATH` | `clipper-cli` | Path to clipper-cli binary |
| `CLIPPER_SUGGEST_LIMIT` | `10` | Max suggestions to fetch |
| `CLIPPER_SUGGEST_TAGS` | (none) | Filter suggestions by tags |
| `CLIPPER_SUGGEST_LOCAL` | `1` | Only show clips from this computer |

**Testing:**

```zsh
# Test the integration
clipper-suggest-test "your query"
```

### clipper-fish-autosuggestions.fish

Autosuggestions from Clipper clipboard history for [fish shell](https://fishshell.com/).

**Installation:**

```fish
# Copy to fish conf.d (auto-loaded):
cp clipper-fish-autosuggestions.fish ~/.config/fish/conf.d/

# Or source in config.fish:
source /path/to/clipper-fish-autosuggestions.fish
```

**Configuration:**

```fish
# In config.fish (before sourcing the plugin):
set -g CLIPPER_CLI_PATH /path/to/clipper-cli
set -g CLIPPER_SUGGEST_LIMIT 10
set -g CLIPPER_SUGGEST_TAGS "tag1,tag2"
set -g CLIPPER_SUGGEST_LOCAL 1  # Only show clips from this computer
```

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_CLI_PATH` | `clipper-cli` | Path to clipper-cli binary |
| `CLIPPER_SUGGEST_LIMIT` | `10` | Max suggestions to fetch |
| `CLIPPER_SUGGEST_TAGS` | (none) | Filter suggestions by tags |
| `CLIPPER_SUGGEST_LOCAL` | `1` | Only show clips from this computer |

**Key bindings:**
- `Ctrl-Alt-C` - Accept clipper suggestion
- `Ctrl-Alt-S` - Interactive fzf search (requires fzf)

**Abbreviations provided:**
- `clip` → `clipper-cli create`
- `clips` → `clipper-cli search`
- `clipl` → `clipper-cli list`

**Testing:**

```fish
clipper-suggest-test "your query"
```

### clipper-fzf.sh

Interactive clip selection using [fzf](https://github.com/junegunn/fzf) with preview support.

**Requirements:**
- fzf
- jq
- clipper-cli

**Installation:**

```bash
# Make executable
chmod +x clipper-fzf.sh

# Option 1: Source for shell functions
source /path/to/clipper-fzf.sh

# Option 2: Create alias
alias clipper-fzf='/path/to/clipper-fzf.sh'
```

**Usage:**

```bash
# Browse all clips interactively
clipper-fzf select

# Browse with initial search query
clipper-fzf select "search term"

# Interactive search with live preview
clipper-fzf search

# Select and copy to system clipboard
clipper-fzf copy
```

**Key bindings in fzf:**
- `ENTER` - Select clip
- `TAB` - Toggle preview
- `Ctrl-Y` - Copy clip ID to clipboard
- `Ctrl-D` - Delete selected clip
- `ESC` - Cancel

**Configuration:**

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIPPER_CLI_PATH` | `clipper-cli` | Path to clipper-cli binary |
| `CLIPPER_FZF_PAGE_SIZE` | `100` | Number of clips to load |
| `CLIPPER_FZF_PREVIEW` | `1` | Enable preview (0 or 1) |

### generate-completions.sh

Generates shell completion files for bash, zsh, fish, and PowerShell.

**Usage:**

```bash
# Generate completions in current directory
./generate-completions.sh

# Generate completions in specific directory
./generate-completions.sh ~/.local/share/completions
```

**Output files:**
- `clipper-cli.bash` - Bash completions
- `clipper-cli.zsh` - Zsh completions (rename to `_clipper-cli`)
- `clipper-cli.fish` - Fish completions
- `clipper-cli.ps1` - PowerShell completions

**Installation:**

```bash
# Bash
source clipper-cli.bash
# Or copy to /etc/bash_completion.d/

# Zsh
mkdir -p ~/.zsh/completions
cp clipper-cli.zsh ~/.zsh/completions/_clipper-cli
# Add to .zshrc: fpath=(~/.zsh/completions $fpath)

# Fish
cp clipper-cli.fish ~/.config/fish/completions/

# PowerShell
# Add to $PROFILE: . /path/to/clipper-cli.ps1
```

## Example .zshrc Configuration

```zsh
# Load zsh-autosuggestions first
source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh

# Load clipper scripts
source /path/to/clipper-cli/scripts/clipper-zsh-autosuggestions.zsh
source /path/to/clipper-cli/scripts/clipper-fzf.sh

# Configure autosuggestions to use clipper
ZSH_AUTOSUGGEST_STRATEGY=(clipper history completion)

# Optional: Add keyboard shortcuts
bindkey '^[c' clipper-select  # Alt-C to browse clips
```

## Example .bashrc Configuration

```bash
# Load fzf integration
source /path/to/clipper-cli/scripts/clipper-fzf.sh

# Load completions
source /path/to/clipper-cli/scripts/clipper-cli.bash

# Optional: Add alias
alias cs='clipper-fzf search'
alias cc='clipper-fzf copy'
```
