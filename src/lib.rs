pub mod payloads;

use futures::{future, Future};
use serde::de::DeserializeOwned;

use reqwest::{
    header::{self, HeaderMap},
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

pub struct Api {
    base_url: &'static str,
    pub version: &'static str,
    merchant_id: header::HeaderValue,
    api_key: header::HeaderValue,
    client: Client,
}

impl Api {
    pub fn new_sandbox(
        merchant_id: &str,
        api_key: &str,
    ) -> Result<Self, header::InvalidHeaderValue> {
        Ok(Self {
            base_url: SANDBOX_API_BASE_URL,
            version: API_VERSION,
            merchant_id: header::HeaderValue::from_str(merchant_id)?,
            api_key: header::HeaderValue::from_str(api_key)?,
            client: Client::new(),
        })
    }

    pub fn new_live(merchant_id: &str, api_key: &str) -> Result<Self, header::InvalidHeaderValue> {
        Ok(Self {
            base_url: LIVE_API_BASE_URL,
            version: API_VERSION,
            merchant_id: header::HeaderValue::from_str(merchant_id)?,
            api_key: header::HeaderValue::from_str(api_key)?,
            client: Client::new(),
        })
    }

    fn url_for_endpoint(&self, endpoint: &str) -> String {
        let mut url: String = self.base_url.to_string();
        url.push_str(self.version);
        url.push_str(endpoint);
        url
    }

    fn add_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        let mut headers = HeaderMap::new();
        headers.insert("X-Pwinty-MerchantId", self.merchant_id.clone());
        headers.insert("X-Pwinty-REST-API-Key", self.api_key.clone());
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "accept",
            header::HeaderValue::from_static("application/json"),
        );
        builder.headers(headers)
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
            self.add_headers(self.client.request(method, &url))
        } else {
            self.client.request(method, &url)
        };

        let request = if let Some(request_body) = body {
            client.body(request_body)
        } else {
            client
        };
        Box::new(
            request
                .send()
                .map_err(ApiError::RequestError)
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
                .and_then(|order| Ok(order.data)),
        )
    }

    // pub fn add_image_to_order(
    //     &self,
    //     order_id: u64,
    //     image_info: &payloads::OrderImageAdd,
    // ) -> Box<dyn Future<Item = payloads::OrderImage, Error = ApiError> + Send> {
    //     let body = match serde_json::to_string(&image_info) {
    //         Ok(body) => body,
    //         Err(e) => return Box::new(future::err(ApiError::InternalError(format!("{:?}", e)))),
    //     };

    //     Box::new(
    //         self.make_request::<payloads::OrderImageWrapper>(
    //             &format!("/orders/{:}/images", order_id),
    //             Method::POST,
    //             true,
    //             Some(body),
    //         )
    //         .and_then(|order| Ok(order.data)),
    //     )
    // }

    pub fn add_images_to_order(
        &self,
        order_id: u64,
        images_info: &[payloads::OrderImageAdd],
    ) -> Box<dyn Future<Item = Vec<payloads::OrderImage>, Error = ApiError> + Send> {
        if images_info.is_empty() {
            return Box::new(future::err(ApiError::InternalError(
                "No images provided".to_string(),
            )));
        }

        // If only a single image is provided we will utilize the endpoint designed to
        // only take a single image, otherwise we fall back to a batch upload
        if images_info.len() == 1 {
            let body = match serde_json::to_string(&images_info[0]) {
                Ok(body) => body,
                Err(e) => {
                    return Box::new(future::err(ApiError::InternalError(format!("{:?}", e))))
                }
            };

            Box::new(
                self.make_request::<payloads::OrderImageWrapper>(
                    &format!("/orders/{:}/images", order_id),
                    Method::POST,
                    true,
                    Some(body),
                )
                .and_then(|order| Ok(vec![order.data])),
            )
        } else {
            let body = match serde_json::to_string(&images_info) {
                Ok(body) => body,
                Err(e) => {
                    return Box::new(future::err(ApiError::InternalError(format!("{:?}", e))))
                }
            };

            Box::new(
                self.make_request::<payloads::OrderImagesWrapper>(
                    &format!("/orders/{:}/images/batch", order_id),
                    Method::POST,
                    true,
                    Some(body),
                )
                .and_then(|order| Ok(order.data.items)),
            )
        }
    }
}
