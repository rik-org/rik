# RIK Command Line Interface

`riktl` is RIK's command interface and tool.
It will send and receive HTTP requests to/from the REST API developed by the controler team.

It will use the crate [clap](https://crates.io/crates/clap), a command line argument parser.

## Cluster connection

---

`riktl` use a `rik.config.yaml` file to connects to a REST API server, and then calls into the exposed REST API.

TBD : will the API need an authentication token to allow the connection ?
Update : Not for the first version.

> _rik.config.yaml_ :

```
cluster:
  name: cluster name
  server: API-address
  auth-token: ApyTUlpd0JnNmRXVGpnaG5BNGhMRXVYVldNSHd1d3gxT0Nac3RmMWpqdEIv
```

The CLI will parse this file on every command, to get the API address and the auth token.
Each request will contain the authentication token in the headers, and the YAML file in the payload.

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

### Create a workload

- In a full command line

  `riktl create workload --name NAME --image IMAGE ...`

- With a YAML file

  `riktl create workload -f work.yaml`

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

#### Help on a specific entity

- `rik pods|pod|po --help|-h`

## Workload description

---

> _workload.yaml_ :

```
kind: pod
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

## Authors

---

- [Julian Labatut](https://github.com/jlabatut)
- [Mathias Flagey](https://github.com/NelopsisCode)
