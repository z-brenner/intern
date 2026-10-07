/// Scans and mixed pages that decide hard facts: a freight claim whose
/// landscape damage schedule was scanned sideways inside the PDF; an
/// insurance cancellation notice scanned at 150 DPI; a remittance advice
/// scanned at 120 DPI and degraded; a lease renewal with a scanned exhibit
/// between two digital pages; a license amendment whose only signature date
/// is in a pasted scan of the signature block; and two scans dense with
/// dates, organisation names and identifiers - a certificate of liability
/// insurance at 300 DPI and a bill of lading at 200 DPI.
import { Flow, Page, pageText, signatureStroke, visualText } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { boxBlur, downsample, grain, pdfImage, rasterize, rotate, threshold } from '../lib/raster.mjs';
import { buildPdf } from '../lib/pdf.mjs';
import { amount, longDate, money, numericDate } from '../lib/format.mjs';
import { png } from '../lib/image.mjs';
import { textWidth } from '../lib/fonts.mjs';
import { result, signatureBlocks } from './common.mjs';
import { scanImages, scanPage, scanPdf, truthFor } from './scanning.mjs';

/// A ruled box with a small caption and lines of text under it, as forms
/// printed for filling in by typewriter look on a scan.
function captionBox(page, x, y, w, h, caption, lines = [], { size = 8.5, captionSize = 6.5, face = 'sans' } = {}) {
  page.rect(x, y, w, h, { fill: null, stroke: 0, width: 0.6 });
  page.text(x + 3, y + 8, caption, { face: 'sans-bold', size: captionSize });
  lines.forEach((line, index) => page.text(x + 5, y + 20 + index * (size + 2.5), line, { face, size }));
}

export function scanRotatedPageInPdf() {
  const id = 'scan-rotated-page-in-pdf';
  const claimant = 'Hollin Brook Furniture Gallery';
  const carrier = 'Redgate Interstate Freight Inc.';
  const shipper = 'Ashmore Upholstery Works';
  const claimed = '2026-04-27';
  const delivered = '2026-04-20';
  const shipped = '2026-04-14';
  const pro = '449-1182-07';
  const bol = 'AUW-26-0331';
  const items = [
    ['1', 'Three-seat sofa, oatmeal linen, SKU 4410-OAT', '1', 'Frame cracked, left arm', '1,960.00', '1,960.00'],
    ['2', 'Lounge chair, walnut frame, SKU 2207-WAL', '2', 'Leg snapped; fabric torn', '845.00', '1,690.00'],
    ['3', 'Ottoman, matching, SKU 2208-WAL', '1', 'Crushed corner', '395.00', '395.00'],
    ['4', 'Sectional, chaise piece only, SKU 5512-GRY', '1', 'Water stain across seat', '1,840.00', '1,840.00'],
    ['5', 'Dining chairs, set of 4, SKU 3301-ASH', '1', 'Two chairs missing', '957.00', '957.00'],
  ];
  const letter = new Flow({ face: 'serif', fontSize: 11, leading: 1.4, margins: { top: 72, bottom: 72, left: 80, right: 80 }, curly: true, keep: [claimant, carrier] });
  letter.paragraph(claimant, { face: 'sans-bold', size: 14, after: 0 });
  letter.paragraph('612 Larch Avenue, Hollin Brook, PA 18925 - (215) 555-0181', { face: 'sans', size: 9, after: 16 });
  letter.paragraph(longDate(claimed), { after: 12 });
  for (const line of ['Claims Department', carrier, '90 Terminal Way', 'Redgate, NJ 08077']) letter.paragraph(line, { after: 0 });
  letter.space(12);
  letter.heading('NOTICE OF FREIGHT CLAIM', { level: 2, face: 'sans-bold', size: 12 });
  const fields = [['Pro number', pro], ['Bill of lading', bol], ['Shipper', shipper], ['Delivery date', numericDate(delivered)], ['Amount claimed', '$6,842.00']];
  letter.fields(fields.map(([label, value]) => [`${label}:`, value]), { labelWidth: 120, size: 11, face: 'serif' });
  letter.paragraph(`The shipment shipped ${numericDate(shipped)} and was delivered to us on ${numericDate(delivered)} with visible damage, which our receiver noted on the delivery receipt. We claim the invoice value of the damaged and missing pieces listed on the attached schedule, $6,842.00 in all, under 49 U.S.C. 14706.`);
  letter.paragraph('The damaged pieces are held at our warehouse for your inspection for thirty days. Please acknowledge this claim in writing within thirty days.');
  letter.paragraph('Sincerely,', { after: 22 });
  letter.paragraph('Wilhelmina Strand, Operations Manager', { after: 0 });
  const [letterPage] = letter.finish();
  // The schedule is a landscape page, fed through a portrait scanner: on the
  // scan it lies on its side.
  const schedule = new Page({ width: 792, height: 612 });
  schedule.text(60, 60, 'SCHEDULE OF DAMAGED AND MISSING ITEMS', { face: 'sans-bold', size: 14 });
  schedule.text(60, 78, `Claim of ${claimant} - Pro ${pro} - delivered ${numericDate(delivered)}`, { face: 'sans', size: 10 });
  const columns = [['Item', 60], ['Description', 100], ['Pieces', 400], ['Damage', 450], ['Unit value', 610], ['Claimed', 690]];
  let y = 112;
  for (const [title, x] of columns) schedule.text(x, y, title, { face: 'sans-bold', size: 10 });
  schedule.line(60, y + 5, 740, y + 5, { width: 0.8 });
  for (const row of items) {
    y += 22;
    row.forEach((cell, index) => schedule.text(columns[index][1], y, cell, { face: 'sans', size: 10 }));
  }
  y += 10;
  schedule.line(60, y, 740, y, { width: 0.8 });
  schedule.text(450, y + 18, 'Total claimed', { face: 'sans-bold', size: 10 });
  schedule.text(690, y + 18, '6,842.00', { face: 'sans-bold', size: 10 });
  const [letterImage] = scanImages([letterPage], { dpi: 300 });
  const [scheduleImage] = scanImages([schedule], { dpi: 300, degrade: (image) => rotate(image, 90) });
  const bytes = buildPdf([scanPage(threshold(letterImage)), scanPage(threshold(scheduleImage))]);
  const truth = truthFor([{ number: 1, page: letterPage }, { number: 2, page: schedule }], {
    dates: [longDate(claimed), numericDate(delivered), numericDate(shipped)],
    names: [claimant, carrier, shipper],
    identifiers: [pro, bol, '4410-OAT', '5512-GRY'],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: truth.pages.map((page) => page.text),
    title: 'Freight claim scanned at 300 DPI; its landscape damage schedule lies on its side',
    kind: 'claim',
    textLayer: 'scan',
    pages: 2,
    categories: ['rotated_scan', 'image_only_scan', 'table', 'competing_dates', 'key_value'],
    notes: `Two 1-bit scans. Page 1 is an upright claim letter dated ${longDate(claimed)}; page 2, a landscape schedule of damaged items, went through the scanner sideways, so its image is turned 90 degrees inside an otherwise upright PDF. The ship date (${numericDate(shipped)}) and delivery date (${numericDate(delivered)}) are traps. From the consignee making the claim to the carrier; the shipper is not a party.`,
    ocrTruth: truth,
    structure: structure({
      tables: [[columns.map(([title]) => title), ...items]],
      keyValues: fields,
      routes: { 1: 'ocr', 2: 'ocr' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Notice of Freight Claim',
      acceptableTypes: ['Freight Claim', 'Freight Claim Letter'],
      date: claimed,
      role: 'notice',
      forbiddenDates: [[delivered, 'delivery date'], [shipped, 'ship date']],
      parties: [claimant],
      relation: 'from',
      acceptablePartySets: [{ parties: [carrier], relation: 'to' }],
      roles: [[claimant, 'issuer'], [claimant, 'sender'], [carrier, 'recipient'], [carrier, 'counterparty']],
      forbiddenParties: [[shipper, 'shipper of the goods'], ['Wilhelmina Strand', 'manager who signs']],
      facts: [['$6,842.00', '6,842'], [pro], [carrier, 'Redgate']],
      subjectTerms: ['freight claim', 'damage', 'shipment'],
      readiness: 'either',
      dateText: [longDate(claimed)],
    }),
  });
}

export function scanCancellationNotice150() {
  const id = 'scan-cancellation-notice-150dpi';
  const insurer = 'Kestrelmoor Casualty Company';
  const insured = 'Brannock Tool & Die Inc.';
  const producer = 'Ridley Cross Insurance Services';
  const mailed = '2026-08-12';
  const cancels = '2026-09-11';
  const periodStart = '2026-03-01';
  const periodEnd = '2027-03-01';
  const premiumDue = '2026-07-01';
  const policy = 'KC-WC-7710345';
  const flow = new Flow({ face: 'serif', fontSize: 10.5, leading: 1.4, margins: { top: 64, bottom: 64, left: 72, right: 72 }, curly: true, keep: [insurer, insured, producer] });
  flow.paragraph(insurer.toUpperCase(), { face: 'sans-bold', size: 13, after: 0 });
  flow.paragraph('Policy Services - 1500 Granite Mill Road, Stoneham Falls, MA 01880', { face: 'sans', size: 8.5, after: 14 });
  flow.heading('NOTICE OF CANCELLATION FOR NONPAYMENT OF PREMIUM', { level: 2, face: 'sans-bold', size: 11.5 });
  const fields = [['Date of notice', numericDate(mailed)], ['Policy number', policy], ['Policy period', `${numericDate(periodStart)} to ${numericDate(periodEnd)}`], ['Named insured', insured], ['Producer', `${producer}, code 4471`], ['Cancellation effective', `${numericDate(cancels)}, 12:01 a.m.`], ['Amount past due', '$4,318.00']];
  flow.fields(fields.map(([label, value]) => [`${label}:`, value]), { labelWidth: 150, size: 10.5, face: 'serif' });
  flow.paragraph(`The installment of premium due on ${numericDate(premiumDue)} for the workers compensation and employers liability policy shown above has not been paid. In accordance with the policy and the law of the Commonwealth, the policy will be cancelled as of the effective time shown above unless we receive the amount past due before that time.`);
  flow.paragraph('If payment is received before the cancellation takes effect, this notice is withdrawn and coverage continues without interruption. A payment received after that time will not reinstate the policy, and any return premium will be calculated pro rata and refunded to the named insured.');
  flow.paragraph('You may pay online at kestrelmoor.example/pay, by telephone at (978) 555-0136, or by mail to the address above. Please write the policy number on your check.');
  flow.paragraph(`Copy to: ${producer}`, { before: 8 });
  const [page] = flow.finish();
  const [image] = scanImages([page], { dpi: 150 });
  const bytes = scanPdf([image], { bits: 8 });
  const truth = truthFor([{ number: 1, page }], {
    dates: [numericDate(mailed), numericDate(cancels), numericDate(periodStart), numericDate(periodEnd), numericDate(premiumDue)],
    names: [insurer.toUpperCase(), insured, producer],
    identifiers: [policy, '$4,318.00'],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Insurance cancellation notice scanned at 150 DPI in grey',
    kind: 'notice',
    textLayer: 'scan',
    pages: 1,
    categories: ['low_resolution_scan', 'image_only_scan', 'notice', 'competing_dates', 'ocr_critical_fields', 'key_value'],
    notes: `A grey scan at 150 DPI: 10.5-point text is about 22 pixels to the em, enough to read but not to read every digit. Five dates compete: the notice is dated ${numericDate(mailed)}; the cancellation's effective date (${numericDate(cancels)}), the policy period (${numericDate(periodStart)} to ${numericDate(periodEnd)}) and the missed installment's due date (${numericDate(premiumDue)}) are traps. From the insurer to the insured; the producer receives a copy.`,
    ocrTruth: truth,
    structure: structure({ keyValues: fields, routes: { 1: 'ocr' } }),
    recording: 'pending',
    gold: gold({
      type: 'Notice of Cancellation',
      acceptableTypes: ['Cancellation Notice', 'Notice of Cancellation for Nonpayment of Premium'],
      date: mailed,
      role: 'notice',
      acceptableDates: [cancels],
      forbiddenDates: [[periodStart, 'policy period start'], [periodEnd, 'policy period end'], [premiumDue, 'unpaid installment due date']],
      parties: [insured],
      relation: 'for',
      acceptablePartySets: [{ parties: [insurer], relation: 'from' }, { parties: [insured], relation: 'to' }],
      roles: [[insured, 'subject'], [insured, 'recipient'], [insurer, 'issuer']],
      forbiddenParties: [[producer, 'insurance producer copied on the notice']],
      facts: [[policy], ['$4,318.00', '4,318'], ['workers compensation']],
      subjectTerms: ['cancellation', 'nonpayment', 'premium'],
      readiness: 'either',
      dateText: [numericDate(mailed)],
      partyText: { [insured]: [insured] },
    }),
  });
}

export function scanRemittanceAdvice120() {
  const id = 'scan-remittance-advice-120dpi';
  const rng = Rng.from(id);
  const payer = 'Tolliver Grain & Feed Cooperative';
  const payee = 'Sablewood Packaging Corp.';
  const bank = 'First Prairie Bank';
  const paid = '2026-07-17';
  const trace = '071000301884412';
  const invoices = [
    ['SPC-30417', '2026-05-28', 1848000, 36960], ['SPC-30552', '2026-06-04', 922500, 18450], ['SPC-30618', '2026-06-11', 487200, 9744],
    ['SPC-30701', '2026-06-18', 1260000, 25200], ['SPC-30779', '2026-06-25', 315600, 0], ['CM-1182', '2026-06-30', -41250, 0],
  ];
  const net = invoices.reduce((sum, [, , gross, discount]) => sum + gross - discount, 0);
  const page = new Page();
  page.text(48, 56, payer.toUpperCase(), { face: 'sans-bold', size: 13 });
  page.text(48, 70, 'Accounts Payable - 2 Elevator Road, Tolliver, KS 67880 - (620) 555-0105', { face: 'sans', size: 8.5 });
  page.textRight(564, 56, 'REMITTANCE ADVICE', { face: 'sans-bold', size: 13 });
  const fields = [['Payment date', numericDate(paid)], ['Payment no.', 'ACH-0071184'], ['Payee', payee], ['Vendor no.', 'V-20931'], ['Paid through', bank], ['Trace no.', trace]];
  fields.forEach(([label, value], index) => {
    const x = 48 + (index % 2) * 270;
    const y = 96 + Math.floor(index / 2) * 15;
    page.text(x, y, `${label}:`, { face: 'sans-bold', size: 9 });
    page.text(x + 82, y, value, { face: 'sans', size: 9 });
  });
  const columns = [['Invoice', 48], ['Invoice date', 140], ['Gross', 300], ['Discount', 390], ['Paid', 480]];
  let y = 160;
  page.line(48, y - 12, 564, y - 12, { width: 0.8 });
  for (const [title, x] of columns) page.text(x, y, title, { face: 'sans-bold', size: 9 });
  page.line(48, y + 5, 564, y + 5, { width: 0.6 });
  const rows = invoices.map(([number, date, gross, discount]) => [number, numericDate(date), amount(gross), amount(discount), amount(gross - discount)]);
  for (const row of rows) {
    y += 16;
    row.forEach((cell, index) => page.text(columns[index][1], y, cell, { face: 'sans', size: 9 }));
  }
  y += 10;
  page.line(48, y, 564, y, { width: 0.6 });
  page.text(390, y + 16, 'Net payment', { face: 'sans-bold', size: 9.5 });
  page.text(480, y + 16, money(net), { face: 'sans-bold', size: 9.5 });
  page.textBlock(48, y + 44, 516, `Funds were sent by ACH credit to the account on file ending 6620 and should be available on ${longDate(paid)}. Discounts were taken under 2% 10 net 30 terms; invoice SPC-30779 was paid after the discount period. Credit memo CM-1182 (returned pallets) was applied. Questions: ap@tolliver-coop.example.`, { size: 8.5 });
  // Scanned at 360 DPI and averaged down to 120, a little blurred and
  // grainy: an old flatbed on its lowest setting.
  const [full] = scanImages([page], { dpi: 360 });
  const image = grain(boxBlur(downsample(full, 3), 1), rng.fork('grain'), 10);
  const bytes = png(image, { bits: 8, dpi: 120, description: 'InternBench scan' });
  const truth = truthFor([{ number: 1, page }], {
    dates: [numericDate(paid), ...invoices.map(([, date]) => numericDate(date))],
    names: [payer.toUpperCase(), payee, bank],
    identifiers: [trace, 'ACH-0071184', 'V-20931', ...invoices.map(([number]) => number)],
  });
  return result({
    id,
    extension: 'png',
    files: [{ name: `${id}.png`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Remittance advice scanned at 120 DPI, blurred and grainy',
    kind: 'remittance_advice',
    textLayer: 'scan',
    pages: 1,
    categories: ['low_resolution_scan', 'noisy_scan', 'image_only_scan', 'png', 'ocr_critical_fields', 'table', 'competing_dates'],
    notes: `A dense page of identifiers - six invoice and credit memo numbers, a 15-digit ACH trace number, payment and vendor numbers - and seven dates, scanned at 120 DPI with blur and grain, so 9-point digits are about 15 pixels tall. The payment date (${numericDate(paid)}) is the document's; the invoice dates are traps. From the cooperative paying to its supplier; the bank only carries the payment.`,
    ocrTruth: truth,
    structure: structure({
      tables: [[columns.map(([title]) => title), ...rows]],
      keyValues: [...fields, ['Net payment', money(net)]],
      routes: { 1: 'ocr' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Remittance Advice',
      acceptableTypes: ['Payment Remittance Advice', 'Remittance'],
      date: paid,
      role: 'issuance',
      forbiddenDates: invoices.map(([number, date]) => [date, `date of ${number}`]),
      parties: [payer],
      relation: 'from',
      acceptablePartySets: [{ parties: [payee], relation: 'to' }],
      roles: [[payer, 'issuer'], [payer, 'payer'], [payee, 'recipient']],
      forbiddenParties: [[bank, 'bank that carried the payment']],
      facts: [[money(net), amount(net)], [trace], [payee, 'Sablewood']],
      subjectTerms: ['remittance', 'payment', 'invoice'],
      readiness: 'needs_review',
      dateText: [numericDate(paid)],
      partyText: { [payer]: [payer.toUpperCase()] },
    }),
  });
}

export function scanMixedMiddlePage() {
  const id = 'scan-mixed-middle-page';
  const rng = Rng.from(id);
  const landlord = 'Ashcombe Commercial Properties LLC';
  const tenant = 'Vellacott Optometry PC';
  const dated = '2026-03-18';
  const original = '2021-05-14';
  const renewalStart = '2026-06-01';
  const renewalEnd = '2031-05-31';
  const flow = new Flow({ face: 'serif', fontSize: 11, leading: 1.4, keep: [landlord, tenant],
    footer: (page, { number }) => page.textCenter(306, 760, `Lease Renewal Agreement - Page ${number} of 3`, { face: 'sans', size: 8, grey: 0.35 }) });
  flow.heading('LEASE RENEWAL AGREEMENT', { level: 1, align: 'center', size: 14, before: 0 });
  flow.paragraph(`This Lease Renewal Agreement is made as of ${longDate(dated)} between ${landlord}, an Ohio limited liability company, as landlord, and ${tenant}, an Ohio professional corporation, as tenant.`);
  flow.paragraph(`The landlord and the tenant are parties to a lease dated ${longDate(original)} for Suite 120 of the Ashcombe Professional Building, 2600 Wexford Pike, Granville Heights, Ohio, which expires on May 31, 2026. The tenant has exercised its option to renew, and the parties wish to record the terms of the renewal.`);
  flow.paragraph(`The term of the lease is renewed for five years, beginning on ${longDate(renewalStart)} and ending on ${longDate(renewalEnd)}. Base rent during the renewal term is payable monthly in advance in the amounts shown in Exhibit B, which the parties have initialed and which is attached to this agreement.`);
  flow.paragraph('The landlord will repaint the suite and replace the carpet in the waiting room before the renewal term begins, at its own cost. The tenant accepts the suite otherwise in its present condition.');
  flow.paragraph('The tenant has one further option to renew the lease for five years on the terms of the lease, at a base rent equal to the fair market rent at the start of that term, by giving notice no later than nine months before the renewal term ends.');
  flow.paragraph('Except as stated in this agreement, the lease remains in full force and effect, and this agreement and the lease are to be read as one document.');
  const [first] = flow.finish();
  // Exhibit B: printed, initialed in ink, scanned back in at 200 DPI grey.
  const exhibit = new Flow({ face: 'serif', fontSize: 11, leading: 1.4, curly: true,
    footer: (page) => page.textCenter(306, 760, 'Lease Renewal Agreement - Page 2 of 3', { face: 'sans', size: 8 }) });
  exhibit.heading('EXHIBIT B - RENT SCHEDULE', { level: 2, face: 'sans-bold', size: 12, before: 0 });
  exhibit.paragraph('Suite 120, 2,140 rentable square feet. Base rent excludes the tenant\'s share of operating expenses.', { size: 10.5 });
  const schedule = [['Lease year', 'Period', 'Annual rate per sq ft', 'Monthly base rent'], ['1', '06/2026 - 05/2027', '$22.50', '$4,012.50'], ['2', '06/2027 - 05/2028', '$23.18', '$4,133.77'], ['3', '06/2028 - 05/2029', '$23.87', '$4,256.82'], ['4', '06/2029 - 05/2030', '$24.59', '$4,385.22'], ['5', '06/2030 - 05/2031', '$25.33', '$4,517.18']];
  const xs = [80, 170, 330, 470];
  for (const row of schedule) {
    exhibit.ensure(22);
    row.forEach((cell, index) => exhibit.page.text(xs[index], exhibit.y + 11, cell, { face: row === schedule[0] ? 'sans-bold' : 'serif', size: 10.5 }));
    exhibit.y += 22;
  }
  exhibit.space(30);
  const initialsLine = 'Initials:   Landlord ________      Tenant ________';
  exhibit.paragraph(initialsLine, { size: 10.5 });
  const initials = exhibit.y - 10;
  const blank = (label) => exhibit.left + textWidth(initialsLine.slice(0, initialsLine.indexOf(label) + label.length + 1), 'serif', 10.5);
  const exhibitPage = exhibit.finish()[0];
  exhibitPage.stroke(signatureStroke(rng.fork('landlord'), blank('Landlord'), initials, 30), { width: 1 });
  exhibitPage.stroke(signatureStroke(rng.fork('tenant'), blank('Tenant'), initials, 30), { width: 1 });
  const [exhibitImage] = scanImages([exhibitPage], { dpi: 200 });
  const signatures = new Flow({ face: 'serif', fontSize: 11, leading: 1.4, keep: [landlord, tenant],
    footer: (page) => page.textCenter(306, 760, 'Lease Renewal Agreement - Page 3 of 3', { face: 'sans', size: 8, grey: 0.35 }) });
  signatures.paragraph('IN WITNESS WHEREOF, the parties have signed this Lease Renewal Agreement as of the date first written above.', { after: 16 });
  signatureBlocks(signatures, [
    { heading: 'LANDLORD', entity: landlord, name: 'Harriet Ashcombe-Lowe', title: 'Managing Member' },
    { heading: 'TENANT', entity: tenant, name: 'Dr. Felix Vellacott', title: 'President' },
  ], { stacked: true, size: 11 });
  const [last] = signatures.finish();
  const bytes = buildPdf([first, scanPage(exhibitImage, { bits: 8 }), last]);
  const truth = truthFor([{ number: 2, page: exhibitPage }], { dates: [], names: [], identifiers: ['$4,012.50', '$4,517.18', '2,140'] });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: [pageText(first), truth.pages[0].text, pageText(last)],
    title: 'Lease renewal: a scanned rent schedule between two digital pages',
    kind: 'contract',
    textLayer: 'mixed',
    pages: 3,
    categories: ['mixed_scan', 'contract', 'table', 'competing_dates', 'referenced_agreement'],
    notes: `Pages 1 and 3 have a text layer; page 2, the initialed rent schedule, was printed and scanned back in at 200 DPI in grey, so its figures exist only through OCR. The renewal is made as of ${longDate(dated)}; the original lease (${longDate(original)}) and the renewal term (${longDate(renewalStart)} to ${longDate(renewalEnd)}) are traps. A reader should read pages 1 and 3 as text and OCR only page 2.`,
    ocrTruth: truth,
    structure: structure({
      tables: [schedule],
      routes: { 1: 'fast', 2: 'ocr', 3: 'fast' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Lease Renewal Agreement',
      acceptableTypes: ['Lease Renewal', 'Lease Extension Agreement'],
      date: dated,
      role: 'effective',
      forbiddenDates: [[original, 'date of the lease being renewed'], [renewalStart, 'start of the renewal term'], [renewalEnd, 'end of the renewal term']],
      parties: [landlord, tenant],
      relation: 'between',
      roles: [[landlord, 'landlord'], [tenant, 'tenant']],
      forbiddenParties: [['Harriet Ashcombe-Lowe', 'signatory'], ['Dr. Felix Vellacott', 'signatory']],
      facts: [['Suite 120'], ['five years', '5 years'], ['$4,012.50', '4,012.50']],
      subjectTerms: ['lease renewal', 'rent', 'suite'],
      readiness: 'either',
      dateText: [longDate(dated)],
    }),
  });
}

export function mixedSignatureRegion() {
  const id = 'mixed-signature-region';
  const rng = Rng.from(id);
  const licensor = 'Corvane Analytics Inc.';
  const licensee = 'Pellworth County Water Authority';
  const original = '2024-09-30';
  const newEnd = '2028-09-30';
  const signedLicensor = '2026-08-18';
  const signedLicensee = '2026-08-21';
  const flow = new Flow({ face: 'serif', fontSize: 11, leading: 1.4, keep: [licensor, licensee],
    footer: (page, { number }) => page.textCenter(306, 760, `First Amendment - Page ${number} of 2`, { face: 'sans', size: 8, grey: 0.35 }) });
  flow.heading('FIRST AMENDMENT TO SOFTWARE LICENSE AGREEMENT', { level: 1, align: 'center', size: 13.5, before: 0 });
  flow.paragraph(`This First Amendment (this "Amendment") is between ${licensor}, a Delaware corporation ("Licensor"), and ${licensee}, a public water authority ("Licensee"), and amends the Software License and Services Agreement between them dated ${longDate(original)} (the "Agreement"). This Amendment is effective on the date of the last signature below (the "Amendment Effective Date").`);
  [
    ['Additional modules', 'Licensor grants Licensee a license to use the Leak Detection and Meter Analytics modules on the terms of the Agreement for up to 48,000 service connections. The license fee for the additional modules is $64,800 per year, invoiced annually in advance from the Amendment Effective Date.'],
    ['Term', `The term of the Agreement is extended to ${longDate(newEnd)}. Either party may end the Agreement at the end of the term by notice given at least ninety days before it ends.`],
    ['Data hosting', 'Licensor will host Licensee\'s data only in data centers in the United States, will keep daily backups for thirty days, and will return all of Licensee\'s data in a standard export format within thirty days after the Agreement ends.'],
    ['Service levels', 'The monthly availability commitment for the hosted service is raised from 99.5% to 99.9%. The service credit for each full 0.1% below the commitment is 2% of the monthly fee, up to 20%.'],
  ].forEach(([title, body], index) => flow.paragraph([{ text: `${index + 1}. ${title}. `, face: 'sans-bold' }, { text: body }]));
  flow.pageBreak();
  flow.paragraph([{ text: '5. Effect of Amendment. ', face: 'sans-bold' }, { text: 'Except as amended here, the Agreement remains in full force and effect. If this Amendment conflicts with the Agreement, this Amendment controls.' }]);
  flow.paragraph([{ text: '6. Counterparts. ', face: 'sans-bold' }, { text: 'This Amendment may be signed in counterparts and delivered electronically, and a scanned signature has the effect of an original.' }]);
  flow.paragraph('IN WITNESS WHEREOF, the parties have signed this Amendment on the dates written below.', { before: 6 });
  const top = flow.y + 14;
  const pages = flow.finish();
  // The signature block was printed, signed in ink, scanned and pasted onto
  // the page as one image: its names and dates exist only as pixels.
  const block = new Page({ width: 468, height: 268 });
  const signers = [
    [licensor.toUpperCase(), 'Rosalind Achterberg', 'Chief Revenue Officer', signedLicensor],
    [licensee.toUpperCase(), 'Barnaby Quist', 'General Manager', signedLicensee],
  ];
  signers.forEach(([entity, name, title, date], index) => {
    const y = 26 + index * 128;
    block.text(14, y, entity, { face: 'sans-bold', size: 11 });
    block.text(14, y + 26, 'By:', { face: 'serif', size: 11 });
    block.line(36, y + 28, 260, y + 28, { width: 0.6 });
    block.stroke(signatureStroke(rng.fork(`signature-${index}`), 50, y + 23, 130), { width: 1.2 });
    block.text(14, y + 46, `Name: ${name}`, { face: 'serif', size: 11 });
    block.text(14, y + 64, `Title: ${title}`, { face: 'serif', size: 11 });
    block.text(14, y + 82, `Date: ${longDate(date)}`, { face: 'serif', size: 11 });
  });
  const [blockImage] = scanImages([block], { dpi: 200, degrade: (image) => grain(image, rng.fork('grain'), 6) });
  const second = pages[1];
  second.image(72, top, 468, 268, pdfImage(blockImage, 8));
  const bytes = buildPdf(pages);
  // What page 2 shows, top to bottom: its own text, the pasted block's,
  // then the page footer.
  const composite = new Page();
  composite.items = [
    ...second.items.filter((item) => item.type === 'text'),
    ...block.items.filter((item) => item.type === 'text').map((item) => ({ ...item, x: item.x + 72, y: item.y + top })),
  ];
  const pageTwo = visualText(composite);
  const ocrTruth = {
    pages: [{ page: 2, text: pageTwo }],
    dates: [longDate(signedLicensor), longDate(signedLicensee)],
    names: [licensor.toUpperCase(), licensee.toUpperCase(), 'Rosalind Achterberg', 'Barnaby Quist'],
    identifiers: [],
  };
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: [pageText(pages[0]), pageTwo],
    title: 'License amendment whose signature block, and the date that makes it effective, is a pasted scan',
    kind: 'amendment',
    textLayer: 'mixed',
    pages: 2,
    categories: ['mixed_scan', 'image_region', 'amendment', 'contract', 'competing_dates', 'referenced_agreement'],
    notes: `Both pages are digital, but page 2's signature block is a scanned image pasted below the text (26% of the page, overlapping no text). The amendment takes effect on the date of the last signature, and the two signature dates - ${longDate(signedLicensor)} and ${longDate(signedLicensee)}, the later one the gold - are printed only in that image; the original agreement (${longDate(original)}) and the new end of term (${longDate(newEnd)}) are in the text and are traps. Page 2's ocr_truth is everything the page shows, top to bottom: its native text, the image's text, then the footer. This is the page the ocr_regions route exists for.`,
    ocrTruth,
    structure: structure({
      readingOrder: ['IN WITNESS WHEREOF', 'Rosalind Achterberg', longDate(signedLicensor), 'Barnaby Quist', longDate(signedLicensee)],
      routes: { 1: 'fast', 2: 'ocr_regions' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'First Amendment to Software License Agreement',
      acceptableTypes: ['Amendment to Software License Agreement', 'License Amendment'],
      date: signedLicensee,
      role: 'amendment',
      forbiddenDates: [[signedLicensor, 'earlier of the two signatures, not the last'], [original, 'date of the agreement being amended'], [newEnd, 'new end of term']],
      parties: [licensor, licensee],
      relation: 'between',
      roles: [[licensor, 'licensor'], [licensee, 'licensee']],
      forbiddenParties: [['Rosalind Achterberg', 'signatory'], ['Barnaby Quist', 'signatory']],
      facts: [['$64,800', '64,800'], [longDate(newEnd), '2028'], ['Leak Detection']],
      subjectTerms: ['amendment', 'software license', 'water'],
      readiness: 'either',
      dateText: [longDate(signedLicensee)],
    }),
  });
}

export function scanCertificateOfInsurance() {
  const id = 'scan-certificate-of-insurance';
  const producer = 'Hartsfield & Quail Insurance Brokers';
  const insured = 'Ostergaard Roofing Contractors LLC';
  const holder = 'Calloway Crossing Shopping Center LLC';
  const insurers = [['A', 'Bramblecote Casualty Company', '24117'], ['B', 'Wexley Indemnity Insurance Company', '39042'], ['C', 'Fenmoor Specialty Insurance Co.', '11583']];
  const issued = '2026-06-23';
  const coverages = [
    ['A', 'Commercial general liability', 'BCC-GL-4471902', '04/01/2026', '04/01/2027', 'Each occurrence $1,000,000'],
    ['B', 'Automobile liability', 'WIC-CA-208815', '01/15/2026', '01/15/2027', 'Combined single limit $1,000,000'],
    ['C', 'Umbrella liability', 'FSI-UMB-77310', '04/01/2026', '04/01/2027', 'Each occurrence $5,000,000'],
    ['A', 'Workers compensation', 'BCC-WC-5530118', '10/01/2025', '10/01/2026', 'E.L. each accident $500,000'],
  ];
  const page = new Page();
  page.text(40, 50, 'CERTIFICATE OF LIABILITY INSURANCE', { face: 'sans-bold', size: 14 });
  captionBox(page, 452, 36, 120, 26, 'DATE (MM/DD/YYYY)', [numericDate(issued)], { size: 10 });
  page.textBlock(40, 72, 532, 'This certificate is issued as a matter of information only and confers no rights upon the certificate holder. It does not amend, extend or alter the coverage afforded by the policies below.', { size: 7.5 });
  captionBox(page, 40, 92, 266, 64, 'PRODUCER', [producer, '300 Bell Tower Road, Suite 5', 'Calloway, SC 29630', 'Tel. (864) 555-0121']);
  captionBox(page, 306, 92, 266, 64, 'INSURERS AFFORDING COVERAGE', insurers.map(([letter, name, naic]) => `INSURER ${letter}: ${name} - NAIC ${naic}`), { size: 7.5 });
  captionBox(page, 40, 156, 266, 54, 'INSURED', [insured, '71 Slate Quarry Lane', 'Easley, SC 29640']);
  captionBox(page, 306, 156, 266, 54, 'CERTIFICATE NUMBER', ['HQ-26-06-4415', 'Revision 1']);
  let y = 224;
  page.text(40, y, 'COVERAGES', { face: 'sans-bold', size: 9 });
  y += 6;
  const columns = [['LTR', 40, 26], ['TYPE OF INSURANCE', 66, 140], ['POLICY NUMBER', 206, 104], ['POLICY EFF', 310, 62], ['POLICY EXP', 372, 62], ['LIMITS', 434, 138]];
  page.rect(40, y, 532, 16, { fill: 0.85, stroke: 0, width: 0.6 });
  for (const [title, x] of columns) page.text(x + 3, y + 11, title, { face: 'sans-bold', size: 7 });
  y += 16;
  for (const row of coverages) {
    page.rect(40, y, 532, 20, { fill: null, stroke: 0, width: 0.5 });
    row.forEach((cell, index) => page.text(columns[index][1] + 3, y + 13.5, cell, { face: 'sans', size: index === 5 ? 7.5 : 8 }));
    y += 20;
  }
  y += 10;
  captionBox(page, 40, y, 532, 52, 'DESCRIPTION OF OPERATIONS / LOCATIONS', ['Roof replacement, Building C, Calloway Crossing Shopping Center, 1800 Calloway Parkway, Calloway, SC.', 'The certificate holder is an additional insured on the general liability policy as required by written contract.']);
  y += 60;
  captionBox(page, 40, y, 266, 64, 'CERTIFICATE HOLDER', [holder, 'c/o Brightmoor Property Management', '1800 Calloway Parkway', 'Calloway, SC 29630']);
  captionBox(page, 306, y, 266, 64, 'CANCELLATION', ['Should any of the above policies be cancelled before', 'the expiration date, notice will be delivered in', 'accordance with the policy provisions.'], { size: 7.5 });
  y += 64;
  captionBox(page, 306, y, 266, 34, 'AUTHORIZED REPRESENTATIVE', ['/s/ Lorna Quail'], { face: 'serif', size: 10 });
  const [image] = scanImages([page], { dpi: 300 });
  const bytes = scanPdf([image], { bits: 1 });
  const truth = truthFor([{ number: 1, page }], {
    dates: [numericDate(issued), '04/01/2026', '04/01/2027', '01/15/2026', '01/15/2027', '10/01/2025', '10/01/2026'],
    names: [producer, insured, holder, ...insurers.map(([, name]) => name)],
    identifiers: [...coverages.map((row) => row[2]), ...insurers.map(([, , naic]) => naic), 'HQ-26-06-4415'],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Certificate of liability insurance scanned at 300 DPI: dense with dates, insurers and policy numbers',
    kind: 'certificate',
    textLayer: 'scan',
    pages: 1,
    categories: ['image_only_scan', 'ocr_critical_fields', 'table', 'form', 'competing_dates', 'irrelevant_names', 'key_value'],
    notes: `A boxed certificate form scanned clean at 300 DPI, holding seven dates, six organisations and eight identifiers. The certificate is dated in its top-right box (${numericDate(issued)}); every policy effective and expiration date in the coverage table is a trap. It certifies the insured's coverage: the insured is the party, the producer issues it, and it goes to the certificate holder; the three insurers are not parties to the certificate.`,
    ocrTruth: truth,
    structure: structure({
      tables: [[columns.map(([title]) => title), ...coverages]],
      keyValues: [['DATE (MM/DD/YYYY)', numericDate(issued)], ['PRODUCER', producer], ['INSURED', insured], ['CERTIFICATE HOLDER', holder], ['CERTIFICATE NUMBER', 'HQ-26-06-4415']],
      routes: { 1: 'ocr' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Certificate of Liability Insurance',
      acceptableTypes: ['Certificate of Insurance'],
      date: issued,
      role: 'issuance',
      forbiddenDates: [['2026-04-01', 'policy effective date'], ['2027-04-01', 'policy expiration date'], ['2026-01-15', 'auto policy effective date'], ['2027-01-15', 'auto policy expiration date'], ['2025-10-01', 'workers compensation effective date'], ['2026-10-01', 'workers compensation expiration date']],
      parties: [insured],
      relation: 'for',
      acceptablePartySets: [{ parties: [producer], relation: 'from' }, { parties: [holder], relation: 'to' }],
      roles: [[insured, 'subject'], [producer, 'issuer'], [holder, 'recipient']],
      forbiddenParties: insurers.map(([letter, name]) => [name, `insurer ${letter} affording coverage`]).concat([['Brightmoor Property Management', 'holder\'s property manager']]),
      facts: [['BCC-GL-4471902'], [holder, 'Calloway Crossing'], ['$5,000,000', '5,000,000']],
      subjectTerms: ['certificate', 'liability insurance', 'roofing'],
      readiness: 'either',
      dateText: [numericDate(issued)],
    }),
  });
}

export function scanBillOfLading() {
  const id = 'scan-bill-of-lading';
  const rng = Rng.from(id);
  const shipper = 'Corriveau Millwork Supply Co.';
  const consignee = 'Tamberlane Builders Supply';
  const carrier = 'Pinecrest Motor Freight Inc.';
  const billTo = 'Quillfeather Logistics Group';
  const shipped = '2026-09-08';
  const deliverBy = '2026-09-11';
  const poDate = '2026-08-29';
  const bol = 'BL-2026-118745';
  const page = new Page();
  page.text(40, 50, 'STRAIGHT BILL OF LADING - SHORT FORM', { face: 'sans-bold', size: 13 });
  page.text(40, 63, 'Original - not negotiable', { face: 'sans', size: 8 });
  const head = [['Bill of lading no.', bol], ['Ship date', numericDate(shipped)], ['Carrier', carrier], ['SCAC', 'PCMF'], ['Pro no.', '552-0917734'], ['Trailer no.', 'PC-53118'], ['Seal no.', 'SL-4407219'], ['Customer PO', `TBS-7731 dated ${numericDate(poDate)}`]];
  head.forEach(([label, value], index) => {
    const y = 82 + index * 14;
    page.text(330, y, `${label}:`, { face: 'sans-bold', size: 8.5 });
    page.text(416, y, value, { face: 'sans', size: 8.5 });
  });
  captionBox(page, 40, 74, 270, 58, 'SHIP FROM', [shipper, '4180 Sawmill Creek Road', 'Corriveau, VT 05473', 'SID: CMS-0019']);
  captionBox(page, 40, 132, 270, 58, 'SHIP TO', [consignee, '22 Depot Street, Dock 3', 'Tamberlane, NH 03276', `Deliver by ${numericDate(deliverBy)}`]);
  captionBox(page, 40, 190, 270, 46, 'THIRD PARTY FREIGHT CHARGES BILL TO', [billTo, '600 Wharf Road, Portsmouth, NH 03801']);
  page.text(330, 214, 'Freight terms: Third party', { face: 'sans', size: 8.5 });
  page.text(330, 228, 'Special instructions: call 2 hrs before arrival', { face: 'sans', size: 8.5 });
  let y = 252;
  const columns = [['Units', 40], ['Type', 80], ['Weight (lb)', 118], ['HM', 182], ['Description of articles', 206], ['NMFC no.', 440], ['Class', 510]];
  page.rect(40, y, 532, 15, { fill: 0.85, stroke: 0, width: 0.6 });
  for (const [title, x] of columns) page.text(x + 3, y + 10.5, title, { face: 'sans-bold', size: 7.5 });
  y += 15;
  const lines = [
    ['6', 'PLT', '3,840', '', 'Interior door slabs, primed pine, wrapped', '109300', '70'],
    ['4', 'PLT', '2,610', '', 'Window casing and trim, finger-jointed pine', '157600', '70'],
    ['2', 'CRT', '1,125', '', 'Stair parts: treads, risers, balusters', '109300', '70'],
    ['1', 'PLT', '480', '', 'Adhesive, construction grade, non-hazardous', '4610', '55'],
  ];
  for (const row of lines) {
    page.rect(40, y, 532, 18, { fill: null, stroke: 0, width: 0.5 });
    row.forEach((cell, index) => cell && page.text(columns[index][1] + 3, y + 12.5, cell, { face: 'sans', size: 8.5 }));
    y += 18;
  }
  page.text(43, y + 14, 'Total: 13 handling units, 8,055 lb', { face: 'sans-bold', size: 8.5 });
  y += 30;
  page.textBlock(40, y, 532, 'Received, subject to the classifications and tariffs in effect on the date of issue of this bill of lading, the property described above in apparent good order, except as noted, marked, consigned and destined as shown. The carrier agrees to carry it to its usual place of delivery at the destination.', { size: 7.5 });
  y += 40;
  captionBox(page, 40, y, 266, 52, 'SHIPPER SIGNATURE / DATE', ['/s/ Odile Corriveau', `Date: ${numericDate(shipped)}`], { face: 'serif', size: 9.5 });
  captionBox(page, 306, y, 266, 52, 'CARRIER SIGNATURE / PICKUP DATE', ['Driver: Ansel Pruett, unit 4417', `Date: ${numericDate(shipped)}   Time in 07:40   out 08:25`], { size: 8.5 });
  const [image] = scanImages([page], { dpi: 200, degrade: (image) => grain(image, rng.fork('grain'), 4) });
  const bytes = scanPdf([image], { bits: 1 });
  const truth = truthFor([{ number: 1, page }], {
    dates: [numericDate(shipped), numericDate(deliverBy), numericDate(poDate)],
    names: [shipper, consignee, carrier, billTo],
    identifiers: [bol, '552-0917734', 'SL-4407219', 'PC-53118', 'CMS-0019', '109300', '157600'],
  });
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text: truth.pages.map((entry) => entry.text),
    title: 'Straight bill of lading scanned at 200 DPI: four companies and seven identifiers',
    kind: 'bill_of_lading',
    textLayer: 'scan',
    pages: 1,
    categories: ['image_only_scan', 'ocr_critical_fields', 'table', 'form', 'competing_dates', 'layout_parties', 'key_value'],
    notes: `A 1-bit scan at 200 DPI of a short-form bill of lading: four organisations (shipper, consignee, carrier, third-party payer) in boxes and labelled lines, a commodity table and seven identifiers (BOL, pro, seal, trailer, shipper ID, NMFC numbers). Issued on the ship date (${numericDate(shipped)}); the deliver-by date (${numericDate(deliverBy)}) and the customer PO date (${numericDate(poDate)}) are traps. From the shipper; the consignee is accepted as the recipient, the carrier as the counterparty; the freight bill-to is a trap.`,
    ocrTruth: truth,
    structure: structure({
      tables: [[columns.map(([title]) => title), ...lines]],
      keyValues: [...head, ['SHIP FROM', shipper], ['SHIP TO', consignee]],
      routes: { 1: 'ocr' },
    }),
    recording: 'pending',
    gold: gold({
      type: 'Bill of Lading',
      acceptableTypes: ['Straight Bill of Lading'],
      date: shipped,
      role: 'issuance',
      forbiddenDates: [[deliverBy, 'deliver-by date'], [poDate, 'customer purchase order date']],
      parties: [shipper],
      relation: 'from',
      acceptablePartySets: [{ parties: [consignee], relation: 'to' }, { parties: [carrier], relation: 'with' }],
      roles: [[shipper, 'issuer'], [shipper, 'sender'], [consignee, 'recipient'], [carrier, 'counterparty']],
      forbiddenParties: [[billTo, 'third party paying the freight'], ['Ansel Pruett', 'driver'], ['Odile Corriveau', 'shipper\'s signer']],
      facts: [[bol], ['8,055 lb', '8,055'], [consignee, 'Tamberlane']],
      subjectTerms: ['bill of lading', 'millwork', 'freight'],
      readiness: 'either',
      dateText: [numericDate(shipped)],
    }),
  });
}
