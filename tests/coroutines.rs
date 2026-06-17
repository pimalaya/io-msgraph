mod common;

use common::{empty_response, json_response, run, text_response};
use io_http::rfc6750::bearer::HttpAuthBearer;
use io_msgraph::v1::{
    query::to_query_pairs,
    rest::users::{
        mail_folders::{list::MsgraphMailFoldersList, list::MsgraphMailFoldersListParams},
        messages::{
            get_raw::MsgraphMessageGetRaw, list::MsgraphMessagesList,
            list::MsgraphMessagesListParams,
        },
        send_mail::MsgraphSendMailMime,
    },
    send::MsgraphSendError,
};

fn auth() -> HttpAuthBearer {
    HttpAuthBearer::new("test-token")
}

#[test]
fn mail_folders_list_parses_value() {
    let body = r#"{
        "value": [
            { "id": "AAA", "displayName": "Inbox", "totalItemCount": 71, "unreadItemCount": 70 },
            { "id": "BBB", "displayName": "Archive" }
        ]
    }"#;

    let mut coroutine = MsgraphMailFoldersList::new(&auth(), "me", &Default::default()).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(request.starts_with("GET /v1.0/me/mailFolders"));
    assert!(request.contains("Authorization: Bearer test-token"));

    assert_eq!(out.response.value.len(), 2);
    assert_eq!(out.response.value[0].id, "AAA");
    assert_eq!(out.response.value[0].display_name, "Inbox");
    assert_eq!(out.response.value[0].total_item_count, Some(71));
    assert_eq!(out.response.value[1].display_name, "Archive");
}

#[test]
fn messages_list_builds_odata_query() {
    let params = MsgraphMessagesListParams {
        top: Some(2),
        select: Some("id,subject,isRead"),
        orderby: Some("receivedDateTime desc"),
        ..Default::default()
    };

    let mut coroutine = MsgraphMessagesList::new(&auth(), "me", Some("inbox"), &params).unwrap();
    let (result, written) = run(
        &mut coroutine,
        &json_response("HTTP/1.1 200 OK", r#"{ "value": [] }"#),
    );
    assert!(result.is_ok());

    let request = String::from_utf8_lossy(&written);
    assert!(request.contains("/v1.0/me/mailFolders/inbox/messages"));
    assert!(request.contains("%24top=2"));
    assert!(request.contains("%24select=id%2Csubject%2CisRead"));
    assert!(request.contains("%24orderby=receivedDateTime+desc"));
    // unset params do not appear
    assert!(!request.contains("%24skip"));
    assert!(!request.contains("%24filter"));
}

#[test]
fn message_get_raw_returns_mime_bytes() {
    let mime = b"From: a@b.c\r\nSubject: hi\r\n\r\nbody\r\n";

    let mut coroutine = MsgraphMessageGetRaw::new(&auth(), "me", "ID123").unwrap();
    let (result, written) = run(&mut coroutine, &text_response("HTTP/1.1 200 OK", mime));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(request.contains("/v1.0/me/messages/ID123/$value"));
    assert_eq!(out.response, mime);
}

#[test]
fn send_mail_mime_base64_encodes_body() {
    let mime = b"From: a@b.c\r\nSubject: hi\r\n\r\nbody";

    let mut coroutine = MsgraphSendMailMime::new(&auth(), "me", mime).unwrap();
    let (result, written) = run(&mut coroutine, &empty_response("HTTP/1.1 202 Accepted"));
    assert!(result.is_ok());

    let request = String::from_utf8_lossy(&written);
    assert!(request.contains("POST /v1.0/me/sendMail"));
    assert!(request.contains("Content-Type: text/plain"));
    // base64 (standard) of the MIME, as Graph requires
    let expected = "RnJvbTogYUBiLmMNClN1YmplY3Q6IGhpDQoNCmJvZHk=";
    assert!(request.contains(expected));
}

#[test]
fn api_error_envelope_is_surfaced() {
    let body =
        r#"{ "error": { "code": "ErrorItemNotFound", "message": "The folder does not exist." } }"#;

    let mut coroutine = MsgraphMailFoldersList::new(&auth(), "me", &Default::default()).unwrap();
    let (result, _) = run(
        &mut coroutine,
        &json_response("HTTP/1.1 404 Not Found", body),
    );

    match result.unwrap_err() {
        MsgraphSendError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 404);
            assert_eq!(code, "ErrorItemNotFound");
            assert_eq!(message, "The folder does not exist.");
        }
        err => panic!("unexpected error: {err:?}"),
    }
}

#[test]
fn query_pairs_rename_to_odata_options() {
    let params = MsgraphMailFoldersListParams {
        top: Some(10),
        include_hidden_folders: Some(true),
        ..Default::default()
    };

    let pairs = to_query_pairs(&params);
    assert!(pairs.contains(&(String::from("$top"), String::from("10"))));
    assert!(pairs.contains(&(String::from("includeHiddenFolders"), String::from("true"))));
    assert_eq!(pairs.len(), 2);
}
