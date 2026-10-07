//! Email extraction: Internet mail (.eml) and Outlook messages (.msg).
//!
//! An email becomes one page: a deterministic header block — `From`, `To`,
//! `Cc`, `Date`, `Subject` — a blank line, the text/plain body (falling back
//! to naively de-tagged text/html), and finally one `Attachment: <filename>`
//! line per attachment. Attachments are listed, never extracted. The `Date`
//! line is the message's own `Date` header, verbatim, so downstream
//! validation can find the sent date in the document text - and finds only
//! that one: an email is filed under the day its sender sent it, in the
//! sender's own offset, and a second rendering of the same instant in UTC
//! falls on the next day for every evening email west of Greenwich.
//!
//! An Outlook `.msg` is the same page built from MAPI properties. A message
//! that travelled over SMTP keeps its transport headers, and their `Date`
//! header is used verbatim. One that never did - a draft, a sent item -
//! has only the submit time as a FILETIME, an instant with no zone, and is
//! dated in this machine's zone with its offset written out, which is the
//! day the person filing it sent it.

use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;

use jiff::tz::TimeZone;
use mailparse::{DispositionType, MailHeaderMap, ParsedMail};

use crate::extract::{
    CancellationToken, ExtractedDocument, ExtractedPage, ExtractionError, PageSource,
};
use crate::limits::ResourceLimits;

/// Header names emitted before the body, in this exact order.
const EMITTED_HEADERS: [&str; 5] = ["From", "To", "Cc", "Date", "Subject"];

/// How many `multipart/` content types a message may declare before it is
/// refused unparsed.
///
/// mailparse walks a MIME tree by recursion, so nesting depth is stack depth
/// on a two-megabyte extraction thread, and a few kilobytes of nested
/// boundaries overflow it — which aborts the worker process rather than
/// failing one document. Counting the declarations is a cheap upper bound on
/// the depth without parsing anything. Measured on this parser a
/// two-thousand-level message still parses, and nothing real comes near this
/// bound: a digest carrying a hundred forwarded messages declares about a
/// hundred, all of them siblings.
const MAX_MULTIPART_DECLARATIONS: usize = 256;

/// The largest decompressed RTF body this will ask for, and the largest
/// expansion it will believe.
///
/// Outlook stores an RTF body compressed, and the compressed body states its
/// own decompressed size in its header; the MS-OXRTFCP decompressor reserves
/// exactly that many bytes before it reads a byte of payload. A
/// three-kilobyte message claiming four gigabytes therefore aborts the worker
/// process on the allocation. A real body is prose, which this LZ77 variant
/// compresses by well under a factor of ten, and no email body approaches
/// sixty-four megabytes of text.
const MAX_RTF_DECOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RTF_EXPANSION: u64 = 64;

pub fn extract_eml(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    cancel.check()?;
    let bytes = read_bounded(path, limits, cancel)?;
    if declares_too_many_multiparts(&bytes) {
        return Err(ExtractionError::parse_failed(format!(
            "email declares more than {MAX_MULTIPART_DECLARATIONS} multipart content types"
        )));
    }
    let mail = mailparse::parse_mail(&bytes)
        .map_err(|error| ExtractionError::parse_failed(format!("email did not parse: {error}")))?;
    cancel.check()?;
    let text = render_email(&mail);
    Ok(ExtractedDocument {
        pages: vec![ExtractedPage::of_text(1, text, PageSource::Text)],
        warnings: vec![],
        truncated: false,
        optional_image: None,
        timings: None,
    })
}

/// Extracts an Outlook `.msg` file the way [`extract_eml`] extracts
/// Internet mail: the same header block, the plain-text body (or the HTML
/// body de-tagged, or the RTF body decompressed and de-tagged), and one
/// `Attachment:` line per attachment. A message with no transport `Date`
/// header is dated in this machine's time zone.
pub fn extract_msg(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<ExtractedDocument, ExtractionError> {
    extract_msg_in_zone(path, limits, cancel, &TimeZone::system())
}

/// [`extract_msg`], dating a message that has no transport `Date` header in
/// the given zone rather than the machine's.
pub fn extract_msg_in_zone(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
    zone: &TimeZone,
) -> Result<ExtractedDocument, ExtractionError> {
    cancel.check()?;
    let bytes = read_bounded(path, limits, cancel)?;
    let message = msg_parser::Outlook::from_slice(&bytes).map_err(|error| {
        ExtractionError::parse_failed(format!("Outlook message did not parse: {error}"))
    })?;
    cancel.check()?;
    let recovered =
        if blank(&message.subject) || blank(&message.body) || blank(&message.sender.name) {
            recover_string8_properties(&bytes, limits)
        } else {
            String8Properties::default()
        };
    cancel.check()?;
    Ok(ExtractedDocument {
        pages: vec![ExtractedPage::of_text(
            1,
            render_outlook(&message, &recovered, zone),
            PageSource::Text,
        )],
        warnings: vec![],
        truncated: false,
        optional_image: None,
        timings: None,
    })
}

fn render_outlook(
    message: &msg_parser::Outlook,
    recovered: &String8Properties,
    zone: &TimeZone,
) -> String {
    let person = |person: &msg_parser::Person| {
        let name = single_line(&person.name.to_string());
        let email = single_line(&person.email.to_string());
        match (name.is_empty(), email.is_empty()) {
            (false, false) => format!("{name} <{email}>"),
            (false, true) => name,
            (true, _) => email,
        }
    };
    let people = |people: &[msg_parser::Person]| {
        people
            .iter()
            .map(person)
            .filter(|entry| !entry.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut sender = person(&message.sender);
    if blank(&message.sender.name)
        && let Some(name) = &recovered.sender_name
    {
        let name = single_line(name);
        sender = if sender.is_empty() {
            name
        } else {
            format!("{name} <{sender}>")
        };
    }
    let subject = if blank(&message.subject) {
        recovered.subject.as_deref().unwrap_or_default()
    } else {
        message.subject.as_str()
    };
    let lines = [
        format!("From: {sender}"),
        format!("To: {}", people(&message.to)),
        format!("Cc: {}", people(&message.cc)),
        format!("Date: {}", outlook_date(message, zone)),
        format!("Subject: {}", single_line(subject)),
    ];
    let body = if !blank(&message.body) {
        message.body.clone()
    } else if let Some(body) = &recovered.body {
        body.clone()
    } else if !blank(&message.html) {
        html_to_text(&html_source(&message.html))
    } else if rtf_body_is_believable(&message.rtf_compressed) {
        message
            .html_from_rtf()
            .map(|html| html_to_text(&html))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let mut text = lines.join("\n");
    text.push_str("\n\n");
    text.push_str(trimmed(&body));
    text.push('\n');
    for attachment in &message.attachments {
        let name = [
            &attachment.long_file_name,
            &attachment.file_name,
            &attachment.display_name,
        ]
        .into_iter()
        .find(|value| !blank(value))
        .map(|value| single_line(value))
        .unwrap_or_default();
        if !name.is_empty() {
            text.push_str(&format!("Attachment: {name}\n"));
        }
    }
    text
}

/// The `Date` line for an Outlook message: the `Date` header it travelled
/// with, verbatim, or else the moment it was sent in `zone`.
fn outlook_date(message: &msg_parser::Outlook, zone: &TimeZone) -> String {
    // The transport headers are an RFC 5322 header block. Reading them as one
    // finds the `Date:` header itself; a search for the text "Date:" can land
    // on `Delivery-Date:` or `Resent-Date:` first.
    let transport = mailparse::parse_headers(message.headers.raw.as_bytes())
        .ok()
        .and_then(|(headers, _)| headers.get_first_value("Date"))
        .map(|value| single_line(&value))
        .filter(|value| !value.is_empty());
    if let Some(date) = transport {
        return date;
    }
    // The moment the sender pressed Send defines an email; delivery and
    // creation times stand in when a draft or an import has no submit time.
    let sent = [
        &message.client_submit_time,
        &message.message_delivery_time,
        &message.creation_time,
    ]
    .into_iter()
    .map(|value| value.trim())
    .find(|value| !value.is_empty())
    .unwrap_or_default();
    local_time(sent, zone)
}

/// msg_parser's `2025-03-02T21:30:00Z` as `2025-03-03 08:30:00 +11:00` in
/// Sydney: the calendar date the sender saw, standing on its own for the
/// date finder, with the offset that makes the instant unambiguous.
fn local_time(utc: &str, zone: &TimeZone) -> String {
    match utc.parse::<jiff::Timestamp>() {
        Ok(instant) => instant
            .to_zoned(zone.clone())
            .strftime("%Y-%m-%d %H:%M:%S %:z")
            .to_string(),
        Err(_) => utc.to_owned(),
    }
}

/// The HTML body as text. msg_parser hands a binary `PidTagHtml` stream
/// over hex-encoded, which is how Outlook normally stores it; the same
/// property stored as a string arrives as the HTML itself. Hex digits alone
/// never make an HTML document, so the two cannot be confused.
fn html_source(html: &str) -> String {
    match decode_hex(trimmed(html)) {
        Some(bytes) => decode_html_bytes(&bytes),
        None => html.to_owned(),
    }
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.is_empty() || text.len() % 2 != 0 {
        return None;
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| (byte as char).to_digit(16);
            Some((digit(pair[0])? * 16 + digit(pair[1])?) as u8)
        })
        .collect()
}

/// Decodes HTML bytes by byte-order mark, then as UTF-8, then by the
/// charset its `<meta>` tag declares, and finally as Windows-1252, the
/// encoding Outlook writes on the machines these messages come from.
fn decode_html_bytes(bytes: &[u8]) -> String {
    if let Some((encoding, mark)) = encoding_rs::Encoding::for_bom(bytes) {
        return encoding
            .decode_without_bom_handling(&bytes[mark..])
            .0
            .into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let encoding = declared_charset(bytes)
        // A document whose bytes were readable enough to find its own
        // meta tag in is not UTF-16, whatever the tag says.
        .filter(|encoding| *encoding != encoding_rs::UTF_16LE && *encoding != encoding_rs::UTF_16BE)
        .unwrap_or(encoding_rs::WINDOWS_1252);
    encoding.decode_without_bom_handling(bytes).0.into_owned()
}

/// The encoding a `<meta charset=...>` or `<meta http-equiv=... content="...;
/// charset=...">` tag declares, looked for in the head of the document.
fn declared_charset(bytes: &[u8]) -> Option<&'static encoding_rs::Encoding> {
    let head = &bytes[..bytes.len().min(64 * 1024)];
    let lower = head.to_ascii_lowercase();
    let mut cursor = 0;
    while let Some(found) = find_bytes(&lower[cursor..], b"<meta") {
        let start = cursor + found;
        let end = find_bytes(&lower[start..], b">").map_or(lower.len(), |end| start + end);
        let tag = &lower[start..end];
        if let Some(at) = find_bytes(tag, b"charset") {
            let label = tag[at + b"charset".len()..]
                .iter()
                .skip_while(|byte| byte.is_ascii_whitespace() || **byte == b'=')
                .skip_while(|byte| **byte == b'"' || **byte == b'\'')
                .take_while(|byte| byte.is_ascii_alphanumeric() || b"-_.:".contains(byte))
                .copied()
                .collect::<Vec<_>>();
            if let Some(encoding) = encoding_rs::Encoding::for_label(&label) {
                return Some(encoding);
            }
        }
        cursor = end;
    }
    None
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Header and body strings msg_parser dropped.
///
/// A message saved in Outlook's ANSI format stores its strings as
/// `PtypString8` - single-byte text in the message's code page - and
/// msg_parser reads those as UTF-8, discarding any that are not. A subject
/// with an `é` or a body with a `£` therefore vanishes. These are read back
/// here, decoded in the code page the message declares, and used only where
/// msg_parser came back empty.
#[derive(Default)]
struct String8Properties {
    subject: Option<String>,
    body: Option<String>,
    sender_name: Option<String>,
}

const PID_TAG_INTERNET_CODEPAGE: u32 = 0x3FDE_0003;
const PID_TAG_MESSAGE_CODEPAGE: u32 = 0x3FFD_0003;

fn recover_string8_properties(bytes: &[u8], limits: &ResourceLimits) -> String8Properties {
    let Ok(mut compound) = cfb::CompoundFile::open(Cursor::new(bytes)) else {
        return String8Properties::default();
    };
    let codepages = message_codepages(&mut compound, limits);
    // MS-OXCMSG: the message code page covers the message's own non-Unicode
    // strings, and the Internet code page the body; each stands in for the
    // other when only one is given.
    let strings = codepages
        .message
        .or(codepages.internet)
        .unwrap_or(encoding_rs::WINDOWS_1252);
    let body = codepages
        .internet
        .or(codepages.message)
        .unwrap_or(encoding_rs::WINDOWS_1252);
    let mut read = |property: &str, encoding: &'static encoding_rs::Encoding| {
        let raw = read_stream(
            &mut compound,
            &format!("/__substg1.0_{property}001E"),
            limits,
        )?;
        let raw = raw.split(|byte| *byte == 0).next().unwrap_or_default();
        let text = encoding.decode_without_bom_handling(raw).0.into_owned();
        (!text.trim().is_empty()).then_some(text)
    };
    String8Properties {
        subject: read("0037", strings),
        body: read("1000", body),
        // The sender's own name, or failing that the name the message was
        // sent on behalf of.
        sender_name: read("0C1A", strings).or_else(|| read("0042", strings)),
    }
}

#[derive(Default)]
struct Codepages {
    internet: Option<&'static encoding_rs::Encoding>,
    message: Option<&'static encoding_rs::Encoding>,
}

/// The code pages the top-level property stream declares. Its entries
/// follow a 32-byte header, sixteen bytes each: a tag (type in the low word,
/// id in the high word), flags, and an eight-byte value whose first four
/// bytes hold a 32-bit integer.
fn message_codepages<F: Read + std::io::Seek>(
    compound: &mut cfb::CompoundFile<F>,
    limits: &ResourceLimits,
) -> Codepages {
    let mut codepages = Codepages::default();
    let Some(properties) = read_stream(compound, "/__properties_version1.0", limits) else {
        return codepages;
    };
    for entry in properties.get(32..).unwrap_or_default().chunks_exact(16) {
        let tag = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]);
        let value = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]);
        let encoding = u16::try_from(value)
            .ok()
            .and_then(codepage::to_encoding_no_replacement);
        match tag {
            PID_TAG_INTERNET_CODEPAGE => codepages.internet = encoding,
            PID_TAG_MESSAGE_CODEPAGE => codepages.message = encoding,
            _ => {}
        }
    }
    codepages
}

fn read_stream<F: Read + std::io::Seek>(
    compound: &mut cfb::CompoundFile<F>,
    path: &str,
    limits: &ResourceLimits,
) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    compound
        .open_stream(path)
        .ok()?
        .take(limits.max_source_bytes)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(bytes)
}

/// Whether a compressed RTF body's own header claims a decompressed size
/// worth allocating for. A body that says nothing readable about its size is
/// not read either.
fn rtf_body_is_believable(compressed_hex: &str) -> bool {
    let compressed = (compressed_hex.len() / 2) as u64;
    match declared_rtf_size(compressed_hex) {
        Some(declared) => {
            declared <= MAX_RTF_DECOMPRESSED_BYTES
                && declared <= compressed.saturating_mul(MAX_RTF_EXPANSION)
        }
        None => false,
    }
}

/// The decompressed size a compressed RTF body declares. The MS-OXRTFCP
/// header is four little-endian 32-bit words — compressed size, decompressed
/// size, magic, CRC — and the MAPI property arrives hex-encoded.
fn declared_rtf_size(compressed_hex: &str) -> Option<u64> {
    let declared = compressed_hex.get(8..16)?;
    let mut size = 0_u64;
    for index in (0..8).step_by(2) {
        let byte = u8::from_str_radix(declared.get(index..index + 2)?, 16).ok()?;
        size |= u64::from(byte) << (4 * index);
    }
    Some(size)
}

fn read_bounded(
    path: &Path,
    limits: &ResourceLimits,
    cancel: &CancellationToken,
) -> Result<Vec<u8>, ExtractionError> {
    let metadata = std::fs::metadata(path).map_err(ExtractionError::io)?;
    limits.validate_source_size(metadata.len())?;
    let file = File::open(path).map_err(ExtractionError::io)?;
    let mut reader = BufReader::new(file).take(limits.max_source_bytes + 1);
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        cancel.check()?;
        let read = reader.read(&mut buffer).map_err(ExtractionError::io)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    limits.validate_source_size(bytes.len() as u64)?;
    Ok(bytes)
}

/// Counts `multipart/` in the raw message, stopping as soon as the bound is
/// passed so an oversized message costs no more than a bounded scan.
fn declares_too_many_multiparts(bytes: &[u8]) -> bool {
    const NEEDLE: &[u8] = b"multipart/";
    bytes
        .windows(NEEDLE.len())
        .filter(|window| window.eq_ignore_ascii_case(NEEDLE))
        .take(MAX_MULTIPART_DECLARATIONS + 1)
        .count()
        > MAX_MULTIPART_DECLARATIONS
}

fn render_email(mail: &ParsedMail) -> String {
    let mut lines = Vec::new();
    for name in EMITTED_HEADERS {
        if let Some(value) = mail.headers.get_first_value(name) {
            let value = single_line(&value);
            if !value.is_empty() {
                lines.push(format!("{name}: {value}"));
            }
        }
    }

    let mut parts = MessageParts::default();
    collect_parts(mail, &mut parts, 0);
    let body = match (parts.plain, parts.html) {
        (Some(plain), _) => plain,
        (None, Some(html)) => html_to_text(&html),
        (None, None) => String::new(),
    };
    let body = body.replace("\r\n", "\n").replace('\r', "\n");

    let mut text = lines.join("\n");
    text.push('\n');
    let body = body.trim_matches(['\n', '\r']).trim_end();
    if !body.is_empty() {
        text.push('\n');
        text.push_str(body);
        text.push('\n');
    }
    if !parts.attachments.is_empty() {
        text.push('\n');
        for attachment in &parts.attachments {
            text.push_str(&format!("Attachment: {}\n", single_line(attachment)));
        }
    }
    text
}

#[derive(Default)]
struct MessageParts {
    plain: Option<String>,
    html: Option<String>,
    attachments: Vec<String>,
}

/// Walks the (possibly nested) MIME tree depth-first, keeping the first
/// text/plain and first text/html bodies and listing every attachment.
///
/// This walk recurses too, so it stops at the same depth the message is
/// allowed to declare rather than trusting the tree it was handed.
fn collect_parts(part: &ParsedMail, parts: &mut MessageParts, depth: usize) {
    if depth > MAX_MULTIPART_DECLARATIONS {
        return;
    }
    let disposition = part.get_content_disposition();
    if disposition.disposition == DispositionType::Attachment {
        let filename = disposition
            .params
            .get("filename")
            .or_else(|| part.ctype.params.get("name"))
            .cloned()
            .unwrap_or_else(|| "(unnamed)".to_owned());
        parts.attachments.push(filename);
        return;
    }
    if part.ctype.mimetype.starts_with("multipart/") {
        for subpart in &part.subparts {
            collect_parts(subpart, parts, depth + 1);
        }
        return;
    }
    match part.ctype.mimetype.as_str() {
        "text/plain" if parts.plain.is_none() => parts.plain = part.get_body().ok(),
        "text/html" if parts.html.is_none() => parts.html = part.get_body().ok(),
        _ => {}
    }
}

/// Collapses a header value onto one line. Control characters count as
/// space: MAPI string properties end in the NUL terminator their stream
/// stores, and msg_parser keeps it.
fn single_line(value: &str) -> String {
    value
        .split(|character: char| character.is_whitespace() || character.is_control())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A MAPI string without the surrounding whitespace and the NUL terminator
/// its stream ends with.
fn trimmed(value: &str) -> &str {
    value.trim_matches(|character: char| character.is_whitespace() || character == '\0')
}

/// Whether a MAPI string says nothing: empty, whitespace, or only the NUL
/// terminator that a property written as an empty string still carries.
fn blank(value: &str) -> bool {
    trimmed(value).is_empty()
}

/// Naive HTML-to-text: drops comments, the document head (keeping its title
/// as the first line), and `<style>`/`<script>` blocks; reads the rest the
/// way it renders - whitespace collapsed, structural tags as line breaks,
/// table cells as ` | `-separated columns - strips every other tag, and
/// decodes the entities that turn up in prose.
///
/// A receipt's `<tr><td>Invoice date</td><td>March 3, 2025</td></tr>` must
/// come out as one line with the label and the value told apart, however
/// the source is indented, and a receipt laid out inside one big table cell
/// must keep the lines its paragraphs and `<br>`s give it.
fn html_to_text(html: &str) -> String {
    let without_comments = strip_comments(html);
    let title = element_text(&without_comments, "title");
    // The title is taken out with the head, or on its own when the head
    // has no end to take it out with, so it is not read a second time.
    let mut without_head = strip_head(&without_comments);
    if title.is_some() {
        without_head = strip_container(&without_head, "title");
    }
    let without_blocks = strip_container(&strip_container(&without_head, "script"), "style");
    let mut output = HtmlText::default();
    if let Some(title) = title {
        output.text(&title);
        output.line_break();
    }
    let mut rest = without_blocks.as_str();
    while let Some(open) = rest.find('<') {
        output.text(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('>') else {
            rest = "";
            break;
        };
        let closing = after.starts_with('/');
        let tag = after[..close]
            .trim_start_matches('/')
            .split([' ', '\t', '\n', '\r', '/'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        match tag.as_str() {
            "table" if closing => output.close_table(),
            "table" => output.open_table(),
            "tr" | "thead" | "tbody" | "tfoot" => output.row_edge(),
            "td" | "th" if closing => output.close_cell(),
            "td" | "th" => output.open_cell(),
            "pre" => {
                output.line_break();
                output.preformatted = if closing {
                    output.preformatted.saturating_sub(1)
                } else {
                    output.preformatted + 1
                };
            }
            "br" | "p" | "div" | "li" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote"
            | "hr" | "ul" | "ol" | "dl" | "dt" | "dd" => output.line_break(),
            _ => {}
        }
        rest = &after[close + 1..];
    }
    output.text(rest);
    collapse_blank_lines(&output.text)
}

/// One open table, and what its cells have held so far: that decides
/// whether a break inside it separates anything and where a ` | ` goes.
#[derive(Default)]
struct OpenTable {
    /// Whether any cell of the table has had text.
    has_text: bool,
    /// Whether any cell of the current row has had text.
    row_has_text: bool,
    in_cell: bool,
    /// Whether the open cell has had text.
    cell_has_text: bool,
}

/// An HTML body's text, built the way a browser lays it out.
///
/// HTML collapses every run of whitespace in its text to one space, and only
/// its structure starts a line. Outlook's HTML is indented and wrapped at
/// seventy-odd columns, and copying those newlines through split a label
/// from the value in the next cell, and a date across two lines. So
/// whitespace here is at most a space, and a line break is owed rather than
/// written: it is written when more text follows, two at most, and dropped
/// when nothing does. That is also what lets a table cell drop the breaks
/// its own first and last paragraph put around it - Word wraps every cell's
/// text in one - while keeping every break between two pieces of its text,
/// which is all a receipt laid out in one big cell has for lines.
#[derive(Default)]
struct HtmlText {
    text: String,
    tables: Vec<OpenTable>,
    /// Newlines owed before the next text: one for a line, two for a paragraph.
    breaks: usize,
    /// A ` | ` owed before the next text: a cell opened after one with text.
    separator: bool,
    /// A space owed before the next text.
    space: bool,
    /// Open `<pre>` elements; inside one, a newline in the text is a line.
    preformatted: usize,
}

impl HtmlText {
    /// Text between two tags, entities and all.
    ///
    /// What collapses is what HTML collapses - ASCII whitespace - plus
    /// control characters and the no-break space, which is how a spacer cell
    /// is written and which nobody's name or date is spelled with. A thin
    /// space in `1,250&#x2009;€` is a character the author chose, and stays.
    fn text(&mut self, raw: &str) {
        let decoded = decode_entities(raw);
        let mut word_start = None;
        for (index, character) in decoded.char_indices() {
            let line = character == '\n' && self.preformatted > 0;
            if line
                || character.is_ascii_whitespace()
                || character.is_control()
                || character == '\u{a0}'
            {
                if let Some(start) = word_start.take() {
                    self.word(&decoded[start..index]);
                }
                if line {
                    self.line_break();
                } else {
                    self.space = true;
                }
            } else if word_start.is_none() {
                word_start = Some(index);
            }
        }
        if let Some(start) = word_start {
            self.word(&decoded[start..]);
        }
    }

    /// Writes one word after whatever is owed before it: line breaks first,
    /// then a cell separator, then a space.
    fn word(&mut self, word: &str) {
        if !self.text.is_empty() {
            if self.breaks > 0 {
                self.text.extend(std::iter::repeat_n('\n', self.breaks));
            } else if self.separator {
                self.text.push_str(" | ");
            } else if self.space {
                self.text.push(' ');
            }
        }
        self.breaks = 0;
        self.separator = false;
        self.space = false;
        self.text.push_str(word);
        if let Some(table) = self.tables.last_mut() {
            table.has_text = true;
            if table.in_cell {
                table.row_has_text = true;
                table.cell_has_text = true;
            }
        }
    }

    /// Whether a break here would stand between two pieces of text. One
    /// before a cell's first text, or before a table's first row, would only
    /// push the cell off the line its row is on.
    fn breaks_here(&self) -> bool {
        match self.tables.last() {
            None => true,
            Some(table) if table.in_cell => table.cell_has_text,
            Some(table) => table.has_text,
        }
    }

    /// `<br>` or a block's edge. Two in a row are a paragraph.
    fn line_break(&mut self) {
        if self.breaks_here() {
            self.breaks = (self.breaks + 1).min(2);
        }
    }

    /// A table's or a row's edge, which starts a line but never a paragraph.
    fn row_break(&mut self) {
        if self.breaks_here() {
            self.breaks = self.breaks.max(1);
        }
    }

    fn open_table(&mut self) {
        self.row_break();
        self.tables.push(OpenTable::default());
    }

    /// `</table>`. A table left open runs to the end of the document, and
    /// its last cell with it, which costs nothing but the breaks around
    /// that cell's first and last text.
    fn close_table(&mut self) {
        self.close_cell();
        if let Some(closed) = self.tables.pop()
            && closed.has_text
            && let Some(table) = self.tables.last_mut()
        {
            // A nested table's text is text in the cell that holds it.
            table.has_text = true;
            if table.in_cell {
                table.row_has_text = true;
                table.cell_has_text = true;
            }
        }
        self.row_break();
    }

    /// `<tr>`, `</tr>`, or a row group's edge; each also ends a cell whose
    /// `</td>` was left out, as HTML allows.
    fn row_edge(&mut self) {
        self.close_cell();
        self.row_break();
        if let Some(table) = self.tables.last_mut() {
            table.row_has_text = false;
        }
    }

    fn open_cell(&mut self) {
        self.close_cell();
        if let Some(table) = self.tables.last_mut() {
            self.separator |= table.row_has_text;
            table.in_cell = true;
            table.cell_has_text = false;
        }
    }

    /// The breaks still owed when a cell with text ends came after its last
    /// text: they would only split it from the next cell.
    fn close_cell(&mut self) {
        if let Some(table) = self.tables.last_mut() {
            if table.in_cell && table.cell_has_text {
                self.breaks = 0;
            }
            table.in_cell = false;
        }
    }
}

/// Removes `<!-- ... -->` comments, an unterminated one to the end. Word's
/// HTML keeps whole XML documents inside conditional comments, and their
/// text otherwise leaks out as `Normal0falsefalse`.
fn strip_comments(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<!--") {
        result.push_str(&rest[..start]);
        rest = match rest[start + 4..].find("-->") {
            Some(end) => &rest[start + 4 + end + 3..],
            None => "",
        };
    }
    result.push_str(rest);
    result
}

/// Finds `<tag` or `</tag` as a whole tag name in lowercased HTML, so that
/// looking for `<head` does not stop at `<header>`.
fn find_tag(lower: &str, from: usize, tag: &str, closing: bool) -> Option<usize> {
    let needle = if closing {
        format!("</{tag}")
    } else {
        format!("<{tag}")
    };
    let mut cursor = from;
    while let Some(found) = lower.get(cursor..)?.find(&needle) {
        let start = cursor + found;
        let after = lower[start + needle.len()..].chars().next();
        if after.is_none_or(|character| {
            character == '>' || character == '/' || character.is_ascii_whitespace()
        }) {
            return Some(start);
        }
        cursor = start + needle.len();
    }
    None
}

/// The text between `<tag ...>` and `</tag>`, markup removed. An element
/// that is never closed has no text that can be told apart from the rest of
/// the document, so it has none.
fn element_text(html: &str, tag: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let open = find_tag(&lower, 0, tag, false)?;
    let content = open + lower[open..].find('>')? + 1;
    let close = find_tag(&lower, content, tag, true)?;
    let inner = &html[content..close];
    // Whatever markup the element held is not part of its text.
    Some(strip_tags(inner))
}

fn strip_tags(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        text.push_str(&rest[..open]);
        rest = match rest[open..].find('>') {
            Some(close) => &rest[open + close + 1..],
            None => "",
        };
    }
    text.push_str(rest);
    text
}

/// Removes the document head. HTML lets `</head>` be left out, and the head
/// then ends where the body starts. A head that ends nowhere is left in:
/// cutting it to the end of the document would take the message with it.
fn strip_head(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let Some(start) = find_tag(&lower, 0, "head", false) else {
        return html.to_owned();
    };
    let end = find_tag(&lower, start, "head", true)
        .and_then(|close| lower[close..].find('>').map(|offset| close + offset + 1))
        .or_else(|| find_tag(&lower, start, "body", false));
    match end {
        Some(end) => format!("{}{}", &html[..start], &html[end..]),
        None => html.to_owned(),
    }
}

/// Removes `<tag ...> ... </tag>` spans case-insensitively, including the tags.
fn strip_container(html: &str, tag: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut result = String::with_capacity(html.len());
    let mut cursor = 0;
    while let Some(start) = find_tag(&lower, cursor, tag, false) {
        result.push_str(&html[cursor..start]);
        cursor = match find_tag(&lower, start, tag, true) {
            Some(end) => match lower[end..].find('>') {
                Some(closing) => end + closing + 1,
                None => lower.len(),
            },
            None => lower.len(),
        };
    }
    result.push_str(&html[cursor..]);
    result
}

/// The named entities that turn up in the prose of real email: typographic
/// quotes and dashes, currency, legal marks, and the accented letters of the
/// names on invoices.
fn named_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => " ",
        "rsquo" => "\u{2019}",
        "lsquo" => "\u{2018}",
        "rdquo" => "\u{201D}",
        "ldquo" => "\u{201C}",
        "ndash" => "\u{2013}",
        "mdash" => "\u{2014}",
        "hellip" => "\u{2026}",
        "euro" => "\u{20AC}",
        "pound" => "\u{00A3}",
        "yen" => "\u{00A5}",
        "cent" => "\u{00A2}",
        "copy" => "\u{00A9}",
        "reg" => "\u{00AE}",
        "trade" => "\u{2122}",
        "sect" => "\u{00A7}",
        "para" => "\u{00B6}",
        "middot" => "\u{00B7}",
        "bull" => "\u{2022}",
        "laquo" => "\u{00AB}",
        "raquo" => "\u{00BB}",
        "times" => "\u{00D7}",
        "divide" => "\u{00F7}",
        "deg" => "\u{00B0}",
        "plusmn" => "\u{00B1}",
        "frac12" => "\u{00BD}",
        "eacute" => "\u{00E9}",
        "Eacute" => "\u{00C9}",
        "egrave" => "\u{00E8}",
        "uuml" => "\u{00FC}",
        "Uuml" => "\u{00DC}",
        "ouml" => "\u{00F6}",
        "Ouml" => "\u{00D6}",
        "auml" => "\u{00E4}",
        "Auml" => "\u{00C4}",
        "ccedil" => "\u{00E7}",
        "szlig" => "\u{00DF}",
        // A soft hyphen is an invisible line-break hint, not a character a
        // name or a date is spelled with.
        "shy" => "",
        _ => return None,
    })
}

fn decode_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(ampersand) = rest.find('&') {
        result.push_str(&rest[..ampersand]);
        let after = &rest[ampersand..];
        // An entity is short, so the semicolon is looked for a dozen
        // characters ahead — characters, not bytes: an accented word after an
        // ampersand would otherwise put the end of the window inside a
        // character and panic the slice.
        let entity_end = after
            .char_indices()
            .take(12)
            .find(|(_, character)| *character == ';')
            .map(|(index, _)| index);
        let Some(end) = entity_end else {
            result.push('&');
            rest = &after[1..];
            continue;
        };
        let entity = &after[1..end];
        let numeric = entity
            .strip_prefix("#x")
            .or_else(|| entity.strip_prefix("#X"))
            .map(|digits| u32::from_str_radix(digits, 16))
            .or_else(|| entity.strip_prefix('#').map(str::parse::<u32>))
            .and_then(Result::ok)
            .and_then(char::from_u32);
        match (numeric, named_entity(entity)) {
            (Some(character), _) => {
                result.push(character);
                rest = &after[end + 1..];
            }
            (None, Some(decoded)) => {
                result.push_str(decoded);
                rest = &after[end + 1..];
            }
            (None, None) => {
                result.push('&');
                rest = &after[1..];
            }
        }
    }
    result.push_str(rest);
    result
}

fn collapse_blank_lines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut blank_run = 0;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            blank_run += 1;
            continue;
        }
        if !result.is_empty() {
            result.push('\n');
            if blank_run > 0 {
                result.push('\n');
            }
        }
        blank_run = 0;
        result.push_str(line);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The entity scan looks a fixed distance past an ampersand for its
    /// semicolon. Measuring that distance in bytes cuts a multi-byte
    /// character in half, and slicing a string there panics — which, in an
    /// HTML body, is a document an accented word away from killing the
    /// extraction thread.
    #[test]
    fn entity_scan_window_does_not_split_a_multibyte_character() {
        assert_eq!(html_to_text("<p>Caf&éééééé</p>"), "Caf&éééééé");
        assert_eq!(
            html_to_text("<p>5 &lt; 6 ✓ &amp; 7 &gt; 6</p>"),
            "5 < 6 ✓ & 7 > 6"
        );
    }

    #[test]
    fn naive_html_detagging_breaks_on_structure_and_decodes_common_entities() {
        let html = "<html><style>p{color:red}</style><body>\
             <p>Dear&nbsp;Bob,</p><p>Fees &amp; taxes are &lt;due&gt;.</p>\
             <script>alert(1)</script></body></html>";
        assert_eq!(html_to_text(html), "Dear Bob,\n\nFees & taxes are <due>.");
    }

    /// A receipt's label and value sit in neighbouring cells. Run together
    /// they are `Invoice dateMarch 3, 2025`, a date nothing can find.
    #[test]
    fn table_cells_are_separated_and_rows_are_lines() {
        assert_eq!(
            html_to_text(
                "<table><tr><td>Invoice date</td><td>March 3, 2025</td></tr>\
                 <tr><th>Bill to</th><td>O&rsquo;Brien &amp; Co</td></tr></table>"
            ),
            "Invoice date | March 3, 2025\nBill to | O\u{2019}Brien & Co"
        );
        // Word wraps every cell's text in a paragraph.
        assert_eq!(
            html_to_text(
                "<table><tr><td><p class=MsoNormal>Invoice date<o:p></o:p></p></td>\
                 <td><p class=MsoNormal>March 3, 2025</p></td></tr></table>"
            ),
            "Invoice date | March 3, 2025"
        );
        // A layout table inside a cell still breaks its own rows.
        assert_eq!(
            html_to_text(
                "<table><tr><td><table><tr><td>Total</td><td>$40</td></tr>\
                 <tr><td>Due</td><td>May 1</td></tr></table></td></tr></table>"
            ),
            "Total | $40\nDue | May 1"
        );
    }

    /// Transactional email - receipts, statements, notifications - wraps its
    /// whole body in one layout cell. The lines its headings, paragraphs and
    /// `<br>`s give it are all it has for structure, and headings, labelled
    /// lines and the date index all read lines; only the breaks that would
    /// split one cell from the next are dropped.
    #[test]
    fn a_layout_cell_keeps_the_lines_inside_it() {
        assert_eq!(
            html_to_text(
                "<table><tr><td><table><tr><td><h1>Receipt from Acme Corporation</h1>\
                 <p>Invoice number: INV-1001</p><p>Date paid: March 3, 2025</p>\
                 <p>Billed to:<br>Juniper Ridge Holdings Inc.<br>12 Elm Street</p>\
                 <p>Thanks for your business.</p></td></tr></table></td></tr></table>"
            ),
            "Receipt from Acme Corporation\n\nInvoice number: INV-1001\n\n\
             Date paid: March 3, 2025\n\nBilled to:\nJuniper Ridge Holdings Inc.\n\
             12 Elm Street\n\nThanks for your business."
        );
        // A table left open runs to the end of the document, cell and all,
        // and its lines are still lines.
        assert_eq!(
            html_to_text(
                "<table><tr><td><p>Statement for March</p><p>Balance due: $40</p>\
                 <p>Due date: April 1, 2025</p>"
            ),
            "Statement for March\n\nBalance due: $40\n\nDue date: April 1, 2025"
        );
        // A `<br>` that ends a label cell does not push its value away.
        assert_eq!(
            html_to_text(
                "<table><tr><td>Invoice date<br></td><td><br>March 3, 2025</td></tr></table>"
            ),
            "Invoice date | March 3, 2025"
        );
    }

    /// HTML collapses whitespace in text, and only its structure starts a
    /// line. Outlook's HTML is indented and wrapped, and copying those
    /// newlines through put a label and its value - and the two halves of
    /// a date - on separate lines.
    #[test]
    fn source_indentation_and_wrapping_are_not_lines() {
        assert_eq!(
            html_to_text(
                "<table>\n  <tr>\n    <td>Invoice date</td>\n    <td>March 3, 2025</td>\n  </tr>\n\
                 \x20 <tr>\n    <td>Bill to</td>\n    <td>O&rsquo;Brien &amp; Co</td>\n  </tr>\n</table>"
            ),
            "Invoice date | March 3, 2025\nBill to | O\u{2019}Brien & Co"
        );
        // Word, which writes Outlook's HTML: each cell on lines of its own,
        // its text in a paragraph, and long text wrapped in the source.
        assert_eq!(
            html_to_text(
                "<table class=MsoNormalTable border=0 cellpadding=0>\r\n <tr>\r\n\
                 \x20 <td width=200 valign=top style='padding:0in 5.4pt'>\r\n\
                 \x20 <p class=MsoNormal>Invoice date<o:p></o:p></p>\r\n  </td>\r\n\
                 \x20 <td width=200 valign=top style='padding:0in 5.4pt'>\r\n\
                 \x20 <p class=MsoNormal>March 3,\r\n  2025<o:p></o:p></p>\r\n  </td>\r\n\
                 \x20</tr>\r\n</table>"
            ),
            "Invoice date | March 3, 2025"
        );
        assert_eq!(
            html_to_text("<p>The invoice for March\r\n2025 is attached.</p>"),
            "The invoice for March 2025 is attached."
        );
        // Spacer cells hold nothing to separate.
        assert_eq!(
            html_to_text(
                "<table><tr><td>Total</td><td>&nbsp;</td><td width=20></td><td>$40</td>\
                 <td>&nbsp;</td></tr></table>"
            ),
            "Total | $40"
        );
        // Preformatted text keeps the lines it was written with.
        assert_eq!(
            html_to_text("<p>Notes:</p><pre>Line one\n  Line two\n</pre><p>End.</p>"),
            "Notes:\n\nLine one\nLine two\n\nEnd."
        );
    }

    #[test]
    fn hex_and_named_entities_decode() {
        assert_eq!(
            decode_entities("&#x2019;&#X201C;&#8212;&euro;5 &pound;3 caf&eacute; co&shy;operate"),
            "\u{2019}\u{201C}\u{2014}\u{20AC}5 \u{00A3}3 caf\u{00E9} cooperate"
        );
        // Not an entity, and not a character: left as written.
        assert_eq!(
            decode_entities("&#xD800; &bogus; R&D"),
            "&#xD800; &bogus; R&D"
        );
    }

    /// Word-generated HTML carries its settings as XML inside conditional
    /// comments, and its head holds style sheets and metadata; neither is
    /// the message. The title is kept, as the first line.
    #[test]
    fn comments_and_the_head_are_dropped_but_the_title_is_kept() {
        let html = "<html><head><title>Statement &ndash; March</title>\
            <meta name=Generator content=\"Microsoft Word 15\">\
            <!--[if gte mso 9]><xml><w:WordDocument><w:View>Normal</w:View>\
            <w:Zoom>0</w:Zoom></w:WordDocument></xml><![endif]--></head>\
            <body><header>Juniper Ridge</header><p>Balance due.</p></body></html>";
        assert_eq!(
            html_to_text(html),
            "Statement \u{2013} March\nJuniper Ridge\nBalance due."
        );
        assert_eq!(html_to_text("<p>a</p><!-- never closed <p>b</p>"), "a");
    }

    /// HTML lets a document leave `</head>` out. Read as running to the end
    /// of the document, that head would take the whole message with it.
    #[test]
    fn a_head_left_open_ends_where_the_body_starts() {
        assert_eq!(
            html_to_text(
                "<html><head><title>Receipt</title><meta charset=utf-8>\
                 <body><p>Paid March 3, 2025.</p></body></html>"
            ),
            "Receipt\n\nPaid March 3, 2025."
        );
        assert_eq!(
            html_to_text("<head><title>Receipt</title><p>Paid in full.</p>"),
            "Receipt\n\nPaid in full."
        );
        // A title that is never closed is not a title.
        assert_eq!(
            html_to_text("<title>Statement<p>Balance due.</p>"),
            "Statement\nBalance due."
        );
    }

    /// MAPI strings end with the NUL their stream stores, and msg_parser
    /// keeps it. It must not end up inside an address or make an empty
    /// property look written.
    #[test]
    fn a_nul_terminator_is_neither_text_nor_content() {
        assert_eq!(single_line("renee@example.com\0"), "renee@example.com");
        assert_eq!(single_line(" Ren\u{e9}e\tDubois\0"), "Ren\u{e9}e Dubois");
        assert!(blank("\0"));
        assert!(blank(" \r\n\0"));
        assert!(!blank("Caf\u{e9}\0"));
        assert_eq!(trimmed("Body text.\r\n\0"), "Body text.");
    }

    #[test]
    fn hex_html_is_decoded_and_real_html_is_left_alone() {
        assert_eq!(html_source("3c703e48693c2f703e"), "<p>Hi</p>");
        assert_eq!(html_source("<p>Hi</p>"), "<p>Hi</p>");
        assert_eq!(html_source("abc"), "abc");
    }

    /// Bytes that are not UTF-8 are read in the charset the document
    /// declares, and Windows-1252 when it declares none.
    #[test]
    fn html_bytes_decode_by_declared_charset_then_windows_1252() {
        let mut koi8 =
            b"<meta http-equiv=Content-Type content=\"text/html; charset=koi8-r\"><p>".to_vec();
        koi8.extend_from_slice(&[0xF0, 0xD2, 0xC9, 0xD7, 0xC5, 0xD4]);
        assert!(decode_html_bytes(&koi8).ends_with("<p>Привет"));

        assert_eq!(decode_html_bytes(b"<p>Caf\xE9 \xA3"), "<p>Café £");
        assert_eq!(decode_html_bytes("<p>Café".as_bytes()), "<p>Café");
    }

    #[test]
    fn a_submit_time_is_written_in_the_given_zone_with_its_offset() {
        let sydney = TimeZone::fixed(jiff::tz::offset(11));
        assert_eq!(
            local_time("2025-03-02T21:30:00Z", &sydney),
            "2025-03-03 08:30:00 +11:00"
        );
        assert_eq!(
            local_time("2025-03-02T21:30:00.250Z", &TimeZone::UTC),
            "2025-03-02 21:30:00 +00:00"
        );
        assert_eq!(local_time("not a time", &TimeZone::UTC), "not a time");
    }
}
