// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: CC0-1.0

package ci

workflows: "ci.yml": {
	name: "CI"
	on: ["push", "pull_request"]
	jobs: {
		build_and_test: {
			name: "Build and Test"
			steps: [
				_steps.checkout,
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
