// SPDX-FileCopyrightText: 2025 Norbert Melzer <timmelzer@gmail.com>
//
// SPDX-License-Identifier: CC0-1.0

package ci

import "cue.dev/x/githubactions"

nixVersion: "2.30.0"

// Reusable step definitions
_steps: {
	checkout: {
		uses: "actions/checkout@v6"
	}
	installNix: {
		uses: "cachix/install-nix-action@v31"
		with: {
			install_url: "https://releases.nixos.org/nix/nix-\(nixVersion)/install"
			extra_nix_config: """
				auto-optimise-store = true
				access-tokens = github.com=${{ secrets.GITHUB_TOKEN }}
				experimental-features = nix-command flakes
				substituters = https://cache.nixos.org https://nix-community.cachix.org
				trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=
				"""
		}
	}
	installRust: {
		uses: "actions-rs/toolchain@v1"
		with: {
			toolchain:  "1.90.0"
			override:   true
			components: "clippy, rustfmt"
			profile:    "minimal"
		}
	}
	setup_cue: {
		name: "Install CUE"
		uses: "cue-lang/setup-cue@v1.0.1"
		with: version: "v0.15.1"
	}
}

_runner: *"ubuntu-latest" | _
_permissions: {contents: "read"}

workflows: [_]: githubactions.#Workflow & {
	//   permissions: _permissions
	jobs: [_]: "runs-on": _runner
}
