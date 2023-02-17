# SPDX-FileCopyrightText: 2023 Norbert Melzer <timmelzer@gmail.com>
#
# SPDX-License-Identifier: MIT
{pkgs, ...}: {
  apps.check.program = "${pkgs.writeShellScript "check.sh" ''
    set -ex

    cargo nextest run

    cargo tarpaulin --target-dir=target/coverage --skip-clean --fail-under 80
  ''}";
}
