use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::sync::mpsc::Sender;

use crate::api;
use crate::api::ApiChannel;
use crate::logger::{LogType, LoggingChannel};

mod instance;
mod tenant;
mod workload;

type Handler = fn(
    &mut tiny_http::Request,
    &route_recognizer::Params,
    &Connection,
    &Sender<ApiChannel>,
    &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError>;

pub struct Router {
    routes: Vec<(tiny_http::Method, route_recognizer::Router<Handler>)>,
}

impl Router {
    pub fn new() -> Router {
        let mut get = route_recognizer::Router::<Handler>::new();
        let mut post = route_recognizer::Router::<Handler>::new();

        let base_path = "/api/v0";

        // GET
        get.add(&format!("{}/instances.list", base_path), instance::get);
        get.add(&format!("{}/tenants.list", base_path), tenant::get);
        get.add(&format!("{}/workloads.list", base_path), workload::get);
        // POST
        post.add(&format!("{}/instances.create", base_path), instance::create);
        post.add(&format!("{}/tenants.create", base_path), tenant::create);
        post.add(&format!("{}/workloads.create", base_path), workload::create);
        post.add(&format!("{}/instances.delete", base_path), instance::delete);
        post.add(&format!("{}/tenants.delete", base_path), tenant::delete);
        post.add(&format!("{}/workloads.delete", base_path), workload::delete);

        Router {
            routes: vec![
                ("GET".parse().unwrap(), get),
                ("POST".parse().unwrap(), post),
            ],
        }
    }

    pub fn handle(
        &self,
        request: &mut tiny_http::Request,
        connection: &Connection,
        internal_sender: &Sender<ApiChannel>,
        logger: &Sender<LoggingChannel>,
    ) -> Option<tiny_http::Response<io::Cursor<Vec<u8>>>> {
        self.routes
            .iter()
            .find(|&&(ref method, _)| method == request.method())
            .and_then(|&(_, ref routes)| {
                if let Ok(res) = routes.recognize(request.url()) {
                    Some(
                        res.handler()(request, res.params(), connection, internal_sender, logger)
                            .unwrap_or_else(|error| {
                                logger
                                    .send(LoggingChannel {
                                        message: error.to_string(),
                                        log_type: LogType::Error,
                                    })
                                    .unwrap();
                                tiny_http::Response::from_string(error.to_string())
                                    .with_status_code(tiny_http::StatusCode::from(400))
                            }),
                    )
                } else {
                    None
                }
            })
    }
}
