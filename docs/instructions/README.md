# Scheduler

## Global architechtur

> Principale structur:
>
> - Worker Service
> - Controller Service
> - Manager
> - State Manager

## Services:

Both Controller and Worker services are implemented on the struc **GRPCService** in a grpc module.

**Worker Service**:  
This service implement two differents _rpc_:

- Register
- SendStatusUpdate

**Controller Service**:
this service implement three differents _rpc_:

- ScheduleInstance
- UnscheduleInstance
- GetStatusUpdates

**Manager**:
The Manager is the central point of scheduler. It's here that all event are handled. The manager is in control of all the diffenrent incomming event of our grpc server.

**Event**:
This _enum_ define all the différent envent that the manager can handle at the moment.
