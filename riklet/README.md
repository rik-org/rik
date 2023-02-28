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

**Prerequisite**: You need firecracker in yout PATH.

To run riklet with FAAS configuration.

```bash
sudo riklet --firecracker-path <FIRECRACKER_LOCATION> --kernel-path <KERNEL_LOCATION> --ifnet <IFACE> --ifnet-ip <IFACE_IP> --script-path <SCRIPT_LOCATION>
```

Exemple:

```bash
sudo riklet --firecracker-path $(which firecracker) --kernel-path ./vmlinux.bin --ifnet wlp2s0 --ifnet-ip 192.168.1.84 --script-path ./scripts/setup-host-tap.sh
```

`IFACE`: Network interface connected to the internet.

`IFACE_IP`: IP of the Network interface connected to the internet.

`FIRECRACKER_LOCATION`: Path to the firecracker binary.

`KERNEL_LOCATION`: Path to the kernel location.

`SCRIPT_LOCATION`: Path to the
[script](https://github.com/polyxia-org/rik/blob/main/scripts/setup-host-tap.sh)
that create tap interface.

You can set this configuration as environement variable.

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

For now the Riklet don't clean his network configuration by his self.

To clean the network configuration:

```bash
sudo iptables -D FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
sudo iptables -D FORWARD -i rik-<FUNCTION_NAME>-tap -o <IFACE> -j ACCEPT
sudo iptables -t nat -D POSTROUTING -o <IFACE> -j MASQUERADE
```

`FUNCTION_NAME`: the name of the function that is scheduled.

## Authors

- Thomas Gouveia - <thomas.gouveia@etu.umontpellier.fr>
- Hugo Amalric - <hugo.amalric01@etu.umontpellier.fr>
- Sylvain Renaud - <sylvain.reynaud@etu.umontpellier.fr>
