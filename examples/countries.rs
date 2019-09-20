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

    let get_countries = api
        .countries()
        .and_then(|res| {
            println!("Supported Countries:");
            for country in res {
                println!("{}: {}", country.iso_code, country.name);
            }
            Ok(())
        })
        .map_err(|e| println!("error: {:?}", e));

    tokio::run(get_countries);
}
