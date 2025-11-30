CUE_DIR := internal/ci
WORKFLOWS_DIR := .github/workflows
CUE_SRC := $(CUE_DIR)/workflows.cue

SPDX_HEADER := \# SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>\n\#\n\# SPDX-License-Identifier: CC0-1.0\n\n\# This file is generated from internal/ci/workflows.cue\n\# Do not edit directly. Run make generate-workflows to regenerate.\n\n

# Generated workflow files
WORKFLOW_CI := $(WORKFLOWS_DIR)/ci.yml
WORKFLOW_VALIDATE := $(WORKFLOWS_DIR)/validate-generated-workflows.yml

.PHONY: generate-workflows
generate-workflows: $(WORKFLOW_CI) $(WORKFLOW_VALIDATE)

$(WORKFLOWS_DIR):
	@mkdir -p $@

$(WORKFLOW_CI): $(CUE_SRC) | $(WORKFLOWS_DIR)
	@printf '$(SPDX_HEADER)' > $@
	@cd $(CUE_DIR) && cue export --out json -e 'workflows."ci.yml"' | yq -P >> $(CURDIR)/$@
	@echo "Generated $@"

$(WORKFLOW_VALIDATE): $(CUE_SRC) | $(WORKFLOWS_DIR)
	@printf '$(SPDX_HEADER)' > $@
	@cd $(CUE_DIR) && cue export --out json -e 'workflows."validate-generated-workflows.yml"' | yq -P >> $(CURDIR)/$@
	@echo "Generated $@"
