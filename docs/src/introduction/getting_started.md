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

### Build from binaries

#### Download binaries

You can download the latest RIK binaries from the [releases page of the RIK repository](https://github.com/rik-org/rik/releases/latest). Here, we will install the version `v1.0.0` of RIK.

```bash
# Download the release artifacts
curl -sL "https://github.com/rik-org/rik/releases/download/v1.0.0/rik-v1.0.0-x86_64.tar.gz" --output rik.tar.gz
mkdir -p rik
tar -xvf rik.tar.gz -C rik

# Remove files that will not be useful here
rm -rf rik.tar.gz rik/LICENSE rik/README.md rik/rikctl
```

Now, if you run `tree` in your current directory, you should have something like this into the `rik` folder:

```bash
tree

-- rik
    |-- controller
    |-- riklet
    |-- scheduler

1 directory, 3 files
```

Move those binaries into `/usr/bin` :

```bash
sudo mv ./rik/* /usr/bin
```

### Install components

Now that you have gathered all the RIK components, you need to install them on your system. To do so, we will use systemd services here to run our stack. To install our different components, we will use a `service.tpl` file to make the installation easier. Create this file in your current working directory and place the following content inside :

```bash
# service.tpl
[Unit]
AssertPathExists=${BIN}

[Service]
WorkingDirectory=~
EnvironmentFile=${ENV_FILE}
ExecStart=${BIN} ${ARGS}
Restart=always
NoNewPrivileges=true
RestartSec=3

[Install]
Alias=${NAME}
WantedBy=default.target
```

#### RIK Scheduler

To install the scheduler, use the following command :

```bash
# Generate the service file based on the template
NAME="rik-scheduler" ARGS="" BIN="/usr/bin/scheduler" envsubst < service.tpl | sudo tee /etc/systemd/system/rik-scheduler.service

# Enable your service
sudo systemctl enable rik-scheduler
sudo systemctl start rik-scheduler
```

#### RIK Controller

To install the controller, use the following command :

```bash
# Generate the service file based on the template
NAME="rik-controller" ARGS="" BIN="/usr/bin/controller" envsubst < service.tpl | sudo tee /etc/systemd/system/rik-controller.service

# Enable and start your service
sudo systemctl enable rik-controller
sudo systemctl start rik-controller
```

#### RIKLET

<Callout type="warning" emoji="⚠️">
  The riklet will create some `iptables` rules at boot time. We strongly
  recommend to backup your `iptables` rules in case you need to restore them :

```bash
sudo apt install iptables
sudo iptables-save > iptables.rules.old
```

To install the riklet, we will first need to determine the name and the IP of the network interface that is connected to internet. Run the following command to retrieve it :

```bash
export IF_NET=$(ip route get 8.8.8.8 | head -n1 | awk '{print $5}')
export IF_NET_IP=$(ip -f inet addr show "${IF_NET}" | awk '/inet / {print $2}' | cut -d "/" -f 1)
```

If you want to be able to run microVMs on your host, you need to install `firecracker` on your system. Follow the instructions from the [Firecracker documentation](https://github.com/firecracker-microvm/firecracker) to install it. Alternatively, you can use the following command to install it :

```bash
wget -q https://github.com/firecracker-microvm/firecracker/releases/download/v1.3.1/firecracker-v1.3.1-$(uname -m).tgz
tar -xvzf firecracker-v1.3.1-x86_64.tgz
sudo cp release*/firecracker-* /usr/bin/firecracker
```

You also need to export the path of your `firecracker` binary on the host :

```bash
# If it is on your path, else put the path where you have installed it
export FIRECRACKER_PATH=$(which firecracker)
```

Finally, you need to download a custom linux kernel. It is needed to run Firecracker microVMs:

```bash
sudo mkdir -p /etc/riklet/
sudo curl -sL "https://morty-faas.github.io/morty-kernel.bin" --output /etc/riklet/morty-kernel.bin
```

Now, you can run the following command to generate the service file :

```bash
# Generate the service file based on the template
NAME=riklet BIN=/usr/bin/riklet ARGS="--master-ip localhost:4995 --firecracker-path ${FIRECRACKER_PATH} --kernel-path /etc/riklet/morty-kernel.bin --ifnet ${IF_NET} --ifnet-ip ${IF_NET_IP}" envsubst < service.tpl | sudo tee /etc/systemd/system/riklet.service

# Enable and start your service
sudo systemctl enable riklet
sudo systemctl start riklet
```

### Verify the installation

Run the following commands to verify that all the services are running:

```bash
systemctl is-active rik-scheduler
systemctl is-active rik-controller
systemctl is-active riklet

# You should have the following output:
active
active
active
```

Now, your RIK cluster API is up and running on http://localhost:5000.

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

## Create you first function in RIK