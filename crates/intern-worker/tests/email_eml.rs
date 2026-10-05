use std::path::PathBuf;

use intern_worker::email::extract_eml;
use intern_worker::extract::{CancellationToken, ExtractedDocument, ExtractionError, PageSource};
use intern_worker::limits::ResourceLimits;
use tempfile::TempDir;

fn write_eml(directory: &TempDir, bytes: &[u8]) -> PathBuf {
    let path = directory.path().join("message.eml");
    std::fs::write(&path, bytes).unwrap();
    path
}

fn extract(bytes: &[u8]) -> Result<ExtractedDocument, ExtractionError> {
    let directory = tempfile::tempdir().unwrap();
    let path = write_eml(&directory, bytes);
    extract_eml(&path, &ResourceLimits::default(), &CancellationToken::new())
}

const NESTED_MULTIPART: &[u8] = b"From: Alice Example <alice@example.com>\r\n\
To: Bob <bob@example.com>\r\n\
Cc: carol@example.com\r\n\
Date: Thu, 21 Aug 2025 09:15:00 -0400\r\n\
Subject: Q3 invoice attached\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"outer\"\r\n\
\r\n\
--outer\r\n\
Content-Type: multipart/alternative; boundary=\"inner\"\r\n\
\r\n\
--inner\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Hi Bob,\r\n\
\r\n\
The Q3 invoice is attached.\r\n\
--inner\r\n\
Content-Type: text/html\r\n\
\r\n\
<p>Hi Bob,</p>\r\n\
--inner--\r\n\
--outer\r\n\
Content-Type: application/pdf; name=\"invoice.pdf\"\r\n\
Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
JVBERi0=\r\n\
--outer--\r\n";

#[test]
fn the_header_block_is_emitted_in_fixed_order_before_the_body() {
    let document = extract(NESTED_MULTIPART).unwrap();

    assert_eq!(document.pages.len(), 1);
    assert_eq!(document.pages[0].page_number, 1);
    assert_eq!(document.pages[0].source, PageSource::Text);
    assert!(document.warnings.is_empty());
    assert_eq!(
        document.pages[0].text,
        "From: Alice Example <alice@example.com>\n\
         To: Bob <bob@example.com>\n\
         Cc: carol@example.com\n\
         Date: Thu, 21 Aug 2025 09:15:00 -0400\n\
         Subject: Q3 invoice attached\n\
         \n\
         Hi Bob,\n\
         \n\
         The Q3 invoice is attached.\n\
         \n\
         Attachment: invoice.pdf\n"
    );
}

#[test]
fn the_date_header_the_engine_validates_against_survives_verbatim() {
    let document = extract(NESTED_MULTIPART).unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains("Date: Thu, 21 Aug 2025 09:15:00 -0400"),
        "{text}"
    );
}

/// An evening email from New York is the next day in UTC. Printed beside the
/// verbatim header, that instant handed the model a second, wrong-day date
/// to quote, and validation accepted it because it was in the text.
#[test]
fn no_utc_sent_line() {
    let document = extract(
        b"From: alice@example.com\r\n\
          Date: Thu, 21 Aug 2025 21:15:00 -0400\r\n\
          Subject: Late invoice\r\n\
          \r\n\
          Body.\r\n",
    )
    .unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains("Date: Thu, 21 Aug 2025 21:15:00 -0400"),
        "{text}"
    );
    assert!(!text.contains("Sent:"), "{text}");
    assert!(!text.contains("2025-08-22"), "{text}");
}

/// Receipts and statements are HTML tables. A label and its value in
/// neighbouring cells stay on one line, told apart, and the curly
/// apostrophe in a client's name arrives as the character it is.
#[test]
fn html_tables_and_entities() {
    let document = extract(
        b"From: billing@example.com\r\n\
          Date: Mon, 3 Mar 2025 08:00:00 +0000\r\n\
          Subject: Your receipt\r\n\
          Content-Type: text/html; charset=utf-8\r\n\
          \r\n\
          <html><head><title>Receipt</title><style>td{padding:0}</style></head><body>\
          <!-- tracking pixel follows -->\
          <table><tr><td>Invoice date</td><td>March 3, 2025</td></tr>\
          <tr><td>Bill to</td><td>O&rsquo;Brien &amp; Co</td></tr>\
          <tr><td>Total</td><td>&euro;1,250&#x2009;&mdash; paid</td></tr></table>\
          </body></html>\r\n",
    )
    .unwrap();
    let text = &document.pages[0].text;

    assert!(text.contains("Invoice date | March 3, 2025"), "{text}");
    assert!(text.contains("Bill to | O\u{2019}Brien & Co"), "{text}");
    assert!(
        text.contains("Total | \u{20AC}1,250\u{2009}\u{2014} paid"),
        "{text}"
    );
    assert!(!text.contains("tracking"), "{text}");
    assert!(!text.contains("padding"), "{text}");
    assert!(!text.contains("&rsquo;"), "{text}");
}

/// Real receipts are not written on one line. The table above, indented the
/// way a person or a template writes it, and the body laid out in one big
/// cell the way transactional email is, still read as the lines a reader
/// sees: the heading, the address under its label, and label | value.
#[test]
fn an_indented_html_receipt_reads_as_it_renders() {
    let document = extract(
        b"From: billing@example.com\r\n\
          Date: Mon, 3 Mar 2025 08:00:00 +0000\r\n\
          Subject: Your receipt\r\n\
          Content-Type: text/html; charset=utf-8\r\n\
          \r\n\
          <html>\r\n<body>\r\n<table width=\"100%\">\r\n  <tr>\r\n    <td>\r\n\
          \x20     <h1>Receipt from Acme Corporation</h1>\r\n\
          \x20     <p>Billed to:<br>\r\n      Juniper Ridge Holdings Inc.</p>\r\n\
          \x20     <table>\r\n        <tr>\r\n          <td>Invoice date</td>\r\n\
          \x20         <td>March 3, 2025</td>\r\n        </tr>\r\n      </table>\r\n\
          \x20   </td>\r\n  </tr>\r\n</table>\r\n</body>\r\n</html>\r\n",
    )
    .unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains("\n\nReceipt from Acme Corporation\n\nBilled to:\n"),
        "{text}"
    );
    assert!(
        text.contains("\nBilled to:\nJuniper Ridge Holdings Inc.\n"),
        "{text}"
    );
    assert!(text.contains("\nInvoice date | March 3, 2025\n"), "{text}");
}

#[test]
fn nested_multiparts_yield_the_plain_body_and_list_the_attachment_without_extracting_it() {
    let document = extract(NESTED_MULTIPART).unwrap();
    let text = &document.pages[0].text;

    assert!(text.contains("The Q3 invoice is attached."), "{text}");
    assert!(!text.contains("<p>"), "{text}");
    assert!(text.ends_with("Attachment: invoice.pdf\n"), "{text}");
    assert!(!text.contains("JVBERi0="), "{text}");
}

#[test]
fn missing_headers_are_omitted_rather_than_emitted_empty() {
    let document = extract(
        b"From: alice@example.com\r\n\
          Subject: No date on this one\r\n\
          \r\n\
          Just a line of text.\r\n",
    )
    .unwrap();

    assert_eq!(
        document.pages[0].text,
        "From: alice@example.com\n\
         Subject: No date on this one\n\
         \n\
         Just a line of text.\n"
    );
}

#[test]
fn an_unparseable_date_header_keeps_the_verbatim_line_but_omits_the_sent_line() {
    let document = extract(
        b"From: alice@example.com\r\n\
          Date: sometime last Tuesday\r\n\
          Subject: Vague\r\n\
          \r\n\
          Body.\r\n",
    )
    .unwrap();
    let text = &document.pages[0].text;

    assert!(text.contains("Date: sometime last Tuesday"), "{text}");
    assert!(!text.contains("Sent:"), "{text}");
}

#[test]
fn an_html_only_email_falls_back_to_naively_detagged_text() {
    let document = extract(
        b"From: newsletter@example.com\r\n\
          Date: Mon, 2 Jun 2025 08:00:00 +0000\r\n\
          Subject: Weekly digest\r\n\
          Content-Type: text/html; charset=utf-8\r\n\
          \r\n\
          <html><body><p>Dear reader,</p><p>Rates &amp; terms changed.</p></body></html>\r\n",
    )
    .unwrap();
    let text = &document.pages[0].text;

    assert!(
        text.contains("Dear reader,\n\nRates & terms changed."),
        "{text}"
    );
    assert!(!text.contains('<'), "{text}");
}

#[test]
fn a_malformed_message_reports_a_parse_error_instead_of_panicking() {
    let error =
        extract(b" : this first header line starts with a space\r\n\r\nbody\r\n").unwrap_err();

    assert_eq!(error.code(), "PARSE_FAILED");
    assert!(!error.retryable());
}

/// Each nested `multipart/` level is another frame in mailparse's recursion,
/// on a two-megabyte extraction thread. A file a few kilobytes long can nest
/// deeply enough to run that stack out, and a stack overflow takes the whole
/// worker process down rather than failing one document.
fn nested_multiparts(depth: usize) -> Vec<u8> {
    let mut message = String::from(
        "From: alice@example.com\r\n\
         Subject: Deeply nested\r\n\
         MIME-Version: 1.0\r\n",
    );
    for level in 0..depth {
        message.push_str(&format!(
            "Content-Type: multipart/mixed; boundary=\"b{level}\"\r\n\r\n--b{level}\r\n"
        ));
    }
    message.push_str("Content-Type: text/plain\r\n\r\nthe innermost body\r\n");
    message.into_bytes()
}

#[test]
fn deeply_nested_multiparts_are_a_parse_error_not_a_crash() {
    let error = extract(&nested_multiparts(5_000)).unwrap_err();

    assert_eq!(error.code(), "PARSE_FAILED");
    assert!(!error.retryable());
}

#[test]
fn ordinary_nesting_depth_still_parses() {
    let document = extract(&nested_multiparts(8)).unwrap();

    assert!(
        document.pages[0].text.contains("the innermost body"),
        "{}",
        document.pages[0].text
    );
}
