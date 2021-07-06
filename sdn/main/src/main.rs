use clap::{App, Arg};
use clap::Clap;
use futures::stream::TryStreamExt;
use ipnetwork::IpNetwork;
use rtnetlink::{Error, new_connection};

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

#[derive(Clap, Debug)]
#[clap(name = "netns parse")]
struct Args {

    /// Path of the netns to use
    #[clap(short, long)]
    netns_path: String,

    ///
    #[clap(short, long, default_value = "host-link")]
    host_link_name: String,

    ///
    #[clap(short, long, default_value = "container-link")]
    container_link_name: String,

}

fn main() -> Result<(), String> {
    let args = Args::parse();

    println!("The netns path is {}!", args.netns_path);
    let namespace_name: String = args.netns_path.split("/").collect();

//mock variables


    let link_name1 = args.container_link_name;
    let link_name2 = args.host_link_name;
    let ip = "10.1.1.1".parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });

    let _ = create_veth(&link_name1, &link_name2);
    let _ = add_address(&link_name1, ip);

    Ok(())
}
