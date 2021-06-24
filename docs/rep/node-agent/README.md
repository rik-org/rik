# RIK - Riklet Architecture

This document provides a technical reflexion about our component, the **riklet**. Our component and the [rik-proxy]("https://example.com") will be installed on each worker node of a rik cluster. We will describe this component in more details throughout this document.

Below a diagram to illustrate how these components work together :

![Global architecture](https://i.imgur.com/ZwLWFYV.png)

The **riklet** is the primary _node agent_ that runs on each node.

## Glossary

- `Pod`: Execution unit in Rik. It should be able to execute docker containers for the first release.
- `Node`: A cluster unit that executes a workload and contains only one `pod` for the first release.

## Riklet

### Objectives

Riklet is responsible to interpret scheduling instructions and run workloads in a dedicated pod.

Also, it should be able to send metrics about node and pods regularly to the scheduler.

### Communication

The **riklet** will connect to th exposed [gRPC](https://grpc.io) API of the scheduler.

See the scheduler API definitions that are defined through [protobuf](https://developers.google.com/protocol-buffers) and that can be found [here](https://github.com/AlexandreBrg/rik/tree/main/proto).

### Metrics

Riklet is responsible to fetch metrics of the host node and the pods that runs on the latter.

For that, it should implement a push model in order to report metrics. When a change is detected, the **riklet** will fetch and send metrics to the scheduler which is reponsible to keep the node metrics and forward the pod metrics to the controller.

Metrics resolution interval has to be configurable with a flag, for example :
`riklet --metric-resolution=5s`

#### Node metrics

Node metrics payload should contain at least **CPU**, **Memory** and **Disk** usage. Without these informations, the scheduler will not be able to correctly schedule workloads.

```json
{
	"cpu": {
		"total": 12, // number of cores
		"free": 20 // Percentage
	},
	"memory": {
		"total": 1234, // Bytes
		"free": 123 // Bytes
	},
	"disks": [
		{
			"diskName": "/dev/sda1",
			"total": 1234, // Bytes
			"free": 123 // Bytes
		}
	]
}
```

#### Pod metrics

Pod metrics payload should contain at least **CPU** and **Memory** usage. Without these informations, the scheduler will not be able to correctly schedule workloads.

```json
{
	"cpu": {
		"total": 12, // number of cores
		"free": 20 // Percentage
	},
	"memory": {
		"total": 1234, // Bytes
		"free": 123 // Bytes
	}
}
```

## Pod

### Lifecycle

The lifecycle of a pod is quite simple and it looks like a simple process on a computer. A pod can be in different states at a specific moment :

- `Creating`: The pod is in creation.
- `Running` : The pod is running and is healthy.
- `Error` : The pod has crashed.
- `Terminated` : The pod is terminated and it'll be deleted.

**Riklet has to manage each of these states, and on every state update, it should inform the scheduler of what happening on the pod.**

Also, it can be in `Pending` state but this is not the riklet responsibility to manage this state. This state means that there is no available node where the pod can be scheduled and it's the scheduler responsibility to take care of it.
