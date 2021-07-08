mod structs;
mod core;
mod traits;
mod emitters;
mod config;
mod constants;

use crate::core::Riklet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // If the process doesn't have root privileges, exit and display error.
    if !nix::unistd::Uid::effective().is_root() {
        log::error!("Riklet must run with root privileges.");
        std::process::exit(1);
    }

    let mut riklet = match Riklet::bootstrap().await {
        Ok(instance) => instance,
        Err(error) => {
            // if there is an error during the boostrap process of the riklet, log & error
            log::error!("An error occured during the bootstraping process of the Riklet. Details : {}", error.to_string());
            std::process::exit(2);
        }
    };

    riklet.accept().await?;

    Ok(())
}