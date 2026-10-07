/// Statements and health-plan correspondence: a business bank statement,
/// an explanation of benefits, and a prior authorization approval. Each
/// prints one date of issue among many dates of activity.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { addDays, amount, longDate, money, numericDate } from '../lib/format.mjs';
import { companies } from '../lib/names.mjs';
import { digitalPdf, letterhead, result } from './common.mjs';

export function accountStatement() {
  const id = 'account-statement';
  const rng = Rng.from(id);
  const bank = 'Kingsfold Community Bank';
  const holder = 'Pennywhistle Print Works LLC';
  const periodStart = '2026-03-01';
  const periodEnd = '2026-03-31';
  const statementDate = '2026-04-01';
  const customers = companies(rng.fork('customers'), 7, { exclude: [holder] });
  const vendors = ['Cinderpath Paper Supply', 'Basalt Drive Ink Co.', 'Wrenfield Electric Utility', 'Ollerton Equipment Leasing', 'Quarry Bend Freight', 'Harrow Lane Properties'];
  // The month's activity in integer cents: generated, put in date order,
  // then given a running balance.
  const opening = 4831275;
  const activity = rng.fork('activity');
  const entries = [];
  let day = 2;
  while (entries.length < 27) {
    const date = `2026-03-${String(day).padStart(2, '0')}`;
    const kind = activity.int(0, 9);
    if (kind < 4) entries.push([date, `ACH CREDIT ${activity.pick(customers).toUpperCase().slice(0, 30)} INV PAYMT`, activity.amount(48000, 1850000, 5), 0]);
    else if (kind < 6) entries.push([date, `ACH DEBIT ${activity.pick(vendors).toUpperCase()}`, 0, activity.amount(9000, 640000, 5)]);
    else if (kind < 7) entries.push([date, `CHECK ${activity.int(4100, 4199)}`, 0, activity.amount(12000, 380000, 5)]);
    else if (kind < 8) entries.push([date, `CARD PURCHASE ${activity.pick(['STARLING OFFICE SUPPLY', 'FENNEL COURT FUEL', 'UMBER STREET HARDWARE', 'CLOUDPRINT SOFTWARE SUB'])}`, 0, activity.amount(1800, 52000, 1)]);
    else if (kind < 9) entries.push([date, 'MOBILE DEPOSIT', activity.amount(25000, 410000, 5), 0]);
    else entries.push([date, `WIRE IN - ${activity.pick(customers).toUpperCase().slice(0, 26)}`, activity.amount(500000, 2400000, 100), 0]);
    day = Math.min(30, day + activity.int(0, 2));
  }
  entries.push(['2026-03-15', 'PAYROLL DEBIT SALTMARSH PAYROLL SVCS', 0, 2148830]);
  entries.push([periodEnd, 'MONTHLY SERVICE FEE', 0, 2500]);
  entries.push([periodEnd, 'INTEREST PAID', 412, 0]);
  const ordered = entries.map((entry, index) => ({ entry, index })).sort((a, b) => (a.entry[0] === b.entry[0] ? a.index - b.index : a.entry[0] < b.entry[0] ? -1 : 1)).map(({ entry }) => entry);
  let running = opening;
  const table = ordered.map(([date, description, credit, debit]) => {
    running += credit - debit;
    return [numericDate(date), description, credit ? amount(credit) : '', debit ? amount(debit) : '', amount(running)];
  });
  // Counterparties whose whole name survives in a transaction description
  // (the bank truncates long ones) are the realistic wrong answers.
  const named = (list) => list.filter((name) => ordered.some(([, description]) => description.includes(name.toUpperCase())));
  const payers = named(customers);
  const payees = named(vendors);
  if (payers.length < 2 || payees.length < 2) throw new Error(`${id}: too few counterparties named in full`);
  const credits = ordered.reduce((sum, entry) => sum + entry[2], 0);
  const debits = ordered.reduce((sum, entry) => sum + entry[3], 0);
  const largest = ordered.reduce((best, entry) => (entry[2] > best[2] ? entry : best));
  const flow = new Flow({ face: 'sans', fontSize: 9, margins: { top: 48, bottom: 56, left: 50, right: 50 }, footer: (page, { number, total }) => {
    page.text(50, 770, `${bank} - Member FDIC - Equal Housing Lender`, { size: 7, grey: 0.4 });
    page.textRight(562, 770, `Page ${number} of ${total}`, { size: 7, grey: 0.4 });
  }, keep: [bank, holder] });
  flow.y = letterhead(flow.page, { name: bank, lines: ['PO Box 7700, Kingsfold, IL 60431  -  Business Banking (630) 555-0190'], x: 50, width: 512, size: 15 });
  const page = flow.page;
  const top = flow.y;
  [holder, 'Accounts Payable', '220 Cinder Street', 'Ashworth, IL 60490'].forEach((line, index) => page.text(50, top + 10 + index * 12, line, { size: 10, face: index === 0 ? 'sans-bold' : 'sans' }));
  const meta = [['Statement Date', longDate(statementDate)], ['Statement Period', `${longDate(periodStart)} through ${longDate(periodEnd)}`], ['Account', 'Business Advantage Checking ...7731'], ['Customer Service', '(630) 555-0190']];
  meta.forEach(([label, value], index) => {
    page.text(300, top + 10 + index * 12, `${label}:`, { size: 8.5, grey: 0.35 });
    page.text(390, top + 10 + index * 12, value, { size: 8.5 });
  });
  flow.y = top + 62;
  flow.heading('Account Summary', { level: 3, size: 10.5 });
  flow.table([{ header: '', width: 0.6 }, { header: '', width: 0.4, align: 'right' }], [
    [`Beginning balance on ${longDate(periodStart)}`, money(opening)],
    [`Deposits and other credits (${table.filter((row) => row[2]).length})`, money(credits)],
    [`Withdrawals and other debits (${table.filter((row) => row[3]).length})`, money(debits)],
    [{ text: `Ending balance on ${longDate(periodEnd)}`, face: 'sans-bold' }, { text: money(opening + credits - debits), face: 'sans-bold' }],
  ], { header: false, border: 'rules', size: 9.5 });
  flow.heading('Transaction Detail', { level: 3, size: 10.5 });
  flow.table([
    { header: 'Date', width: 0.13 },
    { header: 'Description', width: 0.45 },
    { header: 'Credits', width: 0.14, align: 'right' },
    { header: 'Debits', width: 0.14, align: 'right' },
    { header: 'Balance', width: 0.14, align: 'right' },
  ], table, { size: 8.5, border: 'rules', zebra: 0.96 });
  flow.paragraph('Interest rate earned this period: 0.15% (annual percentage yield earned 0.15%). Average collected balance: ' + money(Math.round((opening + opening + credits - debits) / 2)) + '.', { size: 8.5 });
  flow.paragraph('In case of errors or questions about your electronic transfers, call us or write to us at the address above as soon as you can. We must hear from you no later than 60 days after we sent you the first statement on which the error appeared. Business accounts are governed by the Business Deposit Account Agreement.', { size: 7.5, grey: 0.25 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Monthly business checking account statement with 30 transactions',
    kind: 'statement',
    textLayer: 'native',
    pages: pages.length,
    categories: ['statement', 'table', 'competing_dates', 'financial'],
    notes: `The statement date (${longDate(statementDate)}) is one labelled line beside the period (${longDate(periodStart)} through ${longDate(periodEnd)}), which is repeated in the summary, and ${table.length} transaction dates fill the detail table. The period end is accepted - statements are often filed by the month they cover - but the period start and individual transaction dates are traps. The bank issued it; the account holder is also accepted as the party it is for. Counterparties in transaction descriptions are not parties.`,
    gold: gold({
      type: 'Account Statement',
      acceptableTypes: ['Bank Statement', 'Business Checking Statement', 'Statement'],
      date: statementDate,
      acceptableDates: [periodEnd],
      role: 'issuance',
      forbiddenDates: [[periodStart, 'start of the statement period'], [largest[0], 'date of the largest deposit'], ['2026-03-15', 'payroll debit date']],
      parties: [bank],
      relation: 'from',
      acceptablePartySets: [{ parties: [holder], relation: 'for' }],
      roles: [[bank, 'issuer'], [holder, 'subject'], [holder, 'customer']],
      forbiddenParties: [...payers.slice(0, 2).map((name) => [name, 'customer paying the account holder, named in a transaction']), ...payees.slice(0, 2).map((name) => [name, 'vendor paid from the account, named in a transaction'])],
      facts: [[money(opening + credits - debits), amount(opening + credits - debits)], ['March 1, 2026 through March 31, 2026', 'March 2026', '2026-03'], [holder, 'Pennywhistle']],
      subjectTerms: ['checking', 'statement', 'balance', 'March'],
      readiness: 'ready',
      dateText: [longDate(statementDate)],
    }),
  });
}

export function explanationOfBenefits() {
  const id = 'explanation-of-benefits';
  const plan = 'Meadowlark Health Plan';
  const member = 'Paloma Achterberg';
  const memberId = 'MHP-77310492';
  const statementDate = '2026-01-15';
  const claims = [
    ['C26-0041187', 'Hugo Lockridge, MD - Wrenfield Family Medicine', '2025-12-16', 'Office visit, established patient (99214)', 26500, 18240, 14240, 4000],
    ['C26-0041188', 'Bellmoor Diagnostic Laboratory', '2025-12-16', 'Comprehensive metabolic panel (80053)', 8900, 2115, 2115, 0],
    ['C26-0043920', 'Fallow Creek Physical Therapy', '2025-12-22', 'Therapeutic exercise, 3 units (97110)', 31500, 16830, 12830, 4000],
    ['C26-0043921', 'Fallow Creek Physical Therapy', '2025-12-29', 'Therapeutic exercise, 3 units (97110)', 31500, 16830, 12830, 4000],
    ['C26-0047310', 'Fallow Creek Physical Therapy', '2026-01-05', 'Therapeutic exercise, 3 units (97110)', 31500, 16830, 0, 16830],
    ['C26-0047702', 'Wrenfield Regional Imaging', '2026-01-07', 'X-ray, knee, 3 views (73562)', 21000, 9420, 0, 9420],
  ];
  const sum = (index) => claims.reduce((total, claim) => total + claim[index], 0);
  const flow = new Flow({ face: 'sans', fontSize: 9.5, margins: { top: 48, bottom: 56, left: 50, right: 50 }, keep: [plan, member] });
  flow.y = letterhead(flow.page, { name: plan, lines: ['Member Services 1-800-555-0177  -  PO Box 4410, Kestrel Ridge, UT 84047'], x: 50, width: 512, size: 15 });
  flow.heading('EXPLANATION OF BENEFITS', { level: 1, size: 14, after: 0 });
  flow.paragraph('THIS IS NOT A BILL', { face: 'sans-bold', size: 10, after: 8 });
  flow.table([{ header: 'Member', width: 0.25 }, { header: 'Member ID', width: 0.2 }, { header: 'Group', width: 0.3 }, { header: 'Statement Date', width: 0.25 }], [[member, memberId, 'Ashworth Unified Schools - PPO', longDate(statementDate)]], { size: 9 });
  flow.paragraph(`This statement shows how ${plan} processed claims received for you between December 16, 2025 and January 14, 2026. Keep it with your records and compare it with the bills you receive from your providers. You may owe the amount shown under "Your cost" directly to each provider.`, { size: 9.5 });
  flow.table([
    { header: 'Claim', width: 0.12 },
    { header: 'Provider', width: 0.26 },
    { header: 'Service date', width: 0.1 },
    { header: 'Service', width: 0.2 },
    { header: 'Billed', width: 0.08, align: 'right' },
    { header: 'Allowed', width: 0.08, align: 'right' },
    { header: 'Plan paid', width: 0.08, align: 'right' },
    { header: 'Your cost', width: 0.08, align: 'right' },
  ], [
    ...claims.map(([claim, provider, date, service, billed, allowed, paid, owed]) => [claim, provider, numericDate(date), service, amount(billed), amount(allowed), amount(paid), amount(owed)]),
    [{ text: 'Totals', face: 'sans-bold' }, '', '', '', amount(sum(4)), amount(sum(5)), amount(sum(6)), { text: amount(sum(7)), face: 'sans-bold' }],
  ], { size: 8, keep: ['Hugo Lockridge, MD', 'Wrenfield Family Medicine', ...new Set(claims.slice(1).map((claim) => claim[1]))] });
  flow.paragraph('Claim notes: For claims C26-0047310 and C26-0047702 the allowed amount was applied to your 2026 deductible, which began again on January 1, 2026. Office visits are subject to a $40.00 copay. Discounts between the billed and allowed amounts are negotiated by the plan with in-network providers; you are not responsible for them.', { size: 8.5 });
  flow.heading('Your 2026 benefits so far', { level: 3, size: 10 });
  flow.table([{ header: 'Accumulator', width: 0.4 }, { header: 'Met so far', width: 0.2, align: 'right' }, { header: 'Annual limit', width: 0.2, align: 'right' }, { header: 'Remaining', width: 0.2, align: 'right' }], [
    ['In-network deductible (individual)', money(26250), money(150000), money(123750)],
    ['In-network out-of-pocket maximum', money(26250), money(450000), money(423750)],
    ['Physical therapy visits', '1', '30', '29'],
  ], { size: 8.5 });
  flow.paragraph('If you disagree with how a claim was processed, you may ask for an appeal within 180 days of the date you receive this statement. Call Member Services or write to the address above and include the claim number. You may also ask for copies of the documents and criteria used to decide your claim, free of charge.', { size: 8 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Health plan explanation of benefits with six processed claims',
    kind: 'statement',
    textLayer: 'native',
    pages: pages.length,
    categories: ['healthcare', 'statement', 'table', 'competing_dates'],
    notes: `The statement date (${longDate(statementDate)}) is a cell in the member header table. Six service dates in December 2025 and January 2026 fill the claims table, and the processing window and deductible reset date are written in prose. Filed for the member it is about; from the plan is also accepted. Providers are not parties.`,
    gold: gold({
      type: 'Explanation of Benefits',
      date: statementDate,
      role: 'issuance',
      forbiddenDates: [['2025-12-16', 'service date of the first claims'], ['2026-01-07', 'service date of the imaging claim'], ['2026-01-14', 'end of the claims processing window']],
      parties: [member],
      relation: 'for',
      acceptablePartySets: [{ parties: [plan], relation: 'from' }, { parties: [member], relation: 'to' }],
      roles: [[member, 'subject'], [member, 'patient'], [member, 'recipient'], [plan, 'issuer'], [plan, 'payer']],
      forbiddenParties: [['Hugo Lockridge', 'treating physician'], ['Fallow Creek Physical Therapy', 'provider'], ['Bellmoor Diagnostic Laboratory', 'provider']],
      facts: [[money(sum(7)), amount(sum(7))], ['physical therapy'], [member, 'Achterberg']],
      subjectTerms: ['claims', 'physical therapy', 'benefits'],
      readiness: 'ready',
      dateText: [longDate(statementDate)],
    }),
  });
}

export function priorAuthorization() {
  const id = 'prior-authorization';
  const plan = 'Silverlode Health Partners';
  const member = 'Florian Okonkwo';
  const memberId = 'SHP-448190277';
  const birth = '1981-04-19';
  const received = '2026-03-04';
  const decided = '2026-03-09';
  const windowStart = '2026-03-10';
  const windowEnd = addDays(windowStart, 89);
  const authNumber = 'PA-2026-0331775';
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 54, bottom: 60 }, keep: [plan, member] });
  flow.y = letterhead(flow.page, { name: plan, lines: ['Utilization Management Department', '9 Silverlode Plaza, Silverlode, NV 89411  -  Fax (775) 555-0108  -  Provider line (775) 555-0109'], size: 15 });
  flow.paragraph(longDate(decided), { after: 10 });
  for (const line of [member, '58 Umber Street', 'Copper Flats, AZ 85219']) flow.paragraph(line, { after: 0 });
  flow.space(10);
  flow.heading('NOTICE OF PRIOR AUTHORIZATION APPROVAL', { level: 2, align: 'center' });
  flow.table([{ header: 'Member and request', width: 0.38 }, { header: '', width: 0.62 }], [
    ['Member', member],
    ['Member ID', memberId],
    ['Date of birth', numericDate(birth)],
    ['Authorization number', authNumber],
    ['Request received', numericDate(received)],
    ['Date of determination', numericDate(decided)],
    ['Requesting provider', 'Celeste Iwasaki, MD - Bellmoor Orthopedic Associates'],
    ['Servicing facility', 'Copper Flats Imaging Center (in network)'],
    ['Service approved', 'MRI, lumbar spine, without contrast (CPT 72148), 1 unit'],
    ['Diagnosis', 'M54.16 Radiculopathy, lumbar region'],
    ['Approved service window', `${numericDate(windowStart)} through ${numericDate(windowEnd)}`],
  ], { size: 9.5 });
  flow.paragraph(`Dear ${member}:`);
  flow.paragraph(`${plan} has approved the request your provider submitted for the service listed above. We reviewed the request against the plan's medical policy for advanced imaging of the spine, including the record of six weeks of physical therapy and anti-inflammatory medication without improvement and the documented weakness in your left leg. The service must be performed during the approved service window; if it is not, your provider will need to submit a new request.`);
  flow.paragraph('This approval means the service is medically necessary under the terms of your plan. It is not a guarantee of payment. Payment depends on your eligibility on the date of service, your plan\'s benefits, including any deductible, copayment, or coinsurance, and the claim submitted by the servicing facility. Your plan requires a $250 copayment for advanced imaging at an in-network facility.');
  flow.paragraph('If the service is performed at an out-of-network facility, your costs may be higher, and the approval applies only if your plan provides out-of-network benefits. Please contact Member Services at the number on your ID card with any questions about this approval or your coverage.');
  flow.paragraph('Clinical review: Tove Halloran, RN, Utilization Review Nurse. Physician review: Kasimir Dunmore, MD, Associate Medical Director. A copy of this notice has been sent to your requesting provider and to the servicing facility.', { size: 9.5 });
  flow.paragraph('Sincerely,', { after: 4 });
  flow.paragraph(`Utilization Management, ${plan}`, { after: 6 });
  flow.paragraph('You have the right to ask for a copy of the criteria used to make this decision, free of charge. Language assistance services are available at no cost; call Member Services.', { size: 8, grey: 0.3 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Health plan prior authorization approval for a lumbar MRI',
    kind: 'notice',
    textLayer: 'native',
    pages: pages.length,
    categories: ['healthcare', 'competing_dates', 'notice', 'table'],
    notes: `The determination date (${numericDate(decided)}, also written out as ${longDate(decided)} at the top of the letter) is the answer. The table also holds the member's date of birth (${numericDate(birth)}), the request received date (${numericDate(received)}), and the service window (${numericDate(windowStart)} through ${numericDate(windowEnd)}). The member ID and authorization number are identifiers a good description carries. The requesting physician and the two clinical reviewers are named but are not parties.`,
    gold: gold({
      type: 'Notice of Prior Authorization Approval',
      acceptableTypes: ['Prior Authorization Approval', 'Prior Authorization'],
      date: decided,
      role: 'notice',
      forbiddenDates: [[birth, 'member date of birth'], [received, 'date the request was received'], [windowStart, 'start of the approved service window'], [windowEnd, 'end of the approved service window']],
      parties: [member],
      relation: 'for',
      acceptablePartySets: [{ parties: [plan], relation: 'from' }, { parties: [member], relation: 'to' }],
      roles: [[member, 'subject'], [member, 'patient'], [member, 'recipient'], [plan, 'issuer'], [plan, 'payer']],
      forbiddenParties: [['Celeste Iwasaki', 'requesting physician'], ['Tove Halloran', 'clinical reviewer'], ['Kasimir Dunmore', 'physician reviewer']],
      facts: [['MRI'], ['lumbar', '72148'], [authNumber, memberId]],
      subjectTerms: ['MRI', 'lumbar spine', 'prior authorization', 'approved'],
      readiness: 'ready',
      dateText: [longDate(decided), numericDate(decided)],
    }),
  });
}
