# Contributing guide

## Setup your developer environment

### Dependencies 

- `umoci`
- `skopeo`
- `runc`
- `protobuf-compiler`

For ubuntu :

```bash
sudo apt update
sudo apt install -y umoci skopeo runc protobuf-compiler
````

For fedora : 

```bash
sudo dnf update
sudo dnf install -y umoci skopeo runc
# To install protoc
curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v21.12/protoc-21.12-linux-x86_64.zip
unzip protoc-3.21.12-linux-x86_64 -d $HOME/.local
export PATH="$PATH:$HOME/.local/bin"
```

### Launch a rik cluster

#### Configuration

Create one file to specify the rik cluster configuration, for example `/tmp/rikconfig.yml` :
```yml
cluster:
  name: rik-demo
  server: http://127.0.0.1:5000
```

Export the `RIKCONFIG` environment variable to point to this file : 
```bash
export RIKCONFIG=/tmp/rikconfig.yml
```

You will need an example workload to test the cluster, here is one created in `/tmp/workload.json` :
```json
{
  "api_version": "v0",
  "kind": "pods",
  "name": "devopsdday-alpine",
  "spec": {
    "containers": [
      {
        "name": "alpine",
        "image": "alpine:latest"
      }
    ]
  }
}
```

#### Build and run the rik cluster

```bash
# Run & build scheduler
cd scheduler
cargo build --release
./release/scheduler

# Run & build controller
cd controller
cargo build --release
sudo ./release/controller

# Run & build riklet
cd riklet 
cargo build --release
sudo ./release/riklet

# Run & build riktl
cd riklet 
cargo build --release

# Create file rikconfig
nano /tmp/rikconfig.yml
export RIKCONFIG=/tmp/rikconfig.yml

# Create and instantiate workload
nano /tmp/workload.json
./rikctl create workload --file /tmp/workload.json
./rikctl create instance --workload-id [WORKLOAD-ID]

# Verify the container creation
sudo runc list
```

## Troubleshooting

**`cargo build` fails because cannot build `openssl-sys`**

This is due to missing packages in your system, install `libssl-dev` to fix this.

- Ubuntu: `sudo apt update && sudo apt install libssl-dev protobuf-compiler`