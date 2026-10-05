//! An Outlook message is a compound file of MAPI property streams. One is
//! built here from the specification rather than copied from a mailbox, so
//! the test carries no real message and stays byte-deterministic.

use std::io::Write;

use intern_worker::email::{extract_msg, extract_msg_in_zone};
use intern_worker::extract::CancellationToken;
use intern_worker::limits::ResourceLimits;
use jiff::tz::{self, TimeZone};
use tempfile::tempdir;

/// 2026-03-04T15:22:10Z as a FILETIME: 100-nanosecond ticks since 1601.
const SUBMIT_TIME: u64 = 134_171_113_300_000_000;

fn utf16(value: &str) -> Vec<u8> {
    value.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

/// A `__properties_version1.0` stream: the header for its storage kind,
/// then 16-byte entries of tag (type, id), flags, and value.
fn properties(root: bool, entries: &[(u16, u16, [u8; 8])]) -> Vec<u8> {
    let mut bytes = vec![0u8; if root { 32 } else { 8 }];
    for (kind, id, value) in entries {
        bytes.extend_from_slice(&kind.to_le_bytes());
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&6u32.to_le_bytes());
        bytes.extend_from_slice(value);
    }
    bytes
}

fn long(value: u32) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[..4].copy_from_slice(&value.to_le_bytes());
    bytes
}

fn write_stream<F: std::io::Read + std::io::Write + std::io::Seek>(
    file: &mut cfb::CompoundFile<F>,
    path: &str,
    bytes: &[u8],
) {
    let mut stream = file.create_stream(path).unwrap();
    stream.write_all(bytes).unwrap();
}

fn forwarded_invoice(path: &std::path::Path) {
    // Outlook writes version 3 compound files (512-byte sectors), and that
    // is the version msg_parser reads.
    let handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    let mut file = cfb::CompoundFile::create_with_version(cfb::Version::V3, handle).unwrap();
    write_stream(
        &mut file,
        "/__substg1.0_0037001F",
        &utf16("FW: Invoice INV-7741 for January"),
    );
    write_stream(&mut file, "/__substg1.0_0C1A001F", &utf16("Dana Ruiz"));
    write_stream(
        &mut file,
        "/__substg1.0_5D01001F",
        &utf16("dana.ruiz@ridgeline.example"),
    );
    write_stream(
        &mut file,
        "/__substg1.0_0E04001F",
        &utf16("Priya Nandakumar"),
    );
    write_stream(
        &mut file,
        "/__substg1.0_1000001F",
        &utf16(
            "Priya,\r\n\r\nForwarding the January invoice from Acme Corporation, $1,248.00, due \
             February 4, 2026. Please file it with the Contoso Worldwide, Inc. engagement.\r\n\r\nDana",
        ),
    );
    write_stream(
        &mut file,
        "/__properties_version1.0",
        &properties(
            true,
            &[
                (0x0040, 0x0039, SUBMIT_TIME.to_le_bytes()),
                (0x0040, 0x0E06, SUBMIT_TIME.to_le_bytes()),
            ],
        ),
    );
    file.create_storage("/__recip_version1.0_#00000000")
        .unwrap();
    write_stream(
        &mut file,
        "/__recip_version1.0_#00000000/__substg1.0_3001001F",
        &utf16("Priya Nandakumar"),
    );
    write_stream(
        &mut file,
        "/__recip_version1.0_#00000000/__substg1.0_39FE001F",
        &utf16("priya@contoso.example"),
    );
    write_stream(
        &mut file,
        "/__recip_version1.0_#00000000/__properties_version1.0",
        &properties(false, &[(0x0003, 0x0C15, long(1))]),
    );
    file.create_storage("/__recip_version1.0_#00000001")
        .unwrap();
    write_stream(
        &mut file,
        "/__recip_version1.0_#00000001/__substg1.0_3001001F",
        &utf16("Marcus Reyes"),
    );
    write_stream(
        &mut file,
        "/__recip_version1.0_#00000001/__substg1.0_39FE001F",
        &utf16("marcus@reyestolliver.example"),
    );
    write_stream(
        &mut file,
        "/__recip_version1.0_#00000001/__properties_version1.0",
        &properties(false, &[(0x0003, 0x0C15, long(2))]),
    );
    file.create_storage("/__attach_version1.0_#00000000")
        .unwrap();
    write_stream(
        &mut file,
        "/__attach_version1.0_#00000000/__substg1.0_3707001F",
        &utf16("INV-7741.pdf"),
    );
    write_stream(
        &mut file,
        "/__attach_version1.0_#00000000/__properties_version1.0",
        &properties(false, &[(0x0003, 0x3705, long(1))]),
    );
    file.flush().unwrap();
}

#[test]
fn an_outlook_message_becomes_the_same_page_an_eml_would() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("forwarded-invoice.msg");
    forwarded_invoice(&path);

    let extracted = extract_msg_in_zone(
        &path,
        &ResourceLimits::default(),
        &CancellationToken::new(),
        &TimeZone::UTC,
    )
    .unwrap();
    assert_eq!(extracted.pages.len(), 1);
    let text = &extracted.pages[0].text;
    let header = text.split("\n\n").next().unwrap_or_default();
    assert_eq!(
        header,
        "From: Dana Ruiz <dana.ruiz@ridgeline.example>\n\
         To: Priya Nandakumar <priya@contoso.example>\n\
         Cc: Marcus Reyes <marcus@reyestolliver.example>\n\
         Date: 2026-03-04 15:22:10 +00:00\n\
         Subject: FW: Invoice INV-7741 for January",
        "{text}"
    );
    assert!(
        text.contains("Forwarding the January invoice from Acme Corporation"),
        "{text}"
    );
    assert!(text.ends_with("Attachment: INV-7741.pdf\n"), "{text}");
}

#[test]
fn a_file_that_is_not_a_compound_file_is_a_parse_failure_not_a_crash() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("not-really.msg");
    std::fs::write(
        &path,
        b"Subject: this is plain text pretending to be Outlook\r\n\r\nHello",
    )
    .unwrap();
    let error =
        extract_msg(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap_err();
    assert_eq!(error.code(), "PARSE_FAILED");
}

/// An `LZFu` compressed-RTF header, its literal-coded payload, and whatever
/// decompressed size it cares to claim. The MS-OXRTFCP decompressor reserves
/// that claimed size before it reads a byte of the payload.
fn compressed_rtf(rtf: &str, declared_raw_size: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    for chunk in rtf.as_bytes().chunks(8) {
        payload.push(0x00);
        payload.extend_from_slice(chunk);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&((payload.len() + 12) as u32).to_le_bytes());
    bytes.extend_from_slice(&declared_raw_size.to_le_bytes());
    bytes.extend_from_slice(&0x7546_5A4C_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&payload);
    bytes
}

/// A message whose only body is RTF, compressed, declaring `declared_raw_size`
/// bytes once decompressed.
fn rtf_only_message(path: &std::path::Path, declared_raw_size: u32) {
    let handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    let mut file = cfb::CompoundFile::create_with_version(cfb::Version::V3, handle).unwrap();
    write_stream(
        &mut file,
        "/__substg1.0_0037001F",
        &utf16("Ledger for March"),
    );
    write_stream(
        &mut file,
        "/__substg1.0_10090102",
        &compressed_rtf(
            r"{\rtf1\fromhtml1 {\*\htmltag <p>The March ledger is attached.</p>}}",
            declared_raw_size,
        ),
    );
    write_stream(
        &mut file,
        "/__properties_version1.0",
        &properties(true, &[]),
    );
    file.flush().unwrap();
}

#[test]
fn an_rtf_body_declaring_four_gigabytes_is_ignored() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("rtf-bomb.msg");
    rtf_only_message(&path, u32::MAX);

    let extracted =
        extract_msg(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    let text = &extracted.pages[0].text;

    assert!(!text.contains("The March ledger is attached."), "{text}");
    assert!(text.contains("Subject: Ledger for March"), "{text}");
}

#[test]
fn an_honestly_sized_rtf_body_is_still_read() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("rtf-body.msg");
    rtf_only_message(&path, 66);

    let extracted =
        extract_msg(&path, &ResourceLimits::default(), &CancellationToken::new()).unwrap();
    let text = &extracted.pages[0].text;

    assert!(text.contains("The March ledger is attached."), "{text}");
}

/// A message built from top-level property streams and a property stream
/// carrying the given fixed-size properties.
fn message(path: &std::path::Path, streams: &[(&str, Vec<u8>)], fixed: &[(u16, u16, [u8; 8])]) {
    let handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    let mut file = cfb::CompoundFile::create_with_version(cfb::Version::V3, handle).unwrap();
    for (name, bytes) in streams {
        write_stream(&mut file, &format!("/__substg1.0_{name}"), bytes);
    }
    write_stream(
        &mut file,
        "/__properties_version1.0",
        &properties(true, fixed),
    );
    file.flush().unwrap();
}

/// 2025-03-02T21:30:00Z: 08:30 on 3 March in Sydney.
const SYDNEY_MORNING: u64 = 133_854_246_000_000_000;

fn extract_in(path: &std::path::Path, zone: &TimeZone) -> String {
    extract_msg_in_zone(
        path,
        &ResourceLimits::default(),
        &CancellationToken::new(),
        zone,
    )
    .unwrap()
    .pages
    .remove(0)
    .text
}

/// Archiving and eDiscovery tools write messages whose only body is HTML.
/// Outlook stores that HTML as a binary property, which msg_parser hands
/// over as hex digits, and the whole body used to arrive as kilobytes of
/// `3c68746d6c3e...`.
#[test]
fn html_only_msg_body_is_decoded() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("receipt.msg");
    let mut html = b"<html><head><meta http-equiv=\"Content-Type\" \
        content=\"text/html; charset=windows-1252\"><title>Receipt</title></head><body>\
        <p>Receipt for Caf\xE9 M\xFCller</p>\
        <table><tr><td>Invoice date</td><td>March 3, 2025</td></tr></table>"
        .to_vec();
    html.extend_from_slice(b"</body></html>");
    message(
        &path,
        &[("0037001F", utf16("Your receipt")), ("10130102", html)],
        &[(0x0040, 0x0039, SYDNEY_MORNING.to_le_bytes())],
    );

    let text = extract_in(&path, &TimeZone::UTC);

    assert!(text.contains("Receipt for Caf\u{e9} M\u{fc}ller"), "{text}");
    assert!(text.contains("Invoice date | March 3, 2025"), "{text}");
    assert!(!text.contains("3c68746d6c"), "{text}");
    assert!(!text.contains('<'), "{text}");
}

/// Outlook writes its HTML through Word: each cell on lines of its own, its
/// text in a paragraph, all of it indented. Copied through as written, the
/// label and the value came out on separate lines with a stray `|` between
/// them.
#[test]
fn an_outlook_formatted_table_keeps_label_and_value_together() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("outlook-table.msg");
    let html = b"<html>\r\n<body lang=EN-US>\r\n<div class=WordSection1>\r\n\
        <table class=MsoNormalTable border=0 cellspacing=0 cellpadding=0>\r\n <tr>\r\n\
        \x20 <td width=200 valign=top style='padding:0in 5.4pt 0in 5.4pt'>\r\n\
        \x20 <p class=MsoNormal>Invoice date<o:p></o:p></p>\r\n  </td>\r\n\
        \x20 <td width=200 valign=top style='padding:0in 5.4pt 0in 5.4pt'>\r\n\
        \x20 <p class=MsoNormal>March 3, 2025<o:p></o:p></p>\r\n  </td>\r\n </tr>\r\n\
        </table>\r\n</div>\r\n</body>\r\n</html>\r\n"
        .to_vec();
    message(
        &path,
        &[("0037001F", utf16("Invoice")), ("10130102", html)],
        &[],
    );

    let text = extract_in(&path, &TimeZone::UTC);

    assert!(text.contains("\nInvoice date | March 3, 2025\n"), "{text}");
    assert!(!text.contains("\n|"), "{text}");
}

/// The same property stored as a string is the HTML itself, and must not
/// be mistaken for hex.
#[test]
fn an_html_string_property_is_used_as_written() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("string-html.msg");
    message(
        &path,
        &[(
            "1013001F",
            utf16("<p>Engagement confirmed for <b>Juniper Ridge</b>.</p>"),
        )],
        &[],
    );

    let text = extract_in(&path, &TimeZone::UTC);

    assert!(
        text.contains("Engagement confirmed for Juniper Ridge."),
        "{text}"
    );
}

/// Outlook's ANSI format stores strings in the message's code page, and
/// msg_parser drops any that are not UTF-8: the subject, sender and body of
/// a message about a café, or a fee in pounds, simply vanished.
#[test]
fn ansi_msg_strings_are_recovered() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("ansi.msg");
    message(
        &path,
        &[
            ("0037001E", b"Caf\xE9 contract\0".to_vec()),
            ("0C1A001E", b"Ren\xE9e Dubois\0".to_vec()),
            ("0C1F001E", b"renee@example.com\0".to_vec()),
            (
                "1000001E",
                b"The fee is \xA3500, payable to Caf\xE9 Ltd.\0".to_vec(),
            ),
        ],
        &[
            (0x0003, 0x3FFD, long(1252)),
            (0x0003, 0x3FDE, long(1252)),
            (0x0040, 0x0039, SYDNEY_MORNING.to_le_bytes()),
        ],
    );

    let text = extract_in(&path, &TimeZone::UTC);

    assert!(text.contains("Subject: Caf\u{e9} contract"), "{text}");
    assert!(
        text.contains("From: Ren\u{e9}e Dubois <renee@example.com>"),
        "{text}"
    );
    assert!(
        text.contains("The fee is \u{a3}500, payable to Caf\u{e9} Ltd."),
        "{text}"
    );
}

/// The code page the message declares is the one its strings are read in.
#[test]
fn ansi_strings_are_read_in_the_declared_code_page() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("cyrillic.msg");
    message(
        &path,
        // "Договор" in Windows-1251.
        &[("0037001E", b"\xC4\xEE\xE3\xEE\xE2\xEE\xF0\0".to_vec())],
        &[(0x0003, 0x3FFD, long(1251))],
    );

    let text = extract_in(&path, &TimeZone::UTC);

    assert!(
        text.contains("Subject: \u{414}\u{43e}\u{433}\u{43e}\u{432}\u{43e}\u{440}"),
        "{text}"
    );
}

/// A message that travelled over SMTP carries the `Date` header its sender's
/// client wrote, in the sender's offset, and that is the line kept. One that
/// never did is dated in this machine's zone: a Sydney sent item at 08:30
/// on 3 March is 21:30 on 2 March in UTC, and naming it after the UTC day
/// files it a day early.
#[test]
fn msg_date_uses_transport_header_or_local_offset() {
    let directory = tempdir().unwrap();
    let sydney = TimeZone::fixed(tz::offset(11));

    let travelled = directory.path().join("received.msg");
    message(
        &travelled,
        &[
            ("0037001F", utf16("Signed engagement letter")),
            (
                "007D001F",
                utf16(
                    "Received: from mx.example.com by mail.example.com\r\n\
                     Delivery-Date: Sun, 2 Mar 2025 21:31:02 +0000\r\n\
                     Date: Mon, 3 Mar 2025 08:30:00 +1100\r\n\
                     Subject: Signed engagement letter\r\n\r\n",
                ),
            ),
        ],
        &[(0x0040, 0x0039, SYDNEY_MORNING.to_le_bytes())],
    );
    let text = extract_in(&travelled, &TimeZone::UTC);
    assert!(
        text.contains("\nDate: Mon, 3 Mar 2025 08:30:00 +1100\n"),
        "{text}"
    );
    assert!(!text.contains("2025-03-02"), "{text}");

    let sent_item = directory.path().join("sent-item.msg");
    message(
        &sent_item,
        &[("0037001F", utf16("Signed engagement letter"))],
        &[(0x0040, 0x0039, SYDNEY_MORNING.to_le_bytes())],
    );
    let text = extract_in(&sent_item, &sydney);
    assert!(
        text.contains("\nDate: 2025-03-03 08:30:00 +11:00\n"),
        "{text}"
    );
    assert!(!text.contains("Sent:"), "{text}");
    assert!(!text.contains("2025-03-02"), "{text}");

    // The public entry point dates in the machine's own zone.
    let local = extract_msg(
        &sent_item,
        &ResourceLimits::default(),
        &CancellationToken::new(),
    )
    .unwrap()
    .pages
    .remove(0)
    .text;
    let expected = jiff::Timestamp::from_second(1_740_951_000)
        .unwrap()
        .to_zoned(TimeZone::system())
        .strftime("%Y-%m-%d %H:%M:%S %:z")
        .to_string();
    assert!(local.contains(&format!("\nDate: {expected}\n")), "{local}");
}
