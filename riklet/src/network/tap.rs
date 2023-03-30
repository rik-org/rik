use devices::virtio::Net as VirtioNet;
use rand::Rng;
use rate_limiter::RateLimiter;
use std::process::Command;
use tracing::trace;
use utils::net::mac::MacAddr;

use super::net::{NetworkInterfaceConfig, NetworkInterfaceError};

pub const MAX_IFACE_NAME_LEN: usize = 15;

/// Tries to create a new tap interface with the given name
/// Name should be unique and follow RFC, learn more here:
/// https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/8/html/configuring_and_managing_networking/consistent-network-interface-device-naming_configuring-and-managing-networking
pub fn open_tap(config: &NetworkInterfaceConfig) -> Result<VirtioNet, NetworkInterfaceError> {
    let mac_addr = generate_mac_addr();
    let (rx_rate_limiter, tx_rate_limiter) = create_rate_limiters();

    if config.iface_name.len() > MAX_IFACE_NAME_LEN {
        return Err(NetworkInterfaceError::InvalidInterfaceName);
    }

    VirtioNet::new(
        config.id.clone(),
        config.iface_name.clone().as_str(),
        Some(mac_addr),
        rx_rate_limiter,
        tx_rate_limiter,
    )
    .map_err(NetworkInterfaceError::CreateNetworkDevice)
}

/// FIXME: Find a better way to handle tap generation (using firecracker itself)
/// This function creates a tap using legacy commands, it's not secure but it fixes a current issue with TAPs
pub fn open_tap_shell(config: &NetworkInterfaceConfig) -> Result<String, NetworkInterfaceError> {
    let mac_addr = generate_mac_addr();

    if config.iface_name.len() > MAX_IFACE_NAME_LEN {
        return Err(NetworkInterfaceError::InvalidInterfaceName);
    }

    let tap_output = Command::new("ip")
        .args(["tuntap", "add", &config.iface_name, "mode", "tap"])
        .output()
        .map_err(|e| NetworkInterfaceError::ManageTap(e.to_string()))?;

    if !tap_output.status.success() {
        return Err(NetworkInterfaceError::ManageTap(format!(
            "Tap creation failed, code {}, stderr: {}",
            tap_output.status.code().unwrap(),
            String::from_utf8(tap_output.stderr).unwrap()
        )));
    }

    trace!("Shell tap create output: {:#?}", tap_output);
    return Ok(config.iface_name.clone());
}

pub fn close_tap_shell(iface_name: &str) -> Result<(), NetworkInterfaceError> {
    if iface_name.len() > MAX_IFACE_NAME_LEN {
        return Err(NetworkInterfaceError::InvalidInterfaceName);
    }

    let tap_output = Command::new("ip")
        .args(["tuntap", "del", iface_name, "mode", "tap"])
        .output()
        .map_err(|e| NetworkInterfaceError::ManageTap(e.to_string()))?;

    if !tap_output.status.success() {
        return Err(NetworkInterfaceError::ManageTap(format!(
            "Tap creation failed, code {}, stderr: {}",
            tap_output.status.code().unwrap(),
            String::from_utf8(tap_output.stderr).unwrap()
        )));
    }

    trace!("Shell tap delete output: {:#?}", tap_output);
    return Ok(());
}

/// Create a brand new MAC addr, it is fully random and might not be binded to a known
/// vendor.
pub fn generate_mac_addr() -> MacAddr {
    let mut rng = rand::thread_rng();
    let mut mac = [0u8; 6];
    rng.fill(&mut mac[..]);
    mac[0] &= 0xfe; /* clear multicast bit */
    mac[0] |= 0x02; /* set local assignment bit (IEEE802) */
    MacAddr::from_bytes_unchecked(&mac)
}

/// Generate two limiters that are used to determine the bandwidth of the tap interface
/// for both directions.
fn create_rate_limiters() -> (RateLimiter, RateLimiter) {
    let rx_rate_limiter = RateLimiter::default();
    let tx_rate_limiter = RateLimiter::default();
    (rx_rate_limiter, tx_rate_limiter)
}

#[cfg(test)]
mod tests {
    use crate::network::tap::NetworkInterfaceConfig;
    use futures_util::TryStreamExt;
    use pretty_assertions::assert_eq;
    use rtnetlink::new_connection;
    use std::net::Ipv4Addr;

    use super::*;

    fn get_tap_config(iface_name: &str) -> NetworkInterfaceConfig {
        NetworkInterfaceConfig::new(
            iface_name.to_string(),
            iface_name.to_string(),
            Ipv4Addr::new(127, 0, 0, 1),
        )
        .unwrap()
    }

    #[test]
    fn test_tap_shell_manage() {
        let tap_config = get_tap_config("tap007");
        open_tap_shell(&tap_config).unwrap();
        close_tap_shell(&tap_config.iface_name).unwrap();
    }

    #[test]
    fn test_tap_shell_duplicate() {
        let tap_config = get_tap_config("tap008");
        open_tap_shell(&tap_config).unwrap();
        let output = open_tap_shell(&tap_config);

        assert!(output.is_err());
        close_tap_shell(&tap_config.iface_name).unwrap();
    }

    #[test]
    fn create_tap_named() {
        let config = NetworkInterfaceConfig {
            iface_name: "rust0test".to_string(),
            id: "rust0test".to_string(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        let net = open_tap(&config).unwrap();
        assert!(net.iface_name().contains("rust0test"));
    }

    #[test]
    fn create_invalid_named_tap_fails() {
        let config = NetworkInterfaceConfig {
            iface_name: "rust0test-invalid-name-too-long".to_string(),
            id: "rust0test".to_string(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        let net_rs = open_tap(&config);
        assert!(net_rs.is_err());

        let net = net_rs.err().unwrap();
        assert_eq!(
            net.to_string(),
            "Interface name is invalid, expected to be less than 15 characters"
        );
    }

    #[test]
    fn create_tap_already_exist_fails() {
        let config = NetworkInterfaceConfig {
            iface_name: "rust1test".to_string(),
            id: "rust1test".to_string(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        let net_rs = open_tap(&config);
        assert!(net_rs.is_ok());

        let net_rs = open_tap(&config);

        let net = net_rs.err().unwrap();
        assert!(net
            .to_string()
            .contains("Invalid TUN/TAP Backend provided by rust1test."));
    }

    #[test]
    fn generate_mac_addr_test() {
        let mac = generate_mac_addr();
        assert_eq!(mac.to_string().len(), 17);
    }

    #[tokio::test]
    async fn drop_delete_tap() {
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);
        let config = NetworkInterfaceConfig {
            iface_name: "rust3test".to_string(),
            id: "rust3test".to_string(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        {
            let net = open_tap(&config).unwrap();
            assert!(net.iface_name().contains("rust3test"));
        }
        let mut links = handle
            .link()
            .get()
            .match_name("rust3test".to_string())
            .execute();
        let output_rs = links.try_next().await;
        assert_eq!(
            output_rs.err().unwrap().to_string(),
            "Received a netlink error message No such device (os error 19)"
        );
    }
}
