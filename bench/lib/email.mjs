/// Internet mail (.eml) the way a mail client saves it.
///
/// The worker renders an email as a fixed header block (From, To, Cc, Date,
/// Subject, verbatim), a blank line, the text/plain body, and one
/// `Attachment:` line per attachment. Messages here are multipart/mixed with
/// a plain-text body and, optionally, attachments, with CRLF line endings
/// and fixed Message-IDs and boundaries so the bytes never vary.
import { assertSupported } from './fonts.mjs';

function wrapBase64(buffer) {
  return buffer.toString('base64').replace(/.{1,76}/g, (line) => `${line}\r\n`);
}

/// `headers` is an ordered list of [name, value]; `body` plain text with
/// `\n` line ends; `attachments` `{ filename, contentType, bytes }`.
export function eml({ headers, body, attachments = [], boundary = '----=_Part_InternBench_0001' }) {
  assertSupported(body.replaceAll('\n', ' '));
  const lines = headers.map(([name, value]) => `${name}: ${value}`);
  lines.push('MIME-Version: 1.0');
  const crlfBody = body.replace(/\r?\n/g, '\r\n');
  if (!attachments.length) {
    lines.push('Content-Type: text/plain; charset="utf-8"', 'Content-Transfer-Encoding: 8bit', '', crlfBody);
    return Buffer.from(`${lines.join('\r\n')}\r\n`, 'utf8');
  }
  lines.push(`Content-Type: multipart/mixed; boundary="${boundary}"`, '', 'This is a multi-part message in MIME format.', '');
  lines.push(`--${boundary}`, 'Content-Type: text/plain; charset="utf-8"', 'Content-Transfer-Encoding: 8bit', '', crlfBody, '');
  for (const attachment of attachments) {
    lines.push(
      `--${boundary}`,
      `Content-Type: ${attachment.contentType}; name="${attachment.filename}"`,
      'Content-Transfer-Encoding: base64',
      `Content-Disposition: attachment; filename="${attachment.filename}"`,
      '',
      wrapBase64(attachment.bytes).trimEnd(),
      '',
    );
  }
  lines.push(`--${boundary}--`, '');
  return Buffer.from(lines.join('\r\n'), 'utf8');
}

/// Prefixes each line of an earlier message with "> ", as a reply quotes it.
export function quote(text) {
  return text.split('\n').map((line) => (line ? `> ${line}` : '>')).join('\n');
}
