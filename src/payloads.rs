use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub name: String,
    pub iso_code: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Countries {
    pub data: Vec<Country>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreate {
    pub merchant_order_id: Option<u64>,
    pub recipient_name: String,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub address_town_or_city: Option<String>,
    pub state_or_county: Option<String>,
    pub postal_or_zip_code: String,
    pub country_code: String,
    pub preferred_shipping_method: OrderShippingMethod,
    pub payment: Option<OrderPayment>,
    pub packing_slip_url: Option<String>, // set as URL object?
    pub mobile_telephone: Option<String>,
    pub email: Option<String>,
    pub invoice_amount_net: Option<f64>,
    pub invoice_tax: Option<f64>,
    pub invoice_currency: Option<String>,
}

impl OrderCreate {
    pub fn base(
        recipient_name: String,
        postal_or_zip_code: String,
        country_code: String,
        preferred_shipping_method: OrderShippingMethod,
    ) -> Self {
        OrderCreate {
            merchant_order_id: None,
            recipient_name,
            address1: None,
            address2: None,
            address_town_or_city: None,
            state_or_county: None,
            postal_or_zip_code,
            country_code,
            preferred_shipping_method,
            payment: None,
            packing_slip_url: None,
            mobile_telephone: None,
            email: None,
            invoice_amount_net: None,
            invoice_tax: None,
            invoice_currency: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct OrderWrapper {
    pub data: Order,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderShippingMethod {
    Budget,
    Standard,
    Express,
    Overnight,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderPayment {
    InvoiceMe,
    InvoiceRecipient,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderStatus {
    NotYetDownloaded,
    NotYetSubmitted,
    Submitted,
    AwaitingPayment,
    Complete,
    Cancelled,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: u64,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub postal_or_zip_code: String,
    pub country_code: String,
    pub address_town_or_city: Option<String>,
    pub recipient_name: String,
    pub state_or_county: Option<String>,
    pub status: OrderStatus,
    pub payment: OrderPayment,
    pub payment_url: Option<String>, // set as URL object?
    pub price: f64,
    pub shipping_info: OrderShippingInfo,
    pub images: Vec<OrderImage>,
    pub invoice_ammount_net: Option<f64>,
    pub invoice_tax: Option<f64>,
    pub invoice_currency: Option<String>,
    pub merchant_order_id: Option<String>,
    pub preferred_shipping_method: OrderShippingMethod,
    pub mobile_telephone: Option<String>,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub can_cancel: bool,
    pub can_hold: bool,
    pub can_update_shipping: bool,
    pub can_update_images: bool,
    pub tag: Option<String>,
    pub packing_slip_url: Option<String>, // set as URL object?
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OrderShippingCarrier {
    RoyalMail,
    RoyalMailFirstClass,
    RoyalMailSecondClass,
    FedEx,
    FedExUK,
    FedExIntl,
    Interlink,
    UPS,
    UpsTwoDay,
    UKMail,
    TNT,
    ParcelForce,
    DHL,
    UPSMI,
    DpdNextDay,
    EuPostal,
    AuPost,
    AirMail,
    NotKnown,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderShippingInfo {
    pub price: f64,
    pub shipments: Vec<OrderShipment>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderShipment {
    pub carrier: OrderShippingCarrier,
    pub photo_ids: Vec<u64>,
    pub shipment_id: String,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>, // set as URL object
    pub is_tracked: bool,
    pub earliest_estimated_arrival_date: Option<DateTime<Utc>>,
    pub latest_estimated_arrival_date: Option<DateTime<Utc>>,
    pub shipped_on: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderImageWrapper {
    pub data: OrderImage,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderImagesWrapperInner {
    pub items: Vec<OrderImage>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderImagesWrapper {
    pub data: OrderImagesWrapperInner,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderImage {
    pub id: u64,
    pub sku: String,
    pub url: String,
    pub status: OrderStatus,
    pub copies: u64,
    pub sizing: String, // really fucking hope this is a string
    pub price_to_user: Option<f64>,
    pub price: f64,
    pub md5_hash: Option<String>,
    pub preview_url: Option<String>,   // set as URL object
    pub thumbnail_url: Option<String>, // set as URL object
    pub attributes: Option<OrderImageAttributes>,
    pub error_message: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderImageAttributes {
    pub substrate_weight: Option<String>,
    pub frame: Option<String>,
    pub edge: Option<String>,
    pub paper_type: Option<String>,
    pub frame_colour: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ImageResizingMethod {
    Crop,
    ShrinkToFit,
    ShrinkToExactFit,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderImageAdd {
    pub sku: String,
    pub url: String,
    pub copies: u64,
    pub sizing: ImageResizingMethod,
    pub price_to_user: Option<f64>,
    pub md5_hash: Option<String>,
    pub attributes: Option<OrderImageAttributes>,
}
