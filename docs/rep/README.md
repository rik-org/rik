
# RIK Enhancement Proposals

  
  
  
  

# Controller

It allows to handle state of cluster by interracting with CLI and scheduler

  
![controler schema](https://res.cloudinary.com/malo-polese/image/upload/v1617964829/Polytech_DO/rik/controller_c3ood6.png)
### External API (with CLI)

REST API

Can receive YAML file for cluster, deployment, pods, configurations the controller has to check that the given rules are compliant and gives instructions to scheduler accordingly.

We may use [Rocket](https://rocket.rs/) as a web server framework.

#### Endpoints :



TBD

  

### Internal API (with scheduler)

Use a RPC (gRPC) API has it is best suited for actions that imply mostly to only send commands to others party.

We may use [tonic](https://github.com/hyperium/tonic)  to implement gRPC
  

#### Endpoints :

  

TBD

  
  

### Database

The rik system is mostly stateless. The only stateful component of is the etcd database, which acts as the single source of truth for the entire cluster. The API server acts as the gateway to the etcd database through which both internal and external consumers access and manipulate the state.


It stores the configuration, the actual state of the system and the desired state of the system.  
  
We choose to use ETCD because its seem best for this use case. Distributed database (and used by K8S).  
The controller can also uses etcd’s watch functionality to monitor changes between actual vs desired state.
If they diverge, controllers send commands to the scheduler to reconcile the actual state and the desired state.

  
Maybe later we might want to add compatibility with other database like postgres.

  


[@croumegous](https://github.com/croumegous) & [@MaloPolese](https://github.com/MaloPolese)
