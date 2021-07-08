# RIK Enhancement Proposals

# Controller

It allows to handle state of cluster by interracting with CLI and scheduler

The controller is the central management entity of the cluster.  
It provides a REST API that can be used to inspect and manipulate the cluster.  
It processes user requests through that API and maintain the cluster overall state.  
The controller keeps track of the cluster tenants, workloads and instances, and make sure current workloads and instances state are as much as possible equal to the desired state.  
After validating user requests, it interacts with the scheduler for orchestrating instances on the cluster via gRPC request.

![controler schema](https://res.cloudinary.com/malo-polese/image/upload/v1617964829/Polytech_DO/rik/controller_c3ood6.png)

## External API (with CLI)

REST API

Receive request to manage workloads, instances and tenants the controller has to check that the given rules are compliant and gives instructions to scheduler accordingly.

We think of going with Actix web framework.
Even though it might seem too big for our use case, it is as a big community and well documented so by using it we should avoid a lot of headache as it seem easy to use.


### Endpoints :

See endpoints definition [here](./openapi.yaml).


In comparison with K8S we bypass services and replicaset by using directly attributes in workload and instance endpoint to run as many replicas of workload as we want, as we think it's more user friendly.

We will add other kind as well as authentication/authorization later.  


The part we will be working mostly for the moment wil be workloads and instances.  
Tenants will come later and for now, we are not sure about the exact purpose of a tenants, are they a role with permisions (verb and resources) ?

## Internal API (with scheduler)

Use a gRPC (with protobuf) API has it is best suited for actions that imply mostly to only send commands to others party.

We may use [tonic](https://github.com/hyperium/tonic) to implement gRPC



Scheduler send information to the controller about instance and node status being run etc ...
When a node disconnect (or instance crash), the scheduler as to sends to the controller the information that instances on this node have failed and the controller as to decide what to do. (certainly recreate those same workload instances)

### Endpoints :

Defined in scheduler proto files.

## Database

The rik system is mostly stateless. The only stateful component of is the etcd database, which acts as the single source of truth for the entire cluster. The API server acts as the gateway to the etcd database through which both internal and external consumers access and manipulate the state.

It stores the configuration, the actual state of the system and the desired state of the system.  
As well as tenant informations like (IDs, quotas, current utilization), workload definition and ownership.

We choose to use ETCD because its seem best for this use case. Distributed database (and used by K8S).  
The controller can also use etcd’s watch functionality to monitor changes between actual vs desired state.
If they diverge, controllers send commands to the scheduler to reconcile the actual state and the desired state.

Maybe later we might want to add compatibility with other database like postgres, or SQLite.
So we want to be database agnostic and as @sameo pointed out we need to think about using an light abstraction between our controller and the database for long term use.



# Usage 

Rik-scheduler must be running before the controller if you want the controller to work properly.
To run the controller as a daemon on debian like:
```bash
cargo build --release
cargo deb
sudo dpkg -i target/debian/controller_0.1.0_amd64.deb
sudo systemctl daemon-reload
sudo systemctl start rik-controller
```

## Others

K8S architecture example we might want to follow
![K8S architecture example](assets/kubernetes-control-plane.png)

[@croumegous](https://github.com/croumegous) & [@MaloPolese](https://github.com/MaloPolese)
