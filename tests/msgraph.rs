//! End-to-end Microsoft Graph API test.
//!
//! Requires an OAuth 2.0 access token with mail read/write/send scopes:
//!
//! ```sh
//! MSGRAPH_ACCESS_TOKEN="<token>" \
//! cargo test --test msgraph -- --include-ignored --nocapture
//! ```
//!
//! `MSGRAPH_USER_ID` is optional and defaults to `me`.

#![cfg(any(
    feature = "rustls-ring",
    feature = "rustls-aws",
    feature = "native-tls"
))]

use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use io_msgraph::v1::{
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions},
    rest::users::{
        mail_folders::MsgraphMailFolder,
        messages::{
            MsgraphBodyType, MsgraphEmailAddress, MsgraphItemBody, MsgraphMessage,
            MsgraphRecipient, list::MsgraphMessagesListParams,
        },
    },
};

#[test]
#[ignore = "requires MSGRAPH_ACCESS_TOKEN env var and --include-ignored"]
fn msgraph() {
    env_logger::try_init().ok();

    let token = env::var("MSGRAPH_ACCESS_TOKEN").expect("MSGRAPH_ACCESS_TOKEN not set");
    let user_id = env::var("MSGRAPH_USER_ID").unwrap_or_else(|_| String::from("me"));

    let options = MsgraphClientStdConnectOptions {
        user_id,
        ..Default::default()
    };
    let mut client = MsgraphClientStd::connect(token, options).expect("connect");

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let folder_name = format!("io-msgraph-test-{ts}");

    // ── ME ───────────────────────────────────────────────────────────────────

    let me = client.me().expect("me").response;
    let address = me
        .mail
        .clone()
        .or_else(|| me.user_principal_name.clone())
        .expect("user must expose an email address");
    assert!(!address.is_empty());

    // ── MAIL FOLDERS LIST (baseline) ─────────────────────────────────────────

    let folders = client
        .mail_folders_list(&Default::default())
        .expect("mail folders list")
        .response;
    assert!(
        !folders.value.is_empty(),
        "mailbox should expose at least one mail folder"
    );

    // ── MAIL FOLDER CREATE ───────────────────────────────────────────────────

    let new_folder = MsgraphMailFolder {
        display_name: folder_name.clone(),
        ..Default::default()
    };
    let folder = client
        .mail_folder_create(&new_folder)
        .expect("mail folder create")
        .response;
    let folder_id = folder.id.clone();
    assert_eq!(folder.display_name, folder_name);

    // ── MAIL FOLDER GET (verify creation) ────────────────────────────────────

    let fetched = client
        .mail_folder_get(&folder_id)
        .expect("mail folder get")
        .response;
    assert_eq!(fetched.display_name, folder_name);

    // ── MESSAGE CREATE (draft in the new folder) ─────────────────────────────

    let subject = format!("io-msgraph test {ts}");
    let draft = MsgraphMessage {
        subject: Some(subject.clone()),
        body: Some(MsgraphItemBody {
            content_type: Some(MsgraphBodyType::Text),
            content: Some(String::from("hello from io-msgraph")),
        }),
        to_recipients: vec![MsgraphRecipient {
            email_address: MsgraphEmailAddress {
                name: None,
                address: Some(address.clone()),
            },
        }],
        ..Default::default()
    };
    let message = client
        .message_create(Some(&folder_id), &draft)
        .expect("message create")
        .response;
    let message_id = message.id.clone();
    assert_eq!(message.subject.as_deref(), Some(subject.as_str()));

    // ── MESSAGE GET ──────────────────────────────────────────────────────────

    let fetched = client
        .message_get(&message_id)
        .expect("message get")
        .response;
    assert_eq!(fetched.subject.as_deref(), Some(subject.as_str()));

    // ── MESSAGE GET RAW (MIME) ───────────────────────────────────────────────

    let raw = client
        .message_get_raw(&message_id)
        .expect("message get raw")
        .response;
    assert!(!raw.is_empty(), "raw MIME should not be empty");

    // ── MESSAGE UPDATE (mark read) ───────────────────────────────────────────

    let patch = MsgraphMessage {
        is_read: Some(true),
        ..Default::default()
    };
    let updated = client
        .message_update(&message_id, &patch)
        .expect("message update")
        .response;
    assert_eq!(updated.is_read, Some(true));

    // ── MESSAGES LIST (in the new folder) ────────────────────────────────────

    let params = MsgraphMessagesListParams {
        top: Some(10),
        ..Default::default()
    };
    let listed = client
        .messages_list(Some(&folder_id), &params)
        .expect("messages list")
        .response;
    assert!(
        listed.value.iter().any(|m| m.id == message_id),
        "listing should contain the created message"
    );

    // ── MESSAGE COPY then MOVE ───────────────────────────────────────────────

    let copy = client
        .message_copy(&message_id, "drafts")
        .expect("message copy")
        .response;
    let moved = client
        .message_move(&copy.id, "deleteditems")
        .expect("message move")
        .response;

    // ── MESSAGE DELETE (original + the moved copy) ────────────────────────────

    client.message_delete(&message_id).expect("message delete");
    client.message_delete(&moved.id).expect("moved copy delete");

    // ── MAIL FOLDER DELETE (cleanup) ─────────────────────────────────────────

    client
        .mail_folder_delete(&folder_id)
        .expect("mail folder delete");

    // ── SEND MAIL (MIME, to self) ────────────────────────────────────────────

    let mime = format!(
        "To: {address}\r\nSubject: io-msgraph send test {ts}\r\n\r\nsent by the io-msgraph test suite\r\n"
    );
    client
        .send_mail_mime(mime.as_bytes())
        .expect("send mail mime");
}
