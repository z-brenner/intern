/// Letters: an employment offer, a professional engagement letter, and a
/// supplier's credit approval whose sender appears only in the letterhead
/// and whose recipient appears only in the address block.
import { Flow } from '../lib/layout.mjs';
import { gold } from '../lib/gold.mjs';
import { longDate, money } from '../lib/format.mjs';
import { digitalPdf, letterhead, result } from './common.mjs';

export function offerLetter() {
  const id = 'offer-letter';
  const company = 'Corvid Data Systems Inc.';
  const candidate = 'Dalia Brennagh';
  const manager = 'Thaddeus Wainwright';
  const recruiter = 'Greer Nakashima';
  const letterDate = '2026-02-10';
  const start = '2026-03-16';
  const acceptBy = '2026-02-17';
  const vestStart = '2026-03-16';
  const salary = 14200000;
  const bonus = 1000000;
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 54, bottom: 60 }, keep: [company, candidate] });
  flow.y = letterhead(flow.page, { name: company, lines: ['1200 Tidewater Parkway, Halden Bay, WA 98264', 'corviddata.example'], size: 16 });
  flow.paragraph(longDate(letterDate), { after: 12 });
  for (const line of [candidate, '418 Juniper Loop, Apt. 3', 'Halden Bay, WA 98262']) flow.paragraph(line, { after: 0 });
  flow.space(10);
  flow.paragraph('Re: Offer of Employment - Senior Data Engineer', { face: 'sans-bold', size: 10.5 });
  flow.paragraph(`Dear ${candidate}:`);
  flow.paragraph(`On behalf of ${company} (the "Company"), I am delighted to offer you the position of Senior Data Engineer on the Platform Reliability team, reporting to ${manager}, Vice President of Engineering. We were impressed by the streaming pipeline design you presented during your interviews, and we believe you will make an immediate difference to the team.`);
  flow.paragraph('The principal terms of this offer are:', { keepWithNext: 120 });
  flow.table([{ header: 'Term', width: 0.3 }, { header: 'Detail', width: 0.7 }], [
    ['Start date', `${longDate(start)}, or another date we agree in writing`],
    ['Base salary', `${money(salary)} per year, paid semi-monthly, less applicable withholdings`],
    ['Sign-on bonus', `${money(bonus)}, paid with your first regular paycheck; repayable pro rata if you resign within twelve months`],
    ['Annual bonus', 'Target 12% of base salary under the Company Performance Bonus Plan, prorated for 2026'],
    ['Equity', `6,000 restricted stock units, subject to Board approval, vesting over four years from ${longDate(vestStart)} with a one-year cliff`],
    ['Location', 'Hybrid; Halden Bay office Tuesday through Thursday'],
    ['Paid time off', '20 days per year, accruing per pay period, plus 11 Company holidays and 2 floating holidays'],
  ], { size: 9.5 });
  flow.paragraph('Benefits. You will be eligible from your first day for the Company\'s benefit plans on the same terms as other employees, currently including the Meridian PPO 1500 and Meridian HSA 3000 medical plans, Brightline Dental, VisionGuard Plus, basic life and disability insurance, a commuter benefit, and the Corvid 401(k) Savings Plan administered by Saltmarsh Retirement Services, with a Company match of 100% of the first 4% of eligible pay. Plan details are in the enclosed benefits guide; the plans may change from time to time.');
  flow.paragraph('Conditions. This offer is contingent on satisfactory completion of a background check, verification of your authorization to work in the United States on your first day, and your signing the Company\'s Employee Confidentiality and Invention Assignment Agreement, a copy of which is enclosed.');
  flow.paragraph('At-will employment. Your employment with the Company is at will: either you or the Company may end it at any time, with or without cause or notice. Nothing in this letter or in any Company policy changes that, except a written agreement signed by the Chief Executive Officer.');
  flow.paragraph(`To accept, please sign below and return this letter to ${recruiter} by ${longDate(acceptBy)}, after which this offer will expire. We look forward to welcoming you to the team.`);
  flow.paragraph('Sincerely,', { after: 16 });
  flow.paragraph(`/s/ ${recruiter}`, { after: 0 });
  flow.paragraph(`${recruiter}, Senior Talent Partner, on behalf of ${manager}`, { after: 14 });
  flow.paragraph('Accepted and agreed:', { face: 'sans-bold', size: 10 });
  flow.paragraph(`Signature: ____________________________     Name: ${candidate}     Date: ______________`, { size: 10 });
  flow.paragraph('Enclosures: Benefits guide (2026); Employee Confidentiality and Invention Assignment Agreement', { size: 8.5, grey: 0.3 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Employment offer letter for a senior data engineer',
    kind: 'letter',
    textLayer: 'native',
    pages: pages.length,
    categories: ['hr', 'letter', 'competing_dates', 'table', 'irrelevant_names'],
    notes: `The letter is dated ${longDate(letterDate)} on its own line under the letterhead. The start date (${longDate(start)}, also the vesting start) is in the terms table and the acceptance deadline (${longDate(acceptBy)}) closes the letter; both are traps. The benefits paragraph names five plans and a retirement plan administrator, and the hiring manager and recruiter are named - none is a party. The candidate is who the letter is addressed to; filing it "for" her or "from" the company is also accepted.`,
    gold: gold({
      type: 'Offer Letter',
      acceptableTypes: ['Offer of Employment', 'Employment Offer Letter'],
      date: letterDate,
      role: 'issuance',
      forbiddenDates: [[start, 'proposed start date'], [acceptBy, 'acceptance deadline']],
      parties: [candidate],
      relation: 'to',
      acceptablePartySets: [{ parties: [candidate], relation: 'for' }, { parties: [company], relation: 'from' }, { parties: [company, candidate], relation: 'between' }],
      roles: [[candidate, 'recipient'], [candidate, 'subject'], [candidate, 'employee'], [company, 'issuer'], [company, 'employer']],
      forbiddenParties: [[recruiter, 'recruiter who signs'], [manager, 'hiring manager'], ['Saltmarsh Retirement Services', '401(k) administrator named in benefits']],
      facts: [['Senior Data Engineer', 'data engineer'], [money(salary), '$142,000', '142,000'], [company, 'Corvid']],
      subjectTerms: ['offer', 'Senior Data Engineer', 'salary', 'employment'],
      readiness: 'ready',
      dateText: [longDate(letterDate)],
    }),
  });
}

export function engagementLetter() {
  const id = 'engagement-letter';
  const firm = 'Halloran Ostrowski CPAs LLP';
  const client = 'Silverbirch Ceramics Inc.';
  const partner = 'Ines Halloran';
  const clientPerson = 'Joaquin Sandoval';
  const letterDate = '2026-01-09';
  const priorLetter = '2025-01-06';
  const yearEnd = '2025-12-31';
  const fieldworkStart = '2026-02-16';
  const reportBy = '2026-03-27';
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 54, bottom: 64 }, footer: (page, { number, total }) => page.textRight(540, 760, `${number} / ${total}`, { size: 8, face: 'sans', grey: 0.4 }), keep: [firm, client] });
  flow.y = letterhead(flow.page, { name: firm, lines: ['Certified Public Accountants and Advisors', '75 Starling Way, Suite 400, Ashworth, IL 60490', 'Tel. (630) 555-0157'], size: 15, align: 'center' });
  flow.paragraph(longDate(letterDate), { after: 12 });
  for (const line of [clientPerson, 'Chief Financial Officer', client, '1650 Whinstone Road', 'Ashworth, IL 60492']) flow.paragraph(line, { after: 0 });
  flow.space(10);
  flow.paragraph(`Dear Mr. Sandoval:`);
  flow.paragraph(`Thank you for choosing ${firm} ("we" or "the Firm") to serve ${client} (the "Company") again this year. This letter confirms our understanding of the terms and objectives of our engagement and the nature and limitations of the services we will provide. It replaces our engagement letter dated ${longDate(priorLetter)}, which covered the 2024 audit.`);
  flow.heading('Services and objectives', { level: 3 });
  flow.paragraph(`We will audit the Company's balance sheet as of ${longDate(yearEnd)}, and the related statements of income, changes in stockholders' equity, and cash flows for the year then ended, and the related notes, which collectively comprise the financial statements. The objective of our audit is to obtain reasonable assurance about whether the financial statements as a whole are free from material misstatement, whether due to fraud or error, and to issue an auditor's report that includes our opinion.`);
  flow.paragraph('We will also prepare the Company\'s federal corporate income tax return (Form 1120) and the Illinois corporate return for the 2025 tax year, and we will issue a separate letter to management describing any significant deficiencies or material weaknesses in internal control we identify.');
  flow.heading('Management\'s responsibilities', { level: 3 });
  flow.paragraph('Management is responsible for the preparation and fair presentation of the financial statements in accordance with accounting principles generally accepted in the United States; for the design, implementation, and maintenance of internal control relevant to that preparation; for making all financial records and related information available to us; and for providing us with a letter confirming certain representations made during the audit. Management is also responsible for identifying and ensuring compliance with the laws and regulations applicable to the Company\'s activities.');
  flow.heading('Timing and staffing', { level: 3 });
  flow.paragraph(`${partner} is the engagement partner and is responsible for supervising the engagement and signing the report. We expect to begin interim procedures in the last week of January, to begin year-end fieldwork on ${longDate(fieldworkStart)}, and to issue our report on or before ${longDate(reportBy)}, provided the Company's trial balance and supporting schedules are ready when fieldwork begins. Our staff observed the year-end physical inventory counts at the Company's Ashworth and Briarport plants, and we will test the related cost accumulation during fieldwork.`);
  flow.heading('Fees', { level: 3 });
  flow.table([{ header: 'Service', width: 0.65 }, { header: 'Fee', width: 0.35, align: 'right' }], [
    ['Audit of 2025 financial statements', money(6850000)],
    ['Federal and Illinois corporate income tax returns', money(1475000)],
    ['Management letter on internal control', 'Included'],
    [{ text: 'Total fixed fee', face: 'sans-bold' }, { text: money(8325000), face: 'sans-bold' }],
  ], { size: 9.5 });
  flow.paragraph('Our fees are based on the Company\'s records being in the condition described above. We will bill 30% of the fee when fieldwork begins, 50% when we deliver the draft report, and the balance on delivery of the final report and tax returns. Invoices are payable on receipt. If we encounter circumstances that require significant additional time - for example, a change in revenue recognition for the new direct-to-consumer channel - we will discuss it with you before incurring additional fees.');
  flow.heading('Other terms', { level: 3 });
  flow.paragraph('The working papers for this engagement are the property of the Firm and will be retained for seven years. Either party may terminate this engagement on written notice; the Company will pay for work performed through the date of termination. Any dispute arising from this engagement will first be submitted to non-binding mediation in Ashworth, Illinois.');
  flow.paragraph('If this letter correctly expresses your understanding, please sign the enclosed copy where indicated and return it to us. We appreciate the opportunity to continue working with you.');
  flow.paragraph('Very truly yours,', { after: 14 });
  flow.paragraph(`/s/ ${partner}`, { after: 0 });
  flow.paragraph(`${partner}, CPA, Partner, ${firm}`, { after: 16 });
  flow.paragraph('RESPONSE: This letter correctly sets forth the understanding of Silverbirch Ceramics Inc.', { face: 'sans-bold', size: 9.5 });
  flow.paragraph('By: ______________________________   Title: ______________________   Date: ______________', { size: 10 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Audit and tax engagement letter from an accounting firm to its client',
    kind: 'letter',
    textLayer: 'native',
    pages: pages.length,
    categories: ['letter', 'referenced_agreement', 'competing_dates', 'financial'],
    notes: `Dated ${longDate(letterDate)}; it says it replaces the prior year's engagement letter dated ${longDate(priorLetter)} - a document of the same type with a different date, the trap. The audited year ends ${longDate(yearEnd)}, fieldwork begins ${longDate(fieldworkStart)}, and the report is due by ${longDate(reportBy)}. The client's response block is unsigned and undated. An engagement letter is an agreement between firm and client; filing it from the firm or for the client is also accepted.`,
    gold: gold({
      type: 'Engagement Letter',
      acceptableTypes: ['Audit Engagement Letter'],
      date: letterDate,
      role: 'issuance',
      forbiddenDates: [[priorLetter, 'date of the prior engagement letter this one replaces'], [yearEnd, 'balance sheet date being audited'], [fieldworkStart, 'fieldwork start'], [reportBy, 'report deadline']],
      parties: [firm, client],
      relation: 'between',
      acceptablePartySets: [{ parties: [client], relation: 'for' }, { parties: [firm], relation: 'from' }, { parties: [client], relation: 'to' }],
      roles: [[firm, 'firm'], [client, 'client'], [firm, 'issuer'], [client, 'subject'], [client, 'recipient']],
      forbiddenParties: [[partner, 'engagement partner who signs'], [clientPerson, 'client contact the letter is addressed to']],
      facts: [['audit'], [money(8325000), '$83,250', '83,250'], ['2025']],
      subjectTerms: ['audit', 'tax return', 'financial statements', 'engagement'],
      readiness: 'ready',
      dateText: [longDate(letterDate)],
    }),
  });
}

export function letterheadLetter() {
  const id = 'letterhead-letter';
  const sender = 'Moonrake Paper & Packaging Co.';
  const recipient = 'Briarport Coffee Roasters LLC';
  const signer = 'Rosalind Merriweather';
  const attention = 'Keziah Ambrose';
  const letterDate = '2026-03-19';
  const applicationDate = '2026-03-03';
  const reviewDate = '2027-03-31';
  const flow = new Flow({ face: 'serif', fontSize: 11, margins: { top: 54, bottom: 60 } });
  const page = flow.page;
  // Letterhead only: logo block and the company name, nothing in the body names the sender.
  page.rect(72, 50, 34, 34, { fill: 0.2, stroke: null });
  page.rect(80, 58, 18, 18, { fill: 1, stroke: null });
  page.text(116, 66, sender, { face: 'sans-bold', size: 15 });
  page.text(116, 79, 'Corrugated - Folding Cartons - Compostable Bags', { size: 8.5, grey: 0.35 });
  page.text(116, 90, 'Credit Department - 2700 Saltmarsh Road, Bellmoor, WI 53510 - (608) 555-0126', { size: 8.5, grey: 0.35 });
  page.line(72, 100, 540, 100, { width: 0.8, grey: 0.3 });
  flow.y = 124;
  flow.paragraph(longDate(letterDate), { after: 14 });
  for (const line of [recipient, `Attn: ${attention}, Owner`, '310 Harbor Street', 'Briarport, NC 27514']) flow.paragraph(line, { after: 0 });
  flow.space(12);
  flow.paragraph('Re: Approval of Trade Credit Application', { face: 'sans-bold' });
  flow.paragraph(`Dear ${attention}:`);
  flow.paragraph(`Thank you for your letter dated ${longDate(applicationDate)} and the trade credit application, bank reference, and two trade references that accompanied it. We are pleased to tell you that your application has been approved on the terms below.`);
  flow.table([{ header: 'Credit term', width: 0.4 }, { header: 'Approved', width: 0.6 }], [
    ['Credit limit', money(7500000)],
    ['Payment terms', 'Net 30 days from invoice date; 1% discount if paid within 10 days'],
    ['Account number', 'BCR-20418'],
    ['Products covered', 'Stock and custom-printed coffee bags with degassing valves; shipping cartons'],
    ['Annual review', `On or before ${longDate(reviewDate)}`],
  ], { size: 10 });
  flow.paragraph('Orders that would take your outstanding balance above the credit limit will be held until a payment is received or the limit is reviewed. Custom-printed bags require a 50% deposit on the first order of each new design, because the printing plates are made for your artwork alone.');
  flow.paragraph('Please send remittances to the address above, or pay by ACH using the instructions printed on each invoice. If your ownership, legal name, or bank changes, let us know in writing so that we can update your account.');
  flow.paragraph('We appreciate your business and look forward to supplying your new roastery.');
  flow.paragraph('Sincerely,', { after: 16 });
  flow.paragraph(`/s/ ${signer}`, { after: 0 });
  flow.paragraph(`${signer}, Credit Manager`, { after: 0 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Supplier\'s trade-credit approval letter; parties known only from the letterhead and address block',
    kind: 'letter',
    textLayer: 'native',
    pages: pages.length,
    categories: ['layout_parties', 'letter', 'referenced_agreement'],
    notes: `The body says "we" and "you" throughout: the sender is named only in the letterhead and the recipient only in the address block. The letter is dated ${longDate(letterDate)}; it answers "your letter dated ${longDate(applicationDate)}" and sets an annual review by ${longDate(reviewDate)}. Filed from the sender; to (or for) the recipient is also accepted. The credit manager and the recipient's owner are people, not parties.`,
    gold: gold({
      type: 'Credit Approval Letter',
      acceptableTypes: ['Trade Credit Approval', 'Approval of Trade Credit Application', 'Credit Approval'],
      date: letterDate,
      role: 'issuance',
      forbiddenDates: [[applicationDate, 'date of the customer\'s earlier letter'], [reviewDate, 'annual credit review deadline']],
      parties: [sender],
      relation: 'from',
      acceptablePartySets: [{ parties: [recipient], relation: 'to' }, { parties: [recipient], relation: 'for' }],
      roles: [[sender, 'issuer'], [sender, 'seller'], [recipient, 'recipient'], [recipient, 'subject'], [recipient, 'customer']],
      forbiddenParties: [[signer, 'credit manager who signs'], [attention, 'recipient\'s owner named in the attention line']],
      facts: [[money(7500000), '$75,000', '75,000'], ['Net 30'], [recipient, 'Briarport Coffee Roasters']],
      subjectTerms: ['trade credit', 'credit limit', 'approval'],
      readiness: 'ready',
      dateText: [longDate(letterDate)],
    }),
  });
}
