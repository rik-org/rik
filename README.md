<p align="center">
  <img src="https://i.imgur.com/22sf4x7.png" />
</p>
<p align="center"><strong>A lightweight cloud orchestrator in Rust</strong></p>
<p align="center">❗ This project is experimental and should NOT be used in production. ❗</p>

<div>
<img src="https://img.shields.io/badge/Rik-rust-orange?style=for-the-badge&logo=appveyor" />
<img src="https://img.shields.io/github/workflow/status/thomasgouveia/rik/RIK%20CI?style=for-the-badge" />
<img alt="Discord" src="https://img.shields.io/discord/863020591984148502?style=for-the-badge">
</div>

### Contributing

**RIK** is open-source and contributions are welcome. Please read the [CONTRIBUTING.md](CONTRIBUTING.md) for more information on how to contribute to this project.

## Troubleshooting

**`cargo build` fails because cannot build `openssl-sys`**

This is due to missing packages in your system, install `libssl-dev` to fix this.

- Ubuntu: `sudo apt update && sudo apt install libssl-dev protobuf-compiler`
