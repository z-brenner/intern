/// ATLAS training corpus generator. Laboratory tooling; the app does not call it.
///
///   node research/atlas-models/dataset/generate.mjs --out DIR [--count N] [--seed S] [--split train|val]
///       [--avoid OTHER/gold.json ...]
///
/// Writes N synthetic documents into DIR with a gold.json in InternBench's schema, so
/// `intern-bench run --corpus DIR --gold DIR/gold.json` scores them like any other set.
///
/// What it is and is not:
/// - Every name, organisation, address, number and event is invented here from the syllable
///   and word pools below. Names that appear in bench/gold.json or any
///   scripts/realdocs-eval/sample*.json gold are refused (checked at start), so no test entity
///   can leak into training.
/// - It shares no template with InternBench: it imports only bench/lib (the PDF, DOCX and gold
///   writers), never bench/docs.
/// - The families are the kinds the real development sets are made of (agency letters to a
///   company, company letters to an agency, court notices between two sides, enforcement
///   notices for a respondent, invoices, receipts and statements from a vendor), with the
///   naming conventions the real-set gold uses. No real document's text was copied.
/// - Each document carries the traps the real sets fail on: an addressee person or outside
///   counsel where the company is wanted, cc lists, signatories, deadlines and referenced
///   documents' dates.
///
/// - --avoid (repeatable) also refuses a document whose answer parties are answer parties in
///   another generated set, so a validation split never repeats a training entity.
///
/// Deterministic: a document is a pure function of (seed, split, index) and the refused names.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Flow } from '../../../bench/lib/layout.mjs';
import { buildPdf } from '../../../bench/lib/pdf.mjs';
import { pageText } from '../../../bench/lib/layout.mjs';
import { Rng } from '../../../bench/lib/rng.mjs';
import { entry, gold } from '../../../bench/lib/gold.mjs';
import { docx } from '../../../bench/lib/office.mjs';
import { addDays, dayMonthYear, longDate, money, numericDate, shortMonthDate } from '../../../bench/lib/format.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO = resolve(HERE, '../../..');

// ---------------------------------------------------------------------------------------------
// Invented names.

const FIRST = ['Aldric', 'Bettina', 'Caius', 'Delphine', 'Emrys', 'Fenella', 'Gideon', 'Hesper', 'Ivo', 'Jessamy', 'Kester',
  'Leonie', 'Marek', 'Nerys', 'Osric', 'Perpetua', 'Quillon', 'Rosalind', 'Soren', 'Thea', 'Ulric', 'Verity', 'Wendel',
  'Xanthe', 'Yorick', 'Zelda', 'Anouk', 'Benedek', 'Calla', 'Dorian', 'Elspeth', 'Florian', 'Greta', 'Hollis', 'Ingram',
  'Juno', 'Lazlo', 'Mireille', 'Niamh', 'Odalys', 'Piers', 'Rhiannon', 'Sabine', 'Tobiah', 'Una', 'Vashti', 'Wynn',
  'Amara', 'Bastian', 'Cosima', 'Desmond', 'Esme', 'Fitz', 'Ginevra', 'Hamish', 'Isolde', 'Jago', 'Kasimir', 'Lorna',
  'Magnus', 'Noor', 'Orla', 'Pascal', 'Quinta', 'Ravi', 'Saoirse', 'Tamsin', 'Udo', 'Vesna', 'Wilhelmina', 'Yusuf'];
const LAST = ['Abernathy', 'Blackwood', 'Castellane', 'Dunmore', 'Everleigh', 'Fairweather', 'Gallagher', 'Hartigan',
  'Ironside', 'Jablonski', 'Kettering', 'Lindqvist', 'Mortensen', 'Nightingale', 'Okonkwo', 'Pemberton', 'Quarrington',
  'Rasmussen', 'Strathmore', 'Thistlewood', 'Underhill', 'Vandermeer', 'Whitlock', 'Yardley', 'Zabrowski', 'Achterberg',
  'Bellweather', 'Cardenas', 'Delacroix-Hume', 'Eastwick', 'Faraday', 'Grimshaw', 'Holloway', 'Ishikawa', 'Jorgensen',
  'Kowalczyk', 'Lockridge', 'Merriweather', 'Nakagawa', 'Ostrander', 'Pryce', 'Rutherford', 'Sorensen', 'Tamura',
  'Uffington', 'Valcourt', 'Wexley', 'Yilmaz', 'Zeller', 'Aldana', 'Brightwater', 'Coldfield', 'Drummond', 'Etheridge',
  'Fennimore', 'Galloway', 'Hawthorne', 'Inglewood', 'Jessop', 'Kincaid', 'Larkspur', 'Mabry', 'Northcott', 'Oyelaran'];
const STEM_A = ['Alder', 'Brack', 'Cinder', 'Dovet', 'Ember', 'Fallow', 'Glen', 'Harrow', 'Iron', 'Juni', 'Kestrel', 'Larch',
  'Moss', 'North', 'Orrin', 'Pell', 'Quarry', 'Rook', 'Sable', 'Thorn', 'Umber', 'Vale', 'Wren', 'Yarrow', 'Zephyr', 'Ash',
  'Bramble', 'Copper', 'Drift', 'Elm', 'Flint', 'Garnet', 'Heron', 'Indigo', 'Jasper', 'Kiln', 'Lumen', 'Marrow', 'Nettle',
  'Onyx', 'Pike', 'Quill', 'Rowan', 'Slate', 'Tamar', 'Upland', 'Vesper', 'Wick', 'Cobalt', 'Saffron', 'Talon', 'Basalt'];
const STEM_B = ['field', 'brook', 'gate', 'stone', 'wright', 'haven', 'mere', 'crest', 'ridge', 'wood', 'ford', 'vale', 'more',
  'well', 'hurst', 'stead', 'lane', 'point', 'side', 'bay', 'mark', 'line', 'worth', 'holm', 'cliff'];
const TRADE = ['Logistics', 'Pharmaceuticals', 'Foods', 'Biologics', 'Medical Devices', 'Chemical', 'Energy', 'Software',
  'Health Services', 'Packaging', 'Analytics', 'Manufacturing', 'Robotics', 'Marine Services', 'Nutrition', 'Laboratories',
  'Outfitters', 'Telecom', 'Freight', 'Dairy', 'Cosmetics', 'Diagnostics', 'Aviation', 'Water Systems', 'Textiles', 'Mills'];
const SUFFIX = ['Inc.', 'LLC', 'Corp.', 'Co.', 'Ltd.', 'Holdings, Inc.', 'Group LLC', 'Company', 'L.P.', 'Incorporated'];
const LAWSUFFIX = ['LLP', 'P.C.', 'PLLC', 'LLP', '& Partners LLP'];
const STATES = [['Ostrava', 'OS'], ['Calderon', 'CD'], ['New Wexford', 'NW'], ['Marlow', 'ML'], ['East Tamar', 'ET'], ['Harlan', 'HR']];
const STREETS = ['Quarry Road', 'Lantern Way', 'Millrace Avenue', 'Beacon Street', 'Cobblestone Lane', 'Harbor Drive', 'Foundry Row',
  'Larkfield Boulevard', 'Orchard Terrace', 'Signal Hill Road', 'Granary Court', 'Tidewater Parkway', 'Ridgeback Drive'];
const CITIES = ['Fallowmere', 'Port Kestrel', 'Ashby Crossing', 'Wrenford', 'Glenhollow', 'Marrowgate', 'Pellston', 'Sablecourt',
  'Thornbury Falls', 'Umberton', 'Vesperdale', 'Yarrow Point', 'Coldharbor', 'Brackenridge'];

function person(rng) {
  const middle = rng.chance(0.25) ? ` ${String.fromCharCode(65 + rng.int(0, 25))}.` : '';
  return `${rng.pick(FIRST)}${middle} ${rng.pick(LAST)}`;
}
function stem(rng) {
  return `${rng.pick(STEM_A)}${rng.pick(STEM_B)}`;
}
function company(rng) {
  const shape = rng.int(0, 3);
  if (shape === 0) return `${stem(rng)} ${rng.pick(TRADE)} ${rng.pick(SUFFIX)}`;
  if (shape === 1) return `${stem(rng)} ${rng.pick(SUFFIX)}`;
  if (shape === 2) return `${rng.pick(STEM_A)} & ${rng.pick(STEM_A)} ${rng.pick(TRADE)}, ${rng.pick(['Inc.', 'LLC'])}`;
  return `${rng.pick(LAST)} ${rng.pick(TRADE)} ${rng.pick(SUFFIX)}`;
}
function lawFirm(rng) {
  return `${rng.pick(LAST)} ${rng.pick(['&', 'and'])} ${rng.pick(LAST)} ${rng.pick(LAWSUFFIX)}`;
}
function address(rng) {
  const [state, code] = rng.pick(STATES);
  return [`${rng.int(10, 9800)} ${rng.pick(STREETS)}${rng.chance(0.3) ? `, Suite ${rng.int(100, 1900)}` : ''}`,
    `${rng.pick(CITIES)}, ${code} ${String(rng.int(10000, 99999))}`, state];
}
function isoDate(rng, from = 2019, to = 2027) {
  const year = rng.int(from, to);
  const month = rng.int(1, 12);
  const day = rng.int(1, 28);
  return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}
function printDate(rng, iso) {
  return rng.pick([longDate, longDate, longDate, dayMonthYear, numericDate, shortMonthDate])(iso);
}

// Regulators: an invented jurisdiction's offices, never a real agency's name.
const AGENCY_FORMS = [
  (r) => `${r.pick(STATES)[0]} Department of ${r.pick(['Environmental Quality', 'Health Services', 'Consumer Protection', 'Labor and Industry', 'Agriculture', 'Financial Regulation', 'Transportation Safety'])}`,
  (r) => `${r.pick(STATES)[0]} ${r.pick(['Board of Pharmacy', 'Public Utilities Commission', 'Gaming Commission', 'Insurance Bureau', 'Securities Division', 'Medical Board', 'Food Safety Authority'])}`,
  (r) => `Office of ${r.pick(['Product Safety', 'Workplace Standards', 'Trade Practices', 'Drug Quality', 'Pipeline Integrity', 'Privacy Oversight'])}, ${r.pick(STATES)[0]} ${r.pick(['Commerce Agency', 'Health Authority', 'Revenue Department'])}`,
];
function agency(rng) {
  return rng.pick(AGENCY_FORMS)(rng);
}

// ---------------------------------------------------------------------------------------------
// A neutral document model, rendered to PDF, DOCX or plain text.
//   { k: 'head', name, lines }       letterhead
//   { k: 'line', text, bold? }       one short line
//   { k: 'lines', lines }            a block of short lines (an address)
//   { k: 'title', text }             a centred heading
//   { k: 'p', text }                 a paragraph
//   { k: 'fields', pairs }           label: value lines
//   { k: 'table', columns, rows }    columns: [header, weight]
//   { k: 'space' }

function renderPdf(model) {
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 54, bottom: 60 }, keep: model.keep });
  for (const block of model.blocks) {
    switch (block.k) {
      case 'head': {
        flow.paragraph(block.name, { face: 'sans-bold', size: 15, after: 2 });
        for (const line of block.lines) flow.paragraph(line, { face: 'sans', size: 8.5, after: 0, grey: 0.2 });
        flow.rule({ before: 6, after: 12 });
        break;
      }
      case 'line': flow.paragraph(block.text, { face: block.bold ? 'sans-bold' : 'serif', after: 2 }); break;
      case 'lines': for (const line of block.lines) flow.paragraph(line, { after: 0 }); flow.space(8); break;
      case 'title': flow.paragraph(block.text, { face: 'sans-bold', size: 12.5, align: 'center', before: 4, after: 10 }); break;
      case 'p': flow.paragraph(block.text); break;
      case 'fields': flow.fields(block.pairs, { size: 10 }); break;
      case 'table': flow.table(block.columns.map(([header, width, align]) => ({ header, width, align })), block.rows, { size: 9 }); break;
      case 'space': flow.space(10); break;
      default: throw new Error(`unknown block ${block.k}`);
    }
  }
  const pages = flow.finish();
  return { bytes: buildPdf(pages), text: pages.map((page) => pageText(page)), pages: pages.length };
}

function renderDocx(model) {
  const blocks = [];
  for (const block of model.blocks) {
    switch (block.k) {
      case 'head': blocks.push({ type: 'paragraph', runs: [{ text: block.name, bold: true }] }); for (const line of block.lines) blocks.push({ type: 'paragraph', text: line }); break;
      case 'line': blocks.push({ type: 'paragraph', runs: [{ text: block.text, bold: Boolean(block.bold) }] }); break;
      case 'lines': for (const line of block.lines) blocks.push({ type: 'paragraph', text: line }); break;
      case 'title': blocks.push({ type: 'heading', level: 1, text: block.text }); break;
      case 'p': blocks.push({ type: 'paragraph', text: block.text }); break;
      case 'fields': for (const [label, value] of block.pairs) blocks.push({ type: 'paragraph', runs: [{ text: `${label} `, bold: true }, { text: value }] }); break;
      case 'table': blocks.push({ type: 'table', rows: [block.columns.map(([header]) => header), ...block.rows] }); break;
      case 'space': blocks.push({ type: 'paragraph', text: '' }); break;
      default: throw new Error(`unknown block ${block.k}`);
    }
  }
  const bytes = docx({ blocks, title: '', creator: 'ATLAS research' });
  return { bytes, text: [model.blocks.map(plainBlock).join('\n')], pages: 1 };
}

function plainBlock(block) {
  switch (block.k) {
    case 'head': return [block.name, ...block.lines, ''].join('\n');
    case 'line': return block.text;
    case 'lines': return [...block.lines, ''].join('\n');
    case 'title': return `\n${block.text}\n`;
    case 'p': return `${block.text}\n`;
    case 'fields': return block.pairs.map(([label, value]) => `${label} ${value}`).join('\n');
    case 'table': return [block.columns.map(([header]) => header).join('    '), ...block.rows.map((row) => row.join('    '))].join('\n');
    case 'space': return '';
    default: throw new Error(`unknown block ${block.k}`);
  }
}

function renderText(model) {
  const text = model.blocks.map(plainBlock).join('\n');
  return { bytes: Buffer.from(`${text}\n`, 'utf8'), text: [text], pages: 1 };
}

const RENDERERS = { pdf: renderPdf, docx: renderDocx, txt: renderText };
const TEXT_LAYER = { pdf: 'native', docx: 'office', txt: 'text' };

// ---------------------------------------------------------------------------------------------
// Families. Each returns { model, gold, title, kind, categories, notes }.

const AGENCY_LETTER_TYPES = [
  ['Warning Letter', 'violations observed during an inspection of your facility'],
  ['Closing Letter', 'our review of the matter is now closed and no further action is recommended at this time'],
  ['Deficiency Letter', 'deficiencies identified in your application'],
  ['Approval Letter', 'your application has been approved subject to the conditions below'],
  ['Information Request Letter', 'we request that you provide the information listed below'],
  ['Determination Letter', 'the Office has made a determination on your request'],
  ['Untitled Letter', 'certain promotional statements appear to be false or misleading'],
  ['Acknowledgment Letter', 'we acknowledge receipt of your submission'],
];

function agencyLetter(rng) {
  const sender = agency(rng);
  const recipient = company(rng);
  const contact = person(rng);
  const signer = person(rng);
  const counselFirm = rng.chance(0.3) ? lawFirm(rng) : null;
  const counsel = counselFirm ? person(rng) : null;
  const ccs = Array.from({ length: rng.int(0, 3) }, () => person(rng));
  const [type, gist] = rng.pick(AGENCY_LETTER_TYPES);
  const date = isoDate(rng);
  const inspection = addDays(date, -rng.int(20, 200));
  const respondBy = addDays(date, rng.int(10, 30));
  const priorLetter = addDays(date, -rng.int(250, 900));
  const fileNo = `${rng.pick(['CMS', 'ENF', 'REV', 'FILE', 'CASE'])}-${rng.int(2019, 2027)}-${rng.int(1000, 99999)}`;
  const addressLines = address(rng);
  const agencyAddress = address(rng);
  const dateText = printDate(rng, date);
  const heading = rng.pick(['upper', 'title', 'inline']);
  const blocks = [{ k: 'head', name: sender, lines: [agencyAddress[0], agencyAddress[1]] }];
  blocks.push({ k: 'line', text: dateText });
  blocks.push({ k: 'space' });
  if (rng.chance(0.4)) blocks.push({ k: 'line', text: rng.pick(['VIA ELECTRONIC MAIL', 'BY CERTIFIED MAIL', 'VIA OVERNIGHT DELIVERY']), bold: true });
  if (counselFirm) blocks.push({ k: 'lines', lines: [`${counsel}, Esq.`, counselFirm, ...address(rng).slice(0, 2)] });
  else blocks.push({ k: 'lines', lines: [contact, rng.pick(['Chief Executive Officer', 'President', 'Director of Quality', 'General Counsel', 'Compliance Manager']), recipient, ...addressLines.slice(0, 2)] });
  blocks.push({ k: 'line', text: `Re: ${counselFirm ? `${recipient}, ` : ''}${rng.pick(['File No.', 'Reference', 'Matter No.'])} ${fileNo}`, bold: true });
  if (heading === 'upper') blocks.push({ k: 'title', text: type.toUpperCase() });
  else if (heading === 'title') blocks.push({ k: 'title', text: type });
  const salutation = counselFirm ? `Dear ${counsel.split(' ').slice(-1)[0] === counsel ? counsel : `Counsel`}:` : `Dear ${contact}:`;
  blocks.push({ k: 'p', text: salutation });
  const opener = heading === 'inline'
    ? `This ${type.toLowerCase().replace(/^./, (c) => c.toUpperCase())} is to inform ${counselFirm ? `your client, ${recipient},` : recipient} that ${gist}.`
    : `The ${sender} writes to inform ${counselFirm ? `your client, ${recipient},` : recipient} that ${gist}.`;
  blocks.push({ k: 'p', text: opener });
  blocks.push({ k: 'p', text: `Staff conducted an inspection on ${printDate(rng, inspection)} at the ${addressLines[1].split(',')[0]} site. During that inspection staff spoke with ${person(rng)} and ${person(rng)}, who provided records on request. This letter also refers to the Office's earlier correspondence dated ${printDate(rng, priorLetter)}.` });
  if (rng.chance(0.6)) {
    blocks.push({ k: 'table', columns: [['Item', 0.15], ['Observation', 0.85]], rows: Array.from({ length: rng.int(2, 4) }, (_, index) => [String(index + 1), rng.pick(['Records of cleaning were incomplete for the period reviewed.', 'Labels omitted the required lot code.', 'Training records were not available for two operators.', 'Temperature logs showed excursions without corrective action.', 'Complaint files lacked a documented investigation.'])]) });
  }
  blocks.push({ k: 'p', text: `Please respond in writing by ${printDate(rng, respondBy)}, describing the steps you have taken. Direct questions to ${person(rng)}, ${rng.pick(['Compliance Officer', 'Program Analyst', 'Investigator'])}, at the address above.` });
  blocks.push({ k: 'p', text: rng.pick(['Sincerely,', 'Respectfully,', 'Very truly yours,']) });
  blocks.push({ k: 'lines', lines: [signer, rng.pick(['Director', 'Deputy Director', 'Division Chief', 'Associate Director']), sender] });
  if (ccs.length) blocks.push({ k: 'line', text: `cc: ${ccs.join('; ')}` });
  const forbidden = [[contact, 'person the letter is addressed to'], [signer, 'official who signs'], ...ccs.map((name) => [name, 'copied'])];
  if (counselFirm) forbidden.push([counsel, 'outside counsel addressed']);
  return {
    model: { blocks, keep: [sender, recipient, dateText, type, contact, signer] },
    kind: 'letter',
    title: `${type} from a regulator to a company`,
    categories: ['letter', 'competing_dates', 'irrelevant_names'],
    notes: `${type} from ${sender} to ${recipient}, dated ${dateText} under the letterhead. Inspection, prior-letter and response-by dates are traps; the addressed person, signer and copies are not parties.`,
    gold: gold({
      type,
      acceptableTypes: ['Letter'],
      date,
      role: 'issuance',
      forbiddenDates: [[inspection, 'inspection date'], [respondBy, 'response deadline'], [priorLetter, 'earlier letter']],
      parties: [recipient],
      relation: 'to',
      acceptablePartySets: [{ parties: [sender], relation: 'from' }, ...(counselFirm ? [{ parties: [counselFirm], relation: 'to' }] : [])],
      roles: [[recipient, 'recipient'], [sender, 'issuer']],
      forbiddenParties: forbidden,
      readiness: 'ready',
      dateText: [dateText],
    }),
  };
}

const COMPANY_LETTER_TYPES = ['Comment Letter', 'Request Letter', 'Response Letter', 'Petition Letter', 'Notification Letter'];

function companyLetter(rng) {
  const sender = company(rng);
  const recipient = agency(rng);
  const official = person(rng);
  const signer = person(rng);
  const type = rng.pick(COMPANY_LETTER_TYPES);
  const date = isoDate(rng);
  const ruleDate = addDays(date, -rng.int(30, 400));
  const meeting = addDays(date, rng.int(5, 60));
  const dateText = printDate(rng, date);
  const ccs = Array.from({ length: rng.int(0, 2) }, () => person(rng));
  const peer = company(rng);
  const blocks = [{ k: 'head', name: sender, lines: address(rng).slice(0, 2) }];
  blocks.push({ k: 'line', text: dateText });
  blocks.push({ k: 'space' });
  blocks.push({ k: 'lines', lines: [official, rng.pick(['Director', 'Secretary', 'Chief Counsel', 'Commissioner']), recipient, ...address(rng).slice(0, 2)] });
  blocks.push({ k: 'line', text: `Re: ${rng.pick(['Proposed rule published', 'Notice issued', 'Guidance dated'])} ${printDate(rng, ruleDate)}`, bold: true });
  if (rng.chance(0.6)) blocks.push({ k: 'title', text: type.toUpperCase() });
  blocks.push({ k: 'p', text: `Dear ${official}:` });
  blocks.push({ k: 'p', text: `${sender} respectfully submits this ${type.toLowerCase()} to the ${recipient}. We operate ${rng.int(3, 40)} facilities and employ about ${rng.int(120, 9000)} people.` });
  blocks.push({ k: 'p', text: `Our concerns are shared by other operators, including ${peer}, with whom we met on ${printDate(rng, addDays(date, -rng.int(5, 90)))}. We would welcome a meeting with staff on or after ${printDate(rng, meeting)}.` });
  blocks.push({ k: 'p', text: 'Sincerely,' });
  blocks.push({ k: 'lines', lines: [signer, rng.pick(['General Counsel', 'Chief Executive Officer', 'Vice President, Regulatory Affairs']), sender] });
  if (ccs.length) blocks.push({ k: 'line', text: `cc: ${ccs.join('; ')}` });
  return {
    model: { blocks, keep: [sender, recipient, dateText, official, signer] },
    kind: 'letter',
    title: `${type} from a company to a regulator`,
    categories: ['letter', 'competing_dates', 'irrelevant_names'],
    notes: `${type} from ${sender} to ${recipient}, dated ${dateText}. The referenced rule's date and the proposed meeting date are traps; the official, signer, copies and the peer company are not parties.`,
    gold: gold({
      type,
      acceptableTypes: ['Letter'],
      date,
      role: 'issuance',
      forbiddenDates: [[ruleDate, 'date of the rule it comments on'], [meeting, 'proposed meeting']],
      parties: [recipient],
      relation: 'to',
      acceptablePartySets: [{ parties: [sender], relation: 'from' }],
      roles: [[recipient, 'recipient'], [sender, 'sender']],
      forbiddenParties: [[official, 'official addressed'], [signer, 'signs'], [peer, 'other operator mentioned'], ...ccs.map((name) => [name, 'copied'])],
      readiness: 'ready',
      dateText: [dateText],
    }),
  };
}

const COURT_NOTICES = ['Notice of Appearance', 'Notice of Removal', 'Notice of Motion', 'Notice of Voluntary Dismissal',
  'Notice of Settlement', 'Notice of Appeal', 'Notice of Hearing', 'Notice of Deposition'];

function courtNotice(rng) {
  const plaintiff = rng.chance(0.4) ? person(rng) : company(rng);
  const defendant = company(rng);
  const type = rng.pick(COURT_NOTICES);
  const date = isoDate(rng);
  const complaintFiled = addDays(date, -rng.int(30, 500));
  const hearing = addDays(date, rng.int(14, 90));
  const attorney = person(rng);
  const firm = lawFirm(rng);
  const judge = person(rng);
  const caseNo = `${rng.int(1, 9)}:${rng.int(19, 27)}-cv-${String(rng.int(100, 9999)).padStart(5, '0')}`;
  const [state] = rng.pick(STATES);
  const court = rng.pick([`IN THE SUPERIOR COURT OF ${state.toUpperCase()}`, `UNITED STATES DISTRICT COURT FOR THE DISTRICT OF ${state.toUpperCase()}`, `CIRCUIT COURT OF ${rng.pick(CITIES).toUpperCase()} COUNTY`]);
  const dateText = printDate(rng, date);
  const role = rng.pick(['Plaintiff', 'Defendant']);
  const blocks = [{ k: 'title', text: court }];
  blocks.push({ k: 'lines', lines: [`${plaintiff},`, '        Plaintiff,', 'v.', `${defendant},`, '        Defendant.'] });
  blocks.push({ k: 'line', text: `Case No. ${caseNo}` });
  blocks.push({ k: 'line', text: `Hon. ${judge}` });
  blocks.push({ k: 'title', text: type.toUpperCase() });
  blocks.push({ k: 'p', text: `PLEASE TAKE NOTICE that ${role} ${role === 'Plaintiff' ? plaintiff : defendant}, by its undersigned counsel, hereby files this ${type} in the above-captioned action, commenced by the complaint filed on ${printDate(rng, complaintFiled)}.` });
  if (/Hearing|Motion|Deposition/.test(type)) blocks.push({ k: 'p', text: `The matter will be heard on ${printDate(rng, hearing)} at ${rng.int(9, 11)}:00 a.m., or as soon thereafter as counsel may be heard.` });
  else blocks.push({ k: 'p', text: `Any response is due on or before ${printDate(rng, hearing)} under the applicable rules.` });
  blocks.push({ k: 'line', text: `Dated: ${dateText}` });
  blocks.push({ k: 'lines', lines: ['Respectfully submitted,', firm, `By: /s/ ${attorney}`, attorney, `Attorneys for ${role} ${role === 'Plaintiff' ? plaintiff : defendant}`] });
  blocks.push({ k: 'p', text: `CERTIFICATE OF SERVICE. I certify that on ${dateText} I served this document on all counsel of record. /s/ ${attorney}` });
  return {
    model: { blocks, keep: [plaintiff, defendant, dateText, type, firm, attorney] },
    kind: 'notice',
    title: `${type} in a civil case`,
    categories: ['notice', 'competing_dates', 'layout_parties'],
    notes: `${type} filed in ${plaintiff} v. ${defendant}, dated ${dateText}. The complaint's filing date and the hearing or response date are traps; counsel, firm and judge are not parties.`,
    gold: gold({
      type,
      acceptableTypes: ['Notice'],
      date,
      role: 'filing',
      forbiddenDates: [[complaintFiled, 'complaint filed'], [hearing, 'hearing or response date']],
      parties: [plaintiff, defendant],
      relation: 'between',
      roles: [[plaintiff, 'other'], [defendant, 'other']],
      forbiddenParties: [[attorney, 'counsel'], [firm, 'law firm'], [judge, 'judge']],
      readiness: 'ready',
      dateText: [dateText],
    }),
  };
}

const ENFORCEMENT_NOTICES = ['Notice of Violation', 'Notice of Proposed Penalty', 'Notice of Intent to Revoke',
  'Notice of Apparent Liability', 'Notice of Noncompliance', 'Notice of Probable Violation', 'Notice of Suspension'];

function enforcementNotice(rng) {
  const sender = agency(rng);
  const respondent = company(rng);
  const type = rng.pick(ENFORCEMENT_NOTICES);
  const date = isoDate(rng);
  const violation = addDays(date, -rng.int(30, 300));
  const due = addDays(date, rng.int(15, 45));
  const officer = person(rng);
  const agent = person(rng);
  const amount = rng.int(5, 900) * 100000;
  const dateText = printDate(rng, date);
  const noticeNo = `${rng.pick(['NOV', 'NAL', 'NPV', 'EN'])}-${rng.int(2019, 2027)}-${rng.int(100, 9999)}`;
  const blocks = [{ k: 'head', name: sender, lines: address(rng).slice(0, 2) }];
  blocks.push({ k: 'title', text: type.toUpperCase() });
  blocks.push({ k: 'fields', pairs: [['Notice No.:', noticeNo], ['Date Issued:', dateText], ['Respondent:', respondent], ['Registered Agent:', agent], ['Facility:', address(rng)[0]]] });
  blocks.push({ k: 'p', text: `The ${sender} has reason to believe that ${respondent} ("Respondent") violated the applicable regulations on or about ${printDate(rng, violation)}, as described below.` });
  blocks.push({ k: 'table', columns: [['Count', 0.15], ['Description', 0.6], ['Proposed amount', 0.25, 'right']], rows: [['1', rng.pick(['Failure to maintain required records', 'Operating without a current permit', 'Late filing of a required report', 'Exceeding a permitted discharge limit']), money(amount)]] });
  blocks.push({ k: 'p', text: `Respondent may pay the proposed amount or request a hearing by ${printDate(rng, due)}. Failure to respond may result in a final order.` });
  blocks.push({ k: 'lines', lines: [officer, rng.pick(['Enforcement Officer', 'Chief, Enforcement Bureau', 'Regional Administrator']), sender] });
  return {
    model: { blocks, keep: [sender, respondent, dateText, type, officer, agent] },
    kind: 'notice',
    title: `${type} for a company`,
    categories: ['notice', 'form', 'competing_dates', 'key_value'],
    notes: `${type} issued by ${sender} for ${respondent}, issued ${dateText}. The violation date and the response deadline are traps; the officer and registered agent are not parties.`,
    gold: gold({
      type,
      acceptableTypes: ['Notice'],
      date,
      role: 'notice',
      acceptableDates: [],
      forbiddenDates: [[violation, 'alleged violation'], [due, 'response deadline']],
      parties: [respondent],
      relation: 'for',
      acceptablePartySets: [{ parties: [respondent], relation: 'to' }],
      roles: [[respondent, 'subject'], [sender, 'issuer']],
      forbiddenParties: [[officer, 'signs'], [agent, 'registered agent']],
      readiness: 'ready',
      dateText: [dateText],
    }),
  };
}

const SERVICES = ['Consulting services', 'Freight charges', 'Laboratory testing', 'Equipment rental', 'Software subscription',
  'Maintenance visit', 'Cleaning services', 'Packaging materials', 'Calibration', 'Data storage', 'Training session', 'Courier delivery'];

function lineItems(rng) {
  const rows = [];
  let total = 0;
  for (let index = 0; index < rng.int(1, 6); index += 1) {
    const quantity = rng.int(1, 40);
    const unit = rng.int(10, 2500) * 100 + rng.pick([0, 50, 99]);
    total += quantity * unit;
    rows.push([rng.pick(SERVICES), String(quantity), money(unit), money(quantity * unit)]);
  }
  return { rows, total };
}

function invoice(rng) {
  const vendor = company(rng);
  const customer = company(rng);
  const contact = person(rng);
  const date = isoDate(rng);
  const due = addDays(date, rng.pick([15, 30, 45, 60]));
  const periodStart = addDays(date, -rng.int(30, 60));
  const periodEnd = addDays(periodStart, 29);
  const po = `PO-${rng.int(10000, 99999)}`;
  const number = `${rng.pick(['INV', 'IN', '', 'A'])}${rng.int(1000, 999999)}`;
  const dateText = printDate(rng, date);
  const { rows, total } = lineItems(rng);
  const shipTo = rng.chance(0.3) ? company(rng) : null;
  const blocks = [{ k: 'head', name: vendor, lines: [...address(rng).slice(0, 2), `Tax ID ${rng.int(10, 99)}-${rng.int(1000000, 9999999)}`] }];
  const type = rng.pick(['Invoice', 'Invoice', 'Tax Invoice', 'Service Invoice']);
  blocks.push({ k: 'title', text: rng.chance(0.5) ? type.toUpperCase() : type });
  blocks.push({ k: 'fields', pairs: [['Invoice No.:', number], ['Invoice Date:', dateText], ['Due Date:', printDate(rng, due)], ['Customer PO:', po], ['Service Period:', `${printDate(rng, periodStart)} - ${printDate(rng, periodEnd)}`]] });
  blocks.push({ k: 'lines', lines: ['Bill To:', customer, `Attn: ${contact}`, ...address(rng).slice(0, 2)] });
  if (shipTo) blocks.push({ k: 'lines', lines: ['Ship To:', shipTo, ...address(rng).slice(0, 2)] });
  blocks.push({ k: 'table', columns: [['Description', 0.46], ['Qty', 0.1, 'right'], ['Unit price', 0.2, 'right'], ['Amount', 0.24, 'right']], rows: [...rows, ['Total due', '', '', money(total)]] });
  blocks.push({ k: 'p', text: `Please remit payment to ${vendor} by ${printDate(rng, due)}. Late balances accrue interest at 1.5% per month.` });
  return {
    model: { blocks, keep: [vendor, customer, dateText, contact] },
    kind: 'invoice',
    title: 'Vendor invoice',
    categories: ['invoice', 'table', 'competing_dates', 'key_value'],
    notes: `Invoice ${number} from ${vendor} to ${customer}, dated ${dateText}. The due date and the service period are traps; the attention contact${shipTo ? ' and the ship-to company' : ''} are not the issuer.`,
    gold: gold({
      type,
      acceptableTypes: type === 'Invoice' ? [] : ['Invoice'],
      date,
      role: 'invoice',
      forbiddenDates: [[due, 'due date'], [periodStart, 'service period start'], [periodEnd, 'service period end']],
      parties: [vendor],
      relation: 'from',
      acceptablePartySets: [{ parties: [vendor, customer], relation: 'between' }],
      roles: [[vendor, 'vendor'], [customer, 'customer']],
      forbiddenParties: [[contact, 'attention contact'], ...(shipTo ? [[shipTo, 'ship-to location']] : [])],
      readiness: 'ready',
      dateText: [dateText],
    }),
  };
}

function receipt(rng) {
  const vendor = company(rng);
  const payer = rng.chance(0.5) ? person(rng) : company(rng);
  const date = isoDate(rng);
  const invoiceDate = addDays(date, -rng.int(10, 60));
  const amount = rng.int(20, 90000) * 100;
  const dateText = printDate(rng, date);
  const clerk = person(rng);
  const type = rng.pick(['Payment Receipt', 'Receipt', 'Official Receipt', 'Donation Receipt']);
  const blocks = [{ k: 'head', name: vendor, lines: address(rng).slice(0, 2) }];
  blocks.push({ k: 'title', text: type.toUpperCase() });
  blocks.push({ k: 'fields', pairs: [['Receipt No.:', `R-${rng.int(10000, 999999)}`], ['Date:', dateText], ['Received from:', payer], ['Amount:', money(amount)], ['Method:', rng.pick(['Check', 'Wire transfer', 'Card ending ' + rng.int(1000, 9999), 'Cash'])], ['Applied to:', `Invoice dated ${printDate(rng, invoiceDate)}`]] });
  blocks.push({ k: 'p', text: `Thank you for your payment. Received by ${clerk}.` });
  return {
    model: { blocks, keep: [vendor, payer, dateText, clerk] },
    kind: 'receipt',
    title: 'Payment receipt',
    categories: ['form', 'key_value', 'competing_dates'],
    notes: `${type} from ${vendor} for a payment by ${payer}, dated ${dateText}. The paid invoice's date is a trap; the clerk is not a party.`,
    gold: gold({
      type,
      acceptableTypes: ['Receipt'],
      date,
      role: 'issuance',
      forbiddenDates: [[invoiceDate, 'date of the invoice paid']],
      parties: [vendor],
      relation: 'from',
      acceptablePartySets: [{ parties: [payer], relation: 'for' }],
      roles: [[vendor, 'issuer'], [payer, 'payer']],
      forbiddenParties: [[clerk, 'clerk']],
      readiness: 'ready',
      dateText: [dateText],
    }),
  };
}

function statement(rng) {
  const vendor = company(rng);
  const customer = rng.chance(0.4) ? person(rng) : company(rng);
  const date = isoDate(rng);
  const periodEnd = addDays(date, -rng.int(1, 5));
  const periodStart = addDays(periodEnd, -29);
  const due = addDays(date, rng.int(15, 25));
  const dateText = printDate(rng, date);
  const type = rng.pick(['Account Statement', 'Statement of Account', 'Billing Statement', 'Monthly Statement']);
  const rows = Array.from({ length: rng.int(2, 6) }, () => [printDate(rng, addDays(periodStart, rng.int(0, 28))), rng.pick(['Payment received', 'Service charge', 'Usage', 'Credit memo', 'Late fee']), money(rng.int(5, 5000) * 100)]);
  const blocks = [{ k: 'head', name: vendor, lines: address(rng).slice(0, 2) }];
  blocks.push({ k: 'title', text: type });
  blocks.push({ k: 'fields', pairs: [['Statement Date:', dateText], ['Account:', `${rng.int(100000, 9999999)}`], ['Customer:', customer], ['Period:', `${printDate(rng, periodStart)} to ${printDate(rng, periodEnd)}`], ['Payment Due:', printDate(rng, due)]] });
  blocks.push({ k: 'table', columns: [['Date', 0.3], ['Activity', 0.45], ['Amount', 0.25, 'right']], rows });
  return {
    model: { blocks, keep: [vendor, customer, dateText] },
    kind: 'statement',
    title: 'Customer account statement',
    categories: ['statement', 'table', 'competing_dates', 'date_in_table'],
    notes: `${type} from ${vendor} to ${customer}, dated ${dateText}. The period, transaction and due dates are traps.`,
    gold: gold({
      type,
      acceptableTypes: ['Statement'],
      date,
      role: 'issuance',
      forbiddenDates: [[due, 'payment due'], [periodStart, 'period start'], [periodEnd, 'period end']],
      parties: [vendor],
      relation: 'from',
      acceptablePartySets: [{ parties: [customer], relation: 'for' }],
      roles: [[vendor, 'issuer'], [customer, 'customer']],
      forbiddenParties: [],
      readiness: 'ready',
      dateText: [dateText],
    }),
  };
}

// Weighted toward the kinds the real sets hold most of, and where the 9B fails most (letters).
const FAMILIES = [
  ['agency-letter', agencyLetter, 30],
  ['company-letter', companyLetter, 12],
  ['court-notice', courtNotice, 12],
  ['enforcement-notice', enforcementNotice, 14],
  ['invoice', invoice, 18],
  ['receipt', receipt, 6],
  ['statement', statement, 8],
];

function pickFamily(rng) {
  const total = FAMILIES.reduce((sum, [, , weight]) => sum + weight, 0);
  let roll = rng.int(1, total);
  for (const family of FAMILIES) {
    roll -= family[2];
    if (roll <= 0) return family;
  }
  return FAMILIES[0];
}

// ---------------------------------------------------------------------------------------------
// Test-set entity guard.

async function testNames() {
  const names = new Set();
  const add = (value) => { if (typeof value === 'string' && value.trim()) names.add(value.trim().toLowerCase()); };
  const files = [join(REPO, 'bench/gold.json')];
  for (let index = 0; index <= 7; index += 1) files.push(join(REPO, `scripts/realdocs-eval/sample${index ? `-heldout${index === 1 ? '' : index}` : ''}.json`));
  for (const file of files) {
    let data;
    try { data = JSON.parse(await readFile(file, 'utf8')); } catch { continue; }
    for (const document of data.documents ?? data) {
      const g = document.gold ?? {};
      (g.parties ?? []).forEach(add);
      (g.party_roles ?? []).forEach((role) => add(role.name));
      (g.forbidden_parties ?? []).forEach((party) => add(party.name));
      (g.acceptable_party_sets ?? []).forEach((set) => (set.parties ?? []).forEach(add));
    }
  }
  return names;
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

async function main() {
  const args = process.argv.slice(2);
  const value = (flag, fallback) => { const at = args.indexOf(flag); return at >= 0 ? args[at + 1] : fallback; };
  const out = resolve(value('--out', join(REPO, 'target/atlas-dataset')));
  const count = Number(value('--count', '200'));
  const split = value('--split', 'train');
  const seed = value('--seed', 'atlas-2026-11');
  const formats = (value('--formats', 'pdf,pdf,pdf,docx,txt')).split(',');
  const banned = await testNames();
  const avoided = new Set();
  args.forEach((flag, at) => {
    if (flag === '--avoid') avoided.add(args[at + 1]);
  });
  const reused = new Set();
  for (const file of avoided) {
    for (const document of JSON.parse(await readFile(resolve(file), 'utf8')).documents) {
      document.gold.parties.forEach((name) => reused.add(name.toLowerCase()));
    }
  }
  let overlaps = 0;
  await mkdir(out, { recursive: true });
  const documents = [];
  const files = [];
  let collisions = 0;
  for (let index = 0; documents.length < count; index += 1) {
    const rng = Rng.from(`${seed}/${split}/${index}`);
    const [family, build] = pickFamily(rng.fork('family'));
    const built = build(rng.fork('body'));
    const names = [...built.gold.parties, ...built.gold.party_roles.map((role) => role.name), ...built.gold.forbidden_parties.map((party) => party.name)];
    if (names.some((name) => banned.has(name.toLowerCase()))) { collisions += 1; continue; }
    if (built.gold.parties.some((name) => reused.has(name.toLowerCase()))) { overlaps += 1; continue; }
    const format = rng.fork('format').pick(formats);
    const rendered = RENDERERS[format](built.model);
    // An untelling file name, as a scanner or download gives one, so the prompt shows no name hint.
    const id = `atlas-${split}-${String(index).padStart(5, '0')}`;
    const file = `scan${String(index).padStart(5, '0')}.${format}`;
    await writeFile(join(out, file), rendered.bytes);
    files.push({ file, size: rendered.bytes.length, sha256: sha256(rendered.bytes) });
    const document = entry({
      id,
      extension: format,
      title: built.title,
      kind: built.kind,
      textLayer: TEXT_LAYER[format],
      pages: rendered.pages,
      categories: [...new Set([...built.categories, ...(format === 'docx' ? ['docx'] : [])])],
      notes: built.notes,
      gold: built.gold,
    });
    document.file = file;
    document.family = family;
    documents.push(document);
  }
  const goldFile = { schema_version: 1, suite: `atlas-${split}`, notice: 'All names, organizations, addresses, identifiers, and events are fictional, generated by research/atlas-models/dataset/generate.mjs.', documents };
  await writeFile(join(out, 'gold.json'), `${JSON.stringify(goldFile, null, 1)}\n`);
  await writeFile(join(out, 'manifest.json'), `${JSON.stringify({ schema_version: 1, suite: `atlas-${split}`, files }, null, 1)}\n`);
  const byFamily = {};
  for (const document of documents) byFamily[document.family] = (byFamily[document.family] ?? 0) + 1;
  process.stdout.write(`wrote ${documents.length} documents to ${out} (${collisions} skipped for a test-set name, ${overlaps} for a party in an avoided set) ${JSON.stringify(byFamily)}\n`);
}

await main();
