use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::sync::mpsc::Sender;
use tiny_http::Method;
use tracing::{event, Level};

use crate::api;
use crate::api::ApiChannel;

mod instance;
mod tenant;
mod workload;

type Handler = fn(
    &mut tiny_http::Request,
    &route_recognizer::Params,
    &Connection,
    &Sender<ApiChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError>;

pub struct Router {
    routes: Vec<(tiny_http::Method, route_recognizer::Router<Handler>)>,
}

impl Router {
    pub fn new() -> Router {
        let mut get = route_recognizer::Router::<Handler>::new();
        let mut post = route_recognizer::Router::<Handler>::new();

        let base_path = "/api/v0";

        // Workload related routes
        get.add(&format!("{}/workloads.list", base_path), workload::get);
        get.add(
            &format!("{}/workloads.instances/:workloadid", base_path),
            workload::get_instances,
        );
        post.add(&format!("{}/workloads.create", base_path), workload::create);
        post.add(&format!("{}/workloads.delete", base_path), workload::delete);

        // Tenant related routes
        get.add(&format!("{}/tenants.list", base_path), tenant::get);
        post.add(&format!("{}/tenants.create", base_path), tenant::create);
        post.add(&format!("{}/tenants.delete", base_path), tenant::delete);

        // Instance related routes
        get.add(&format!("{}/instances.list", base_path), instance::get);
        post.add(&format!("{}/instances.create", base_path), instance::create);
        post.add(&format!("{}/instances.delete", base_path), instance::delete);

        Router {
            routes: vec![(Method::Get, get), (Method::Post, post)],
        }
    }

    pub fn handle(
        &self,
        request: &mut tiny_http::Request,
        connection: &Connection,
        internal_sender: &Sender<ApiChannel>,
    ) -> Option<tiny_http::Response<io::Cursor<Vec<u8>>>> {
        self.routes
            .iter()
            .find(|&(method, _)| method == request.method())
            .and_then(|(_, routes)| {
                if let Ok(res) = routes.recognize(request.url()) {
                    event!(
                        Level::INFO,
                        "Route found, method: {}, path: {}",
                        request.method(),
                        request.url()
                    );
                    Some(
                        res.handler()(request, res.params(), connection, internal_sender)
                            .unwrap_or_else(|error| {
                                event!(Level::ERROR, "Could not handle route: {}", error);
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
