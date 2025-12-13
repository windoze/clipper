# Clipper fish autosuggestions plugin
#
# This script provides autosuggestions from Clipper clipboard history for fish shell.
#
# Installation:
#   1. Copy this file to ~/.config/fish/conf.d/clipper-autosuggestions.fish
#      Or source it in your config.fish:
#      source /path/to/clipper-fish-autosuggestions.fish
#
# Requirements:
#   - fish shell 3.0+
#   - clipper-cli in PATH (or set CLIPPER_CLI_PATH)
#   - jq for JSON parsing
#   - clipper-server running
#
# Configuration (set in config.fish before sourcing):
#   set -g CLIPPER_CLI_PATH /path/to/clipper-cli
#   set -g CLIPPER_SUGGEST_LIMIT 10
#   set -g CLIPPER_SUGGEST_TAGS "tag1,tag2"
#   set -g CLIPPER_SUGGEST_LOCAL 1  # Only show clips from this computer
#
# Usage:
#   Type at least 2 characters and suggestions from your clip history will appear.
#   Press Right arrow or Ctrl-F to accept the suggestion.

# Default configuration
if not set -q CLIPPER_CLI_PATH
    set -g CLIPPER_CLI_PATH clipper-cli
end

if not set -q CLIPPER_SUGGEST_LIMIT
    set -g CLIPPER_SUGGEST_LIMIT 10
end

if not set -q CLIPPER_SUGGEST_TAGS
    set -g CLIPPER_SUGGEST_TAGS ""
end

if not set -q CLIPPER_SUGGEST_LOCAL
    set -g CLIPPER_SUGGEST_LOCAL 1
end

# Custom autosuggestion function for clipper
function __clipper_suggest
    set -l prefix (commandline -cp)

    # Skip if prefix is too short
    if test (string length "$prefix") -lt 2
        return
    end

    # Build tags filter
    set -l tags_filter "$CLIPPER_SUGGEST_TAGS"

    # Add host tag filter if local-only mode is enabled
    if test "$CLIPPER_SUGGEST_LOCAL" = "1"
        set -l host_tag "\$host:"(hostname)
        if test -n "$tags_filter"
            set tags_filter "$tags_filter,$host_tag"
        else
            set tags_filter "$host_tag"
        end
    end

    # Build command
    set -l cmd $CLIPPER_CLI_PATH search "$prefix" --page-size $CLIPPER_SUGGEST_LIMIT --format json

    # Add tags filter if configured
    if test -n "$tags_filter"
        set cmd $cmd --tags $tags_filter
    end

    # Execute and find matching suggestion
    set -l items (eval $cmd 2>/dev/null | jq -r '.items[].content' 2>/dev/null)

    for item in $items
        # Check if item starts with prefix
        if string match -q "$prefix*" "$item"
            echo $item
            return
        end
    end
end

# Register as custom autosuggestion provider
# Fish doesn't have a plugin system like zsh-autosuggestions,
# so we provide helper functions and key bindings

# Function to insert clipper suggestion
function clipper-autosuggest-accept
    set -l suggestion (__clipper_suggest)
    if test -n "$suggestion"
        commandline -r $suggestion
        commandline -f end-of-line
    end
end

# Function to show clipper suggestion as ghost text
function clipper-autosuggest-show
    set -l current (commandline -cp)
    set -l suggestion (__clipper_suggest)

    if test -n "$suggestion"
        # Calculate the part to show as suggestion
        set -l suffix (string sub -s (math (string length "$current") + 1) "$suggestion")
        if test -n "$suffix"
            echo -n (set_color brblack)"$suffix"(set_color normal)
        end
    end
end

# Interactive clipper search with fzf (if available)
function clipper-fzf-search
    if not command -q fzf
        echo "fzf is required for interactive search" >&2
        return 1
    end

    if not command -q jq
        echo "jq is required for JSON parsing" >&2
        return 1
    end

    set -l selected (eval $CLIPPER_CLI_PATH list --page-size 100 --format json 2>/dev/null | \
        jq -r '.items[] | "\(.id)\t\(.content | gsub("\n"; "↵") | .[0:80])"' 2>/dev/null | \
        fzf --delimiter='\t' --with-nth=2 --preview="$CLIPPER_CLI_PATH get {1} --format text 2>/dev/null" | \
        cut -f1)

    if test -n "$selected"
        set -l content (eval $CLIPPER_CLI_PATH get "$selected" --format text 2>/dev/null)
        commandline -i $content
    end

    commandline -f repaint
end

# Key bindings
# Ctrl-Alt-C: Insert clipper suggestion
bind \e\cc clipper-autosuggest-accept

# Ctrl-Alt-S: Interactive clipper search with fzf
bind \e\cs clipper-fzf-search

# Test function
function clipper-suggest-test -d "Test clipper suggestion for a query"
    set -l query $argv[1]
    if test -z "$query"
        set query "test"
    end

    echo "Testing clipper suggestion for: $query"
    echo "Using CLI: $CLIPPER_CLI_PATH"
    echo "---"

    set -l result (eval $CLIPPER_CLI_PATH search "$query" --page-size 5 --format text 2>&1)
    set -l exit_code $status

    if test $exit_code -eq 0
        echo "Results:"
        echo $result
    else
        echo "Error (exit code $exit_code):"
        echo $result
    end
end

# Abbreviation for quick clip creation
abbr -a -g clip "$CLIPPER_CLI_PATH create"
abbr -a -g clips "$CLIPPER_CLI_PATH search"
abbr -a -g clipl "$CLIPPER_CLI_PATH list"
