// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: CC0-1.0

package ci

// Reusable step definitions
_#steps: {
	checkout: {
		uses: "actions/checkout@v4"
	}
	setup_cue: {
		name: "Install CUE"
		uses: "cue-lang/setup-cue@v1.0.1"
	}
	actionlint: {
		name: "Run actionlint"
		uses: "ivankatliarchuk/actionlint@v1"
	}
}

// Common configurations
_#defaults: {
	runner:      "ubuntu-latest"
	permissions: {contents: "read"}
}

workflows: {
	"ci.yml": {
		name:        "CI"
		on:          ["push", "pull_request"]
		permissions: _#defaults.permissions
		jobs: {
			build_and_test: {
				name:      "Build and Test"
				"runs-on": _#defaults.runner
				steps: [
					_#steps.checkout,
					{
						name: "Build all targets"
						run:  "cargo build --all-targets"
					},
					{
						name: "Run tests"
						run:  "cargo nextest run --no-fail-fast || cargo test"
					},
				]
			}
		}
	}

	"validate-generated-workflows.yml": {
		name:        "Validate Generated Workflows"
		on:          ["pull_request"]
		permissions: _#defaults.permissions
		jobs: {
			validate: {
				name:      "Validate Workflows"
				"runs-on": _#defaults.runner
				steps: [
					_#steps.checkout,
					_#steps.setup_cue,
					{
						name: "Generate workflows from CUE"
						run:  "make generate-workflows"
					},
					{
						name: "Check for uncommitted changes"
						run:  "if ! git diff --exit-code .github/workflows/; then echo 'Generated workflows are out of date. Run make generate-workflows and commit the changes.'; exit 1; fi"
					},
					_#steps.actionlint,
				]
			}
		}
	}
}
