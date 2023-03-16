use devices::virtio::Net as VirtioNet;
use rand::Rng;
use rate_limiter::RateLimiter;
use utils::net::mac::MacAddr;

use super::net::{NetworkInterfaceConfig, NetworkInterfaceError};

/// Tries to create a new tap interface with the given name
/// Name should be unique and follow RFC, learn more here:
/// https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/8/html/configuring_and_managing_networking/consistent-network-interface-device-naming_configuring-and-managing-networking
pub fn open_tap(config: &NetworkInterfaceConfig) -> Result<VirtioNet, NetworkInterfaceError> {
    let mac_addr = generate_mac_addr();
    let (rx_rate_limiter, tx_rate_limiter) = create_rate_limiters();
    VirtioNet::new(
        config.id.clone(),
        config.iface_name.clone().as_str(),
        Some(mac_addr),
        rx_rate_limiter,
        tx_rate_limiter,
    )
    .map_err(NetworkInterfaceError::CreateNetworkDevice)
}

/// Create a brand new MAC addr, it is fully random and might not be binded to a known
/// vendor.
fn generate_mac_addr() -> MacAddr {
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
    use futures_util::TryStreamExt;
    use pretty_assertions::assert_eq;
    use rtnetlink::new_connection;
    use std::net::Ipv4Addr;

    use super::*;

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
            "Open tap device failed: Invalid interface name"
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
