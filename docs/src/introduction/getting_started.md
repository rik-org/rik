# Getting started

## Prerequisites

> Our worker component, **Riklet**, is currently only available for Linux
> systems with a x86_64 architecture.

* [Git](https://git-scm.com/downloads)
* From source install: [Rust](https://www.rust-lang.org/tools/install),
  [protoc](https://grpc.io/docs/protoc-installation/)(>=3.15.0),

Install packages required to build the project:

- Ubuntu: `sudo apt update && sudo apt install curl libssl-dev protobuf-compiler git pkg-config umoci skopeo build-essential iptables`
- Fedora: `sudo dnf update && sudo dnf install curl openssl-devel protobuf-compiler protobuf-devel git pkg-config umoci skopeo build-essential iptables`

You can install Rust using [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Clone the project

Start by cloning the project using Git:

```bash
git clone https://github.com/rik-org/rik.git
cd rik
```

## Start a cluster


### Build from source

> Be aware that each component of the project is a **separate** binary, and
> that you
> need to execute them in a specific order.

Build all components of the project

```bash
cargo build --release
```

Start the scheduler in a terminal
```bash
cargo run --release --bin scheduler
```

Start the controller in a terminal
```bash
DATABASE_LOCATION=~/.rik/data cargo run --release --bin controller
```

Start the worker in a terminal
```bash
sudo ./target/release/riklet
```

If you experience any issue, please refer to the [troubleshooting](../troubleshooting.md),
if you can't find a solution, please open an issue.

## Create your first workload

### Create a definition of your workload

Create a workload using example in `examples/workload-1.json`:

```bash
# Create an alpine container workload
RIKCONFIG=docs/src/examples/config.json cargo run \
  --bin rikctl --release -- create workloads \
  --file docs/src/examples/workloads/workload-1.json

# The ID of the workload is returned, it will be useful next
# Workload alpine has been successfully created with ID : "0e4c1da4-0277-4088-9f37-8f445cbe8e46"
```

### Deploy an instance

Based on your workload ID you can now deploy an instance:

```bash
# Please replace the following value with the ID of your workload
export WORKLOAD_ID=0e4c1da4-0277-4088-9f37-8f445cbe8e46

RIKCONFIG=docs/src/examples/config.json cargo run \
  --bin rikctl --release -- create instance \
  --workload-id ${WORKLOAD_ID}
```

### Check your instance

You should now see your instance running:

```bash
RIKCONFIG=docs/src/examples/config.json cargo run \
  --bin rikctl --release -- get instances
```

## Configuration

You can configure a remote cluster by setting the `RIKCONFIG` environment variable
to the path of a configuration file. Here is an example of configuration:

```json
{{#include ../examples/config.json}}
```