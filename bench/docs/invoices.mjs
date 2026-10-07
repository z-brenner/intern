/// Invoices and a purchase order: the commercial paper whose dates live in
/// header grids and whose parties are told apart by where they sit on the
/// page rather than by any sentence naming them.
import { Flow } from '../lib/layout.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { amount, ledgerDate, longDate, money, numericDate } from '../lib/format.mjs';
import { digitalPdf, result } from './common.mjs';

/// Line items -> rows and totals, all in integer cents.
export function priceLines(items, { taxRate = 0, freight = 0 } = {}) {
  const rows = items.map((item) => ({ ...item, total: item.quantity * item.unit }));
  const subtotal = rows.reduce((sum, row) => sum + row.total, 0);
  const taxable = rows.filter((row) => row.taxable !== false).reduce((sum, row) => sum + row.total, 0);
  const tax = Math.round((taxable * taxRate) / 10000);
  return { rows, subtotal, tax, freight, total: subtotal + tax + freight };
}

export function invoiceDateInTable() {
  const id = 'invoice-date-in-table';
  const issuer = 'Halvorsen Fixture Works LLC';
  const customer = 'Quillon Ridge Bakery, Inc.';
  const invoiceDate = '2026-03-04';
  const dueDate = '2026-04-03';
  const poDate = '2026-02-11';
  const shipDate = '2026-02-27';
  const invoiceNumber = 'INV-20417';
  const poNumber = 'PO-88213';
  const priced = priceLines([
    { sku: 'HFW-DC48', description: 'Curved-glass refrigerated pastry display case, 48 in., walnut base', quantity: 1, unit: 238500 },
    { sku: 'HFW-SH12', description: 'Tiered bread shelving, powder-coated steel, 72 x 18 in.', quantity: 3, unit: 26450 },
    { sku: 'HFW-CT06', description: 'Point-of-sale counter top, quartz, 6 ft., with cut-out', quantity: 1, unit: 54825 },
    { sku: 'HFW-LBR', description: 'On-site installation and leveling, 2 technicians (hours)', quantity: 6, unit: 9500, taxable: false },
  ], { taxRate: 870, freight: 18500 });
  const flow = new Flow({ face: 'sans', fontSize: 9.5, margins: { top: 48, bottom: 60, left: 54, right: 54 }, keep: [issuer, customer] });
  const page = flow.page;
  // Letterhead: the only place the issuer is named.
  page.rect(54, 44, 46, 46, { fill: 0.15, stroke: null });
  page.text(63, 74, 'HFW', { face: 'sans-bold', size: 13, grey: 1 });
  page.text(110, 62, issuer, { face: 'sans-bold', size: 16 });
  page.text(110, 76, 'Commercial display fixtures since 1987', { size: 8.5, grey: 0.3 });
  page.text(110, 88, '4410 Ostrander Road, Halden Bay, WA 98264   (360) 555-0119   billing@halvorsenfixture.example', { size: 8, grey: 0.3 });
  page.textRight(558, 66, 'INVOICE', { face: 'sans-bold', size: 22 });
  flow.y = 108;
  flow.table([
    { header: 'Invoice No.', width: 0.15 },
    { header: 'Invoice Date', width: 0.14 },
    { header: 'Customer PO', width: 0.15 },
    { header: 'PO Date', width: 0.14 },
    { header: 'Ship Date', width: 0.14 },
    { header: 'Terms', width: 0.13 },
    { header: 'Due Date', width: 0.15 },
  ], [[invoiceNumber, numericDate(invoiceDate), poNumber, numericDate(poDate), numericDate(shipDate), 'Net 30', numericDate(dueDate)]], { size: 9, headerFill: 0.85 });
  const top = flow.y;
  page.text(54, top + 10, 'BILL TO', { face: 'sans-bold', size: 8, grey: 0.35 });
  ['Quillon Ridge Bakery, Inc.', 'Accounts Payable', '1188 Pennant Street', 'Halden Bay, WA 98261'].forEach((line, index) => page.text(54, top + 23 + index * 11.5, line, { size: 9.5, face: index === 0 ? 'sans-bold' : 'sans' }));
  page.text(306, top + 10, 'SHIP TO', { face: 'sans-bold', size: 8, grey: 0.35 });
  ['Quillon Ridge Bakery - Harbor Street Cafe', 'Attn: Marta Quillon', '62 Harbor Street', 'Halden Bay, WA 98262'].forEach((line, index) => page.text(306, top + 23 + index * 11.5, line, { size: 9.5 }));
  flow.y = top + 82;
  flow.table([
    { header: 'Item', width: 0.13 },
    { header: 'Description', width: 0.47 },
    { header: 'Qty', width: 0.08, align: 'right' },
    { header: 'Unit Price', width: 0.15, align: 'right' },
    { header: 'Amount', width: 0.17, align: 'right' },
  ], priced.rows.map((row) => [row.sku, row.description, String(row.quantity), amount(row.unit), amount(row.total)]), { size: 9, border: 'rules', headerFill: 0.92 });
  const totals = [
    ['Subtotal', amount(priced.subtotal)],
    ['Freight (LTL, liftgate)', amount(priced.freight)],
    ['Sales tax 8.7% (fixtures only)', amount(priced.tax)],
  ];
  for (const [label, value] of totals) {
    flow.page.textRight(470, flow.y + 9, label, { size: 9.5 });
    flow.page.textRight(558, flow.y + 9, value, { size: 9.5 });
    flow.y += 14;
  }
  flow.page.line(380, flow.y + 2, 558, flow.y + 2, { width: 0.8 });
  flow.page.textRight(470, flow.y + 15, 'TOTAL DUE (USD)', { face: 'sans-bold', size: 10.5 });
  flow.page.textRight(558, flow.y + 15, money(priced.total), { face: 'sans-bold', size: 10.5 });
  flow.y += 36;
  flow.paragraph('Payment terms: Net 30 from the invoice date. Please reference the invoice number on your remittance. ACH: account ending 0193, routing on file. Checks payable to the company named above. A finance charge of 1.5% per month applies to balances unpaid after the due date.', { size: 8.5, face: 'sans' });
  flow.paragraph('Warranty: display cases carry a two-year parts warranty from the ship date; refrigeration compressors carry five years. Report shipping damage within five business days of delivery.', { size: 8.5, face: 'sans' });
  flow.paragraph('Thank you for your business.', { size: 9, face: 'sans-bold', align: 'center' });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  const total = money(priced.total);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Fixture maker\'s invoice with every date in a header grid',
    kind: 'invoice',
    textLayer: 'native',
    pages: pages.length,
    categories: ['invoice', 'table', 'date_in_table', 'competing_dates', 'layout_parties'],
    structure: structure({
      tables: [
        [['Invoice No.', 'Invoice Date', 'Customer PO', 'PO Date', 'Ship Date', 'Terms', 'Due Date'], [invoiceNumber, numericDate(invoiceDate), poNumber, numericDate(poDate), numericDate(shipDate), 'Net 30', numericDate(dueDate)]],
        [['Item', 'Description', 'Qty', 'Unit Price', 'Amount'], ...priced.rows.map((row) => [row.sku, row.description, String(row.quantity), amount(row.unit), amount(row.total)])],
      ],
      keyValues: [['Invoice Date', numericDate(invoiceDate)], ['PO Date', numericDate(poDate)], ['Ship Date', numericDate(shipDate)], ['Due Date', numericDate(dueDate)], ['BILL TO', 'Quillon Ridge Bakery, Inc.'], ['SHIP TO', 'Quillon Ridge Bakery - Harbor Street Cafe'], ['TOTAL DUE (USD)', money(priced.total)]],
      routes: { 1: 'layout' },
    }),
    notes: `Four dates share one header row - invoice ${numericDate(invoiceDate)}, PO ${numericDate(poDate)}, ship ${numericDate(shipDate)}, due ${numericDate(dueDate)} - and the row's labels sit on the line above the values, so the date's meaning comes only from column position. Dates are numeric; the ship and due dates have days above 12, which settles month-first order. The issuer is named only in the letterhead; the bill-to and ship-to blocks name the customer twice, in bold, and the customer is the trap. Total ${total}.`,
    gold: gold({
      type: 'Invoice',
      date: invoiceDate,
      role: 'invoice',
      forbiddenDates: [[dueDate, 'payment due date'], [poDate, 'date of the customer\'s purchase order'], [shipDate, 'ship date']],
      parties: [issuer],
      relation: 'from',
      roles: [[issuer, 'issuer'], [customer, 'customer']],
      forbiddenParties: [[customer, 'bill-to customer'], ['Marta Quillon', 'ship-to contact']],
      facts: [[total, total.slice(1)], ['Quillon Ridge Bakery'], [invoiceNumber, '20417']],
      forbiddenFacts: [],
      subjectTerms: ['display case', 'display', 'shelving', 'fixtures', 'installation'],
      readiness: 'ready',
      dateText: [numericDate(invoiceDate)],
    }),
  });
}

export function invoiceLayoutOnly() {
  const id = 'invoice-layout-only';
  const issuer = 'Ironvale Marine Coatings Co.';
  const customer = 'Thornbury Harbor Marina LLC';
  const carrier = 'Saltash Freight Lines';
  const invoiceDate = '2026-02-17';
  const orderDate = '2026-02-03';
  const dueDate = '2026-03-19';
  const invoiceNumber = '7731-B';
  const priced = priceLines([
    { sku: 'IMC-2210', description: 'Two-part epoxy barrier coat, grey, 4 gal kit', quantity: 12, unit: 31840 },
    { sku: 'IMC-4415', description: 'Copper-free ablative antifouling, black, 1 gal', quantity: 30, unit: 17925 },
    { sku: 'IMC-0090', description: 'Solvent reducer #90, 1 gal', quantity: 8, unit: 4450 },
    { sku: 'IMC-SVC', description: 'Technical rep site visit, Dock C haul-out (half day)', quantity: 1, unit: 65000 },
  ], { taxRate: 0, freight: 41200 });
  const flow = new Flow({ face: 'sans', fontSize: 9.5, margins: { top: 50, bottom: 60, left: 50, right: 50 } });
  const page = flow.page;
  // Issuer block: small, top left, unlabelled - as most invoice templates print it.
  [issuer, '18 Ferry Landing Road', 'Gilchrist Harbor, ME 04112', 'Tel. (207) 555-0164'].forEach((line, index) => page.text(50, 62 + index * 11, line, { size: index === 0 ? 10 : 8.5, face: index === 0 ? 'sans-bold' : 'sans' }));
  page.textRight(562, 70, 'INVOICE', { face: 'sans-bold', size: 24, grey: 0.25 });
  const meta = [['Invoice #', invoiceNumber], ['Invoice Date', ledgerDate(invoiceDate)], ['Order Date', ledgerDate(orderDate)], ['Customer #', 'C-00418'], ['Due', ledgerDate(dueDate)]];
  meta.forEach(([label, value], index) => {
    page.text(420, 92 + index * 12, label, { size: 8.5, grey: 0.35 });
    page.textRight(562, 92 + index * 12, value, { size: 9 });
  });
  // Sold-to box: the most prominent name on the page.
  page.rect(50, 120, 300, 70, { fill: 0.95, stroke: 0.4 });
  page.text(58, 133, 'SOLD TO', { face: 'sans-bold', size: 7.5, grey: 0.35 });
  page.text(58, 150, customer, { face: 'sans-bold', size: 13 });
  page.text(58, 164, 'Harbormaster\'s Office, 3 Breakwater Road', { size: 9 });
  page.text(58, 176, 'Pemberly Falls, NY 12804', { size: 9 });
  page.text(50, 212, `Job: Dock C haul-out refit     Ship via: ${carrier}     F.O.B.: Origin     Terms: Net 30`, { size: 8.5 });
  flow.y = 224;
  flow.table([
    { header: 'Item', width: 0.12 },
    { header: 'Description', width: 0.5 },
    { header: 'Qty', width: 0.08, align: 'right' },
    { header: 'Price', width: 0.14, align: 'right' },
    { header: 'Extended', width: 0.16, align: 'right' },
  ], priced.rows.map((row) => [row.sku, row.description, String(row.quantity), amount(row.unit), amount(row.total)]), { size: 9, border: 'rules', headerFill: 0.9 });
  for (const [label, value] of [['Merchandise', amount(priced.subtotal)], [`Freight - ${carrier}`, amount(priced.freight)], ['Sales tax (resale certificate on file)', '0.00']]) {
    page.textRight(470, flow.y + 9, label, { size: 9 });
    page.textRight(562, flow.y + 9, value, { size: 9 });
    flow.y += 13;
  }
  page.textRight(470, flow.y + 14, 'Balance Due', { face: 'sans-bold', size: 10.5 });
  page.textRight(562, flow.y + 14, money(priced.total), { face: 'sans-bold', size: 10.5 });
  flow.y += 40;
  page.rect(50, flow.y, 512, 58, { fill: null, stroke: 0.5 });
  page.text(58, flow.y + 14, 'Remit to:', { face: 'sans-bold', size: 9 });
  page.text(110, flow.y + 14, `${issuer}, PO Box 2290, Gilchrist Harbor, ME 04112`, { size: 9 });
  page.text(110, flow.y + 27, 'ACH/Wire: Gilchrist Harbor Savings, acct. ending 6620', { size: 9 });
  page.text(110, flow.y + 40, `Please write invoice # ${invoiceNumber} on your check.`, { size: 9 });
  flow.y += 76;
  flow.paragraph('Coating products are mixed to order and are not returnable once tinted. Technical data sheets and safety data sheets for every product above are available on request.', { size: 8 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  const total = money(priced.total);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Marine coatings invoice whose issuer is known only from the page layout',
    kind: 'invoice',
    textLayer: 'native',
    pages: pages.length,
    categories: ['invoice', 'layout_parties', 'table', 'competing_dates'],
    structure: structure({
      tables: [[['Item', 'Description', 'Qty', 'Price', 'Extended'], ...priced.rows.map((row) => [row.sku, row.description, String(row.quantity), amount(row.unit), amount(row.total)])]],
      keyValues: [...meta, ['SOLD TO', customer], ['Balance Due', money(priced.total)]],
      routes: { 1: 'layout' },
    }),
    notes: `No sentence says who issued this invoice: the issuer is the small unlabelled block at the top left and the "Remit to" box at the foot; the customer is the largest name on the page, in a bold "SOLD TO" box, and a freight carrier is named twice. Dates are in ledger form: invoice ${ledgerDate(invoiceDate)}, order ${ledgerDate(orderDate)}, due ${ledgerDate(dueDate)}.`,
    gold: gold({
      type: 'Invoice',
      date: invoiceDate,
      role: 'invoice',
      forbiddenDates: [[dueDate, 'payment due date'], [orderDate, 'order date']],
      parties: [issuer],
      relation: 'from',
      roles: [[issuer, 'issuer'], [customer, 'customer']],
      forbiddenParties: [[customer, 'sold-to customer'], [carrier, 'freight carrier']],
      facts: [[total, total.slice(1)], [customer, 'Thornbury Harbor Marina'], [invoiceNumber]],
      forbiddenFacts: [],
      subjectTerms: ['coatings', 'antifouling', 'epoxy', 'marine'],
      readiness: 'ready',
      dateText: [ledgerDate(invoiceDate)],
    }),
  });
}

export function purchaseOrder() {
  const id = 'purchase-order';
  const buyer = 'Lamplighter Robotics Inc.';
  const vendor = 'Kestrel Instruments Ltd.';
  const poDate = '2026-01-29';
  const quoteDate = '2026-01-22';
  const deliveryDate = '2026-03-02';
  const cancelDate = '2026-03-16';
  const poNumber = 'LR-PO-260117';
  const buyerPerson = 'Juniper Okonkwo';
  const approver = 'Rafferty Delacroix-Hale';
  const priced = priceLines([
    { sku: 'KI-SM57-24', description: 'Brushless servo motor, 400 W, 24 V, IP65', quantity: 120, unit: 31275 },
    { sku: 'KI-ENC-17A', description: 'Absolute rotary encoder, 17-bit, hollow shaft', quantity: 120, unit: 8840 },
    { sku: 'KI-DRV-2', description: 'Dual-axis servo drive, EtherCAT', quantity: 60, unit: 46500 },
    { sku: 'KI-CAB-5M', description: 'Shielded motor/feedback cable assembly, 5 m', quantity: 240, unit: 3125 },
    { sku: 'KI-CAL', description: 'Factory calibration certificate per lot', quantity: 4, unit: 22500 },
  ], { taxRate: 0, freight: 0 });
  const flow = new Flow({ face: 'sans', fontSize: 9.5, margins: { top: 48, bottom: 60, left: 54, right: 54 }, keep: [buyer, vendor] });
  const page = flow.page;
  page.text(54, 70, buyer, { face: 'sans-bold', size: 15 });
  page.text(54, 84, 'Procurement Department - 2400 Lamplighter Drive, Kestrel Ridge, UT 84047', { size: 8.5, grey: 0.3 });
  page.text(54, 95, 'Tel. (801) 555-0133   purchasing@lamplighter-robotics.example', { size: 8.5, grey: 0.3 });
  page.textRight(558, 72, 'PURCHASE ORDER', { face: 'sans-bold', size: 18 });
  page.textRight(558, 88, poNumber, { face: 'sans', size: 11 });
  flow.y = 110;
  flow.table([
    { header: 'PO Number', width: 0.18 },
    { header: 'PO Date', width: 0.14 },
    { header: 'Buyer', width: 0.18 },
    { header: 'Ship Via', width: 0.16 },
    { header: 'F.O.B.', width: 0.16 },
    { header: 'Payment Terms', width: 0.18 },
  ], [[poNumber, longDate(poDate), buyerPerson, 'Best way, prepaid', 'Destination', '2% 10, Net 45']], { size: 8.5, headerFill: 0.85 });
  const top = flow.y;
  page.text(54, top + 10, 'VENDOR', { face: 'sans-bold', size: 8, grey: 0.35 });
  [vendor, 'Attn: Order Desk', '77 Calder Way', 'Marrow Springs, CO 80512', 'Vendor No. V-30981'].forEach((line, index) => page.text(54, top + 23 + index * 11.5, line, { size: 9.5, face: index === 0 ? 'sans-bold' : 'sans' }));
  page.text(306, top + 10, 'SHIP TO', { face: 'sans-bold', size: 8, grey: 0.35 });
  [`${buyer} - Receiving Dock 3`, '2400 Lamplighter Drive', 'Kestrel Ridge, UT 84047'].forEach((line, index) => page.text(306, top + 23 + index * 11.5, line, { size: 9.5 }));
  flow.y = top + 90;
  flow.paragraph(`Reference: your Quotation Q-55120 dated ${longDate(quoteDate)}. Prices below are firm per that quotation. Requested delivery date (on dock): ${longDate(deliveryDate)}. Partial shipments accepted with prior approval.`, { size: 9 });
  flow.table([
    { header: 'Line', width: 0.06, align: 'right' },
    { header: 'Part No.', width: 0.14 },
    { header: 'Description', width: 0.42 },
    { header: 'Qty', width: 0.08, align: 'right' },
    { header: 'Unit Price', width: 0.13, align: 'right' },
    { header: 'Ext. Price', width: 0.17, align: 'right' },
  ], priced.rows.map((row, index) => [String(index + 1), row.sku, row.description, String(row.quantity), amount(row.unit), amount(row.total)]), { size: 9, border: 'grid', headerFill: 0.9 });
  page.textRight(470, flow.y + 4, 'Order Total (USD)', { face: 'sans-bold', size: 10 });
  page.textRight(558, flow.y + 4, money(priced.total), { face: 'sans-bold', size: 10 });
  flow.y += 22;
  flow.heading('Terms and Conditions', { level: 3, size: 9.5 });
  [
    `1. Acceptance. This purchase order is accepted by Vendor's written acknowledgement or by shipment. If goods cannot be delivered by the requested delivery date, Vendor must notify Buyer within three business days of receipt of this order; Buyer may cancel any portion not shipped by ${longDate(cancelDate)} without liability.`,
    '2. Inspection. All goods are subject to inspection and testing by Buyer at destination. Nonconforming goods may be rejected and returned at Vendor\'s expense within thirty days of receipt.',
    '3. Certificates. Each lot must ship with a certificate of conformance and the factory calibration certificate listed above, referencing this PO number and the lot number.',
    '4. Invoicing. Invoice in duplicate to Accounts Payable at the address above, quoting the PO number and line numbers. Invoices without a PO number will be returned unpaid.',
    '5. Compliance. Vendor warrants that the goods comply with applicable product safety and export control regulations and are free of conflict minerals.',
  ].forEach((clause) => flow.paragraph(clause, { size: 8.5 }));
  flow.space(10);
  flow.page.text(flow.left, flow.y + 10, `Authorized by: /s/ ${approver}, Director of Supply Chain`, { size: 9.5 });
  flow.page.text(flow.left, flow.y + 24, `Buyer: ${buyerPerson}, (801) 555-0133`, { size: 9.5 });
  flow.y += 34;
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  const total = money(priced.total);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Robotics manufacturer\'s purchase order for servo components',
    kind: 'purchase_order',
    textLayer: 'native',
    pages: pages.length,
    categories: ['purchase_order', 'table', 'date_in_table', 'competing_dates'],
    structure: structure({
      tables: [
        [['PO Number', 'PO Date', 'Buyer', 'Ship Via', 'F.O.B.', 'Payment Terms'], [poNumber, longDate(poDate), buyerPerson, 'Best way, prepaid', 'Destination', '2% 10, Net 45']],
        [['Line', 'Part No.', 'Description', 'Qty', 'Unit Price', 'Ext. Price'], ...priced.rows.map((row, index) => [String(index + 1), row.sku, row.description, String(row.quantity), amount(row.unit), amount(row.total)])],
      ],
      keyValues: [['VENDOR', vendor], ['SHIP TO', `${buyer} - Receiving Dock 3`], ['Order Total (USD)', total]],
      routes: { 1: 'layout' },
    }),
    notes: `The PO date (${longDate(poDate)}) appears only as a cell in the header grid. The vendor's quotation date (${longDate(quoteDate)}), the requested delivery date (${longDate(deliveryDate)}), and the cancellation cut-off (${longDate(cancelDate)}) are written out in sentences and are easier to find. A purchase order is filed under the buyer that issued it; "to" the vendor is also accepted.`,
    gold: gold({
      type: 'Purchase Order',
      date: poDate,
      role: 'issuance',
      forbiddenDates: [[quoteDate, 'date of the vendor quotation it references'], [deliveryDate, 'requested delivery date'], [cancelDate, 'cancellation cut-off']],
      parties: [buyer],
      relation: 'from',
      acceptablePartySets: [{ parties: [vendor], relation: 'to' }, { parties: [buyer, vendor], relation: 'between' }],
      roles: [[buyer, 'issuer'], [buyer, 'buyer'], [vendor, 'recipient'], [vendor, 'seller']],
      forbiddenParties: [[buyerPerson, 'buyer contact'], [approver, 'approving manager']],
      facts: [[total, total.slice(1)], [vendor, 'Kestrel Instruments'], [poNumber]],
      forbiddenFacts: [],
      subjectTerms: ['servo', 'encoder', 'drive', 'servo motor'],
      readiness: 'ready',
      dateText: [longDate(poDate)],
    }),
  });
}
