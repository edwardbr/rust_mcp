#[allow(warnings)]
mod bindings;

// use std::result;

use bindings::Guest;
use serde_json::json;
use serde_json::Value;

use http::{
    Request, // , Response
             // , StatusCode
};

struct Component;

impl Guest for Component {
    fn register_routes() {
        klave::router::add_user_query("load-from-ledger");
        klave::router::add_user_transaction("insert-in-ledger");

        klave::router::add_user_query("cricket-scores");
        klave::router::add_user_query("post-data");
    }

    fn load_from_ledger(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("failed to parse '{cmd}' as json"));
            return;
        };
        let key = v["key"].as_str().unwrap();
        let Ok(res) = klave::ledger::get_table("my_table").get(key) else {
            klave::notifier::send_string(&format!("failed to read from ledger: '{cmd}'"));
            return;
        };
        let msg = if res.is_empty() {
            format!("the key '{cmd}' was not found in table my_table")
        } else {
            let result_as_json = json!({
                "value": String::from_utf8(res).unwrap_or("!! utf8 parsing error !!".to_owned()),
            });
            format!("{result_as_json}")
        };
        klave::notifier::send_string(&msg);
    }

    fn insert_in_ledger(cmd: String) {
        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("failed to parse '{cmd}' as json"));
            klave::router::cancel_transaction();
            return;
        };
        let key = v["key"].as_str().unwrap();
        let value = v["value"].as_str().unwrap().as_bytes();
        if let Err(e) = klave::ledger::get_table("my_table").set(key, value) {
            klave::notifier::send_string(&format!("failed to write to ledger: '{e}'"));
            klave::router::cancel_transaction();
            return;
        }

        let result_as_json = json!({
        "inserted": true,
        "key": key,
        "value": value
        });
        klave::notifier::send_string(&result_as_json.to_string());
    }

    fn post_data(json: String) {
        klave::notifier::send_string(&json);

        let Ok(v) = serde_json::from_str::<Value>(&json) else {
            klave::notifier::send_string(&format!("failed to parse '{json}' as json"));
            return;
        };

        klave::notifier::send_string(&v.as_str().unwrap());

        let url = v["url"].as_str().unwrap();
        klave::notifier::send_string(&url);

        let method: &str = v["method"].as_str().unwrap();
        klave::notifier::send_string(&method);

        let body: &str = v["body"].as_str().unwrap();
        klave::notifier::send_string(&body);

        let https_request = Request::builder()
            .method(method)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .unwrap();

        let response: http::Response<String> = match klave::https::request(&https_request) {
            Ok(r) => r,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "https_query {} failure: {}",
                    https_request.body(),
                    e
                ));
                return;
            }
        };

        klave::notifier::send_string(&format!("body {}", response.body()));
    }

    fn cricket_scores(cmd: String) {
        klave::notifier::send_string(&cmd);

        let Ok(v) = serde_json::from_str::<Value>(&cmd) else {
            klave::notifier::send_string(&format!("failed to parse '{cmd}' as json"));
            klave::router::cancel_transaction();
            return;
        };

        klave::notifier::send_string(&v.to_string().as_str());

        let url = v["url"].as_str().unwrap();
        klave::notifier::send_string(&url);

        let https_request = Request::builder()
            .method("GET")
            .uri(url)
            .header("Content-Type", "application/json")
            .body(String::from(""))
            .unwrap();

        klave::notifier::send_string("message sent");

        let response: http::Response<String> = match klave::https::request(&https_request) {
            Ok(r) => r,
            Err(e) => {
                klave::notifier::send_string(&format!(
                    "https_query {} failure: {}",
                    https_request.body(),
                    e
                ));
                return;
            }
        };

        klave::notifier::send_string(&response.status().to_string());

        klave::notifier::send_string(&format!("body {}", response.body()));
    }
}

bindings::export!(Component with_types_in bindings);
