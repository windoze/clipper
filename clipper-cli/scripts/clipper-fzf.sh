#!/usr/bin/env bash
# Clipper fzf integration
#
# This script provides interactive clip selection using fzf with preview.
#
# Installation:
#   1. Make this script executable:
#      chmod +x clipper-fzf.sh
#
#   2. Add to your shell config (e.g., .bashrc or .zshrc):
#      source /path/to/clipper-fzf.sh
#
#   3. Or create an alias:
#      alias clipper-fzf='/path/to/clipper-fzf.sh'
#
# Requirements:
#   - fzf (https://github.com/junegunn/fzf)
#   - jq (https://stedolan.github.io/jq/)
#   - clipper-cli in PATH (or set CLIPPER_CLI_PATH)
#   - clipper-server running
#
# Configuration:
#   CLIPPER_CLI_PATH      - Path to clipper-cli binary (default: clipper-cli)
#   CLIPPER_FZF_PAGE_SIZE - Number of clips to load (default: 100)
#   CLIPPER_FZF_PREVIEW   - Enable preview (default: 1)

set -euo pipefail

# Default configuration
: "${CLIPPER_CLI_PATH:=clipper-cli}"
: "${CLIPPER_FZF_PAGE_SIZE:=100}"
: "${CLIPPER_FZF_PREVIEW:=1}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check dependencies
check_deps() {
    local missing=()
    command -v fzf >/dev/null 2>&1 || missing+=("fzf")
    command -v jq >/dev/null 2>&1 || missing+=("jq")
    command -v "$CLIPPER_CLI_PATH" >/dev/null 2>&1 || missing+=("clipper-cli")

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo -e "${RED}Error: Missing dependencies: ${missing[*]}${NC}" >&2
        exit 1
    fi
}

# Get clip preview by ID
get_clip_preview() {
    local id="$1"
    "$CLIPPER_CLI_PATH" get "$id" --format json 2>/dev/null | jq -r '
        "ID: \(.id)",
        "Created: \(.created_at)",
        "Tags: \(.tags | join(", "))",
        "Notes: \(.additional_notes // "none")",
        "---",
        .content
    ' 2>/dev/null || echo "Failed to load clip"
}

# Export for fzf preview
export -f get_clip_preview
export CLIPPER_CLI_PATH

# Interactive clip selection
clipper_fzf_select() {
    local query="${1:-}"
    local output_format="${2:-content}"  # content, id, or json

    check_deps

    # Build the list command
    local list_cmd
    if [[ -n "$query" ]]; then
        list_cmd="$CLIPPER_CLI_PATH search '$query' --page-size $CLIPPER_FZF_PAGE_SIZE --format json"
    else
        list_cmd="$CLIPPER_CLI_PATH list --page-size $CLIPPER_FZF_PAGE_SIZE --format json"
    fi

    # Get clips and format for fzf
    # Format: ID | CONTENT_PREVIEW | TAGS
    local clips
    clips=$(eval "$list_cmd" 2>/dev/null | jq -r '
        .items[] |
        "\(.id)\t\(.content | gsub("\n"; "↵") | .[0:80])\t[\(.tags | join(", "))]"
    ' 2>/dev/null)

    if [[ -z "$clips" ]]; then
        echo -e "${YELLOW}No clips found${NC}" >&2
        return 1
    fi

    # Build fzf command
    local fzf_opts=(
        --delimiter='\t'
        --with-nth=2,3
        --header='Select a clip (TAB to preview, ENTER to select)'
        --preview-window='right:50%:wrap'
        --bind='ctrl-y:execute-silent(echo -n {1} | pbcopy 2>/dev/null || echo -n {1} | xclip -selection clipboard 2>/dev/null || true)'
        --bind='ctrl-d:execute($CLIPPER_CLI_PATH delete {1} >/dev/null 2>&1)+reload('"$list_cmd"' | jq -r ".items[] | \"\(.id)\t\(.content | gsub(\"\n\"; \"↵\") | .[0:80])\t[\(.tags | join(\", \"))]\"" 2>/dev/null)'
    )

    if [[ "$CLIPPER_FZF_PREVIEW" == "1" ]]; then
        fzf_opts+=(--preview="bash -c 'get_clip_preview {1}'")
    fi

    # Run fzf and get selection
    local selected
    selected=$(echo "$clips" | fzf "${fzf_opts[@]}")

    if [[ -z "$selected" ]]; then
        return 1
    fi

    # Extract ID from selection
    local id
    id=$(echo "$selected" | cut -f1)

    # Output based on format
    case "$output_format" in
        id)
            echo "$id"
            ;;
        json)
            "$CLIPPER_CLI_PATH" get "$id" --format json
            ;;
        content|*)
            "$CLIPPER_CLI_PATH" get "$id" --format text
            ;;
    esac
}

# Interactive clip search with live preview
clipper_fzf_search() {
    check_deps

    # Use fzf with dynamic reloading based on query
    local reload_cmd="$CLIPPER_CLI_PATH search {q} --page-size $CLIPPER_FZF_PAGE_SIZE --format json 2>/dev/null | jq -r '.items[] | \"\(.id)\t\(.content | gsub(\"\n\"; \"↵\") | .[0:80])\t[\(.tags | join(\", \"))]\"' 2>/dev/null || true"

    local selected
    selected=$(: | fzf \
        --delimiter='\t' \
        --with-nth=2,3 \
        --header='Type to search clips (ENTER to select)' \
        --preview-window='right:50%:wrap' \
        --preview="bash -c 'get_clip_preview {1}'" \
        --bind="change:reload($reload_cmd)" \
        --phony \
        --query="")

    if [[ -z "$selected" ]]; then
        return 1
    fi

    local id
    id=$(echo "$selected" | cut -f1)
    "$CLIPPER_CLI_PATH" get "$id" --format text
}

# Copy selected clip to system clipboard
clipper_fzf_copy() {
    local content
    content=$(clipper_fzf_select "$@")

    if [[ -n "$content" ]]; then
        # Try different clipboard commands
        if command -v pbcopy >/dev/null 2>&1; then
            echo -n "$content" | pbcopy
            echo -e "${GREEN}Copied to clipboard (pbcopy)${NC}" >&2
        elif command -v xclip >/dev/null 2>&1; then
            echo -n "$content" | xclip -selection clipboard
            echo -e "${GREEN}Copied to clipboard (xclip)${NC}" >&2
        elif command -v xsel >/dev/null 2>&1; then
            echo -n "$content" | xsel --clipboard --input
            echo -e "${GREEN}Copied to clipboard (xsel)${NC}" >&2
        elif command -v wl-copy >/dev/null 2>&1; then
            echo -n "$content" | wl-copy
            echo -e "${GREEN}Copied to clipboard (wl-copy)${NC}" >&2
        else
            echo -e "${RED}No clipboard command found${NC}" >&2
            echo "$content"
        fi
    fi
}

# Shell functions for easy access
clipper-select() { clipper_fzf_select "$@"; }
clipper-search() { clipper_fzf_search "$@"; }
clipper-copy() { clipper_fzf_copy "$@"; }

# Main entry point when run as script
main() {
    local cmd="${1:-select}"
    shift || true

    case "$cmd" in
        select|s)
            clipper_fzf_select "$@"
            ;;
        search|/)
            clipper_fzf_search "$@"
            ;;
        copy|c)
            clipper_fzf_copy "$@"
            ;;
        help|--help|-h)
            cat <<EOF
Clipper fzf integration - Interactive clip selection

Usage: $(basename "$0") [command] [options]

Commands:
    select, s [query]   Select a clip (optionally with initial search query)
    search, /           Interactive search with live preview
    copy, c [query]     Select and copy clip to system clipboard
    help                Show this help message

Key bindings in fzf:
    ENTER       Select clip
    TAB         Toggle preview
    Ctrl-Y      Copy clip ID to clipboard
    Ctrl-D      Delete selected clip
    ESC         Cancel

Environment variables:
    CLIPPER_CLI_PATH      Path to clipper-cli (default: clipper-cli)
    CLIPPER_FZF_PAGE_SIZE Number of clips to load (default: 100)
    CLIPPER_FZF_PREVIEW   Enable preview, 0 or 1 (default: 1)

Examples:
    $(basename "$0") select              # Browse all clips
    $(basename "$0") select "todo"       # Browse clips matching "todo"
    $(basename "$0") search              # Interactive search
    $(basename "$0") copy                # Select and copy to clipboard
EOF
            ;;
        *)
            echo "Unknown command: $cmd" >&2
            echo "Run '$(basename "$0") help' for usage" >&2
            exit 1
            ;;
    esac
}

# Run main if executed as script (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
