#!/usr/bin/env bash
# Generate shell completions for clipper-cli
#
# This script generates shell completion files for bash, zsh, fish, and PowerShell.
#
# Usage:
#   ./generate-completions.sh [output-dir]
#
# Requirements:
#   - clipper-cli built and available in PATH or target/release
#
# Output files:
#   - clipper-cli.bash     (Bash completions)
#   - clipper-cli.zsh      (Zsh completions, rename to _clipper-cli)
#   - clipper-cli.fish     (Fish completions)
#   - clipper-cli.ps1      (PowerShell completions)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Determine script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Output directory (default: current directory)
OUTPUT_DIR="${1:-.}"

# Find clipper-cli binary
find_clipper_cli() {
    # Check if in PATH
    if command -v clipper-cli >/dev/null 2>&1; then
        echo "clipper-cli"
        return
    fi

    # Check release build
    local release="$PROJECT_ROOT/target/release/clipper-cli"
    if [[ -x "$release" ]]; then
        echo "$release"
        return
    fi

    # Check debug build
    local debug="$PROJECT_ROOT/target/debug/clipper-cli"
    if [[ -x "$debug" ]]; then
        echo "$debug"
        return
    fi

    return 1
}

# Main
main() {
    echo "Generating shell completions for clipper-cli..."

    # Find binary
    local cli_path
    if ! cli_path=$(find_clipper_cli); then
        echo -e "${RED}Error: clipper-cli not found${NC}" >&2
        echo "Please build clipper-cli first:" >&2
        echo "  cargo build -p clipper-cli --release" >&2
        exit 1
    fi

    echo "Using: $cli_path"
    echo "Output directory: $OUTPUT_DIR"

    # Create output directory if needed
    mkdir -p "$OUTPUT_DIR"

    # Generate completions using clap's built-in mechanism
    # Note: This requires clipper-cli to have completion generation support
    # For now, we'll create static completion files

    # Generate Bash completions
    cat > "$OUTPUT_DIR/clipper-cli.bash" << 'BASH_COMPLETIONS'
# Bash completion for clipper-cli
# Source this file in your .bashrc or place in /etc/bash_completion.d/

_clipper_cli() {
    local cur prev words cword
    _init_completion || return

    local commands="create get update search delete watch list upload share export import search-tag"
    local aliases="c g u s d w l e i st"

    case "${prev}" in
        clipper-cli)
            COMPREPLY=($(compgen -W "${commands} ${aliases} --help --version -c -u -t" -- "${cur}"))
            return
            ;;
        create|c)
            COMPREPLY=($(compgen -W "--tags --notes -t -n" -- "${cur}"))
            return
            ;;
        get|g)
            COMPREPLY=($(compgen -W "--format -f" -- "${cur}"))
            return
            ;;
        update|u)
            COMPREPLY=($(compgen -W "--tags --notes -t -n" -- "${cur}"))
            return
            ;;
        search|s)
            COMPREPLY=($(compgen -W "--tags --start-date --end-date --page --page-size --format -t -p -f" -- "${cur}"))
            return
            ;;
        delete|d)
            return
            ;;
        watch|w)
            return
            ;;
        list|l)
            COMPREPLY=($(compgen -W "--tags --start-date --end-date --page --page-size --format -t -p -f" -- "${cur}"))
            return
            ;;
        upload)
            COMPREPLY=($(compgen -f -- "${cur}"))
            return
            ;;
        share)
            COMPREPLY=($(compgen -W "--expires --format -e -f" -- "${cur}"))
            return
            ;;
        export|e)
            COMPREPLY=($(compgen -W "--output -o" -- "${cur}"))
            return
            ;;
        import|i)
            COMPREPLY=($(compgen -f -- "${cur}"))
            return
            ;;
        search-tag|st)
            COMPREPLY=($(compgen -W "--page --page-size --format -p -f" -- "${cur}"))
            return
            ;;
        --format|-f)
            COMPREPLY=($(compgen -W "json text" -- "${cur}"))
            return
            ;;
        --config|-c)
            COMPREPLY=($(compgen -f -- "${cur}"))
            return
            ;;
        --output|-o)
            COMPREPLY=($(compgen -f -- "${cur}"))
            return
            ;;
    esac

    COMPREPLY=($(compgen -W "${commands}" -- "${cur}"))
}

complete -F _clipper_cli clipper-cli
BASH_COMPLETIONS

    echo -e "${GREEN}Created: $OUTPUT_DIR/clipper-cli.bash${NC}"

    # Generate Zsh completions
    cat > "$OUTPUT_DIR/clipper-cli.zsh" << 'ZSH_COMPLETIONS'
#compdef clipper-cli

# Zsh completion for clipper-cli
# Place this file as _clipper-cli in your $fpath (e.g., ~/.zsh/completions/_clipper-cli)

_clipper_cli() {
    local -a commands
    commands=(
        'create:Create a new clip'
        'c:Create a new clip (alias)'
        'get:Get a clip by ID'
        'g:Get a clip by ID (alias)'
        'update:Update a clip'\''s tags and/or notes'
        'u:Update a clip (alias)'
        'search:Search clips'
        's:Search clips (alias)'
        'delete:Delete a clip by ID'
        'd:Delete a clip (alias)'
        'watch:Watch for real-time notifications'
        'w:Watch (alias)'
        'list:List clips'
        'l:List clips (alias)'
        'upload:Upload a file to create a clip'
        'share:Create a short URL for a clip'
        'export:Export all clips to a tar.gz archive'
        'e:Export (alias)'
        'import:Import clips from a tar.gz archive'
        'i:Import (alias)'
        'search-tag:Search tags'
        'st:Search tags (alias)'
    )

    _arguments -C \
        '-c[Path to config file]:config file:_files' \
        '--config[Path to config file]:config file:_files' \
        '-u[Server URL]:url:' \
        '--url[Server URL]:url:' \
        '-t[Bearer token for authentication]:token:' \
        '--token[Bearer token for authentication]:token:' \
        '-h[Show help]' \
        '--help[Show help]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'clipper-cli commands' commands
            ;;
        args)
            case $words[1] in
                create|c)
                    _arguments \
                        '-t[Tags (comma-separated)]:tags:' \
                        '--tags[Tags (comma-separated)]:tags:' \
                        '-n[Additional notes]:notes:' \
                        '--notes[Additional notes]:notes:' \
                        '1:content:'
                    ;;
                get|g)
                    _arguments \
                        '-f[Output format]:format:(json text)' \
                        '--format[Output format]:format:(json text)' \
                        '1:clip ID:'
                    ;;
                update|u)
                    _arguments \
                        '-t[New tags (comma-separated)]:tags:' \
                        '--tags[New tags (comma-separated)]:tags:' \
                        '-n[New additional notes]:notes:' \
                        '--notes[New additional notes]:notes:' \
                        '1:clip ID:'
                    ;;
                search|s)
                    _arguments \
                        '-t[Filter by tags]:tags:' \
                        '--tags[Filter by tags]:tags:' \
                        '--start-date[Filter by start date (ISO 8601)]:date:' \
                        '--end-date[Filter by end date (ISO 8601)]:date:' \
                        '-p[Page number]:page:' \
                        '--page[Page number]:page:' \
                        '--page-size[Items per page]:size:' \
                        '-f[Output format]:format:(json text)' \
                        '--format[Output format]:format:(json text)' \
                        '1:search query:'
                    ;;
                delete|d)
                    _arguments '1:clip ID:'
                    ;;
                list|l)
                    _arguments \
                        '-t[Filter by tags]:tags:' \
                        '--tags[Filter by tags]:tags:' \
                        '--start-date[Filter by start date (ISO 8601)]:date:' \
                        '--end-date[Filter by end date (ISO 8601)]:date:' \
                        '-p[Page number]:page:' \
                        '--page[Page number]:page:' \
                        '--page-size[Items per page]:size:' \
                        '-f[Output format]:format:(json text)' \
                        '--format[Output format]:format:(json text)'
                    ;;
                upload)
                    _arguments \
                        '-t[Tags (comma-separated)]:tags:' \
                        '--tags[Tags (comma-separated)]:tags:' \
                        '-n[Additional notes]:notes:' \
                        '--notes[Additional notes]:notes:' \
                        '-c[Content override]:content:' \
                        '--content[Content override]:content:' \
                        '1:file:_files'
                    ;;
                share)
                    _arguments \
                        '-e[Expiration in hours]:hours:' \
                        '--expires[Expiration in hours]:hours:' \
                        '-f[Output format]:format:(url json)' \
                        '--format[Output format]:format:(url json)' \
                        '1:clip ID:'
                    ;;
                export|e)
                    _arguments \
                        '-o[Output file path]:file:_files' \
                        '--output[Output file path]:file:_files'
                    ;;
                import|i)
                    _arguments \
                        '-f[Output format]:format:(text json)' \
                        '--format[Output format]:format:(text json)' \
                        '1:archive file:_files -g "*.tar.gz"'
                    ;;
                search-tag|st)
                    _arguments \
                        '-p[Page number]:page:' \
                        '--page[Page number]:page:' \
                        '--page-size[Items per page]:size:' \
                        '-f[Output format]:format:(text json)' \
                        '--format[Output format]:format:(text json)' \
                        '1:search query:'
                    ;;
            esac
            ;;
    esac
}

_clipper_cli "$@"
ZSH_COMPLETIONS

    echo -e "${GREEN}Created: $OUTPUT_DIR/clipper-cli.zsh${NC}"

    # Generate Fish completions
    cat > "$OUTPUT_DIR/clipper-cli.fish" << 'FISH_COMPLETIONS'
# Fish completion for clipper-cli
# Place this file in ~/.config/fish/completions/clipper-cli.fish

# Disable file completions for clipper-cli by default
complete -c clipper-cli -f

# Global options
complete -c clipper-cli -s c -l config -d "Path to config file" -r
complete -c clipper-cli -s u -l url -d "Server URL" -r
complete -c clipper-cli -s t -l token -d "Bearer token for authentication" -r
complete -c clipper-cli -s h -l help -d "Show help"

# Commands
complete -c clipper-cli -n "__fish_use_subcommand" -a create -d "Create a new clip"
complete -c clipper-cli -n "__fish_use_subcommand" -a c -d "Create a new clip (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a get -d "Get a clip by ID"
complete -c clipper-cli -n "__fish_use_subcommand" -a g -d "Get a clip by ID (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a update -d "Update a clip's tags and/or notes"
complete -c clipper-cli -n "__fish_use_subcommand" -a u -d "Update a clip (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a search -d "Search clips"
complete -c clipper-cli -n "__fish_use_subcommand" -a s -d "Search clips (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a delete -d "Delete a clip by ID"
complete -c clipper-cli -n "__fish_use_subcommand" -a d -d "Delete a clip (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a watch -d "Watch for real-time notifications"
complete -c clipper-cli -n "__fish_use_subcommand" -a w -d "Watch (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a list -d "List clips"
complete -c clipper-cli -n "__fish_use_subcommand" -a l -d "List clips (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a upload -d "Upload a file to create a clip"
complete -c clipper-cli -n "__fish_use_subcommand" -a share -d "Create a short URL for a clip"
complete -c clipper-cli -n "__fish_use_subcommand" -a export -d "Export all clips to tar.gz"
complete -c clipper-cli -n "__fish_use_subcommand" -a e -d "Export (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a import -d "Import clips from tar.gz"
complete -c clipper-cli -n "__fish_use_subcommand" -a i -d "Import (alias)"
complete -c clipper-cli -n "__fish_use_subcommand" -a search-tag -d "Search tags"
complete -c clipper-cli -n "__fish_use_subcommand" -a st -d "Search tags (alias)"

# create options
complete -c clipper-cli -n "__fish_seen_subcommand_from create c" -s t -l tags -d "Tags (comma-separated)" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from create c" -s n -l notes -d "Additional notes" -r

# get options
complete -c clipper-cli -n "__fish_seen_subcommand_from get g" -s f -l format -d "Output format" -ra "json text"

# update options
complete -c clipper-cli -n "__fish_seen_subcommand_from update u" -s t -l tags -d "New tags" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from update u" -s n -l notes -d "New notes" -r

# search options
complete -c clipper-cli -n "__fish_seen_subcommand_from search s" -s t -l tags -d "Filter by tags" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from search s" -l start-date -d "Start date (ISO 8601)" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from search s" -l end-date -d "End date (ISO 8601)" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from search s" -s p -l page -d "Page number" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from search s" -l page-size -d "Items per page" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from search s" -s f -l format -d "Output format" -ra "json text"

# list options
complete -c clipper-cli -n "__fish_seen_subcommand_from list l" -s t -l tags -d "Filter by tags" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from list l" -l start-date -d "Start date (ISO 8601)" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from list l" -l end-date -d "End date (ISO 8601)" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from list l" -s p -l page -d "Page number" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from list l" -l page-size -d "Items per page" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from list l" -s f -l format -d "Output format" -ra "json text"

# upload options
complete -c clipper-cli -n "__fish_seen_subcommand_from upload" -s t -l tags -d "Tags" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from upload" -s n -l notes -d "Additional notes" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from upload" -s c -l content -d "Content override" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from upload" -F

# share options
complete -c clipper-cli -n "__fish_seen_subcommand_from share" -s e -l expires -d "Expiration in hours" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from share" -s f -l format -d "Output format" -ra "url json"

# export options
complete -c clipper-cli -n "__fish_seen_subcommand_from export e" -s o -l output -d "Output file path" -r

# import options
complete -c clipper-cli -n "__fish_seen_subcommand_from import i" -s f -l format -d "Output format" -ra "text json"
complete -c clipper-cli -n "__fish_seen_subcommand_from import i" -F

# search-tag options
complete -c clipper-cli -n "__fish_seen_subcommand_from search-tag st" -s p -l page -d "Page number" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from search-tag st" -l page-size -d "Items per page" -r
complete -c clipper-cli -n "__fish_seen_subcommand_from search-tag st" -s f -l format -d "Output format" -ra "text json"
FISH_COMPLETIONS

    echo -e "${GREEN}Created: $OUTPUT_DIR/clipper-cli.fish${NC}"

    # Generate PowerShell completions
    cat > "$OUTPUT_DIR/clipper-cli.ps1" << 'PS_COMPLETIONS'
# PowerShell completion for clipper-cli
# Add to your PowerShell profile: . /path/to/clipper-cli.ps1

using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName clipper-cli -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @{
        'create' = 'Create a new clip'
        'c' = 'Create a new clip (alias)'
        'get' = 'Get a clip by ID'
        'g' = 'Get a clip by ID (alias)'
        'update' = "Update a clip's tags and/or notes"
        'u' = 'Update a clip (alias)'
        'search' = 'Search clips'
        's' = 'Search clips (alias)'
        'delete' = 'Delete a clip by ID'
        'd' = 'Delete a clip (alias)'
        'watch' = 'Watch for real-time notifications'
        'w' = 'Watch (alias)'
        'list' = 'List clips'
        'l' = 'List clips (alias)'
        'upload' = 'Upload a file to create a clip'
        'share' = 'Create a short URL for a clip'
        'export' = 'Export all clips to tar.gz'
        'e' = 'Export (alias)'
        'import' = 'Import clips from tar.gz'
        'i' = 'Import (alias)'
        'search-tag' = 'Search tags'
        'st' = 'Search tags (alias)'
    }

    $globalOptions = @(
        @{ Name = '-c'; Description = 'Path to config file' }
        @{ Name = '--config'; Description = 'Path to config file' }
        @{ Name = '-u'; Description = 'Server URL' }
        @{ Name = '--url'; Description = 'Server URL' }
        @{ Name = '-t'; Description = 'Bearer token for authentication' }
        @{ Name = '--token'; Description = 'Bearer token for authentication' }
        @{ Name = '-h'; Description = 'Show help' }
        @{ Name = '--help'; Description = 'Show help' }
    )

    $elements = $commandAst.CommandElements
    $command = $null

    for ($i = 1; $i -lt $elements.Count; $i++) {
        $element = $elements[$i].Extent.Text
        if ($commands.ContainsKey($element)) {
            $command = $element
            break
        }
    }

    if ($null -eq $command) {
        # Complete commands
        $commands.Keys | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [CompletionResult]::new($_, $_, 'ParameterValue', $commands[$_])
        }
        # Complete global options
        $globalOptions | Where-Object { $_.Name -like "$wordToComplete*" } | ForEach-Object {
            [CompletionResult]::new($_.Name, $_.Name, 'ParameterName', $_.Description)
        }
    }
    else {
        # Command-specific completions
        $options = switch -Wildcard ($command) {
            'create' { @('--tags', '-t', '--notes', '-n') }
            'c' { @('--tags', '-t', '--notes', '-n') }
            'get' { @('--format', '-f') }
            'g' { @('--format', '-f') }
            'update' { @('--tags', '-t', '--notes', '-n') }
            'u' { @('--tags', '-t', '--notes', '-n') }
            'search' { @('--tags', '-t', '--start-date', '--end-date', '--page', '-p', '--page-size', '--format', '-f') }
            's' { @('--tags', '-t', '--start-date', '--end-date', '--page', '-p', '--page-size', '--format', '-f') }
            'list' { @('--tags', '-t', '--start-date', '--end-date', '--page', '-p', '--page-size', '--format', '-f') }
            'l' { @('--tags', '-t', '--start-date', '--end-date', '--page', '-p', '--page-size', '--format', '-f') }
            'upload' { @('--tags', '-t', '--notes', '-n', '--content', '-c') }
            'share' { @('--expires', '-e', '--format', '-f') }
            'export' { @('--output', '-o') }
            'e' { @('--output', '-o') }
            'import' { @('--format', '-f') }
            'i' { @('--format', '-f') }
            'search-tag' { @('--page', '-p', '--page-size', '--format', '-f') }
            'st' { @('--page', '-p', '--page-size', '--format', '-f') }
            default { @() }
        }

        $options | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [CompletionResult]::new($_, $_, 'ParameterName', $_)
        }
    }
}
PS_COMPLETIONS

    echo -e "${GREEN}Created: $OUTPUT_DIR/clipper-cli.ps1${NC}"

    # Summary
    echo
    echo "Shell completions generated successfully!"
    echo
    echo "Installation instructions:"
    echo
    echo "  Bash:"
    echo "    source $OUTPUT_DIR/clipper-cli.bash"
    echo "    # Or copy to /etc/bash_completion.d/"
    echo
    echo "  Zsh:"
    echo "    mkdir -p ~/.zsh/completions"
    echo "    cp $OUTPUT_DIR/clipper-cli.zsh ~/.zsh/completions/_clipper-cli"
    echo "    # Add to .zshrc: fpath=(~/.zsh/completions \$fpath)"
    echo
    echo "  Fish:"
    echo "    cp $OUTPUT_DIR/clipper-cli.fish ~/.config/fish/completions/"
    echo
    echo "  PowerShell:"
    echo "    Add to \$PROFILE: . $OUTPUT_DIR/clipper-cli.ps1"
}

main "$@"
