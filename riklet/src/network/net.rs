use std::net::{IpAddr, Ipv4Addr};

use crate::network::tap;
use devices::virtio::Net as VirtioNet;
use futures_util::TryStreamExt;
use rtnetlink::new_connection;

use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterfaceConfig {
    /// Name of the interface created
    pub iface_name: String,
    /// Unique identifier for the network interface, used to compare
    pub id: String,

    pub ipv4_addr: Ipv4Addr,
}

impl NetworkInterfaceConfig {
    pub fn new(id: String, iface_name: String, ipv4_addr: Ipv4Addr) -> Self {
        NetworkInterfaceConfig {
            iface_name,
            id,
            ipv4_addr,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkInterfaceError {
    // [devices::virtio::net::Error] already uses thiserror, it's not needed to overwrap our error
    #[error("{0}")]
    CreateNetworkDevice(#[from] devices::virtio::net::Error),
    #[error("Could not open connection to netlink: {0}")]
    IpSocket(#[from] std::io::Error),
    #[error("Could not allocate IP address to interface: {0}")]
    IpAllocation(#[from] rtnetlink::Error),
}

pub enum NetworkInterface {
    TapInterface(VirtioNet),
}

/// An instance of a network implementation, it can be either a tap interface or a just a veth
/// pair.
/// # Example
/// ```
/// use crate::network::Net;
/// use crate::network::NetworkInterfaceConfig;
/// use crate::network::NetworkInterfaceError;
///
/// let config = NetworkInterfaceConfig {
///     iface_name: "test".to_string(),
///     id: "test".to_string(),
///    ipv4_addr: Ipv4Addr::new(127, 0, 0, 1),
/// };
/// let net = Net::new_with_tap(config).unwrap();
/// ```
pub struct Net {
    /// Unique identifier for the network interface, used to compare
    pub id: String,
    /// Type of interface that has been added
    interface: NetworkInterface,
}

impl Drop for Net {
    fn drop(&mut self) {
        debug!("Drop net interface with id: {}", self.id);
    }
}

impl Net {
    /// Creates a new network interface with a tap interface, it will allocate an IP address to the
    /// interface depending on the input configuration
    pub async fn new_with_tap(
        config: NetworkInterfaceConfig,
    ) -> Result<Self, NetworkInterfaceError> {
        debug!("New net tap interface with name: {}", config.iface_name);
        let interface = NetworkInterface::TapInterface(tap::open_tap(&config)?);
        let net = Net {
            id: config.id.clone(),
            interface,
        };

        net.configure_ipv4_address(config.ipv4_addr, 24).await?;
        Ok(net)
    }

    /// Configures the IP address of the interface
    async fn configure_ipv4_address(
        &self,
        ipv4_addr: Ipv4Addr,
        prefix: u8,
    ) -> Result<(), NetworkInterfaceError> {
        debug!("Give IP address to netid: {} -> {}", ipv4_addr, self.id);
        let (connection, handle, _) = new_connection().map_err(NetworkInterfaceError::IpSocket)?;
        tokio::spawn(connection);

        let iface = match self.interface {
            NetworkInterface::TapInterface(ref iface) => iface.iface_name(),
        };

        let mut links = handle.link().get().match_name(iface.to_string()).execute();

        if let Some(link) = links
            .try_next()
            .await
            .map_err(NetworkInterfaceError::IpAllocation)?
        {
            handle
                .address()
                .add(link.header.index, IpAddr::V4(ipv4_addr), prefix)
                .execute()
                .await
                .map_err(NetworkInterfaceError::IpAllocation)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn create_net_tap_named() {
        let config = NetworkInterfaceConfig {
            iface_name: "rust0nettest".to_string(),
            id: "rust0nettest".to_string(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        let net = Net::new_with_tap(config).await.unwrap();
        match net.interface {
            NetworkInterface::TapInterface(ref iface) => {
                assert_eq!(iface.iface_name(), "rust0nettest")
            }
            _ => panic!("Wrong interface type"),
        }
    }

    #[tokio::test]
    async fn create_net_duplicate_tap_named() {
        let config = NetworkInterfaceConfig {
            iface_name: "rust1nettest".to_string(),
            id: "rust1nettest".to_string(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        // Must keep a name of variable to avoid drop
        let _net = Net::new_with_tap(config.clone()).await.unwrap();
        let net = Net::new_with_tap(config).await;
        assert!(net.is_err());
        assert!(net
            .err()
            .unwrap()
            .to_string()
            .contains("Invalid TUN/TAP Backend provided by rust1nettest"));
    }

    #[tokio::test]
    async fn allocate_ip_to_interface() {
        let iface_test = "rust3nettest".to_string();
        let config = NetworkInterfaceConfig {
            iface_name: iface_test.clone(),
            id: iface_test.clone(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        let net = Net::new_with_tap(config).await.unwrap();
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);
        let mut links = handle.link().get().match_name(iface_test.clone()).execute();
        let link = links.try_next().await.unwrap().unwrap();
        let mut addresses = handle
            .address()
            .get()
            .set_link_index_filter(link.header.index)
            .execute();

        let mut ips = vec![];
        while let Some(msg) = addresses.try_next().await.unwrap() {
            ips.push(msg.nlas[0].clone());
        }
        assert_eq!(ips.len(), 1);
        // We test against the debug representation of the IP address as AddressMessage is coming from a sub crate
        // of rtnetlink which is not accessible from the test
        let debug_ips = format!("{:?}", ips);
        assert_eq!(debug_ips, "[Address([172, 0, 0, 17])]")
    }
}
