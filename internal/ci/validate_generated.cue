// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: CC0-1.0

package ci

workflows: {
	"validate-generated-workflows.yml": {
		name: "Validate Generated Workflows"
		on: ["pull_request"]
		jobs: {
			validate: {
				name: "Validate Workflows"
				steps: [
					_steps.checkout,
					_steps.setup_cue,
					{
						name: "Generate workflows from CUE"
						run:  "make check"
					},
				]
			}
		}
	}
}
