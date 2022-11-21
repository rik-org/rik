use anyhow::Result;
use log::error;
use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::sync::mpsc::Sender;

use crate::api::{ApiChannel, RikError};
use tiny_http::{Method, Response, StatusCode};

mod instance;
mod tenant;
mod workload;

type HttpResult<T = io::Cursor<Vec<u8>>> = Result<Response<T>, RikError>;

type Handler = fn(
    &mut tiny_http::Request,
    &route_recognizer::Params,
    &Connection,
    &Sender<ApiChannel>,
) -> HttpResult;

pub struct Router {
    routes: Vec<(Method, route_recognizer::Router<Handler>)>,
}

impl Router {
    pub fn new() -> Result<Router> {
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

        Ok(Router {
            routes: vec![(Method::Get, get), (Method::Post, post)],
        })
    }

    pub fn handle(
        &self,
        request: &mut tiny_http::Request,
        connection: &Connection,
        internal_sender: &Sender<ApiChannel>,
    ) -> Option<Response<io::Cursor<Vec<u8>>>> {
        self.routes
            .iter()
            .find(|&&(ref method, _)| method == request.method())
            .and_then(|&(_, ref routes)| {
                if let Ok(res) = routes.recognize(request.url()) {
                    Some(
                        res.handler()(request, res.params(), connection, internal_sender)
                            .unwrap_or_else(|error| {
                                error!("{}", error);
                                Response::from_string(error.to_string())
                                    .with_status_code(StatusCode::from(400))
                            }),
                    )
                } else {
                    None
                }
            })
    }
}
