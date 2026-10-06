/// Notices: a residential rent increase, a lender's notice of default, and a
/// private fund's capital call. Each is dated once as a notice and carries
/// several other dates that belong to something else - the agreement it
/// refers to, the deadline it sets, the day the change takes effect.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { longDate, money, percent } from '../lib/format.mjs';
import { digitalPdf, letterhead, result } from './common.mjs';

export function noticeRentIncrease() {
  const id = 'notice-rent-increase';
  const tenant = 'Imogen Castellanos';
  const owner = 'Cresthaven Court Holdings LLC';
  const manager = 'Tamarack Property Management Co.';
  const signer = 'Odalys Fennimore';
  const noticeDate = '2026-05-12';
  const leaseDate = '2024-08-01';
  const effective = '2026-08-01';
  const termEnd = '2026-07-31';
  const vacateBy = '2026-06-30';
  const current = 184500;
  const next = 196500;
  const flow = new Flow({ face: 'serif', fontSize: 11, margins: { top: 54, bottom: 60 }, keep: [longDate(noticeDate), longDate(leaseDate), longDate(effective), longDate(termEnd), longDate(vacateBy), tenant, owner, manager] });
  flow.y = letterhead(flow.page, {
    name: manager,
    lines: ['Leasing and Resident Services', '880 Harrow Lane, Suite 210, Port Alder, OR 97321', 'Tel. (541) 555-0148   residents@tamarack-pm.example'],
    size: 15,
  });
  flow.heading('NOTICE OF RENT INCREASE', { level: 1, align: 'center', size: 14 });
  flow.fields([
    ['Date of Notice:', longDate(noticeDate)],
    ['Resident:', tenant],
    ['Premises:', 'Apartment 4C, 2210 Brackenridge Avenue, Port Alder, OR 97321'],
    ['Owner:', owner],
  ], { labelWidth: 96, size: 10.5, face: 'serif' });
  flow.paragraph(`Re: Residential Lease Agreement dated ${longDate(leaseDate)} for Apartment 4C (the "Lease")`, { face: 'sans-bold', size: 10 });
  flow.paragraph(`Dear ${tenant}:`);
  flow.paragraph(`This letter is written notice, given on behalf of ${owner} as owner of the Premises, that the monthly rent for your apartment will increase. The increase takes effect on ${longDate(effective)}, the first day after your current lease term ends on ${longDate(termEnd)}, and applies to every month of occupancy from that date forward.`);
  flow.table([
    { header: 'Charge', width: 0.46 },
    { header: 'Current monthly amount', width: 0.27, align: 'right' },
    { header: `From ${longDate(effective)}`, width: 0.27, align: 'right' },
  ], [
    ['Base rent', money(current), money(next)],
    ['Assigned parking, space 31', money(7500), money(7500)],
    ['Pet rent (one cat)', money(3500), money(3500)],
    [{ text: 'Total monthly charges', face: 'sans-bold' }, { text: money(current + 11000), face: 'sans-bold' }, { text: money(next + 11000), face: 'sans-bold' }],
  ], { size: 9.5 });
  flow.paragraph(`The base rent increase is ${money(next - current)} per month, or ${percent(Math.round(((next - current) * 10000) / current), 1)}. It reflects the reassessed property tax on the building for the 2026-27 tax year, a 9% increase in the owner's insurance premium, and the cost of the roof replacement completed this spring. Parking and pet rent are unchanged.`);
  flow.paragraph(`If you plan to remain in the apartment, no action is needed: your tenancy will continue month to month at the new rate, and your automatic payment through the resident portal will be updated for the August payment. If you would prefer to sign a new twelve-month lease at the new rate, contact the leasing office and we will prepare one.`);
  flow.paragraph(`If you do not wish to continue your tenancy at the new rate, please give us written notice of your intent to vacate no later than ${longDate(vacateBy)}. Your security deposit of ${money(150000)} will be handled as the Lease provides after you return the keys.`);
  flow.paragraph('This notice is provided at least ninety days before the increase takes effect, as applicable law requires. Nothing in it changes any other term of the Lease.');
  flow.paragraph('Sincerely,', { after: 18 });
  flow.paragraph(signer, { after: 0 });
  flow.paragraph(`Community Manager, ${manager}`, { after: 0 });
  flow.paragraph(`as managing agent for ${owner}`, { after: 10 });
  flow.paragraph('Delivered by first-class mail and by posting to the resident portal. cc: resident file 2210-4C', { size: 8.5, face: 'sans', grey: 0.3 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'One-page notice from a property manager raising a tenant\'s rent',
    kind: 'notice',
    textLayer: 'native',
    pages: pages.length,
    categories: ['simple_digital', 'notice', 'competing_dates', 'referenced_agreement'],
    notes: `The notice is dated ${longDate(noticeDate)}. It also names the lease it acts under (${longDate(leaseDate)} - the same month and day as the increase, two years earlier), the end of the current term (${longDate(termEnd)}), a notice-to-vacate deadline (${longDate(vacateBy)}), and the day the increase takes effect (${longDate(effective)}). The effective date is accepted because the prompt allows a notice to be dated by the change it brings about; the lease date, the term end, and the deadline are traps. The party is the resident the notice is about; the community manager who signs is not a party.`,
    gold: gold({
      type: 'Notice of Rent Increase',
      acceptableTypes: ['Rent Increase Notice'],
      date: noticeDate,
      acceptableDates: [effective],
      role: 'notice',
      forbiddenDates: [[leaseDate, 'date of the lease the notice refers to'], [termEnd, 'end of the current lease term'], [vacateBy, 'deadline to give notice of intent to vacate']],
      parties: [tenant],
      relation: 'for',
      acceptablePartySets: [{ parties: [tenant], relation: 'to' }, { parties: [owner], relation: 'from' }],
      roles: [[tenant, 'subject'], [tenant, 'recipient'], [tenant, 'tenant'], [owner, 'issuer'], [owner, 'landlord']],
      forbiddenParties: [[signer, 'community manager who signs on the owner\'s behalf']],
      facts: [[money(next), '$1,965', '1,965'], [tenant, 'Castellanos'], [longDate(effective), '2026-08-01', 'August 2026']],
      forbiddenFacts: [],
      subjectTerms: ['rent', 'Apartment 4C', 'Brackenridge'],
      readiness: 'ready',
      dateText: [longDate(noticeDate)],
    }),
  });
}

export function noticeOfDefault() {
  const id = 'notice-of-default';
  const rng = Rng.from(id);
  const lender = 'Basalt Commercial Credit Corp.';
  const borrower = 'Glasswing Ceramics LLC';
  const guarantor = 'Hollis Varga';
  const signer = 'Renata Lindqvist';
  const counsel = 'Pemberton & Achterberg LLP';
  const counselPerson = 'Quentin Ollerton';
  const noticeDate = '2025-10-14';
  const loanDate = '2022-03-18';
  const cure = '2025-10-24';
  const dues = ['2025-07-01', '2025-08-01', '2025-09-01'];
  const quarterEnd = '2025-06-30';
  const installment = 1844217;
  const lateCharge = Math.round(installment * 0.05);
  const defaultInterest = 1315000 + rng.int(0, 90000);
  const total = installment * 3 + lateCharge * 3 + defaultInterest;
  const loanNumber = '40-118273';
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 54, bottom: 64 }, keep: [longDate(noticeDate), longDate(loanDate), longDate(cure), ...dues.map(longDate), longDate(quarterEnd), lender, borrower, counsel] });
  flow.y = letterhead(flow.page, {
    name: lender,
    lines: ['Special Assets Group', '1400 Wexley Boulevard, 9th Floor, Linden Cross, PA 19047', 'Tel. (215) 555-0172'],
    size: 15,
  });
  flow.paragraph(longDate(noticeDate), { after: 10 });
  flow.paragraph('VIA OVERNIGHT COURIER AND CERTIFIED MAIL, RETURN RECEIPT REQUESTED', { face: 'sans-bold', size: 9 });
  for (const line of [borrower, `Attn: ${guarantor}, Managing Member`, '5120 Kingsfold Avenue', 'Wrenfield, OH 44012']) flow.paragraph(line, { size: 10, after: 0 });
  flow.space(8);
  flow.paragraph(`With a copy to: ${guarantor}, individually, as Guarantor, 77 Fennel Court, Wrenfield, OH 44016`, { size: 10 });
  flow.heading('NOTICE OF DEFAULT AND RESERVATION OF RIGHTS', { level: 2, align: 'center' });
  flow.paragraph(`Re: Loan No. ${loanNumber} - Loan Agreement dated ${longDate(loanDate)} (as amended, the "Loan Agreement") between ${lender} ("Lender") and ${borrower} ("Borrower"), the related Promissory Note in the original principal amount of ${money(240000000)}, and the Guaranty of ${guarantor}`, { face: 'sans-bold', size: 9.5 });
  flow.paragraph('Ladies and Gentlemen:');
  flow.paragraph(`Lender hereby notifies Borrower that Events of Default have occurred and are continuing under Section 8.1 of the Loan Agreement. Capitalized terms used and not defined in this notice have the meanings given in the Loan Agreement.`);
  flow.paragraph('1. Payment Defaults. Borrower failed to pay the monthly installments of principal and interest that were due on the following dates, and each remains unpaid:', { keepWithNext: 60 });
  flow.table([
    { header: 'Installment due date', width: 0.34 },
    { header: 'Scheduled installment', width: 0.22, align: 'right' },
    { header: 'Late charge (5%)', width: 0.22, align: 'right' },
    { header: 'Days past due', width: 0.22, align: 'right' },
  ], dues.map((due, index) => [longDate(due), money(installment), money(lateCharge), String([105, 74, 43][index])]), { size: 9.5 });
  flow.paragraph(`2. Financial Covenant Default. Borrower's Debt Service Coverage Ratio for the fiscal quarter ended ${longDate(quarterEnd)} was 0.94 to 1.00, below the minimum of 1.25 to 1.00 required by Section 6.12, and Borrower did not deliver the compliance certificate for that quarter within forty-five days after its end as Section 6.1(c) requires.`);
  flow.paragraph('3. Amounts Due. As of the date of this notice, the following amounts are past due:', { keepWithNext: 70 });
  flow.table([
    { header: 'Item', width: 0.7 },
    { header: 'Amount', width: 0.3, align: 'right' },
  ], [
    ['Past-due installments (3)', money(installment * 3)],
    ['Late charges', money(lateCharge * 3)],
    ['Default interest at the Default Rate (2.00% above the Note Rate) accrued through the date of this notice', money(defaultInterest)],
    [{ text: 'Total past due', face: 'sans-bold' }, { text: money(total), face: 'sans-bold' }],
  ], { size: 9.5 });
  flow.paragraph(`4. Demand to Cure. Lender demands that Borrower pay the total past-due amount of ${money(total)}, together with Lender's reasonable attorneys' fees incurred to date, within ten (10) days after the date of this notice, that is, on or before ${longDate(cure)}. Payment must be made by wire transfer of immediately available funds in accordance with the payment instructions previously provided; Lender will not accept partial payment as a cure.`);
  flow.paragraph(`5. Remedies. If the Events of Default are not cured by ${longDate(cure)}, Lender may, without further notice, declare the entire unpaid principal balance of the Loan, all accrued interest, and all other Obligations immediately due and payable; enforce its security interest in the Collateral, including the kilns, glazing lines, and inventory located at Borrower's Wrenfield facility; and proceed against the Guarantor under the Guaranty.`);
  flow.paragraph('6. Reservation of Rights. Lender expressly reserves all of its rights and remedies under the Loan Documents and applicable law. No delay or forbearance by Lender, no acceptance of any partial payment, and no discussion between Lender and Borrower regarding a possible restructuring shall waive any Event of Default or constitute a course of dealing, and any agreement to modify the Loan Documents must be in a writing signed by Lender.');
  flow.paragraph(`Please direct any questions about this notice to the undersigned at (215) 555-0172 or to Lender's counsel, ${counselPerson} of ${counsel}.`);
  flow.paragraph('Very truly yours,', { after: 4 });
  flow.paragraph(lender.toUpperCase(), { face: 'sans-bold', size: 10, after: 16 });
  flow.paragraph(`By: /s/ ${signer}`, { after: 0 });
  flow.paragraph(`${signer}, Senior Vice President, Special Assets`, { after: 10 });
  flow.paragraph(`cc: ${counselPerson}, Esq., ${counsel} (by email)`, { size: 9, grey: 0.2 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Lender\'s notice of payment and covenant defaults to a commercial borrower',
    kind: 'notice',
    textLayer: 'native',
    pages: pages.length,
    categories: ['notice', 'financial', 'competing_dates', 'referenced_agreement', 'table'],
    notes: `Dated ${longDate(noticeDate)} at the top of the letter, with no "Date:" label. The loan agreement's date (${longDate(loanDate)}) is repeated in the reference line, three missed installment due dates sit in a table, the covenant test period ends ${longDate(quarterEnd)}, and the cure deadline (${longDate(cure)}) is stated twice - all traps. The notice is about the borrower; the lender sends it, so "from" the lender is also accepted. The guarantor is addressed too but is not who the notice is about, and the lender's officer and outside counsel are not parties.`,
    gold: gold({
      type: 'Notice of Default',
      acceptableTypes: ['Notice of Default and Reservation of Rights'],
      date: noticeDate,
      role: 'notice',
      forbiddenDates: [[loanDate, 'date of the loan agreement the notice refers to'], [cure, 'cure deadline'], ...dues.map((due) => [due, 'missed installment due date']), [quarterEnd, 'end of the covenant test quarter']],
      parties: [borrower],
      relation: 'for',
      acceptablePartySets: [{ parties: [borrower], relation: 'to' }, { parties: [lender], relation: 'from' }],
      roles: [[borrower, 'subject'], [borrower, 'recipient'], [borrower, 'borrower'], [lender, 'issuer'], [lender, 'lender']],
      forbiddenParties: [[signer, 'lender officer who signs'], [counsel, 'lender\'s outside counsel, copied'], [counselPerson, 'lender\'s counsel, copied']],
      facts: [[money(total), money(total).slice(1)], [borrower, 'Glasswing'], [loanNumber, 'Loan No']],
      forbiddenFacts: [],
      subjectTerms: ['default', 'past-due', 'cure', 'loan'],
      readiness: 'ready',
      dateText: [longDate(noticeDate)],
    }),
  });
}

export function capitalCallNotice() {
  const id = 'capital-call-notice';
  const fund = 'Ravensmoor Growth Partners III, L.P.';
  const generalPartner = 'Ravensmoor Growth GP III, LLC';
  const investor = 'Fairbanks Family Foundation';
  const portfolio = ['Starling Telemetry Inc.', 'Oldcastle Water Systems Corporation'];
  const signer = 'Leopold Dahlquist';
  const noticeDate = '2026-02-06';
  const dueDate = '2026-02-20';
  const lpaDate = '2023-06-30';
  const closing = '2026-02-27';
  const commitment = 500000000;
  const callPercent = 420; // 4.20% in hundredths of a percent
  const call = (commitment * callPercent) / 10000;
  const investmentShare = Math.round(call * 0.81);
  const feeShare = Math.round(call * 0.15);
  const expenseShare = call - investmentShare - feeShare;
  const contributedBefore = Math.round(commitment * 0.4635);
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 54, bottom: 64 }, keep: [longDate(noticeDate), longDate(dueDate), longDate(lpaDate), longDate(closing), fund, investor, generalPartner] });
  flow.y = letterhead(flow.page, {
    name: fund,
    lines: [`c/o ${generalPartner}`, '300 Oriel Junction Plaza, Suite 1800, Oriel Junction, MO 64101', 'investor.relations@ravensmoor-gp.example'],
    size: 14,
  });
  flow.heading('CAPITAL CALL NOTICE No. 7', { level: 1, align: 'center', size: 14, after: 2 });
  flow.paragraph('CONFIDENTIAL - FOR THE ADDRESSEE ONLY', { align: 'center', face: 'sans', size: 8.5, grey: 0.3 });
  flow.fields([
    ['Notice date:', longDate(noticeDate)],
    ['Limited Partner:', investor],
    ['Investor ID:', 'RGP3-0148'],
    ['Capital Commitment:', money(commitment)],
    ['Funding due date:', `${longDate(dueDate)}, by 2:00 p.m. Central Time`],
  ], { labelWidth: 120, size: 10, face: 'serif' });
  flow.paragraph(`Pursuant to Section 4.2 of the Amended and Restated Agreement of Limited Partnership of ${fund} dated ${longDate(lpaDate)} (the "Partnership Agreement"), ${generalPartner}, as general partner of the Partnership, hereby calls for a capital contribution from each Limited Partner equal to ${percent(callPercent)} of its Capital Commitment. Your share of this call is set out below.`);
  flow.table([
    { header: 'Purpose of call', width: 0.62 },
    { header: 'Your share', width: 0.38, align: 'right' },
  ], [
    [`New investment: Series B preferred stock of ${portfolio[0]}, closing expected ${longDate(closing)}`, money(investmentShare)],
    ['Management fee for the quarter beginning January 1, 2026 (Section 5.1)', money(feeShare)],
    ['Partnership expenses: fund administration, audit, and legal (Section 5.3)', money(expenseShare)],
    [{ text: 'Total amount due from you', face: 'sans-bold' }, { text: money(call), face: 'sans-bold' }],
  ], { size: 9.5 });
  flow.table([
    { header: 'Your capital account summary', width: 0.62 },
    { header: 'Amount', width: 0.38, align: 'right' },
  ], [
    ['Capital Commitment', money(commitment)],
    ['Contributed through prior calls (Nos. 1-6)', money(contributedBefore)],
    ['This call (No. 7)', money(call)],
    ['Cumulative contributions after this call', money(contributedBefore + call)],
    ['Remaining Unfunded Commitment', money(commitment - contributedBefore - call)],
    ['Cumulative contributions as a percentage of Capital Commitment', percent(Math.round(((contributedBefore + call) * 10000) / commitment))],
  ], { size: 9.5 });
  flow.paragraph(`Please wire your contribution in immediately available funds so that it is received no later than ${longDate(dueDate)}. Wire instructions are unchanged from Capital Call No. 6: Linden Cross Trust Company, for the account of ${fund}, account ending 4471, reference "RGP3-0148 Call 7". The general partner will never change wire instructions by email; confirm any change by telephone with Investor Relations before sending funds.`);
  flow.paragraph(`A Limited Partner that fails to fund by the due date is subject to the default provisions of Section 4.5 of the Partnership Agreement, including interest at the Default Rate from the due date until paid. The investment in ${portfolio[0]} is the Partnership's ninth platform investment; a summary of the company and the investment thesis is included in the enclosed quarterly letter. The Partnership also expects to fund a follow-on investment in ${portfolio[1]} in the second quarter, for which a separate notice will be sent.`);
  flow.paragraph('Sincerely,', { after: 4 });
  flow.paragraph(`${generalPartner}, its general partner`, { face: 'sans-bold', size: 10, after: 14 });
  flow.paragraph(`By: /s/ ${signer}`, { after: 0 });
  flow.paragraph(`${signer}, Chief Financial Officer`, { after: 10 });
  flow.paragraph('Enclosures: Fourth Quarter 2025 Letter to Limited Partners; Wire confirmation form', { size: 9, grey: 0.2 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Private equity fund capital call notice to a limited partner',
    kind: 'notice',
    textLayer: 'native',
    pages: pages.length,
    categories: ['financial', 'notice', 'referenced_agreement', 'competing_dates', 'table'],
    notes: `The notice date (${longDate(noticeDate)}) is a labelled field; the funding due date (${longDate(dueDate)}) is labelled just below it and repeated in the wire paragraph, the partnership agreement is dated ${longDate(lpaDate)}, and the portfolio closing is expected ${longDate(closing)}. Only the notice date is right. A capital call is filed by the fund that issued it; naming the limited partner it is addressed to is also accepted. Portfolio companies and the signing officer are not parties.`,
    gold: gold({
      type: 'Capital Call Notice',
      acceptableTypes: ['Capital Call'],
      date: noticeDate,
      role: 'notice',
      forbiddenDates: [[dueDate, 'funding due date'], [lpaDate, 'date of the partnership agreement'], [closing, 'expected closing of the portfolio investment']],
      parties: [fund],
      relation: 'from',
      acceptablePartySets: [{ parties: [investor], relation: 'for' }, { parties: [investor], relation: 'to' }],
      roles: [[fund, 'issuer'], [fund, 'fund'], [investor, 'subject'], [investor, 'recipient'], [investor, 'investor']],
      forbiddenParties: [[portfolio[0], 'portfolio company the called capital will buy'], [portfolio[1], 'portfolio company of a future call'], [signer, 'officer who signs']],
      facts: [[money(call), money(call).slice(1), '$210,000'], [investor, 'Fairbanks'], ['No. 7', 'Call 7', 'seventh']],
      forbiddenFacts: [],
      subjectTerms: ['capital call', 'contribution', portfolio[0].split(' ')[0]],
      readiness: 'ready',
      dateText: [longDate(noticeDate)],
    }),
  });
}
