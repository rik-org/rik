<p align="center">
  <img src="https://i.imgur.com/22sf4x7.png" />
</p>
<p align="center"><strong>A lightweight cloud orchestrator in Rust</strong></p>
<p align="center">❗ This project is experimental and should NOT be used in production. ❗</p>

<div>
<img src="https://img.shields.io/badge/Rik-rust-orange?style=for-the-badge&logo=appveyor" />
<img src="https://img.shields.io/github/workflow/status/thomasgouveia/rik/RIK%20CI?style=for-the-badge" />
<!-- <img alt="Discord" src="https://img.shields.io/discord/863020591984148502?style=for-the-badge">
</div> -->

### How to start

mandatory scheduler before controller

```sh
cargo build
cargo run --bin scheduler
cargo run --bin controller
RIKCONFIG=$PWD/riklet/rikconfig.yml
```

You need to be root in order to run the riklet, this is due that we need to be root to create network namespaces.

```sh
./target/debug/riklet # in root
```

```sh
cargo run --bin rikctl create workload --file $PWD/examples/workloads/workload-2.json
cargo run --bin rikctl get workload
```
### Contributing

**RIK** is open-source and contributions are welcome. Please read the [CONTRIBUTING.md](CONTRIBUTING.md) for more information on how to contribute to this project.

## Troubleshooting

**`cargo build` fails because cannot build `openssl-sys`**

This is due to missing packages in your system, install `libssl-dev` to fix this.

- Ubuntu: `sudo apt update && sudo apt install libssl-dev`

**`cargo build` fails because linker `cc` not found**

This is due to missing packages in your system, install `build-essential` to fix this.

- Ubuntu: `sudo apt install build-essential`

**`cargo build` fails because it could not find `protoc`**

This is due to missing packages in your system, install `protobuf-compiler` to fix this.

- Ubuntu: `sudo apt install -y protobuf-compiler`
