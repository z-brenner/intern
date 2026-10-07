/// Multi-column pages and a turned one: a three-column association
/// newsletter; a two-column supply agreement with a full-width title and
/// footnotes; one notice of a members' meeting written three ways into the
/// content stream (column by column, row by row across both columns, and
/// back to front), the same page to the eye; and a freight rate
/// confirmation whose landscape page is stored portrait with /Rotate 90.
import { Flow, Page, pageText } from '../lib/layout.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { amount, longDate, money, numericDate } from '../lib/format.mjs';
import { digitalPdf, readingSnippets, restream, result, signatureBlocks } from './common.mjs';

/// Article headings for a narrow column: bold, a little larger than the
/// text, and kept with the lines after them.
function article(flow, title, paragraphs) {
  flow.heading(title, { level: 3, size: 10.5, face: 'sans-bold', before: 6, after: 3 });
  for (const text of paragraphs) flow.paragraph(text, { after: 4 });
}

export function newsletterThreeColumn() {
  const id = 'newsletter-three-column';
  const association = 'Saltmarsh Point Homeowners Association';
  const contractor = 'Brackwell Marine Construction LLC';
  const landscaper = 'Fernhollow Grounds Care';
  const manager = 'Tamsin Oyelaran';
  const published = '2026-09-14';
  const meeting = '2026-10-22';
  const poolCloses = '2026-09-27';
  const repairsStart = '2026-10-05';
  const assessmentDue = '2026-11-30';
  const flow = new Flow({ face: 'serif', fontSize: 9, leading: 1.3, margins: { top: 48, bottom: 60, left: 48, right: 48 }, keep: [association, contractor, landscaper] });
  const page = flow.page;
  page.text(48, 74, 'THE TIDEWATER LEDGER', { face: 'sans-bold', size: 24 });
  page.text(48, 90, `Newsletter of the ${association}`, { face: 'sans', size: 9.5 });
  page.textRight(564, 74, 'Issue 41', { face: 'sans-bold', size: 11 });
  page.textRight(564, 90, `Published ${longDate(published)}`, { face: 'sans', size: 9.5 });
  page.line(48, 98, 564, 98, { width: 1.6 });
  flow.y = 106;
  flow.startColumns(3, 16);
  article(flow, 'Annual Meeting', [
    `The annual meeting of members will be held on ${longDate(meeting)} at 7:00 p.m. in the Grange Hall, 14 Wharf Lane. Members will elect three directors, hear the treasurer's report on the reserve fund, and vote on the special assessment described in this issue.`,
    'A quorum is one quarter of the 212 lots, present in person or by proxy. Proxy forms are enclosed with this newsletter and may be returned to the office or left in the drop box at the marina gate.',
  ]);
  article(flow, 'Seawall Survey Findings', [
    `In July the board engaged ${contractor} to survey the 1,860 feet of seawall along the north shore. The survey found undermining behind eleven sections between the boat ramp and Heron Quay, and two places where the cap has separated from the wall.`,
    'The engineer rated the north sections as needing repair within two years. The south wall, rebuilt in 2011, is in good condition and needs only routine sealing of joints.',
  ]);
  article(flow, 'Special Assessment Vote', [
    'The repair estimate for the north sections is $186,400. The reserve fund holds $61,250 that can be applied after keeping the minimum balance the bylaws require. The board therefore proposes a special assessment of $1,150 per lot to cover the balance and a ten percent contingency.',
    `If members approve it at the annual meeting, the assessment will be due on ${longDate(assessmentDue)}. Owners may instead pay in three equal installments with their quarterly dues, with no interest charged.`,
  ]);
  article(flow, 'Dock Repairs Begin', [
    `Separately from the seawall work, ${contractor} will replace the decking and four pilings on the community dock starting ${longDate(repairsStart)}. The work is paid for from this year's maintenance budget and does not depend on the assessment vote.`,
    'The dock will be closed for about three weeks. Kayak racks will be moved to the grass beside the boat ramp while the work is under way.',
  ]);
  article(flow, 'Board Candidates', [
    'Three seats are open this year. The nominating committee has put forward Desmond Achterberg (Lot 44), Priyanka Velloso (Lot 117) and Rowan Fairweather (Lot 9). Nominations may also be made from the floor at the meeting.',
    'Candidate statements are posted on the bulletin board in the Grange Hall and on the association website.',
  ]);
  article(flow, 'Pool Closing Weekend', [
    `The pool will close for the season at 6:00 p.m. on ${longDate(poolCloses)}. Volunteers are needed that morning to stack chairs and store the lane lines; coffee and doughnuts are provided.`,
  ]);
  article(flow, 'Landscaping Contract', [
    `The board renewed the grounds contract with ${landscaper} for two more years at $3,940 per month, unchanged from the current rate. The renewal adds leaf removal in November and two extra mowings in June.`,
    'Owners who want the crew to skip their frontage should tell the office in writing.',
  ]);
  article(flow, 'From the Manager', [
    'Thank you to everyone who returned the parking survey: 164 households answered. Overnight trailer parking on Gull Street will be limited to weekends from October through April.',
    `The office is open Tuesday and Thursday mornings. - ${manager}, Community Manager`,
  ]);
  flow.endColumns();
  const pages = flow.finish();
  if (pages.length !== 1) throw new Error(`${id}: the newsletter must fit one page, laid out ${pages.length}`);
  page.line(48, 742, 564, 742, { width: 0.6 });
  page.textCenter(306, 754, `${association} - 1 Heron Quay, Saltmarsh Point, ME 04563 - office@saltmarshpoint.example`, { face: 'sans', size: 7.5, grey: 0.3 });
  const { bytes, text } = digitalPdf(pages);
  const lines = text[0].split('\n');
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Homeowners association newsletter set in three columns',
    kind: 'newsletter',
    textLayer: 'native',
    pages: 1,
    categories: ['multi_column', 'complex_pdf', 'competing_dates', 'irrelevant_names'],
    notes: `A one-page newsletter under a full-width masthead, its eight articles set in three narrow columns. The issue is dated in the masthead (${longDate(published)}); the meeting (${longDate(meeting)}), the pool closing (${longDate(poolCloses)}), the start of dock repairs (${longDate(repairsStart)}) and the assessment due date (${longDate(assessmentDue)}) are traps in the body. Written column by column, so a reader that follows the stream reads the right order; one that reads across the page does not. The association issues it; the contractor, the landscaper, the manager and the candidates are not parties.`,
    structure: structure({
      readingOrder: readingSnippets(lines.slice(2, -1), 9),
      routes: { 1: 'layout' },
    }),
    gold: gold({
      type: 'Newsletter',
      acceptableTypes: ['Homeowners Association Newsletter', 'Association Newsletter'],
      date: published,
      role: 'issuance',
      forbiddenDates: [[meeting, 'date of the annual meeting'], [poolCloses, 'pool closing'], [repairsStart, 'start of dock repairs'], [assessmentDue, 'special assessment due date']],
      parties: [association],
      relation: 'from',
      roles: [[association, 'issuer']],
      forbiddenParties: [[contractor, 'contractor surveying the seawall'], [landscaper, 'grounds contractor'], [manager, 'community manager'], ['Desmond Achterberg', 'board candidate']],
      facts: [['$1,150', '1,150'], ['$186,400', '186,400'], [longDate(meeting), 'October 22']],
      subjectTerms: ['annual meeting', 'special assessment', 'seawall', 'pool'],
      readiness: 'either',
      dateText: [longDate(published)],
    }),
  });
}

export function agreementTwoColumnFootnotes() {
  const id = 'agreement-two-column-footnotes';
  const company = 'Larkhaven Seed Company';
  const grower = 'Prairie Wren Growers Cooperative';
  const dated = '2026-03-02';
  const letterOfIntent = '2025-12-09';
  const delivery = '2026-09-30';
  const termEnd = '2027-12-31';
  const flow = new Flow({ face: 'serif', fontSize: 9.5, leading: 1.3, margins: { top: 64, bottom: 64, left: 64, right: 64 }, keep: [company, grower] });
  flow.heading('SEED PRODUCTION AND SUPPLY AGREEMENT', { level: 1, size: 14, align: 'center', before: 0 });
  flow.paragraph(`This Seed Production and Supply Agreement (this "Agreement") is dated as of ${longDate(dated)} and is made between ${company}, a Nebraska corporation ("Company"), and ${grower}, an Iowa cooperative association ("Grower").[1] Company develops and sells soybean seed; Grower's members farm in Story, Boone and Hamilton Counties, Iowa. The parties agree as follows:`, { align: 'left', after: 8 });
  // Every note is placed before the columns, so the columns stop above them.
  flow.footnote('[1]', `This Agreement supersedes the letter of intent between the parties dated ${longDate(letterOfIntent)}, which has no further effect.`);
  flow.footnote('[2]', `All harvested seed must be delivered no later than ${longDate(delivery)}; loads received after that date may be refused.`);
  flow.footnote('[3]', `${longDate(termEnd)}.`);
  flow.startColumns(2, 18);
  const sections = [
    ['Definitions', '"Seed" means certified soybean seed of the varieties LS-2207 and LS-2391 grown under this Agreement. "Clean Seed" means Seed that has been conditioned, tested and accepted under Section 4. "Season" means the 2026 growing season.'],
    ['Production', 'Grower will cause its members to plant 640 acres of foundation seed supplied by Company, in fields approved by Company, and to grow the crop under the isolation, roguing and harvest practices in Company\'s production guide.'],
    ['Inspection', 'Company may inspect the fields at any reasonable time. The state crop improvement association will inspect each field at least twice before harvest, and Grower will remove any field that fails inspection from production at its own cost.'],
    ['Conditioning and Testing', 'Grower will deliver harvested seed to Company\'s conditioning plant in Ames. Company will clean and test each lot for germination, purity and seed-borne disease. A lot below 85% germination is rejected and becomes Grower\'s property for sale as commodity grain.'],
    ['Delivery', 'Grower will deliver all harvested seed by the date in footnote [2]. Title and risk of loss pass to Company when each truckload is weighed at the plant.'],
    ['Price', 'Company will pay $18.40 per bushel of Clean Seed, plus a premium of $1.25 per bushel for any lot testing at 92% germination or higher. Commodity grain from rejected lots is not paid for under this Agreement.'],
    ['Payment', 'Company will pay half of the price within 30 days after conditioning and the balance by January 15 of the following year, by wire transfer to the account Grower designates in writing.'],
    ['Quality Claims', 'Grower makes no warranty of yield. Company\'s only remedy for Seed that fails inspection or testing is rejection under Sections 3 and 4.'],
    ['Term', 'This Agreement covers the Season and renews for the 2027 growing season unless either party gives notice by November 1, 2026. It ends in any event on the date in footnote [3].'],
    ['General', 'This Agreement is governed by Iowa law, may be amended only in writing signed by both parties, and may be signed in counterparts.'],
  ];
  sections.forEach(([title, body], index) => {
    // Five sections to a column.
    if (index === 5) flow.nextFrame();
    flow.paragraph([{ text: `${index + 1}. ${title}. `, face: 'sans-bold' }, { text: body }], { after: 5 });
  });
  if (flow.pageNumber !== 1) throw new Error(`${id}: the columns must fit page 1`);
  flow.endColumns();
  flow.pageBreak();
  flow.paragraph('IN WITNESS WHEREOF, the parties have signed this Agreement as of the date first written above.', { after: 12 });
  signatureBlocks(flow, [
    { heading: 'COMPANY', entity: company, name: 'Ilse Brannagan', title: 'Vice President, Production' },
    { heading: 'GROWER', entity: grower, name: 'Thaddeus Okonkwo', title: 'General Manager' },
  ], { stacked: true });
  const pages = flow.finish();
  if (pages.length !== 2) throw new Error(`${id}: expected two pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  const firstPage = text[0].split('\n');
  const snippets = readingSnippets(firstPage, 8);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Seed supply agreement: full-width title, two columns of terms, footnotes',
    kind: 'contract',
    textLayer: 'native',
    pages: 2,
    categories: ['multi_column', 'contract', 'competing_dates', 'referenced_agreement'],
    notes: `Page 1 opens with a full-width title and preamble, sets ten sections in two columns, and ends with footnotes across the page; page 2 is the signature page. The agreement is dated as of ${longDate(dated)} in the preamble. The superseded letter of intent (${longDate(letterOfIntent)}), the delivery deadline (${longDate(delivery)}) and the end of the term (${longDate(termEnd)}) are all in footnotes, which a reader must keep apart from the columns. Between the seed company and the growers' cooperative.`,
    structure: structure({
      readingOrder: ['SEED PRODUCTION AND SUPPLY AGREEMENT', ...snippets, 'supersedes the letter of intent'],
      routes: { 1: 'layout', 2: 'fast' },
    }),
    gold: gold({
      type: 'Seed Production and Supply Agreement',
      acceptableTypes: ['Seed Supply Agreement', 'Supply Agreement', 'Seed Production Agreement'],
      date: dated,
      role: 'effective',
      forbiddenDates: [[letterOfIntent, 'superseded letter of intent'], [delivery, 'delivery deadline'], [termEnd, 'end of the term']],
      parties: [company, grower],
      relation: 'between',
      roles: [[company, 'buyer'], [grower, 'seller']],
      forbiddenParties: [['Ilse Brannagan', 'signatory'], ['Thaddeus Okonkwo', 'signatory']],
      facts: [['$18.40', '18.40'], ['640 acres', '640'], ['soybean']],
      subjectTerms: ['seed', 'production', 'supply', 'bushel'],
      readiness: 'ready',
      dateText: [longDate(dated)],
    }),
  });
}

/// The notice of a members' meeting, laid out once in two columns under a
/// full-width heading and above a full-width signature line.
function meetingNoticePage() {
  const union = 'Larchmont Valley Federal Credit Union';
  const partner = 'Ironbridge Community Credit Union';
  const flow = new Flow({ face: 'serif', fontSize: 10, leading: 1.35, margins: { top: 60, bottom: 64, left: 60, right: 60 }, keep: [union, partner, 'Cobbleford Election Services'] });
  flow.paragraph(union.toUpperCase(), { face: 'sans-bold', size: 15, align: 'center', after: 2 });
  flow.paragraph('4100 Orchard Road, Larchmont, OR 97370 - Charter No. 24518', { face: 'sans', size: 8.5, align: 'center', after: 10 });
  flow.paragraph('NOTICE OF SPECIAL MEETING OF MEMBERS', { face: 'sans-bold', size: 12.5, align: 'center', after: 2 });
  flow.paragraph(`Dated ${longDate('2026-08-03')}`, { face: 'sans', size: 9.5, align: 'center', after: 12 });
  flow.rule({ width: 0.8 });
  flow.startColumns(2, 22);
  const heading = (title) => flow.paragraph(title, { face: 'sans-bold', size: 10, after: 3, keepWithNext: 30 });
  heading('Date, Time and Place');
  flow.paragraph(`A special meeting of the members of ${union} will be held on ${longDate('2026-09-15')} at 6:30 p.m. Pacific time in the community room of the main branch at 4100 Orchard Road, Larchmont, Oregon. Doors open at 6:00 p.m. for registration.`);
  heading('Purpose of the Meeting');
  flow.paragraph(`The only business is a vote on the proposal of the Board of Directors to merge ${union} into ${partner}, a state-chartered credit union in Ashby Falls, Oregon. If the members approve and the regulators consent, the merger would take effect on ${longDate('2027-01-01')}.`);
  flow.paragraph('Every member account would become an account of the continuing credit union with the same balance, the same account numbers and the same dividend rates, and the branches in Larchmont and Sefton would stay open.');
  heading('Record Date');
  flow.paragraph(`Members of record at the close of business on ${longDate('2026-07-31')} are entitled to vote. Each member has one vote, regardless of the number or size of the member's accounts.`);
  // The second column starts with voting.
  flow.nextFrame();
  heading('How to Vote');
  flow.paragraph('Members may vote in person at the meeting or by the written ballot enclosed with this notice. A ballot must be signed by the member named on it; a joint owner who is not a member may not vote.');
  heading('Written Ballots');
  flow.paragraph(`Return the ballot in the postage-paid envelope. Ballots must be received by the independent teller, Cobbleford Election Services, no later than ${longDate('2026-09-11')}. Ballots received after that date will not be counted.`);
  heading('Questions');
  flow.paragraph('The merger plan, the financial statements of both credit unions and the Board\'s reasons for recommending the merger are available at every branch and at larchmontvalley.example/merger. Call member services at (503) 555-0140 with any question.');
  flow.endColumns();
  flow.rule({ width: 0.5, before: 6 });
  flow.paragraph('By order of the Board of Directors', { face: 'serif', size: 10, after: 2 });
  flow.paragraph('Corinne Abernethy-Vale, Secretary of the Board', { face: 'serif', size: 10 });
  const pages = flow.finish();
  if (pages.length !== 1) throw new Error(`the meeting notice must fit one page, laid out ${pages.length}`);
  return { page: pages[0], union, partner };
}

const STREAM_ORDERS = {
  columns: 'column by column, the order it is read in',
  rows: 'row by row across both columns, so each left-column line is followed by the right-column line beside it',
  reverse: 'back to front: the signature line first and the heading last',
};

function meetingNotice(id, order) {
  const { page, union, partner } = meetingNoticePage();
  const reading = pageText(page);
  const streamed = restream(page, order);
  const { bytes, text } = digitalPdf([streamed]);
  const lines = reading.split('\n');
  const notice = '2026-08-03';
  const meeting = '2026-09-15';
  const effective = '2027-01-01';
  const record = '2026-07-31';
  const ballots = '2026-09-11';
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    structureText: [reading],
    title: `Credit union notice of a special members' meeting, two columns written ${order === 'columns' ? 'column by column' : order === 'rows' ? 'row by row' : 'back to front'}`,
    kind: 'notice',
    textLayer: 'native',
    pages: 1,
    categories: order === 'columns' ? ['multi_column', 'notice', 'competing_dates', 'stream_order'] : ['multi_column', 'complex_pdf', 'notice', 'competing_dates', 'stream_order'],
    notes: `One of three files that are the same page to the eye and differ only in the order the text is written into the content stream: here ${STREAM_ORDERS[order]}. The notice is dated ${longDate(notice)} under its heading; the meeting (${longDate(meeting)}), the record date (${longDate(record)}), the ballot deadline (${longDate(ballots)}) and the merger's proposed effective date (${longDate(effective)}) are traps. The credit union calling the meeting is the party; its merger partner and the teller are not.`,
    structure: structure({
      readingOrder: ['NOTICE OF SPECIAL MEETING OF MEMBERS', ...readingSnippets(lines.slice(4, -2), 9), 'By order of the Board of Directors'],
      routes: { 1: 'layout' },
    }),
    gold: gold({
      type: 'Notice of Special Meeting of Members',
      acceptableTypes: ['Notice of Special Meeting', 'Special Meeting Notice'],
      date: notice,
      role: 'notice',
      forbiddenDates: [[meeting, 'date of the meeting'], [record, 'record date'], [ballots, 'ballot deadline'], [effective, 'proposed effective date of the merger']],
      parties: [union],
      relation: 'from',
      roles: [[union, 'issuer']],
      forbiddenParties: [[partner, 'merger partner'], ['Cobbleford Election Services', 'independent teller'], ['Corinne Abernethy-Vale', 'board secretary who signs']],
      facts: [[partner, 'Ironbridge'], ['merge', 'merger']],
      subjectTerms: ['special meeting', 'members', 'ballot'],
      readiness: 'ready',
      dateText: [longDate(notice)],
    }),
  });
}

export const meetingNoticeColumns = () => meetingNotice('meeting-notice-columns', 'columns');
export const meetingNoticeInterleaved = () => meetingNotice('meeting-notice-interleaved', 'rows');
export const meetingNoticeReversed = () => meetingNotice('meeting-notice-reversed', 'reverse');

export function rateConfirmationRotated() {
  const id = 'rate-confirmation-rotated';
  const broker = 'Copperline Freight Brokerage LLC';
  const carrier = 'Halden Ridge Trucking Inc.';
  const shipper = 'Ostrowski Cold Storage';
  const confirmed = '2026-06-08';
  const pickup = '2026-06-10';
  const deliver = '2026-06-12';
  const load = 'CFB-260608-117';
  // Page 1 is landscape: drawn as it is displayed, stored portrait with
  // /Rotate 90.
  const sheet = new Page({ width: 792, height: 612, rotate: 90 });
  sheet.storeTurned = true;
  sheet.text(40, 50, broker, { face: 'sans-bold', size: 15 });
  sheet.text(40, 64, '2900 Tannery Road, Suite 300, Cedar Bluffs, NE 68015 - Dispatch (402) 555-0166 - MC 884207', { size: 8, grey: 0.3 });
  sheet.textRight(752, 50, 'CARRIER RATE CONFIRMATION', { face: 'sans-bold', size: 14 });
  sheet.textRight(752, 64, `Confirmed ${longDate(confirmed)}`, { size: 9 });
  sheet.line(40, 72, 752, 72, { width: 1.2 });
  const fields = [['Load number', load], ['Carrier', carrier], ['Carrier MC', 'MC 610553'], ['Equipment', "53' reefer, set at 34 F"]];
  fields.forEach(([label, value], index) => {
    sheet.text(40 + (index % 2) * 360, 92 + Math.floor(index / 2) * 15, `${label}:`, { face: 'sans-bold', size: 9 });
    sheet.text(120 + (index % 2) * 360, 92 + Math.floor(index / 2) * 15, value, { size: 9 });
  });
  const columns = [['Stop', 40], ['Type', 80], ['Facility', 140], ['Address', 300], ['Date', 520], ['Window', 600], ['Reference', 680]];
  const stops = [
    ['1', 'Pickup', shipper, '1120 Dock Street, Grand Isle, NE 68803', numericDate(pickup), '06:00-10:00', 'PU 448120'],
    ['2', 'Drop', 'Fennimore Grocers DC', '75 Commerce Loop, Pella, IA 50219', numericDate('2026-06-11'), '14:00-18:00', 'PO 91-2240'],
    ['3', 'Drop', 'Brightwater Market #12', '400 River Road, Galena, IL 61036', numericDate(deliver), '05:00-09:00', 'PO 77-0918'],
  ];
  let y = 134;
  sheet.rect(40, y - 11, 712, 15, { fill: 0.86, stroke: null });
  for (const [title, x] of columns) sheet.text(x + 2, y, title, { face: 'sans-bold', size: 8.5 });
  for (const stop of stops) {
    y += 18;
    stop.forEach((cell, index) => sheet.text(columns[index][1] + 2, y, cell, { size: 8.5 }));
    sheet.line(40, y + 5, 752, y + 5, { width: 0.3, grey: 0.4 });
  }
  y += 34;
  const charges = [['Linehaul', 268000], ['Fuel surcharge', 41300], ['Extra stop (stop 2)', 7500]];
  sheet.text(520, y, 'Charge', { face: 'sans-bold', size: 9 });
  sheet.textRight(752, y, 'Amount (USD)', { face: 'sans-bold', size: 9 });
  for (const [label, cents] of charges) {
    y += 14;
    sheet.text(520, y, label, { size: 9 });
    sheet.textRight(752, y, amount(cents), { size: 9 });
  }
  const total = charges.reduce((sum, [, cents]) => sum + cents, 0);
  y += 16;
  sheet.line(520, y - 10, 752, y - 10, { width: 0.6 });
  sheet.text(520, y, 'Total rate', { face: 'sans-bold', size: 9.5 });
  sheet.textRight(752, y, money(total), { face: 'sans-bold', size: 9.5 });
  sheet.text(40, 560, 'Carrier must call dispatch on arrival and departure at every stop. Temperature must be logged every two hours. See page 2 for terms.', { size: 8 });
  // Page 2: the terms, an ordinary portrait page.
  const flow = new Flow({ face: 'serif', fontSize: 10, leading: 1.35, keep: [broker, carrier] });
  flow.heading('TERMS AND CONDITIONS OF CARRIAGE', { level: 2, size: 11.5, before: 0 });
  [
    `This confirmation incorporates the Broker-Carrier Agreement between ${broker} and ${carrier}. If they conflict, this confirmation controls as to rate, stops and appointment times.`,
    'Carrier will not re-broker, co-broker or assign this load. A load re-brokered without written consent will not be paid.',
    'Detention is paid at $50.00 per hour after two hours at any stop, only if the driver reported arrival and departure times to dispatch.',
    'Carrier is liable for loss of or damage to the cargo, including loss caused by a temperature excursion, up to $100,000 per load.',
    'Carrier will be paid within 30 days after Broker receives a signed proof of delivery and the carrier invoice, or within 3 days for a 3% quick-pay fee.',
  ].forEach((text, index) => flow.paragraph(`${index + 1}. ${text}`));
  flow.paragraph(`Accepted for ${carrier} by: ____________________   Date: __________`, { before: 14 });
  const terms = flow.finish();
  const pages = [sheet, ...terms];
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Freight rate confirmation with a landscape page stored turned (/Rotate 90)',
    kind: 'rate_confirmation',
    textLayer: 'native',
    pages: pages.length,
    categories: ['rotated_page', 'table', 'key_value', 'competing_dates', 'layout_parties'],
    notes: `Page 1 is a landscape stop table stored as a portrait page with /Rotate 90 and its content turned into it, as many dispatch systems write it; a reader that ignores the rotation measures every box sideways. The confirmation is dated ${longDate(confirmed)} in its heading; the pickup (${numericDate(pickup)}) and delivery (${numericDate(deliver)}) dates in the stop table are traps. Issued by the broker to the carrier; the shipper and consignees are stops, not parties. Page 2 is ordinary portrait text.`,
    structure: structure({
      tables: [[
        columns.map(([title]) => title),
        ...stops,
      ]],
      keyValues: [...fields, ['Total rate', money(total)]],
      routes: { 1: 'layout', 2: 'fast' },
    }),
    gold: gold({
      type: 'Carrier Rate Confirmation',
      acceptableTypes: ['Rate Confirmation', 'Load Confirmation'],
      date: confirmed,
      role: 'issuance',
      forbiddenDates: [[pickup, 'pickup date'], [deliver, 'final delivery date'], ['2026-06-11', 'second stop date']],
      parties: [broker],
      relation: 'from',
      acceptablePartySets: [{ parties: [carrier], relation: 'to' }, { parties: [broker, carrier], relation: 'between' }],
      roles: [[broker, 'issuer'], [carrier, 'recipient'], [carrier, 'counterparty']],
      forbiddenParties: [[shipper, 'shipper at the pickup stop'], ['Fennimore Grocers DC', 'consignee at a drop stop']],
      facts: [[load], [money(total), amount(total)], [carrier, 'Halden Ridge']],
      subjectTerms: ['freight', 'reefer', 'linehaul', 'load'],
      readiness: 'ready',
      dateText: [longDate(confirmed)],
    }),
  });
}
