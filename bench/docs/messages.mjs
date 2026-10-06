/// An email reply quoting the message it answers, and a court notice saved
/// as plain text.
import { gold } from '../lib/gold.mjs';
import { emailDate, legalDate, longDate } from '../lib/format.mjs';
import { eml, quote } from '../lib/email.mjs';
import { buildPdf } from '../lib/pdf.mjs';
import { Flow } from '../lib/layout.mjs';
import { result } from './common.mjs';

export function emailApprovalThread() {
  const id = 'email-approval-thread';
  const sender = 'Signe Holmqvist';
  const recipient = 'Saul Ravenscroft';
  const vendor = 'Basalt Telemetry Inc.';
  const sent = '2026-03-17';
  const quoted = '2026-03-09';
  const quoteDate = '2026-03-02';
  const renewal = '2026-04-01';
  const termEnd = '2026-03-31';
  const approveBy = '2026-03-20';
  const domain = 'pemberly-falls-dist.example';
  const date = emailDate(sent, '09:42:11', '-0500');
  const earlier = `Hi Signe,

I need your approval to renew our fleet monitoring subscription with ${vendor} before the current term ends on ${longDate(termEnd)}. Their quote Q-7781 dated ${longDate(quoteDate)} covers all 214 tractors and 380 reefer trailers:

  - Three-year term, renewal effective ${longDate(renewal)}
  - $86,400 per year, fixed for the term (2025 price was $79,200)
  - Adds driver-camera retention of 90 days and the temperature exception API
  - 60-day termination for convenience after year one

Procurement has compared it with two alternatives; switching would cost about $140,000 in hardware swaps and training, so we recommend renewing. Because the amount is over my $50,000 limit, I need your approval by ${longDate(approveBy)} to get the order form signed in time.

The quote and procurement's comparison are attached.

Thanks,
Saul

--
Saul Ravenscroft | Director, Distribution Systems
Pemberly Falls Distribution Co. | (518) 555-0140`;
  const body = `Saul,

Approved. Please issue PO-55102 for the three-year renewal at $86,400 per year, and make sure the 90-day camera retention and the data deletion clause we discussed are in the order form before you sign it. Copying procurement so they can release the PO today.

Thanks for pulling the comparison together.

Signe

--
Signe Holmqvist | VP Operations
Pemberly Falls Distribution Co.

On Mon, 9 Mar 2026 at 16:18, ${recipient} <saul.ravenscroft@${domain}> wrote:
${quote(earlier)}`;
  const attachmentFlow = new Flow({ face: 'sans', fontSize: 10 });
  attachmentFlow.heading(`${vendor} - Quotation Q-7781`, { level: 2 });
  attachmentFlow.paragraph(`Prepared for Pemberly Falls Distribution Co. on ${longDate(quoteDate)}. Fleet monitoring subscription, 594 assets, three-year term, $86,400 per year.`);
  const attachment = buildPdf(attachmentFlow.finish());
  const headers = [
    ['From', `${sender} <signe.holmqvist@${domain}>`],
    ['To', `${recipient} <saul.ravenscroft@${domain}>`],
    ['Cc', `Procurement <procurement@${domain}>`],
    ['Date', date],
    ['Subject', `RE: Approval needed: ${vendor} fleet monitoring renewal (PO-55102)`],
    ['Message-ID', '<20260317144211.7781.signe@pemberly-falls-dist.example>'],
    ['In-Reply-To', '<20260309211801.4410.saul@pemberly-falls-dist.example>'],
  ];
  const bytes = eml({ headers, body, attachments: [{ filename: 'Basalt-Telemetry-Quote-Q7781.pdf', contentType: 'application/pdf', bytes: attachment }] });
  const text = `${headers.slice(0, 5).map(([name, value]) => `${name}: ${value}`).join('\n')}\n\n${body}\n\nAttachment: Basalt-Telemetry-Quote-Q7781.pdf\n`;
  return result({
    id,
    extension: 'eml',
    files: [{ name: `${id}.eml`, bytes }],
    text: [text],
    title: 'Email approving a vendor renewal, replying to and quoting the request',
    kind: 'email',
    textLayer: 'email',
    pages: 1,
    categories: ['email', 'competing_dates', 'referenced_agreement'],
    notes: `An email is dated by when it was sent: the Date header (${date}). The quoted request below the reply carries its own sent date (Mon, 9 Mar 2026) and the vendor quote date (${longDate(quoteDate)}), the renewal effective date (${longDate(renewal)}), the end of the current term (${longDate(termEnd)}), and the approval deadline (${longDate(approveBy)}). The sender approves; the recipient is accepted. The vendor is the subject of the approval, not a correspondent.`,
    gold: gold({
      type: 'Approval Email',
      acceptableTypes: ['Email', 'Email Approval', 'Renewal Approval', 'Purchase Approval', 'Approval'],
      date: sent,
      role: 'issuance',
      forbiddenDates: [[quoted, 'sent date of the quoted earlier message'], [quoteDate, 'date of the vendor quotation'], [renewal, 'renewal effective date'], [termEnd, 'end of the current subscription term'], [approveBy, 'approval deadline in the quoted request']],
      parties: [sender],
      relation: 'from',
      acceptablePartySets: [{ parties: [recipient], relation: 'to' }],
      roles: [[sender, 'issuer'], [sender, 'sender'], [recipient, 'recipient']],
      forbiddenParties: [],
      facts: [[vendor, 'Basalt Telemetry'], ['$86,400', '86,400'], ['PO-55102', 'renewal']],
      subjectTerms: ['approval', 'renewal', 'fleet monitoring', 'Basalt Telemetry'],
      readiness: 'ready',
      dateText: ['17 Mar 2026', date],
    }),
  });
}

export function courtHearingNotice() {
  const id = 'court-hearing-notice';
  const plaintiff = 'Gilchrist Harbor Marine Supply, LLC';
  const defendant = 'Tavistock Boatworks, Inc.';
  const dated = '2026-07-29';
  const filed = '2026-07-30';
  const hearing = '2026-08-21';
  const opposition = '2026-08-10';
  const motionFiled = '2026-07-22';
  const served = '2026-04-03';
  const responsesDue = '2026-05-04';
  const attorney = 'Desmond Kavanagh';
  const firm = 'Kavanagh Trask PLLC';
  const opposing = 'Elspeth Quarles';
  const opposingFirm = 'Underhill & Vargas LLP';
  const judge = 'Rosalind Fairbanks';
  const pad = (left, right) => `${left.padEnd(46)})  ${right}`;
  const text = [
    `E-FILED: ${longDate(filed)} 10:14 AM - Halden County Superior Court Clerk`,
    '',
    '              SUPERIOR COURT OF THE STATE OF WASHINGTON',
    '                         FOR HALDEN COUNTY',
    '',
    pad('GILCHRIST HARBOR MARINE SUPPLY, LLC,', ''),
    pad('a Washington limited liability company,', 'No. 26-2-04417-1'),
    pad('                    Plaintiff,', ''),
    pad('          v.', 'NOTICE OF HEARING'),
    pad('TAVISTOCK BOATWORKS, INC.,', '(Clerk\'s Action Required)'),
    pad('a Washington corporation,', ''),
    pad('                    Defendant.', ''),
    '',
    'TO:     THE CLERK OF THE COURT',
    `AND TO: ${defendant}, Defendant, and ${opposing} of ${opposingFirm}, its attorneys of record`,
    '',
    `PLEASE TAKE NOTICE that Plaintiff ${plaintiff} will bring on for hearing Plaintiff's Motion to Compel Discovery Responses and for an Award of Expenses, filed ${longDate(motionFiled)}, as follows:`,
    '',
    `    Date of hearing:     ${longDate(hearing)}`,
    '    Time:                9:00 a.m.',
    `    Judge:               Hon. ${judge}, Department 7`,
    '    Place:               Halden County Courthouse, 400 Calder Way, Halden Bay, WA',
    '    Type of hearing:     Civil motion, with oral argument (20 minutes per side)',
    '',
    `The motion concerns Plaintiff's First Interrogatories and Requests for Production, served on ${longDate(served)}, to which responses were due on ${longDate(responsesDue)}. Defendant has served no answers and no objections. Plaintiff certifies that counsel conferred by telephone on June 30, 2026 in a good-faith effort to resolve the dispute without court action.`,
    '',
    `Under the local civil rules, any opposition to the motion must be filed and served no later than ${longDate(opposition)}, and any reply no later than noon two court days before the hearing. Working copies must be delivered to Department 7 by the same deadlines.`,
    '',
    'Parties wishing to appear remotely must request a video link from the judicial assistant no later than two court days before the hearing.',
    '',
    `DATED this ${legalDate(dated)}.`,
    '',
    `                              ${firm}`,
    '',
    `                              By: /s/ ${attorney}`,
    `                              ${attorney}, WSBA No. 48817`,
    `                              Attorneys for Plaintiff ${plaintiff}`,
    '                              2201 Wexley Boulevard, Suite 700',
    '                              Halden Bay, WA 98264',
    '                              (360) 555-0123',
    '',
    'CERTIFICATE OF SERVICE',
    `I certify that on ${longDate(filed)} I caused this notice to be served on counsel for Defendant through the court's electronic filing system.`,
    `/s/ Ines Penhaligon, Legal Assistant, ${firm}`,
    '',
  ].join('\n');
  return result({
    id,
    extension: 'txt',
    files: [{ name: `${id}.txt`, bytes: Buffer.from(text, 'utf8') }],
    text: [text],
    title: 'Notice of hearing on a motion to compel, saved as plain text',
    kind: 'notice',
    textLayer: 'text',
    pages: 1,
    categories: ['notice', 'competing_dates', 'referenced_agreement'],
    notes: `Plain text with a court caption laid out in spaces. The notice is dated "this ${legalDate(dated)}" at the end; the e-filing line at the top and the certificate of service say ${longDate(filed)} (accepted: a court filing date), and the hearing itself is ${longDate(hearing)} (accepted: the event the notice exists to bring about). The motion filing date, discovery dates, and opposition deadline are traps. The litigants are the parties; the attorneys, firms, and judge are not.`,
    gold: gold({
      type: 'Notice of Hearing',
      date: dated,
      acceptableDates: [filed, hearing],
      role: 'notice',
      forbiddenDates: [[opposition, 'opposition deadline'], [motionFiled, 'date the motion was filed'], [served, 'date discovery was served'], [responsesDue, 'date discovery responses were due'], ['2026-06-30', 'meet-and-confer call']],
      parties: [plaintiff, defendant],
      relation: 'between',
      acceptablePartySets: [{ parties: [plaintiff], relation: 'from' }, { parties: [defendant], relation: 'to' }],
      roles: [[plaintiff, 'issuer'], [plaintiff, 'counterparty'], [defendant, 'recipient'], [defendant, 'counterparty']],
      forbiddenParties: [[attorney, 'plaintiff\'s attorney'], [firm, 'plaintiff\'s law firm'], [opposing, 'defendant\'s attorney'], [opposingFirm, 'defendant\'s law firm'], [judge, 'judge']],
      facts: [['Motion to Compel', 'motion to compel', 'discovery'], [longDate(hearing), 'August 21'], ['26-2-04417-1']],
      subjectTerms: ['hearing', 'motion to compel', 'discovery'],
      readiness: 'ready',
      dateText: [legalDate(dated)],
    }),
  });
}
