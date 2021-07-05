use futures::stream::TryStreamExt;
use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error};

#[tokio::main]
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

#[tokio::main]
async fn add_address(link_name: &str, ip: IpNetwork) -> Result<(), Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

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

fn main() -> Result<(), String> {
    //mock variables
    let link_name1 = "veth-rs-1";
    let link_name2 = "veth-rs-2";
    let ip: IpNetwork = "1.1.1.1".parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });

    let _ = create_veth(link_name1, link_name2);
    let _ = add_address(link_name1, ip);

    Ok(())
}
