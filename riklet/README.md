# Riklet

[Rust in Kube (RIK)](https://github.com/dev-sys-do/rik) node agent

## Dependencies

To work, Riklet requires the following packages :

- [Runc](https://github.com/opencontainers/runc) : to run containers
- [Skopeo](https://github.com/containers/skopeo) : to pull containers images
  from multiples sources.
- [Umoci](https://github.com/opencontainers/umoci) : to unpack & modifies OCI
  images.

## Concept

The **riklet** is the primary node agent that runs on each node.

It is responsible to interpret scheduling instructions and run workloads.

Also, it is able to send metrics about node and pods regularly to the scheduler.

## Usage

### Général Usage

**To run the Riklet, you must have setup previously your RIK master node and get
it's IP address.**

```bash
riklet --master-ip <YOUR_IP>:<PORT>
```

To run riklet as root with **cargo**:

```bash
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E' cargo run --bin riklet
```

### Faas Usage

**Prerequisite**: You need firecracker in your PATH.

Here is a list of environment variables that can be used to configure run riklet:

| Environment Variable   | Description                                                                                                             | Default |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------- |
| `IFACE`                | Network interface connected to the internet                                                                             | ""      |
| `IFACE_IP`             | IP of the Network interface connected to the internet                                                                   | ""      |
| `FIRECRACKER_LOCATION` | Path to the firecracker binary                                                                                          | ""      |
| `KERNEL_LOCATION`      | Path to the kernel location                                                                                             | ""      |

To run riklet with FAAS configuration.

```bash
sudo riklet --kernel-path ${KERNEL_LOCATION} \
            --ifnet ${IFACE} \
            --ifnet-ip ${IFACE_IP} \
```

> The firecracker binary location is determined by the following order:
>
> - **$FIRECRACKER_LOCATION** environment variable: direct path to the binary
> - **$PATH** environment variable: search for the binary in the directories
> - firecracker binary in the current working directory

Exemple:

```bash
sudo riklet --kernel-path ./vmlinux.bin  \
            --ifnet wlp2s0  \
            --ifnet-ip 192.168.1.84
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

## Cleaning up

For now the Riklet doesn't clean its network configuration by itself.

| Environment Variable | Description                                | Default |
| -------------------- | ------------------------------------------ | ------- |
| `FUNCTION_NAME`      | the name of the function that is scheduled | ""      |

To clean the network configuration:

```bash
sudo iptables -D FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
sudo iptables -D FORWARD -i rik-${FUNCTION_NAME}-tap -o ${IFACE} -j ACCEPT
sudo iptables -t nat -D POSTROUTING -o ${IFACE} -j MASQUERADE
```

## Authors

- Thomas Gouveia - <thomas.gouveia@etu.umontpellier.fr>
- Hugo Amalric - <hugo.amalric01@etu.umontpellier.fr>
- Sylvain Renaud - <sylvain.reynaud@etu.umontpellier.fr>
