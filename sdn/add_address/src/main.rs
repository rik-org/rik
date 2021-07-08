use futures::stream::TryStreamExt;
use std::env;

use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error, Handle};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let link_name = "lo";
    let ip = "1.1.1.1/32".parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });

    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    if let Err(e) = add_address(link_name, ip, handle.clone()).await {
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
