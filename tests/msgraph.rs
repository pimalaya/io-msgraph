//! End-to-end Microsoft Graph API tests.
//!
//! Requires an OAuth 2.0 access token with mail read/write/send scopes
//! for the mail test, and Contacts.ReadWrite for the contacts test:
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
    MsgraphField,
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions},
    rest::users::{
        contact_folders::MsgraphContactFolder,
        contacts::{MsgraphContact, list::MsgraphContactsListParams},
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

#[test]
#[ignore = "requires MSGRAPH_ACCESS_TOKEN env var and --include-ignored"]
fn msgraph_contacts() {
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

    // ── CONTACT FOLDERS LIST (baseline) ──────────────────────────────────────

    client
        .contact_folders_list(&Default::default())
        .expect("contact folders list");

    // ── CONTACT FOLDER CREATE ────────────────────────────────────────────────

    let new_folder = MsgraphContactFolder {
        display_name: folder_name.clone(),
        ..Default::default()
    };
    let folder = client
        .contact_folder_create(&new_folder)
        .expect("contact folder create")
        .response;
    let folder_id = folder.id.clone();
    assert_eq!(folder.display_name, folder_name);

    // ── CONTACT FOLDER GET (verify creation) ─────────────────────────────────

    let fetched = client
        .contact_folder_get(&folder_id)
        .expect("contact folder get")
        .response;
    assert_eq!(fetched.display_name, folder_name);

    // ── CONTACT FOLDER UPDATE (rename) ───────────────────────────────────────

    let renamed_folder_name = format!("{folder_name}-renamed");
    let folder_patch = MsgraphContactFolder {
        display_name: renamed_folder_name.clone(),
        ..Default::default()
    };
    let renamed = client
        .contact_folder_update(&folder_id, &folder_patch)
        .expect("contact folder update")
        .response;
    assert_eq!(renamed.display_name, renamed_folder_name);

    // ── CONTACT CHILD FOLDERS LIST (empty on a fresh folder) ─────────────────

    let children = client
        .contact_child_folders_list(&folder_id, &Default::default())
        .expect("contact child folders list")
        .response;
    assert!(
        children.value.is_empty(),
        "fresh contact folder should have no child folders"
    );

    // ── CONTACT CREATE (in the new folder) ───────────────────────────────────

    let given_name = format!("io-msgraph-{ts}");
    let new_contact = MsgraphContact {
        given_name: MsgraphField::Set(given_name.clone()),
        surname: MsgraphField::Set(String::from("Test")),
        email_addresses: MsgraphField::Set(vec![MsgraphEmailAddress {
            name: Some(given_name.clone()),
            address: Some(String::from("io-msgraph-test@example.com")),
        }]),
        business_phones: MsgraphField::Set(vec![String::from("+1 234 567 890")]),
        ..Default::default()
    };
    let contact = client
        .contact_create(Some(&folder_id), &new_contact)
        .expect("contact create")
        .response;
    let contact_id = contact.id.clone();
    assert_eq!(contact.given_name.as_deref(), Some(given_name.as_str()));

    // ── CONTACT GET ──────────────────────────────────────────────────────────

    let fetched = client
        .contact_get(&contact_id, None)
        .expect("contact get")
        .response;
    assert_eq!(fetched.given_name.as_deref(), Some(given_name.as_str()));
    let fetched_emails = fetched
        .email_addresses
        .as_option()
        .expect("email addresses");
    assert_eq!(
        fetched_emails[0].address.as_deref(),
        Some("io-msgraph-test@example.com")
    );

    // ── CONTACT UPDATE (rename) ──────────────────────────────────────────────

    let patch = MsgraphContact {
        surname: MsgraphField::Set(String::from("Renamed")),
        ..Default::default()
    };
    let updated = client
        .contact_update(&contact_id, &patch)
        .expect("contact update")
        .response;
    assert_eq!(updated.surname.as_deref(), Some("Renamed"));

    // ── CONTACTS LIST (in the new folder) ────────────────────────────────────

    let params = MsgraphContactsListParams {
        top: Some(10),
        ..Default::default()
    };
    let listed = client
        .contacts_list(Some(&folder_id), &params)
        .expect("contacts list")
        .response;
    assert!(
        listed.value.iter().any(|c| c.id == contact_id),
        "listing should contain the created contact"
    );

    // ── CONTACT DELETE ───────────────────────────────────────────────────────

    client.contact_delete(&contact_id).expect("contact delete");

    // ── CONTACT CREATE then DELETE (default Contacts folder) ─────────────────

    let default_contact = MsgraphContact {
        given_name: MsgraphField::Set(given_name.clone()),
        ..Default::default()
    };
    let contact = client
        .contact_create(None, &default_contact)
        .expect("default contact create")
        .response;
    let listed = client
        .contacts_list(None, &Default::default())
        .expect("default contacts list")
        .response;
    assert!(
        listed.value.iter().any(|c| c.id == contact.id),
        "default folder listing should contain the created contact"
    );
    client
        .contact_delete(&contact.id)
        .expect("default contact delete");

    // ── CONTACT FOLDER DELETE (cleanup) ──────────────────────────────────────

    client
        .contact_folder_delete(&folder_id)
        .expect("contact folder delete");
}
