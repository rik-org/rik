extern crate reqwest;
use crate::config::Config;
use crate::{ApiError, HttpVerb};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ApiRequest {
    uri: String,
    endpoint: &'static str,
    header: Option<&'static str>,
    body: Option<HashMap<&'static str, &'static str>>,
}

impl ApiRequest {
    pub fn new(
        endpoint: &'static str,
        body: Option<HashMap<&'static str, &'static str>>,
        header: Option<&'static str>,
    ) -> Result<Self, ApiError> {
        let uri = Config::get_uri()?;
        let api_request = ApiRequest {
            endpoint,
            header,
            body,
            uri,
        };

        Ok(api_request)
    }

    pub fn get(self) -> Result<String, ApiError> {
        self.send_request(HttpVerb::GET)
    }

    pub fn post(self) -> Result<String, ApiError> {
        if self.body == None {
            return Err(ApiError::EmptyBody);
        }
        self.send_request(HttpVerb::POST)
    }

    pub fn delete(self) -> Result<String, ApiError> {
        self.send_request(HttpVerb::DELETE)
    }

    fn send_request(self, request_type: HttpVerb) -> Result<String, ApiError> {
        println!(
            "Sending request to {}",
            format!("{}{}", self.uri, self.endpoint)
        );

        match request_type {
            HttpVerb::GET => {
                //Get request
                match reqwest::blocking::get(format!("{}{}", self.uri, self.endpoint)) {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::OK {
                            match response.text() {
                                Ok(text) => Ok(text),
                                Err(_) => Err(ApiError::CantReadResponse),
                            }
                        } else {
                            Err(ApiError::BadStatus(response.status()))
                        }
                    }
                    Err(_) => Err(ApiError::BadURI(self.uri)),
                }
            }
            HttpVerb::POST => {
                //Post Request
                let client = reqwest::blocking::Client::new();
                match client
                    .post(format!("{}{}", self.uri, self.endpoint))
                    .json(&self.body)
                    .send()
                {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::CREATED {
                            return Ok("Test".to_string());
                        } else {
                            return Err(ApiError::BadStatus(response.status()));
                        }
                    }
                    Err(_) => return Err(ApiError::BadURI(self.uri)),
                }
            }
            HttpVerb::DELETE => {
                //Delete Request
                let client = reqwest::blocking::Client::new();
                match client
                    .delete(format!("{}{}", self.uri, self.endpoint))
                    .send()
                {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::OK {
                            return Ok("Test".to_string());
                        } else {
                            return Err(ApiError::BadStatus(response.status()));
                        }
                    }
                    Err(_) => return Err(ApiError::BadURI(self.uri)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::ApiRequest;
    use crate::ApiError;

    #[test]
    fn post_empty_body_return_error() -> Result<(), ApiError> {
        let api_request = ApiRequest::new("/NelopsisCode", None, None)?;
        assert!(api_request.post().is_err());

        Ok(())
    }
    #[test]
    fn invalid_endpoint_return_error() -> Result<(), ApiError> {
        let api_request = ApiRequest::new("/fez5f4e6157ae6f4faf7ef5aze4f3fa56", None, None)?;
        assert!(api_request.get().is_err());

        Ok(())
    }
}
