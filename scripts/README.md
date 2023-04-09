# Scripts folder

This directory contains a few scripts that can be used to help our CI and our developers to work with RIK.
There is a single entrypoint for all scripts which is `tool.sh`, you can run `./scripts/tool.sh --help` to
learn about available commands.

## Available commands

### `mkkernel`

Creates a new kernel that can be used as a kernel base for VM Workloads. The created kernel is 
available at `scripts/vmlinux.bin`

### `mkrootfs`

Creates a new rootfs based on `ubuntu:22.04` image with a few initial packages. It weight 150 MB, it's large,
but the aime of this rootfs isn't to be small, but to be a base system that can be used for testing purposes.