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
  -
- `riktl` supports the following resource `types`:

  - instance
  - workload

- `riktl` supports the following `options`:
  - --workload
  - -f | --file
  - --id
  - --replicas

## Commands

---

### Create a workload from a YAML file

- `riktl create workload -f work.yaml`

### Delete a workload.

- `riktl delete workload --workload <workload-id>`

### Create an instance

- ` riktl create instance --workload <workload-id> [--replicas N]`

### Delete an instance

- `riktl delete instance --instance <instance-id>`

### View a workload

- `riktl get workload [--id <workload-id>]`

### View an instance

- `riktl get instance [--id <instance-id>]`

### Lint a YAML file

Lint Yaml file with [Yaml validator crate](https://crates.io/crates/yaml-validator)

- `riktl lint workload.yaml`

Linter auto triggers on rik deploy but can be executed manually before deploy.\_

### Help

#### Global help

- `riktl --help`

####  Help on a specific entity

- `rik pods|pod|po --help|-h`

## Workload description

TBD (we need openapi of the controller to determine this)

> _workload.yaml_ :

```
kind: instance
name: nginx-server
replicas: 3
image: nginx:1.9.0 or url
env:
  - name: ENV
    value: prod
 - name: USER
    value: john
 ...
```

## Next RIK versions

---

- Authentication system for multi tenant usage ? (`rik register`, `rik login`)
- Possibility to use the CLI in an imperative way ?
- Create workload in a full command line ? (`riktl create workload --name NAME --image IMAGE ...`)

## Authors

---

- [Julian Labatut](https://github.com/jlabatut)
- [Mathias Flagey](https://github.com/NelopsisCode)
