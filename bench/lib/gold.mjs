/// Builds gold entries in the exact shape and key order of bench/gold.json,
/// and refuses anything outside the schema's vocabularies, so a builder
/// cannot quietly invent a relation, role, or category the runner does not
/// score.
import { isRealDate } from './format.mjs';

export const TEXT_LAYERS = ['native', 'scan', 'mixed', 'ocr_corrupted', 'office', 'sheet', 'text', 'email'];
export const DATE_ROLES = ['effective', 'execution', 'invoice', 'notice', 'termination', 'amendment', 'filing', 'issuance', 'other'];
export const RELATIONS = ['between', 'for', 'with', 'from', 'to', 'none'];
export const READINESS = ['ready', 'needs_review', 'either'];
export const PARTY_ROLES = [
  'issuer', 'customer', 'recipient', 'subject', 'sender', 'counterparty', 'buyer', 'seller', 'lender', 'borrower',
  'landlord', 'tenant', 'employer', 'employee', 'patient', 'payer', 'provider', 'client', 'firm', 'fund', 'investor',
  'licensor', 'licensee', 'assignor', 'assignee', 'other',
];
export const CATEGORIES = [
  'simple_digital', 'complex_pdf', 'multi_column', 'form', 'invoice', 'table', 'statement', 'purchase_order', 'contract',
  'amendment', 'sow', 'notice', 'hr', 'healthcare', 'financial', 'letter', 'presentation', 'spreadsheet', 'image_only_scan',
  'mixed_scan', 'rotated_scan', 'low_resolution_scan', 'noisy_scan', 'ocr_corrupted', 'pages_5', 'pages_10', 'pages_25',
  'pages_50', 'pages_100', 'middle_fact', 'competing_dates', 'referenced_agreement', 'irrelevant_names', 'date_in_table',
  'layout_parties', 'unusual', 'information_dense', 'email', 'docx', 'pptx', 'xlsx', 'csv', 'tiff', 'png',
  'key_value', 'stream_order', 'rotated_page', 'image_region', 'ocr_critical_fields',
];
/// The routes the worker reads a PDF page by.
export const ROUTES = ['fast', 'layout', 'ocr', 'ocr_regions'];

function check(condition, message) {
  if (!condition) throw new Error(message);
}

function checkDate(date, where) {
  check(isRealDate(date), `${where}: ${date} is not a real ISO date`);
}

/// The `gold` object. Arguments use short names; output uses the schema's.
export function gold({
  type,
  acceptableTypes = [],
  date,
  acceptableDates = [],
  role,
  forbiddenDates = [],
  parties = [],
  relation,
  acceptablePartySets = [],
  roles = [],
  forbiddenParties = [],
  facts = [],
  forbiddenFacts = [],
  subjectTerms = [],
  readiness,
  dateText = [],
  partyText = null,
}) {
  check(type === null || (typeof type === 'string' && type.length > 0), 'document_type must be a string or null');
  // A trap computed from data (the largest deposit, a milestone) can land on
  // a date already listed; the first reason given wins.
  forbiddenDates = forbiddenDates.filter(([value], index) => forbiddenDates.findIndex(([other]) => other === value) === index);
  if (date !== null) checkDate(date, 'document_date');
  acceptableDates.forEach((value) => checkDate(value, 'acceptable_dates'));
  forbiddenDates.forEach(([value]) => checkDate(value, 'forbidden_dates'));
  check(role === null || DATE_ROLES.includes(role), `unknown date_role ${role}`);
  check((date === null) === (role === null), 'date_role is set exactly when document_date is');
  check(RELATIONS.includes(relation), `unknown relation ${relation}`);
  check(relation !== 'between' || parties.length === 2, 'between takes two parties');
  check(!['for', 'from', 'to', 'with'].includes(relation) || parties.length === 1, `${relation} takes one party`);
  check(relation !== 'none' || parties.length === 0, 'none takes no parties');
  for (const set of acceptablePartySets) {
    check(RELATIONS.includes(set.relation), `unknown relation ${set.relation}`);
    check(set.relation !== 'between' || set.parties.length === 2, 'acceptable between takes two parties');
  }
  roles.forEach(([, value]) => check(PARTY_ROLES.includes(value), `unknown party role ${value}`));
  check(READINESS.includes(readiness), `unknown readiness ${readiness}`);
  check(!forbiddenDates.some(([value]) => value === date || acceptableDates.includes(value)), 'a forbidden date is also acceptable');
  const forbiddenNames = forbiddenParties.map(([name]) => name);
  check(!parties.some((name) => forbiddenNames.includes(name)), 'a gold party is forbidden');
  check(date === null || dateText.length > 0, 'a dated document needs date evidence');
  const evidenceParties = partyText ?? Object.fromEntries(parties.map((name) => [name, [name]]));
  for (const name of parties) check(evidenceParties[name]?.length, `no evidence for party ${name}`);
  return {
    document_type: type,
    acceptable_types: acceptableTypes,
    document_date: date,
    acceptable_dates: acceptableDates,
    date_role: role,
    forbidden_dates: forbiddenDates.map(([value, why]) => ({ date: value, why })),
    parties,
    party_relation: relation,
    acceptable_party_sets: acceptablePartySets.map((set) => ({ parties: set.parties, relation: set.relation })),
    party_roles: roles.map(([name, value]) => ({ name, role: value })),
    forbidden_parties: forbiddenParties.map(([name, why]) => ({ name, why })),
    description_facts: facts.map((group) => (Array.isArray(group) ? group : [group])),
    description_forbidden: forbiddenFacts,
    subject_terms: subjectTerms,
    expected_readiness: readiness,
    evidence: { date_text: dateText, party_text: evidenceParties },
  };
}

/// The `structure` block: what the page's layout plainly says, for scoring
/// extraction. `readingOrder` is distinctive phrases in the order a person
/// reads them, each printed once; `tables` lists each table's rows (header
/// first; '' for a blank cell); `keyValues` is [label, value] pairs, the
/// label as printed without its colon; `routes` maps a page number to the
/// route it should take, given only where that is not a judgement call.
/// Parts left empty are left out.
export function structure({ readingOrder = [], tables = [], keyValues = [], routes = {} }) {
  check(readingOrder.length !== 1, 'a reading order needs at least two snippets');
  check(new Set(readingOrder).size === readingOrder.length, 'a reading order repeats a snippet');
  for (const rows of tables) {
    check(Array.isArray(rows) && rows.length >= 2, 'a table needs a header and a row');
    for (const row of rows) check(row.every((cell) => typeof cell === 'string') && row.some(Boolean), 'a table row is strings, not all blank');
  }
  for (const [key, value] of keyValues) check(key && value && !key.endsWith(':'), `key value ${key}: label without its colon, and a value`);
  for (const [page, route] of Object.entries(routes)) {
    check(Number.isInteger(Number(page)) && Number(page) > 0, `route for page ${page}`);
    check(ROUTES.includes(route), `unknown route ${route}`);
  }
  const block = {};
  if (readingOrder.length) block.reading_order = readingOrder;
  if (tables.length) block.tables = tables.map((rows) => ({ rows }));
  if (keyValues.length) block.key_values = keyValues.map(([key, value]) => ({ key, value }));
  if (Object.keys(routes).length) block.expected_routes = routes;
  return block;
}

/// A complete document entry. `text` is the generated text model (one string
/// per page); it is returned beside the entry for tests and never written to
/// gold.json. `recording: 'pending'` marks a document added before anyone
/// recorded it live; replay leaves it unscored until then.
export function entry({ id, extension, title, kind, textLayer, pages, categories, notes, gold: goldEntry, ocrTruth = null, cleanText = undefined, structure: layout = null, recording = null }) {
  check(/^[a-z0-9]+(-[a-z0-9]+)*$/.test(id), `id ${id} is not kebab-case`);
  check(TEXT_LAYERS.includes(textLayer), `unknown text_layer ${textLayer}`);
  categories.forEach((category) => check(CATEGORIES.includes(category), `${id}: unknown category ${category}`));
  check(new Set(categories).size === categories.length, `${id}: duplicate category`);
  check(Number.isInteger(pages) && pages > 0, `${id}: pages must be a positive integer`);
  check(typeof notes === 'string' && notes.length > 40, `${id}: notes must explain the trap`);
  if (['scan', 'mixed'].includes(textLayer) || ocrTruth) {
    check(ocrTruth && ocrTruth.pages.length > 0, `${id}: scanned pages need ocr_truth`);
  }
  const document = {
    id,
    file: `${id}.${extension}`,
    title,
    kind,
    format: extension,
    text_layer: textLayer,
    pages,
    categories,
    notes,
    gold: goldEntry,
    ocr_truth: ocrTruth,
  };
  if (cleanText !== undefined) document.clean_text = cleanText;
  if (layout) {
    for (const page of Object.keys(layout.expected_routes ?? {})) check(Number(page) <= pages, `${id}: a route for page ${page} of ${pages}`);
    document.structure = layout;
  }
  if (recording !== null) {
    check(recording === 'pending', `${id}: recording is 'pending' or absent`);
    document.recording = recording;
  }
  return document;
}
