pub mod payloads;

use futures::{future, Future};
use serde::de::DeserializeOwned;

use reqwest::{
    header::HeaderMap,
    r#async::{Client, RequestBuilder, Response},
    Method, StatusCode,
};

use http;

const SANDBOX_API_BASE_URL: &str = "https://sandbox.pwinty.com/";
const LIVE_API_BASE_URL: &str = "https://api.pwinty.com/";
const API_VERSION: &str = "v3.0";

#[derive(Debug)]
pub enum ApiError {
    InternalError(String),
    RequestError(reqwest::Error),
    ResponseError { status: StatusCode },
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError::RequestError(error)
    }
}

impl From<http::header::InvalidHeaderValue> for ApiError {
    fn from(error: http::header::InvalidHeaderValue) -> Self {
        ApiError::InternalError(format!("{:?}", error).to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::InternalError(format!("{:?}", error).to_string())
    }
}

// impl<T: DeserializeOwned> Future for ApiError {
//     type Item = T;
//     type Error = ApiError;
//     fn poll(&mut self) -> Poll<T, Self::Error> {
//         Err(self)
//     }
// }

pub struct Api {
    base_url: &'static str,
    pub version: &'static str,
    merchant_id: String,
    api_key: String,
}

impl Api {
    pub fn new_sandbox(merchant_id: &str, api_key: &str) -> Self {
        Self {
            base_url: SANDBOX_API_BASE_URL,
            version: API_VERSION,
            merchant_id: merchant_id.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn new_live(merchant_id: &str, api_key: &str) -> Self {
        Self {
            base_url: LIVE_API_BASE_URL,
            version: API_VERSION,
            merchant_id: merchant_id.to_string(),
            api_key: api_key.to_string(),
        }
    }

    fn url_for_endpoint(&self, endpoint: &str) -> String {
        let mut url: String = self.base_url.to_string();
        url.push_str(self.version);
        url.push_str(endpoint);
        url
    }

    fn add_headers(&self, builder: RequestBuilder) -> Result<RequestBuilder, ApiError> {
        let mut headers = HeaderMap::new();
        headers.insert("X-Pwinty-MerchantId", self.merchant_id.parse()?);
        headers.insert("X-Pwinty-REST-API-Key", self.api_key.parse()?);
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("accept", "application/json".parse()?);
        Ok(builder.headers(headers))
    }

    fn make_request<Resp: 'static + DeserializeOwned + Send>(
        &self,
        endpoint: &str,
        method: Method,
        add_headers: bool,
        body: Option<String>,
    ) -> Box<dyn Future<Item = Resp, Error = ApiError> + Send> {
        let url = self.url_for_endpoint(endpoint);
        let client = if add_headers {
            match self.add_headers(Client::new().request(method, &url)) {
                Ok(res) => res,
                Err(e) => {
                    return Box::new(future::err(ApiError::InternalError(format!("{:?}", e))))
                }
            }
        } else {
            Client::new().request(method, &url)
        };

        let request = if let Some(request_body) = body {
            client.body(request_body)
        } else {
            client
        };
        Box::new(
            request
                .send()
                .map_err(|e| ApiError::RequestError(e))
                .and_then(|mut res: Response| {
                    if !res.status().is_success() {
                        return future::err(ApiError::ResponseError {
                            status: res.status(),
                        });
                    }
                    future::ok(res.json::<Resp>().map_err(|e| e.into()))
                })
                .flatten(),
        )
    }

    pub fn countries(
        &self,
    ) -> Box<dyn Future<Item = Vec<payloads::Country>, Error = ApiError> + Send> {
        Box::new(
            self.make_request::<payloads::Countries>("/countries", Method::GET, false, None)
                .and_then(|countries| Ok(countries.data)),
        )
    }

    pub fn create_order(
        &self,
        order_info: &payloads::OrderCreate,
    ) -> Box<dyn Future<Item = payloads::Order, Error = ApiError> + Send> {
        let body = match serde_json::to_string(&order_info) {
            Ok(body) => body,
            Err(e) => return Box::new(future::err(ApiError::InternalError(format!("{:?}", e)))),
        };

        Box::new(
            self.make_request::<payloads::OrderWrapper>("/orders", Method::POST, true, Some(body))
                .and_then(|order| future::ok(order.data)),
        )
    }

    pub fn add_image_to_order(
        &self,
        order_id: u64,
        image_info: &payloads::OrderImageAdd,
    ) -> Box<dyn Future<Item = payloads::OrderImage, Error = ApiError> + Send> {
        let body = match serde_json::to_string(&image_info) {
            Ok(body) => body,
            Err(e) => return Box::new(future::err(ApiError::InternalError(format!("{:?}", e)))),
        };

        Box::new(
            self.make_request::<payloads::OrderImageWrapper>(
                &format!("/orders/{:}/images", order_id),
                Method::POST,
                true,
                Some(body),
            )
            .and_then(|order| Ok(order.data)),
        )
    }
}
