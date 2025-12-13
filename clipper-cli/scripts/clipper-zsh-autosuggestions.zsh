#!/usr/bin/env zsh
# Clipper zsh-autosuggestions strategy
#
# This script provides a custom strategy for zsh-autosuggestions that uses
# Clipper clipboard history as a source of suggestions.
#
# Installation:
#   1. Source this file in your .zshrc (after zsh-autosuggestions):
#      source /path/to/clipper-zsh-autosuggestions.zsh
#
#   2. Add 'clipper' to your ZSH_AUTOSUGGEST_STRATEGY:
#      ZSH_AUTOSUGGEST_STRATEGY=(clipper history completion)
#
#   3. Optionally configure the clipper-cli path:
#      CLIPPER_CLI_PATH=/path/to/clipper-cli
#
#      On macOS with Clipper.app, it may be:
#      /Applications/Clipper.app/Contents/MacOS/clipper-cli
#
# Requirements:
#   - zsh-autosuggestions plugin
#   - clipper-cli in PATH (or set CLIPPER_CLI_PATH)
#   - clipper-server running
#
# Configuration:
#   CLIPPER_CLI_PATH        - Path to clipper-cli binary (default: clipper-cli)
#   CLIPPER_SUGGEST_LIMIT   - Max suggestions to fetch (default: 10)
#   CLIPPER_SUGGEST_TAGS    - Filter by tags, comma-separated (default: none)
#   CLIPPER_SUGGEST_LOCAL   - Only show clips from this computer (default: 1)

# Default configuration
: ${CLIPPER_CLI_PATH:=clipper-cli}
: ${CLIPPER_SUGGEST_LIMIT:=10}
: ${CLIPPER_SUGGEST_TAGS:=}
: ${CLIPPER_SUGGEST_LOCAL:=1}

# Custom strategy function for zsh-autosuggestions
# This function is called by zsh-autosuggestions to get suggestions
_zsh_autosuggest_strategy_clipper() {
    # $1 is the current buffer (what the user has typed)
    local prefix="$1"

    # Skip if prefix is too short (less than 2 characters)
    [[ ${#prefix} -lt 2 ]] && return

    # Build tags filter
    local tags_filter="$CLIPPER_SUGGEST_TAGS"

    # Add host tag filter if local-only mode is enabled
    if [[ "$CLIPPER_SUGGEST_LOCAL" == "1" ]]; then
        local host_tag="\$host:$(hostname)"
        if [[ -n "$tags_filter" ]]; then
            tags_filter="$tags_filter,$host_tag"
        else
            tags_filter="$host_tag"
        fi
    fi

    # Build the clipper-cli command
    local cmd=("$CLIPPER_CLI_PATH" search "$prefix" --page-size "$CLIPPER_SUGGEST_LIMIT" --format json)

    # Add tags filter if configured
    if [[ -n "$tags_filter" ]]; then
        cmd+=(--tags "$tags_filter")
    fi

    local -a items
    items=("${(@0)$($cmd | jq -j '.items[] | .content + "\u0000"')}")

    # 2. Iterate through the array to find a match
    for item in "${items[@]}"; do
        # Check if the item starts with the current prefix
        if [[ "$item" == "$prefix"* ]]; then
            # 3. Set the global suggestion variable and exit
            typeset -g suggestion="$item"
            return
        fi
    done
}

# Register the strategy with zsh-autosuggestions
# This makes 'clipper' available as a strategy name
if (( $+functions[_zsh_autosuggest_strategy_] )); then
    # zsh-autosuggestions is loaded, we can register
    :
else
    # Define a stub that will be picked up when zsh-autosuggestions loads
    :
fi

# Helper function to test the clipper integration
clipper-suggest-test() {
    local query="${1:-test}"
    echo "Testing clipper suggestion for: $query"
    echo "Using CLI: $CLIPPER_CLI_PATH"
    echo "---"

    local result
    result=$("$CLIPPER_CLI_PATH" search "$query" --page-size 5 --format text 2>&1)
    local exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        echo "Results:"
        echo "$result"
    else
        echo "Error (exit code $exit_code):"
        echo "$result"
    fi
}

# Print info when sourced
if [[ -o interactive ]]; then
    # Only print if this is an interactive shell
    : # Silent by default, uncomment below for verbose
    # echo "Clipper zsh-autosuggestions strategy loaded"
    # echo "  Add 'clipper' to ZSH_AUTOSUGGEST_STRATEGY to enable"
fi
