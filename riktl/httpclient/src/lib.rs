extern crate reqwest;

#[derive(Debug)]
pub enum ApiError {
    BadURI(String),
    BadStatus(reqwest::StatusCode),
    CantReadResponse,
    EmptyBody,
    CantOpenConfigFile,
    CantReadConfigFile,
    BadConfigFile,
}

pub mod api;
pub mod config;
