extern crate reqwest;
use std::collections::HashMap;
use crate::{ApiError, HttpVerb};

#[derive(Debug)]
pub struct ApiRequest {
    endpoint: &'static str,
    header: &'static str, 
    body: Option<HashMap<&'static str,&'static str>>,
}

pub const URI: &str = "https://github.com/";


impl ApiRequest {
    pub fn new(endpoint: &'static str,
        header: &'static str, 
        body: Option<HashMap<&'static str,&'static str>>) -> Self {
        let api_request = ApiRequest {
        endpoint,
        header,
        body
        };

        api_request
    }


    pub fn get (self) -> Result<(), ApiError> {
        self.send_request(HttpVerb::GET)
    }

    pub fn post (self) -> Result<(), ApiError> {
        if self.body == None {
            return Err(ApiError::EmptyBody)
        }
        self.send_request(HttpVerb::POST)
    }

    pub fn delete (self) -> Result<(), ApiError> {
        self.send_request(HttpVerb::DELETE)
    }

    fn send_request (self, request_type: HttpVerb) -> Result<(), ApiError> {
        println!("Sending request to {}", format!("{}{}",URI, self.endpoint));

        match request_type {
            HttpVerb::GET => {
                //Get request
                match reqwest::blocking::get(format!("{}{}",URI, self.endpoint)) {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::OK {
                            match response.text() {
                                Ok(text) => {
                                    println!("{}", text);
                                    Ok(())
                                },
                                Err(_) => Err(ApiError::CantReadResponse)
                            }
                        } else {
                            Err(ApiError::BadStatus(response.status()))
                        }
                    }
                    Err(_) => Err(ApiError::BadURI(URI))
                }
            }
            HttpVerb::POST => {
                //Post Request
                let client = reqwest::blocking::Client::new();
                match client
                .post(format!("{}{}",URI, self.endpoint))
                .json(&self.body)
                .send() {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::CREATED {
                            return Ok(());
                        } else {
                            return Err(ApiError::BadStatus(response.status()));
                        }
                    }
                    Err(_) => return Err(ApiError::BadURI(URI))
                }
            }
            HttpVerb::DELETE => {
                //Delete Request
                let client = reqwest::blocking::Client::new();
                match client
                .delete(format!("{}{}",URI, self.endpoint))
                .send() {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::OK {
                            return Ok(());
                        } else {
                            return Err(ApiError::BadStatus(response.status()));
                        }
                    }
                    Err(_) => return Err(ApiError::BadURI(URI))
                }
            }
        }
        
    }
}

