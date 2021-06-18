extern crate reqwest;

#[derive(Debug)]
pub enum ApiError {
    BadURI(&'static str),
    BadStatus(reqwest::StatusCode),
    CantReadResponse,
    EmptyBody,
}

pub enum HttpVerb {
    GET,
    POST,
    DELETE,
}

pub mod api;