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

In order to communicate with the scheduler and potentially other rik components, **riklet** had to expose an API.

This API will use the [gRPC](https://grpc.io) protocol to send & receive requests. API definitions will be defined through [protobuf](https://developers.google.com/protocol-buffers) files that allow us to define the API easily, and generate the associated code.

### Metrics

Riklet is responsible to fetch metrics of the host node and the pods that runs on the latter.

For that, it should implement a push model in order to report metrics. That means that every `x` seconds for example, the **riklet** will fetch and send metrics to the scheduler which is reponsible to keep the node metrics and forward the pod metrics to the controller.

Metrics resolution interval has to be configurable with a flag, for example :
`riklet --metric-resolution=5s`

#### Node metrics

Node metrics payload should contain at least **CPU**, **Memory** and **Disk** usage. Without these informations, the scheduler will not be able to correctly schedule workloads.

```json
{
  "cpu": {
    "totalCpu": 12, // number of cores
    "currentUsage": 20 // Percentage
  },
  "memory": {
    "totalMemory": 34359738368, // Bytes
    "freeMemory": 17179869184, // Bytes
    "freeMemoryAmount": 50 //Percentage
  },
  "disks": [
      {
          "diskName": "/dev/sda1"
          "total": 356448698, // Bytes
          "used": 12564, // Bytes
          "available": 3564428698 // Bytes
      }
  ]
}
```

#### Pod metrics

Pod metrics payload should contain at least **CPU** and **Memory** usage. Without these informations, the scheduler will not be able to correctly schedule workloads.

```json
{
	"cpu": {
		"totalCpu": 12, // number of cores
		"currentUsage": 20 // Percentage
	},
	"memory": {
		"totalMemory": 34359738368, // Bytes
		"freeMemory": 17179869184, // Bytes
		"freeMemoryAmount": 50 //Percentage
	}
}
```

## Pod

### Lifecycle

The lifecycle of a pod is quite simple and it looks like a simple process on a computer. A pod can be in different states at a specific moment :

- `Creating`: The pod is in creation.
- `Running` : The pod is running and is healthy.
- `Error` : The pod has crashed.
- `Terminating` : The pod is terminated and it'll be deleted.

**Riklet has to manage each of these states, and on every state update, it should inform the scheduler of what happening on the pod.**

Also, it can be in `Pending` state but this is not the riklet responsibility to manage this state. This state means that there is no available node where the pod can be scheduled and it's the scheduler responsibility to take care of it.
