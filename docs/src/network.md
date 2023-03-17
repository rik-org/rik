# Network

This project has network features, it's state is **unstable** and should be used
with caution as it is managing your network interfaces and routing. We are doing
our best not to break your system's network!

*In this document words Network and SDN (Software Designed Network) have the
same meaning*.

## Workloads

Current workloads cannot be configured with network implementation, however
`Function` workload implement a first version of network configuration which
can't be configured yet.

## Riklet

This component onboard a network component which will manage network exposure
and routing. Depending on the workload, it will be configured to use a specific
network implementation. For now, only `Function` workload have an implementation
of network configuration. This implementation is based on `iptables` and
[`rtnetlink`](https://man7.org/linux/man-pages/man7/rtnetlink.7.html).