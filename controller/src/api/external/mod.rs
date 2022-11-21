mod routes;
mod services;

use crate::api::ApiChannel;
use crate::database::RikDataBase;
use anyhow::{ Result, Error };
use dotenv::dotenv;
use log::{ info, warn, error };
use std::sync::mpsc::{ Receiver, Sender };
use std::sync::Arc;
use std::{ thread };
use tiny_http::{ Request, Server as TinyServer };

use colored::Colorize;

pub struct Server {
    internal_sender: Sender<ApiChannel>,
    external_receiver: Receiver<ApiChannel>,
}

impl Server {
    pub fn new(
        internal_sender: Sender<ApiChannel>,
        external_receiver: Receiver<ApiChannel>
    ) -> Server {
        Server {
            internal_sender,
            external_receiver,
        }
    }

    pub fn run(&self, db: Arc<RikDataBase>) -> Result<()> {
        self.run_server(db)?;
        self.listen_notification();
        Ok(())
    }

    fn listen_notification(&self) {
        for notification in &self.external_receiver {
            info!("{}", notification);
        }
    }

    fn run_server(&self, db: Arc<RikDataBase>) -> Result<()> {
        let host = String::from("0.0.0.0");
        dotenv().ok();
        let port: usize = match std::env::var("PORT") {
            Ok(val) => val.parse().unwrap(),
            Err(_e) => 5000,
        };
        let server = TinyServer::http(format!("{}:{}", host, port)).map_err(|e| {
            let message = format!("Server failed to stard, {}", e);
            error!("{}", message);
            Error::msg(message)
        })?;
        let server = Arc::new(server);

        let mut guards = Vec::with_capacity(4);

        for _ in 0..4 {
            let server = server.clone();
            let db = db.clone();
            let internal_sender = self.internal_sender.clone();

            let guard = thread::spawn(
                move || -> Result<()> {
                    loop {
                        let router = routes::Router::new()?;
                        let connection = db.open()?;

                        let mut req: Request = server.recv()?;

                        if let Some(res) = router.handle(&mut req, &connection, &internal_sender) {
                            req.respond(res)?;
                            continue;
                        }
                        warn!("Route not found");
                        req.respond(tiny_http::Response::empty(tiny_http::StatusCode::from(404)))?;
                    }
                }
            );

            guards.push(guard);
        }
        info!("{}", format!("Server running on http://{}:{}", host, port).green());
        Ok(())
    }
}