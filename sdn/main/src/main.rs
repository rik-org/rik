use clap::Clap;
use clap::{App, Arg};
use futures::stream::TryStreamExt;
use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error};

#[tokio::main]
async fn create_veth(link_name1: String, link_name2: String) -> Result<(), Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .add()
        .veth(link_name1, link_name2)
        .execute()
        .await?;
    Ok(())
}

#[tokio::main]
async fn add_address(link_name: String, ip: IpNetwork) -> Result<(), Error> {
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
#[clap(name = "Network NameSpace Configurator v0.3")]
struct Args {
    /// Path of the netns to use
    #[clap(short, long)]
    netns_path: String,

    /// Name of the link created and set on the host
    #[clap(short, long, default_value = "host-link")]
    host_link_name: String,

    /// Name of the link created and set on the container
    #[clap(short, long, default_value = "container-link")]
    container_link_name: String,

    /// Globally unique identifier of the container
    #[clap(short, long)]
    guid: String,
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

    match create_veth(link_name1.clone(), link_name2) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error create_veth: {}", error.to_string());
            std::process::exit(1);
        }
    };
    match add_address(link_name1, ip) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error add_address: {}", error.to_string());
            std::process::exit(1);
        }
    };

    Ok(())
}
