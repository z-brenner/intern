/// Word, PowerPoint, Excel, and CSV documents. The worker reads Word and
/// PowerPoint through AnyDoc as Markdown (headings, paragraphs, tables; not
/// running headers or footers) and workbooks one sheet per page, so the
/// facts the gold depends on live in the body, slides, and cells.
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { addDays, amount, longDate, money, numericDate, toDays } from '../lib/format.mjs';
import { companies, people } from '../lib/names.mjs';
import { docx, docxText, pptx, pptxText, xlsx, xlsxText } from '../lib/office.mjs';
import { result } from './common.mjs';

const P = (text) => ({ type: 'paragraph', text });
const H = (text, level = 1) => ({ type: 'heading', text, level });

export function separationAgreement() {
  const id = 'separation-agreement';
  const company = 'Mossgiel Biomedical Inc.';
  const employee = 'Corinne Abernathy';
  const signer = 'Anika Rosenthal';
  const asOf = '2026-04-06';
  const separation = '2026-03-31';
  const considerBy = '2026-04-27';
  const employmentDate = '2019-06-02';
  const cobraEnd = '2026-09-30';
  const optionDeadline = '2026-12-31';
  const blocks = [
    { type: 'title', text: 'Separation Agreement and General Release' },
    P(`This Separation Agreement and General Release (this "Agreement") is entered into as of ${longDate(asOf)} by and between ${company}, a Delaware corporation (the "Company"), and ${employee} ("Employee").`),
    H('Recitals', 2),
    P(`A. Employee has been employed by the Company since ${longDate(employmentDate)}, most recently as Director of Clinical Operations, under an Employment Agreement dated ${longDate(employmentDate)} (the "Employment Agreement").`),
    P(`B. Employee's employment with the Company ended on ${longDate(separation)} (the "Separation Date") as part of the consolidation of the Company's clinical operations function in its Wrenfield, Ohio facility.`),
    P('C. The Company and Employee wish to resolve fully and finally all matters between them, including any arising out of Employee\'s employment and its termination.'),
    H('1. Final Pay and Benefits'),
    P('The Company has paid Employee all base salary earned through the Separation Date and all accrued but unused paid time off (96.5 hours), whether or not Employee signs this Agreement. Employee\'s group health coverage continued through the end of the month that includes the Separation Date.'),
    H('2. Severance Benefits'),
    P('Provided that Employee signs this Agreement within the consideration period in Section 6 and does not revoke it, the Company will provide the following (the "Severance Benefits"), which Employee acknowledges exceed anything to which Employee is otherwise entitled:'),
    { type: 'table', rows: [
      ['Benefit', 'Amount', 'Timing'],
      ['Base salary continuation', money(4875000), 'Thirteen equal biweekly installments on regular payroll dates, beginning on the first payroll date after the Effective Date'],
      ['COBRA premium reimbursement', `Up to ${money(1134000)}`, `Monthly, for coverage through ${longDate(cobraEnd)} or earlier eligibility for other coverage`],
      ['Prorated 2026 bonus', money(920000), 'Lump sum within thirty days after the Effective Date'],
      ['Outplacement services', money(450000), 'Paid directly to the provider; must begin within ninety days'],
    ], widths: [2800, 2200, 4360] },
    H('3. Equity Awards'),
    P(`The 4,200 vested stock options held by Employee on the Separation Date will remain exercisable until ${longDate(optionDeadline)}, notwithstanding the shorter post-termination exercise period in the applicable award agreements. All unvested options and restricted stock units were forfeited on the Separation Date.`),
    H('4. General Release of Claims'),
    P('In exchange for the Severance Benefits, Employee, on behalf of Employee and Employee\'s heirs and assigns, releases the Company, its affiliates, and their officers, directors, employees, and agents from all claims, known or unknown, arising on or before the date Employee signs this Agreement, including claims arising out of Employee\'s employment or its termination, claims under the Employment Agreement, and claims under federal, state, and local laws prohibiting discrimination, harassment, or retaliation, including the Age Discrimination in Employment Act. This release does not waive rights that cannot be waived by law, rights to vested benefits, or the right to file a charge with a government agency, although Employee waives any right to recover money in connection with such a charge.'),
    H('5. Return of Property; Continuing Obligations'),
    P('Employee confirms that Employee has returned all Company property, including the Company laptop and badge, and has not retained copies of Company documents. Employee\'s obligations under the Employee Confidentiality and Invention Assignment Agreement remain in effect. Each party agrees not to make statements that disparage the other; the Company\'s obligation is limited to instructing the members of its executive leadership team.'),
    H('6. Consideration and Revocation'),
    P(`Employee has twenty-one days to consider this Agreement and must sign and return it no later than ${longDate(considerBy)}. Employee is advised to consult an attorney before signing. Employee may revoke this Agreement within seven days after signing it by written notice to the Company's Chief People Officer. This Agreement becomes effective and enforceable on the eighth day after Employee signs it without revoking it (the "Effective Date").`),
    H('7. Cooperation'),
    P('For six months after the Separation Date, Employee will reasonably cooperate with the Company in transitioning Employee\'s responsibilities and in any regulatory inspection or litigation concerning matters within Employee\'s knowledge. The Company will reimburse reasonable expenses and pay $150 per hour for time spent at its request after the first ten hours.'),
    H('8. Miscellaneous'),
    P('This Agreement is governed by the laws of the State of Ohio. It is the entire agreement between the parties on its subject matter and supersedes all prior agreements, except the agreements expressly preserved in Sections 3 and 5. It may be amended only in a writing signed by both parties, and may be signed in counterparts and electronically. Nothing in this Agreement is an admission of wrongdoing by either party.'),
    { type: 'table', rows: [
      [company, employee],
      [`By: ______________________ Name: ${signer}, Chief People Officer Date: __________`, 'Signature: ______________________ Date: __________'],
    ], widths: [4680, 4680] },
  ];
  const bytes = docx({ blocks, header: `${company} - Separation Agreement - Confidential`, footer: 'Employee initials: ______', title: 'Separation Agreement and General Release' });
  return result({
    id,
    extension: 'docx',
    files: [{ name: `${id}.docx`, bytes }],
    text: [docxText(blocks)],
    title: 'Employee separation agreement and general release (Word)',
    kind: 'contract',
    textLayer: 'office',
    pages: 1,
    categories: ['hr', 'docx', 'competing_dates', 'referenced_agreement', 'contract'],
    notes: `"Entered into as of ${longDate(asOf)}" in the first sentence is the answer. The separation date (${longDate(separation)}) is defined in the recitals, the employment agreement (${longDate(employmentDate)}) is referenced, the consideration deadline (${longDate(considerBy)}), COBRA end (${longDate(cobraEnd)}), and option exercise deadline (${longDate(optionDeadline)}) are all stated, and the defined "Effective Date" is relative (the eighth day after signing) with no calendar date. Signature lines are blank. One page as authored: a Word file has no fixed pagination for the reader.`,
    gold: gold({
      type: 'Separation Agreement and General Release',
      acceptableTypes: ['Separation Agreement'],
      date: asOf,
      role: 'effective',
      forbiddenDates: [[separation, 'separation date (last day of employment)'], [considerBy, 'deadline to sign'], [employmentDate, 'date of the employment agreement and hire date'], [cobraEnd, 'end of COBRA reimbursement'], [optionDeadline, 'option exercise deadline']],
      parties: [company, employee],
      relation: 'between',
      acceptablePartySets: [{ parties: [employee], relation: 'with' }],
      roles: [[company, 'employer'], [employee, 'employee'], [employee, 'counterparty']],
      forbiddenParties: [[signer, 'company signatory']],
      facts: [[money(4875000), '48,750'], [employee, 'Abernathy'], ['release', 'severance']],
      subjectTerms: ['separation', 'severance', 'release', 'COBRA'],
      readiness: 'ready',
      dateText: [longDate(asOf)],
    }),
  });
}

export function demandLetter() {
  const id = 'demand-letter';
  const firm = 'Kilbride Moncrieff LLP';
  const client = 'Highmeadow Orchard Supply Co.';
  const debtor = 'Fernvale Cider Works LLC';
  const attorney = 'Valentina Kilbride';
  const owner = 'Oswin Haverford';
  const letterDate = '2026-07-08';
  const deadline = '2026-07-22';
  const invoices = [
    ['HOS-30412', '2026-02-09', 612480],
    ['HOS-30588', '2026-03-02', 488000],
    ['HOS-30701', '2026-03-23', 731220],
    ['HOS-30815', '2026-04-13', 559860],
    ['HOS-30922', '2026-04-27', 350000],
  ];
  const total = invoices.reduce((sum, [, , value]) => sum + value, 0);
  const interest = 128740;
  const blocks = [
    P(longDate(letterDate)),
    { type: 'paragraph', runs: [{ text: 'VIA EMAIL AND CERTIFIED MAIL, RETURN RECEIPT REQUESTED', bold: true }] },
    P(debtor),
    P(`Attn: ${owner}, Managing Member`),
    P('2950 Orchard Row, Tamsin Valley, NM 87501'),
    { type: 'paragraph', runs: [{ text: `Re: Demand for Payment - ${client} - Past-Due Invoices Totaling ${money(total)}`, bold: true }] },
    P(`Dear Mr. Haverford:`),
    P(`This firm represents ${client} ("Highmeadow") in connection with amounts owed to it by ${debtor} ("Fernvale"). Between February and April 2026 Highmeadow delivered apple bins, picking ladders, cold-storage supplies, and replacement press cloths to Fernvale's cidery on open account, on terms of net thirty days. Fernvale accepted every delivery without objection, and Highmeadow's records show no disputes and no returns.`),
    P('The following invoices remain unpaid:'),
    { type: 'table', rows: [
      ['Invoice', 'Invoice date', 'Due date', 'Amount', 'Days past due'],
      ...invoices.map(([number, date, value]) => [number, longDate(date), longDate(addDays(date, 30)), money(value), String(toDays(letterDate) - toDays(date) - 30)]),
      ['Total', '', '', money(total), ''],
    ], widths: [1700, 2000, 2000, 1900, 1760] },
    P(`Highmeadow's credit terms provide for interest on past-due balances at one and one-half percent per month. Interest accrued through the date of this letter is ${money(interest)}, for a total now due of ${money(total + interest)}.`),
    P(`Highmeadow hereby demands payment of ${money(total + interest)} in full, by wire transfer or certified check payable to Highmeadow, no later than ${longDate(deadline)}. Our client has authorized us to accept payment in two equal installments, the second due thirty days after the first, if Fernvale signs the enclosed payment agreement by the same date.`),
    P('If payment or a signed payment agreement is not received by that date, Highmeadow intends to file suit without further notice to recover the full balance, accrued interest, and its costs and attorneys\' fees as its terms of sale allow. Nothing in this letter waives any of Highmeadow\'s rights or remedies, all of which are expressly reserved.'),
    P('Please direct all communications about this matter to me rather than to Highmeadow.'),
    P('Very truly yours,'),
    P(`/s/ ${attorney}`),
    P(`${attorney}, Partner`),
    P(`cc: Hollis Brightwater, Credit Manager, ${client} (by email)`),
    P('Enclosure: Payment agreement'),
  ];
  const bytes = docx({ blocks, header: `${firm} - 1400 Ridgewell Terrace, Suite 900, Tamsin Valley, NM 87505 - (505) 555-0181`, footer: 'Confidential settlement communication', title: 'Demand for payment' });
  return result({
    id,
    extension: 'docx',
    files: [{ name: `${id}.docx`, bytes }],
    text: [docxText(blocks)],
    title: 'Law firm demand letter for past-due supply invoices (Word)',
    kind: 'letter',
    textLayer: 'office',
    pages: 1,
    categories: ['letter', 'docx', 'competing_dates', 'table'],
    notes: `The letter's date (${longDate(letterDate)}) is its first paragraph. Five invoice dates and five due dates sit in a table, and the payment deadline (${longDate(deadline)}) is stated in the demand; all are traps. The law firm's letterhead is in the Word header, which the reader does not extract, so the firm is named only in "This firm represents". The letter is addressed to the debtor; it is also accepted as from the client or for the debtor. The attorney, the debtor's managing member, and the client's credit manager are not parties.`,
    gold: gold({
      type: 'Demand Letter',
      acceptableTypes: ['Demand for Payment', 'Payment Demand Letter'],
      date: letterDate,
      role: 'notice',
      forbiddenDates: [[deadline, 'payment deadline'], ...invoices.slice(0, 3).map(([number, date]) => [date, `date of invoice ${number}`]), [addDays(invoices[0][1], 30), 'due date of the oldest invoice']],
      parties: [debtor],
      relation: 'to',
      acceptablePartySets: [{ parties: [debtor], relation: 'for' }, { parties: [client], relation: 'from' }, { parties: [client, debtor], relation: 'between' }],
      roles: [[debtor, 'recipient'], [debtor, 'subject'], [client, 'issuer'], [client, 'seller'], [debtor, 'buyer']],
      forbiddenParties: [[attorney, 'attorney who signs'], [owner, 'debtor\'s managing member'], ['Hollis Brightwater', 'client\'s credit manager, copied']],
      facts: [[money(total + interest), amount(total + interest), money(total), amount(total)], [client, 'Highmeadow'], ['invoices', 'past-due', 'past due']],
      subjectTerms: ['demand', 'past-due invoices', 'payment'],
      readiness: 'ready',
      dateText: [longDate(letterDate)],
    }),
  });
}

export function boardResolution() {
  const id = 'board-resolution';
  const rng = Rng.from(id);
  const company = 'Summerhill Robotics, Inc.';
  const effective = '2025-11-03';
  const valuation = '2025-10-15';
  const priorMeeting = '2025-09-09';
  const directors = [['Celeste Fontaine', effective], ['Jasper Okafor', '2025-11-04'], ['Priya Lindqvist', '2025-11-05'], ['Hugo Ramasamy', effective], ['Marisol Fennimore', '2025-11-04']];
  const optionees = people(rng.fork('optionees'), 12, { exclude: directors.map(([name]) => name) });
  const grants = optionees.map((name, index) => [name, ['Software Engineer', 'Senior Software Engineer', 'Controls Engineer', 'Field Applications Engineer', 'Product Manager', 'Mechanical Engineer'][index % 6], String(rng.int(8, 60) * 500), longDate(addDays('2025-09-01', rng.int(0, 60)))]);
  const blocks = [
    { type: 'title', text: 'Action by Unanimous Written Consent of the Board of Directors' },
    P(company),
    P(`The undersigned, being all of the members of the Board of Directors (the "Board") of ${company}, a Delaware corporation (the "Company"), acting pursuant to Section 141(f) of the Delaware General Corporation Law and the Company's Bylaws, hereby adopt the following resolutions by unanimous written consent, effective as of ${longDate(effective)}, with the same force and effect as if adopted at a duly called meeting of the Board:`),
    H('1. Approval of Minutes', 2),
    P(`RESOLVED, that the minutes of the meeting of the Board held on ${longDate(priorMeeting)}, in the form presented to the Board, are approved.`),
    H('2. 2026 Operating Plan', 2),
    P('RESOLVED, that the Company\'s 2026 operating plan, providing for operating expenses of $18,400,000, capital expenditures of $1,150,000, and year-end headcount of 104, in the form presented to the Board, is approved and adopted.'),
    H('3. Fair Market Value and Stock Option Grants', 2),
    P(`WHEREAS, the Board has reviewed the independent valuation of the Company's common stock as of ${longDate(valuation)} prepared by Wexcombe Valuation Advisors LLC, which concluded that the fair market value of a share of common stock was $2.14;`),
    P('RESOLVED, that the Board determines the fair market value of the common stock to be $2.14 per share, and grants to each person listed below an option under the Company\'s 2021 Equity Incentive Plan to purchase the number of shares of common stock shown, at an exercise price of $2.14 per share, vesting over four years from the vesting commencement date shown, with one-quarter vesting on the first anniversary and the balance in thirty-six equal monthly installments, subject to continued service:'),
    { type: 'table', rows: [['Optionee', 'Position', 'Shares', 'Vesting commencement'], ...grants], widths: [2600, 2900, 1400, 2460] },
    H('4. Venture Debt Facility', 2),
    P('RESOLVED, that the Company is authorized to enter into a Loan and Security Agreement with Linden Cross Trust Company providing for term loans of up to $5,000,000, on terms substantially consistent with the term sheet presented to the Board, and to grant the lender a warrant to purchase up to 120,000 shares of Series B Preferred Stock at $3.85 per share; and that the Chief Executive Officer and Chief Financial Officer, each acting alone, are authorized to negotiate, execute, and deliver the loan documents.'),
    H('5. General Authority', 2),
    P('RESOLVED, that the officers of the Company are authorized to take all further actions and to execute all further documents that they consider necessary or advisable to carry out the foregoing resolutions, and all actions previously taken by them consistent with these resolutions are ratified.'),
    P('This consent may be executed in counterparts and by electronic signature, and will be filed with the minutes of the proceedings of the Board.'),
    { type: 'table', rows: [['Director', 'Signature', 'Date signed'], ...directors.map(([name, date]) => [name, `/s/ ${name}`, longDate(date)])], widths: [3120, 3120, 3120] },
  ];
  const bytes = docx({ blocks, header: 'Summerhill Robotics - Board Consent - Confidential', title: 'Unanimous Written Consent of the Board' });
  return result({
    id,
    extension: 'docx',
    files: [{ name: `${id}.docx`, bytes }],
    text: [docxText(blocks)],
    title: 'Board of directors unanimous written consent approving option grants and debt (Word)',
    kind: 'resolution',
    textLayer: 'office',
    pages: 1,
    categories: ['docx', 'irrelevant_names', 'competing_dates', 'table'],
    notes: `Effective as of ${longDate(effective)}, stated once in the preamble. Five directors sign on three different dates in a table at the end; the 409A valuation date (${longDate(valuation)}), the prior meeting (${longDate(priorMeeting)}), and twelve vesting commencement dates are also dated. Seventeen people (directors and optionees), a lender, and a valuation firm are named; the consent is the company's.`,
    gold: gold({
      type: 'Action by Unanimous Written Consent of the Board of Directors',
      acceptableTypes: ['Unanimous Written Consent of the Board of Directors', 'Written Consent of the Board of Directors', 'Board Resolution', 'Board Consent', 'Unanimous Written Consent'],
      date: effective,
      role: 'effective',
      forbiddenDates: [['2025-11-04', 'a later director signature date'], ['2025-11-05', 'last director signature date'], [valuation, 'valuation date of the common stock'], [priorMeeting, 'prior board meeting whose minutes are approved']],
      parties: [company],
      relation: 'for',
      acceptablePartySets: [{ parties: [company], relation: 'from' }],
      roles: [[company, 'subject'], [company, 'issuer']],
      forbiddenParties: [['Linden Cross Trust Company', 'lender under the authorized facility'], ['Wexcombe Valuation Advisors LLC', 'valuation firm'], [directors[0][0], 'director'], [optionees[0], 'optionee']],
      facts: [['option', 'options'], ['$2.14'], ['$5,000,000', 'venture debt', 'Loan and Security Agreement']],
      subjectTerms: ['written consent', 'stock option grants', 'venture debt', 'operating plan'],
      readiness: 'ready',
      dateText: [longDate(effective)],
    }),
  });
}

export function quarterlyBusinessReview() {
  const id = 'quarterly-business-review';
  const vendor = 'Lindenhall Logistics Software Inc.';
  const customer = 'Pemberly Falls Distribution Co.';
  const presented = '2026-09-24';
  const release = '2026-11-12';
  const nextRelease = '2027-02-18';
  const nextQbr = '2026-12-17';
  const termEnd = '2027-03-31';
  const proposalDue = '2027-01-15';
  const slides = [
    { title: 'Quarterly Business Review - Q3 2026', subtitle: [`Prepared for ${customer}`, `Presented by ${vendor}`, `Presented ${longDate(presented)}`] },
    { title: 'Agenda', body: ['Attendees and goals for the session', 'Q3 results: volume, accuracy, and speed', 'Support summary', 'Value delivered year to date', 'Product roadmap', 'Open action items', 'Renewal planning and next steps'] },
    { title: 'Attendees', table: [['Name', 'Title', 'Company'], ['Signe Holmqvist', 'VP Operations', customer], ['Saul Ravenscroft', 'Director, Distribution Systems', customer], ['Mireille Ostrowski', 'Warehouse Manager, Building 2', customer], ['Kofi Achterberg', 'Customer Success Manager', vendor], ['Juniper Dahlquist', 'Account Executive', vendor], ['Thaddeus Okonkwo', 'Solutions Architect', vendor]] },
    { title: 'Executive summary', body: ['Order volume up 14% quarter over quarter with no added headcount', 'Pick accuracy 99.71% (target 99.5%) across both buildings', 'Dock-to-stock time down from 11.2 to 7.9 hours after slotting changes', 'Two Severity 2 incidents, both resolved within SLA', 'Wave planning v2 beta requested for Building 2'] },
    { title: 'Q3 2026 operating metrics', table: [['Metric', 'July', 'August', 'September*'], ['Orders processed', '182,440', '196,118', '204,905'], ['Lines picked', '1,284,300', '1,377,912', '1,431,070'], ['Pick accuracy', '99.66%', '99.72%', '99.74%'], ['Dock-to-stock (hours)', '9.8', '8.4', '7.9'], ['Active users', '214', '221', '229']], footer: '* September through the 21st, annualized for the month' },
    { title: 'Support summary', table: [['Severity', 'Opened', 'Resolved in SLA', 'Median time to resolve'], ['Severity 1', '0', '-', '-'], ['Severity 2', '2', '2', '5.5 hours'], ['Severity 3', '17', '16', '2.1 days'], ['How-to questions', '41', '41', '4.0 hours']] },
    { title: 'Value delivered year to date', body: ['Labor savings from slotting optimization: $412,000 (estimate agreed with Finance)', 'Mis-ship credits avoided: $96,500 versus 2025 run rate', 'Overtime hours down 22% in peak weeks', 'Carrier compliance chargebacks down 61%'] },
    { title: 'Product roadmap', table: [['Release', 'Target date', 'Highlights'], ['26.4', longDate(release), 'Wave planning v2, labor forecasting dashboard'], ['27.1', longDate(nextRelease), 'Cartonization rules engine, returns grading app'], ['27.2', 'Q2 2027', 'Yard management integration (beta)']] },
    { title: 'Open action items', table: [['Item', 'Owner', 'Due'], ['Enable wave planning v2 beta in Building 2', 'Thaddeus Okonkwo', '2026-10-09'], ['Share Q4 peak staffing plan', 'Mireille Ostrowski', '2026-10-16'], ['Review label printer failover runbook', 'Saul Ravenscroft', '2026-10-23']] },
    { title: 'Renewal planning', body: [`Current subscription term ends ${longDate(termEnd)}`, `Renewal proposal to be delivered by ${longDate(proposalDue)}`, 'Expansion option: third building (Oriel Junction) under evaluation for 2027', 'Multi-year pricing available for a three-year term'] },
    { title: 'Next steps and contacts', body: [`Next quarterly review: ${longDate(nextQbr)}`, 'Customer Success: Kofi Achterberg, kofi.achterberg@lindenhall-logistics.example', 'Account Executive: Juniper Dahlquist, (608) 555-0164', 'Support portal: support.lindenhall-logistics.example'] },
  ];
  const bytes = pptx({ slides, title: 'Q3 2026 Quarterly Business Review' });
  return result({
    id,
    extension: 'pptx',
    files: [{ name: `${id}.pptx`, bytes }],
    text: [pptxText(slides).join('\n\n')],
    title: 'Software vendor\'s quarterly business review deck for a distribution customer',
    kind: 'presentation',
    textLayer: 'office',
    pages: slides.length,
    categories: ['presentation', 'pptx', 'competing_dates', 'irrelevant_names', 'table'],
    notes: `Presented ${longDate(presented)} (title slide). Roadmap release dates (${longDate(release)}, ${longDate(nextRelease)}), the subscription term end (${longDate(termEnd)}), the renewal proposal due date (${longDate(proposalDue)}), the next review (${longDate(nextQbr)}), and three action-item due dates are traps. Six attendees are named in a table and three contacts on the last slide; they are not parties. Prepared for the customer; from the vendor is also accepted.`,
    gold: gold({
      type: 'Quarterly Business Review',
      acceptableTypes: ['Business Review', 'QBR'],
      date: presented,
      role: 'issuance',
      forbiddenDates: [[release, 'roadmap release date'], [nextRelease, 'roadmap release date'], [termEnd, 'subscription term end'], [proposalDue, 'renewal proposal due'], [nextQbr, 'next quarterly review']],
      parties: [customer],
      relation: 'for',
      acceptablePartySets: [{ parties: [vendor], relation: 'from' }, { parties: [customer], relation: 'with' }, { parties: [vendor, customer], relation: 'between' }],
      roles: [[customer, 'subject'], [customer, 'client'], [customer, 'counterparty'], [vendor, 'issuer'], [vendor, 'seller']],
      forbiddenParties: [['Kofi Achterberg', 'vendor customer success manager'], ['Signe Holmqvist', 'customer attendee'], ['Juniper Dahlquist', 'vendor account executive']],
      facts: [['Q3 2026', 'third quarter'], ['99.71%', 'pick accuracy', '14%'], [customer, 'Pemberly Falls']],
      subjectTerms: ['quarterly business review', 'pick accuracy', 'roadmap', 'renewal'],
      readiness: 'ready',
      dateText: [longDate(presented)],
    }),
  });
}

export function productLaunchPlan() {
  const id = 'product-launch-plan';
  const company = 'Quarrystone Audio Labs Inc.';
  const version = '2026-06-03';
  const freeze = '2026-04-24';
  const cert = '2026-07-10';
  const press = '2026-08-18';
  const launch = '2026-09-01';
  const retail = '2026-09-15';
  const slides = [
    { title: 'Fieldnote S2 Product Launch Plan', subtitle: [company, 'Launch Council review', `Version 2.1 - ${longDate(version)}`] },
    { title: 'Launch goals', body: ['18,000 units sold through in the first 90 days', '$5.2 million net revenue in the launch quarter', 'Net promoter score of 50 or better from beta users', 'Attach rate of 35% for the S2 field kit accessory bundle'] },
    { title: 'Who it is for', body: ['Podcasters and field journalists who record on location', 'Sound designers capturing ambience for film and games', 'Positioning: the first pocket recorder with 32-bit float on four inputs', 'Key competitor gap: battery life (14 hours vs. 6-8 hours)'] },
    { title: 'Pricing and SKUs', table: [['SKU', 'Description', 'MSRP'], ['FN-S2-BLK', 'Fieldnote S2 recorder, black', '$449'], ['FN-S2-KIT', 'S2 + windscreen, case, 128 GB card', '$549'], ['FN-ACC-XLR2', 'Dual XLR input module', '$129']] },
    { title: 'Launch milestones', table: [['Milestone', 'Date', 'Owner', 'Status'], ['Design freeze', longDate(freeze), 'Engineering', 'Done'], ['Beta program', 'May 18 - June 26, 2026', 'Product', 'In progress'], ['Regulatory certification', longDate(cert), 'Compliance', 'On track'], ['Press briefing (embargoed)', longDate(press), 'Marketing', 'On track'], ['Launch (online store)', longDate(launch), 'All', 'On track'], ['Retail availability', longDate(retail), 'Sales', 'At risk']] },
    { title: 'Channel plan', body: ['Direct: online store, launch-week bundle', 'Retail: Saltmarsh Music Exchange (42 stores) and Corvid Camera & Audio (online)', 'Creators: 20 seeded units to field recordists, no paid placements', 'Education: campus pricing through Wrenfield Media Supply'] },
    { title: 'Risks and mitigations', table: [['Risk', 'Impact', 'Mitigation'], ['Preamp IC lead time slips', 'Retail date', 'Second source qualified; air freight budget held'], ['Firmware 1.0 battery reporting bug', 'Reviews', 'Fix in 1.0.2 before press units ship'], ['Certification retest', 'Launch date', 'Pre-scan completed; lab slot reserved']] },
    { title: 'Decisions needed today', body: ['Approve launch date and retail date as shown', 'Approve $410,000 launch marketing budget', 'Approve air freight contingency of up to $60,000', 'Owners: Product - Ilse Marchetti; Marketing - Rafferty Okonkwo'] },
  ];
  const bytes = pptx({ slides, title: 'Fieldnote S2 Product Launch Plan' });
  return result({
    id,
    extension: 'pptx',
    files: [{ name: `${id}.pptx`, bytes }],
    text: [pptxText(slides).join('\n\n')],
    title: 'Audio hardware company\'s product launch plan deck with a milestone table',
    kind: 'presentation',
    textLayer: 'office',
    pages: slides.length,
    categories: ['presentation', 'table', 'pptx', 'competing_dates'],
    notes: `The plan is dated by its version line (${longDate(version)}) on the title slide. The milestone table carries six dates (design freeze, certification, press, launch ${longDate(launch)}, retail), all traps. Retailers named in the channel plan are not parties. An internal plan is filed from the company; no party at all is also accepted.`,
    gold: gold({
      type: 'Product Launch Plan',
      acceptableTypes: ['Launch Plan'],
      date: version,
      role: 'issuance',
      forbiddenDates: [[launch, 'planned launch date'], [retail, 'retail availability date'], [cert, 'regulatory certification milestone'], [press, 'press briefing'], [freeze, 'design freeze']],
      parties: [company],
      relation: 'from',
      acceptablePartySets: [{ parties: [], relation: 'none' }, { parties: [company], relation: 'for' }],
      roles: [[company, 'issuer'], [company, 'subject']],
      forbiddenParties: [['Saltmarsh Music Exchange', 'retail channel partner'], ['Ilse Marchetti', 'product owner']],
      facts: [['Fieldnote S2', 'S2'], ['recorder'], ['$449', '18,000', '$5.2 million']],
      subjectTerms: ['launch plan', 'Fieldnote S2', 'recorder', 'milestones'],
      readiness: 'ready',
      dateText: [longDate(version)],
    }),
  });
}

export function payrollRegister() {
  const id = 'payroll-register';
  const rng = Rng.from(id);
  const company = 'Wrenfield Bakehouse LLC';
  const processor = 'Saltmarsh Payroll Services';
  const payDate = '2026-07-15';
  const periodStart = '2026-06-29';
  const periodEnd = '2026-07-12';
  const depositDue = '2026-07-17';
  const departments = ['Production', 'Production', 'Production', 'Retail', 'Retail', 'Delivery', 'Office'];
  const staff = people(rng.fork('staff'), 40);
  const rows = staff.map((name, index) => {
    const department = rng.pick(departments);
    const rate = department === 'Office' ? rng.int(2400, 3400) : rng.int(1650, 2650);
    const regular = rng.int(48, 80);
    const overtime = department === 'Production' ? rng.int(0, 9) : rng.int(0, 2);
    const gross = Math.round(rate * regular + rate * 1.5 * overtime);
    const federal = Math.round(gross * 0.09);
    const fica = Math.round(gross * 0.0765);
    const state = Math.round(gross * 0.0275);
    const retirement = rng.chance(0.6) ? Math.round(gross * rng.pick([0.03, 0.04, 0.05, 0.06])) : 0;
    const net = gross - federal - fica - state - retirement;
    return { id: `E${String(1040 + index * 7)}`, name, department, rate, regular, overtime, gross, federal, fica, state, retirement, net };
  });
  const dollars = (cents) => cents / 100;
  const sum = (key) => rows.reduce((total, row) => total + row[key], 0);
  const register = [
    [{ text: `${company} - Payroll Register`, bold: true }],
    ['Pay Date', { date: payDate }, 'Pay Period', { date: periodStart }, 'to', { date: periodEnd }, 'Check Run', 'PR-2026-14'],
    [`Prepared by ${processor}`, null, 'Company code', 'WB-2210', 'Frequency', 'Biweekly'],
    ['Emp ID', 'Employee', 'Department', 'Rate', 'Reg Hrs', 'OT Hrs', 'Gross Pay', 'Fed WH', 'FICA', 'State WH', '401(k)', 'Net Pay'].map((text) => ({ text, bold: true })),
    ...rows.map((row) => [row.id, row.name, row.department, dollars(row.rate), row.regular, row.overtime, dollars(row.gross), dollars(row.federal), dollars(row.fica), dollars(row.state), dollars(row.retirement), dollars(row.net)]),
    [{ text: 'Totals', bold: true }, `${rows.length} employees`, null, null, sum('regular'), sum('overtime'), dollars(sum('gross')), dollars(sum('federal')), dollars(sum('fica')), dollars(sum('state')), dollars(sum('retirement')), dollars(sum('net'))],
  ];
  const byDepartment = [...new Set(departments)].map((department) => {
    const members = rows.filter((row) => row.department === department);
    return [department, members.length, dollars(members.reduce((total, row) => total + row.gross, 0)), dollars(members.reduce((total, row) => total + row.net, 0))];
  });
  const sheets = [
    { name: 'Payroll Register', rows: register, widths: [10, 24, 12, 8, 8, 8, 11, 10, 10, 10, 10, 11] },
    { name: 'Department Summary', rows: [['Department', 'Employees', 'Gross Pay', 'Net Pay'].map((text) => ({ text, bold: true })), ...byDepartment] },
    { name: 'Tax Liabilities', sharedStrings: false, rows: [
      ['Liability', 'Amount', 'Deposit due'].map((text) => ({ text, bold: true })),
      ['Federal income tax withheld', dollars(sum('federal')), { date: depositDue }],
      ['Social Security and Medicare (employee + employer)', dollars(sum('fica') * 2), { date: depositDue }],
      ['State income tax withheld', dollars(sum('state')), { date: '2026-07-31' }],
      ['401(k) deferrals to plan trustee', dollars(sum('retirement')), { date: '2026-07-22' }],
    ] },
  ];
  const bytes = xlsx({ sheets, title: 'Payroll Register' });
  return result({
    id,
    extension: 'xlsx',
    files: [{ name: `${id}.xlsx`, bytes }],
    text: xlsxText(sheets),
    title: 'Bakery payroll register for one biweekly pay date, 40 employees (Excel)',
    kind: 'payroll',
    textLayer: 'sheet',
    pages: sheets.length,
    categories: ['spreadsheet', 'xlsx', 'hr', 'irrelevant_names', 'competing_dates', 'table'],
    notes: `The pay date is a real date cell (Excel serial with a date format) in the header row, which the reader renders as ${payDate}. The pay period start and end are date cells beside it, and the tax sheet carries deposit due dates (${depositDue} and later); all are traps. Forty employee names fill the register; none is a party. The register is the employer's; "from" the payroll processor that prepared it is also accepted.`,
    gold: gold({
      type: 'Payroll Register',
      date: payDate,
      role: 'other',
      forbiddenDates: [[periodStart, 'pay period start'], [periodEnd, 'pay period end'], [depositDue, 'federal tax deposit due date']],
      parties: [company],
      relation: 'for',
      acceptablePartySets: [{ parties: [company], relation: 'from' }, { parties: [processor], relation: 'from' }],
      roles: [[company, 'subject'], [company, 'employer'], [company, 'issuer'], [processor, 'issuer']],
      forbiddenParties: staff.slice(0, 3).map((name) => [name, 'employee in the register']),
      facts: [['40 employees', 'forty employees', '40'], [String(dollars(sum('gross'))), amount(sum('gross')), String(dollars(sum('net'))), amount(sum('net'))], [company, 'Wrenfield Bakehouse']],
      subjectTerms: ['payroll', 'pay date', 'biweekly', 'gross pay'],
      readiness: 'ready',
      dateText: [payDate],
    }),
  });
}

export function apAgingReport() {
  const id = 'ap-aging-report';
  const rng = Rng.from(id);
  const company = 'Lowmarsh Farm Equipment Co.';
  const asOf = '2026-08-31';
  const vendors = companies(rng.fork('vendors'), 32, { exclude: [company] });
  const buckets = vendors.map(() => {
    const current = rng.chance(0.8) ? rng.amount(20000, 2400000, 5) : 0;
    const days30 = rng.chance(0.55) ? rng.amount(10000, 1200000, 5) : 0;
    const days60 = rng.chance(0.3) ? rng.amount(5000, 600000, 5) : 0;
    const days90 = rng.chance(0.15) ? rng.amount(5000, 300000, 5) : 0;
    const over = rng.chance(0.08) ? rng.amount(5000, 180000, 5) : 0;
    return [current, days30, days60, days90, over];
  });
  const csvField = (value) => (/[",\n]/.test(value) ? `"${value.replaceAll('"', '""')}"` : value);
  const cell = (cents) => (cents ? amount(cents).replaceAll(',', '') : '');
  const totals = [0, 1, 2, 3, 4].map((index) => buckets.reduce((sum, row) => sum + row[index], 0));
  const records = [
    [company],
    ['Accounts Payable Aging Summary'],
    [`As of ${numericDate(asOf)}`],
    ['Report run 09/02/2026 07:41 AM by tmarchetti'],
    [],
    ['Vendor', 'Current', '1 - 30', '31 - 60', '61 - 90', '91 and over', 'Total'],
    ...vendors.map((vendor, index) => [vendor, ...buckets[index].map(cell), cell(buckets[index].reduce((a, b) => a + b, 0))]),
    ['TOTAL', ...totals.map(cell), cell(totals.reduce((a, b) => a + b, 0))],
  ];
  const width = 7;
  const csv = records.map((record) => [...record, ...Array(width - record.length).fill('')].map(csvField).join(',')).join('\r\n') + '\r\n';
  const textRows = records.filter((record) => record.length).map((record) => `| ${[...record, ...Array(width - record.length).fill('')].join(' | ')} |`);
  const top = vendors.map((vendor, index) => [vendor, buckets[index].reduce((a, b) => a + b, 0)]).sort((a, b) => b[1] - a[1]);
  return result({
    id,
    extension: 'csv',
    files: [{ name: `${id}.csv`, bytes: Buffer.from(csv, 'utf8') }],
    text: [textRows.join('\n')],
    title: 'Accounts payable aging summary exported from an accounting system (CSV)',
    kind: 'report',
    textLayer: 'sheet',
    pages: 1,
    categories: ['spreadsheet', 'csv', 'financial', 'irrelevant_names', 'table'],
    notes: `An accounting-system export: four title lines in the first column, then a table of 32 vendors with five aging buckets. The report is "As of ${numericDate(asOf)}" in a header cell; the run timestamp (09/02/2026) is left unscored - filing by the run date is wrong but not a trap. Every vendor is a name the reader should leave out; the report is the company's own.`,
    gold: gold({
      type: 'Accounts Payable Aging Summary',
      acceptableTypes: ['Accounts Payable Aging Report', 'AP Aging Report', 'A/P Aging Summary', 'Aging Report'],
      date: asOf,
      role: 'issuance',
      forbiddenDates: [],
      parties: [company],
      relation: 'for',
      acceptablePartySets: [{ parties: [company], relation: 'from' }],
      roles: [[company, 'subject'], [company, 'issuer']],
      forbiddenParties: top.slice(0, 3).map(([vendor]) => [vendor, 'vendor with a large balance']),
      facts: [[cell(totals.reduce((a, b) => a + b, 0)), amount(totals.reduce((a, b) => a + b, 0))], ['32 vendors', 'vendor'], [company, 'Lowmarsh']],
      subjectTerms: ['accounts payable', 'aging summary', 'vendor'],
      readiness: 'ready',
      dateText: [numericDate(asOf)],
    }),
  });
}

export function harvestLog() {
  const id = 'harvest-log';
  const rng = Rng.from(id);
  const vineyard = 'Rowanbrae Vineyards';
  const generated = '2026-10-02';
  const firstPick = '2026-08-28';
  const lastPick = '2026-09-30';
  const winemaker = 'Esme Varga';
  const blocks = [['B1', 'Sauvignon Blanc', 4.2], ['B2', 'Chardonnay', 6.8], ['B3', 'Pinot Noir', 5.5], ['B4', 'Pinot Noir', 3.9], ['B5', 'Syrah', 4.6], ['B6', 'Cabernet Franc', 3.1], ['B7', 'Grenache', 2.7]];
  const crews = people(rng.fork('crew'), 9);
  const picks = [];
  let date = firstPick;
  for (const [block, variety, acres] of blocks) {
    const passes = acres > 4.5 ? 2 : 1;
    for (let pass = 0; pass < passes; pass += 1) {
      const tons = Math.round(acres / passes * rng.int(260, 380)) / 100;
      picks.push([{ date }, block, variety, Math.round(tons * 2.2), tons, rng.pick(crews), `T-${rng.int(1, 14)}`]);
      date = addDays(date, rng.int(2, 4));
    }
  }
  picks[picks.length - 1][0] = { date: lastPick };
  const chemistry = [];
  for (const [block, variety] of blocks) {
    for (let sample = 0; sample < 3; sample += 1) {
      chemistry.push([{ date: addDays(firstPick, -14 + sample * 6 + rng.int(0, 3)) }, block, variety, rng.int(205, 262) / 10, rng.int(318, 372) / 100, rng.int(52, 84) / 10]);
    }
  }
  const sheets = [
    { name: 'Summary', rows: [
      [{ text: `${vineyard} - 2026 Harvest Log`, bold: true }],
      ['Report generated', { date: generated }],
      ['Vintage', 2026],
      ['Winemaker', winemaker],
      ['Estate acreage harvested', 30.8],
      [],
      ['Variety', 'Tons', 'Bins', 'Picks'].map((text) => ({ text, bold: true })),
      ...[...new Set(blocks.map(([, variety]) => variety))].map((variety) => {
        const rows = picks.filter((row) => row[2] === variety);
        return [variety, Math.round(rows.reduce((total, row) => total + row[4], 0) * 100) / 100, rows.reduce((total, row) => total + row[3], 0), rows.length];
      }),
    ] },
    { name: 'Pick Log', rows: [['Date', 'Block', 'Variety', 'Bins', 'Tons', 'Crew lead', 'Tank'].map((text) => ({ text, bold: true })), ...picks] },
    { name: 'Brix & Chemistry', rows: [['Sample date', 'Block', 'Variety', 'Brix', 'pH', 'TA (g/L)'].map((text) => ({ text, bold: true })), ...chemistry] },
    { name: 'Crew Hours', sharedStrings: false, rows: [['Crew member', 'Hours', 'Days worked'].map((text) => ({ text, bold: true })), ...crews.map((name) => [name, rng.int(60, 190), rng.int(8, 22)])] },
  ];
  const bytes = xlsx({ sheets, title: 'Harvest Log 2026' });
  return result({
    id,
    extension: 'xlsx',
    files: [{ name: `${id}.xlsx`, bytes }],
    text: xlsxText(sheets),
    title: 'Vineyard harvest log workbook: picks, chemistry, and crew hours (Excel)',
    kind: 'log',
    textLayer: 'sheet',
    pages: sheets.length,
    categories: ['unusual', 'xlsx', 'spreadsheet', 'competing_dates', 'table'],
    notes: `An unusual working document: four sheets of a vineyard's harvest. Dozens of pick and sample dates (date cells rendered as ISO dates) run from mid-August through ${lastPick}; the summary sheet's "Report generated" cell (${generated}) is the only date that dates the workbook. A running log can reasonably go to review, so either outcome is acceptable; the first and last pick dates are traps.`,
    gold: gold({
      type: 'Harvest Log',
      acceptableTypes: ['2026 Harvest Log'],
      date: generated,
      role: 'issuance',
      forbiddenDates: [[firstPick, 'first pick of the harvest'], [lastPick, 'last pick of the harvest']],
      parties: [vineyard],
      relation: 'for',
      acceptablePartySets: [{ parties: [vineyard], relation: 'from' }, { parties: [], relation: 'none' }],
      roles: [[vineyard, 'subject'], [vineyard, 'issuer']],
      forbiddenParties: [[winemaker, 'winemaker'], [crews[0], 'crew member']],
      facts: [['2026'], ['harvest', 'Harvest'], ['Pinot Noir', 'Chardonnay', 'Syrah', 'tons']],
      subjectTerms: ['harvest', 'vintage', 'tons', 'Brix'],
      readiness: 'either',
      dateText: [generated],
    }),
  });
}
