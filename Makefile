CUE_DIR := internal/ci
WORKFLOWS_DIR := .github/workflows
CUE_SRC := $(wildcard $(CUE_DIR)/*.cue)

SPDX_HEADER := \# SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>\n\#\n\# SPDX-License-Identifier: CC0-1.0\n\n\# This file is generated from internal/ci/\n\# Do not edit directly. Run make generate-workflows to regenerate.\n\n

# Generated workflow files
WORKFLOW_CI := $(WORKFLOWS_DIR)/ci.yml
WORKFLOW_VALIDATE := $(WORKFLOWS_DIR)/validate-generated-workflows.yml
WORKFLOW_COMMIT_CHECKS := $(WORKFLOWS_DIR)/commit_checks.yml

WORKFLOWS := $(WORKFLOW_CI) $(WORKFLOW_VALIDATE) $(WORKFLOW_COMMIT_CHECKS)

.PHONY: all workflows check

all: workflows

workflows: $(WORKFLOWS)

check:
	cue vet -c ./internal/ci/ $(WORKFLOW_CI) -d 'workflows."ci.yml"'
	cue vet -c ./internal/ci/ $(WORKFLOW_VALIDATE) -d 'workflows."validate-generated-workflows.yml"'
	cue vet -c ./internal/ci/ $(WORKFLOW_COMMIT_CHECKS) -d 'workflows."commit_checks.yml"'

$(WORKFLOWS_DIR):
	@mkdir -p $@

define generate-workflow
$(WORKFLOWS_DIR)/$(1): $(CUE_SRC) | $(WORKFLOWS_DIR)
	printf '$$(SPDX_HEADER)' > $$@
	cue export --out yaml ./internal/ci/ -e 'workflows."$(1)"' >> $$@
endef

$(foreach wf,ci.yml validate-generated-workflows.yml commit_checks.yml,$(eval $(call generate-workflow,$(wf))))
