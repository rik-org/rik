# Riklet SDN

This component onboard a network component which will manage network exposure
and routing. Depending on the workload, it will be configured to use a specific
network implementation. For now, only `Function` workload have an implementation
of network configuration. This implementation is based on `iptables` and
[`rtnetlink`](https://man7.org/linux/man-pages/man7/rtnetlink.7.html).

## Function network implementation

This network feature allows you to forward traffic from a specific port to a
Function instance port. We achieve this using `iptables`, a widely used linux
tool for managing network traffic. The translation of IP and port is targetting
a [TAP](https://www.gabriel.urdhr.fr/2021/05/08/tuntap/) interface on the
machine that is communicating with the Function instance (microVM).

```ignore

┌──────────────────────────────────────────────────────────────────┐
│                      Host Machine (riklet)                       │
│                                                                  │
│                                                                  │
│  ┌─────────────────────────────┐   ┌─────────────────────────┐   │
│  │Iptables                     │   │    Function Instance    │   │
│  │                             │   │                         │   │
│  │ ┌─────────────────────────┐ │   │                         │   │
│  │ │APPLY NAT ON             │ │   │┌───────────────────────┐│   │
│  │ │host:${port}             │ │   ││      Guest_veth       ││   │
│  │ │                         │ │   ││                       ││   │
│  │ │TO                       │─┼┐  │└───────────────────────┘│   │
│  │ │host_tap:${service_port} │ ││  │            ▲            │   │
│  │ │                         │ ││  └────────────┼────────────┘   │
│  │ └─────────────────────────┘ ││               │                │
│  │              ▲              ││               │                │
│  └──────────────┼──────────────┘│   ┌───────────────────────┐    │
│                 │               │   │       Host_tap        │    │
│                 │               └──▶│                       │    │
│                 │                   └───────────────────────┘    │
│     ┌───────────────────────┐                                    │
│     │Host Ethernet Interface│                                    │
│     │                       │                                    │
│     └───────────────────────┘                                    │
│                 ▲                                                │
└─────────────────┼────────────────────────────────────────────────┘
                  │
```

This is what the network configuration looks like when you deploy a Function
instance with a port mapping, please not it is very specific to Function. The
`host_tap` interface is created by the
`riklet` and is used to communicate with the Function instance. The `Guest_veth`
interface is created by the `firecracker` microVM and is used to communicate
with the `host_tap` interface. The `host_tap` is connecteed to the internet and
is not restricted in bandwidth.

## Iptables

Riklet will use a custom chain called `RIKLET` on the table nat to do DNAT (Destination NAT), it
matches two use cases:

- Local processes: when another workload wants to communicate with a Function
  instance
- Internet: when the workload needs to be exposed externally on the worker node


```ignore
    .─────────────────.               .─────────────────.
 ,─'                   '─.         ,─'                   '─.
(     Local processes     )       (        Internet         )
 `──.                 _.─'         `──.                 _.─'
     `───────────────'                 `───────────────'
             │                                 │
             │                                 │
             │                                 │
             ▼                                 ▼
  ┌────────────────────┐            ┌────────────────────┐
  │    OUTPUT (nat)    │            │  PREROUTING (nat)  │
  └────────────────────┘            └────────────────────┘
             │                                 │
             │                                 │
             │      ┌──────────────────────────┤
             │      │                          │
             ▼      ▼                          ▼
  ┌────────────────────┐            ┌────────────────────┐
  │    RIKLET (nat)    │───────────▶│      FORWARD       │
  └────────────────────┘            └────────────────────┘
                                                │
                                                │
                                                │
                                                │
                                                ▼
                                     ┌────────────────────┐
                                     │    POSTROUTING     │
                                     └────────────────────┘
                                                │
                                                │
                                                │
                                                ▼
```