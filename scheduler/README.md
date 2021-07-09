# Scheduler 

Scheduler is one of the main component which aims to provide workers workloads they need to run. 

## Design

The purpose of the scheduler is to give decisions about scheduling requests made by the controller. This component
is aware of everything that is running in the cluster. The current version doesn't allow extensions and filtering.

Two endpoints are exposed by the scheduler, the first one is to let workers register to the scheduler and listen
to scheduling requests, by default it is exposed as `4995`. The second one is to let your controller listen to
metrics and to receive scheduling requests, it is exposed on `4996`. You can see more about the APIs exposed in
[protolib](../proto). A basic command line interface is available to customize endpoints options.

## Usage

```
USAGE:
    rik-scheduler [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity, info is the default
    -V, --version    Prints version information

OPTIONS:
    -c, --ctrlip <CONTROLLERS_IP>    Controllers endpoint IPv4 [default: 0.0.0.0:4996]
    -w, --workersip <WORKERS_IP>     Workers endpoint IPv4 [default: 0.0.0.0:4995]
```

## Logging

This component is using [`env_logger`](https://docs.rs/env_logger/0.8.4/env_logger/)
to log information to `stdout`. You can define the logging level either by command line options or
by setting `RUST_LOG`. Levels available are: `debug`, `trace`, `info`, `warn`, `error`.

**Note**: `Debug` mode will output gRPC logs, which can make your logs too much verbose. You can scope these logs
by setting `RUST_LOG=rik-scheduler=debug`.