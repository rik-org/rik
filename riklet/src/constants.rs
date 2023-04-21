pub const DEFAULT_COMMAND_TIMEOUT: u64 = 30000;

/// A path to a directory which will contain the firecracker VMs
pub const DEFAULT_FIRECRACKER_WORKSPACE: &str = "/var/lib/riklet/vm";

/// IPv4 adresse mask that is used to configure IP address for the guest VM and host interface
pub const DEFAULT_FIRECRACKER_NETWORK_MASK: u8 = 30;
