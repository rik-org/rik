mod routes;
mod services;

use crate::api::ApiChannel;
use crate::database::RikDataBase;
use dotenv::dotenv;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use tiny_http::{Request, Server as TinyServer};

use tracing::{event, Level};

use super::RikError;

pub struct Server {
    internal_sender: Sender<ApiChannel>,
}

impl Server {
    pub fn new(internal_sender: Sender<ApiChannel>) -> Server {
        Server { internal_sender }
    }

    pub fn run(&self, db: Arc<RikDataBase>) -> Result<(), RikError> {
        self.run_server(db)
    }

    fn run_server(&self, db: Arc<RikDataBase>) -> Result<(), RikError> {
        let host = String::from("0.0.0.0");
        dotenv().ok();
        let port: usize = match std::env::var("PORT") {
            Ok(val) => val.parse().unwrap(),
            Err(_e) => 5000,
        };
        let server = TinyServer::http(format!("{}:{}", host, port)).unwrap();
        let server = Arc::new(server);

        let mut guards = Vec::with_capacity(4);

        for _ in 0..4 {
            let server = server.clone();
            let db = db.clone();
            let internal_sender = self.internal_sender.clone();

            let guard = thread::spawn(move || -> Result<(), RikError> {
                loop {
                    let router = routes::Router::new();
                    let connection = db.open().map_err(RikError::DatabaseError)?;

                    let mut req: Request = server.recv().unwrap();

                    if let Some(res) = router.handle(&mut req, &connection, &internal_sender) {
                        req.respond(res).unwrap();
                        continue;
                    }
                    event!(
                        Level::INFO,
                        "Route {} ({}) could not be found",
                        req.url(),
                        req.method()
                    );
                    req.respond(tiny_http::Response::empty(tiny_http::StatusCode::from(404)))
                        .unwrap();
                }
            });

            guards.push(guard);
        }

        for guard in guards {
            guard
                .join()
                .expect("Couldn't join on the associated thread")?
        }

        event!(Level::INFO, "Server running on http://{}:{}", host, port);
        Ok(())
    }
}
