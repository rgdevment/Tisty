use super::{ASKED, read};

const LISTED: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns">
  <d:response>
    <d:href>/dav/Tisty/</d:href>
    <d:propstat>
      <d:prop><d:resourcetype><d:collection/></d:resourcetype><d:quota-available-bytes>2048</d:quota-available-bytes></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/Tisty/tisty.toml</d:href>
    <d:propstat>
      <d:prop><d:resourcetype/><d:getcontentlength>12</d:getcontentlength><d:getetag>"e1"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
    <d:propstat>
      <d:prop><d:quota-available-bytes/></d:prop>
      <d:status>HTTP/1.1 404 Not Found</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

#[test]
fn each_response_reads_as_what_its_server_answered_200_for() {
    let found = read(LISTED).unwrap();

    assert_eq!(found.len(), 2);
    assert!(found[0].collection);
    assert_eq!(found[0].room, Some(2048));
    assert_eq!(found[1].href, "/dav/Tisty/tisty.toml");
    assert!(!found[1].collection);
    assert_eq!(found[1].bytes, Some(12));
    assert_eq!(found[1].revision(), "\"e1\"");
    assert_eq!(found[1].room, None);
}

#[test]
fn any_prefix_reads_the_same_since_only_the_dav_namespace_counts() {
    let said = r#"<multistatus xmlns="DAV:"><response><href>/a</href><propstat><prop><getetag>"x"</getetag></prop><status>HTTP/1.1 200 OK</status></propstat></response></multistatus>"#;

    let found = read(said).unwrap();

    assert_eq!(found[0].etag.as_deref(), Some("\"x\""));
}

#[test]
fn a_server_with_no_etag_still_gives_a_revision_from_size_and_date() {
    let said = r#"<d:multistatus xmlns:d="DAV:"><d:response><d:href>/a</d:href><d:propstat><d:prop><d:getcontentlength>3</d:getcontentlength><d:getlastmodified>Tue, 20 Oct 2026 10:00:00 GMT</d:getlastmodified></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response></d:multistatus>"#;

    let found = read(said).unwrap();

    assert_eq!(
        found[0].revision(),
        "modified:Tue, 20 Oct 2026 10:00:00 GMT:3"
    );
}

#[test]
fn what_is_not_a_multistatus_is_refused() {
    assert!(read("<html><body>login</body></html>").is_err());
    assert!(read("not xml").is_err());
}

#[test]
fn the_question_asks_only_what_tells_a_file_apart_and_the_room_left() {
    assert!(ASKED.contains("getetag"));
    assert!(ASKED.contains("quota-available-bytes"));
    assert!(!ASKED.contains("owner"));
}
