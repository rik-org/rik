use std::net::Ipv4Addr;

use futures_util::TryStreamExt;
use rand::Rng;
use rtnetlink::new_connection;
use tracing::{trace, warn};
use utils::net::mac::MacAddr;

#[tracing::instrument()]
pub async fn set_link_up(iface_name: String) -> Result<(), rtnetlink::Error> {
    trace!("link {} up", &iface_name);
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let mut links = handle.link().get().match_name(iface_name.clone()).execute();
    if let Some(link) = links.try_next().await? {
        handle.link().set(link.header.index).up().execute().await?;

        return Ok(());
    }

    warn!("Could not get the interface {}", iface_name);
    return Err(rtnetlink::Error::RequestFailed);
}

#[tracing::instrument()]
/// For a given iface_name, tries to apply a ipv4/mask on it
pub async fn set_link_ipv4(
    iface_name: String,
    ipv4: Ipv4Addr,
    mask: u8,
) -> Result<(), rtnetlink::Error> {
    trace!("link {} ipv4: {}/{}", &iface_name, ipv4, mask);
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let mut links = handle.link().get().match_name(iface_name.clone()).execute();
    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(link.header.index, ipv4.into(), mask)
            .execute()
            .await?;

        return Ok(());
    }

    warn!("Could not get the interface {}", iface_name);
    return Err(rtnetlink::Error::RequestFailed);
}

/// Generate a new interface name with based on the id and a randomly generated number
///
/// Random format is expected to be the following: {id}-1234 where 1234 is a random number
/// Also, {id} is truncated to 10 characters
///
/// # Example
/// ```
/// use crate::network::NetworkInterfaceConfig;
/// use std::net::Ipv4Addr;
///
/// let config = netutils::new_tap_random_name("superlonginterfacename".to_string());
/// assert_eq!(config.iface_name, "1234-superlongi".to_string());
/// ```
pub fn new_tap_random_name(id: String) -> String {
    let random = rand::random::<u16>();
    let random = format!("{:04}", random);
    // Truncate to id to 9 characters, as we need to add the random number
    let id_shorten = if id.len() > 9 { &id[..9] } else { &id };
    // expected format:
    // IFACE = [a-zA-Z]{0,9}-[0-9]{4}
    format!("{}-{}", id_shorten, random)
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
