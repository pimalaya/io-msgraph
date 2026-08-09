//! In-memory coroutine tests: each coroutine runs against a scripted
//! HTTP response (no network), checking the request it writes and the
//! value it returns.

mod common;

use common::{empty_response, json_response, run, text_response};
use io_http::rfc6750::bearer::HttpAuthBearer;
use io_msgraph::v1::{
    field::MsgraphField,
    query::to_query_pairs,
    rest::users::{
        contact_folders::{
            MsgraphContactFolder, create::MsgraphContactFolderCreate,
            delete::MsgraphContactFolderDelete, list::MsgraphContactFoldersList,
        },
        contacts::{
            MsgraphContact, create::MsgraphContactCreate, delete::MsgraphContactDelete,
            delta::MsgraphContactsDelta, list::MsgraphContactsList,
            list::MsgraphContactsListParams, update::MsgraphContactUpdate,
        },
        get::MsgraphUserGet,
        mail_folders::{
            MsgraphMailFolder, create::MsgraphMailFolderCreate, delete::MsgraphMailFolderDelete,
            list::MsgraphMailFoldersList, list::MsgraphMailFoldersListParams,
            update::MsgraphMailFolderUpdate,
        },
        messages::{
            MsgraphEmailAddress, MsgraphMessage,
            attachments::{
                delete::MsgraphAttachmentDelete, get_raw::MsgraphAttachmentGetRaw,
                list::MsgraphAttachmentsList,
            },
            create_mime::MsgraphMessageCreateMime,
            delta::MsgraphMessagesDelta,
            get_raw::MsgraphMessageGetRaw,
            list::MsgraphMessagesList,
            list::MsgraphMessagesListParams,
            r#move::MsgraphMessageMove,
            update::MsgraphMessageUpdate,
        },
        send_mail::MsgraphMailSendMime,
    },
    send::{MsgraphSendError, parse_api_error},
};

fn auth() -> HttpAuthBearer {
    HttpAuthBearer::new("test-token")
}

#[test]
fn gets_user() {
    let body = r#"{"id":"abc","displayName":"Alice","mail":"alice@example.com","userPrincipalName":"alice@example.com"}"#;

    let mut coroutine = MsgraphUserGet::new(&auth(), "me").unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(request.starts_with("GET /v1.0/me"));
    assert!(request.contains("Authorization: Bearer test-token"));
    assert_eq!(out.response.mail.as_deref(), Some("alice@example.com"));
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

    assert_eq!(out.response.value.len(), 2);
    assert_eq!(out.response.value[0].id, "AAA");
    assert_eq!(out.response.value[0].display_name, "Inbox");
    assert_eq!(out.response.value[0].total_item_count, Some(71));
    assert_eq!(out.response.value[1].display_name, "Archive");
}

#[test]
fn creates_folder_posts_display_name() {
    let body = r#"{ "id": "AAA", "displayName": "todo" }"#;

    let folder = MsgraphMailFolder {
        display_name: "todo".into(),
        ..Default::default()
    };
    let mut coroutine = MsgraphMailFolderCreate::new(&auth(), "me", &folder).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 201 Created", body));

    assert_eq!(result.unwrap().response.id, "AAA");

    let request = String::from_utf8_lossy(&written);
    assert!(request.starts_with("POST /v1.0/me/mailFolders"));
    assert!(
        request.contains("\"displayName\":\"todo\""),
        "got: {request}"
    );
}

#[test]
fn rejects_empty_folder_name() {
    let folder = MsgraphMailFolder {
        display_name: "  ".into(),
        ..Default::default()
    };
    let result = MsgraphMailFolderCreate::new(&auth(), "me", &folder);
    assert!(matches!(result, Err(MsgraphSendError::InvalidRequest(_))));
}

#[test]
fn deletes_folder_issues_delete() {
    let mut coroutine = MsgraphMailFolderDelete::new(&auth(), "me", "AAA").unwrap();
    let (result, written) = run(&mut coroutine, &empty_response("HTTP/1.1 204 No Content"));

    result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("DELETE /v1.0/me/mailFolders/AAA"),
        "got: {request}"
    );
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
    // NOTE: unset params do not appear
    assert!(!request.contains("%24skip"));
    assert!(!request.contains("%24filter"));
}

#[test]
fn messages_delta_parses_changes_and_removals() {
    let body = r#"{
        "value": [
            {
                "id": "MSG1",
                "subject": "hello",
                "isRead": false,
                "receivedDateTime": "2026-08-06T10:00:00Z",
                "parentFolderId": "INBOX1",
                "from": { "emailAddress": { "name": "Alice", "address": "alice@example.com" } }
            },
            { "id": "MSG2", "@removed": { "reason": "deleted" } }
        ],
        "@odata.nextLink": "https://graph.microsoft.com/v1.0/me/messages/delta?$skiptoken=abc"
    }"#;

    let mut coroutine = MsgraphMessagesDelta::new(&auth(), "me", None, None).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("GET /v1.0/me/messages/delta"),
        "got: {request}"
    );

    assert_eq!(out.response.value.len(), 2);
    let changed = &out.response.value[0];
    assert_eq!(changed.message.id, "MSG1");
    assert_eq!(changed.message.subject.as_deref(), Some("hello"));
    assert_eq!(changed.message.is_read, Some(false));
    assert_eq!(
        changed.message.received_date_time.as_deref(),
        Some("2026-08-06T10:00:00Z")
    );
    assert_eq!(changed.message.parent_folder_id.as_deref(), Some("INBOX1"));
    assert_eq!(
        changed
            .message
            .from
            .as_ref()
            .and_then(|from| from.email_address.address.as_deref()),
        Some("alice@example.com")
    );
    assert!(changed.removed.is_none());

    let removed = &out.response.value[1];
    assert_eq!(removed.message.id, "MSG2");
    assert_eq!(removed.removed.as_ref().unwrap().reason, "deleted");

    assert_eq!(
        out.response.next_link.as_deref(),
        Some("https://graph.microsoft.com/v1.0/me/messages/delta?$skiptoken=abc")
    );
    assert!(out.response.delta_link.is_none());
}

#[test]
fn messages_delta_from_link_closes_round_with_delta_link() {
    let body = r#"{
        "value": [],
        "@odata.deltaLink": "https://graph.microsoft.com/v1.0/me/messages/delta?$deltatoken=xyz"
    }"#;

    let link = "https://graph.microsoft.com/v1.0/me/messages/delta?$skiptoken=abc";
    let mut coroutine = MsgraphMessagesDelta::from_link(&auth(), link).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.contains("/v1.0/me/messages/delta"),
        "got: {request}"
    );
    assert!(request.contains("skiptoken=abc"), "got: {request}");

    assert!(out.response.value.is_empty());
    assert!(out.response.next_link.is_none());
    assert_eq!(
        out.response.delta_link.as_deref(),
        Some("https://graph.microsoft.com/v1.0/me/messages/delta?$deltatoken=xyz")
    );
}

#[test]
fn messages_delta_builds_folder_scoped_url_with_select() {
    let mut coroutine =
        MsgraphMessagesDelta::new(&auth(), "me", Some("inbox"), Some("id,subject,isRead")).unwrap();
    let (result, written) = run(
        &mut coroutine,
        &json_response("HTTP/1.1 200 OK", r#"{ "value": [] }"#),
    );
    assert!(result.is_ok());

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("GET /v1.0/me/mailFolders/inbox/messages/delta"),
        "got: {request}"
    );
    assert!(
        request.contains("%24select=id%2Csubject%2CisRead"),
        "got: {request}"
    );
}

#[test]
fn messages_delta_addresses_explicit_user() {
    let mut coroutine = MsgraphMessagesDelta::new(&auth(), "USER1", None, None).unwrap();
    let (result, written) = run(
        &mut coroutine,
        &json_response("HTTP/1.1 200 OK", r#"{ "value": [] }"#),
    );
    assert!(result.is_ok());

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("GET /v1.0/users/USER1/messages/delta"),
        "got: {request}"
    );
}

#[test]
fn updates_message_patches_is_read() {
    let body = r#"{ "id": "ID1", "isRead": true }"#;

    let patch = MsgraphMessage {
        is_read: Some(true),
        ..Default::default()
    };
    let mut coroutine = MsgraphMessageUpdate::new(&auth(), "me", "ID1", &patch).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));

    assert_eq!(result.unwrap().response.is_read, Some(true));

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("PATCH /v1.0/me/messages/ID1"),
        "got: {request}"
    );
    assert!(request.contains("\"isRead\":true"), "got: {request}");
}

#[test]
fn move_posts_destination_id() {
    let body = r#"{ "id": "NEW" }"#;

    let mut coroutine = MsgraphMessageMove::new(&auth(), "me", "ID1", "archive").unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 201 Created", body));

    assert_eq!(result.unwrap().response.id, "NEW");

    let request = String::from_utf8_lossy(&written);
    assert!(request.contains("/v1.0/me/messages/ID1/move"));
    assert!(
        request.contains("\"destinationId\":\"archive\""),
        "got: {request}"
    );
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

    let mut coroutine = MsgraphMailSendMime::new(&auth(), "me", mime).unwrap();
    let (result, written) = run(&mut coroutine, &empty_response("HTTP/1.1 202 Accepted"));
    assert!(result.is_ok());

    let request = String::from_utf8_lossy(&written);
    assert!(request.contains("POST /v1.0/me/sendMail"));
    assert!(request.contains("Content-Type: text/plain"));
    // NOTE: base64 (standard) of the MIME, as Graph requires
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
fn parse_api_error_falls_back_on_missing_message_and_non_json() {
    let (status, code, message) =
        parse_api_error(404, br#"{ "error": { "code": "ErrorItemNotFound" } }"#);
    assert_eq!(status, 404);
    assert_eq!(code, "ErrorItemNotFound");
    assert_eq!(message, "unknown Microsoft Graph API error");

    let (status, code, message) = parse_api_error(502, b"upstream failure");
    assert_eq!(status, 502);
    assert_eq!(code, "unknown");
    assert_eq!(message, "upstream failure");
}

#[test]
fn updates_folder_patches_display_name() {
    let body = r#"{ "id": "AAA", "displayName": "renamed" }"#;

    let folder = MsgraphMailFolder {
        display_name: "renamed".into(),
        ..Default::default()
    };
    let mut coroutine = MsgraphMailFolderUpdate::new(&auth(), "me", "AAA", &folder).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));

    assert_eq!(result.unwrap().response.display_name, "renamed");

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("PATCH /v1.0/me/mailFolders/AAA"),
        "got: {request}"
    );
    assert!(
        request.contains("\"displayName\":\"renamed\""),
        "got: {request}"
    );
}

#[test]
fn create_mime_base64_encodes_to_messages() {
    let mime = b"From: a@b.c\r\nSubject: hi\r\n\r\nbody";
    let body = r#"{ "id": "DRAFT1", "isDraft": true }"#;

    let mut coroutine = MsgraphMessageCreateMime::new(&auth(), "me", None, mime).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 201 Created", body));

    assert_eq!(result.unwrap().response.id, "DRAFT1");

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("POST /v1.0/me/messages"),
        "got: {request}"
    );
    assert!(request.contains("Content-Type: text/plain"));
    let expected = "RnJvbTogYUBiLmMNClN1YmplY3Q6IGhpDQoNCmJvZHk=";
    assert!(request.contains(expected), "got: {request}");
}

#[test]
fn lists_message_attachments() {
    let body = r#"{
        "value": [
            { "id": "ATT1", "name": "report.pdf", "contentType": "application/pdf", "size": 1024 }
        ]
    }"#;

    let mut coroutine = MsgraphAttachmentsList::new(&auth(), "me", "ID1").unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("GET /v1.0/me/messages/ID1/attachments"),
        "got: {request}"
    );
    assert_eq!(out.response.value.len(), 1);
    assert_eq!(out.response.value[0].name.as_deref(), Some("report.pdf"));
    assert_eq!(out.response.value[0].size, Some(1024));
}

#[test]
fn attachment_get_raw_returns_bytes() {
    let content = b"%PDF-1.7 ...";

    let mut coroutine = MsgraphAttachmentGetRaw::new(&auth(), "me", "ID1", "ATT1").unwrap();
    let (result, written) = run(&mut coroutine, &text_response("HTTP/1.1 200 OK", content));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.contains("/v1.0/me/messages/ID1/attachments/ATT1/$value"),
        "got: {request}"
    );
    assert_eq!(out.response, content);
}

#[test]
fn deletes_attachment_issues_delete() {
    let mut coroutine = MsgraphAttachmentDelete::new(&auth(), "me", "ID1", "ATT1").unwrap();
    let (result, written) = run(&mut coroutine, &empty_response("HTTP/1.1 204 No Content"));

    result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("DELETE /v1.0/me/messages/ID1/attachments/ATT1"),
        "got: {request}"
    );
}

#[test]
fn contact_folders_list_parses_value() {
    let body = r#"{
        "value": [
            { "id": "AAA", "displayName": "Contacts" },
            { "id": "BBB", "displayName": "Work", "parentFolderId": "AAA" }
        ]
    }"#;

    let mut coroutine = MsgraphContactFoldersList::new(&auth(), "me", &Default::default()).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(request.starts_with("GET /v1.0/me/contactFolders"));

    assert_eq!(out.response.value.len(), 2);
    assert_eq!(out.response.value[0].id, "AAA");
    assert_eq!(out.response.value[0].display_name, "Contacts");
    assert_eq!(
        out.response.value[1].parent_folder_id.as_deref(),
        Some("AAA")
    );
}

#[test]
fn creates_contact_folder_posts_display_name() {
    let body = r#"{ "id": "AAA", "displayName": "friends" }"#;

    let folder = MsgraphContactFolder {
        display_name: "friends".into(),
        ..Default::default()
    };
    let mut coroutine = MsgraphContactFolderCreate::new(&auth(), "me", &folder).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 201 Created", body));

    assert_eq!(result.unwrap().response.id, "AAA");

    let request = String::from_utf8_lossy(&written);
    assert!(request.starts_with("POST /v1.0/me/contactFolders"));
    assert!(
        request.contains("\"displayName\":\"friends\""),
        "got: {request}"
    );
}

#[test]
fn rejects_empty_contact_folder_name() {
    let folder = MsgraphContactFolder {
        display_name: "  ".into(),
        ..Default::default()
    };
    let result = MsgraphContactFolderCreate::new(&auth(), "me", &folder);
    assert!(matches!(result, Err(MsgraphSendError::InvalidRequest(_))));
}

#[test]
fn deletes_contact_folder_issues_delete() {
    let mut coroutine = MsgraphContactFolderDelete::new(&auth(), "me", "AAA").unwrap();
    let (result, written) = run(&mut coroutine, &empty_response("HTTP/1.1 204 No Content"));

    result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("DELETE /v1.0/me/contactFolders/AAA"),
        "got: {request}"
    );
}

#[test]
fn contacts_list_builds_odata_query() {
    let params = MsgraphContactsListParams {
        top: Some(5),
        select: Some("id,displayName,emailAddresses"),
        orderby: Some("displayName"),
        ..Default::default()
    };

    let mut coroutine = MsgraphContactsList::new(&auth(), "me", Some("AAA"), &params).unwrap();
    let (result, written) = run(
        &mut coroutine,
        &json_response("HTTP/1.1 200 OK", r#"{ "value": [] }"#),
    );
    assert!(result.is_ok());

    let request = String::from_utf8_lossy(&written);
    assert!(request.contains("/v1.0/me/contactFolders/AAA/contacts"));
    assert!(request.contains("%24top=5"));
    assert!(request.contains("%24select=id%2CdisplayName%2CemailAddresses"));
    assert!(request.contains("%24orderby=displayName"));
    // NOTE: unset params do not appear
    assert!(!request.contains("%24skip"));
    assert!(!request.contains("%24filter"));
}

#[test]
fn contacts_delta_parses_changes_and_removals() {
    let body = r#"{
        "value": [
            { "id": "CT1", "givenName": "Alice" },
            { "id": "CT2", "@removed": { "reason": "deleted" } }
        ],
        "@odata.deltaLink": "https://graph.microsoft.com/v1.0/me/contacts/delta?$deltatoken=xyz"
    }"#;

    let mut coroutine = MsgraphContactsDelta::new(&auth(), "me", Some("AAA"), Some("id")).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));
    let out = result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.contains("/v1.0/me/contactFolders/AAA/contacts/delta"),
        "got: {request}"
    );
    assert!(request.contains("%24select=id"), "got: {request}");

    assert_eq!(out.response.value.len(), 2);
    assert_eq!(out.response.value[0].contact.id, "CT1");
    assert!(out.response.value[0].removed.is_none());
    assert_eq!(out.response.value[1].contact.id, "CT2");
    assert_eq!(
        out.response.value[1].removed.as_ref().unwrap().reason,
        "deleted"
    );
    assert_eq!(
        out.response.delta_link.as_deref(),
        Some("https://graph.microsoft.com/v1.0/me/contacts/delta?$deltatoken=xyz")
    );
}

#[test]
fn contacts_delta_from_link_requests_it_verbatim() {
    let link = "https://graph.microsoft.com/v1.0/me/contactFolders('AAA')/contacts/delta?$deltatoken=xyz";
    let mut coroutine = MsgraphContactsDelta::from_link(&auth(), link).unwrap();
    let (result, written) = run(
        &mut coroutine,
        &json_response("HTTP/1.1 200 OK", r#"{ "value": [] }"#),
    );
    assert!(result.is_ok());

    // NOTE: the link carries the folder, the select and the token, so
    // none of them are rebuilt.
    let request = String::from_utf8_lossy(&written);
    assert!(request.contains("/contacts/delta"), "got: {request}");
    assert!(request.contains("deltatoken=xyz"), "got: {request}");
}

#[test]
fn creates_contact_posts_email_addresses() {
    let body = r#"{ "id": "CT1", "givenName": "Alice" }"#;

    let contact = MsgraphContact {
        given_name: MsgraphField::Set("Alice".into()),
        email_addresses: MsgraphField::Set(vec![MsgraphEmailAddress {
            name: Some("Alice".into()),
            address: Some("alice@example.com".into()),
        }]),
        ..Default::default()
    };
    let mut coroutine = MsgraphContactCreate::new(&auth(), "me", None, &contact).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 201 Created", body));

    assert_eq!(result.unwrap().response.id, "CT1");

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("POST /v1.0/me/contacts"),
        "got: {request}"
    );
    assert!(
        request.contains("\"givenName\":\"Alice\""),
        "got: {request}"
    );
    assert!(
        request.contains(
            "\"emailAddresses\":[{\"name\":\"Alice\",\"address\":\"alice@example.com\"}]"
        ),
        "got: {request}"
    );
    // NOTE: unset fields do not appear in the JSON body
    assert!(!request.contains("\"surname\""), "got: {request}");
}

#[test]
fn updates_contact_patches_given_name() {
    let body = r#"{ "id": "CT1", "givenName": "Bob" }"#;

    let patch = MsgraphContact {
        given_name: MsgraphField::Set("Bob".into()),
        ..Default::default()
    };
    let mut coroutine = MsgraphContactUpdate::new(&auth(), "me", "CT1", &patch).unwrap();
    let (result, written) = run(&mut coroutine, &json_response("HTTP/1.1 200 OK", body));

    assert_eq!(result.unwrap().response.given_name.as_deref(), Some("Bob"));

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("PATCH /v1.0/me/contacts/CT1"),
        "got: {request}"
    );
    assert!(request.contains("\"givenName\":\"Bob\""), "got: {request}");
}

#[test]
fn deletes_contact_issues_delete() {
    let mut coroutine = MsgraphContactDelete::new(&auth(), "me", "CT1").unwrap();
    let (result, written) = run(&mut coroutine, &empty_response("HTTP/1.1 204 No Content"));

    result.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("DELETE /v1.0/me/contacts/CT1"),
        "got: {request}"
    );
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
