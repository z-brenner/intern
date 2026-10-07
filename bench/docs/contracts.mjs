/// Contracts: a master services agreement, a second amendment, a statement
/// of work issued under a master agreement, an assignment of a lease, a
/// two-column commercial lease, and a promissory note. Each states its own
/// date once or twice among several dates that belong to other documents
/// or to later events.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { addMonths, decimal, longDate, money, numericDate } from '../lib/format.mjs';
import { COMMERCIAL, writeClauses } from './clauses.mjs';
import { digitalPdf, result, signatureBlocks } from './common.mjs';

function pageFooter(label) {
  return (page, { number, total }) => {
    page.text(72, 760, label, { face: 'sans', size: 7.5, grey: 0.35 });
    page.textRight(540, 760, `Page ${number} of ${total}`, { face: 'sans', size: 7.5, grey: 0.35 });
  };
}

/// Which page of `pages` (1-based) first contains `needle` in its text.
export function pageContaining(text, needle) {
  const index = text.findIndex((page) => page.replace(/\s+/g, ' ').includes(needle));
  if (index < 0) throw new Error(`no page contains ${JSON.stringify(needle)}`);
  return index + 1;
}

export function servicesAgreement() {
  const id = 'services-agreement';
  const rng = Rng.from(id);
  const provider = 'Cedarmark Cloud Services Inc.';
  const customer = 'Pinehollow Credit Union';
  const effective = '2026-01-12';
  const ndaDate = '2025-10-03';
  const signedProvider = '2026-01-08';
  const signedCustomer = '2026-01-09';
  const people = { providerSigner: 'Amara Rehnquist', customerSigner: 'Wendell Coldwell', providerNotice: 'General Counsel', customerNotice: 'Chief Operating Officer' };
  const flow = new Flow({ face: 'serif', fontSize: 10, margins: { top: 66, bottom: 66 }, footer: pageFooter('Cedarmark / Pinehollow - Master Services Agreement - Confidential'), keep: [provider, customer] });
  flow.heading('MASTER SERVICES AGREEMENT', { level: 1, align: 'center', size: 15 });
  flow.paragraph(`This Master Services Agreement (this "Agreement") is entered into as of ${longDate(effective)} (the "Effective Date") by and between ${provider}, a Delaware corporation with its principal office at 61 Basalt Drive, Kestrel Ridge, UT 84047 ("Provider"), and ${customer}, a state-chartered credit union with its principal office at 400 Old Mill Road, Bellmoor, WI 53511 ("Customer").`);
  flow.heading('Recitals', { level: 3 });
  flow.paragraph(`A. Provider operates managed cloud hosting, database administration, and security monitoring services for regulated financial institutions. Customer is migrating its member-facing digital banking platform and its loan origination system from an on-premises data center to hosted infrastructure.`);
  flow.paragraph(`B. The parties exchanged information about Customer's systems under a Mutual Nondisclosure Agreement dated ${longDate(ndaDate)} (the "NDA"). From the Effective Date, Section 8 of this Agreement governs information exchanged between the parties, and the NDA continues to govern information exchanged before it.`);
  flow.paragraph('C. The parties wish to set out the terms on which Provider will perform services for Customer from time to time under statements of work.');
  flow.paragraph('NOW, THEREFORE, in consideration of the mutual promises below, the parties agree as follows:');
  flow.heading('1. Definitions', { level: 3 });
  for (const [term, meaning] of [
    ['Affiliate', 'an entity that controls, is controlled by, or is under common control with a party, where control means ownership of more than fifty percent of the voting interests of the entity'],
    ['Business Day', 'a day other than a Saturday, Sunday, or day on which federally insured depository institutions in Wisconsin are authorized or required to close'],
    ['Customer Data', 'all data, including member personal information and account records, that Customer or its members provide to Provider or that Provider processes on Customer\'s behalf'],
    ['Hosted Environment', 'the computing, storage, and network resources Provider operates or procures to deliver the Services, including the primary site in Silverlode, NV and the recovery site in Quarry Bend, TN'],
    ['Service Levels', 'the availability, response, and restoration commitments in Exhibit A'],
    ['Services', 'the services Provider performs under a Statement of Work, including the Deliverables'],
  ]) flow.paragraph([{ text: `"${term}" `, face: 'sans-bold' }, { text: `means ${meaning}.` }], { indent: 18 });
  const ctx = {
    rng: rng.fork('clauses'), a: 'Provider', b: 'Customer', aName: provider, bName: customer,
    aAddress: '61 Basalt Drive, Kestrel Ridge, UT 84047', bAddress: '400 Old Mill Road, Bellmoor, WI 53511',
    aNotice: people.providerNotice, bNotice: people.customerNotice, state: 'Wisconsin', venue: 'Dane County, Wisconsin',
  };
  const next = writeClauses(flow, COMMERCIAL, ctx, { start: 2 });
  flow.paragraph([{ text: `${next}. Regulatory Cooperation. `, face: 'sans-bold' }, { text: 'Provider acknowledges that Customer is subject to examination by its prudential and consumer protection regulators. Provider will make its personnel, facilities, and records relating to the Services available to those regulators on request, will maintain a SOC 2 Type II report covering the Hosted Environment renewed at least annually, and will notify Customer within five Business Days of any change in its subcontracted data centers.' }]);
  flow.paragraph('IN WITNESS WHEREOF, the parties have executed this Agreement as of the Effective Date.', { before: 6 });
  signatureBlocks(flow, [
    { heading: 'PROVIDER', entity: provider.toUpperCase(), name: people.providerSigner, title: 'Chief Executive Officer', date: longDate(signedProvider) },
    { heading: 'CUSTOMER', entity: customer.toUpperCase(), name: people.customerSigner, title: 'President and Chief Executive Officer', date: longDate(signedCustomer) },
  ]);
  flow.heading('Exhibit A - Service Levels', { level: 2 });
  flow.table([
    { header: 'Service', width: 0.34 },
    { header: 'Commitment', width: 0.38 },
    { header: 'Service credit', width: 0.28 },
  ], [
    ['Digital banking platform availability', '99.95% per calendar month, excluding scheduled maintenance', '5% of monthly fee per 0.1% below commitment, up to 30%'],
    ['Loan origination system availability', '99.9% per calendar month during 6:00 a.m. - 10:00 p.m. Central', '5% of monthly fee per 0.1% below commitment, up to 20%'],
    ['Severity 1 incident response', '15 minutes, 24 x 7', '2% of monthly fee per missed response'],
    ['Severity 2 incident response', '1 hour, 24 x 7', '1% of monthly fee per missed response'],
    ['Recovery time objective (disaster)', '4 hours', 'Credit as agreed in the disaster recovery plan'],
    ['Recovery point objective (disaster)', '15 minutes of data', 'Credit as agreed in the disaster recovery plan'],
    ['Security event notification', 'Within 24 hours of confirmation', 'Not credit-bearing; material breach if missed twice'],
  ], { size: 9 });
  flow.paragraph('Scheduled maintenance may occur only between 11:00 p.m. Saturday and 5:00 a.m. Sunday Central time, on at least five Business Days\' notice, and may not exceed eight hours in any calendar month. Service credits are Customer\'s sole monetary remedy for a failure to meet a Service Level, but do not limit Customer\'s right to terminate for chronic failure, defined as missing the same availability commitment in three months of any rolling six-month period.', { size: 9.5 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Master services agreement between a cloud hosting provider and a credit union',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['contract', 'referenced_agreement', 'competing_dates'],
    notes: `The effective date (${longDate(effective)}) is stated once, in the opening paragraph on page 1. The recitals name an earlier NDA dated ${longDate(ndaDate)}, and the signature blocks on the last page of the body carry two signing dates (${longDate(signedProvider)} and ${longDate(signedCustomer)}) that precede the effective date; both are traps under the prompt's rule that a stated effective date beats a signing date. The rest is ${pages.length - 1} pages of operative clauses with their own numbers and notice periods.`,
    gold: gold({
      type: 'Master Services Agreement',
      date: effective,
      role: 'effective',
      forbiddenDates: [[ndaDate, 'date of the earlier NDA the recitals mention'], [signedProvider, 'provider\'s signature date'], [signedCustomer, 'customer\'s signature date']],
      parties: [provider, customer],
      relation: 'between',
      roles: [[provider, 'seller'], [customer, 'client']],
      forbiddenParties: [[people.providerSigner, 'signatory'], [people.customerSigner, 'signatory']],
      facts: [['Cedarmark Cloud Services Inc.', 'Cedarmark'], [customer, 'Pinehollow']],
      subjectTerms: ['managed cloud', 'hosting', 'digital banking', 'service levels'],
      readiness: 'ready',
      dateText: [longDate(effective)],
    }),
  });
}

export function secondAmendment() {
  const id = 'second-amendment';
  const licensor = 'Umberlee Imaging Software Inc.';
  const licensee = 'Fallowmere Regional Health System';
  const own = '2025-09-29';
  const original = '2021-04-30';
  const first = '2023-02-14';
  const feeChange = '2026-01-01';
  const newEnd = '2028-12-31';
  const oldEnd = '2026-04-29';
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 66, bottom: 66 }, footer: pageFooter('Second Amendment - Umberlee / Fallowmere'), keep: [licensor, licensee] });
  const agreement = `Software License and Support Agreement dated ${longDate(original)}`;
  flow.heading('SECOND AMENDMENT TO SOFTWARE LICENSE AND SUPPORT AGREEMENT', { level: 1, align: 'center', size: 13 });
  flow.paragraph(`This Second Amendment (this "Second Amendment") is entered into as of ${longDate(own)} by and between ${licensor}, a Massachusetts corporation ("Licensor"), and ${licensee}, an Ohio nonprofit corporation ("Licensee"), and amends the ${agreement} between Licensor and Licensee, as amended by the First Amendment dated ${longDate(first)} (as so amended, the "Agreement").`);
  flow.heading('Recitals', { level: 3 });
  flow.paragraph(`A. Under the ${agreement}, Licensor licensed to Licensee its PACSView enterprise imaging platform for use at Licensee's three hospitals and eleven outpatient imaging centers.`);
  flow.paragraph(`B. The First Amendment dated ${longDate(first)} added the cardiology and dental imaging modules and increased the licensed study volume to 410,000 studies per year.`);
  flow.paragraph('C. Licensee is opening a fourth hospital and wishes to license the artificial intelligence triage module and the cloud archive, and the parties wish to extend the term and revise the fees, on the terms below.');
  flow.paragraph('The parties therefore agree as follows:');
  const sections = [
    ['Additional Modules', `Schedule A of the Agreement (Licensed Software) is amended by adding the modules listed in the table below. The added modules are "Licensed Software" for all purposes of the Agreement, are licensed for the same facilities and on the same terms as the existing modules, and are covered by the support and maintenance obligations of Section 9.`],
  ];
  flow.paragraph([{ text: `1. ${sections[0][0]}. `, face: 'sans-bold' }, { text: sections[0][1] }]);
  flow.table([
    { header: 'Module', width: 0.4 },
    { header: 'Licensed metric', width: 0.32 },
    { header: 'Annual fee', width: 0.28, align: 'right' },
  ], [
    ['PACSView AI Triage (stroke, pulmonary embolism, fracture)', 'Up to 180,000 studies per year', money(9600000)],
    ['PACSView Cloud Archive', 'Up to 1.4 petabytes stored', money(11250000)],
    ['Zero-footprint referring physician viewer', 'Unlimited named users', money(2400000)],
  ], { size: 9.5 });
  flow.paragraph([{ text: '2. Additional Facility. ', face: 'sans-bold' }, { text: 'Schedule B of the Agreement (Licensed Facilities) is amended by adding Fallowmere North Hospital, 2200 Tidewater Parkway, Wrenfield, OH 44011. Licensor will complete installation and interface testing at that facility before its scheduled opening, and Licensee will provide the network access and hardware described in the implementation plan attached as Exhibit 1.' }]);
  flow.paragraph([{ text: '3. Term. ', face: 'sans-bold' }, { text: `Section 12.1 of the Agreement is amended to replace the expiration date of ${longDate(oldEnd)} with ${longDate(newEnd)}. After that date the Agreement renews for successive one-year terms unless either party gives notice of non-renewal at least one hundred eighty (180) days before the end of the then-current term.` }]);
  flow.paragraph([{ text: '4. Fees. ', face: 'sans-bold' }, { text: `Beginning ${longDate(feeChange)}, the annual license and support fee under Section 6.1 is ${money(41250000)}, inclusive of the modules added by this Second Amendment, payable annually in advance. The annual fee may increase on each anniversary of that date by no more than the lesser of four percent (4%) and the change in the Consumer Price Index for the preceding twelve months. Fees for the period before ${longDate(feeChange)} remain as stated in the Agreement.` }]);
  flow.paragraph([{ text: '5. Service Levels for the Cloud Archive. ', face: 'sans-bold' }, { text: 'Licensor will make the Cloud Archive available 99.95% of each calendar month and will retrieve any archived study within ninety seconds of request. Studies will be stored in two geographically separate data centers in the United States, encrypted with keys that Licensee may rotate on demand.' }]);
  flow.paragraph([{ text: '6. Effect of Amendment. ', face: 'sans-bold' }, { text: 'Except as expressly amended by this Second Amendment, the Agreement remains unchanged and in full force and effect. References in the Agreement to "this Agreement" mean the Agreement as amended by this Second Amendment. If this Second Amendment conflicts with the Agreement, this Second Amendment controls.' }]);
  flow.paragraph([{ text: '7. Counterparts. ', face: 'sans-bold' }, { text: 'This Second Amendment may be executed in counterparts and by electronic signature, each of which is an original and all of which together are one instrument.' }]);
  flow.paragraph('IN WITNESS WHEREOF, the parties have caused this Second Amendment to be executed by their duly authorized representatives.', { before: 6 });
  signatureBlocks(flow, [
    { heading: 'LICENSOR', entity: licensor.toUpperCase(), name: 'Mireille Kowalczyk', title: 'Chief Revenue Officer' },
    { heading: 'LICENSEE', entity: licensee.toUpperCase(), name: 'Darius Seabrook', title: 'Chief Information Officer' },
  ]);
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Second amendment to a hospital imaging software license',
    kind: 'amendment',
    textLayer: 'native',
    pages: pages.length,
    categories: ['amendment', 'contract', 'referenced_agreement', 'competing_dates', 'table'],
    notes: `The amendment's own date (${longDate(own)}) appears exactly once, in the first sentence. The original agreement (${longDate(original)}) and the First Amendment (${longDate(first)}) are each named twice, and the new fee start (${longDate(feeChange)}, "Beginning ...") and the new and old expiration dates are traps too. Signature blocks are undated.`,
    gold: gold({
      type: 'Second Amendment to Software License and Support Agreement',
      acceptableTypes: ['Second Amendment to Software License Agreement'],
      date: own,
      role: 'amendment',
      forbiddenDates: [[original, 'date of the original license agreement'], [first, 'date of the first amendment'], [feeChange, 'date the new fees begin'], [newEnd, 'new expiration date'], [oldEnd, 'old expiration date being replaced']],
      parties: [licensor, licensee],
      relation: 'between',
      roles: [[licensor, 'licensor'], [licensee, 'licensee']],
      forbiddenParties: [['Mireille Kowalczyk', 'signatory'], ['Darius Seabrook', 'signatory']],
      facts: [['AI Triage', 'triage', 'Cloud Archive'], [longDate(newEnd), '2028'], [money(41250000), '412,500']],
      subjectTerms: ['imaging', 'license', 'modules', 'term'],
      readiness: 'ready',
      dateText: [longDate(own)],
    }),
  });
}

export function sowUnderMsa() {
  const id = 'sow-under-msa-5p';
  const provider = 'Thornbury Data Labs LLC';
  const client = 'Saltmarsh Regional Water Authority';
  const msaDate = '2024-10-07';
  const effective = '2026-05-26';
  const completion = '2026-11-30';
  const signedProvider = '2026-06-01';
  const signedClient = '2026-06-03';
  const milestones = [
    ['M1', 'Discovery report and data inventory', '2026-06-26', 3800000],
    ['M2', 'Meter data platform configured in test', '2026-08-07', 6200000],
    ['M3', 'Leak-detection models validated on 2025 data', '2026-09-18', 5400000],
    ['M4', 'Production cut-over and operator training', '2026-10-30', 4900000],
    ['M5', 'Hypercare complete; final documentation', completion, 2100000],
  ];
  const fee = milestones.reduce((sum, row) => sum + row[3], 0);
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 66, bottom: 66 }, footer: pageFooter(`SOW No. 3 under MSA dated ${longDate(msaDate)}`), keep: [provider, client] });
  flow.heading('STATEMENT OF WORK No. 3', { level: 1, align: 'center', size: 15, after: 2 });
  flow.paragraph('Advanced Metering Analytics and Leak Detection', { align: 'center', face: 'sans', size: 11, after: 10 });
  flow.paragraph(`This Statement of Work No. 3 ("SOW") is issued under, and is governed by, the Master Services Agreement dated ${longDate(msaDate)} (the "MSA") between ${provider} ("Provider") and ${client} ("Client"). Capitalized terms used but not defined in this SOW have the meanings given in the MSA.`);
  flow.heading('1. Background', { level: 3 });
  flow.paragraph('Client supplies drinking water to approximately 212,000 customers through 1,940 miles of distribution main. Between 2023 and 2025 Client replaced 96% of its customer meters with cellular advanced metering infrastructure (AMI) meters that report hourly consumption. Client\'s non-revenue water averaged 17.8% of production in fiscal year 2025, against a target of 12% by fiscal year 2028. Client wants to use the hourly meter data, together with pressure and flow data from its 61 district metered areas, to find leaks earlier and to prioritize main replacement.');
  flow.heading('2. Scope of Services', { level: 3 });
  for (const [label, body] of [
    ['2.1 Discovery.', 'Provider will interview Client\'s operations, engineering, customer service, and IT staff; inventory the AMI head-end, SCADA historian, GIS, work order, and billing data sources; and document data quality issues, including meters with stale reads, reversed flow, and register rollovers.'],
    ['2.2 Meter Data Platform.', 'Provider will configure a meter data analytics platform in Client\'s cloud tenancy that ingests hourly reads from the AMI head-end and five-minute pressure and flow data from the SCADA historian, validates and estimates missing reads using Client\'s approved rules, and retains raw and validated data for seven years.'],
    ['2.3 Leak Detection Models.', 'Provider will develop and validate models that flag continuous customer-side leaks (24 or more consecutive hours of non-zero consumption), district-level water balance anomalies, and pressure transients associated with main breaks. Models will be validated against Client\'s 2025 work orders and confirmed leak repairs.'],
    ['2.4 Operator Workflows.', 'Provider will build dashboards and alert queues for Client\'s customer service and distribution operations teams, integrate alerts with Client\'s work order system so that a confirmed alert opens a work order, and configure customer leak notifications by email and text message using Client\'s approved templates.'],
    ['2.5 Training and Transition.', 'Provider will deliver four instructor-led training sessions of up to twelve participants each, recorded for later use, and will provide thirty days of hypercare support after production cut-over.'],
  ]) flow.paragraph([{ text: `${label} `, face: 'sans-bold' }, { text: body }]);
  flow.paragraph('The data sources in scope, and the access Provider needs to each, are:', { keepWithNext: 60 });
  flow.table([
    { header: 'Source system', width: 0.3 },
    { header: 'Content', width: 0.36 },
    { header: 'Volume / frequency', width: 0.2 },
    { header: 'Access', width: 0.14 },
  ], [
    ['AMI head-end', 'Hourly register reads, alarms (tamper, leak, reverse flow)', '~5.1 million reads per day', 'API, read-only'],
    ['SCADA historian', 'Pressure, flow, and tank level at 61 district meters and 14 pump stations', '5-minute samples', 'ODBC replica'],
    ['GIS', 'Mains, valves, hydrants, service lines, district boundaries', 'Weekly export', 'File share'],
    ['Work order system', 'Leak repairs, main breaks, meter exchanges since 2019', '~38,000 orders', 'Reporting database'],
    ['Customer information system', 'Accounts, premises, contact preferences, billing consumption', 'Nightly extract', 'SFTP'],
    ['Weather service feed', 'Daily minimum temperature and precipitation for frost and drought correlation', 'Daily', 'Public API'],
  ], { size: 8.5 });
  flow.paragraph('Provider will not write to any source system. Where a source system cannot supply data in the frequency shown, Provider will document the gap in the discovery report and propose an alternative, and the parties will agree any resulting change to the schedule by Change Order.');
  flow.heading('3. Deliverables', { level: 3 });
  flow.table([
    { header: 'No.', width: 0.08 },
    { header: 'Deliverable', width: 0.52 },
    { header: 'Acceptance criteria', width: 0.4 },
  ], [
    ['D1', 'Discovery report, data inventory, and data quality findings', 'Covers all sources in 2.1; reviewed by Client\'s Director of Operations'],
    ['D2', 'Configured meter data platform (test and production)', 'Ingests 30 consecutive days of reads with < 0.5% unexplained gaps'],
    ['D3', 'Leak detection models with validation report', 'Detects at least 70% of confirmed 2025 customer-side leaks with <= 15% false positives'],
    ['D4', 'Dashboards, alert queues, and work order integration', 'Passes the user acceptance test script agreed in M2'],
    ['D5', 'Training materials, recordings, and run book', 'Delivered in editable form; run book approved by Client IT'],
  ], { size: 9 });
  flow.heading('4. Assumptions and Client Responsibilities', { level: 3 });
  for (const item of [
    '(a) Client will provide read-only access to the AMI head-end, SCADA historian, GIS, and work order databases within ten business days after the SOW Effective Date.',
    '(b) Client will designate a project manager with authority to make day-to-day decisions and will make subject matter experts available for up to six hours per week each.',
    '(c) Client\'s cloud tenancy has sufficient quota for the platform; cloud consumption charges are paid directly by Client and are not part of the fees in this SOW.',
    '(d) Customer notification templates will be approved by Client\'s communications department; Provider is not responsible for the content of customer communications.',
    '(e) Pressure logger installation, if Client decides more loggers are needed, is outside the scope of this SOW and will be addressed by a Change Order.',
    '(f) Client will make field crews available to verify a sample of at least 150 model alerts during validation, so that model accuracy can be measured against confirmed leaks rather than estimated.',
  ]) flow.paragraph(item, { indent: 14 });
  flow.heading('5. Project Governance', { level: 3 });
  flow.paragraph('The parties will hold a weekly status meeting and a monthly steering committee meeting. Provider will circulate a written status report two business days before each steering committee meeting covering progress against milestones, open risks and issues, decisions needed, and hours consumed against budget. Decisions of the steering committee will be recorded in the status report and are binding on the project teams unless they change scope, fees, or schedule, in which case a Change Order is required.');
  flow.paragraph('Each party will escalate any issue that threatens a milestone date by more than ten business days to its steering committee members within two business days after identifying it. Provider will maintain a risk register with an owner, likelihood, impact, and mitigation for each identified risk, and will review it with Client at each steering committee meeting.');
  flow.heading('6. Term', { level: 3 });
  flow.paragraph(`6.1 This SOW is effective as of ${longDate(effective)} (the "SOW Effective Date") and continues until Client accepts the final milestone or ${longDate(completion)}, whichever is later, unless terminated earlier under the MSA.`);
  flow.paragraph('6.2 Provider will perform the services according to the milestone schedule in Section 7. A milestone date that slips because Client did not perform a responsibility in Section 4 moves by the length of the delay, and Provider will not be charged with the delay.');
  flow.paragraph('6.3 The MSA\'s termination provisions apply to this SOW. If Client terminates this SOW for convenience, Client will pay the fees for each milestone accepted before termination and, for the milestone in progress, a pro-rata amount based on the percentage of that milestone\'s work completed, as reasonably documented by Provider.');
  flow.heading('7. Milestones and Fees', { level: 3 });
  flow.paragraph(`The fees for this SOW are fixed at ${money(fee)}, payable by milestone as follows. Provider will invoice each milestone fee on Client's written acceptance of the milestone, and Client will pay each invoice within thirty days of receipt as provided in the MSA.`);
  flow.table([
    { header: 'Milestone', width: 0.12 },
    { header: 'Description', width: 0.46 },
    { header: 'Target date', width: 0.2 },
    { header: 'Fee', width: 0.22, align: 'right' },
  ], [...milestones.map(([code, description, date, amount]) => [code, description, longDate(date), money(amount)]), [{ text: 'Total', face: 'sans-bold' }, '', '', { text: money(fee), face: 'sans-bold' }]], { size: 9 });
  flow.paragraph('Travel to Client\'s operations center in Briarport is included in the fixed fee for up to eighteen trips. Additional travel requested by Client will be reimbursed at cost under the MSA\'s expense provisions. Out-of-scope work authorized by Change Order will be billed at the following rates: engagement lead $245 per hour, data engineer $195 per hour, data scientist $210 per hour, and trainer $150 per hour.');
  flow.heading('8. Acceptance', { level: 3 });
  flow.paragraph('Client will review each Deliverable within ten business days after Provider submits it and will either accept it in writing or describe in writing how it fails the applicable acceptance criteria. Provider will correct any failure and resubmit the Deliverable, and the review period will begin again. A Deliverable that Client does not reject within the review period is deemed accepted. Client\'s use of a Deliverable in production, other than for testing, is acceptance of that Deliverable.');
  flow.heading('9. Key Personnel', { level: 3 });
  flow.table([
    { header: 'Role', width: 0.34 },
    { header: 'Provider', width: 0.33 },
    { header: 'Client', width: 0.33 },
  ], [
    ['Executive sponsor', 'Rhiannon Iwasaki', 'Thaddeus Marchetti'],
    ['Project manager', 'Kofi Brightwater', 'Elodie Saltonstall'],
    ['Lead data scientist', 'Tamsin Oduya', '-'],
    ['Distribution operations lead', '-', 'Gideon Whitcombe'],
    ['IT and security liaison', 'Ulrich Penhaligon', 'Noor Kettleborough'],
  ], { size: 9 });
  flow.heading('10. Reporting and Security', { level: 3 });
  flow.paragraph('10.1 During hypercare Provider will send Client a daily summary of alerts raised, alerts confirmed by field crews, false positives, and platform data latency, and a closing report at the end of hypercare comparing measured model accuracy with the acceptance thresholds in Deliverable D3.');
  flow.paragraph('10.2 Provider personnel with access to Client systems will complete Client\'s security awareness training before access is granted, will use Client-issued accounts with multi-factor authentication, and will not use shared credentials. Client will revoke access within one business day after Provider notifies it that an individual has left the project.');
  flow.paragraph('10.3 Provider will report any suspected security incident affecting Client systems or data to Client\'s IT and security liaison by telephone within two hours of discovery and in writing within twenty-four hours, and will preserve all relevant logs.');
  flow.heading('11. Data Handling', { level: 3 });
  flow.paragraph('Customer account and consumption data processed under this SOW is Client Confidential Information under the MSA. Provider will process it only within Client\'s cloud tenancy, will not copy it to Provider systems except for anonymized samples approved in writing by Client\'s IT and security liaison, and will delete any such samples at the end of the project.');
  flow.paragraph('IN WITNESS WHEREOF, the parties have executed this Statement of Work.', { before: 6 });
  signatureBlocks(flow, [
    { heading: 'PROVIDER', entity: provider.toUpperCase(), name: 'Rhiannon Iwasaki', title: 'Managing Partner', date: longDate(signedProvider) },
    { heading: 'CLIENT', entity: client.toUpperCase(), name: 'Thaddeus Marchetti', title: 'General Manager', date: longDate(signedClient) },
  ]);
  flow.heading('Exhibit A - User Acceptance Test Outline (Milestone M4)', { level: 3 });
  flow.table([
    { header: 'Test', width: 0.08 },
    { header: 'Scenario', width: 0.56 },
    { header: 'Pass condition', width: 0.36 },
  ], [
    ['T1', 'Continuous-consumption alert raised for a premise with 26 hours of non-zero flow', 'Alert in queue within 2 hours of the 24th hour'],
    ['T2', 'Customer notification sent for a confirmed customer-side leak', 'Email and text delivered using approved template'],
    ['T3', 'District water balance anomaly in DMA-17 during a simulated main break', 'Anomaly flagged within 30 minutes'],
    ['T4', 'Pressure transient below 35 psi at two adjacent loggers', 'Transient alert with map location'],
    ['T5', 'Operator confirms an alert and opens a work order', 'Work order created with asset ID and coordinates'],
    ['T6', 'Stale meter (no read for 72 hours) excluded from leak scoring', 'Meter listed on data quality report, no alert'],
    ['T7', 'Reverse-flow alarm routed to cross-connection control', 'Alert assigned to the backflow program queue'],
    ['T8', 'Role-based access: customer service cannot edit model thresholds', 'Edit control hidden and API call refused'],
    ['T9', 'Platform restores from backup into the test environment', 'Restore completes within 4 hours with no data loss'],
    ['T10', 'Monthly non-revenue water report generated for the board packet', 'Report matches the validated water balance to 0.1%'],
  ], { size: 8.5 });
  flow.heading('Exhibit B - Meter Read Validation Rules', { level: 3 });
  flow.table([
    { header: 'Rule', width: 0.1 },
    { header: 'Condition', width: 0.5 },
    { header: 'Treatment', width: 0.4 },
  ], [
    ['V1', 'Hourly read missing for one to three consecutive hours', 'Linear interpolation; flagged as estimated'],
    ['V2', 'Hourly read missing for more than three consecutive hours', 'Profile estimate from the same weekday of the prior four weeks'],
    ['V3', 'Register decreases without a reverse-flow alarm', 'Treated as rollover if the drop exceeds 90% of register capacity; otherwise read rejected'],
    ['V4', 'Consumption above the meter\'s rated maximum flow for the hour', 'Read rejected and meter queued for field check'],
    ['V5', 'Read timestamp more than 15 minutes from the hour boundary', 'Shifted to the nearest hour if within 30 minutes; otherwise rejected'],
    ['V6', 'Duplicate reads for the same meter and hour', 'Latest received read kept'],
    ['V7', 'Meter exchanged during the hour (work order shows a swap)', 'Old and new registers combined; hour flagged'],
    ['V8', 'Zero consumption for 60 consecutive days at an active account', 'Meter queued for stopped-meter investigation'],
  ], { size: 8.5 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  const effectivePage = pageContaining(text, `effective as of ${longDate(effective)}`);
  if (effectivePage !== 3) throw new Error(`${id}: the SOW effective date must sit on page 3, not ${effectivePage}`);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Five-page statement of work for water-meter analytics under a master agreement',
    kind: 'sow',
    textLayer: 'native',
    pages: pages.length,
    categories: ['sow', 'contract', 'pages_5', 'middle_fact', 'referenced_agreement', 'competing_dates'],
    notes: `The first sentence and every page footer carry the master agreement's date (${longDate(msaDate)}); the SOW's own effective date (${longDate(effective)}) appears once, in Section 6 "Term" on page ${effectivePage} of ${pages.length}. Milestone dates fill a table, the end of the term (${longDate(completion)}) appears twice, and the signatures on the last page are dated ${longDate(signedProvider)} and ${longDate(signedClient)}. Personnel names in a table are not parties.`,
    gold: gold({
      type: 'Statement of Work',
      date: effective,
      role: 'effective',
      forbiddenDates: [[msaDate, 'date of the master agreement the SOW is issued under'], [completion, 'end of the SOW term / final milestone'], [signedProvider, 'provider signature date'], [signedClient, 'client signature date'], ...milestones.slice(0, 4).map(([code, , date]) => [date, `milestone ${code} target date`])],
      parties: [provider, client],
      relation: 'between',
      roles: [[provider, 'seller'], [client, 'client']],
      forbiddenParties: [['Rhiannon Iwasaki', 'provider executive sponsor and signatory'], ['Thaddeus Marchetti', 'client signatory'], ['Kofi Brightwater', 'project manager']],
      facts: [[money(fee), money(fee).slice(1), '$224,000'], ['leak detection', 'leak']],
      subjectTerms: ['meter', 'leak detection', 'AMI', 'analytics'],
      readiness: 'ready',
      dateText: [longDate(effective)],
    }),
  });
}

export function assignmentAssumption() {
  const id = 'assignment-assumption';
  const assignor = 'Brindle Logistics Inc.';
  const assignee = 'Vantage Peak Freight LLC';
  const landlord = 'Calderwood Industrial Partners LLC';
  const leaseDate = '2021-11-01';
  const effective = '2026-06-01';
  const signed = '2026-05-22';
  const expiry = '2031-10-31';
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 66, bottom: 66 }, footer: pageFooter('Assignment and Assumption of Lease - 3150 Gristmill Lane'), keep: [assignor, assignee, landlord] });
  flow.heading('ASSIGNMENT AND ASSUMPTION OF LEASE', { level: 1, align: 'center', size: 14 });
  flow.paragraph(`This Assignment and Assumption of Lease (this "Assignment") is made by and between ${assignor}, a Texas corporation ("Assignor"), and ${assignee}, a Delaware limited liability company ("Assignee"), and is effective as of ${longDate(effective)} (the "Effective Date").`);
  flow.heading('Recitals', { level: 3 });
  flow.paragraph(`A. ${landlord} ("Landlord"), as landlord, and Assignor, as tenant, are parties to that certain Industrial Lease dated ${longDate(leaseDate)} (the "Lease"), under which Assignor leases approximately 184,600 square feet of warehouse and distribution space, including 22 dock-high doors and two drive-in doors, at 3150 Gristmill Lane, Fallow Creek, TX 76117 (the "Premises"), for a term expiring ${longDate(expiry)}.`);
  flow.paragraph(`B. Assignee is acquiring Assignor's regional distribution business, which operates from the Premises, under an Asset Purchase Agreement between them, and as part of that transaction Assignor wishes to assign the Lease to Assignee and Assignee wishes to assume it.`);
  flow.paragraph('C. Section 14.1 of the Lease requires Landlord\'s prior written consent to an assignment, and Landlord is giving that consent in the Consent of Landlord attached to this Assignment.');
  flow.paragraph('NOW, THEREFORE, for good and valuable consideration, the receipt and sufficiency of which are acknowledged, the parties agree as follows:');
  [
    ['Assignment', `Effective as of the Effective Date, Assignor assigns, transfers, and sets over to Assignee all of Assignor's right, title, and interest as tenant in, to, and under the Lease, including Assignor's interest in the security deposit of ${money(31400000)} held by Landlord and in the rights of first offer in Section 32 of the Lease.`],
    ['Assumption', 'Assignee accepts the assignment and assumes and agrees to perform and observe all of the covenants, obligations, and conditions of the tenant under the Lease that arise or accrue on or after the Effective Date, including the payment of all base rent, operating expenses, taxes, and insurance charges.'],
    ['Assignor Obligations Before the Effective Date', 'Assignor remains responsible for all obligations of the tenant under the Lease that arose or accrued before the Effective Date, including the reconciliation of operating expenses for calendar year 2025 and the pro-rata share of operating expenses for calendar year 2026 through the day before the Effective Date, which the parties will settle within thirty days after Landlord delivers its annual reconciliation.'],
    ['Indemnities', 'Assignor will indemnify, defend, and hold Assignee harmless from any claim arising out of Assignor\'s failure to perform the tenant\'s obligations under the Lease before the Effective Date. Assignee will indemnify, defend, and hold Assignor harmless from any claim arising out of Assignee\'s failure to perform those obligations on or after the Effective Date.'],
    ['Condition of the Premises', 'Assignee has inspected the Premises and accepts them in their condition on the Effective Date. Assignor makes no representation about the condition of the Premises except that, to Assignor\'s knowledge, it has received no written notice from Landlord of any default under the Lease that remains uncured, and the roof replacement over the north bays completed in 2024 remains under the contractor\'s warranty.'],
    ['Notices', `From the Effective Date, notices to the tenant under the Lease are to be sent to ${assignee}, 905 Vantage Drive, Fallow Creek, TX 76119, Attention: Vice President, Real Estate.`],
    ['Governing Law; Counterparts', 'This Assignment is governed by the laws of the State of Texas. It may be executed in counterparts and by electronic signature, each of which is an original and all of which together are one instrument.'],
  ].forEach(([title, body], index) => flow.paragraph([{ text: `${index + 1}. ${title}. `, face: 'sans-bold' }, { text: body }]));
  flow.paragraph(`Executed on ${longDate(signed)}, to be effective as of the Effective Date.`, { before: 4 });
  signatureBlocks(flow, [
    { heading: 'ASSIGNOR', entity: assignor.toUpperCase(), name: 'Saul Garroway', title: 'President' },
    { heading: 'ASSIGNEE', entity: assignee.toUpperCase(), name: 'Hana Lachance', title: 'Chief Executive Officer' },
  ]);
  flow.heading('CONSENT OF LANDLORD', { level: 3, align: 'center' });
  flow.paragraph(`${landlord}, as Landlord under the Lease, consents to the foregoing assignment of the Lease to Assignee. This consent does not release Assignor from its obligations under the Lease, which continue as those of a guarantor of the tenant's obligations for the balance of the current term, and is not a consent to any further assignment or sublease. Landlord confirms that, as of the date of its signature, the Lease is in full force and effect, base rent has been paid through May 31, 2026, and Landlord has not given Assignor any notice of default that remains uncured.`);
  signatureBlocks(flow, [{ heading: 'LANDLORD', entity: landlord.toUpperCase(), name: 'Vesna Albrecht', title: 'Authorized Signatory' }], { stacked: true });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Assignment and assumption of a warehouse lease, with landlord consent',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['contract', 'referenced_agreement', 'competing_dates'],
    notes: `The assignment is effective as of ${longDate(effective)}. It assigns a lease dated ${longDate(leaseDate)} that expires ${longDate(expiry)}, was executed on ${longDate(signed)}, and the landlord's consent confirms rent paid through May 31, 2026. The landlord signs a consent but is not a party to the assignment; filing it as "between" the landlord and either side is wrong.`,
    gold: gold({
      type: 'Assignment and Assumption of Lease',
      date: effective,
      role: 'effective',
      forbiddenDates: [[leaseDate, 'date of the lease being assigned'], [signed, 'execution date; the assignment states its own effective date'], [expiry, 'lease expiration'], ['2026-05-31', 'rent paid-through date in the landlord consent']],
      parties: [assignor, assignee],
      relation: 'between',
      roles: [[assignor, 'assignor'], [assignee, 'assignee']],
      forbiddenParties: [[landlord, 'landlord that consents; not a party to the assignment'], ['Saul Garroway', 'signatory'], ['Hana Lachance', 'signatory']],
      facts: [['3150 Gristmill Lane', 'Gristmill Lane'], [landlord, 'Calderwood']],
      subjectTerms: ['lease', 'warehouse', 'Fallow Creek', 'assignment'],
      readiness: 'ready',
      dateText: [longDate(effective)],
    }),
  });
}

export function leaseTwoColumn() {
  const id = 'lease-two-column';
  const landlord = 'Oakhaven Retail Properties LLC';
  const tenant = 'Sorrel & Thistle Tea House LLC';
  const effective = '2026-05-18';
  const commencement = '2026-07-01';
  const expiration = '2031-06-30';
  const rentStart = '2026-09-01';
  const flow = new Flow({ face: 'serif', fontSize: 9.5, leading: 1.25, margins: { top: 60, bottom: 60, left: 54, right: 54 }, footer: (page, { number, total }) => page.textCenter(306, 762, `Oakhaven Commons - Retail Lease - Suite 108 - Page ${number} of ${total}`, { face: 'sans', size: 7.5, grey: 0.35 }), keep: [landlord, tenant] });
  flow.heading('RETAIL LEASE', { level: 1, align: 'center', size: 16, after: 2 });
  flow.paragraph('Oakhaven Commons Shopping Center', { align: 'center', face: 'sans', size: 10, after: 8 });
  flow.paragraph(`This Retail Lease is made as of ${longDate(effective)} (the "Effective Date") between ${landlord}, a North Carolina limited liability company ("Landlord"), and ${tenant}, a North Carolina limited liability company ("Tenant").`, { size: 10 });
  flow.heading('Article 1. Basic Lease Information', { level: 3, size: 10 });
  const base = [3400, 3502, 3607, 3715, 3827];
  flow.table([{ header: 'Term', width: 0.3 }, { header: 'Provision', width: 0.7 }], [
    ['Premises', 'Suite 108, approximately 2,140 rentable square feet, Oakhaven Commons, 1800 Lamplighter Drive, Briarport, NC 27519'],
    ['Commencement Date', `${longDate(commencement)}, or the date Landlord delivers the Premises with Landlord's Work substantially complete, if later`],
    ['Rent Commencement Date', `${longDate(rentStart)} (sixty days of free base rent for Tenant's build-out)`],
    ['Expiration Date', longDate(expiration)],
    ['Base Rent', base.map((value, index) => `Year ${index + 1}: $${decimal(value, 2)} per sq. ft.`).join('; ')],
    ['Additional Rent', 'Tenant\'s Share (3.1%) of Operating Costs, Taxes, and Insurance; 2026 estimate $7.85 per sq. ft.'],
    ['Security Deposit', money(1450000)],
    ['Permitted Use', 'Retail sale of loose-leaf tea, tea ware, and pastries; on-premises tea service with seating for up to 40'],
    ['Guarantor', 'Linnea Thorne, individually'],
  ], { size: 8.5, border: 'grid' });
  flow.startColumns(2, 20);
  const clauses = [
    ['Premises and Common Areas', `Landlord leases the Premises to Tenant for the Term. Tenant may use, in common with other tenants, the parking areas, sidewalks, and other common areas of the Shopping Center, subject to rules Landlord adopts from time to time. Landlord may change the common areas if access to and visibility of the Premises are not materially impaired.`],
    ['Term', `The Term begins on the Commencement Date and ends on the Expiration Date. Tenant has one option to extend the Term for five years on twelve months' notice, at the greater of the base rent then in effect and ninety-five percent of fair market rent.`],
    ['Rent', `Tenant will pay Base Rent in monthly installments in advance on the first day of each month beginning on the Rent Commencement Date, prorated for any partial month. Rent not received within five days after it is due bears a late charge of five percent of the overdue amount.`],
    ['Operating Costs', `Tenant will pay Tenant's Share of Operating Costs, Taxes, and Insurance in monthly installments based on Landlord's reasonable estimate. Within 120 days after each calendar year, Landlord will deliver a statement of actual costs; any overpayment will be credited against the next installments, and any underpayment paid within thirty days. Controllable Operating Costs may not increase by more than five percent per year on a cumulative basis.`],
    ['Use', `Tenant will use the Premises only for the Permitted Use, will keep them open during the Shopping Center's core hours (10:00 a.m. to 7:00 p.m. Monday through Saturday), and will not sell coffee as its primary product, which is reserved to another tenant under an existing exclusive.`],
    ['Landlord\'s Work and Tenant Improvements', `Landlord will deliver the Premises with the base building work described in Exhibit C complete, including a 200-amp electrical service, a grease interceptor, and a demised HVAC unit. Landlord will reimburse Tenant up to ${money(6420000)} for Tenant's improvements, paid within thirty days after Tenant opens for business and delivers lien waivers.`],
    ['Utilities', 'Tenant will pay for all utilities serving the Premises, which are separately metered. Landlord is not liable for any interruption of utilities not caused by its negligence, but if an interruption within Landlord\'s control lasts more than three business days, Base Rent abates until service is restored.'],
    ['Maintenance and Repairs', 'Landlord will maintain the roof, structure, and common areas. Tenant will maintain the interior of the Premises, the storefront glass, and the HVAC unit serving the Premises, under a service contract with a licensed contractor that provides for inspections at least quarterly.'],
    ['Alterations', 'Tenant may not make alterations costing more than $15,000 or affecting the structure, roof, or building systems without Landlord\'s prior written consent. All alterations become Landlord\'s property at the end of the Term unless Landlord requires their removal when it consents.'],
    ['Signs', 'Tenant will install a storefront sign within sixty days after the Commencement Date, conforming to the Shopping Center sign criteria, and may place its name on both panels of the pylon sign at Lamplighter Drive at its own cost.'],
    ['Insurance', 'Tenant will carry commercial general liability insurance of at least $2,000,000 per occurrence, property insurance on its improvements and personal property at full replacement cost, and liquor liability insurance if it serves alcohol. Landlord will insure the building at full replacement cost. Each party waives claims against the other to the extent covered by property insurance.'],
    ['Assignment and Subletting', 'Tenant may not assign this Lease or sublet the Premises without Landlord\'s consent, which Landlord will not unreasonably withhold. A transfer of more than half of the membership interests in Tenant is an assignment.'],
    ['Default and Remedies', 'Tenant is in default if it fails to pay rent within ten days after notice or fails to perform any other obligation within thirty days after notice. Landlord may then terminate this Lease or Tenant\'s right of possession and recover the rent due for the balance of the Term, discounted to present value, less the rent Landlord can reasonably obtain by reletting.'],
    ['Surrender and Holdover', 'At the end of the Term Tenant will surrender the Premises broom clean and in good repair. If Tenant holds over, it will pay 150% of the Base Rent then in effect for each month of holdover.'],
    ['Subordination', 'This Lease is subordinate to any mortgage on the Shopping Center, provided the mortgagee agrees not to disturb Tenant\'s possession while Tenant is not in default.'],
  ];
  clauses.forEach(([title, body], index) => flow.paragraph([{ text: `${index + 2}. ${title}. `, face: 'sans-bold' }, { text: body }]));
  flow.endColumns();
  flow.paragraph('IN WITNESS WHEREOF, the parties have executed this Lease as of the Effective Date.', { before: 6, size: 10 });
  signatureBlocks(flow, [
    { heading: 'LANDLORD', entity: landlord.toUpperCase(), name: 'Beatrix Hollingsworth', title: 'Manager' },
    { heading: 'TENANT', entity: tenant.toUpperCase(), name: 'Linnea Thorne', title: 'Managing Member' },
  ], { size: 9.5 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Shopping-center retail lease typeset in two columns',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['multi_column', 'contract', 'competing_dates', 'table'],
    notes: `Two-column body, written column by column so the text layer reads in order. The lease is made as of ${longDate(effective)}; the basic lease information table also gives a Commencement Date (${longDate(commencement)}), a Rent Commencement Date (${longDate(rentStart)}), and an Expiration Date (${longDate(expiration)}). The commencement date is accepted because the prompt treats a commencement date as an agreement's own date; the rent start and expiration are traps.`,
    gold: gold({
      type: 'Retail Lease',
      acceptableTypes: ['Lease Agreement', 'Lease'],
      date: effective,
      acceptableDates: [commencement],
      role: 'effective',
      forbiddenDates: [[rentStart, 'rent commencement date'], [expiration, 'lease expiration date']],
      parties: [landlord, tenant],
      relation: 'between',
      roles: [[landlord, 'landlord'], [tenant, 'tenant']],
      forbiddenParties: [['Linnea Thorne', 'guarantor and signatory'], ['Beatrix Hollingsworth', 'signatory']],
      facts: [['Suite 108'], ['Oakhaven Commons'], ['tea']],
      subjectTerms: ['retail lease', 'Suite 108', 'Oakhaven Commons', 'tea'],
      readiness: 'ready',
      dateText: [longDate(effective)],
    }),
  });
}

export function promissoryNote() {
  const id = 'promissory-note';
  const borrower = 'Foxhallow Outfitters LLC';
  const lender = 'Ashgrove Capital Partners LLC';
  const noteDate = '2025-09-08';
  const firstPayment = '2025-10-08';
  const maturity = '2030-09-08';
  const principal = 25000000;
  const rateBasisPoints = 825; // 8.25% per year
  // Level monthly payment over 60 months, computed in integer cents with a
  // rational approximation of the annuity formula (no Math.pow surprises:
  // repeated multiplication is exact enough and deterministic).
  const monthly = rateBasisPoints / 10000 / 12;
  let factor = 1;
  for (let index = 0; index < 60; index += 1) factor *= 1 + monthly;
  const payment = Math.round((principal * monthly * factor) / (factor - 1));
  const schedule = [];
  let balance = principal;
  for (let index = 0; index < 60; index += 1) {
    const interest = Math.round(balance * monthly);
    const principalPart = index === 59 ? balance : payment - interest;
    balance -= principalPart;
    schedule.push({ number: index + 1, date: addMonths(firstPayment, index), payment: principalPart + interest, interest, principal: principalPart, balance });
  }
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 66, bottom: 66 }, footer: pageFooter(`Promissory Note - ${borrower}`), keep: [borrower, lender] });
  flow.heading('PROMISSORY NOTE', { level: 1, align: 'center', size: 15 });
  flow.table([{ header: '', width: 0.33 }, { header: '', width: 0.34 }, { header: '', width: 0.33 }], [[money(principal), 'Copper Flats, Arizona', `Effective Date: ${longDate(noteDate)}`]], { header: false, border: 'none', size: 10.5, face: 'sans-bold' });
  flow.paragraph(`FOR VALUE RECEIVED, ${borrower}, an Arizona limited liability company ("Borrower"), promises to pay to the order of ${lender}, a Delaware limited liability company (together with any holder of this Note, "Lender"), the principal sum of ${money(principal)}, with interest on the unpaid principal balance at the rate of ${decimal(rateBasisPoints, 2)}% per year, as provided in this Promissory Note (this "Note").`);
  [
    ['Payments', `Borrower will pay principal and interest in sixty (60) consecutive monthly installments of ${money(payment)} each, beginning on ${longDate(firstPayment)} and continuing on the eighth day of each month after that, with a final installment of all unpaid principal and accrued interest due on ${longDate(maturity)} (the "Maturity Date"). Payments are applied first to late charges, then to accrued interest, and then to principal. The amortization schedule attached as Schedule 1 is for convenience; if it conflicts with this Note, this Note controls.`],
    ['Interest', 'Interest is computed on the basis of a 360-day year of twelve 30-day months. After maturity, whether by acceleration or otherwise, and while any Event of Default continues, the unpaid principal bears interest at the stated rate plus five percent (5%) per year.'],
    ['Prepayment', 'Borrower may prepay this Note in whole or in part at any time. A prepayment made before the second anniversary of the Effective Date is subject to a prepayment premium of two percent (2%) of the principal prepaid, and one made after the second and before the third anniversary to a premium of one percent (1%). Partial prepayments are applied to installments in inverse order of maturity.'],
    ['Late Charge', 'If any installment is not received within ten (10) days after its due date, Borrower will pay a late charge equal to five percent (5%) of the overdue installment.'],
    ['Security', 'This Note is secured by a Security Agreement of even date granting Lender a first-priority security interest in Borrower\'s inventory, equipment, and accounts, and by the personal guaranty of Borrower\'s managing member.'],
    ['Events of Default', 'Each of the following is an Event of Default: (a) Borrower fails to pay any amount within ten days after it is due; (b) Borrower breaches the Security Agreement and the breach continues for thirty days after notice; (c) Borrower becomes insolvent or is the subject of a bankruptcy petition not dismissed within sixty days; or (d) Borrower sells all or substantially all of its assets or merges without Lender\'s consent. On an Event of Default Lender may declare the entire unpaid principal and accrued interest immediately due.'],
    ['Waivers; Costs', 'Borrower waives presentment, demand, protest, and notice of dishonor, and will pay Lender\'s reasonable costs of collection, including attorneys\' fees, after an Event of Default.'],
    ['Governing Law', 'This Note is governed by the laws of the State of Arizona.'],
  ].forEach(([title, body], index) => flow.paragraph([{ text: `${index + 1}. ${title}. `, face: 'sans-bold' }, { text: body }]));
  signatureBlocks(flow, [{ heading: 'BORROWER', entity: borrower.toUpperCase(), name: 'Adaeze Calloway', title: 'Managing Member' }], { stacked: true });
  flow.pageBreak();
  flow.heading('Schedule 1 - Amortization Schedule (first 24 installments)', { level: 3 });
  flow.table([
    { header: 'No.', width: 0.08, align: 'right' },
    { header: 'Due date', width: 0.2 },
    { header: 'Payment', width: 0.18, align: 'right' },
    { header: 'Interest', width: 0.17, align: 'right' },
    { header: 'Principal', width: 0.17, align: 'right' },
    { header: 'Balance', width: 0.2, align: 'right' },
  ], schedule.slice(0, 24).map((row) => [String(row.number), numericDate(row.date), money(row.payment), money(row.interest), money(row.principal), money(row.balance)]), { size: 8.5, border: 'rules' });
  const totalInterest = schedule.reduce((sum, row) => sum + row.interest, 0);
  flow.paragraph(`Installments 25 through 60 continue on the same basis. Total of payments over the full term: ${money(principal + totalInterest)}, of which ${money(totalInterest)} is interest, assuming every installment is paid on its due date.`, { size: 9 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Five-year secured promissory note with an amortization schedule',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['financial', 'contract', 'competing_dates', 'table'],
    notes: `The note's date is a labelled "Effective Date" (${longDate(noteDate)}) in the header line beside the principal amount. The first installment (${longDate(firstPayment)}), the maturity date (${longDate(maturity)}), and 24 installment due dates in the schedule are traps. The note is signed by the borrower only; it is filed between borrower and lender.`,
    gold: gold({
      type: 'Promissory Note',
      date: noteDate,
      role: 'effective',
      forbiddenDates: [[firstPayment, 'first installment due date'], [maturity, 'maturity date'], [schedule[1].date, 'installment due date']],
      parties: [borrower, lender],
      relation: 'between',
      acceptablePartySets: [{ parties: [borrower], relation: 'from' }, { parties: [lender], relation: 'to' }],
      roles: [[borrower, 'borrower'], [lender, 'lender'], [borrower, 'issuer'], [lender, 'recipient']],
      forbiddenParties: [['Adaeze Calloway', 'signatory for the borrower']],
      facts: [[money(principal), '$250,000'], [`${decimal(rateBasisPoints, 2)}%`, '8.25'], [longDate(maturity), '2030']],
      subjectTerms: ['promissory note', 'installments', 'secured', 'principal'],
      readiness: 'ready',
      dateText: [longDate(noteDate)],
    }),
  });
}

