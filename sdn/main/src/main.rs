use clap::Clap;
use futures::stream::TryStreamExt;
use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error, NetworkNamespace};
use std::process::Command;

#[tokio::main]
async fn create_netns(name: String) -> Result<(), Error> {
    NetworkNamespace::add(name.to_string()).await?;
    Ok(())
}

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

fn add_address_in_netns(link_name: String, netns: String, ip: IpNetwork) -> Result<(), Error> {
    Command::new("sh")
        .arg("-c")
        .arg(
            "ip netns exec ".to_string()
                + netns.as_str()
                + " ip address add "
                + ip.to_string().as_str()
                + " dev "
                + link_name.as_str(),
        )
        .output()
        .expect("failed to execute process");
    Ok(())
}

#[derive(Clap, Debug)]
#[clap(name = "Network NameSpace Configurator v0.3")]
struct Args {
    /// Path of the netns to use
    //#[clap(short, long)]
    //path_netns: String,

    /// Globally unique identifier of the container
    #[clap(short, long)]
    namespace: String,
}

fn move_veth_to_netns(veth: String, netns: String) -> Result<(), Error> {
    //println!("ip link set {} netns {}", veth, netns);
    Command::new("sh")
        .arg("-c")
        .arg("ip link set ".to_string() + veth.as_str() + " netns " + netns.as_str())
        .output()
        .expect("failed to execute process");
    Ok(())
}

#[tokio::main]
async fn set_up_veth(link_name: String) -> Result<(), Error> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    let mut links = handle
        .link()
        .get()
        .set_name_filter(link_name.clone())
        .execute();
    if let Some(link) = links.try_next().await? {
        handle.link().set(link.header.index).up().execute().await?
    } else {
        println!("no link link {} found", link_name);
    }
    Ok(())
}

#[tokio::main]
async fn set_up_veth_in_netns(link_name: String, netns: String) -> Result<(), Error> {
    Command::new("sh")
        .arg("-c")
        .arg(
            "ip netns exec ".to_string() + netns.as_str() + " ip link set up " + link_name.as_str(),
        )
        .output()
        .expect("failed to execute process");
    Ok(())
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    //println!("The netns path is {}!", args.netns_path);

    // Cause -> veth can't have more than 15 characters
    if args.namespace.len() > 12 {
        eprintln!("name must have less than 13 characters");
        std::process::exit(2);
    }
    let name_host = args.namespace.clone() + "-h";
    let name_container = args.namespace.clone() + "-c";
    // Must be generated
    let ip_host = "10.1.1.1".parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });
    let ip_container = "10.1.1.2".parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });

    match create_netns(args.namespace.clone()) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error create_namespace: {}", error.to_string());
            std::process::exit(1);
        }
    };

    match create_veth(name_host.clone(), name_container.clone()) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error create_veth: {}", error.to_string());
            std::process::exit(1);
        }
    };

    match move_veth_to_netns(name_container.clone(), args.namespace.clone()) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error move_veth_to_netns: {}", error.to_string());
            std::process::exit(1);
        }
    };

    match set_up_veth(name_host.clone()) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error set_up_veth: {}", error.to_string());
            std::process::exit(1);
        }
    };

    match set_up_veth_in_netns(name_container.clone(), args.namespace.clone()) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error set_up_veth_in_netns: {}", error.to_string());
            std::process::exit(1);
        }
    };

    match add_address(name_host.clone(), ip_host) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error add_address host: {}", error.to_string());
            std::process::exit(1);
        }
    };
    match add_address_in_netns(name_container.clone(), args.namespace.clone(), ip_container) {
        Ok(yes) => yes,
        Err(error) => {
            eprintln!("Error add_address container: {}", error.to_string());
            std::process::exit(1);
        }
    };
    Ok(())
}
