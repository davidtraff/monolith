#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

#[macro_use]
extern crate clap;

mod macros;

use monolith::html::{html_to_dom, stringify_document, walk_and_embed_assets};
use monolith::http::retrieve_asset;
use monolith::utils::{data_url_to_text, is_data_url, is_http_url};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::collections::HashMap;
use std::process;
use std::time::Duration;
use rocket::response::status::BadRequest;
use rocket::response::content::Content;
use rocket::http::ContentType;

fn fmt_err(result: String) -> Result<Content<String>, BadRequest<String>> {
    Err(BadRequest(Some(result)))
}

#[get("/?<target_url>&<timeout>&<insecure>&<no_js>&<no_images>&<no_frames>&<no_css>")]
fn index(
    target_url: Option<String>,
    timeout: Option<u64>,
    insecure: Option<bool>,
    no_js: Option<bool>,
    no_images: Option<bool>,
    no_frames: Option<bool>,
    no_css: Option<bool>
) -> Result<Content<String>, BadRequest<String>> {
    let target_url = &target_url.unwrap_or(String::from(""));

    if !is_http_url(&target_url) && !is_data_url(&target_url) {
        return fmt_err(format!("Only HTTP(S) or data URLs are supported but got: {}", &target_url));
    }

    // Initialize client
    let mut cache = HashMap::new();
    let mut header_map = HeaderMap::new();
    let base_url;
    let dom;

    header_map.insert(
        USER_AGENT,
        HeaderValue::from_str("Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:73.0) Gecko/20100101 Firefox/73.0").unwrap(),
    );

    let timeout = timeout.unwrap_or(0);
    let timeout: u64 = if timeout > 0 {
        timeout
    } else {
        std::u64::MAX / 4
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .danger_accept_invalid_certs(insecure.unwrap_or(false))
        .default_headers(header_map)
        .build()
        .expect("Failed to initialize HTTP client");

    // Retrieve root document
    if is_http_url(target_url) {
        let (data, final_url) =
            retrieve_asset(&mut cache, &client, target_url, false, "", true)
                .expect("Could not retrieve assets in HTML");
        base_url = final_url;
        dom = html_to_dom(&data);
    } else if is_data_url(target_url) {
        let text: String = data_url_to_text(target_url);

        if text.len() == 0 {
            return fmt_err(String::from("Unsupported data URL input"));
        }

        base_url = str!();
        dom = html_to_dom(&text);
    } else {
        return fmt_err(String::from("Invalid response"));
    }

    walk_and_embed_assets(
        &mut cache,
        &client,
        &base_url,
        &dom.document,
        no_css.unwrap_or(false),
        no_js.unwrap_or(false),
        no_images.unwrap_or(false),
        true,
        no_frames.unwrap_or(false),
    );

    let html: String = stringify_document(
        &dom.document,
        no_css.unwrap_or(false),
        no_frames.unwrap_or(false),
        no_js.unwrap_or(false),
        no_images.unwrap_or(false),
        true,
    );

    Ok(Content(ContentType::HTML, html))
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
