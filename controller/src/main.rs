mod api;
mod database;
mod tests;
mod cli;

use std::{ sync::mpsc::channel, thread::JoinHandle };
use std::{ thread, error };

use crate::cli::Cli;
use crate::database::RikDataBase;
use anyhow::{ Result, Error };
use api::{ external, internal, ApiChannel };
use clap::Parser;
use env_logger::{ Builder };
use log::{ LevelFilter, info, error };

use tokio::runtime;

fn main() {
    // if !nix::unistd::Uid::effective().is_root() {
    //     println!("Rik controller must run with root privileges.");
    //     std::process::exit(1);
    // }

    let cli: Cli = Cli::parse();
    Builder::new()
        .filter_level(match cli.verbose {
            Some(1) => LevelFilter::Debug,
            Some(2) => LevelFilter::Trace,
            _ => LevelFilter::Info,
        })
        .init();
    info!("Starting up...");

    let db = RikDataBase::new(String::from("rik"));
    db.init_tables().unwrap_or_else(|e| { error!("Init table error: {}", e) });

    let (internal_sender, internal_receiver) = channel::<ApiChannel>();
    let (external_sender, external_receiver) = channel::<ApiChannel>();

    let internal_api = internal::Server::new(external_sender.clone(), internal_receiver);
    let external_api = external::Server::new(internal_sender.clone(), external_receiver);
    let mut threads: Vec<JoinHandle<()>> = Vec::new();

    let db_clone_internal = db.clone();
    threads.push(
        thread::spawn(move || {
            let future = async move {
                internal_api
                    .run(db_clone_internal).await
                    .unwrap_or_else(|e| { error!("Internal API error: {}", e) })
            };
            runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(future)
        })
    );

    let db_clone_external = db.clone();
    threads.push(
        thread::spawn(move || {
            external_api
                .run(db_clone_external)
                .unwrap_or_else(|e| { error!("External API error: {}", e) })
        })
    );

    for thread in threads {
        thread
            .join()
            .map_err(|_| Error::msg("Cannot join threads"))
            .unwrap();
    }
}