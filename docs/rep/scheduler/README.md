# Scheduler

## Main goal

The `scheduler` goal is to determine whether or not a workload can work on the cluster.
In the case a workload fits to a node, we send severals orders & informations to the node so
he can handle the needed workload. The controller can also determine in a first time, if a workload can be managed in the cluster, but
the scheduler has the **last word** on it. 

This service **must** be able at anytime, to request a delete or an update of a workload on a specific
node. These requests will be sent based on event the scheduler receive. The events are coming either from the controller or from a node.

The `scheduler` is a central point to the whole cluster communication. The service is able to filter
data coming from nodes and lift them up to the controller. By being a central point of communication
the `scheduler` can be considered as a [SPOF](https://en.wikipedia.org/wiki/Single_point_of_failure), 
considering that, this service must be able to recover from a crash **quickly** and **dynamically**.

**Features summary**:

* Allocate a workload to a node
* Request workload update 
* Request workload deletion
* Handle statistics & hardware informations from nodes
* Lift up informations from nodes to controller 
* Handle various events from nodes 

