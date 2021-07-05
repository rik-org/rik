use clap::{AppSettings, Clap};
use rtnetlink::new_connection;

#[tokio::main]
async fn create_veth() -> Result<(), String> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .add()
        .veth("veth-rs-1".into(), "veth-rs-2".into())
        .execute()
        .await
        .map_err(|e| format!("{}", e))
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    namespace_path: String,

    #[clap(subcommand)]
    subcmd: SubCommand,

}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "1.3", author = "Someone E. <someone_else@other.com>")]
    Create(create),
}

/// A subcommand for controlling testing
#[derive(Clap)]
struct create {
    /// Print debug info
    #[clap(short)]
    debug: bool,

}

fn main() -> Result<(), String> {
    let opts: Opts = Opts::parse();


    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::Create(t) => {
            if t.debug {
                println!("Printing debug info...");
            } else {
                println!("Printing normally...");
            }
        }
    }


    create_veth()
}
