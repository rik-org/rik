<center>
<img src="https://i.imgur.com/wSdoLvt.png"/>
</center>

# Riklet architecture

> This document is only valid for RIK v1. It will be updated as development proceeds.

This document provides a technical reflexion about the rik node agent component, as known as the **riklet**.

This component should be run on every rik cluster node as a daemon in order to execute workloads on it.

Below a schema describing the actual architecture of the component. We will go into technical details throughout this document.

![Component architecture](https://i.imgur.com/RrJkfxe.png)

## Glossary

- `Node`: A cluster unit that executes a workload
- `OCI` : An open governance structure for the express purpose of creating open industry standards around container formats and runtimes.
- `gRPC` : An implementation of the Remote Procedure Call by Google.
- `Workload` : Object that contains its definition, itself composed of the containers specifications

## Riklet

It is responsible to interpret scheduling instructions and run workloads on the node on which it runs. It should be capable to get metrics about the resources usage of the node (CPU, RAM, Disk usage). 

At the moment, riklet only supports **containers** workloads.

In order to communicate with the scheduler component, riklet runs as a [gRPC](https://grpc.io) client.

API definitions will be defined through [protobuf](https://developers.google.com/protocol-buffers) files that allow us to define the API easily, and generate the associated code.


### Container runtime

Under the hood, riklet take advantages of [Runc](https://github.com/opencontainers/runc) in order to manage containers.

**Runc** is a command line interface tool allowing to run & manage containers according to the [OCI](https://opencontainers.org/) (Open Containers Initiative) specification.

By providing a layer of abstraction to interact with Runc, the riklet container runtime exposes Rust structs & methods to interact with the binary.


### OCI Manager

In order to run containers, we need to have a valid OCI bundle. To make easier the process of the bundle creation, we make use of two binaries : 

- [Skopeo](https://github.com/containers/skopeo) to interact with image repositories
- [Umoci](https://github.com/opencontainers/umoci) to create an OCI bundle from an image

Like Runc, riklet provides a layer of abstraction to interact with these binary with rust functions & structs.


### Metrics

Riklet is responsible to fetch metrics of the host node and the workload that runs on the latter.

For that, it should implement a push model in order to report metrics. When a change is detected, the **riklet** will fetch and send metrics to the scheduler which is reponsible to keep the node metrics and forward the pod metrics to the controller.

#### Node metrics

Node metrics payload should contain at least **CPU**, **Memory** and **Disk** usage. Without these informations, the scheduler will not be able to correctly schedule workloads.

```json
{
   "cpu": {
      "total": 8, //number of cores
      "free": 17.445435 //usage percentage
   },
   "memory": {
      "total": 34054195200, //bytes
      "free": 29200210944 //bytes
   },
   "disks": [
      {
         "disk_name": "/dev/nvme0n1p3",
         "total": 496896393216, //bytes
         "free": 247193731072 //bytes
      },
      {
         "disk_name": "/dev/nvme0n1p1",
         "total": 824180736, //bytes
         "free": 765984768 //bytes
      }
   ]
}
```

### Lifecycle

The lifecycle of a workload is quite simple and it looks like a simple process on a computer. A pod can be in different states at a specific moment : 

- `Creating`: The workload is in creation. 
- `Running` : The workload is running and is healthy.
- `Failed` : The workload has crashed.
- `Terminated` : The workload is terminated and it'll be deleted.

**Riklet has to manage each of these states, and on every state update, it should inform the scheduler of what happening on the workload.**

Also, it can be in `Pending` state but this is not the riklet responsibility to manage this state. This state means that there is no available node where the workload can be scheduled and it's the scheduler responsibility to take care of it.
