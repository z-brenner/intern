/// Tables, labelled values and forms: a ruled inspection log that runs over
/// a page break; an unruled price list whose columns line up only by
/// position; three invoices whose header facts sit under their labels,
/// across the page from them, and in a grid of boxes; a benefits change
/// form of check boxes; and a property loss notice of boxed fields.
import { Flow, Page } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { amount, longDate, money, numericDate } from '../lib/format.mjs';
import { digitalPdf, result } from './common.mjs';
import { checkBox, fieldBox } from './forms.mjs';
import { priceLines } from './invoices.mjs';

export function inspectionLogTwoPages() {
  const id = 'inspection-log-ruled-2p';
  const rng = Rng.from(id);
  const company = 'Emberwatch Fire Protection Inc.';
  const owner = 'Corbel Street Lofts Condominium Association';
  const inspected = '2026-05-19';
  const nextDue = '2027-05-19';
  const inspector = 'Leopold Haverkamp';
  const locations = [];
  for (const floor of ['Garage P1', 'Garage P2', 'Floor 1', 'Floor 2', 'Floor 3', 'Floor 4', 'Floor 5', 'Floor 6']) {
    for (const spot of ['north stair', 'south stair', 'elevator lobby', 'corridor east', 'corridor west', 'trash room']) {
      if (floor.startsWith('Garage') && spot === 'elevator lobby') continue;
      locations.push(`${floor}, ${spot}`);
    }
  }
  const rows = locations.slice(0, 44).map((location, index) => {
    const kitchen = location.includes('trash room');
    const type = kitchen ? 'K' : rng.pick(['ABC', 'ABC', 'ABC', 'CO2']);
    const size = type === 'K' ? '6 L' : type === 'CO2' ? '10 lb' : rng.pick(['5 lb', '10 lb']);
    // A CO2 cylinder is tested every five years, so none in service is older.
    const made = type === 'CO2' ? rng.int(2022, 2025) : rng.int(2014, 2024);
    const hydro = String(made + (type === 'CO2' ? 5 : 12));
    const status = index === 13 ? 'Recharged' : index === 31 ? 'Replaced' : 'Pass';
    return [`FE-${String(index + 1).padStart(3, '0')}`, location, type, size, String(made), hydro, status];
  });
  const flow = new Flow({ face: 'sans', fontSize: 9, margins: { top: 54, bottom: 60, left: 54, right: 54 }, keep: [company, owner],
    footer: (page, { number, total }) => page.textRight(558, 760, `Report FP-26-0519   Page ${number} of ${total}`, { size: 7.5, grey: 0.35 }) });
  flow.paragraph(company, { face: 'sans-bold', size: 15, after: 1 });
  flow.paragraph('1840 Kiln Road, Ashby Falls, OR 97321 - (541) 555-0177 - State license FE-20944', { size: 8, after: 10 });
  flow.paragraph('FIRE EXTINGUISHER INSPECTION REPORT', { face: 'sans-bold', size: 13, after: 8 });
  const fields = [['Property', owner], ['Address', '220 Corbel Street, Ashby Falls, OR 97321'], ['Inspection date', numericDate(inspected)], ['Inspector', `${inspector}, certificate 7731`], ['Next annual inspection due', numericDate(nextDue)]];
  flow.fields(fields.map(([label, value]) => [`${label}:`, value]), { size: 9, labelWidth: 150, face: 'sans', after: 8 });
  flow.paragraph('Every portable extinguisher was inspected under NFPA 10 annual maintenance: tag, seal and pin, gauge, hose and nozzle, mounting, and access. Types: ABC dry chemical, CO2 carbon dioxide, K wet chemical.', { size: 8.5, after: 8 });
  const columns = [{ header: 'Unit', width: 0.1 }, { header: 'Location', width: 0.34 }, { header: 'Type', width: 0.08 }, { header: 'Size', width: 0.1 }, { header: 'Made', width: 0.1 }, { header: 'Hydro due', width: 0.12 }, { header: 'Result', width: 0.16 }];
  flow.table(columns, rows, { size: 8.5, border: 'grid', headerFill: 0.85 });
  flow.paragraph('Summary: 44 extinguishers inspected; 42 passed, FE-014 was recharged after a low gauge reading, and FE-032 was replaced because its cylinder was dented. All units were tagged with the inspection date.', { size: 8.5, before: 4 });
  flow.paragraph(`Inspector: ${inspector}`, { size: 9, before: 6, after: 1 });
  const pages = flow.finish();
  if (pages.length !== 2) throw new Error(`${id}: the log must run onto a second page, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Fire extinguisher inspection log: a ruled table split across two pages',
    kind: 'inspection_report',
    textLayer: 'native',
    pages: 2,
    categories: ['table', 'date_in_table', 'competing_dates', 'key_value'],
    notes: `A 44-row ruled grid starts on page 1 and finishes on page 2 under its repeated header. The report is dated by its inspection date (${numericDate(inspected)}) in the header fields; the next inspection due (${numericDate(nextDue)}) is a trap, and every row carries a manufacture year and a hydrostatic test year. Prepared by the fire protection company for the condominium association.`,
    structure: structure({
      tables: [[columns.map((column) => column.header), ...rows]],
      keyValues: fields,
      routes: { 1: 'layout', 2: 'layout' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Fire Extinguisher Inspection Report',
      acceptableTypes: ['Inspection Report', 'Fire Extinguisher Inspection'],
      date: inspected,
      role: 'issuance',
      forbiddenDates: [[nextDue, 'next annual inspection due']],
      parties: [owner],
      relation: 'for',
      acceptablePartySets: [{ parties: [company], relation: 'from' }],
      roles: [[owner, 'subject'], [owner, 'customer'], [company, 'issuer'], [company, 'provider']],
      forbiddenParties: [[inspector, 'inspector who signs']],
      facts: [['44 extinguishers', '44'], ['FE-032'], ['220 Corbel Street', 'Corbel Street']],
      subjectTerms: ['fire extinguisher', 'inspection', 'NFPA 10'],
      readiness: 'ready',
      dateText: [numericDate(inspected)],
    }),
  });
}

export function priceListUnruled() {
  const id = 'price-list-unruled';
  const nursery = 'Thistledown Wholesale Nursery';
  const issued = '2026-03-16';
  const from = '2026-04-01';
  const until = '2026-06-30';
  const rng = Rng.from(id);
  const plants = [
    ['Acer rubrum', 'Red maple'], ['Amelanchier laevis', 'Allegheny serviceberry'], ['Betula nigra', 'River birch'], ['Cercis canadensis', 'Eastern redbud'],
    ['Cornus sericea', 'Red twig dogwood'], ['Hydrangea arborescens', 'Smooth hydrangea'], ['Ilex verticillata', 'Winterberry'], ['Itea virginica', 'Virginia sweetspire'],
    ['Juniperus virginiana', 'Eastern red cedar'], ['Physocarpus opulifolius', 'Ninebark'], ['Quercus bicolor', 'Swamp white oak'], ['Rhus aromatica', 'Fragrant sumac'],
    ['Sambucus canadensis', 'American elderberry'], ['Spiraea alba', 'Meadowsweet'], ['Thuja occidentalis', 'Arborvitae'], ['Viburnum dentatum', 'Arrowwood viburnum'],
    ['Echinacea purpurea', 'Purple coneflower'], ['Rudbeckia fulgida', 'Orange coneflower'], ['Schizachyrium scoparium', 'Little bluestem'], ['Asclepias tuberosa', 'Butterfly weed'],
  ];
  const rows = plants.map(([botanical, common], index) => {
    const perennial = index >= 16;
    const size = perennial ? '#1 cont.' : rng.pick(['#3 cont.', '#5 cont.', '#7 cont.', 'B&B 2 in.']);
    const price = perennial ? rng.amount(650, 1150, 25) : rng.amount(2400, 18900, 50);
    return [`TW-${1100 + index * 7}`, common, botanical, size, String(rng.int(12, 480)), amount(price)];
  });
  const flow = new Flow({ face: 'sans', fontSize: 9, margins: { top: 54, bottom: 60, left: 54, right: 54 }, keep: [nursery] });
  flow.paragraph(nursery.toUpperCase(), { face: 'sans-bold', size: 15, after: 1 });
  flow.paragraph('7 Gristmill Hollow Road, Pembury, PA 17356 - Trade sales (717) 555-0129 - Wholesale only', { size: 8, after: 10 });
  flow.paragraph('SPRING WHOLESALE AVAILABILITY AND PRICE LIST', { face: 'sans-bold', size: 12.5, after: 3 });
  flow.paragraph(`Issued ${longDate(issued)}. Prices apply to orders shipped from ${longDate(from)} through ${longDate(until)}. Quantities are on hand as of the issue date and are not reserved until an order is confirmed.`, { size: 8.5, after: 10 });
  const columns = [
    { header: 'Item', width: 0.11 }, { header: 'Common name', width: 0.24 }, { header: 'Botanical name', width: 0.29 },
    { header: 'Size', width: 0.14 }, { header: 'Avail.', width: 0.09, align: 'right' }, { header: 'Price', width: 0.13, align: 'right' },
  ];
  flow.table(columns, rows, { size: 8.5, border: 'none', headerFill: null, padding: 2.5 });
  flow.paragraph('Terms: net 30 days for approved accounts. Minimum order $750.00. Delivery within 60 miles is $95.00 per stop; larger orders by quote. B&B trees are guaranteed to leaf out in the first season when planted by a licensed contractor.', { size: 8.5, before: 6 });
  const pages = flow.finish();
  if (pages.length !== 1) throw new Error(`${id}: one page, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Wholesale nursery price list: an unruled table aligned only by position',
    kind: 'price_list',
    textLayer: 'native',
    pages: 1,
    categories: ['table', 'competing_dates', 'unusual'],
    notes: `Twenty items in six columns with no rules, boxes or shading: only alignment says which number is a quantity and which a price. Issued ${longDate(issued)}; the price period's start (${longDate(from)}) and end (${longDate(until)}) are traps. From the nursery, to no one in particular.`,
    structure: structure({
      tables: [[columns.map((column) => column.header), ...rows]],
      routes: { 1: 'layout' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Price List',
      acceptableTypes: ['Wholesale Price List', 'Availability and Price List'],
      date: issued,
      role: 'issuance',
      forbiddenDates: [[from, 'start of the price period'], [until, 'end of the price period']],
      parties: [nursery],
      relation: 'from',
      roles: [[nursery, 'issuer'], [nursery, 'seller']],
      forbiddenParties: [],
      facts: [['$750.00', '750.00'], ['Red maple', 'Acer rubrum']],
      subjectTerms: ['nursery', 'wholesale', 'availability'],
      readiness: 'ready',
      dateText: [longDate(issued)],
      partyText: { [nursery]: [nursery.toUpperCase()] },
    }),
  });
}

/// A caption over its value, as invoice templates print header facts.
function captioned(page, x, y, caption, value, { size = 10, face = 'sans' } = {}) {
  page.text(x, y, caption, { face: 'sans-bold', size: 6.5, grey: 0.4 });
  page.text(x, y + 13, value, { face, size });
}

export function invoiceLabelAbove() {
  const id = 'invoice-label-above';
  const issuer = 'Quarrystone Signs & Graphics';
  const customer = 'Bellhaven Physical Therapy PLLC';
  const invoiceDate = '2026-05-11';
  const dueDate = '2026-06-10';
  const installed = '2026-05-06';
  const number = 'QSG-10482';
  const priced = priceLines([
    { description: 'Illuminated channel letters, 18 in., "BELLHAVEN"', quantity: 1, unit: 684000 },
    { description: 'Raceway, painted to match fascia', quantity: 1, unit: 92500 },
    { description: 'Window vinyl, hours and logo, front doors', quantity: 2, unit: 21800 },
    { description: 'Installation crew and lift (hours)', quantity: 5, unit: 14500, taxable: false },
    { description: 'Sign permit, City of Larchmont (pass-through)', quantity: 1, unit: 18500, taxable: false },
  ], { taxRate: 600 });
  const page = new Page();
  page.text(54, 66, issuer, { face: 'sans-bold', size: 16 });
  page.text(54, 80, '61 Tanner Row, Larchmont, OR 97370 - (503) 555-0172 - accounts@quarrystone.example', { size: 8, grey: 0.3 });
  page.textRight(558, 66, 'INVOICE', { face: 'sans-bold', size: 22, grey: 0.2 });
  page.rect(54, 96, 504, 38, { fill: 0.95, stroke: null });
  const header = [['INVOICE NUMBER', number], ['INVOICE DATE', numericDate(invoiceDate)], ['CUSTOMER NO.', 'C-2291'], ['TERMS', 'Net 30'], ['DUE DATE', numericDate(dueDate)]];
  header.forEach(([caption, value], index) => captioned(page, 62 + index * 100, 110, caption, value));
  captioned(page, 54, 156, 'BILL TO', customer, { face: 'sans-bold' });
  ['Attn: Accounts Payable', '908 Mill Pond Road, Suite 4', 'Larchmont, OR 97370'].forEach((line, index) => page.text(54, 182 + index * 12, line, { size: 9.5 }));
  captioned(page, 330, 156, 'INSTALLED AT', '908 Mill Pond Road, front elevation');
  captioned(page, 330, 196, 'INSTALLATION DATE', numericDate(installed));
  const flow = new Flow({ face: 'sans', fontSize: 9, margins: { top: 54, bottom: 60, left: 54, right: 54 } });
  flow.page = page;
  flow.pages = [page];
  flow.y = 232;
  flow.table([{ header: 'Description', width: 0.58 }, { header: 'Qty', width: 0.08, align: 'right' }, { header: 'Unit price', width: 0.16, align: 'right' }, { header: 'Amount', width: 0.18, align: 'right' }],
    priced.rows.map((row) => [row.description, String(row.quantity), amount(row.unit), amount(row.total)]), { size: 9, border: 'rules', headerFill: 0.9 });
  let y = flow.y + 4;
  const totals = [['Subtotal', amount(priced.subtotal)], ['Sales tax 6% (signs and vinyl)', amount(priced.tax)], ['TOTAL DUE', money(priced.total)]];
  for (const [label, value] of totals) {
    const bold = label === 'TOTAL DUE';
    captioned(page, 380, y, label.toUpperCase(), value, { face: bold ? 'sans-bold' : 'sans', size: bold ? 11 : 9.5 });
    y += 28;
  }
  page.text(54, y + 10, `Please pay by ${longDate(dueDate)}. Make checks payable to ${issuer}, or pay by ACH to the account on your vendor file.`, { size: 8.5 });
  const { bytes, text } = digitalPdf([page]);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Sign shop invoice whose header facts sit under small captions',
    kind: 'invoice',
    textLayer: 'native',
    pages: 1,
    categories: ['invoice', 'key_value', 'competing_dates', 'layout_parties'],
    notes: `Five header facts in a shaded band, each under a small grey caption, so a line-by-line reading gives five captions and then five values and leaves the reader to pair them by position. The invoice date (${numericDate(invoiceDate)}) is the second; the due date (${numericDate(dueDate)}) and the installation date (${numericDate(installed)}) are traps. The issuer is named only in the letterhead; the customer, in bold under BILL TO, is the trap.`,
    structure: structure({
      keyValues: [...header, ['BILL TO', customer], ['INSTALLATION DATE', numericDate(installed)], ['TOTAL DUE', money(priced.total)]],
      routes: { 1: 'layout' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Invoice',
      date: invoiceDate,
      role: 'invoice',
      forbiddenDates: [[dueDate, 'payment due date'], [installed, 'installation date']],
      parties: [issuer],
      relation: 'from',
      roles: [[issuer, 'issuer'], [customer, 'customer']],
      forbiddenParties: [[customer, 'bill-to customer']],
      facts: [[money(priced.total), amount(priced.total)], [number], ['channel letters']],
      subjectTerms: ['sign', 'installation', 'vinyl'],
      readiness: 'ready',
      dateText: [numericDate(invoiceDate)],
    }),
  });
}

export function invoiceRightAligned() {
  const id = 'invoice-right-aligned';
  const issuer = 'Mossgiel Laboratory Services Inc.';
  const customer = 'Ferncastle Veterinary Hospital';
  const invoiceDate = '2026-05-04';
  const periodStart = '2026-04-01';
  const periodEnd = '2026-04-30';
  const dueDate = '2026-06-03';
  const number = 'MLS-2604-0317';
  const tests = [['Complete blood count, canine/feline', 38, 2450], ['Chemistry panel, 24 analytes', 31, 4275], ['Urinalysis with sediment', 12, 1890], ['Fecal flotation and Giardia antigen', 17, 2160], ['Thyroid panel (T4, free T4)', 6, 5240], ['Courier pickups', 20, 0]];
  const priced = priceLines(tests.map(([description, quantity, unit]) => ({ description, quantity, unit })));
  const flow = new Flow({ face: 'sans', fontSize: 9.5, margins: { top: 54, bottom: 60, left: 60, right: 60 }, keep: [issuer, customer] });
  flow.paragraph(issuer, { face: 'sans-bold', size: 15, after: 1 });
  flow.paragraph('Reference Laboratory - 300 Assay Court, Brackwater, WI 53704 - client services (608) 555-0131', { size: 8, after: 12 });
  flow.paragraph('INVOICE', { face: 'sans-bold', size: 18, after: 8 });
  const header = [['Invoice number', number], ['Invoice date', numericDate(invoiceDate)], ['Client account', 'FVH-0088'], ['Service period', `${numericDate(periodStart)} - ${numericDate(periodEnd)}`], ['Payment due', numericDate(dueDate)]];
  // Label in the first cell, value pushed to the right edge of the second:
  // each fact sits a hand's width from its label.
  flow.table([{ header: '', width: 0.3 }, { header: '', width: 0.7, align: 'right' }], header, { header: false, size: 9.5, border: 'rules' });
  flow.paragraph(`Client: ${customer}, 2150 Ferncastle Road, Brackwater, WI 53711`, { size: 9.5, before: 6, after: 10 });
  flow.table([{ header: 'Test', width: 0.52 }, { header: 'Count', width: 0.12, align: 'right' }, { header: 'Unit price', width: 0.16, align: 'right' }, { header: 'Amount', width: 0.2, align: 'right' }],
    priced.rows.map((row) => [row.description, String(row.quantity), amount(row.unit), amount(row.total)]), { size: 9, border: 'rules', headerFill: 0.9 });
  const totals = [['Subtotal', amount(priced.subtotal)], ['Client discount (5%)', `-${amount(Math.round(priced.subtotal * 0.05))}`], ['Amount due', money(priced.subtotal - Math.round(priced.subtotal * 0.05))]];
  flow.table([{ header: '', width: 0.3 }, { header: '', width: 0.7, align: 'right' }], totals, { header: false, size: 9.5, border: 'rules' });
  flow.paragraph('Results for every accession listed on the attached statement were reported through the client portal. Questions about this invoice: billing@mossgiel-lab.example.', { size: 8.5, before: 6 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  const due = totals[2][1];
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Laboratory invoice with each label at the left margin and its value at the right',
    kind: 'invoice',
    textLayer: 'native',
    pages: pages.length,
    categories: ['invoice', 'key_value', 'date_in_table', 'table', 'competing_dates'],
    notes: `The header facts are two-cell table rows with the label at the left margin and the value right-aligned at the right one, so each value - the invoice date (${numericDate(invoiceDate)}) among them - sits about 350 points from its label. The service period's ends (${numericDate(periodStart)}, ${numericDate(periodEnd)}) and the payment due date (${numericDate(dueDate)}) are traps. From the laboratory to the veterinary hospital.`,
    structure: structure({
      tables: [[['Test', 'Count', 'Unit price', 'Amount'], ...priced.rows.map((row) => [row.description, String(row.quantity), amount(row.unit), amount(row.total)])]],
      keyValues: [...header, ...totals],
      routes: { 1: 'layout' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Invoice',
      date: invoiceDate,
      role: 'invoice',
      forbiddenDates: [[dueDate, 'payment due date'], [periodStart, 'start of the service period'], [periodEnd, 'end of the service period']],
      parties: [issuer],
      relation: 'from',
      acceptablePartySets: [{ parties: [issuer, customer], relation: 'between' }],
      roles: [[issuer, 'issuer'], [customer, 'customer']],
      forbiddenParties: [],
      facts: [[due, due.slice(1)], [number], [customer, 'Ferncastle']],
      subjectTerms: ['laboratory', 'chemistry panel', 'blood count'],
      readiness: 'ready',
      dateText: [numericDate(invoiceDate)],
    }),
  });
}

export function invoiceBoxedGrid() {
  const id = 'invoice-boxed-grid';
  const issuer = 'Whitlock & Sons Plumbing LLC';
  const customer = 'Ravensby Dental Studio';
  const invoiceDate = '2026-07-09';
  const serviceDate = '2026-07-02';
  const dueDate = '2026-07-24';
  const number = '88-2041';
  const page = new Page();
  page.text(48, 62, issuer, { face: 'sans-bold', size: 16 });
  page.text(48, 76, 'Licensed master plumber PL-55120 - 12 Cooper Lane, Ravensby, VT 05401 - (802) 555-0158', { size: 8, grey: 0.3 });
  page.textRight(564, 62, 'SERVICE INVOICE', { face: 'sans-bold', size: 14 });
  // A grid of boxes, two labelled values to a row.
  const grid = [
    [['Invoice No.', number], ['Invoice Date', numericDate(invoiceDate)]],
    [['Job No.', 'J-5512'], ['Service Date', numericDate(serviceDate)]],
    [['Customer PO', 'RDS-0413'], ['Due Date', numericDate(dueDate)]],
    [['Technician', 'Gideon Whitlock'], ['Terms', 'Net 15']],
  ];
  let y = 92;
  for (const row of grid) {
    row.forEach(([label, value], index) => {
      const x = 48 + index * 258;
      page.rect(x, y, 258, 20, { fill: null, stroke: 0.2, width: 0.6 });
      page.rect(x, y, 92, 20, { fill: 0.9, stroke: 0.2, width: 0.6 });
      page.text(x + 5, y + 13.5, label, { face: 'sans-bold', size: 8.5 });
      page.text(x + 98, y + 13.5, value, { size: 9.5 });
    });
    y += 20;
  }
  y += 14;
  const blocks = [
    ['Bill To', [customer, 'Dr. Amara Ellingsworth', '41 Lantern Street', 'Ravensby, VT 05401']],
    ['Service Address', ['Ravensby Dental Studio - Suite 200', '41 Lantern Street', 'Ravensby, VT 05401', 'Access via rear stair']],
  ];
  blocks.forEach(([label, lines], index) => {
    const x = 48 + index * 258;
    page.rect(x, y, 258, 72, { fill: null, stroke: 0.2, width: 0.6 });
    page.rect(x, y, 258, 15, { fill: 0.9, stroke: 0.2, width: 0.6 });
    page.text(x + 5, y + 11, label, { face: 'sans-bold', size: 8.5 });
    lines.forEach((line, row) => page.text(x + 5, y + 28 + row * 12, line, { size: 9.5, face: row === 0 ? 'sans-bold' : 'sans' }));
  });
  const priced = priceLines([
    { description: 'Replace dental vacuum pump check valve and union', quantity: 1, unit: 28600 },
    { description: 'Clear and jet 2 in. waste line, operatory 3', quantity: 1, unit: 34500 },
    { description: 'Install amalgam separator cartridge (customer supplied)', quantity: 1, unit: 9500 },
    { description: 'Labor, journeyman plumber (hours)', quantity: 4.5, unit: 11800 },
  ], { taxRate: 0 });
  const flow = new Flow({ face: 'sans', fontSize: 9, margins: { top: 48, bottom: 60, left: 48, right: 48 } });
  flow.page = page;
  flow.pages = [page];
  flow.y = y + 90;
  flow.table([{ header: 'Work performed', width: 0.6 }, { header: 'Qty', width: 0.1, align: 'right' }, { header: 'Rate', width: 0.13, align: 'right' }, { header: 'Amount', width: 0.17, align: 'right' }],
    priced.rows.map((row) => [row.description, String(row.quantity), amount(row.unit), amount(row.total)]), { size: 9, border: 'grid', headerFill: 0.9 });
  page.rect(306, flow.y + 4, 258, 20, { fill: 0.9, stroke: 0.2, width: 0.6 });
  page.text(311, flow.y + 17.5, 'Balance Due', { face: 'sans-bold', size: 9.5 });
  page.textRight(559, flow.y + 17.5, money(priced.total), { face: 'sans-bold', size: 10 });
  page.text(48, flow.y + 50, 'Thank you. A 1.5% monthly charge applies to balances unpaid after the due date. Warranty on parts: one year; labor: 90 days.', { size: 8.5 });
  const { bytes, text } = digitalPdf([page]);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Plumbing service invoice laid out as a grid of labelled boxes',
    kind: 'invoice',
    textLayer: 'native',
    pages: 1,
    categories: ['invoice', 'key_value', 'competing_dates', 'layout_parties'],
    notes: `Header facts in a grid of bordered boxes, two labelled values to a row, and the bill-to and service addresses in two boxes side by side, so their lines are read across each other. The invoice date (${numericDate(invoiceDate)}) shares a row with the invoice number; the service date (${numericDate(serviceDate)}) and due date (${numericDate(dueDate)}) are traps. From the plumber; the dental studio is the customer, named in bold in both address boxes.`,
    structure: structure({
      keyValues: [...grid.flat(), ['Bill To', customer], ['Service Address', 'Ravensby Dental Studio - Suite 200'], ['Balance Due', money(priced.total)]],
      tables: [[['Work performed', 'Qty', 'Rate', 'Amount'], ...priced.rows.map((row) => [row.description, String(row.quantity), amount(row.unit), amount(row.total)])]],
      routes: { 1: 'layout' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Invoice',
      acceptableTypes: ['Service Invoice'],
      date: invoiceDate,
      role: 'invoice',
      forbiddenDates: [[serviceDate, 'service date'], [dueDate, 'payment due date']],
      parties: [issuer],
      relation: 'from',
      roles: [[issuer, 'issuer'], [customer, 'customer']],
      forbiddenParties: [[customer, 'bill-to customer'], ['Dr. Amara Ellingsworth', 'customer contact'], ['Gideon Whitlock', 'technician']],
      facts: [[money(priced.total), amount(priced.total)], [number], ['vacuum pump']],
      subjectTerms: ['plumbing', 'waste line', 'amalgam separator'],
      readiness: 'ready',
      dateText: [numericDate(invoiceDate)],
    }),
  });
}

export function benefitsChangeForm() {
  const id = 'benefits-change-checkbox-form';
  const employer = 'Westbrook Kettle Brewing Co.';
  const employee = 'Annika Solberg-Reyes';
  const spouse = 'Mateo Solberg-Reyes';
  const event = '2026-06-20';
  const effective = '2026-07-01';
  const signed = '2026-06-29';
  const spouseBirth = '1991-02-14';
  const page = new Page();
  page.text(40, 52, employer.toUpperCase(), { face: 'sans-bold', size: 13 });
  page.text(40, 65, 'Human Resources - 75 Malthouse Lane, Westbrook, ME 04092', { size: 8, grey: 0.3 });
  page.textRight(572, 52, 'BENEFITS ENROLLMENT CHANGE FORM', { face: 'sans-bold', size: 12 });
  page.textRight(572, 65, 'Submit within 30 days of a qualifying event', { size: 8, grey: 0.3 });
  const bar = (y, title) => {
    page.rect(40, y, 532, 14, { fill: 0.82, stroke: null });
    page.text(44, y + 10, title, { face: 'sans-bold', size: 8.5 });
    return y + 14;
  };
  let y = bar(80, 'SECTION 1 - EMPLOYEE');
  const fields = [['Employee name', employee, 300], ['Employee ID', 'WK-0417', 232]];
  let x = 40;
  for (const [label, value, width] of fields) {
    fieldBox(page, x, y, width, 26, label, value, { face: 'sans', size: 10 });
    x += width;
  }
  y += 26;
  fieldBox(page, 40, y, 300, 26, 'Department', 'Brewhouse operations', { face: 'sans', size: 10 });
  fieldBox(page, 340, y, 232, 26, 'Date of hire', numericDate('2021-03-08'), { face: 'sans', size: 10 });
  y += 32;
  const groups = [
    ['SECTION 2 - REASON FOR CHANGE', [['Marriage', true], ['Birth or adoption', false], ['Divorce', false], ['Loss of other coverage', false], ['Spouse gained coverage', false], ['Other', false]]],
    ['SECTION 3 - MEDICAL PLAN', [['Gold PPO', false], ['Silver PPO', true], ['High-deductible HSA', false], ['Waive coverage', false]]],
    ['SECTION 4 - COVERAGE LEVEL', [['Employee only', false], ['Employee and spouse', true], ['Employee and children', false], ['Family', false]]],
    ['SECTION 5 - DENTAL AND VISION', [['Dental', true], ['Vision', true], ['No change', false]]],
  ];
  for (const [title, options] of groups) {
    y = bar(y, title);
    options.forEach(([caption, checked], index) => checkBox(page, 50 + (index % 3) * 176, y + 16 + Math.floor(index / 3) * 15, caption, checked, { size: 9 }));
    y += 16 + Math.ceil(options.length / 3) * 15;
  }
  y = bar(y + 2, 'SECTION 6 - EVENT AND DEPENDENTS');
  fieldBox(page, 40, y, 180, 26, 'Date of qualifying event', numericDate(event), { face: 'sans', size: 10 });
  fieldBox(page, 220, y, 180, 26, 'Requested effective date', numericDate(effective), { face: 'sans', size: 10 });
  fieldBox(page, 400, y, 172, 26, 'Number of dependents added', '1', { face: 'sans', size: 10 });
  y += 26;
  fieldBox(page, 40, y, 300, 26, 'Spouse name', spouse, { face: 'sans', size: 10 });
  fieldBox(page, 340, y, 232, 26, 'Spouse date of birth', numericDate(spouseBirth), { face: 'sans', size: 10 });
  y += 32;
  y = bar(y, 'SECTION 7 - SIGNATURE');
  page.textBlock(44, y + 12, 524, 'I certify that the information above is true and that a qualifying event occurred. I authorize payroll deductions for the coverage elected. I will provide a copy of the marriage certificate within 30 days.', { size: 8 });
  y += 30;
  fieldBox(page, 40, y, 300, 30, 'Employee signature', `/s/ ${employee}`, { face: 'serif', size: 11 });
  fieldBox(page, 340, y, 232, 30, 'Date signed', numericDate(signed), { face: 'sans', size: 10 });
  page.text(40, 768, 'HR-BEN-7 (Rev. 01/2026)', { size: 7, grey: 0.35 });
  const { bytes, text } = digitalPdf([page]);
  const boxes = [['Employee name', employee], ['Employee ID', 'WK-0417'], ['Department', 'Brewhouse operations'], ['Date of qualifying event', numericDate(event)], ['Requested effective date', numericDate(effective)], ['Spouse name', spouse], ['Date signed', numericDate(signed)]];
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Employee benefits change form: check boxes and boxed fields',
    kind: 'form',
    textLayer: 'native',
    pages: 1,
    categories: ['form', 'hr', 'key_value', 'competing_dates'],
    notes: `Four groups of check boxes, three to a row, with an X drawn in the boxes chosen (marriage; Silver PPO; employee and spouse; dental and vision), and boxed fields with captions over their values. Dated by the signature box (${numericDate(signed)}); the event date (${numericDate(event)}), the requested effective date (${numericDate(effective)}), the hire date and the spouse's date of birth are traps. The employee completes it for the employer.`,
    structure: structure({
      // Each group of boxes is a table of two columns, the mark and the
      // option: a chosen option keeps its X on its line.
      tables: groups.map(([, options]) => options.map(([caption, checked]) => [checked ? 'X' : '', caption])),
      keyValues: boxes,
      routes: { 1: 'layout' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Benefits Enrollment Change Form',
      acceptableTypes: ['Benefits Change Form', 'Benefits Enrollment Form'],
      date: signed,
      role: 'execution',
      forbiddenDates: [[event, 'date of the qualifying event (marriage)'], [effective, 'requested effective date'], [spouseBirth, 'spouse date of birth'], ['2021-03-08', 'date of hire']],
      parties: [employee],
      relation: 'for',
      acceptablePartySets: [{ parties: [employee], relation: 'from' }, { parties: [employer], relation: 'with' }],
      roles: [[employee, 'subject'], [employee, 'employee'], [employee, 'issuer'], [employer, 'employer'], [employer, 'counterparty']],
      forbiddenParties: [[spouse, 'spouse being added as a dependent']],
      facts: [['Silver PPO'], ['Employee and spouse', 'spouse'], [employee, 'Solberg-Reyes']],
      subjectTerms: ['benefits', 'marriage', 'medical plan'],
      readiness: 'ready',
      dateText: [numericDate(signed)],
    }),
  });
}

export function lossNoticeBoxedFields() {
  const id = 'loss-notice-boxed-fields';
  const insurer = 'Harrowmere Mutual Insurance Company';
  const insured = 'Delacroix Bakehouse LLC';
  const agency = 'Pemberton & Vail Insurance Agency';
  const loss = '2026-02-14';
  const reported = '2026-02-16';
  const periodStart = '2025-09-01';
  const periodEnd = '2026-09-01';
  const page = new Page();
  page.text(40, 52, 'PROPERTY LOSS NOTICE', { face: 'sans-bold', size: 15 });
  page.text(40, 66, `${insurer} - Claims Intake - 9 Granary Street, Hollis Bend, NH 03049`, { size: 8, grey: 0.3 });
  page.textRight(572, 52, `Date reported: ${numericDate(reported)}`, { face: 'sans', size: 9.5 });
  let y = 80;
  const rows = [
    [['Policy number', 'CPP-88-410273', 150], ['Policy period', `${numericDate(periodStart)} to ${numericDate(periodEnd)}`, 186], ['Agency', agency, 196]],
    [['Named insured', insured, 300], ['Insured phone', '(603) 555-0119', 232]],
    [['Mailing address', '17 Ovenstone Street, Hollis Bend, NH 03049', 532]],
    [['Date of loss', numericDate(loss), 120], ['Time of loss', '3:40 a.m.', 100], ['Kind of loss', 'Water damage', 140], ['Police or fire report', 'None', 172]],
    [['Location of loss', 'Same as mailing address - kitchen and walk-in cooler', 532]],
    [['Description of loss', 'Supply line to the proofing cabinet burst; water spread across the kitchen floor', 532]],
    [['Estimated amount of loss', '$38,500.00', 176], ['Property damaged', 'Flooring, two mixers, stock', 196], ['Damage still occurring', 'No', 160]],
    [['Reported by', 'Solenne Delacroix, Owner', 300], ['Contact phone', '(603) 555-0144', 232]],
  ];
  for (const row of rows) {
    let x = 40;
    for (const [label, value, width] of row) {
      fieldBox(page, x, y, width, 30, label, value, { face: 'sans', size: 9.5, labelSize: 7 });
      x += width;
    }
    y += 30;
  }
  y += 12;
  page.textBlock(40, y, 532, 'The insured must protect the property from further damage, keep records of repair costs, and allow the company to inspect the damaged property before it is discarded. This notice does not confirm coverage; the company will respond in writing within 15 days.', { size: 8.5 });
  page.text(40, 768, 'PLN-2 (Rev. 03/2025)', { size: 7, grey: 0.35 });
  const { bytes, text } = digitalPdf([page]);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Property loss notice of boxed fields, up to four to a row',
    kind: 'form',
    textLayer: 'native',
    pages: 1,
    categories: ['form', 'key_value', 'competing_dates', 'layout_parties'],
    notes: `Every fact is a value in a bordered box under a small caption, two to four boxes to a row, so a line-by-line reading strings the captions of a row together and the values after them. The notice is dated by its report date (${numericDate(reported)}, top right); the date of loss (${numericDate(loss)}) and the policy period (${numericDate(periodStart)} to ${numericDate(periodEnd)}) are traps. The insured gives notice to the insurer; the agency is not a party.`,
    structure: structure({
      keyValues: rows.flat().map(([label, value]) => [label, value]),
      routes: { 1: 'layout' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Property Loss Notice',
      acceptableTypes: ['Notice of Loss', 'Loss Notice'],
      date: reported,
      role: 'notice',
      forbiddenDates: [[loss, 'date of loss'], [periodStart, 'policy period start'], [periodEnd, 'policy period end']],
      parties: [insured],
      relation: 'for',
      acceptablePartySets: [{ parties: [insured], relation: 'from' }, { parties: [insurer], relation: 'to' }],
      roles: [[insured, 'subject'], [insured, 'issuer'], [insurer, 'recipient'], [insurer, 'counterparty']],
      forbiddenParties: [[agency, 'insurance agency'], ['Solenne Delacroix', 'owner who reported the loss']],
      facts: [['$38,500.00', '38,500'], ['CPP-88-410273'], ['water damage', 'burst']],
      subjectTerms: ['loss', 'water', 'claim'],
      readiness: 'ready',
      dateText: [numericDate(reported)],
    }),
  });
}
