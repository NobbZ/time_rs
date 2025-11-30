#!/bin/sh
# SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: CC0-1.0

# Generate GitHub Actions workflow YAML files from CUE definitions

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CUE_DIR="$REPO_ROOT/internal/ci"
WORKFLOWS_DIR="$REPO_ROOT/.github/workflows"

# SPDX header for generated files
SPDX_HEADER="# SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: CC0-1.0

# This file is generated from internal/ci/workflows.cue
# Do not edit directly. Run ./scripts/generate-workflows.sh to regenerate.

"

# Create workflows directory if it doesn't exist
mkdir -p "$WORKFLOWS_DIR"

# Export workflows from CUE and write each one to a separate YAML file
cd "$CUE_DIR"
cue export --out json -e workflows | \
    jq -r 'to_entries[] | "\(.key)\n\(.value | @json)"' | \
    while read -r filename; do
        read -r content
        printf '%s' "$SPDX_HEADER" > "$WORKFLOWS_DIR/$filename"
        echo "$content" | yq -P >> "$WORKFLOWS_DIR/$filename"
        echo "Generated $WORKFLOWS_DIR/$filename"
    done
