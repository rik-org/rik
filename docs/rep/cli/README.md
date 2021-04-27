# RIK CLI

RIK CLI is a user interface for the rustlang based cloud orchestrator **Rust In Kube**.
It will send and receive HTTP requests to/from the REST API developed by the controler team.

It will use the crate [clap](https://crates.io/crates/clap), a command line argument parser.

# Cluster connection

RIK CLI will use a `rik.config.yaml` file to establish the connection with the REST API.

TBD : will the API need an authentication token to allow the connection ?

> _rik.config.yaml_ :

```
cluster:
  name: cluster name
  server: API-address
  auth-token: ApyTUlpd0JnNmRXVGpnaG5BNGhMRXVYVldNSHd1d3gxT0Nac3RmMWpqdEIv
```

The CLI will parse this file on every command, to get the API address and the auth token.
Each request will contain the authentication token in the headers, and the YAML file in the payload.

## Commands

- **List of all pods**

  `> rik pods|pod|po list|ls`

- **Apply a workload**

  `> rik pods|pod|po apply workload.yaml`

- **Delete a pod**

  `> rik pods|pod|po delete <POD_NAME>`

- **Lint a YAML file**

  Lint Yaml file with [Yaml validator crate](https://crates.io/crates/yaml-validator)  
  `> rik lint workload.yaml`  
  _Linter auto triggers on rik deploy but can be executed manually before deploy._

- **Help**

  Global help
  `> rik --help`
  `> rik -h`

  Help on a specific entity
  `> rik pods|pod|po --help|-h`

## Workload description

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

- Authentication system for multi tenant usage ? (`rik register`, `rik login`)
- Possibility to use the CLI in an imperative way ?
- Provide multiple nodes ? Node management (create, list, delete).

### Authors

- [Julian Labatut](https://github.com/jlabatut)
- [Mathias Flagey](https://github.com/NelopsisCode)
