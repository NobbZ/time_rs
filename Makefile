CUE_DIR := internal/ci
WORKFLOWS_DIR := .github/workflows

SPDX_HEADER := \# SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>\n\#\n\# SPDX-License-Identifier: CC0-1.0\n\n\# This file is generated from internal/ci/workflows.cue\n\# Do not edit directly. Run make generate-workflows to regenerate.\n\n

.PHONY: generate-workflows
generate-workflows:
	@mkdir -p $(WORKFLOWS_DIR)
	@cd $(CUE_DIR) && cue export --out json -e workflows | jq -r 'keys[]' | while read -r filename; do \
		printf '$(SPDX_HEADER)' > $(CURDIR)/$(WORKFLOWS_DIR)/$$filename; \
		cue export --out json -e "workflows.\"$$filename\"" | yq -P >> $(CURDIR)/$(WORKFLOWS_DIR)/$$filename; \
		echo "Generated $(WORKFLOWS_DIR)/$$filename"; \
	done
