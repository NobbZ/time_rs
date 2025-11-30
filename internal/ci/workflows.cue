// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: CC0-1.0

package ci

workflows: {
	"ci.yml": {
		name: "CI"
		on: ["push", "pull_request"]
		jobs: {
			build_and_test: {
				name:        "Build and Test"
				"runs-on":   "ubuntu-latest"
				steps: [
					{
						uses: "actions/checkout@v4"
					},
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
		name: "Validate Generated Workflows"
		on: ["pull_request"]
		jobs: {
			validate: {
				name:      "Validate Workflows"
				"runs-on": "ubuntu-latest"
				steps: [
					{
						uses: "actions/checkout@v4"
					},
					{
						name: "Install CUE"
						uses: "cue-lang/setup-cue@v1.0.1"
					},
					{
						name: "Generate workflows from CUE"
						run:  "./scripts/generate-workflows.sh"
					},
					{
						name: "Check for uncommitted changes"
						run:  "if ! git diff --exit-code .github/workflows/; then echo 'Generated workflows are out of date. Run ./scripts/generate-workflows.sh and commit the changes.'; exit 1; fi"
					},
					{
						name: "Run actionlint"
						uses: "ivankatliarchuk/actionlint@v1"
					},
				]
			}
		}
	}
}
