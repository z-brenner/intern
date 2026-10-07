/// Forms: a county vendor registration filled in by a supplier, and a
/// construction change order. Facts live in labelled boxes; the defining
/// date is one box among several dated ones.
import { Page } from '../lib/layout.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { amount, money, numericDate } from '../lib/format.mjs';
import { digitalPdf, result } from './common.mjs';

/// A labelled form box: a small caption in the top-left corner and the
/// entered value below it, the way fillable PDF forms print.
export function fieldBox(page, x, y, w, h, label, value, { face = 'mono', size = 9.5, labelSize = 6.5 } = {}) {
  page.rect(x, y, w, h, { fill: null, stroke: 0.25, width: 0.5 });
  page.text(x + 3, y + 8, label, { face: 'sans', size: labelSize, grey: 0.25 });
  if (value) page.text(x + 5, y + h - 5, value, { face, size });
}

/// A check box with its caption; `checked` draws an X.
export function checkBox(page, x, y, caption, checked, { size = 8.5 } = {}) {
  page.rect(x, y - 8, 8, 8, { fill: null, stroke: 0.2, width: 0.5 });
  if (checked) page.text(x + 1.3, y - 1.2, 'X', { face: 'sans-bold', size: 8 });
  page.text(x + 12, y, caption, { face: 'sans', size });
}

function sectionBar(page, y, title) {
  page.rect(40, y, 532, 14, { fill: 0.82, stroke: null });
  page.text(44, y + 10, title, { face: 'sans-bold', size: 8.5 });
  return y + 14;
}

export function vendorRegistrationForm() {
  const id = 'vendor-registration-form';
  const vendor = 'Emberglow Coatings Ltd.';
  const county = 'Hartwell County';
  const signer = 'Rufus Gillespie';
  const signed = '2026-04-22';
  const incorporated = '2014-11-03';
  const insuranceExpires = '2026-12-31';
  const page = new Page();
  page.text(40, 46, 'HARTWELL COUNTY', { face: 'sans-bold', size: 14 });
  page.text(40, 59, 'Procurement Office - 100 Courthouse Square, Pemberly Falls, NY 12804', { size: 8, grey: 0.3 });
  page.textRight(572, 46, 'VENDOR REGISTRATION FORM', { face: 'sans-bold', size: 13 });
  page.textRight(572, 59, 'Form VR-1   Return to: vendors@hartwellcounty.example', { size: 8, grey: 0.3 });
  page.text(40, 76, 'Complete every section. Type or print in ink. Incomplete forms will be returned. All vendors must also submit a current IRS Form W-9.', { size: 7.5 });
  let y = sectionBar(page, 84, 'SECTION A - BUSINESS INFORMATION');
  fieldBox(page, 40, y, 532, 26, '1. Legal business name (as shown on your income tax return)', vendor);
  y += 26;
  fieldBox(page, 40, y, 300, 26, '2. Doing business as (DBA) / trade name, if different', 'Emberglow Industrial Finishes');
  fieldBox(page, 340, y, 232, 26, '3. Federal employer identification number (EIN)', '84-0293157');
  y += 26;
  page.rect(40, y, 532, 30, { fill: null, stroke: 0.25, width: 0.5 });
  page.text(43, y + 8, '4. Business type', { size: 6.5, grey: 0.25 });
  [['Sole proprietor', false], ['Partnership', false], ['Corporation', true], ['LLC', false], ['Nonprofit', false], ['Government', false]].forEach(([caption, checked], index) => checkBox(page, 50 + index * 86, y + 24, caption, checked));
  y += 30;
  fieldBox(page, 40, y, 180, 26, '5. State of incorporation / organization', 'Colorado');
  fieldBox(page, 220, y, 150, 26, '6. Date of incorporation', numericDate(incorporated));
  fieldBox(page, 370, y, 202, 26, '7. Years in business', '11');
  y += 26;
  fieldBox(page, 40, y, 532, 26, '8. Remit-to address (street, city, state, ZIP)', '4410 Marrowfield Road, Marrow Springs, CO 80513');
  y += 26;
  fieldBox(page, 40, y, 180, 26, '9. Telephone', '(970) 555-0151');
  fieldBox(page, 220, y, 352, 26, '10. Email for purchase orders', 'orders@emberglow-coatings.example');
  y += 30;
  y = sectionBar(page, y, 'SECTION B - CONTACTS');
  fieldBox(page, 40, y, 266, 26, '11. Sales contact (name, phone)', 'Signe Ostrowski (970) 555-0152');
  fieldBox(page, 306, y, 266, 26, '12. Accounts receivable contact (name, phone)', 'Yusuf Bellweather (970) 555-0153');
  y += 30;
  y = sectionBar(page, y, 'SECTION C - COMMODITIES OFFERED (NIGP class-item codes)');
  [['15045', 'Coatings, industrial (epoxy, urethane)'], ['15050', 'Coatings, anti-graffiti'], ['91022', 'Coating services, protective - bridges and structures']].forEach(([code, description], index) => {
    fieldBox(page, 40, y, 90, 22, index === 0 ? '13. Code' : '', code);
    fieldBox(page, 130, y, 442, 22, index === 0 ? 'Description' : '', description);
    y += 22;
  });
  y += 4;
  y = sectionBar(page, y, 'SECTION D - CERTIFICATIONS (attach certificates)');
  [['Small business', true], ['Minority-owned', false], ['Women-owned', true], ['Veteran-owned', false], ['Local (Hartwell County)', false]].forEach(([caption, checked], index) => checkBox(page, 50 + index * 104, y + 16, caption, checked));
  y += 24;
  y = sectionBar(page, y, 'SECTION E - INSURANCE');
  fieldBox(page, 40, y, 220, 26, '14. General liability carrier', 'Highmeadow Casualty Company');
  fieldBox(page, 260, y, 150, 26, '15. Policy number', 'GL-30-552917');
  fieldBox(page, 410, y, 162, 26, '16. Policy expiration date', numericDate(insuranceExpires));
  y += 30;
  y = sectionBar(page, y, 'SECTION F - ELECTRONIC PAYMENT (ACH)');
  fieldBox(page, 40, y, 240, 26, '17. Bank name', 'Marrow Springs Federal Credit Union');
  fieldBox(page, 280, y, 140, 26, '18. Account type', 'Business checking');
  fieldBox(page, 420, y, 152, 26, '19. Account number (last four)', '...6604');
  y += 30;
  y = sectionBar(page, y, 'SECTION G - CERTIFICATION AND SIGNATURE');
  page.text(44, y + 11, 'I certify that the information on this form is true and complete, that the business is not debarred from public contracting, and that I am authorized to sign for it.', { size: 7.5 });
  y += 16;
  fieldBox(page, 40, y, 200, 30, '20. Authorized signature', `/s/ ${signer}`, { face: 'serif', size: 11 });
  fieldBox(page, 240, y, 140, 30, '21. Printed name and title', `${signer}, President`, { size: 7.5 });
  fieldBox(page, 380, y, 192, 30, '22. Date signed', numericDate(signed));
  y += 40;
  page.rect(40, y, 532, 34, { fill: 0.94, stroke: 0.4, width: 0.5 });
  page.text(44, y + 10, 'FOR COUNTY USE ONLY', { face: 'sans-bold', size: 7 });
  page.text(44, y + 24, 'Vendor no. __________   Approved by __________   W-9 received [ ]   Insurance verified [ ]', { size: 7.5 });
  page.text(40, 768, 'Form VR-1 (Rev. 09/2024)', { size: 7, grey: 0.35 });
  const pages = [page];
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'County vendor registration form filled in by a coatings supplier',
    kind: 'form',
    textLayer: 'native',
    pages: pages.length,
    categories: ['form', 'layout_parties', 'competing_dates'],
    structure: structure({
      keyValues: [['1. Legal business name (as shown on your income tax return)', vendor], ['3. Federal employer identification number (EIN)', '84-0293157'], ['6. Date of incorporation', numericDate(incorporated)], ['14. General liability carrier', 'Highmeadow Casualty Company'], ['16. Policy expiration date', numericDate(insuranceExpires)], ['22. Date signed', numericDate(signed)]],
      routes: { 1: 'layout' },
    }),
    notes: `Every fact is a typed value in a labelled box. The defining date is the "Date signed" box (${numericDate(signed)}); the date of incorporation (${numericDate(incorporated)}) and the insurance expiration (${numericDate(insuranceExpires)}) are other boxes, and the form revision is a month and year in the footer. The legal name appears only in box 1. The county owns the form; filing it "with" the county is accepted, but the insurance carrier, bank, and contacts are not parties.`,
    gold: gold({
      type: 'Vendor Registration Form',
      date: signed,
      role: 'execution',
      forbiddenDates: [[incorporated, 'date of incorporation'], [insuranceExpires, 'insurance policy expiration']],
      parties: [vendor],
      relation: 'for',
      acceptablePartySets: [{ parties: [vendor], relation: 'from' }, { parties: [county], relation: 'with' }],
      roles: [[vendor, 'subject'], [vendor, 'issuer'], [county, 'counterparty']],
      forbiddenParties: [['Highmeadow Casualty Company', 'insurance carrier'], ['Marrow Springs Federal Credit Union', 'vendor\'s bank'], [signer, 'authorized signer'], ['Signe Ostrowski', 'sales contact']],
      facts: [[vendor, 'Emberglow'], ['Hartwell County'], ['coatings']],
      subjectTerms: ['vendor registration', 'coatings', 'Hartwell County'],
      readiness: 'ready',
      dateText: [numericDate(signed)],
    }),
  });
}

export function changeOrderForm() {
  const id = 'change-order-form';
  const owner = 'Briarport Library District';
  const contractor = 'Stonebridge Builders Inc.';
  const architect = 'Larkspur Design Studio LLP';
  const coDate = '2026-05-07';
  const contractDate = '2025-01-15';
  const completion = '2026-11-20';
  const changes = [
    ['RFI 41', 'Replace specified skylight glazing in the reading room with laminated low-E insulating units', 1842000],
    ['ASI 12', 'Add structural steel lintel and flashing at the relocated north entry', 621500],
    ['PCO 58', 'Owner-requested additional data drops (24) and two floor boxes in the teen area', 488750],
    ['PCO 61', 'Unforeseen condition: abandoned fuel oil tank removal and soil disposal at the east addition', 1307300],
    ['PCO 63', 'Delete bicycle rack allowance (owner to furnish)', -350000],
  ];
  const original = 481200000;
  const previous = 9643000;
  const net = changes.reduce((sum, [, , value]) => sum + value, 0);
  const page = new Page();
  page.text(40, 48, 'CHANGE ORDER', { face: 'sans-bold', size: 18 });
  page.text(40, 62, 'Owner - Contractor - Architect', { size: 8.5, grey: 0.3 });
  page.textRight(572, 48, `Distribution: ${owner} / ${contractor} / ${architect} / Field`, { size: 7, grey: 0.35 });
  let y = 74;
  const header = [['PROJECT', 'Briarport Public Library Renovation and Addition, 600 Pennant Street, Briarport, NC 27514', 532]];
  fieldBox(page, 40, y, header[0][2], 26, header[0][0], header[0][1], { face: 'sans', size: 9.5 });
  y += 26;
  fieldBox(page, 40, y, 266, 40, 'TO OWNER', owner, { face: 'sans', size: 9.5 });
  fieldBox(page, 306, y, 266, 40, 'TO CONTRACTOR', contractor, { face: 'sans', size: 9.5 });
  y += 40;
  const cells = [['CHANGE ORDER NUMBER', '006', 120], ['DATE', numericDate(coDate), 110], ['CONTRACT DATE', numericDate(contractDate), 110], ['ARCHITECT\'S PROJECT NO.', 'LDS-2318', 100], ['CONTRACT FOR', 'General Construction', 92]];
  let x = 40;
  for (const [label, value, width] of cells) {
    fieldBox(page, x, y, width, 26, label, value, { face: 'sans', size: 9.5 });
    x += width;
  }
  y += 36;
  page.text(40, y, 'THE CONTRACT IS CHANGED AS FOLLOWS:', { face: 'sans-bold', size: 8.5 });
  y += 8;
  page.rect(40, y, 532, 14, { fill: 0.85, stroke: 0.3, width: 0.5 });
  page.text(44, y + 10, 'Reference', { face: 'sans-bold', size: 8 });
  page.text(110, y + 10, 'Description', { face: 'sans-bold', size: 8 });
  page.textRight(568, y + 10, 'Amount', { face: 'sans-bold', size: 8 });
  y += 14;
  for (const [reference, description, value] of changes) {
    page.rect(40, y, 532, 18, { fill: null, stroke: 0.3, width: 0.4 });
    page.text(44, y + 12, reference, { size: 8.5 });
    page.text(110, y + 12, description, { size: 8.5 });
    page.textRight(568, y + 12, value < 0 ? `(${money(-value)})` : money(value), { size: 8.5 });
    y += 18;
  }
  y += 12;
  const sums = [
    ['The original Contract Sum was', money(original)],
    ['The net change by previously authorized Change Orders', money(previous)],
    ['The Contract Sum prior to this Change Order was', money(original + previous)],
    [`The Contract Sum will be increased by this Change Order in the amount of`, money(net)],
    ['The new Contract Sum including this Change Order will be', money(original + previous + net)],
    ['The Contract Time will be increased by', 'Fourteen (14) days'],
    ['The date of Substantial Completion as of the date of this Change Order therefore is', numericDate(completion)],
  ];
  for (const [label, value] of sums) {
    page.text(40, y, label, { size: 9 });
    page.textRight(572, y, value, { face: 'sans-bold', size: 9 });
    y += 14;
  }
  y += 6;
  y = page.textBlock(40, y, 532, 'NOTE: This Change Order does not include changes in the Contract Sum, Contract Time, or Guaranteed Maximum Price that have been authorized by Construction Change Directive until the cost and time have been agreed upon.', { size: 7 });
  y += 2;
  page.text(40, y, 'NOT VALID UNTIL SIGNED BY THE ARCHITECT, CONTRACTOR, AND OWNER.', { face: 'sans-bold', size: 7.5 });
  y += 14;
  const signers = [['ARCHITECT', architect, 'Bram Escobedo, AIA', '05/07/2026'], ['CONTRACTOR', contractor, 'Gwendolyn Haverford, Project Executive', '05/11/2026'], ['OWNER', owner, 'Orrin Yamashiro, Facilities Director', '05/14/2026']];
  signers.forEach(([role, firm, person, date], index) => {
    const left = 40 + index * 178;
    fieldBox(page, left, y, 176, 24, role, firm, { face: 'sans', size: 8 });
    fieldBox(page, left, y + 24, 176, 24, 'BY (signature)', `/s/ ${person.split(',')[0]}`, { face: 'serif', size: 10 });
    fieldBox(page, left, y + 48, 176, 24, 'PRINTED NAME AND TITLE', person, { face: 'sans', size: 7.5 });
    fieldBox(page, left, y + 72, 176, 24, 'DATE', date, { face: 'sans', size: 9 });
  });
  page.text(40, 768, 'Change Order form CO-1 (2024 edition). Copies: Owner, Contractor, Architect, Field.', { size: 7, grey: 0.35 });
  const pages = [page];
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Construction change order for a library renovation',
    kind: 'form',
    textLayer: 'native',
    pages: pages.length,
    categories: ['form', 'table', 'date_in_table', 'competing_dates', 'layout_parties'],
    structure: structure({
      tables: [[['Reference', 'Description', 'Amount'], ...changes.map(([reference, description, value]) => [reference, description, value < 0 ? `(${money(-value)})` : money(value)])]],
      // Not the change order's DATE: the architect signs under a DATE box
      // with the same date, so the pair could be credited to either.
      keyValues: [['TO OWNER', owner], ['TO CONTRACTOR', contractor], ['CHANGE ORDER NUMBER', '006'], ['CONTRACT DATE', numericDate(contractDate)], ['The original Contract Sum was', money(original)], ['The new Contract Sum including this Change Order will be', money(original + previous + net)]],
      routes: { 1: 'layout' },
    }),
    notes: `The change order's date (${numericDate(coDate)}) is a cell in a row of boxed fields beside the contract date (${numericDate(contractDate)}), which belongs to the underlying construction contract. The revised substantial completion date (${numericDate(completion)}) is a trap; the three signature dates (05/07, 05/11, 05/14/2026) are left unscored. A change order modifies the contract between owner and contractor; the architect prepares and signs it but is not a party.`,
    gold: gold({
      type: 'Change Order',
      date: coDate,
      role: 'amendment',
      forbiddenDates: [[contractDate, 'date of the underlying construction contract'], [completion, 'revised substantial completion date']],
      parties: [owner, contractor],
      relation: 'between',
      acceptablePartySets: [{ parties: [contractor], relation: 'with' }, { parties: [owner], relation: 'for' }],
      roles: [[owner, 'client'], [contractor, 'seller'], [contractor, 'counterparty'], [owner, 'subject']],
      forbiddenParties: [[architect, 'architect who prepares the change order'], ['Bram Escobedo', 'architect signatory']],
      facts: [[money(net), amount(net)], ['006', 'No. 6', 'Change Order 6'], ['library']],
      subjectTerms: ['change order', 'library renovation', 'skylight', 'contract sum'],
      readiness: 'ready',
      dateText: [numericDate(coDate)],
    }),
  });
}
