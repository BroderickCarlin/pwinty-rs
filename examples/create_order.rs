use futures;
use futures::future::Future;
use pwinty;
use tokio;

fn main() {
    let merchant_id =
        std::env::var("XPwintyMerchantId").expect("Could not find 'XPwintyMerchantId' env var");
    let api_key =
        std::env::var("XPwintyRESTAPIKey").expect("Could not find 'XPwintyRESTAPIKey' env var");

    let api = pwinty::Api::new_sandbox(&merchant_id, &api_key)
        .expect("Something went wrong creating the API instance");

    let create_order = api
        .create_order(&pwinty::payloads::OrderCreate::base(
            "Best Customer Ever".to_string(),
            "012345".to_string(),
            "US".to_string(),
            pwinty::payloads::OrderShippingMethod::Express,
        ))
        .and_then(move |res| {
            println!("Created order: {:}", res.id);
            api.add_images_to_order(
                res.id,
                &[
                    pwinty::payloads::OrderImageAdd {
                        sku: "FRA-INSTA-40X40".to_string(),
                        url: "https://i.imgur.com/4AiXzf8.jpg".to_string(),
                        copies: 1,
                        sizing: pwinty::payloads::ImageResizingMethod::Crop,
                        price_to_user: None,
                        md5_hash: None,
                        attributes: None,
                    },
                    pwinty::payloads::OrderImageAdd {
                        sku: "FRA-INSTA-40X40".to_string(),
                        url: "https://i.imgur.com/H37kxPH.jpg".to_string(),
                        copies: 2,
                        sizing: pwinty::payloads::ImageResizingMethod::Crop,
                        price_to_user: None,
                        md5_hash: None,
                        attributes: None,
                    },
                ],
            )
        })
        .map(|res| println!("added image: {:?}", res))
        .map_err(|e| println!("error: {:?}", e));

    tokio::run(create_order);
}
