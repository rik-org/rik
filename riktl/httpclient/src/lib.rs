extern crate reqwest;

#[derive(Debug)]
pub enum ApiError {
    BadURI(String),
    BadStatus(reqwest::StatusCode),
    CantReadResponse,
    EmptyBody,
    CantOpenConfigFile,
    CantReadConfigFile,
    BadConfigFile
}

pub enum HttpVerb {
    GET,
    POST,
    DELETE,
}

pub mod api;
pub mod config;