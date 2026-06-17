//! End-to-end test against the live Microsoft Graph API.
//!
//! Ignored by default; run with a TLS feature and a token:
//!
//! ```sh
//! MSGRAPH_ACCESS_TOKEN=... cargo test --test msgraph -- --ignored --nocapture
//! ```

#![cfg(any(
    feature = "rustls-ring",
    feature = "rustls-aws",
    feature = "native-tls"
))]

use std::env;

use io_msgraph::v1::client::{MsgraphClientStd, MsgraphClientStdConnectOptions};

#[test]
#[ignore = "requires a live Microsoft Graph access token"]
fn live_smoke() {
    let _ = env_logger::builder().is_test(true).try_init();

    let token = env::var("MSGRAPH_ACCESS_TOKEN").expect("MSGRAPH_ACCESS_TOKEN must be set");
    let user_id = env::var("MSGRAPH_USER_ID").unwrap_or_else(|_| String::from("me"));

    let options = MsgraphClientStdConnectOptions {
        user_id,
        ..Default::default()
    };
    let mut client = MsgraphClientStd::connect(token, options).unwrap();

    let me = client.me().unwrap();
    println!("user: {:?}", me.response);

    let folders = client.mail_folders_list(&Default::default()).unwrap();
    for folder in &folders.response.value {
        println!("{}: {}", folder.id, folder.display_name);
    }
}
