use std::net::{IpAddr, Ipv4Addr};

use crate::network::tap::{self, open_tap_shell};
use futures_util::TryStreamExt;
use rtnetlink::new_connection;

use tracing::{debug, error, trace};

type Result<T> = std::result::Result<T, NetworkInterfaceError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterfaceConfig {
    /// Name of the interface created
    pub iface_name: String,
    /// Unique identifier for the network interface, used to compare
    pub id: String,

    pub ipv4_addr: Ipv4Addr,
}

impl NetworkInterfaceConfig {
    pub fn new(
        id: String,
        iface_name: String,
        ipv4_addr: Ipv4Addr,
    ) -> Result<NetworkInterfaceConfig> {
        if iface_name.len() > tap::MAX_IFACE_NAME_LEN {
            return Err(NetworkInterfaceError::InvalidInterfaceName);
        }
        Ok(NetworkInterfaceConfig {
            iface_name,
            id,
            ipv4_addr,
        })
    }

    /// Creates a new network interface with a random name
    ///
    /// Random format is expected to be the following: {id}-1234 where 1234 is a random number
    /// Also, {id} is truncated to 10 characters
    ///
    /// # Example
    /// ```
    /// use crate::network::NetworkInterfaceConfig;
    /// use std::net::Ipv4Addr;
    ///
    /// let config = NetworkInterfaceConfig::new_with_random_name("superlonginterfacename".to_string(), Ipv4Addr::new(127, 0, 0, 1));
    /// assert_eq!(config.iface_name, "1234-superlongi".to_string());
    /// ```
    pub fn new_with_random_name(id: String, ipv4_addr: Ipv4Addr) -> Result<NetworkInterfaceConfig> {
        // Generate 4 random digits
        let random = rand::random::<u16>();
        let random = format!("{:04}", random);
        let id_shorten = if id.len() > 9 { &id[..9] } else { &id };
        let iface_name = format!("{}-{}", id_shorten, random);
        Ok(NetworkInterfaceConfig {
            iface_name,
            id,
            ipv4_addr,
        })
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
    #[error("Interface name is invalid, expected to be less than 15 characters")]
    InvalidInterfaceName,
    #[error("Failed to create TAP: {0}")]
    ManageTap(String),
}

pub enum NetworkInterface {
    TapInterface(String),
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

impl Net {
    /// Creates a new network interface with a tap interface, it will allocate an IP address to the
    /// interface depending on the input configuration
    pub async fn new_with_tap(config: NetworkInterfaceConfig) -> Result<Self> {
        debug!("New net tap interface with name: {}", config.iface_name);
        let tap = open_tap_shell(&config)?;
        let interface = NetworkInterface::TapInterface(tap);
        let net = Net {
            id: config.id.clone(),
            interface,
        };

        net.configure_ipv4_address(config.ipv4_addr, 24).await?;
        Ok(net)
    }

    /// Configures the IP address of the interface
    async fn configure_ipv4_address(&self, ipv4_addr: Ipv4Addr, prefix: u8) -> Result<()> {
        debug!("Give IP address to netid: {} -> {}", ipv4_addr, self.id);
        let (connection, handle, _) = new_connection().map_err(NetworkInterfaceError::IpSocket)?;
        tokio::spawn(connection);

        let mut links = handle.link().get().match_name(self.iface_name()).execute();

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

    pub async fn set_link_up(&self) -> Result<()> {
        let iface = self.iface_name();
        trace!("link {} up", &iface);
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);

        let mut links = handle.link().get().match_name(iface.clone()).execute();
        if let Some(link) = links.try_next().await? {
            handle
                .link()
                .set(link.header.index)
                .up()
                .execute()
                .await
                .map_err(NetworkInterfaceError::IpAllocation)?;

            return Ok(());
        }

        return Err(NetworkInterfaceError::ManageTap(format!(
            "Could not get the interface {}",
            iface
        )));
    }

    pub fn iface_name(&self) -> String {
        match self.interface {
            NetworkInterface::TapInterface(ref iface) => iface.clone(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use netlink_packet_route::link::nlas::Nla;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn create_net_tap_named() {
        let config = NetworkInterfaceConfig {
            iface_name: "rust0nettest".to_string(),
            id: "rust0nettest".to_string(),
            ipv4_addr: Ipv4Addr::new(172, 0, 0, 17),
        };
        let net = Net::new_with_tap(config).await.unwrap();
        assert_eq!(net.iface_name(), "rust0nettest");
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
        let error = net.err().unwrap().to_string();
        let expected_error = "Failed to create TAP: Tap creation failed, code 1, stderr: ioctl(TUNSETIFF): Device or resource busy\n";
        assert_eq!(error, expected_error);
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

    #[test]
    fn new_with_random_name_long() {
        let config = NetworkInterfaceConfig::new_with_random_name(
            "rust5nettest".to_string(),
            Ipv4Addr::new(172, 0, 0, 17),
        )
        .unwrap();
        assert!(config.iface_name.starts_with("rust5nett-"));
    }

    #[tokio::test]
    async fn new_with_random_name_long_tap() {
        let config = NetworkInterfaceConfig::new_with_random_name(
            "mysuperlongname".to_string(),
            Ipv4Addr::new(172, 0, 0, 17),
        )
        .unwrap();
        let _net = Net::new_with_tap(config).await.unwrap();
        let (connection, handle, _) = new_connection().unwrap();
        tokio::spawn(connection);

        let mut exist = false;
        let mut links = handle.link().get().execute();
        'outer: while let Some(msg) = links.try_next().await.unwrap() {
            for nla in msg.nlas.into_iter() {
                if let Nla::IfName(name) = nla {
                    println!("found link {} ({})", msg.header.index, name);
                    if name.starts_with("mysuper") {
                        exist = true;
                    }
                    continue 'outer;
                }
            }
        }

        assert!(exist);
    }

    #[test]
    fn new_with_random_name_short() {
        let config = NetworkInterfaceConfig::new_with_random_name(
            "short".to_string(),
            Ipv4Addr::new(172, 0, 0, 17),
        )
        .unwrap();
        assert!(config.iface_name.starts_with("short-"));
    }
}
