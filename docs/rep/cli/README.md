# RIK Command Line Interface

`riktl` is RIK's command interface and tool.
It will send and receive HTTP requests to/from the REST API developed by the controler team.

It will use the crate [clap](https://crates.io/crates/clap), a command line argument parser.

## Cluster connection

---

`riktl` use a `rik.config.yaml` file to connects to a REST API server, and then calls into the exposed REST API.

> _rik.config.yaml_ :

```
cluster:
  name: cluster name
  server: API-address
```

The CLI will parse this file on every command, to get the API address.
Each request will contain the YAML file in the payload.

## Syntax

---

Use the following syntax to run `riktl` commands from your terminal window.

`riktl COMMAND TYPE [--NAME <id>] [OPTIONS]`

- `riktl` supports the following `commands`:

  - riktl `create`
  - riktl `delete`
  - riktl `get`

- `riktl` supports the following resource `types`:

  - instance
  - workload

- `riktl` supports the following `options`:
  - --workload
  - --instance
  - -f | --file
  - -n | --replicas

## Commands

---

### Create a workload from a JSON file

- `riktl create workload -f work.json`

### Delete a workload.

- `riktl delete workload --workload <workload-id>`

### Create an instance

- ` riktl create instance --workload <workload-id> [--replicas N]`

### Delete an instance

- `riktl delete instance --instance <instance-id>`

### Get all workloads

- `riktl get workload`

### Help

- `riktl --help | -h`

## Workload description

> _workload.json_ :

```json
{
  "api_version": "v0",
  "kind": "pods",
  "name": "workload-name",
  "spec": {
    "containers": [
      {
        "name": "<name>",
        "image": "<image>",
        "env": [
          {
            "name": "key1",
            "value": "value1"
          },
          {
            "name": "key2",
            "value": "value2"
          }
        ],
        "ports": {
          "port": 80,
          "target_port": 80,
          "protocol": "TCP",
          "type": "clusterIP|nodePort|loadBalancer"
        }
      }
    ]
  }
}
```

## Next RIK versions

---

- Authentication system for multi tenant usage ? (`rik register`, `rik login`)
- Possibility to use the CLI in an imperative way ?
- Create workload in a full command line ? (`riktl create workload --name NAME --image IMAGE ...`)
- Lint JSON workload file. Linter should auto triggers on rik deploy but can be executed manually before deploy. (`riktl lint workload.yaml`)

## Authors

---

- [Julian Labatut](https://github.com/jlabatut)
- [Mathias Flagey](https://github.com/NelopsisCode)
