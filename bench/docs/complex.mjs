/// Complex PDFs: an insurance declarations page whose two columns are
/// written row by row into the content stream, so a reader that follows the
/// stream interleaves them; and an eight-page annual report excerpt with a
/// cover, running headers and footers, footnotes, tables, and sidebars.
import { Flow, Page } from '../lib/layout.mjs';
import { gold } from '../lib/gold.mjs';
import { amount, decimal, grouped, longDate, money, numericDate, percent } from '../lib/format.mjs';
import { digitalPdf, result } from './common.mjs';

export function declarationsInterleaved() {
  const id = 'declarations-interleaved';
  const insurer = 'Northfell Mutual Insurance Company';
  const insured = 'Wexcombe Bicycle Cooperative';
  const agency = 'Kingsfold Insurance Agency, Inc.';
  const producer = 'Selby Marchetti';
  const start = '2026-07-01';
  const end = '2027-07-01';
  const issued = '2026-06-18';
  const policy = 'CPP-4471-208815';
  const premiums = [['Commercial Property', 481200], ['Commercial General Liability', 320500], ['Commercial Inland Marine', 64400], ['Commercial Crime', 31800], ['Terrorism (TRIA)', 9700]];
  const total = premiums.reduce((sum, [, value]) => sum + value, 0);
  const page = new Page();
  page.rect(54, 40, 504, 50, { fill: 0.9, stroke: null });
  page.textCenter(306, 60, insurer.toUpperCase(), { face: 'sans-bold', size: 13 });
  page.textCenter(306, 78, 'COMMERCIAL PACKAGE POLICY DECLARATIONS', { face: 'sans-bold', size: 11 });
  page.text(54, 104, 'Home Office: 600 Northfell Road, Dunmore Lake, ID 83814. A mutual insurance company.', { size: 7.5, grey: 0.3 });
  const left = [
    ['POLICY NUMBER:', policy],
    ['POLICY PERIOD:', `From ${numericDate(start)} To ${numericDate(end)}`],
    ['', '12:01 A.M. standard time at the mailing address'],
    ['NAMED INSURED:', ''],
    ['', insured],
    ['', '2214 Kingsfold Avenue'],
    ['', 'Westharrow, MI 49103'],
    ['FORM OF BUSINESS:', 'Cooperative corporation'],
    ['BUSINESS DESCRIPTION:', ''],
    ['', 'Retail bicycle sales, repair, and rental'],
    ['PRIOR POLICY:', 'CPP-4471-197322'],
    ['AUDIT PERIOD:', 'Annual'],
  ];
  const right = [
    ['AGENT / PRODUCER:', ''],
    ['', agency],
    ['', `Producer: ${producer}`],
    ['', 'Agent code 00-4417'],
    ['', '41 Thornbury Place, Westharrow, MI 49101'],
    ['DATE ISSUED:', numericDate(issued)],
    ['COVERAGE PARTS', 'PREMIUM'],
    ...premiums.map(([part, value]) => [part, `$${amount(value)}`]),
  ];
  const rows = Math.max(left.length, right.length);
  // Row-interleaved on purpose: left line, then right line, top to bottom.
  for (let row = 0; row < rows; row += 1) {
    const y = 132 + row * 15;
    const [leftLabel, leftValue] = left[row] ?? ['', ''];
    const [rightLabel, rightValue] = right[row] ?? ['', ''];
    if (leftLabel) page.text(54, y, leftLabel, { face: 'sans-bold', size: 8.5 });
    if (leftValue) page.text(150, y, leftValue, { face: 'sans', size: 9 });
    if (rightLabel) page.text(330, y, rightLabel, { face: rightValue === 'PREMIUM' || !rightValue ? 'sans-bold' : 'sans', size: rightValue === 'PREMIUM' || !rightValue ? 8.5 : 9 });
    if (rightValue) page.textRight(558, y, rightValue, { face: rightValue === 'PREMIUM' ? 'sans-bold' : 'sans', size: rightValue === 'PREMIUM' ? 8.5 : 9 });
  }
  let y = 132 + rows * 15 + 4;
  page.line(330, y - 10, 558, y - 10, { width: 0.6 });
  page.text(330, y, 'TOTAL ADVANCE PREMIUM', { face: 'sans-bold', size: 9 });
  page.textRight(558, y, money(total), { face: 'sans-bold', size: 9 });
  y += 22;
  page.line(54, y, 558, y, { width: 0.8 });
  y += 16;
  page.text(54, y, 'LOCATION SCHEDULE', { face: 'sans-bold', size: 9 });
  const locations = [
    ['1', '2214 Kingsfold Avenue, Westharrow, MI 49103', 'Retail store and repair shop', '$1,840,000 building / $612,000 contents'],
    ['2', '88 Starling Way, Westharrow, MI 49101', 'Rental fleet storage', 'Contents only: $275,000'],
  ];
  for (const [number, address, use, limits] of locations) {
    y += 13;
    page.text(54, y, number, { size: 8.5 });
    page.text(70, y, address, { size: 8.5 });
    page.text(300, y, use, { size: 8.5 });
    page.text(430, y, limits, { size: 8 });
  }
  y += 22;
  page.text(54, y, 'FORMS AND ENDORSEMENTS APPLICABLE TO ALL COVERAGE PARTS', { face: 'sans-bold', size: 9 });
  for (const form of ['NM-IL 00 17 Common Policy Conditions', 'NM-IL 00 21 Nuclear Energy Liability Exclusion Endorsement', 'NM-CP 10 30 Causes of Loss - Special Form (Northfell edition 01/2024)', 'NM-GL 20 26 Additional Insured - Designated Person or Organization', 'NM-IL 09 52 Cap on Losses from Certified Acts of Terrorism']) {
    y += 12;
    page.text(66, y, form, { size: 8.5 });
  }
  y += 24;
  y = page.textBlock(54, y, 504, 'These declarations, together with the common policy conditions, coverage part declarations, coverage forms, and endorsements, complete the above-numbered policy.', { size: 8 });
  y += 20;
  page.text(330, y, '/s/ Ione Abernathy', { face: 'serif', size: 10 });
  page.line(330, y + 4, 558, y + 4, { width: 0.5 });
  page.text(330, y + 15, 'Authorized Representative', { size: 8 });
  page.text(54, 760, `NM-DEC 01 25   Insured copy   Page 1 of 2`, { size: 7, grey: 0.4 });

  const flow = new Flow({ face: 'sans', fontSize: 9, margins: { top: 60, bottom: 60, left: 54, right: 54 } });
  flow.heading('COMMERCIAL GENERAL LIABILITY COVERAGE PART DECLARATIONS', { level: 2, size: 11 });
  flow.paragraph(`Policy number ${policy}. This coverage part is effective on the same dates as the policy period shown on the common declarations.`, { size: 8.5 });
  flow.table([{ header: 'Limits of insurance', width: 0.7 }, { header: 'Limit', width: 0.3, align: 'right' }], [
    ['Each occurrence', '$1,000,000'], ['Damage to premises rented to you (any one premises)', '$300,000'], ['Medical expense (any one person)', '$10,000'],
    ['Personal and advertising injury', '$1,000,000'], ['General aggregate (other than products-completed operations)', '$2,000,000'], ['Products-completed operations aggregate', '$2,000,000'],
  ], { size: 8.5 });
  flow.table([{ header: 'Classification', width: 0.4 }, { header: 'Code', width: 0.12 }, { header: 'Premium basis', width: 0.2 }, { header: 'Rate', width: 0.12, align: 'right' }, { header: 'Premium', width: 0.16, align: 'right' }], [
    ['Bicycle stores - sales and repair', '10140', 'Gross sales $3,420,000', '0.614', '$2,100'],
    ['Bicycle rental', '10150', 'Gross receipts $286,000', '3.180', '$909'],
    ['Classes - instruction (guided rides)', '41675', 'Each participant 1,480', '0.132', '$196'],
  ], { size: 8.5 });
  flow.paragraph('Additional insureds: Westharrow Downtown Development Authority (as respects the annual Riverfront Ride event only).', { size: 8.5 });
  const second = flow.finish()[0];
  second.text(54, 760, 'NM-DEC 01 25   Insured copy   Page 2 of 2', { size: 7, grey: 0.4 });
  const pages = [page, second];
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Insurance policy declarations page with row-interleaved columns',
    kind: 'insurance',
    textLayer: 'native',
    pages: pages.length,
    categories: ['multi_column', 'complex_pdf', 'competing_dates', 'layout_parties'],
    notes: `The declarations page is two columns drawn row by row, so the text layer puts each left line beside the right line at the same height: the named insured shares a line with the agency, and the policy period shares one with the producer. The policy period starts ${numericDate(start)} (the gold, as the policy's effective date) and ends ${numericDate(end)}; the issue date (${numericDate(issued)}) is accepted as the date the declarations were issued. The insured is the party; the insurer is accepted as the issuer; the agency and producer are traps.`,
    gold: gold({
      type: 'Commercial Package Policy Declarations',
      acceptableTypes: ['Policy Declarations', 'Insurance Policy Declarations', 'Declarations Page'],
      date: start,
      acceptableDates: [issued],
      role: 'effective',
      forbiddenDates: [[end, 'policy expiration']],
      parties: [insured],
      relation: 'for',
      acceptablePartySets: [{ parties: [insurer], relation: 'from' }, { parties: [insurer, insured], relation: 'between' }],
      roles: [[insured, 'subject'], [insured, 'customer'], [insurer, 'issuer'], [insurer, 'provider']],
      forbiddenParties: [[agency, 'insurance agency (producer)'], [producer, 'individual producer'], ['Westharrow Downtown Development Authority', 'additional insured on one event']],
      facts: [[policy], [money(total), amount(total), '9,076'], ['bicycle', 'Bicycle']],
      subjectTerms: ['commercial package', 'property', 'general liability', 'policy'],
      readiness: 'ready',
      dateText: [numericDate(start)],
    }),
  });
}

export function annualReportExcerpt() {
  const id = 'annual-report-excerpt-8p';
  const company = 'Rookwood Precision Metals Corporation';
  const ceo = 'Casimir Ellingsen';
  const chair = 'Wilhelmina Strickland';
  const letterDate = '2026-03-12';
  const yearEnd = '2025-12-31';
  const meeting = '2026-05-21';
  const record = '2026-03-27';
  const segments = [
    { name: 'Aerospace Components', sales: [41230, 46810, 52975], margin: [1420, 1510, 1630], employees: 1180, plants: 'Kestrel Ridge, UT and Silverlode, NV', products: 'turbine blade blanks, landing-gear forgings, and titanium fasteners', customers: 31, backlog: 61400 },
    { name: 'Medical Implants', sales: [18440, 20115, 23090], margin: [2210, 2290, 2405], employees: 640, plants: 'Bellmoor, WI', products: 'cobalt-chrome knee femoral components, titanium spinal cages, and trauma plates', customers: 12, backlog: 9200 },
    { name: 'Industrial Tooling', sales: [27610, 25980, 26340], margin: [870, 640, 790], employees: 910, plants: 'Wrenfield, OH and Linden Cross, PA', products: 'carbide inserts, die sets, and hot-work tool steel bars', customers: 2400, backlog: 4100 },
  ];
  const years = ['2023', '2024', '2025'];
  const totalSales = years.map((_, index) => segments.reduce((sum, segment) => sum + segment.sales[index], 0));
  const flow = new Flow({
    face: 'serif', fontSize: 10, margins: { top: 72, bottom: 72 },
    header: (page, { number }) => {
      if (number === 1) return;
      page.text(72, 46, `${company} - 2025 Annual Report`, { face: 'sans', size: 8, grey: 0.35 });
      page.line(72, 52, 540, 52, { width: 0.4, grey: 0.5 });
    },
    footer: (page, { number }) => {
      if (number === 1) return;
      page.textCenter(306, 754, String(number), { face: 'sans', size: 8.5, grey: 0.35 });
    },
    keep: [company],
  });
  // Cover.
  const cover = flow.page;
  cover.rect(0, 0, 612, 792, { fill: 0.93, stroke: null });
  cover.rect(72, 210, 468, 4, { fill: 0.2, stroke: null });
  cover.text(72, 190, company, { face: 'sans-bold', size: 20 });
  cover.text(72, 262, '2025 Annual Report', { face: 'sans-bold', size: 34 });
  cover.text(72, 292, 'Forged for the long run', { face: 'serif', size: 16, grey: 0.25 });
  cover.text(72, 640, 'Excerpt: Letter to Shareholders, Financial Highlights, Segment Review, Shareholder Information', { face: 'sans', size: 9.5, grey: 0.25 });
  cover.text(72, 654, 'The complete report, including audited financial statements, is available from Investor Relations.', { face: 'sans', size: 9.5, grey: 0.25 });
  flow.pageBreak();
  flow.heading('Letter to Shareholders', { level: 1, size: 18 });
  flow.paragraph('Dear Fellow Shareholders,');
  flow.paragraph(`2025 was the strongest year in Rookwood's ninety-one-year history. Net sales rose ${percent(Math.round(((totalSales[2] - totalSales[1]) * 10000) / totalSales[1]), 1)} to $${decimal(Math.round(totalSales[2] / 100), 1)} million, adjusted operating margin expanded for the third consecutive year, and we ended the year with the largest order backlog the company has ever carried. Those results belong to the 2,730 people in our plants, laboratories, and offices, and I want to begin by thanking them.`);
  flow.box((box) => {
    box.paragraph('2025 at a glance', { face: 'sans-bold', size: 10, after: 4 });
    for (const line of [
      `Net sales: $${decimal(Math.round(totalSales[2] / 100), 1)} million`,
      'Adjusted operating margin: 15.0%',
      'Free cash flow: $11.8 million',
      'Backlog at year end: $74.7 million',
      'Recordable injury rate: 0.91 (2024: 1.37)',
      'Dividends paid: $1.12 per share',
    ]) box.paragraph(line, { face: 'sans', size: 9, after: 1 });
  }, { estimate: 120 });
  flow.paragraph('Aerospace Components carried the year. Narrow-body build rates recovered, our new isothermal forging press in Kestrel Ridge reached full output in June, and we qualified on two new engine programs. Medical Implants grew faster than its markets as our largest orthopedic customer moved spinal cage production to us from a competitor that exited the business. Industrial Tooling was the exception: demand from automotive die shops stayed soft, and we responded by closing the oldest of our three heat-treat lines and consolidating carbide pressing in Wrenfield.');
  flow.paragraph('We invested $18.4 million in capital projects during the year, nearly half of it in additive manufacturing and in-process inspection equipment that shortens qualification of new parts. We also completed the acquisition of Ashdown Surface Technologies, a small coatings business whose thermal-spray capability our aerospace customers had been asking us to bring in-house.');
  flow.paragraph('Our priorities for 2026 are unchanged: safety first; disciplined growth in aerospace and medical; a smaller, more profitable tooling business; and returning cash to shareholders. The Board has approved a 7% increase in the quarterly dividend and a new $25 million share repurchase authorization.');
  flow.paragraph(`Thank you for your confidence in Rookwood. I look forward to seeing many of you at our Annual Meeting on ${longDate(meeting)}.`);
  flow.paragraph('Sincerely,', { after: 14 });
  flow.paragraph(ceo, { face: 'sans-bold', after: 0 });
  flow.paragraph('President and Chief Executive Officer', { after: 0 });
  flow.paragraph(longDate(letterDate), { after: 6 });
  flow.pageBreak();
  flow.heading('Financial Highlights', { level: 1, size: 18 });
  flow.paragraph(`(in thousands of dollars, except per-share data and employees; fiscal years ended December 31)`, { size: 9, face: 'sans', grey: 0.3 });
  flow.table([{ header: '', width: 0.46 }, ...years.map((year) => ({ header: year, width: 0.18, align: 'right' }))], [
    ['Net sales', ...totalSales.map((value) => amount(value * 100).slice(0, -3))],
    ['Gross profit', ...['21,904', '23,872', '26,519']],
    ['Operating income', ...['10,388', '11,442', '13,251']],
    ['Adjusted operating income (1)', ...['11,020', '12,105', '15,322']],
    ['Net income', ...['7,416', '8,207', '9,684']],
    ['Diluted earnings per share', ...['$2.31', '$2.57', '$3.06']],
    ['Dividends declared per share', ...['$1.00', '$1.04', '$1.12']],
    ['Capital expenditures', ...['9,870', '14,215', '18,402']],
    ['Free cash flow (2)', ...['8,904', '7,310', '11,796']],
    ['Total debt', ...['42,500', '47,900', '39,600']],
    ['Employees at year end', ...['2,655', '2,688', '2,730']],
  ], { size: 9, border: 'rules' });
  flow.footnote('(1)', 'Adjusted operating income excludes restructuring charges of $632 thousand in 2023, $663 thousand in 2024, and $2,071 thousand in 2025, principally the closure of the Linden Cross heat-treat line.');
  flow.footnote('(2)', 'Free cash flow is cash provided by operating activities less capital expenditures. It is not a measure defined by generally accepted accounting principles and may not be comparable to similarly titled measures of other companies.');
  flow.paragraph(`Return on invested capital improved to 13.8% from 12.1%, and net debt fell to 1.2 times adjusted EBITDA at ${longDate(yearEnd)}, the lowest leverage since the 2019 acquisition of our Bellmoor implant plant.`);
  flow.paragraph('Sales by end market, 2025: commercial aerospace 41%; defense 9%; orthopedic and spine 23%; automotive tooling 15%; general industrial 12%. Sales outside the United States were 27% of the total, principally to engine makers and implant companies in Europe.');
  segments.forEach((segment, index) => {
    flow.pageBreak();
    flow.heading(`Segment Review: ${segment.name}`, { level: 1, size: 16 });
    flow.paragraph(`${segment.name} makes ${segment.products} at plants in ${segment.plants}. The segment employs ${grouped(segment.employees)} people and served ${grouped(segment.customers)} customers in 2025.`);
    flow.table([{ header: '(in thousands of dollars)', width: 0.46 }, ...years.map((year) => ({ header: year, width: 0.18, align: 'right' }))], [
      ['Segment net sales', ...segment.sales.map((value) => amount(value * 100).slice(0, -3))],
      ['Segment operating margin', ...segment.margin.map((value) => percent(value, 1))],
      ['Year-end backlog', '', '', amount(segment.backlog * 100).slice(0, -3)],
    ], { size: 9, border: 'rules' });
    const commentary = [
      ['Build rates for narrow-body aircraft rose through the year, and our share of titanium fan-case forgings increased after we qualified the new isothermal press. Defense sales were flat as two programs moved between production lots.', 'We expect 2026 segment sales to grow at a high-single-digit rate, with margin held back in the first half by start-up costs on two new engine programs.', 'Customer concentration: our two largest engine customers accounted for 38% of segment sales.'],
      ['Volume growth came from spinal cages transferred from a competitor and from a new knee system launched by our largest customer. Pricing was stable under long-term supply agreements that run through 2028.', 'We are adding a second cleanroom passivation line in Bellmoor, which will be validated in the third quarter of 2026.', 'All three of the segment\'s FDA registered sites completed their 2025 inspections with no observations.'],
      ['Demand from automotive die shops weakened as model-year changeovers slipped, partly offset by stronger sales of hot-work tool steel to forging customers. We closed the Linden Cross heat-treat line in September and moved its work to Wrenfield.', 'The restructuring is expected to save about $1.9 million a year from 2026. We are evaluating options for the carbide insert product line, which earned below its cost of capital.', 'Distributor inventories of carbide inserts were reduced by about four weeks of supply during the year.'],
    ][index];
    flow.paragraph(commentary[0]);
    flow.box((box) => {
      box.paragraph(`${segment.name} - 2026 priorities`, { face: 'sans-bold', size: 9.5, after: 3 });
      box.paragraph(commentary[1], { face: 'sans', size: 9, after: 2 });
    }, { estimate: 80 });
    flow.paragraph(commentary[2]);
    flow.footnote('*', `Segment operating margin is segment operating income divided by segment net sales and excludes corporate costs, which were $3.1 million in 2025. Backlog is firm orders scheduled to ship within eighteen months.`);
  });
  flow.pageBreak();
  flow.heading('Board of Directors and Officers', { level: 1, size: 16 });
  flow.table([{ header: 'Director', width: 0.26 }, { header: 'Principal occupation', width: 0.44 }, { header: 'Committees', width: 0.18 }, { header: 'Since', width: 0.12, align: 'right' }], [
    [chair, 'Chair of the Board; retired Chief Operating Officer, an engine maker', 'Executive', '2014'],
    [ceo, 'President and Chief Executive Officer', 'Executive', '2019'],
    ['Adaeze Okonkwo-Hale', 'Professor of Materials Science, Kestrel Ridge Institute of Technology', 'Technology', '2021'],
    ['Florian Brandvold', 'Former Chief Financial Officer, a medical device company', 'Audit (chair)', '2017'],
    ['Imogen Vanterpool', 'Managing Director, a private equity firm', 'Compensation', '2020'],
    ['Jasper Kowalczyk', 'Retired partner, an accounting firm', 'Audit', '2016'],
    ['Renata Ellingsen-Moss', 'Chief Executive Officer, an industrial distributor', 'Compensation (chair), Governance', '2022'],
    ['Wendell Abernathy', 'Retired Major General, U.S. Air Force', 'Governance (chair)', '2018'],
  ], { size: 8.5, border: 'rules' });
  flow.heading('Corporate officers', { level: 3 });
  flow.table([{ header: 'Officer', width: 0.35 }, { header: 'Title', width: 0.65 }], [
    [ceo, 'President and Chief Executive Officer'],
    ['Tariq Holmqvist', 'Executive Vice President and Chief Financial Officer'],
    ['Signe Corrigan', 'President, Aerospace Components'],
    ['Mateo Rehnquist', 'President, Medical Implants'],
    ['Linnea Okafor', 'President, Industrial Tooling'],
    ['Quentin Barlowe', 'Senior Vice President, General Counsel and Secretary'],
    ['Esme Galbraith', 'Vice President, Environment, Health and Safety'],
  ], { size: 8.5, border: 'rules' });
  flow.paragraph('Board committees met 23 times in 2025. Every director attended at least 90% of the meetings of the Board and of the committees on which the director served. Seven of the eight directors are independent under the listing standards of the exchange on which our shares trade.', { size: 9.5 });
  flow.pageBreak();
  flow.heading('Shareholder Information', { level: 1, size: 16 });
  flow.fields([
    ['Annual Meeting:', `${longDate(meeting)}, 10:00 a.m. Mountain Time, Rookwood Technical Center, 900 Basalt Drive, Kestrel Ridge, UT 84047. Shareholders of record at the close of business on ${longDate(record)} are entitled to vote.`],
    ['Stock listing:', 'Common stock traded under the symbol RPMC.'],
    ['Transfer agent:', 'Cobble Hill Stock Transfer Company, PO Box 5520, Cobble Hill Station, VT 05401'],
    ['Independent auditors:', 'Wainwright Lindqvist LLP, Salt Lake City'],
    ['Investor relations:', 'Rhiannon Adeyemi, Vice President, Investor Relations - investors@rookwood-metals.example'],
    ['Board chair:', chair],
  ], { labelWidth: 130, size: 9.5 });
  flow.heading('Forward-looking statements', { level: 3 });
  flow.paragraph('This report contains forward-looking statements about our expected sales, margins, capital spending, and restructuring savings. They are based on current expectations and are subject to risks including changes in aircraft build rates, the timing of customer qualifications, raw material prices for titanium, cobalt, and nickel alloys, and labor availability. Actual results may differ materially. We undertake no obligation to update these statements.', { size: 9 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Eight-page excerpt of a metals manufacturer\'s annual report',
    kind: 'annual_report',
    textLayer: 'native',
    pages: pages.length,
    categories: ['complex_pdf', 'financial', 'table', 'competing_dates'],
    notes: `A designed report: a full-bleed cover with no date, running headers and page numbers, footnotes, sidebar boxes, and five-year tables. The report's own date is the date under the CEO's signature (${longDate(letterDate)}) at the end of the letter on page 3; the fiscal year end (${longDate(yearEnd)}) is accepted. The annual meeting (${longDate(meeting)}) and its record date (${longDate(record)}) are traps.`,
    gold: gold({
      type: 'Annual Report',
      acceptableTypes: ['2025 Annual Report', 'Annual Report Excerpt'],
      date: letterDate,
      acceptableDates: [yearEnd],
      role: 'issuance',
      forbiddenDates: [[meeting, 'annual meeting date'], [record, 'record date for the annual meeting']],
      parties: [company],
      relation: 'from',
      acceptablePartySets: [{ parties: [company], relation: 'for' }],
      roles: [[company, 'issuer'], [company, 'subject']],
      forbiddenParties: [[ceo, 'chief executive who signs the letter'], ['Ashdown Surface Technologies', 'acquired business mentioned in the letter'], ['Wainwright Lindqvist LLP', 'auditors listed in shareholder information']],
      facts: [['2025'], ['aerospace', 'Aerospace'], ['$11.8 million', '11,796', 'net sales', 'Net sales']],
      subjectTerms: ['annual report', 'aerospace', 'medical implants', 'tooling', 'segment'],
      readiness: 'ready',
      dateText: [longDate(letterDate)],
    }),
  });
}
