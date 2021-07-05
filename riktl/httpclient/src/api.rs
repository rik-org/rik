extern crate reqwest;
use crate::config::Config;
use crate::ApiError;
use serde_json::Value;

#[derive(Debug)]
pub struct ApiRequest {
    uri: String,
    endpoint: String,
    body: Option<String>,
}

impl ApiRequest {
    pub fn new(endpoint: String, body: Option<String>) -> Result<Self, ApiError> {
        let uri = Config::get_uri()?;

        Ok(ApiRequest {
            uri,
            endpoint,
            body,
        })
    }

    pub fn get(self) -> Result<Vec<Value>, ApiError> {
        //Get request
        match reqwest::blocking::get(format!("{}{}", self.uri, self.endpoint)) {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.text() {
                        Ok(text) => {
                            let body_value: Vec<Value> = serde_json::from_str(&text).unwrap();
                            Ok(body_value)
                        }
                        Err(_) => Err(ApiError::CantReadResponse),
                    }
                } else {
                    Err(ApiError::BadStatus(response.status()))
                }
            }
            Err(_) => Err(ApiError::BadURI(self.uri)),
        }
    }

    pub fn post(self) -> Result<Value, ApiError> {
        if self.body == None {
            return Err(ApiError::EmptyBody);
        }
        //Post Request
        let client = reqwest::blocking::Client::new();
        if let Some(body) = self.body {
            let body_value: Value = serde_json::from_str(&body).unwrap();
            match client
                .post(format!("{}{}", self.uri, self.endpoint))
                .json(&body_value)
                .header("Content-Type", "application/json")
                .send()
            {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::CREATED
                        || response.status() == reqwest::StatusCode::NO_CONTENT
                        || response.status() == reqwest::StatusCode::OK
                    {
                        match response.text() {
                            Ok(text) => {
                                if text.is_empty() {
                                    return Ok(Value::Null);
                                }
                                let body_value: Value = serde_json::from_str(&text).unwrap();
                                Ok(body_value)
                            }
                            Err(_) => Err(ApiError::CantReadResponse),
                        }
                    } else {
                        let error = response.status();
                        println!("{}", &response.text().unwrap());
                        Err(ApiError::BadStatus(error))
                    }
                }
                Err(_) => Err(ApiError::BadURI(self.uri)),
            }
        } else {
            Err(ApiError::EmptyBody)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::ApiRequest;
    use crate::ApiError;

    #[test]
    fn post_empty_body_return_error() -> Result<(), ApiError> {
        let api_request = ApiRequest::new("/NelopsisCode".to_string(), None)?;
        assert!(api_request.post().is_err());

        Ok(())
    }
    #[test]
    fn invalid_endpoint_return_error() -> Result<(), ApiError> {
        let api_request = ApiRequest::new("/fez5f4e6157ae6f4faf7ef5aze4f3fa56".to_string(), None)?;
        assert!(api_request.get().is_err());

        Ok(())
    }
}
