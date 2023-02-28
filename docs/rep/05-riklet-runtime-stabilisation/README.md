# Stability of the Riklet

## Status

Approved

## Context

Riklet needs to be more modular and easier to maintain. Not only we want to 
be able to work on multiple parts of the software at the same time, but we
also want to be able to add new features without having to change the
entire codebase. Here are the current issues diagnosed:

- Container and VM spawns are handled in the same function separated by a `if` 
  statement
- Error handling is done through `Box<dyn Error>` which make it nearly 
  impossible to handle errors
- Any error that is happening in the code is not handled, and won't be reported
- Riklet is not verbose enough to be able to debug issues
- TAP creation and IP allocation is done through a bash script which is not 
  portable and prone to failures

## Decision

Splitting the codebase should be done through several steps, for some of 
them, they can be handled in parallel:

- Definition of a common trait between `containers` and `VM` to be able to 
  handle them in the same way
- Error handling should be done with a custom error type [^3]
- TAP handling should be done through usage of a library[^1]
- Migrating logs to library `tokio::tracing`
- Network IP allocation should be done through a library[^2]
- Definition of a singleton to handle network configuration

## Consequences

- Riklet will be more modular and easier to maintain
  - Extension of the software for Software Designed Network (SDN) will be easier
- Riklet will be more verbose and easier to debug
- Riklet will be more portable and less prone to failures
  - Error handling will be more explicit and then easier to debug
- Riklet becomes dependant to libraries, and more maintenance will need to 
  be made in order to keep versions up to date
- Makes easier to add tests to the codebase

[^1]: Library [virtio/net](https://github.com/firecracker-microvm/firecracker/tree/main/src/devices/src/virtio/net)
      from firecracker can be used as a base to create TAP devices
[^2]: Library [rtnetlink](https://github.com/rust-netlink/rtnetlink) can be used
[^3]: Crates like `thiserror` can also help to make error more explicit
