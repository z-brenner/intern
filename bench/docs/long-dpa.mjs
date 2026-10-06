/// A ten-page data processing addendum: operative terms, a description of
/// the processing, a security annex of named controls, and a sub-processor
/// list of more than twenty companies, none of them a party.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { grouped, longDate } from '../lib/format.mjs';
import { digitalPdf, result, signatureBlocks } from './common.mjs';

const SUBPROCESSORS = [
  ['Starling Compute Ltd.', 'Ireland', 'Primary cloud hosting (EU region)', 'Adequacy not required (EEA)'],
  ['Starling Compute Inc.', 'United States', 'Disaster recovery hosting (us-east)', 'SCCs Module 3'],
  ['Kettleborough Mail Systems Ltd.', 'United Kingdom', 'Transactional email delivery', 'UK adequacy regulations'],
  ['Quillfeather Support Software Inc.', 'United States', 'Customer support ticketing', 'SCCs Module 3; EU-US framework certified'],
  ['Basalt Observability GmbH', 'Germany', 'Application performance monitoring', 'Adequacy not required (EEA)'],
  ['Larkspur Identity Services Inc.', 'United States', 'Single sign-on and directory sync', 'SCCs Module 3'],
  ['Driftmoor Payments B.V.', 'Netherlands', 'Card payment processing for parent fees', 'Adequacy not required (EEA)'],
  ['Mossgiel Translation AB', 'Sweden', 'Machine translation of feedback comments', 'Adequacy not required (EEA)'],
  ['Calderwood Video Platforms Ltd.', 'Ireland', 'Lecture video transcoding and streaming', 'Adequacy not required (EEA)'],
  ['Corvid Security Operations Inc.', 'United States', '24x7 security monitoring (SOC)', 'SCCs Module 3; access to logs only'],
  ['Amberline Data Warehousing ApS', 'Denmark', 'Analytics warehouse (pseudonymised)', 'Adequacy not required (EEA)'],
  ['Pennywhistle SMS Ltd.', 'United Kingdom', 'SMS notifications to parents', 'UK adequacy regulations'],
  ['Hazelmoor Backup Services S.A.', 'Luxembourg', 'Encrypted off-site backups', 'Adequacy not required (EEA)'],
  ['Thornbury Accessibility Labs LLC', 'United States', 'Accessibility audits on anonymised screens', 'SCCs Module 3; no direct data access'],
  ['Ravensmoor Search Technologies Oy', 'Finland', 'Full-text search of course materials', 'Adequacy not required (EEA)'],
  ['Elmstead Proctoring Ltd.', 'Ireland', 'Optional exam proctoring (opt-in schools only)', 'Adequacy not required (EEA)'],
  ['Silverbirch Analytics Pte. Ltd.', 'Singapore', 'Follow-the-sun support tier 2 (read-only)', 'SCCs Module 3; transfer impact assessment'],
  ['Oakhaven Document Signing Inc.', 'United States', 'Electronic signature of consent forms', 'SCCs Module 3'],
  ['Granite Bay Status Pages Ltd.', 'United Kingdom', 'Incident status page subscriptions', 'UK adequacy regulations'],
  ['Juniper Hill Feature Flags SAS', 'France', 'Feature rollout configuration', 'Adequacy not required (EEA)'],
  ['Cresthaven Customer Surveys Ltd.', 'Ireland', 'Teacher satisfaction surveys', 'Adequacy not required (EEA)'],
  ['Saltmarsh Logging GmbH', 'Austria', 'Centralised log storage (90-day retention)', 'Adequacy not required (EEA)'],
  ['Foxhallow Fraud Screening Inc.', 'Canada', 'Account takeover detection signals', 'Canada adequacy decision'],
];

const CONTROLS = [
  ['GOV-01', 'Governance', 'An information security policy approved by the board and reviewed at least annually; a named Chief Information Security Officer reports to the CEO.'],
  ['GOV-02', 'Governance', 'Risk assessments of the platform at least annually and before any material architecture change, recorded in a risk register with owners and due dates.'],
  ['HR-01', 'Personnel', 'Background checks before hire where lawful; confidentiality undertakings signed before access; security and privacy training at hire and every year.'],
  ['HR-02', 'Personnel', 'Access removed within 24 hours of departure; quarterly reconciliation of HR records against identity provider accounts.'],
  ['AC-01', 'Access control', 'Role-based access with least privilege; production data access only through a bastion with just-in-time approval lasting no more than eight hours.'],
  ['AC-02', 'Access control', 'Phishing-resistant multi-factor authentication for all workforce accounts; no shared accounts; service credentials held in a managed vault and rotated every 90 days.'],
  ['AC-03', 'Access control', 'Quarterly access reviews by system owners, with exceptions tracked to closure within 30 days.'],
  ['CR-01', 'Cryptography', 'TLS 1.2 or higher for data in transit; AES-256 for data at rest; customer-managed keys available for the analytics warehouse.'],
  ['CR-02', 'Cryptography', 'Keys generated and stored in hardware security modules; key usage logged; keys rotated annually and on suspected compromise.'],
  ['OP-01', 'Operations', 'Hardened, immutable server images rebuilt weekly; critical vulnerabilities patched within 7 days and high within 30 days.'],
  ['OP-02', 'Operations', 'Anti-malware and endpoint detection on all workforce devices; full-disk encryption enforced by device management.'],
  ['OP-03', 'Operations', 'Centralised logging of authentication, administrative, and data export events, retained for at least one year and protected from alteration.'],
  ['NW-01', 'Network', 'Production networks segmented by function; default-deny security groups; web application firewall in front of all public endpoints.'],
  ['NW-02', 'Network', 'Distributed denial-of-service protection; rate limiting on authentication and export endpoints.'],
  ['DV-01', 'Development', 'Peer review of every change; static analysis and dependency scanning in the build pipeline; no production data in development or test environments.'],
  ['DV-02', 'Development', 'Annual third-party penetration test of the platform and mobile apps; findings rated high or critical remediated before the next release.'],
  ['BC-01', 'Continuity', 'Daily encrypted backups with 35-day retention, restore tested quarterly; recovery point objective 1 hour and recovery time objective 8 hours.'],
  ['BC-02', 'Continuity', 'Primary and recovery regions at least 300 km apart; annual failover exercise with written results.'],
  ['IR-01', 'Incident response', 'Documented incident response plan with on-call rotation; tabletop exercise at least twice a year including a personal data breach scenario.'],
  ['IR-02', 'Incident response', 'Forensic preservation of logs and images on any suspected breach; post-incident review within 10 business days.'],
  ['PH-01', 'Physical', 'Hosting only in data centres with ISO 27001 certification, 24x7 guarding, and biometric access; no customer data stored at Processor offices.'],
  ['DM-01', 'Data management', 'Pseudonymisation of student identifiers in the analytics warehouse; aggregation thresholds of at least 10 students in reports.'],
  ['DM-02', 'Data management', 'Automated deletion of personal data at the end of the retention period; deletion certificates on request.'],
  ['VM-01', 'Vendor management', 'Security and privacy due diligence of each sub-processor before engagement and annually after, scaled to the data it receives.'],
];

export function dataProcessingAgreement() {
  const id = 'data-processing-agreement-10p';
  const rng = Rng.from(id);
  const controller = 'Thistledown Learning Inc.';
  const processor = 'Skylark Analytics Ltd.';
  const effective = '2026-02-02';
  const msaDate = '2023-08-14';
  const tiaDate = '2025-11-20';
  const signedProcessor = '2026-01-28';
  const signedController = '2026-01-30';
  const flow = new Flow({ face: 'serif', fontSize: 10.5, leading: 1.35, margins: { top: 64, bottom: 64 }, keep: [controller, processor],
    header: (page, { number }) => {
      if (number > 1) page.text(72, 44, 'Data Processing Addendum - Thistledown / Skylark', { face: 'sans', size: 7.5, grey: 0.4 });
    },
    footer: (page, { number, total }) => page.textRight(540, 760, `Page ${number} of ${total}`, { face: 'sans', size: 7.5, grey: 0.4 }),
  });
  flow.heading('DATA PROCESSING ADDENDUM', { level: 1, align: 'center', size: 15 });
  flow.paragraph(`This Data Processing Addendum ("DPA") is entered into as of ${longDate(effective)} (the "DPA Effective Date") between ${controller}, a Delaware corporation ("Customer"), and ${processor}, a company registered in Ireland under number 714302 ("Skylark"), and forms part of the Master Subscription Agreement between them dated ${longDate(msaDate)} (the "Agreement"). This DPA replaces the data processing terms in Schedule 4 of the Agreement from the DPA Effective Date.`);
  flow.heading('1. Definitions', { level: 3 });
  for (const [term, meaning] of [
    ['Applicable Data Protection Law', 'the General Data Protection Regulation (EU) 2016/679 ("GDPR"), the GDPR as retained in the law of the United Kingdom, the Swiss Federal Act on Data Protection, and the United States federal and state student privacy laws listed in Annex IV, in each case as amended'],
    ['Customer Personal Data', 'personal data that Skylark processes on behalf of Customer in providing the Services, as described in Annex I'],
    ['Data Subject Request', 'a request by a data subject to exercise a right under Applicable Data Protection Law'],
    ['Personal Data Breach', 'a breach of security leading to the accidental or unlawful destruction, loss, alteration, unauthorised disclosure of, or access to, Customer Personal Data'],
    ['Restricted Transfer', 'a transfer of Customer Personal Data to a country that Applicable Data Protection Law does not recognise as providing an adequate level of protection'],
    ['SCCs', 'the standard contractual clauses annexed to Commission Implementing Decision (EU) 2021/914, together with the UK International Data Transfer Addendum where the transfer is from the United Kingdom'],
    ['Services', 'the Thistledown classroom analytics and parent engagement services described in the Agreement and its order forms'],
    ['Sub-processor', 'any processor engaged by Skylark that processes Customer Personal Data'],
  ]) flow.paragraph([{ text: `"${term}" `, face: 'sans-bold' }, { text: `means ${meaning}.` }], { indent: 14 });
  const sections = [
    ['Roles and Scope', [
      'Customer is the controller of Customer Personal Data, or acts on the instructions of the schools and districts that are its customers and are the controllers. Skylark is a processor, or a sub-processor where Customer is itself a processor. Each party will comply with the obligations Applicable Data Protection Law places on it in that role.',
      'Skylark will process Customer Personal Data only to provide the Services, on Customer\'s documented instructions, which are this DPA, the Agreement, and Customer\'s configuration of the Services. Skylark will tell Customer promptly if it believes an instruction infringes Applicable Data Protection Law, and may suspend the affected processing until the instruction is confirmed or changed.',
      'Skylark will not sell Customer Personal Data, use it for targeted advertising, build profiles of students except as the Services require, or combine it with data from other customers, and will not use it to train machine learning models made available to anyone other than Customer.',
    ]],
    ['Processor Obligations', [
      'Skylark will ensure that every person it authorises to process Customer Personal Data is bound by a duty of confidentiality and receives training appropriate to the data, and will limit access to those who need it to provide the Services.',
      'Taking into account the nature of the processing, Skylark will assist Customer by appropriate technical and organisational measures in responding to Data Subject Requests, will forward to Customer within five business days any request it receives directly, and will not respond to the request itself except to direct the data subject to Customer.',
      'Skylark will provide reasonable assistance with data protection impact assessments and prior consultations with supervisory authorities that relate to the Services, including by providing the information in its most recent security assessment. Assistance beyond eight hours in a calendar year is chargeable at the rates in the Agreement.',
    ]],
    ['Sub-processors', [
      'Customer gives Skylark general authorisation to engage Sub-processors. The Sub-processors engaged on the DPA Effective Date are listed in Annex III, and Customer approves them.',
      'Skylark will give Customer at least thirty days\' notice of any new Sub-processor by updating the list at its trust centre and emailing the contacts Customer has registered. Customer may object on reasonable data protection grounds within that period; if the parties cannot resolve the objection within a further thirty days, Customer may terminate the affected Services and receive a refund of prepaid fees for the unused term.',
      'Skylark will impose on each Sub-processor data protection obligations that are no less protective than those in this DPA, and remains liable to Customer for the performance of each Sub-processor\'s obligations.',
    ]],
    ['Security', [
      'Skylark will implement and maintain the technical and organisational measures in Annex II. Skylark may update those measures as technology and threats change, provided the update does not materially decrease the overall security of the Services.',
      'Skylark will maintain an ISO/IEC 27001 certification for the information security management system that covers the Services and will obtain a SOC 2 Type II report on the Services each year, and will make the current certificate and report available to Customer on request under confidentiality terms.',
    ]],
    ['Personal Data Breaches', [
      'Skylark will notify Customer without undue delay, and in any event within forty-eight hours, after becoming aware of a Personal Data Breach. The notice will describe, to the extent then known, the nature of the breach, the categories and approximate number of data subjects and records concerned, the likely consequences, and the measures taken or proposed.',
      'Skylark will take reasonable steps to contain and remediate the breach, will update Customer as further information becomes available, and will cooperate with Customer in any notification to supervisory authorities, schools, parents, or students. Skylark will not notify any of them about a breach affecting Customer Personal Data without Customer\'s approval, except as the law requires.',
    ]],
    ['International Transfers', [
      `Customer Personal Data is hosted in the European Economic Area. Where Services require a Restricted Transfer, the SCCs apply: Module 2 where Customer is a controller and Module 3 where it is a processor, with the optional docking clause, option 2 of clause 9(a) with the notice period in Section 4.2, and Irish law and courts in clauses 17 and 18.`,
      `Skylark has documented a transfer impact assessment for each Restricted Transfer, most recently updated on ${longDate(tiaDate)}, and will provide a summary to Customer on request. If Skylark can no longer comply with the SCCs, it will notify Customer and suspend the affected transfer.`,
    ]],
    ['Audits', [
      'Customer may audit Skylark\'s compliance with this DPA once in any twelve-month period, and additionally after a Personal Data Breach or at the direction of a supervisory authority. Skylark will first make its certifications and audit reports available, and Customer will rely on them unless they do not reasonably address its question.',
      'An on-site audit requires thirty days\' notice, takes place during business hours over no more than two days, is conducted by Customer or an independent auditor bound by confidentiality, and may not access other customers\' data. Each party bears its own costs unless the audit reveals a material breach of this DPA by Skylark.',
    ]],
    ['Return and Deletion', [
      'At the end of the Services Skylark will, at Customer\'s choice, return Customer Personal Data in a standard machine-readable format or delete it, within thirty days, and will delete remaining copies from backups within the following thirty-five days as the backup cycle completes. Skylark will certify the deletion in writing on request.',
      'Skylark may retain Customer Personal Data only where the law requires it to, and then only for the period required, protected as this DPA requires, and for no other purpose.',
    ]],
    ['Customer Obligations', [
      'Customer is responsible for the lawfulness of the processing instructions it gives, for providing schools with the notices and obtaining any consents Applicable Data Protection Law requires before student data is entered into the Services, and for configuring the Services so that only the data needed for the schools\' purposes is collected.',
      'Customer will not instruct Skylark to process special categories of personal data or data about criminal convictions, and will ensure that teachers are told not to enter health or disciplinary details in free-text fields.',
    ]],
    ['United States State Privacy Laws', [
      'To the extent Customer Personal Data is "personal information" under a United States state privacy law, Skylark is a "service provider" or "processor" as those laws define the terms, and will not retain, use, or disclose that information outside the direct business relationship with Customer or for any purpose other than performing the Services.',
      'Skylark certifies that it understands these restrictions and will comply with them, will notify Customer if it can no longer meet its obligations, and will allow Customer to take reasonable steps to stop and remediate unauthorised use.',
      'Where a state student privacy law requires a separate data privacy agreement with a school district, Skylark will sign the form of agreement adopted by that state\'s student data privacy consortium, and that agreement prevails for that district.',
    ]],
    ['Government Access Requests', [
      'If a government authority asks Skylark for Customer Personal Data, Skylark will attempt to redirect the authority to Customer, will notify Customer promptly unless the law prohibits it, and will challenge any request it reasonably considers unlawful, overbroad, or inconsistent with the SCCs.',
      'Skylark will publish at least annually a transparency report stating the number of government requests for customer data it received and how it responded, and will disclose only the minimum data necessary to comply with a binding order.',
    ]],
    ['Records and Cooperation', [
      'Skylark will keep records of its processing activities for Customer as Article 30(2) GDPR requires and will make them available to Customer and to supervisory authorities on request. Each party will designate a privacy contact and keep the other informed of changes to it.',
    ]],
    ['Liability and Term', [
      'Each party\'s liability under this DPA is subject to the limitations and exclusions of liability in the Agreement, except that those limitations do not restrict a data subject\'s rights under the SCCs.',
      'This DPA remains in effect for as long as Skylark processes Customer Personal Data. If this DPA conflicts with the Agreement, this DPA controls as to the processing of personal data; if it conflicts with the SCCs, the SCCs control.',
    ]],
  ];
  sections.forEach(([title, paragraphs], index) => {
    flow.paragraph(`${index + 2}. ${title}`, { face: 'sans-bold', size: 10.5, after: 2, keepWithNext: 30 });
    paragraphs.forEach((text, part) => flow.paragraph([{ text: `${index + 2}.${part + 1} `, face: 'sans-bold' }, { text }]));
  });
  flow.paragraph('IN WITNESS WHEREOF, the parties have executed this DPA by their authorised representatives.', { before: 4 });
  signatureBlocks(flow, [
    { heading: 'CUSTOMER', entity: controller.toUpperCase(), name: 'Rosalind Ellingsen', title: 'Chief Privacy Officer', date: longDate(signedController) },
    { heading: 'SKYLARK', entity: processor.toUpperCase(), name: 'Ulrich Tavistock', title: 'Managing Director', date: longDate(signedProcessor) },
  ]);
  flow.pageBreak();
  flow.heading('Annex I - Description of the Processing', { level: 2 });
  const studentCount = rng.int(410, 470) * 1000;
  flow.table([{ header: 'Item', width: 0.28 }, { header: 'Description', width: 0.72 }], [
    ['Data subjects', `Students (approximately ${grouped(studentCount)} enrolled at Customer's schools), parents and guardians, teachers, school administrators, and district staff`],
    ['Categories of personal data', 'Name; school email address; student identifier assigned by the school; grade level and class enrolment; assignments, submissions, and teacher feedback; attendance flags; parent contact details and language preference; device identifiers and IP addresses; usage events (page views, time on task)'],
    ['Special categories', 'None intended. Teachers may record accommodation notes (for example extended time) in free-text fields; Customer is responsible for instructing schools to minimise such notes'],
    ['Nature of processing', 'Hosting, storage, analytics, generating progress reports, sending notifications, support and troubleshooting, backup and recovery'],
    ['Purposes', 'Providing the Services to Customer and its schools; maintaining their security and availability; producing aggregated, de-identified statistics on Service performance'],
    ['Frequency', 'Continuous for the duration of the Agreement'],
    ['Retention', 'For the duration of each school\'s subscription, plus 30 days; usage events older than 25 months are aggregated and the raw events deleted; backups expire after 35 days'],
    ['Location of processing', 'Ireland and Germany (primary); United States and Singapore for support and security monitoring as listed in Annex III'],
    ['Competent supervisory authority', 'Data Protection Commission (Ireland)'],
  ], { size: 9 });
  flow.heading('Annex II - Technical and Organisational Measures', { level: 2 });
  flow.table([{ header: 'Control', width: 0.11 }, { header: 'Domain', width: 0.17 }, { header: 'Measure', width: 0.72 }], CONTROLS, { size: 8.5 });
  flow.pageBreak();
  flow.heading('Annex III - Approved Sub-processors', { level: 2 });
  flow.paragraph('The following Sub-processors are approved as of the DPA Effective Date. Entries marked "opt-in" process Customer Personal Data only for schools that enable the related feature.', { size: 9.5 });
  flow.table([{ header: 'Sub-processor', width: 0.32 }, { header: 'Location', width: 0.15 }, { header: 'Service', width: 0.3 }, { header: 'Transfer mechanism', width: 0.23 }], SUBPROCESSORS, { size: 8.5 });
  flow.paragraph('Changes to this list during the twelve months before the DPA Effective Date:', { size: 9.5, before: 4, keepWithNext: 60 });
  flow.table([{ header: 'Date', width: 0.2 }, { header: 'Change', width: 0.8 }], [
    [longDate('2025-02-17'), 'Added Elmstead Proctoring Ltd. (opt-in exam proctoring)'],
    [longDate('2025-04-30'), 'Removed Hollowell Chat Services Ltd. (live chat widget retired)'],
    [longDate('2025-06-09'), 'Added Silverbirch Analytics Pte. Ltd. for tier 2 support outside European business hours'],
    [longDate('2025-09-22'), 'Moved disaster recovery hosting from eu-west-2 to us-east under SCCs Module 3'],
    [longDate('2025-12-01'), 'Added Foxhallow Fraud Screening Inc. (account takeover signals)'],
  ], { size: 8.5 });
  flow.heading('Annex IV - Student Privacy Laws and Contacts', { level: 2 });
  flow.table([{ header: 'Jurisdiction', width: 0.25 }, { header: 'Law or requirement', width: 0.45 }, { header: 'Customer contact', width: 0.3 }], [
    ['United States (federal)', 'Family Educational Rights and Privacy Act; Children\'s Online Privacy Protection Act (school authorisation)', 'privacy@thistledown-learning.example'],
    ['California', 'Student Online Personal Information Protection Act', 'Same'],
    ['Illinois', 'Student Online Personal Protection Act; breach notices to schools within 30 days', 'Same'],
    ['New York', 'Education Law section 2-d and the parents\' bill of rights', 'nys.compliance@thistledown-learning.example'],
    ['European Union', 'GDPR; national education data rules where schools are located', 'dpo@thistledown-learning.example'],
    ['United Kingdom', 'UK GDPR; Age Appropriate Design Code', 'dpo@thistledown-learning.example'],
  ], { size: 8.5 });
  flow.paragraph('Skylark\'s data protection officer can be reached at dpo@skylark-analytics.example or by post at 4 Calder Quay, Dublin 2, Ireland.', { size: 9 });
  flow.heading('Annex V - Retention Schedule', { level: 2 });
  flow.table([{ header: 'Data set', width: 0.3 }, { header: 'Retention', width: 0.38 }, { header: 'Deletion method', width: 0.32 }], [
    ['Student roster and enrolment', 'Subscription term plus 30 days', 'Hard delete; backups age out in 35 days'],
    ['Assignments and submissions', 'Subscription term plus 30 days, unless the school exports earlier', 'Hard delete'],
    ['Teacher feedback comments', 'Subscription term plus 30 days', 'Hard delete'],
    ['Attendance flags', 'Current and previous school year', 'Rolling deletion each August 1'],
    ['Usage events (raw)', '25 months', 'Aggregated to daily counts, raw rows deleted'],
    ['Usage aggregates (de-identified)', 'Indefinitely', 'Not personal data after aggregation'],
    ['Parent contact details', 'While the student is enrolled', 'Deleted within 30 days of unenrolment'],
    ['Notification delivery logs', '13 months', 'Rolling deletion'],
    ['Support tickets', '3 years from closure', 'Attachments purged at 12 months'],
    ['Security and audit logs', '1 year (authentication); 3 years (administrative actions)', 'Rolling deletion from write-once storage'],
    ['Backups', '35 days', 'Automatic expiry'],
    ['Consent records', '6 years after withdrawal or account closure', 'Hard delete'],
  ], { size: 8.5 });
  flow.pageBreak();
  flow.heading('Annex VI - Details of Restricted Transfers', { level: 2 });
  flow.table([{ header: 'Item', width: 0.3 }, { header: 'Module 2 (controller to processor)', width: 0.35 }, { header: 'Module 3 (processor to processor)', width: 0.35 }], [
    ['Data exporter', `${controller}, acting for itself`, `${controller}, acting for schools that are controllers`],
    ['Data importer', `${processor} and its affiliates listed in Annex III`, `${processor} and its affiliates listed in Annex III`],
    ['Activities relevant to the transfer', 'Support, security monitoring, and disaster recovery for the Services', 'Same, on the schools\' instructions passed through Customer'],
    ['Categories of data subjects and data', 'As in Annex I', 'As in Annex I'],
    ['Frequency', 'Continuous', 'Continuous'],
    ['Onward transfers', 'Only to Sub-processors in Annex III', 'Only to Sub-processors in Annex III'],
    ['Supplementary measures', 'Encryption with keys held in the EEA; pseudonymised identifiers in support tooling; access logging', 'Same'],
    ['Docking clause', 'Applies', 'Applies'],
  ], { size: 8.5 });
  flow.heading('Annex VIII - Processing Locations', { level: 2 });
  flow.table([{ header: 'Facility', width: 0.3 }, { header: 'Operator', width: 0.26 }, { header: 'Use', width: 0.26 }, { header: 'Certifications', width: 0.18 }], [
    ['Dublin (two availability zones)', 'Starling Compute Ltd.', 'Primary production', 'ISO 27001, SOC 2'],
    ['Frankfurt', 'Starling Compute Ltd.', 'Read replicas, search', 'ISO 27001, SOC 2'],
    ['Ashburn, Virginia', 'Starling Compute Inc.', 'Disaster recovery (warm standby)', 'ISO 27001, SOC 2'],
    ['Luxembourg', 'Hazelmoor Backup Services S.A.', 'Encrypted backups', 'ISO 27001'],
    ['Copenhagen', 'Amberline Data Warehousing ApS', 'Pseudonymised analytics', 'ISO 27001, SOC 2'],
    ['Vienna', 'Saltmarsh Logging GmbH', 'Log storage', 'ISO 27001'],
    ['Singapore (support desk)', 'Silverbirch Analytics Pte. Ltd.', 'Remote support, read-only', 'ISO 27001'],
    ['Dublin (Skylark office)', processor, 'Engineering and support; no data stored', 'ISO 27001 scope'],
  ], { size: 8.5 });
  flow.heading('Annex VII - Data Subject Request Handling', { level: 2 });
  flow.table([{ header: 'Step', width: 0.08 }, { header: 'Action', width: 0.62 }, { header: 'Target time', width: 0.3 }], [
    ['1', 'Request received by Skylark is logged and forwarded to Customer\'s privacy contact', 'Within 5 business days'],
    ['2', 'Customer confirms the requester\'s identity and authority through the school', 'Customer\'s responsibility'],
    ['3', 'Customer runs the self-service export or deletion tool, or asks Skylark to act', 'As needed'],
    ['4', 'Skylark completes any requested action that the tools cannot perform', 'Within 10 business days of Customer\'s instruction'],
    ['5', 'Skylark confirms completion, including deletion from search indexes', 'Same day as completion'],
    ['6', 'Backups containing the data expire on their normal schedule', 'Within 35 days'],
    ['7', 'Request and outcome recorded for accountability', 'Retained 3 years'],
  ], { size: 8.5 });
  flow.heading('Annex IX - Personal Data Breach Notification Content', { level: 2 });
  flow.paragraph('Each notice under Section 6 will contain the following, updated as facts are established. Items not yet known are marked "under investigation" with an expected date.', { size: 9.5 });
  flow.table([{ header: 'Field', width: 0.32 }, { header: 'Content', width: 0.68 }], [
    ['Reference', 'Skylark incident number and the time Skylark became aware of the breach (UTC)'],
    ['Summary', 'What happened, how it was detected, and whether it is contained'],
    ['Systems affected', 'Service components, environments, and Sub-processors involved'],
    ['Data affected', 'Categories of Customer Personal Data and whether any was encrypted, pseudonymised, or aggregated'],
    ['Data subjects', 'Categories and approximate numbers, by school where known'],
    ['Likely consequences', 'Risks to students, parents, and staff, including any risk of identity misuse'],
    ['Measures taken', 'Containment, credential resets, forensic steps, and law enforcement contact'],
    ['Measures proposed', 'Remediation plan with owners and dates'],
    ['Assistance offered', 'Draft notices for schools and parents, call-centre support, and credit monitoring if relevant'],
    ['Contact', 'Name and 24-hour telephone number of Skylark\'s incident lead'],
  ], { size: 8.5 });
  flow.paragraph('Skylark will deliver the first notice by email to Customer\'s registered security contact and by telephone to the on-call number in the Customer portal, and will hold a call with Customer within twelve hours of the first notice if Customer asks.', { size: 9.5 });
  const pages = flow.finish();
  if (pages.length !== 10) throw new Error(`${id}: expected 10 pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Ten-page data processing addendum with security annex and sub-processor list',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_10', 'contract', 'irrelevant_names', 'referenced_agreement', 'competing_dates', 'table', 'information_dense'],
    notes: `The DPA Effective Date (${longDate(effective)}) is defined in the first sentence on page 1, in the same sentence as the master agreement's date (${longDate(msaDate)}). The signatures carry ${longDate(signedProcessor)} and ${longDate(signedController)}, and a transfer impact assessment is dated ${longDate(tiaDate)}. Annex III lists ${SUBPROCESSORS.length} sub-processors - company names that are not parties; Annex II lists ${CONTROLS.length} named security controls.`,
    gold: gold({
      type: 'Data Processing Addendum',
      acceptableTypes: ['Data Processing Agreement', 'DPA'],
      date: effective,
      role: 'effective',
      forbiddenDates: [[msaDate, 'date of the master subscription agreement'], [signedProcessor, 'processor signature date'], [signedController, 'customer signature date'], [tiaDate, 'transfer impact assessment update']],
      parties: [controller, processor],
      relation: 'between',
      roles: [[controller, 'client'], [processor, 'provider']],
      forbiddenParties: SUBPROCESSORS.slice(0, 4).map(([name]) => [name, 'sub-processor listed in Annex III']),
      facts: [['sub-processor', 'sub-processors', 'Sub-processor'], ['student', 'students', 'classroom'], ['48 hours', 'forty-eight hours', 'breach']],
      subjectTerms: ['data processing', 'sub-processors', 'GDPR', 'student data'],
      readiness: 'ready',
      dateText: [longDate(effective)],
    }),
  });
}
