use futures::stream::TryStreamExt;

use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error, Handle};

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Initialization of variables
    let link_name1 = "veth-rs-1";
    let link_name2 = "veth-rs-2";
    let ip: IpNetwork = "10.1.2.4".parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });

    create_veth(link_name1, link_name2);

    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    if let Err(e) = add_address(link_name1, ip, handle.clone()).await {
        eprintln!("{}", e);
    }
    Ok(())
}

async fn add_address(link_name: &str, ip: IpNetwork, handle: Handle) -> Result<(), Error> {
    let mut links = handle
        .link()
        .get()
        .set_name_filter(link_name.to_string())
        .execute();
    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(link.header.index, ip.ip(), ip.prefix())
            .execute()
            .await?
    }
    Ok(())
}

async fn create_veth(link_name1: &str, link_name2: &str) -> Result<(), String> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .add()
        .veth(link_name1.into(), link_name2.into())
        .execute()
        .await
        .map_err(|e| format!("{}", e))
}
