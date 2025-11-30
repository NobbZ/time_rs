// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: CC0-1.0

package ci

workflows: "commit_checks.yml": {
	name: "Commit Checks"
	on: ["push", "pull_request"]
	jobs: {
		generate_matrix: {
			outputs: checks: "${{ steps.gen_checks.outputs.checks }}"
			steps: [
				_steps.checkout,
				{
					id: "gen_checks"
					run: """
						set -ex
						yq --version
						checks=$(yq -o j . .pre-commit-config.yaml | jq -c '.repos | map(.hooks | map(.id)) | flatten')
						printf "checks=%s" "$checks" >> $GITHUB_OUTPUT
						"""
				},
			]
		}

		run_checks: {
			needs: ["generate_matrix"]
			strategy: {
				"fail-fast":    false
				"max-parallel": 5
				matrix: check: "${{ fromJson(needs.generate_matrix.outputs.checks) }}"
			}
			steps: [
				_steps.checkout,
				_steps.installNix,
				_steps.installRust,
				{
					name: "run ${{ matrix.check }}"
					run:  "nix develop . -c pre-commit run ${{ matrix.check }} --all-files --verbose"
				},
			]
		}
	}
}
