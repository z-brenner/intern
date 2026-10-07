/// Scanned documents: clean office scans, a mixed digital/scanned amendment,
/// turned and skewed pages, a low-resolution receipt, a noisy statement, a
/// faint letter, two long scanned agreements, an OCR-corrupted text layer,
/// a two-frame fax, and a scanned patient form. Each carries `ocr_truth`:
/// exactly the text drawn on every scanned page.
import { Flow, Page, pageText, signatureStroke, visualText } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { boxBlur, downsample, grain, lowContrast, pdfImage, rasterize, rotate, skew, speckle, threshold } from '../lib/raster.mjs';
import { buildPdf } from '../lib/pdf.mjs';
import { addDays, amount, longDate, money, numericDate, shortMonthDate } from '../lib/format.mjs';
import { png, tiff } from '../lib/image.mjs';
import { result, signatureBlocks } from './common.mjs';
import { COMMERCIAL } from './clauses.mjs';
import { BUILDING_RULES, leaseExhibits, officeLeaseArticles } from './lease-clauses.mjs';
import { scanImages, scanPdf, truthFor } from './scanning.mjs';

/// A scanned letter-size page model with typewriter-like margins.
function typedFlow(options = {}) {
  return new Flow({ face: 'serif', fontSize: 11.5, leading: 1.45, margins: { top: 72, bottom: 72, left: 80, right: 80 }, curly: true, ...options });
}

export function scanCleanLease() {
  const id = 'scan-clean-lease-2p';
  const rng = Rng.from(id);
  const landlord = 'Dahlquist Family Rentals LLC';
  const tenant = 'Paloma Iwasaki';
  const made = '2026-02-20';
  const start = '2026-03-01';
  const end = '2027-02-28';
  const flow = typedFlow();
  flow.heading('RESIDENTIAL LEASE AGREEMENT', { level: 1, align: 'center', size: 15, face: 'sans-bold' });
  flow.paragraph(`This Residential Lease Agreement is made on ${longDate(made)} between ${landlord} ("Landlord") and ${tenant} ("Tenant") for the house at 1428 Juniper Loop, Kestrel Ridge, UT 84047 (the "Premises").`);
  const terms = [
    ['Term', `The lease term begins on ${longDate(start)} and ends on ${longDate(end)}. After that date the tenancy continues month to month unless either party gives at least thirty days' written notice.`],
    ['Rent', 'Tenant will pay rent of $2,150.00 per month, due on the first day of each month, by electronic transfer to the account Landlord designates. Rent received after the fifth day of the month is late, and Tenant will pay a late charge of $75.00.'],
    ['Security Deposit', 'Tenant has paid a security deposit of $2,150.00. Landlord will return the deposit, less any lawful deductions itemized in writing, within thirty days after Tenant returns the keys.'],
    ['Utilities', 'Tenant will pay for electricity, natural gas, internet, and trash collection. Landlord will pay for water and sewer service up to $90.00 per month; Tenant will reimburse any amount above that within ten days after receiving a copy of the bill.'],
    ['Occupants and Pets', 'Only Tenant and one minor child may live at the Premises. Tenant may keep one dog weighing under forty pounds; no other animals are allowed without Landlord\'s written consent.'],
    ['Maintenance', 'Tenant will keep the Premises clean, replace furnace filters every three months, and promptly report any water leak or needed repair. Landlord will maintain the roof, plumbing, heating, and appliances supplied with the house.'],
    ['Alterations', 'Tenant will not paint, install fixtures, or make other alterations without Landlord\'s written consent, and will not change the locks.'],
    ['Entry', 'Landlord may enter the Premises at reasonable times to inspect, make repairs, or show the house, after giving at least twenty-four hours\' notice, except in an emergency.'],
    ['Default', 'If Tenant fails to pay rent when due or breaks another term of this lease, Landlord may end the lease as allowed by law and recover unpaid rent and reasonable costs.'],
  ];
  terms.forEach(([title, body], index) => flow.paragraph([{ text: `${index + 1}. ${title}. `, face: 'sans-bold' }, { text: body }]));
  flow.paragraph('The parties have signed this lease on the date first written above.', { before: 6 });
  signatureBlocks(flow, [
    { heading: 'LANDLORD', entity: landlord, name: 'Florian Dahlquist', title: 'Manager' },
    { heading: 'TENANT', name: tenant },
  ], { rng: rng.fork('signatures'), size: 11, stacked: true });
  const pages = flow.finish();
  const images = scanImages(pages, { dpi: 300 });
  const bytes = scanPdf(images, { bits: 1 });
  const truth = truthFor(pages.map((page, index) => ({ number: index + 1, page })), {
    dates: [longDate(made), longDate(start), longDate(end)],
    names: [landlord, tenant],
    identifiers: ['1428 Juniper Loop'],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Two-page residential lease scanned at 300 DPI, black and white',
    kind: 'contract',
    textLayer: 'scan',
    pages: pages.length,
    categories: ['image_only_scan', 'contract'],
    notes: `A clean office scan with no text layer: every word must come from OCR. The lease is made on ${longDate(made)}; the term begins ${longDate(start)} (accepted, as a lease's commencement) and ends ${longDate(end)} (a trap). Both parties are named in the first sentence. A careful reader can file it, but a cautious one may send any scan to review, so either outcome is acceptable.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Residential Lease Agreement',
      acceptableTypes: ['Lease Agreement'],
      date: made,
      acceptableDates: [start],
      role: 'effective',
      forbiddenDates: [[end, 'end of the lease term']],
      parties: [landlord, tenant],
      relation: 'between',
      roles: [[landlord, 'landlord'], [tenant, 'tenant']],
      forbiddenParties: [['Florian Dahlquist', 'manager signing for the landlord']],
      facts: [['$2,150.00', '$2,150', '2,150'], ['1428 Juniper Loop', 'Juniper Loop'], [tenant, 'Iwasaki']],
      subjectTerms: ['lease', 'house', 'rent', 'Kestrel Ridge'],
      readiness: 'either',
      dateText: [longDate(made)],
    }),
  });
}

export function scanMixedAmendment() {
  const id = 'scan-mixed-amendment';
  const rng = Rng.from(id);
  const supplier = 'Hollowell Kitchenware Inc.';
  const distributor = 'Driftmoor Home Goods Distributors LLC';
  const own = '2026-07-15';
  const original = '2024-01-04';
  const pricing = '2026-09-01';
  const newEnd = '2029-12-31';
  const signedSupplier = '2026-07-20';
  const signedDistributor = '2026-07-22';
  const flow = new Flow({ face: 'serif', fontSize: 11, leading: 1.4, margins: { top: 72, bottom: 72 }, keep: [supplier, distributor],
    footer: (page, { number }) => page.textCenter(306, 760, `First Amendment to Distribution Agreement - Page ${number} of 3`, { face: 'sans', size: 8, grey: 0.35 }) });
  flow.heading('FIRST AMENDMENT TO DISTRIBUTION AGREEMENT', { level: 1, align: 'center', size: 14 });
  flow.paragraph(`This First Amendment to Distribution Agreement (this "Amendment") is made and entered into as of ${longDate(own)} by and between ${supplier}, an Ohio corporation ("Supplier"), and ${distributor}, a Washington limited liability company ("Distributor").`);
  flow.heading('Recitals', { level: 3 });
  flow.paragraph(`A. Supplier and Distributor are parties to a Distribution Agreement dated ${longDate(original)} (the "Agreement"), under which Distributor is Supplier's exclusive distributor of cookware, bakeware, and kitchen tools to independent retailers in Idaho, Montana, and Wyoming.`);
  flow.paragraph('B. Distributor has opened a warehouse in Halden Bay, Washington, and the parties wish to expand the territory, revise the price schedule and minimum purchase commitments, and extend the term.');
  flow.paragraph('The parties therefore agree as follows:');
  [
    ['Territory', 'Schedule A of the Agreement is amended to add the States of Oregon and Washington to the Territory. Distributor\'s exclusivity in the added States is limited to independent retailers with fewer than ten stores; Supplier may continue to sell directly to the national chains listed in Schedule A-1.'],
    ['Prices', `The price schedule attached to this Amendment as Schedule B replaces Schedule B of the Agreement for all orders placed on or after ${longDate(pricing)}. Orders placed before that date are invoiced at the prices in effect when placed. Supplier may adjust prices once in each calendar year on ninety days' notice, by no more than the percentage change in its published wholesale price list.`],
    ['Minimum Purchases', 'Section 5.2 of the Agreement is amended so that Distributor\'s minimum annual purchases, measured at net invoice prices, are $1,850,000 for 2026, $2,400,000 for 2027, $2,750,000 for 2028, and $3,100,000 for 2029. If Distributor fails to meet the minimum for a year, Supplier\'s only remedy is to end Distributor\'s exclusivity in the added States on sixty days\' notice.'],
    ['Term', `Section 12.1 of the Agreement is amended to extend the term through ${longDate(newEnd)}. Either party may terminate the Agreement without cause on one hundred eighty days' notice given after ${longDate('2027-12-31')}.`],
    ['Marketing Support', 'Supplier will provide Distributor a cooperative marketing allowance of two percent of net purchases in the added States during the first eighteen months after the date of this Amendment, to be used for trade shows, retailer training, and in-store displays approved in advance by Supplier.'],
    ['Effect of Amendment', 'Except as amended by this Amendment, the Agreement remains in full force and effect. If this Amendment conflicts with the Agreement, this Amendment controls. This Amendment may be signed in counterparts, and a scanned signature is as effective as an original.'],
  ].forEach(([title, body], index) => flow.paragraph([{ text: `${index + 1}. ${title}. `, face: 'sans-bold' }, { text: body }]));
  flow.heading('Schedule B - Price Schedule (effective for orders on or after the date stated in Section 2)', { level: 3 });
  flow.table([{ header: 'Item', width: 0.16 }, { header: 'Description', width: 0.5 }, { header: 'Case pack', width: 0.14, align: 'right' }, { header: 'Price per case', width: 0.2, align: 'right' }], [
    ['HK-1010', 'Tri-ply stainless saucepan, 2 qt', '6', '$171.00'],
    ['HK-1024', 'Tri-ply stainless saute pan, 12 in', '4', '$198.40'],
    ['HK-2201', 'Aluminized steel half-sheet pan', '12', '$96.00'],
    ['HK-2240', 'Nonstick loaf pan, 9 x 5 in', '12', '$84.60'],
    ['HK-3105', 'Silicone spatula set (3)', '24', '$151.20'],
    ['HK-3310', 'Bench scraper with ruler', '24', '$88.80'],
    ['HK-4402', 'Enameled cast iron Dutch oven, 5.5 qt', '2', '$142.00'],
  ], { size: 9.5 });
  const digital = flow.finish();
  if (digital.length !== 2) throw new Error(`${id}: the digital part must be two pages, laid out ${digital.length}`);
  // Page 3: the signature page, printed, signed in ink, and scanned back in.
  const signature = typedFlow({ footer: (page) => page.textCenter(306, 760, 'First Amendment to Distribution Agreement - Page 3 of 3', { face: 'sans', size: 8 }) });
  signature.paragraph('[Signature page to First Amendment to Distribution Agreement]', { align: 'center', size: 10.5 });
  signature.paragraph('IN WITNESS WHEREOF, the parties have signed this Amendment, each by its authorized representative, on the dates written below.', { before: 8 });
  signatureBlocks(signature, [
    { heading: 'SUPPLIER', entity: supplier, name: 'Odalys Kettleborough', title: 'Vice President, Sales', date: longDate(signedSupplier) },
    { heading: 'DISTRIBUTOR', entity: distributor, name: 'Joaquin Halloran', title: 'Managing Member', date: longDate(signedDistributor) },
  ], { rng: rng.fork('signatures'), stacked: true, size: 11 });
  const signaturePage = signature.finish()[0];
  const [image] = scanImages([signaturePage], { dpi: 300 });
  const scanned = new Page();
  scanned.image(0, 0, 612, 792, pdfImage(threshold(image), 1));
  const bytes = buildPdf([...digital, scanned]);
  const truth = truthFor([{ number: 3, page: signaturePage }], {
    dates: [longDate(signedSupplier), longDate(signedDistributor)],
    names: [supplier, distributor, 'Odalys Kettleborough', 'Joaquin Halloran'],
    identifiers: [],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: [...digital.map((page) => pageText(page)), truth.pages[0].text],
    title: 'Distribution agreement amendment: two digital pages and a scanned signature page',
    kind: 'amendment',
    textLayer: 'mixed',
    pages: 3,
    categories: ['mixed_scan', 'amendment', 'referenced_agreement', 'competing_dates', 'contract'],
    notes: `Pages 1-2 have a text layer; page 3 is a signed signature page scanned back in at 300 DPI, so its text exists only through OCR. The amendment's own date (${longDate(own)}) is in the first sentence; the original agreement (${longDate(original)}), the new price effective date (${longDate(pricing)}), the new term end (${longDate(newEnd)}), and the two signing dates on the scanned page (${longDate(signedSupplier)}, ${longDate(signedDistributor)}) are traps.`,
    ocrTruth: truth,
    gold: gold({
      type: 'First Amendment to Distribution Agreement',
      date: own,
      role: 'amendment',
      forbiddenDates: [[original, 'date of the distribution agreement being amended'], [pricing, 'effective date of the new price schedule'], [newEnd, 'new end of term'], [signedSupplier, 'supplier signature date on the scanned page'], [signedDistributor, 'distributor signature date on the scanned page']],
      parties: [supplier, distributor],
      relation: 'between',
      roles: [[supplier, 'seller'], [distributor, 'buyer']],
      forbiddenParties: [['Odalys Kettleborough', 'signatory'], ['Joaquin Halloran', 'signatory']],
      facts: [['Oregon', 'Washington'], ['$1,850,000', '1,850,000'], [longDate(newEnd), '2029']],
      subjectTerms: ['amendment', 'distribution', 'territory', 'kitchenware'],
      readiness: 'either',
      dateText: [longDate(own)],
    }),
  });
}

export function scanRotatedInvoice() {
  const id = 'scan-rotated-90-invoice';
  const issuer = 'Wrenfield Restaurant Supply Co.';
  const customer = 'Larkspur Bistro LLC';
  const invoiceDate = '2026-04-14';
  const dueDate = '2026-05-14';
  const number = 'WRS-55871';
  const items = [['Combi oven gasket kit', 1, 18900], ['Sheet pan rack, 20-tier', 2, 26450], ['Cutting boards, color-coded set', 3, 6475], ['Thermometer probes (6-pack)', 2, 4290], ['Labor: oven door hinge repair (hours)', 2, 11000]];
  const subtotal = items.reduce((sum, [, quantity, unit]) => sum + quantity * unit, 0);
  const tax = Math.round(subtotal * 0.0825);
  const flow = new Flow({ face: 'sans', fontSize: 10.5, leading: 1.4, margins: { top: 64, bottom: 64, left: 72, right: 72 }, curly: true, keep: [issuer, customer] });
  flow.heading(issuer, { level: 1, size: 16, before: 0 });
  flow.paragraph('900 Ostrander Road, Wrenfield, OH 44012  -  (440) 555-0137', { size: 9.5, after: 14 });
  flow.heading('INVOICE', { level: 1, size: 18, before: 0 });
  flow.fields([['Invoice No.:', number], ['Invoice Date:', longDate(invoiceDate)], ['Due Date:', longDate(dueDate)], ['Bill To:', `${customer}, 77 Fennel Court, Wrenfield, OH 44016`]], { labelWidth: 96, size: 10.5, face: 'sans' });
  for (const [description, quantity, unit] of items) flow.paragraph(`${description}  ${quantity} x ${money(unit)} = ${money(quantity * unit)}`, { size: 10.5, after: 3 });
  flow.space(6);
  flow.paragraph(`Subtotal: ${money(subtotal)}`, { size: 10.5, after: 2 });
  flow.paragraph(`Sales tax (8.25%): ${money(tax)}`, { size: 10.5, after: 2 });
  flow.paragraph(`Total due: ${money(subtotal + tax)}`, { face: 'sans-bold', size: 12 });
  flow.paragraph('Terms: Net 30. Thank you for your business.', { size: 10 });
  const pages = flow.finish();
  const images = scanImages(pages, { dpi: 300, degrade: (image) => rotate(threshold(image), 90) });
  const bytes = png(images[0], { bits: 1, dpi: 300, description: 'InternBench scan' });
  const truth = truthFor([{ number: 1, page: pages[0] }], { dates: [longDate(invoiceDate), longDate(dueDate)], names: [issuer, customer], identifiers: [number] });
  return result({
    id,
    extension: 'png',
    files: [{ name: `${id}.png`, bytes }],
    text: [truth.pages[0].text],
    title: 'Restaurant supply invoice scanned sideways (rotated 90 degrees)',
    kind: 'invoice',
    textLayer: 'scan',
    pages: 1,
    categories: ['rotated_scan', 'png', 'invoice', 'image_only_scan'],
    notes: `The page was fed sideways: the image is turned 90 degrees clockwise, so an upright OCR pass reads nothing useful and the reader has to find the orientation. Once upright it is an ordinary invoice dated ${longDate(invoiceDate)} with a due date (${longDate(dueDate)}) and a bill-to customer as traps.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Invoice',
      date: invoiceDate,
      role: 'invoice',
      forbiddenDates: [[dueDate, 'payment due date']],
      parties: [issuer],
      relation: 'from',
      roles: [[issuer, 'issuer'], [customer, 'customer']],
      forbiddenParties: [[customer, 'bill-to customer']],
      facts: [[money(subtotal + tax), amount(subtotal + tax)], [customer, 'Larkspur Bistro'], [number]],
      subjectTerms: ['restaurant supply', 'oven', 'invoice'],
      readiness: 'either',
      dateText: [longDate(invoiceDate)],
    }),
  });
}

export function scanUpsideDownPo() {
  const id = 'scan-upside-down-po';
  const buyer = 'Thornbury Outdoor Gear Co.';
  const vendor = 'Ridgewell Textiles Ltd.';
  const poDate = '2026-06-03';
  const deliver = '2026-07-15';
  const number = 'TOG-PO-4471';
  const flow = typedFlow({ keep: [buyer, vendor] });
  flow.heading(buyer, { level: 1, size: 16, face: 'sans-bold', before: 0 });
  flow.paragraph('Purchasing - 41 Starling Way, Westharrow, MI 49101 - (269) 555-0153', { size: 10, face: 'sans', after: 12 });
  flow.heading('PURCHASE ORDER', { level: 1, size: 17, face: 'sans-bold', before: 0 });
  flow.fields([['PO Number:', number], ['PO Date:', longDate(poDate)], ['Vendor:', `${vendor}, 6 Mill Race Lane, Linden Cross, PA 19047`], ['Ship To:', `${buyer}, Receiving, 41 Starling Way, Westharrow, MI 49101`], ['Deliver By:', longDate(deliver)], ['Terms:', 'Net 45, FOB destination']], { labelWidth: 96, size: 11 });
  const lines = [['Ripstop nylon, 70D, forest green (yards)', 1200, 685], ['Waterproof-breathable membrane laminate (yards)', 800, 1240], ['Polyester mesh, 40 in. (yards)', 450, 395], ['Coil zipper tape, #5 (yards)', 2000, 88]];
  for (const [description, quantity, unit] of lines) flow.paragraph(`${description}: ${quantity} at ${money(unit)} = ${money(quantity * unit)}`, { size: 11, after: 3 });
  const total = lines.reduce((sum, [, quantity, unit]) => sum + quantity * unit, 0);
  flow.paragraph(`Order total: ${money(total)}`, { face: 'sans-bold', size: 12, before: 6 });
  flow.paragraph('All fabric must meet the attached color standard and ship with mill test reports. Partial shipments require approval. Please acknowledge this order within three business days.', { size: 10.5 });
  flow.paragraph('Authorized by: Imogen Wexley, Purchasing Manager', { size: 11, before: 8 });
  const pages = flow.finish();
  const images = scanImages(pages, { dpi: 300, degrade: (image) => rotate(threshold(image), 180) });
  const bytes = scanPdf(images, { bits: 1 });
  const truth = truthFor([{ number: 1, page: pages[0] }], { dates: [longDate(poDate), longDate(deliver)], names: [buyer, vendor], identifiers: [number] });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: [truth.pages[0].text],
    title: 'Purchase order scanned upside down (page image rotated 180 degrees)',
    kind: 'purchase_order',
    textLayer: 'scan',
    pages: 1,
    categories: ['rotated_scan', 'purchase_order', 'image_only_scan'],
    notes: `The sheet went through the scanner upside down, so the PDF page image is rotated 180 degrees (the page carries no /Rotate fix). The PO is dated ${longDate(poDate)}; the delivery date (${longDate(deliver)}) is a trap. Issued by the buyer; "to" the vendor is accepted.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Purchase Order',
      date: poDate,
      role: 'issuance',
      forbiddenDates: [[deliver, 'deliver-by date']],
      parties: [buyer],
      relation: 'from',
      acceptablePartySets: [{ parties: [vendor], relation: 'to' }],
      roles: [[buyer, 'issuer'], [buyer, 'buyer'], [vendor, 'recipient'], [vendor, 'seller']],
      forbiddenParties: [['Imogen Wexley', 'purchasing manager']],
      facts: [[money(total), amount(total)], [vendor, 'Ridgewell'], [number]],
      subjectTerms: ['fabric', 'nylon', 'purchase order'],
      readiness: 'either',
      dateText: [longDate(poDate)],
    }),
  });
}

export function scanSkewedNotice() {
  const id = 'scan-skewed-notice';
  const rng = Rng.from(id);
  const landlord = 'Harrowgate Apartments LLC';
  const tenant = 'Wendell Okafor';
  const noticeDate = '2026-08-03';
  const leaseDate = '2025-10-01';
  const ends = '2026-09-30';
  const flow = typedFlow({ keep: [landlord, tenant] });
  flow.paragraph(landlord, { face: 'sans-bold', size: 14, after: 0 });
  flow.paragraph('Leasing Office - 3300 Harrow Lane - Briarport, NC 27514 - (919) 555-0164', { face: 'sans', size: 9.5, after: 18 });
  flow.paragraph(longDate(noticeDate), { after: 12 });
  for (const line of [tenant, '3300 Harrow Lane, Apartment 12B', 'Briarport, NC 27514']) flow.paragraph(line, { after: 0 });
  flow.space(12);
  flow.heading('NOTICE OF NONRENEWAL OF LEASE', { level: 2, align: 'center', face: 'sans-bold' });
  flow.paragraph(`Dear ${tenant}:`);
  flow.paragraph(`This is written notice that ${landlord} will not renew your Apartment Lease dated ${longDate(leaseDate)} for Apartment 12B. Your lease term ends on ${longDate(ends)}, and you must move out and return all keys and parking permits to the leasing office by 5:00 p.m. on that date.`);
  flow.paragraph('This decision is not based on any default by you. The building is scheduled for a full renovation of its plumbing and electrical systems beginning this fall, and all units on the twelfth floor must be vacant while the work is done.');
  flow.paragraph('Please schedule a move-out inspection with the leasing office at least one week before you leave. Your security deposit, less any lawful deductions, will be returned to the forwarding address you provide within the time required by law. If you would like to transfer to another of our buildings, we will waive the application fee and give your application priority.');
  flow.paragraph('Thank you for having been our resident. Please call us with any questions.');
  flow.paragraph('Sincerely,', { after: 20 });
  flow.paragraph('Greer Sandoval, Property Manager', { after: 0 });
  flow.paragraph(`for ${landlord}`, { after: 0 });
  const pages = flow.finish();
  // A sheet laid three degrees off true on the glass, scanned in grey.
  const images = scanImages(pages, { dpi: 300, degrade: (image) => grain(skew(image, 3), rng.fork('grain'), 6) });
  const bytes = tiff(images, { dpi: 300 });
  const truth = truthFor([{ number: 1, page: pages[0] }], { dates: [longDate(noticeDate), longDate(leaseDate), longDate(ends)], names: [landlord, tenant], identifiers: ['Apartment 12B'] });
  return result({
    id,
    extension: 'tiff',
    files: [{ name: `${id}.tiff`, bytes }],
    text: [truth.pages[0].text],
    title: 'Landlord\'s notice of lease nonrenewal, scanned 3 degrees off square (TIFF)',
    kind: 'notice',
    textLayer: 'scan',
    pages: 1,
    categories: ['rotated_scan', 'tiff', 'notice', 'image_only_scan', 'referenced_agreement'],
    notes: `A grey 300-DPI TIFF of a letter laid three degrees off square. The notice is dated ${longDate(noticeDate)}; it refers to the lease dated ${longDate(leaseDate)} (a trap) and brings the tenancy to an end on ${longDate(ends)}, which is accepted as the event the notice exists to bring about. The notice is about the tenant.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Notice of Nonrenewal of Lease',
      acceptableTypes: ['Notice of Nonrenewal', 'Lease Nonrenewal Notice'],
      date: noticeDate,
      acceptableDates: [ends],
      role: 'notice',
      forbiddenDates: [[leaseDate, 'date of the lease that is not being renewed']],
      parties: [tenant],
      relation: 'for',
      acceptablePartySets: [{ parties: [tenant], relation: 'to' }, { parties: [landlord], relation: 'from' }],
      roles: [[tenant, 'subject'], [tenant, 'recipient'], [tenant, 'tenant'], [landlord, 'issuer'], [landlord, 'landlord']],
      forbiddenParties: [['Greer Sandoval', 'property manager who signs']],
      facts: [[landlord, 'Harrowgate'], ['Apartment 12B', '12B'], [longDate(ends), 'September 30']],
      subjectTerms: ['nonrenewal', 'lease', 'renovation', 'move out'],
      readiness: 'either',
      dateText: [longDate(noticeDate)],
    }),
  });
}

export function scanLowResReceipt() {
  const id = 'scan-low-res-receipt';
  const store = 'Quarry Bend Hardware & Feed';
  const saleDate = '2026-05-09';
  const promoEnds = '2026-05-31';
  const returnBy = '2026-06-08';
  const items = [['FENCE STAPLES 1-3/4 50LB', 1, 8999], ['T-POST 6FT GREEN', 25, 579], ['BARBED WIRE 12.5GA 1320FT', 2, 8649], ['POST DRIVER HEAVY DUTY', 1, 4495], ['WORK GLOVES LEATHER L', 2, 1299], ['MINERAL TUB 125LB', 1, 7450]];
  const subtotal = items.reduce((sum, [, quantity, unit]) => sum + quantity * unit, 0);
  const tax = Math.round(subtotal * 0.0975);
  // A receipt is a strip of thermal paper about 3.15 inches wide.
  const page = new Page({ width: 227, height: 560 });
  const lines = [
    ['QUARRY BEND HARDWARE & FEED', 'sans-bold', 9.5, 'center'],
    ['412 DEPOT STREET', 'sans', 8.5, 'center'],
    ['QUARRY BEND, TN 37201', 'sans', 8.5, 'center'],
    ['(615) 555-0142', 'sans', 8.5, 'center'],
    ['', 'sans', 8.5],
    [`STORE 3  REG 2  CASHIER HOLLIS`, 'sans', 8.5],
    [`DATE ${numericDate(saleDate)}  TIME 14:37`, 'sans', 8.5],
    ['--------------------------------------', 'sans', 8.5],
    ...items.flatMap(([description, quantity, unit]) => [[description, 'sans', 8.5], [`  ${quantity} @ ${amount(unit)}    ${amount(quantity * unit)}`, 'sans', 8.5]]),
    ['--------------------------------------', 'sans', 8.5],
    [`SUBTOTAL    ${amount(subtotal)}`, 'sans', 9],
    [`TAX 9.75%    ${amount(tax)}`, 'sans', 9],
    [`TOTAL    ${amount(subtotal + tax)}`, 'sans-bold', 10],
    [`VISA ****4471    ${amount(subtotal + tax)}`, 'sans', 8.5],
    ['AUTH 08812C  APPROVED', 'sans', 8.5],
    ['', 'sans', 8.5],
    [`SPRING FENCING SALE THRU ${numericDate(promoEnds)}`, 'sans', 8.5],
    [`RETURNS WITH RECEIPT BY ${numericDate(returnBy)}`, 'sans', 8.5],
    ['THANK YOU FOR SHOPPING LOCAL', 'sans-bold', 8.5, 'center'],
  ];
  let y = 22;
  for (const [text, face, size, align] of lines) {
    y += size * 1.45;
    if (!text) continue;
    if (align === 'center') page.textCenter(113.5, y, text, { face, size });
    else page.text(12, y, text, { face, size });
  }
  // Drawn at 300 DPI and averaged down to 100, as a low-resolution
  // scanner setting samples the strip; kept grey, never thresholded.
  const [full] = scanImages([page], { dpi: 300 });
  const image = downsample(full, 3);
  const bytes = png(image, { bits: 8, dpi: 100, description: 'InternBench scan' });
  const truth = truthFor([{ number: 1, page }], { dates: [numericDate(saleDate), numericDate(promoEnds), numericDate(returnBy)], names: ['QUARRY BEND HARDWARE & FEED'], identifiers: ['08812C', '4471'] });
  return result({
    id,
    extension: 'png',
    files: [{ name: `${id}.png`, bytes }],
    text: [truth.pages[0].text],
    title: 'Farm store receipt scanned at 100 DPI in grey',
    kind: 'receipt',
    textLayer: 'scan',
    pages: 1,
    categories: ['low_resolution_scan', 'png', 'image_only_scan'],
    notes: `A thermal receipt scanned at 100 DPI: 8.5-point text is about twelve pixels to the em, so digits are easily misread. The sale date is ${numericDate(saleDate)}; the promotion end (${numericDate(promoEnds)}) and return deadline (${numericDate(returnBy)}) are traps, and the promotion date's day above 12 settles month-first order. The store name is printed in capitals. Its digits cannot be trusted at this resolution, so it belongs in review.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Receipt',
      acceptableTypes: ['Sales Receipt', 'Store Receipt'],
      date: saleDate,
      role: 'issuance',
      forbiddenDates: [[promoEnds, 'end of a sale promotion'], [returnBy, 'return deadline']],
      parties: [store],
      relation: 'from',
      roles: [[store, 'issuer'], [store, 'seller']],
      forbiddenParties: [],
      facts: [[money(subtotal + tax), amount(subtotal + tax)], ['barbed wire', 'T-POST']],
      subjectTerms: ['fencing', 'fence staples', 'barbed wire', 'hardware'],
      readiness: 'needs_review',
      dateText: [numericDate(saleDate)],
      partyText: { [store]: ['QUARRY BEND HARDWARE & FEED'] },
    }),
  });
}

export function scanNoisyStatement() {
  const id = 'scan-noisy-statement';
  const rng = Rng.from(id);
  const supplier = 'Brindle Paper & Janitorial Supply';
  const customer = 'Copper Flats Dental Group PLLC';
  const statementDate = '2026-08-31';
  const payBy = '2026-09-30';
  const entries = [['2026-06-02', 'Invoice 66120', 41275, 0], ['2026-06-19', 'Invoice 66388', 18640, 0], ['2026-07-01', 'Payment - check 2214', 0, 41275], ['2026-07-08', 'Invoice 66702', 27390, 0], ['2026-07-22', 'Invoice 66951', 9815, 0], ['2026-08-05', 'Payment - ACH', 0, 18640], ['2026-08-12', 'Invoice 67240', 33460, 0], ['2026-08-26', 'Credit memo 1187 (returned gloves)', 0, 2150]];
  let balance = 12480;
  const flow = typedFlow({ keep: [supplier, customer] });
  flow.paragraph(supplier, { face: 'sans-bold', size: 14, after: 0 });
  flow.paragraph('77 Cinder Street - Copper Flats, AZ 85219 - (480) 555-0115', { face: 'sans', size: 9.5, after: 14 });
  flow.heading('STATEMENT OF ACCOUNT', { level: 1, face: 'sans-bold', size: 15, before: 0 });
  flow.fields([['Statement Date:', numericDate(statementDate)], ['Account:', `${customer} (No. 30418)`], ['Address:', '1400 Calder Way, Suite 2, Copper Flats, AZ 85219']], { labelWidth: 110, size: 11 });
  flow.paragraph(`Balance forward from ${numericDate('2026-05-31')}: ${amount(balance)}`, { size: 11 });
  for (const [date, description, charge, credit] of entries) {
    balance += charge - credit;
    flow.paragraph(`${numericDate(date)}  ${description}  ${charge ? amount(charge) : ''}${credit ? `(${amount(credit)})` : ''}  Balance ${amount(balance)}`, { size: 10.5, after: 2 });
  }
  flow.paragraph(`Aging: 0-30 days ${amount(33460 - 2150)}   31-60 days ${amount(27390 + 9815)}   over 60 days ${amount(12480)}`, { size: 10.5, before: 6 });
  flow.paragraph(`AMOUNT DUE: ${money(balance)}`, { face: 'sans-bold', size: 13 });
  flow.paragraph(`Please pay by ${numericDate(payBy)}. Balances over 60 days are subject to a 1.5% monthly service charge. Remit to the address above or call (480) 555-0115 to pay by card.`, { size: 10.5 });
  const pages = flow.finish();
  const images = scanImages(pages, { dpi: 200, degrade: (image) => speckle(grain(boxBlur(image, 1), rng.fork('grain'), 14), rng.fork('speckle'), 2600) });
  const bytes = scanPdf(images, { bits: 8 });
  const truth = truthFor([{ number: 1, page: pages[0] }], { dates: [numericDate(statementDate), numericDate(payBy)], names: [supplier, customer], identifiers: ['30418', '67240'] });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: [truth.pages[0].text],
    title: 'Supplier statement of account scanned in grey with speckle and blur',
    kind: 'statement',
    textLayer: 'scan',
    pages: 1,
    categories: ['noisy_scan', 'statement', 'image_only_scan', 'competing_dates'],
    notes: `A worn 200-DPI grey scan: box blur, grain, and dust specks. The statement date (${numericDate(statementDate)}) is a labelled field; eight transaction dates, the balance-forward date, and the pay-by date (${numericDate(payBy)}) compete with it. Issued by the supplier to its customer. Speckle corrupts digits, so it belongs in review.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Statement of Account',
      acceptableTypes: ['Account Statement', 'Statement'],
      date: statementDate,
      role: 'issuance',
      forbiddenDates: [[payBy, 'pay-by date'], ['2026-05-31', 'balance-forward date'], ['2026-08-26', 'last transaction date']],
      parties: [supplier],
      relation: 'from',
      acceptablePartySets: [{ parties: [customer], relation: 'for' }, { parties: [customer], relation: 'to' }],
      roles: [[supplier, 'issuer'], [supplier, 'seller'], [customer, 'customer'], [customer, 'subject'], [customer, 'recipient']],
      forbiddenParties: [],
      facts: [[money(balance), amount(balance)], [customer, 'Copper Flats Dental']],
      subjectTerms: ['statement', 'balance', 'invoice', 'janitorial'],
      readiness: 'needs_review',
      dateText: [numericDate(statementDate)],
    }),
  });
}

export function scanFaintLetter() {
  const id = 'scan-faint-letter';
  const employee = 'Mireille Saltonstall';
  const employer = 'Ashgrove Veterinary Clinic';
  const letterDate = '2026-03-02';
  const lastDay = '2026-03-27';
  const flow = typedFlow({ keep: [employee, employer] });
  for (const line of [employee, '18 Alder Court', 'Bellmoor, WI 53511']) flow.paragraph(line, { after: 0 });
  flow.space(14);
  flow.paragraph(longDate(letterDate), { after: 14 });
  for (const line of ['Dr. Rufus Pemberton, Practice Owner', employer, '640 Kingsfold Avenue', 'Bellmoor, WI 53510']) flow.paragraph(line, { after: 0 });
  flow.space(12);
  flow.paragraph('Re: Letter of Resignation', { face: 'sans-bold' });
  flow.paragraph('Dear Dr. Pemberton:');
  flow.paragraph(`Please accept this letter as formal notice of my resignation from my position as Lead Veterinary Technician at ${employer}. My last day of work will be ${longDate(lastDay)}, which gives the clinic more than three weeks to arrange coverage for the surgery schedule.`);
  flow.paragraph('I have accepted a position as a teaching technician at the regional college\'s veterinary technology program, which will let me train the next group of technicians. It was not an easy decision. The eight years I have spent at the clinic have taught me most of what I know, and I am grateful for the trust you placed in me, especially when you asked me to lead the move to the new surgical suite.');
  flow.paragraph('Before I leave I will finish the controlled-substance log audit, update the anesthesia monitoring checklists, and train whoever will take over scheduling for the surgery team. I am also happy to help interview candidates for my position.');
  flow.paragraph('Thank you again for everything. I hope we can stay in touch.');
  flow.paragraph('Sincerely,', { after: 26 });
  flow.page.stroke(signatureStroke(Rng.from(`${id}/signature`), flow.left + 6, flow.y - 4, 120), { width: 1.1, grey: 0.15 });
  flow.paragraph(employee, { after: 0 });
  const pages = flow.finish();
  const images = scanImages(pages, { dpi: 200, degrade: (image) => lowContrast(image, { paper: 214, ink: 138, falloff: 0.22 }) });
  const bytes = png(images[0], { bits: 8, dpi: 200, description: 'InternBench scan' });
  const truth = truthFor([{ number: 1, page: pages[0] }], { dates: [longDate(letterDate), longDate(lastDay)], names: [employee, employer, 'Dr. Rufus Pemberton'], identifiers: [] });
  return result({
    id,
    extension: 'png',
    files: [{ name: `${id}.png`, bytes }],
    text: [truth.pages[0].text],
    title: 'Resignation letter scanned faint, on grey paper under uneven light',
    kind: 'letter',
    textLayer: 'scan',
    pages: 1,
    categories: ['noisy_scan', 'letter', 'png', 'image_only_scan'],
    notes: `Low-toner print on grey paper, scanned at 200 DPI with the light falling off toward one corner: ink is mid-grey and the background darkens by about a fifth. Dated ${longDate(letterDate)}; the last day of work (${longDate(lastDay)}) is accepted as the event a resignation notice brings about. From the employee to the clinic.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Letter of Resignation',
      acceptableTypes: ['Resignation Letter'],
      date: letterDate,
      acceptableDates: [lastDay],
      role: 'notice',
      forbiddenDates: [],
      parties: [employee],
      relation: 'from',
      acceptablePartySets: [{ parties: [employer], relation: 'to' }, { parties: [employee], relation: 'for' }],
      roles: [[employee, 'issuer'], [employee, 'employee'], [employee, 'subject'], [employer, 'recipient'], [employer, 'employer']],
      forbiddenParties: [['Dr. Rufus Pemberton', 'practice owner the letter is addressed to']],
      facts: [['March 27, 2026', 'March 27'], [employer, 'Ashgrove'], ['Lead Veterinary Technician', 'veterinary technician']],
      subjectTerms: ['resignation', 'veterinary', 'last day'],
      readiness: 'either',
      dateText: [longDate(letterDate)],
    }),
  });
}

/// Distribution terms for the scanned ten-page agreement; the general
/// clauses come from the shared commercial clause bank.
function distributionSections(c) {
  return [
    ['Appointment', [`Supplier appoints Distributor as its exclusive distributor of the Products to Customers in the Territory during the Term, and Distributor accepts the appointment. Supplier will not appoint any other distributor or agent for the Products in the Territory, but reserves the right to sell directly to the national laboratory accounts listed in Exhibit B and to sell through its own website to customers anywhere.`, 'Distributor will not actively seek customers for the Products outside the Territory, establish a branch or warehouse for the Products outside the Territory, or appoint sub-distributors without Supplier\'s prior written consent.']],
    ['Products', ['"Products" means the microscopes, stereo microscopes, digital cameras, objective lenses, and accessories listed in Exhibit A, as Supplier may update the list on sixty days\' notice. Supplier may discontinue any Product on ninety days\' notice and will offer Distributor a last-time buy of discontinued Products at the then-current price.']],
    ['Minimum Purchases', [`Distributor will purchase Products with a net invoice value of at least the amounts in Exhibit C for each contract year. If Distributor fails to meet the minimum for any contract year, Supplier may, as its sole remedy, convert Distributor's appointment to a non-exclusive one on thirty days' notice, given within ninety days after the end of that contract year.`]],
    ['Orders and Forecasts', ['Distributor will submit written purchase orders through Supplier\'s ordering portal. Each order is subject to acceptance by Supplier, which will be deemed given unless Supplier rejects the order within three business days. By the tenth day of each month Distributor will provide a rolling six-month forecast of its requirements, the first two months of which are binding.', 'Supplier will use commercially reasonable efforts to ship accepted orders within fifteen business days. If Supplier cannot fill an order because of a shortage, it will allocate available Products among its distributors in proportion to their purchases over the preceding six months.']],
    ['Prices and Payment', [`Prices for the Products are Supplier's distributor prices in Exhibit A, which are at least 32% below Supplier's published list prices. Supplier may change prices once in each contract year on ninety days' notice. Distributor may set its own resale prices.`, 'Supplier will invoice each shipment when it ships. Distributor will pay each invoice within forty-five days after the invoice date, in United States dollars, by electronic transfer. Supplier may suspend shipments while any undisputed invoice is more than thirty days overdue.']],
    ['Delivery and Risk', ['Products are delivered FCA Supplier\'s warehouse in Copper Flats, Arizona (Incoterms 2020). Title and risk of loss pass to Distributor on delivery to Distributor\'s carrier. Distributor will inspect each shipment and report shortages or visible damage within ten business days after receipt.']],
    ['Marketing and Sales Efforts', ['Distributor will use its best efforts to promote and sell the Products in the Territory, maintain a sales force of at least four representatives trained on the Products, exhibit the Products at the regional science education and clinical laboratory trade shows listed in Exhibit B, and maintain a demonstration inventory of at least one unit of each current microscope model.', 'Supplier will provide Distributor with product literature, training for Distributor\'s sales and service personnel twice a year at no charge, and a cooperative advertising allowance of three percent of Distributor\'s net purchases, payable as a credit against future orders for approved advertising.']],
    ['Service and Warranty', ['Supplier warrants to Distributor and to each end customer that each Product will be free from defects in materials and workmanship for three years from delivery to the end customer, or five years for optical components. Distributor will perform first-line warranty service using parts Supplier provides free of charge, and Supplier will reimburse Distributor\'s labor at $85.00 per hour under the warranty claim procedure in Supplier\'s service manual.', 'THE WARRANTY IN THIS SECTION IS EXCLUSIVE AND IN PLACE OF ALL OTHER WARRANTIES, EXPRESS OR IMPLIED, INCLUDING ANY IMPLIED WARRANTY OF MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.']],
    ['Recalls', ['If Supplier or any government authority determines that a Product must be recalled or corrected, Distributor will cooperate fully, including by providing customer contact information for the affected units within five business days. Supplier will bear the reasonable costs of any recall caused by a defect in the Product.']],
    ['Trademarks', [`Supplier grants Distributor a non-exclusive license during the Term to use Supplier's trademarks in the Territory solely to advertise and sell the Products, in accordance with Supplier's brand guidelines. Distributor will not register any of Supplier's trademarks or any confusingly similar mark or domain name, and all goodwill arising from Distributor's use of the trademarks belongs to Supplier.`]],
    ['Reports', ['Within twenty days after the end of each calendar quarter, Distributor will provide a report of its sales of Products by customer type and state, its inventory of Products on hand, and any complaints received about the Products, in the format Supplier reasonably specifies.']],
    ['Term and Termination', ['This Agreement begins on the Effective Date and continues for an initial term of three years, and then renews for successive one-year terms unless either party gives notice of non-renewal at least one hundred twenty days before the end of the then-current term.', 'Either party may terminate this Agreement if the other materially breaches it and fails to cure the breach within forty-five days after written notice, or immediately if the other becomes insolvent. On termination Supplier will repurchase Distributor\'s saleable inventory of current Products at the price Distributor paid, less a restocking charge of ten percent.']],
  ];
}

export function scanAgreement() {
  const id = 'scan-agreement-10p';
  const rng = Rng.from(id);
  const supplier = 'Basalt Ridge Optics Inc.';
  const distributor = 'Saltash Point Scientific Supply LLC';
  const effective = '2025-11-10';
  const signedSupplier = '2025-11-03';
  const signedDistributor = '2025-11-05';
  const flow = typedFlow({ fontSize: 12, leading: 1.55, margins: { top: 72, bottom: 84, left: 90, right: 90 }, keep: [supplier, distributor], footer: (page, { number, total }) => page.textCenter(306, 755, `Page ${number} of ${total}`, { face: 'serif', size: 10 }) });
  flow.heading('EXCLUSIVE DISTRIBUTION AGREEMENT', { level: 1, align: 'center', size: 15, face: 'sans-bold' });
  flow.paragraph(`This Exclusive Distribution Agreement (this "Agreement") is effective as of ${longDate(effective)} (the "Effective Date") and is made between ${supplier}, an Arizona corporation ("Supplier"), and ${distributor}, a California limited liability company ("Distributor").`);
  flow.paragraph('Supplier designs and manufactures optical instruments for education, clinical, and industrial laboratories. Distributor sells laboratory equipment to schools, colleges, hospitals, and research laboratories in the western United States. The parties agree as follows:');
  flow.paragraph([{ text: '1. Definitions. ', face: 'sans-bold' }, { text: '"Customers" means schools, colleges, universities, hospitals, clinics, and research and industrial laboratories located in the Territory. "Territory" means the States listed in Exhibit B. "Contract year" means each twelve-month period beginning on the Effective Date or an anniversary of it. Other capitalized terms are defined where they first appear.' }]);
  let number = 2;
  for (const [title, paragraphs] of distributionSections({})) {
    if (paragraphs.length === 1) flow.paragraph([{ text: `${number}. ${title}. `, face: 'sans-bold' }, { text: paragraphs[0] }]);
    else {
      flow.paragraph(`${number}. ${title}.`, { face: 'sans-bold', after: 2, keepWithNext: 30 });
      paragraphs.forEach((text, index) => flow.paragraph([{ text: `${number}.${index + 1} `, face: 'sans-bold' }, { text }]));
    }
    number += 1;
  }
  const general = ['Confidentiality', 'Limitation of Liability', 'Insurance', 'Force Majeure', 'Assignment', 'Notices', 'Governing Law and Disputes', 'Independent Contractors', 'General'];
  const ctx = { rng: rng.fork('clauses'), a: 'Supplier', b: 'Distributor', aName: supplier, bName: distributor, aAddress: '1200 Basalt Drive, Copper Flats, AZ 85219', bAddress: '415 Ferry Landing Road, Saltash Point, CA 94923', aNotice: 'Vice President, Sales', bNotice: 'President', state: 'Arizona', venue: 'Maricopa County, Arizona' };
  for (const [title, body] of COMMERCIAL.filter(([title]) => general.includes(title))) {
    const paragraphs = body(ctx).map((text) => text.replace(/Statement of Work|Deliverables?|services/g, (word) => ({ 'Statement of Work': 'order', Deliverable: 'Product', Deliverables: 'Products', services: 'obligations' }[word])));
    if (paragraphs.length === 1) flow.paragraph([{ text: `${number}. ${title}. `, face: 'sans-bold' }, { text: paragraphs[0] }]);
    else {
      flow.paragraph(`${number}. ${title}.`, { face: 'sans-bold', after: 2, keepWithNext: 30 });
      paragraphs.forEach((text, index) => flow.paragraph([{ text: `${number}.${index + 1} `, face: 'sans-bold' }, { text }]));
    }
    number += 1;
  }
  flow.paragraph('IN WITNESS WHEREOF, the parties have signed this Agreement on the dates below, to be effective as of the Effective Date.', { before: 6 });
  signatureBlocks(flow, [
    { heading: 'SUPPLIER', entity: supplier, name: 'Kasimir Fontaine', title: 'President', date: longDate(signedSupplier) },
    { heading: 'DISTRIBUTOR', entity: distributor, name: 'Delphine Corrigan', title: 'Managing Member', date: longDate(signedDistributor) },
  ], { rng: rng.fork('signatures'), stacked: true, size: 11.5 });
  flow.pageBreak();
  flow.heading('Exhibit A - Products and Distributor Prices', { level: 2, face: 'sans-bold' });
  const products = [['BR-200', 'Student compound microscope, LED, 40x-400x', 21500], ['BR-310', 'Laboratory compound microscope, 40x-1000x, mechanical stage', 64800], ['BR-420', 'Phase contrast clinical microscope', 289000], ['BR-510', 'Stereo zoom microscope, 7x-45x', 87500], ['BR-CAM5', '5-megapixel microscope camera with software', 32400], ['BR-OBJ100', '100x oil immersion objective, plan achromat', 19800], ['BR-LED2', 'Replacement LED illuminator module', 4200], ['BR-CASE', 'Fitted carrying case for BR-200 and BR-310', 6900]];
  for (const [code, description, price] of products) flow.paragraph(`${code} - ${description} - ${money(price)}`, { after: 3 });
  flow.pageBreak();
  flow.heading('Exhibit B - Territory and Reserved Accounts', { level: 2, face: 'sans-bold' });
  flow.paragraph('Territory: the States of California, Oregon, Washington, Nevada, Idaho, Utah, and Arizona.');
  flow.paragraph('Reserved national accounts: two national clinical laboratory networks and one university purchasing consortium, identified by account number in Supplier\'s confidential account list delivered to Distributor on the Effective Date.');
  flow.paragraph('Trade shows: the western regional science teachers\' convention, the state clinical laboratory association meetings in California and Washington, and the Pacific Northwest research laboratory expo.');
  flow.pageBreak();
  flow.heading('Exhibit C - Minimum Annual Purchases', { level: 2, face: 'sans-bold' });
  for (const [year, value] of [['First contract year', 92000000], ['Second contract year', 118000000], ['Third contract year', 141000000], ['Each renewal year', 'the prior year\'s minimum plus 5%']]) flow.paragraph(`${year}: ${typeof value === 'number' ? money(value) : value}`, { after: 3 });
  const pages = flow.finish();
  if (pages.length !== 10) throw new Error(`${id}: expected 10 pages, laid out ${pages.length}`);
  const images = scanImages(pages, { dpi: 300 });
  const bytes = scanPdf(images, { bits: 1 });
  const truth = truthFor(pages.map((page, index) => ({ number: index + 1, page })), {
    dates: [longDate(effective), longDate(signedSupplier), longDate(signedDistributor)],
    names: [supplier, distributor],
    identifiers: ['BR-310', 'BR-CAM5'],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Ten-page exclusive distribution agreement, scanned at 300 DPI black and white',
    kind: 'contract',
    textLayer: 'scan',
    pages: pages.length,
    categories: ['image_only_scan', 'pages_10', 'contract', 'competing_dates'],
    notes: `Ten clean 1-bit pages and no text layer, so all ten go through OCR. The effective date (${longDate(effective)}) is in the first sentence; the signature dates at the end of the body (${longDate(signedSupplier)} and ${longDate(signedDistributor)}) precede it and are traps. Exhibits list products, territory, and minimum purchases.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Exclusive Distribution Agreement',
      acceptableTypes: ['Distribution Agreement'],
      date: effective,
      role: 'effective',
      forbiddenDates: [[signedSupplier, 'supplier signature date'], [signedDistributor, 'distributor signature date']],
      parties: [supplier, distributor],
      relation: 'between',
      roles: [[supplier, 'seller'], [distributor, 'buyer']],
      forbiddenParties: [['Kasimir Fontaine', 'signatory'], ['Delphine Corrigan', 'signatory']],
      facts: [['microscopes', 'microscope'], ['Basalt Ridge Optics Inc.', 'Basalt Ridge'], ['western United States', 'California']],
      subjectTerms: ['distribution', 'microscopes', 'exclusive', 'territory'],
      readiness: 'either',
      dateText: [longDate(effective)],
    }),
  });
}

export function scanLease() {
  const id = 'scan-lease-25p';
  const rng = Rng.from(id);
  const landlord = 'Palisade Tower Partners LLC';
  const tenant = 'Fennel Court Software Inc.';
  const dated = '2026-04-22';
  const start = '2026-06-01';
  const end = '2033-05-31';
  const area = '14,860';
  const flow = typedFlow({ fontSize: 12, leading: 1.95, margins: { top: 72, bottom: 80, left: 96, right: 96 }, keep: [landlord, tenant], footer: (page, { number }) => page.textCenter(306, 750, `Palisade Tower - Office Lease - ${number}`, { face: 'serif', size: 10 }) });
  flow.heading('OFFICE LEASE', { level: 1, align: 'center', size: 16, face: 'sans-bold' });
  flow.paragraph('Palisade Tower, 2200 Vantage Drive, Saltash Point, California', { align: 'center' });
  flow.paragraph(`This Office Lease (this "Lease") is dated as of ${longDate(dated)} and is made between ${landlord}, a Delaware limited liability company ("Landlord"), and ${tenant}, a California corporation ("Tenant").`, { before: 8 });
  flow.heading('Basic Lease Information', { level: 2, face: 'sans-bold' });
  const basics = [
    ['Premises', `Suite 400, fourth floor, approximately ${area} rentable square feet`],
    ['Commencement Date', longDate(start)],
    ['Expiration Date', longDate(end)],
    ['Base Year', 'Calendar year 2026'],
    ['Tenant\'s Share', '6.42%'],
    ['Security Deposit', '$24,250.00'],
    ['Permitted Use', 'General office and software development'],
    ['Parking', 'Up to 42 unreserved spaces in the Building garage'],
    ['Landlord\'s address', `${landlord}, c/o Kingsfold Property Services, 2200 Vantage Drive, Suite 110, Saltash Point, CA 94923`],
    ['Tenant\'s address', `${tenant}, 2200 Vantage Drive, Suite 400, Saltash Point, CA 94923, Attention: Chief Financial Officer`],
  ];
  for (const [label, value] of basics) flow.paragraph([{ text: `${label}: `, face: 'sans-bold' }, { text: value }], { after: 3 });
  flow.paragraph('Base Rent schedule (monthly installments):', { face: 'sans-bold', before: 6, after: 3 });
  let rate = 4950;
  for (let year = 1; year <= 7; year += 1) {
    const monthly = Math.round((14860 * rate) / 12);
    flow.paragraph(`Lease Year ${year}: $${(rate / 100).toFixed(2)} per rentable square foot per year; monthly installment ${money(monthly)}`, { after: 2 });
    rate = Math.round(rate * 1.03);
  }
  const articles = officeLeaseArticles({ landlord, tenant, building: 'Palisade Tower', suite: 'Suite 400', area, share: '6.42%', start: longDate(start), end: longDate(end) });
  articles.forEach(([title, paragraphs], index) => {
    flow.paragraph(`ARTICLE ${index + 1} - ${title.toUpperCase()}`, { face: 'sans-bold', before: 6, after: 4, keepWithNext: 40 });
    paragraphs.forEach((text, part) => flow.paragraph(`${index + 1}.${part + 1}  ${text}`));
  });
  flow.paragraph('IN WITNESS WHEREOF, Landlord and Tenant have executed this Lease as of the date first written above.', { before: 8 });
  signatureBlocks(flow, [
    { heading: 'LANDLORD', entity: landlord, name: 'Saoirse Whitcombe', title: 'Authorized Signatory' },
    { heading: 'TENANT', entity: tenant, name: 'Lucan Fairbanks', title: 'Chief Executive Officer' },
  ], { rng: rng.fork('signatures'), stacked: true, size: 12 });
  flow.pageBreak();
  flow.heading('EXHIBIT A - DESCRIPTION OF THE PREMISES', { level: 2, face: 'sans-bold' });
  flow.paragraph(`The Premises consist of the entire rentable area of the fourth floor of Palisade Tower except the elevator lobby, stairwells, and the electrical and telecommunications rooms, containing approximately ${area} rentable square feet. The Premises include a reception area, forty-four workstations, nine private offices, four conference rooms, a server room of approximately 220 square feet with a dedicated cooling unit, a kitchenette, and storage. The floor plan initialed by the parties on the date of this Lease is incorporated by reference.`);
  flow.pageBreak();
  flow.heading('EXHIBIT B - BUILDING RULES AND REGULATIONS', { level: 2, face: 'sans-bold' });
  BUILDING_RULES.forEach((rule, index) => flow.paragraph(`${index + 1}. ${rule}`));
  flow.heading('EXHIBIT C - WORK LETTER', { level: 2, face: 'sans-bold' });
  for (const text of [
    'Landlord Work. Landlord, at its cost, will deliver the Premises with the base building systems in good working order, the restrooms on the fourth floor upgraded to current accessibility standards, and a new energy-efficient lighting system with occupancy sensors throughout the Premises.',
    'Tenant Improvements. Tenant may construct improvements to the Premises in accordance with plans approved by Landlord, which approval will not be unreasonably withheld. Landlord will provide a tenant improvement allowance of $60.00 per rentable square foot ($891,600.00), which may be applied to design, permits, construction, cabling, and up to $10.00 per rentable square foot of furniture and moving costs.',
    'Disbursement. Landlord will disburse the allowance monthly as work progresses, within thirty days after receiving an invoice, lien waivers for the work covered, and a certificate from Tenant\'s architect that the work has been completed as invoiced, holding back ten percent until final completion. Any part of the allowance not requested within twelve months after the Commencement Date is forfeited.',
    'Construction Management. Tenant\'s general contractor must be licensed in California and approved by Landlord. Landlord\'s construction management fee is two percent of the hard costs of the Tenant Improvements and may be paid from the allowance.',
  ]) flow.paragraph(text);
  flow.heading('EXHIBIT D - CLEANING SPECIFICATIONS', { level: 2, face: 'sans-bold' });
  for (const text of [
    'Nightly (Monday through Friday): empty wastebaskets and recycling bins; vacuum carpeted traffic areas; damp-mop hard floors; clean and restock restrooms; wipe kitchenette counters and sinks; remove fingerprints from entrance glass.',
    'Weekly: vacuum all carpeted areas including under desks where accessible; dust horizontal surfaces below seventy inches that are clear of papers; clean conference room tables and whiteboards on request.',
    'Monthly: dust high surfaces, vents, and light fixtures; clean interior glass partitions; spot-clean carpets.',
    'Annually: shampoo carpets in traffic areas; strip and refinish resilient floors; wash exterior windows inside and out (exterior twice a year).',
  ]) flow.paragraph(text);
  flow.heading('EXHIBIT E - FAIR MARKET RENT DETERMINATION', { level: 2, face: 'sans-bold' });
  flow.paragraph('If the parties cannot agree on the fair market rent for the extension term within thirty days after Tenant exercises its renewal option, each party will appoint a licensed commercial real estate broker with at least ten years\' experience leasing office space in the county, and the two brokers will determine the fair market rent within thirty days. If they cannot agree, they will appoint a third broker, who will select whichever of the first two brokers\' determinations is closer to the fair market rent. Each party pays its own broker and half the cost of the third.');
  for (const [title, paragraphs] of leaseExhibits({ landlord, tenant, building: 'Palisade Tower', suite: 'Suite 400', area, share: '6.42%', start: longDate(start), end: longDate(end) })) {
    flow.pageBreak();
    flow.heading(title, { level: 2, face: 'sans-bold' });
    for (const text of paragraphs) flow.paragraph(text);
  }
  const pages = flow.finish();
  if (pages.length !== 25) throw new Error(`${id}: expected 25 pages, laid out ${pages.length}`);
  const images = scanImages(pages, { dpi: 200 });
  const bytes = scanPdf(images, { bits: 1 });
  const truth = truthFor(pages.map((page, index) => ({ number: index + 1, page })), {
    dates: [longDate(dated), longDate(start), longDate(end)],
    names: [landlord, tenant],
    identifiers: ['Suite 400'],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Twenty-five-page office lease scanned at 200 DPI black and white',
    kind: 'contract',
    textLayer: 'scan',
    pages: pages.length,
    categories: ['image_only_scan', 'pages_25', 'contract', 'information_dense'],
    notes: `Twenty-five 1-bit pages at 200 DPI with no text layer: this document exists to measure how OCR time scales with page count. The lease is dated as of ${longDate(dated)} in its first sentence; the Commencement Date (${longDate(start)}) is accepted and the Expiration Date (${longDate(end)}) is a trap. Double-spaced like a typed lease: ${articles.length} articles, then eleven exhibits (building rules, work letter, cleaning specifications, operating expense exclusions, service hours, parking rules, and forms of commencement memorandum and estoppel certificate) - no page repeats another.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Office Lease',
      acceptableTypes: ['Lease', 'Lease Agreement'],
      date: dated,
      acceptableDates: [start],
      role: 'effective',
      forbiddenDates: [[end, 'lease expiration date']],
      parties: [landlord, tenant],
      relation: 'between',
      roles: [[landlord, 'landlord'], [tenant, 'tenant']],
      forbiddenParties: [['Saoirse Whitcombe', 'signatory'], ['Kingsfold Property Services', 'landlord\'s property manager']],
      facts: [['Suite 400', 'fourth floor'], ['Palisade Tower'], ['14,860 rentable square feet', '14,860']],
      subjectTerms: ['office lease', 'Suite 400', 'Palisade Tower', 'tenant improvement'],
      readiness: 'either',
      dateText: [longDate(dated)],
    }),
  });
}

/// What a poor OCR engine makes of clean text: l and 1 confused, O and 0
/// confused, "m" read as "rn" and back, commas dropped, and the digits of
/// dates mangled. Deterministic for a given generator.
export function corruptOcr(text, rng, { rate = 0.35 } = {}) {
  const word = (token) => {
    let out = token;
    if (/\d{1,2}\/\d{1,2}\/\d{4}/.test(out)) return out.replace(/0/g, 'O').replace(/1/g, 'l');
    if (rng.chance(rate)) out = out.replace(/l/g, '1');
    if (rng.chance(rate)) out = out.replace(/I/g, 'l');
    if (rng.chance(rate)) out = out.replace(/O/g, '0');
    if (rng.chance(rate)) out = out.replace(/0/g, 'O');
    if (rng.chance(rate)) out = out.replace(/m/g, 'rn');
    if (rng.chance(rate * 0.5)) out = out.replace(/rn/g, 'm');
    if (/\d,\d/.test(out) && rng.chance(0.7)) out = out.replace(/,/g, '');
    return out;
  };
  return text.split(' ').map(word).join(' ');
}

export function ocrCorruptedInvoice() {
  const id = 'ocr-corrupted-invoice';
  const rng = Rng.from(id);
  const issuer = 'Wexcombe Millwork Co.';
  const customer = 'Ostrander Homebuilders LLC';
  const invoiceDate = '2026-03-10';
  const dueDate = '2026-04-09';
  const number = 'WMC-11047';
  const items = [['Custom white oak stair treads, 42 in.', 14, 18650], ['Oak risers, primed', 15, 4875], ['Handrail, 16 ft, with brackets', 2, 31400], ['Interior door casing kit, colonial', 22, 6990], ['Delivery and installation, Lot 17 Harrow Ridge (hours)', 9, 8500]];
  const subtotal = items.reduce((sum, [, quantity, unit]) => sum + quantity * unit, 0);
  const tax = Math.round(subtotal * 0.06);
  const flow = typedFlow({ fontSize: 11, keep: [issuer, customer] });
  flow.paragraph(issuer, { face: 'sans-bold', size: 15, after: 0 });
  flow.paragraph('3310 Gristmill Lane, Linden Cross, PA 19047 - (215) 555-0148', { face: 'sans', size: 9.5, after: 12 });
  flow.heading('INVOICE', { level: 1, face: 'sans-bold', size: 18, before: 0 });
  flow.fields([['Invoice Number:', number], ['Invoice Date:', numericDate(invoiceDate)], ['Due Date:', numericDate(dueDate)], ['Bill To:', `${customer}, 1200 Calder Way, Linden Cross, PA 19046`], ['Job:', 'Lot 17, Harrow Ridge subdivision']], { labelWidth: 110, size: 11 });
  for (const [description, quantity, unit] of items) flow.paragraph(`${description} - ${quantity} @ ${money(unit)} = ${money(quantity * unit)}`, { after: 3 });
  flow.paragraph(`Subtotal ${money(subtotal)}   Sales tax 6% ${money(tax)}`, { before: 6, after: 2 });
  flow.paragraph(`TOTAL DUE ${money(subtotal + tax)}`, { face: 'sans-bold', size: 12.5 });
  flow.paragraph('Terms: net 30 days. Make checks payable to Wexcombe Millwork Co. Questions: billing@wexcombe-millwork.example', { size: 10 });
  const [visible] = flow.finish();
  // The page as a scanning service delivered it: the scanned image with an
  // invisible OCR text layer placed over each line, full of recognition
  // errors. A reader that trusts the text layer reads the errors.
  const [image] = scanImages([visible], { dpi: 300 });
  const delivered = new Page();
  delivered.image(0, 0, 612, 792, pdfImage(threshold(image), 1));
  const corrupt = rng.fork('ocr');
  // The bold letterhead name is always misread the same way; elsewhere the
  // errors fall where the generator's dice put them.
  const misreadIssuer = issuer.replace('mb', 'rnb').replace('ll', '11');
  for (const item of visible.items) {
    if (item.type !== 'text') continue;
    const text = item.text === issuer ? misreadIssuer : corruptOcr(item.text, corrupt);
    delivered.text(item.x, item.y, text, { face: item.face, size: item.size, render: 3 });
  }
  const bytes = buildPdf([delivered]);
  const layer = pageText(delivered);
  const clean = visualText(visible);
  const corruptedDate = layer.match(/[Il]nvoice Date: (\S+)/)[1];
  if (!layer.startsWith(`${misreadIssuer}\n`)) throw new Error(`${id}: the letterhead is not the first line of the text layer`);
  if (layer.includes(numericDate(invoiceDate))) throw new Error(`${id}: the invoice date survived corruption`);
  if (!layer.includes(` to ${issuer}`)) throw new Error(`${id}: the notes say the payment line keeps the issuer's name`);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: [layer],
    cleanText: clean,
    title: 'Scanned invoice with an invisible OCR text layer full of recognition errors',
    kind: 'invoice',
    textLayer: 'ocr_corrupted',
    pages: 1,
    categories: ['ocr_corrupted', 'invoice'],
    notes: `The page image is a clean scan of an invoice dated ${numericDate(invoiceDate)} from ${issuer}, but it carries an invisible (render mode 3) text layer from a poor OCR pass: l/1 and O/0 swapped, rn for m, commas dropped, and every date's zeros read as letters (the invoice date reads "${corruptedDate}", the letterhead "${misreadIssuer}"). The worker trusts a text layer of this length, so the engine sees only the corrupted text: the date cannot be read with confidence, and the right outcome is review. Gold facts come from the image (the issuer's name survives intact only in the payment line); evidence strings are as the text layer states them; clean_text is the text actually printed.`,
    gold: gold({
      type: 'Invoice',
      date: invoiceDate,
      role: 'invoice',
      forbiddenDates: [[dueDate, 'payment due date']],
      parties: [issuer],
      relation: 'from',
      roles: [[issuer, 'issuer'], [customer, 'customer']],
      forbiddenParties: [[customer, 'bill-to customer']],
      facts: [[money(subtotal + tax), amount(subtotal + tax), money(subtotal + tax).replace(',', '')], ['stair treads', 'oak']],
      subjectTerms: ['millwork', 'stair treads', 'invoice'],
      readiness: 'needs_review',
      dateText: [corruptedDate],
      partyText: { [issuer]: [misreadIssuer] },
    }),
  });
}

export function scanFaxTwoFrames() {
  const id = 'scan-fax-two-frames';
  const rng = Rng.from(id);
  const sender = 'Kestrel Ridge Aero Services';
  const recipient = 'Highmeadow Air Charter LLC';
  const sent = '2026-08-12';
  const validUntil = '2026-09-11';
  const quoteNumber = 'KRA-Q-2611';
  const header = (frame) => `AUG 12 2026 09:14   FROM: KESTREL RIDGE AERO   TO: 5550133   P.${frame}/2`;
  const cover = typedFlow({ fontSize: 12, keep: [sender, recipient] });
  cover.page.text(40, 30, header(1), { face: 'sans', size: 8 });
  cover.heading('FAX', { level: 1, face: 'sans-bold', size: 28, before: 0 });
  cover.fields([['To:', `${recipient}, Attn: Director of Maintenance`], ['Fax:', '(801) 555-0133'], ['From:', `${sender} - Parts and Quotes`], ['Date:', longDate(sent)], ['Pages:', '2 (including this cover)'], ['Re:', 'Quote for propeller overhaul - N0612X']], { labelWidth: 64, size: 12 });
  cover.paragraph('Please find our quotation on the next page. Call us if any page is missing or unreadable. The quote is also available on request by email.', { before: 10 });
  cover.paragraph('CONFIDENTIALITY NOTICE: This facsimile is intended only for the addressee and may contain confidential information. If you received it in error, please notify the sender and destroy it.', { size: 10, before: 10 });
  const [coverPage] = cover.finish();
  const quote = typedFlow({ fontSize: 11.5, keep: [sender, recipient] });
  quote.page.text(40, 30, header(2), { face: 'sans', size: 8 });
  quote.paragraph(sender, { face: 'sans-bold', size: 14, after: 0 });
  quote.paragraph('Hangar 6, Kestrel Ridge Municipal Airport, UT 84047 - Repair Station KRAS-417R', { face: 'sans', size: 9.5, after: 12 });
  quote.heading(`QUOTATION No. ${quoteNumber}`, { level: 2, face: 'sans-bold' });
  quote.fields([['Date:', longDate(sent)], ['Customer:', recipient], ['Aircraft:', 'N0612X, Halden Aero Works HA-180'], ['Valid until:', longDate(validUntil)]], { labelWidth: 80, size: 11.5 });
  for (const [description, value] of [['Propeller overhaul, two-blade fixed pitch, per manufacturer manual', 412000], ['Blade straightening and re-pitch, if required', 68000], ['Paint and balance', 54000], ['Remove and reinstall, including spinner inspection (labor)', 96000], ['Freight both ways', 54000]]) quote.paragraph(`${description}: ${money(value)}`, { after: 3 });
  quote.paragraph(`Total quoted price: ${money(684000)}. Turnaround is twelve business days from receipt of the propeller. A 50% deposit is due with the work order; the balance is due on return to service.`, { before: 6 });
  quote.paragraph('Accepted by: ______________________   Date: ____________', { before: 10 });
  const [quotePage] = quote.finish();
  // A fine-mode fax: 200 DPI, thresholded, with line noise from the phone line.
  const frames = scanImages([coverPage, quotePage], { dpi: 200, degrade: (image, index) => speckle(threshold(image, 140), rng.fork(`noise-${index}`), 300, { darkShare: 0.8, maxSize: 1 }) });
  const bytes = tiff(frames, { dpi: 200 });
  const truth = truthFor([{ number: 1, page: coverPage }, { number: 2, page: quotePage }], { dates: [longDate(sent), longDate(validUntil)], names: [sender, recipient], identifiers: [quoteNumber, 'N0612X'] });
  return result({
    id,
    extension: 'tiff',
    files: [{ name: `${id}.tiff`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Two-frame fax TIFF: a cover sheet and a repair quotation',
    kind: 'quotation',
    textLayer: 'scan',
    pages: 2,
    categories: ['tiff', 'image_only_scan', 'competing_dates'],
    notes: `A two-frame fax. The worker reads only the first frame - the cover sheet - and reports the second as unread (TEXT_TRUNCATED). The document being sent is the quotation on frame 2 (No. ${quoteNumber}, dated ${longDate(sent)}, valid until ${longDate(validUntil)}), which the reader never sees; the cover sheet's date happens to match. Truncated input must go to review whatever the model proposes. Because only the cover sheet is visible, filing it as a fax or fax cover sheet is accepted. ocr_truth covers both frames; only frame 1 can be scored for OCR.`,
    ocrTruth: truth,
    gold: gold({
      type: 'Quotation',
      acceptableTypes: ['Quote', 'Price Quotation', 'Fax', 'Fax Cover Sheet'],
      date: sent,
      role: 'issuance',
      forbiddenDates: [[validUntil, 'quote expiry']],
      parties: [sender],
      relation: 'from',
      acceptablePartySets: [{ parties: [recipient], relation: 'to' }],
      roles: [[sender, 'issuer'], [sender, 'seller'], [recipient, 'recipient'], [recipient, 'customer']],
      forbiddenParties: [],
      facts: [['propeller'], [recipient, 'Highmeadow Air Charter'], ['N0612X']],
      subjectTerms: ['propeller overhaul', 'quotation'],
      readiness: 'needs_review',
      dateText: [longDate(sent)],
    }),
  });
}

export function scanPatientIntakeForm() {
  const id = 'scan-patient-intake-form';
  const rng = Rng.from(id);
  const clinic = 'Calder Way Family Medicine';
  const patient = 'Ione Kowalczyk';
  const signed = '2026-08-27';
  const birth = '1990-11-23';
  const coverage = '2026-01-01';
  const memberId = 'MHP-60218843';
  const page = new Page();
  page.text(48, 56, clinic, { face: 'sans-bold', size: 15 });
  page.text(48, 70, '1400 Calder Way, Suite 3, Copper Flats, AZ 85219 - (480) 555-0126', { face: 'sans', size: 8.5 });
  page.textRight(564, 56, 'NEW PATIENT INTAKE FORM', { face: 'sans-bold', size: 12 });
  page.textRight(564, 70, 'Please print clearly', { face: 'sans', size: 8.5 });
  let y = 88;
  const bar = (title) => {
    page.rect(48, y, 516, 14, { fill: 0.85, stroke: null });
    page.text(52, y + 10, title, { face: 'sans-bold', size: 9 });
    y += 14;
  };
  const box = (x, w, label, value) => {
    page.rect(x, y, w, 30, { fill: null, stroke: 0.3, width: 0.5 });
    page.text(x + 3, y + 9, label, { face: 'sans', size: 7 });
    if (value) page.text(x + 6, y + 24, value, { face: 'serif', size: 11 });
  };
  bar('PATIENT INFORMATION');
  box(48, 300, 'Full legal name', patient); box(348, 108, 'Date of birth', numericDate(birth)); box(456, 108, 'Sex', 'F'); y += 30;
  box(48, 516, 'Home address', '52 Umber Street, Copper Flats, AZ 85219'); y += 30;
  box(48, 172, 'Mobile phone', '(480) 555-0187'); box(220, 220, 'Email', 'ione.k@mailbox.example'); box(440, 124, 'Preferred language', 'English'); y += 34;
  bar('EMERGENCY CONTACT');
  box(48, 240, 'Name', 'Rufus Kowalczyk'); box(288, 120, 'Relationship', 'Spouse'); box(408, 156, 'Phone', '(480) 555-0188'); y += 34;
  bar('INSURANCE');
  box(48, 240, 'Insurance plan', 'Meadowlark Health Plan'); box(288, 140, 'Member ID', memberId); box(428, 136, 'Group number', '77120'); y += 30;
  box(48, 240, 'Subscriber', 'Self'); box(288, 140, 'Coverage effective', numericDate(coverage)); box(428, 136, 'Copay', '$30.00'); y += 34;
  bar('VISIT');
  box(48, 300, 'Requested physician', 'Dr. Saoirse Brightwater'); box(348, 216, 'Reason for visit', 'Establish care; annual physical'); y += 30;
  box(48, 516, 'Current medications', 'Levothyroxine 50 mcg daily; vitamin D 2000 IU'); y += 30;
  box(48, 258, 'Allergies', 'Penicillin (rash)'); box(306, 258, 'Last physical exam', 'June 2024'); y += 34;
  bar('MEDICAL HISTORY (mark all that apply)');
  y += 4;
  [['Asthma', false], ['Diabetes', false], ['High blood pressure', false], ['Thyroid disease', true], ['Heart disease', false], ['Depression or anxiety', true]].forEach(([label, checked], index) => {
    const x = 52 + (index % 3) * 172;
    const top = y + Math.floor(index / 3) * 16;
    page.rect(x, top, 9, 9, { fill: null, stroke: 0.2, width: 0.5 });
    if (checked) page.text(x + 1.5, top + 8, 'X', { face: 'sans-bold', size: 9 });
    page.text(x + 14, top + 8, label, { face: 'sans', size: 9.5 });
  });
  y += 40;
  bar('CONSENT AND SIGNATURE');
  page.textBlock(52, y + 12, 508, 'I consent to examination and treatment, authorize the clinic to bill my insurance plan, and assign insurance benefits to the clinic. I have received the Notice of Privacy Practices.', { face: 'serif', size: 9.5 });
  y += 40;
  box(48, 300, 'Patient signature', ''); box(348, 216, 'Date', numericDate(signed));
  page.stroke(signatureStroke(rng.fork('signature'), 70, y + 24, 140), { width: 1.1 });
  const [image] = scanImages([page], { dpi: 300 });
  const bytes = png(threshold(image), { bits: 1, dpi: 300, description: 'InternBench scan' });
  const truth = truthFor([{ number: 1, page }], { dates: [numericDate(signed), numericDate(birth), numericDate(coverage)], names: [patient, clinic, 'Rufus Kowalczyk'], identifiers: [memberId, '77120'] });
  return result({
    id,
    extension: 'png',
    files: [{ name: `${id}.png`, bytes }],
    text: [truth.pages[0].text],
    title: 'New patient intake form, filled in and scanned at 300 DPI',
    kind: 'form',
    textLayer: 'scan',
    pages: 1,
    categories: ['form', 'healthcare', 'image_only_scan', 'png', 'competing_dates'],
    notes: `A filled clinic form scanned as an image: values in boxes under small captions, two and three boxes to a row, so OCR order across a row is not reading order. The form is dated by the signature date box (${numericDate(signed)}); the date of birth (${numericDate(birth)}) and insurance coverage date (${numericDate(coverage)}) are traps. The patient is the party; the emergency contact and requested physician are not.`,
    ocrTruth: truth,
    gold: gold({
      type: 'New Patient Intake Form',
      acceptableTypes: ['Patient Intake Form', 'Patient Registration Form'],
      date: signed,
      role: 'execution',
      forbiddenDates: [[birth, 'patient date of birth'], [coverage, 'insurance coverage effective date']],
      parties: [patient],
      relation: 'for',
      acceptablePartySets: [{ parties: [patient], relation: 'from' }, { parties: [clinic], relation: 'with' }],
      roles: [[patient, 'subject'], [patient, 'patient'], [patient, 'issuer'], [clinic, 'provider'], [clinic, 'counterparty']],
      forbiddenParties: [['Rufus Kowalczyk', 'emergency contact'], ['Dr. Saoirse Brightwater', 'requested physician'], ['Meadowlark Health Plan', 'insurance plan']],
      facts: [[patient, 'Kowalczyk'], [clinic, 'Calder Way'], ['Meadowlark Health Plan', 'Meadowlark']],
      subjectTerms: ['intake', 'new patient', 'insurance', 'medical history'],
      readiness: 'either',
      dateText: [numericDate(signed)],
    }),
  });
}
