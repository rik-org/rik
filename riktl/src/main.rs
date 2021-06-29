use httpclient::api::ApiRequest;
use httpclient::ApiError;
//use std::collections::HashMap;

fn main() -> Result<(), ApiError> {
    let api_request = ApiRequest::new("sameo", None, None)?;
    let resp: String = api_request.get()?;
    println!("{}", resp);
    return Ok(());
}
