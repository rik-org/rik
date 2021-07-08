# Riklet 

[Rust in Kube (RIK)](https://github.com/dev-sys-do/rik) node agent

## Dependencies

To work, Riklet requires the following packages : 

- [Runc](https://github.com/opencontainers/runc) : to run containers
- [Skopeo](https://github.com/containers/skopeo) : to pull containers images from multiples sources.
- [Umoci](https://github.com/opencontainers/umoci) : to unpack & modifies OCI images.

## Concept

The **riklet** is the primary node agent that runs on each node.

It is responsible to interpret scheduling instructions and run workloads.

Also, it is able to send metrics about node and pods regularly to the scheduler.

## Usage

**To run the Riklet, you must have setup previously your RIK master node and get it's IP address.**

```bash
riklet --master-ip <YOUR_IP>:<PORT>
```

You should see something like that : 

```

        ______ _____ _   __ _      _____ _____
        | ___ \_   _| | / /| |    |  ___|_   _|
        | |_/ / | | | |/ / | |    | |__   | |
        |    /  | | |    \ | |    |  __|  | |
        | |\ \ _| |_| |\  \| |____| |___  | |
        \_| \_|\___/\_| \_/\_____/\____/  \_/
        
[2021-07-03T15:04:09Z INFO  riklet::core] Riklet (v0.1.0) is ready to accept connections.
```

## Authors

- Thomas Gouveia - <thomas.gouveia@etu.umontpellier.fr>
- Hugo Amalric - <hugo.amalric01@etu.umontpellier.fr>
- Sylvain Renaud - <sylvain.reynaud@etu.umontpellier.fr>
