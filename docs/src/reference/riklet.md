# Riklet

[Rust in Kube (RIK)](https://github.com/dev-sys-do/rik) node agent

## Faas Configuration

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
            --ifnet-ip 192.168.1.84  \
            --script-path ./scripts/setup-host-tap.sh
```
