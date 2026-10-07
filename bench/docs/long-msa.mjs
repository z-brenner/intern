/// A twelve-page master services agreement between a logistics company and
/// its systems integrator. The first page says only that the agreement is
/// entered into "as of the Effective Date (as defined in Schedule D)"; the
/// date itself is stated once, in the definitions schedule on page 9,
/// between the start of the first statement of work and the end of the
/// initial term. The recitals on page 1 date the request for proposal and
/// the integrator's proposal; the signature page dates both signatures.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { longDate, money } from '../lib/format.mjs';
import { people } from '../lib/names.mjs';
import { digitalPdf, readingSnippets, result } from './common.mjs';
import { blocks, chain, expectPages, fillTo, tableBlocks } from './dense.mjs';

const CLIENT = 'Halcyon Ridge Logistics, Inc.';
const CONTRACTOR = 'Quillmont Systems Integration LLC';
const AGREEMENT_NUMBER = 'QSI-MSA-2026-014';

/// Subcontractors the integrator may use: companies named in the
/// agreement that are not parties to it.
const SUBCONTRACTORS = [
  ['Larchmont Field Services Inc.', 'On-site device installation and break-fix at the Western region sites'],
  ['Tidewell Labeling Systems LLC', 'Thermal printer supply, calibration and label stock'],
  ['Brackenfold Data Centers LLC', 'Colocation of the integration servers at its Columbus facility'],
  ['Osprey Point Translation Services', 'Spanish and French operator documentation'],
  ['Fernhill Network Engineering LLC', 'Wireless surveys and access point installation'],
];

/// The service catalogue: code, name, and what the service does.
const SERVICES = [
  ['WMS-01', 'Warehouse management configuration', 'Configuration of receiving, putaway, replenishment, picking, packing and shipping workflows in the Client warehouse management system for each Site, including location master data, slotting rules and carton label formats.'],
  ['WMS-02', 'Wave and labor planning', 'Design and tuning of wave release rules, pick-path sequencing and labor standards, with a monthly review of planned against actual units per hour by Site.'],
  ['TMS-01', 'Transportation management integration', 'Interfaces between the warehouse management system and the Client transportation management system for load building, carrier tendering, dock appointments and proof of delivery.'],
  ['EDI-01', 'Trading partner onboarding', 'Mapping, testing and certification of EDI 940, 943, 944, 945 and 856 documents for new retailers and suppliers, including vendor compliance label testing.'],
  ['EDI-02', 'EDI monitoring', 'Monitoring of inbound and outbound document queues, reprocessing of failed transactions, and a daily exception report to the Client integration desk.'],
  ['RF-01', 'Mobile device management', 'Enrollment, configuration, patching and remote support of handheld scanners, vehicle-mounted terminals and wearable ring scanners.'],
  ['RF-02', 'Wireless network support', 'Surveys, access point configuration and roaming tuning for warehouse wireless networks, with heat maps delivered after each layout change.'],
  ['PRN-01', 'Label printing', 'Installation and support of thermal label printers, print servers and label templates, including compliance labels required by Client customers.'],
  ['AUT-01', 'Conveyor and sortation interfaces', 'Interfaces to conveyor controls, print-and-apply stations and sorters, including divert confirmations and no-read handling.'],
  ['AUT-02', 'Goods-to-person integration', 'Integration of autonomous mobile robots and shuttle systems with task interleaving, battery management alerts and fault recovery procedures.'],
  ['DAT-01', 'Operational reporting', 'Daily and weekly dashboards for inbound, inventory accuracy, order cycle time, on-time shipping and labor productivity, delivered to Site managers by 07:00 local time.'],
  ['DAT-02', 'Inventory reconciliation', 'Nightly reconciliation of on-hand balances between the warehouse management system and the Client enterprise resource planning system, with variance research tickets.'],
  ['SEC-01', 'Access administration', 'Provisioning and removal of user accounts and roles in the systems Contractor supports, within the times set out in Schedule E.'],
  ['SUP-01', 'Service desk', 'Telephone, portal and chat support for Client users in English, Spanish and French, with ticket triage against the priorities in Schedule B.'],
  ['SUP-02', 'Hypercare', 'Extended on-site and remote support for six weeks after each Site go-live, with a daily defect review and a closure report.'],
  ['TRN-01', 'Operator training', 'Classroom and floor training for receivers, pickers, packers and leads, with job aids and a competency check signed by the Site trainer.'],
  ['TRN-02', 'Administrator training', 'Training for Client system administrators on configuration, user management, reports and first-line troubleshooting.'],
  ['PMO-01', 'Program management', 'A program office that keeps the integrated plan, risk and issue logs, and change log, and runs the governance meetings in Schedule F.'],
  ['DR-01', 'Disaster recovery', 'Recovery runbooks, an annual failover test of the integration servers, and restoration of interfaces within the recovery time objective in Schedule B.'],
  ['ENG-01', 'Release engineering', 'Packaging, regression testing and deployment of configuration and interface changes in the monthly release window, with a back-out plan for each release.'],
];

/// The Sites, each with its region, city, building type and shifts.
const SITES = [
  ['CMH-1', 'Central', 'Groveport, Ohio', 'Ambient distribution center, 640,000 sq ft', 3],
  ['CMH-2', 'Central', 'Obetz, Ohio', 'Returns processing center, 210,000 sq ft', 2],
  ['IND-1', 'Central', 'Plainfield, Indiana', 'Ambient distribution center, 820,000 sq ft', 3],
  ['CHI-1', 'Central', 'Joliet, Illinois', 'Cross-dock, 180,000 sq ft', 2],
  ['ATL-1', 'Southeast', 'McDonough, Georgia', 'Ambient distribution center, 710,000 sq ft', 3],
  ['ATL-2', 'Southeast', 'Union City, Georgia', 'Temperature-controlled center, 260,000 sq ft', 3],
  ['CLT-1', 'Southeast', 'Statesville, North Carolina', 'E-commerce fulfillment center, 450,000 sq ft', 3],
  ['JAX-1', 'Southeast', 'Jacksonville, Florida', 'Import deconsolidation center, 300,000 sq ft', 2],
  ['DFW-1', 'South Central', 'Lancaster, Texas', 'Ambient distribution center, 760,000 sq ft', 3],
  ['DFW-2', 'South Central', 'Wilmer, Texas', 'Bulk storage center, 390,000 sq ft', 1],
  ['HOU-1', 'South Central', 'Baytown, Texas', 'Import deconsolidation center, 520,000 sq ft', 2],
  ['PHX-1', 'Western', 'Goodyear, Arizona', 'E-commerce fulfillment center, 410,000 sq ft', 3],
  ['RNO-1', 'Western', 'Sparks, Nevada', 'Ambient distribution center, 600,000 sq ft', 2],
  ['ONT-1', 'Western', 'Ontario, California', 'Ambient distribution center, 950,000 sq ft', 3],
  ['ONT-2', 'Western', 'Fontana, California', 'Temperature-controlled center, 230,000 sq ft', 3],
  ['SEA-1', 'Western', 'Kent, Washington', 'Cross-dock, 150,000 sq ft', 2],
  ['ALL-1', 'Northeast', 'Bethlehem, Pennsylvania', 'Ambient distribution center, 880,000 sq ft', 3],
  ['NJ-1', 'Northeast', 'Cranbury, New Jersey', 'E-commerce fulfillment center, 500,000 sq ft', 3],
];

/// The rate card: role, and its rate in cents per hour.
const ROLES = [
  ['Program director', 26500], ['Engagement manager', 23800], ['Solution architect', 22400], ['Integration architect', 21600],
  ['Senior WMS consultant', 19800], ['WMS consultant', 16900], ['EDI developer', 15800], ['Interface developer', 16400],
  ['Automation controls engineer', 18900], ['Data engineer', 17200], ['Reporting analyst', 13600], ['Test lead', 15200],
  ['Test analyst', 11800], ['Release engineer', 14900], ['Network engineer', 15600], ['Mobile device technician', 9800],
  ['Field technician', 9200], ['Service desk analyst (tier 1)', 7400], ['Service desk analyst (tier 2)', 9600], ['Trainer', 10800],
  ['Technical writer', 9900], ['Project coordinator', 8700],
];

/// Security controls: identifier, control, evidence, and how often.
const CONTROLS = [
  ['AC-1', 'Unique user identifiers; no shared accounts in production systems', 'Quarterly account listing', 'Quarterly'],
  ['AC-2', 'Access removed within one business day of a role change or departure', 'Joiner-mover-leaver ticket sample', 'Monthly'],
  ['AC-3', 'Privileged access granted through a time-limited request and approval', 'Privileged access log', 'Monthly'],
  ['AC-4', 'Multi-factor authentication for every remote and administrative login', 'Identity provider policy export', 'Quarterly'],
  ['AU-1', 'Security event logs kept for 400 days and protected from alteration', 'Log retention configuration', 'Semi-annual'],
  ['AU-2', 'Alerts on failed logins, privilege changes and interface credential use', 'Alert rule list and sample alerts', 'Quarterly'],
  ['CM-1', 'Configuration changes approved through the change board in Schedule F', 'Change records for the period', 'Monthly'],
  ['CM-2', 'Baseline hardening standard applied to integration servers', 'Compliance scan summary', 'Quarterly'],
  ['CP-1', 'Backups of interface configuration and message archives every night', 'Backup job report', 'Monthly'],
  ['CP-2', 'Annual failover test of the integration environment', 'Test report with recovery times', 'Annual'],
  ['IR-1', 'Security incidents reported to Client within twenty-four hours of discovery', 'Incident register', 'As occurs'],
  ['IR-2', 'Tabletop exercise with the Client security team', 'Exercise summary and actions', 'Annual'],
  ['MA-1', 'Remote support sessions recorded and limited to approved tools', 'Session tool configuration', 'Semi-annual'],
  ['MP-1', 'Removable media disabled on servers that process Client Data', 'Endpoint policy export', 'Semi-annual'],
  ['PE-1', 'Badge access to server rooms reviewed against an approved list', 'Access review sign-off', 'Quarterly'],
  ['PS-1', 'Background checks for personnel with access to Client Data', 'Attestation by Contractor HR', 'Annual'],
  ['RA-1', 'Risk assessment of new interfaces before they go live', 'Assessment records', 'As occurs'],
  ['SA-1', 'Secure development training for developers', 'Training completion report', 'Annual'],
  ['SC-1', 'Encryption of Client Data in transit with TLS 1.2 or later', 'Configuration scan', 'Quarterly'],
  ['SC-2', 'Encryption of Client Data at rest with keys held in a managed key service', 'Key service configuration', 'Semi-annual'],
  ['SI-1', 'Critical vulnerabilities remediated within fifteen days', 'Vulnerability aging report', 'Monthly'],
  ['SI-2', 'Malware protection on every workstation used to support Client', 'Endpoint coverage report', 'Monthly'],
  ['SR-1', 'Subcontractors bound by security terms no less protective than Schedule E', 'Subcontractor register', 'Annual'],
  ['PT-1', 'Personal data processed only for the Services and deleted at the end of the Term', 'Deletion certificate', 'At exit'],
];

/// What Client provides before a service can start at a Site.
const INPUTS = [
  ['WMS-01, WMS-02', 'item master with dimensions and weights, location master, the Site\'s current slotting and a year of order history'],
  ['TMS-01', 'carrier list with service levels, dock door assignments, and appointment rules'],
  ['EDI-01, EDI-02', 'trading partner specifications, test identifiers, and a contact at each partner'],
  ['RF-01, RF-02', 'device inventory with serial numbers, floor plans for the wireless survey, and network credentials for device management'],
  ['PRN-01', 'printer inventory, label templates, and customer compliance manuals'],
  ['AUT-01, AUT-02', 'controls documentation for conveyors, sorters and robots, and access for the automation vendors'],
  ['DAT-01, DAT-02', 'reporting requirements signed off by the Site general manager and finance, and read access to the ERP inventory tables'],
  ['SEC-01', 'role matrix approved by Client security, and the joiner-mover-leaver process contacts'],
  ['SUP-01, SUP-02', 'escalation lists for each Site, and the hypercare exit criteria agreed with operations'],
  ['TRN-01, TRN-02', 'training rooms, trainee schedules, and the Site\'s standard operating procedures'],
  ['PMO-01, ENG-01', 'the Client release calendar, change board membership, and test environment access'],
  ['DR-01', 'recovery time and recovery point objectives approved by Client\'s business continuity office'],
];

/// Expense rules.
const EXPENSES = [
  'Air travel is economy class, booked at least fourteen days ahead where the work allows, through Client\'s travel agency.',
  'Lodging is reimbursed up to the per-diem rate for the Site\'s county published by the General Services Administration.',
  'Meals and incidental expenses are reimbursed at seventy-five percent of the federal per-diem rate on travel days and in full on other days.',
  'Mileage for personal vehicles is reimbursed at the rate the Internal Revenue Service publishes for the year of travel.',
  'Rental cars are intermediate class or smaller, shared where two or more people travel to the same Site.',
  'No expense is reimbursed for travel within fifty miles of the traveller\'s home office.',
  'Equipment, software and tools that Contractor uses for other customers are not reimbursable.',
  'Expenses over $500 for a single item need the Client program manager\'s written approval in advance.',
  'Receipts are required for every item over $25 and must be submitted within sixty days of the expense.',
  'Client will not pay for alcohol, entertainment, upgrades, laundry on trips shorter than five nights, or traffic fines.',
];

/// Training modules: module, audience, hours.
const TRAINING = [
  ['Receiving and putaway on handhelds', 'receivers and lift drivers', 4], ['Replenishment and cycle counting', 'inventory control associates', 3],
  ['Wave picking and pick-to-tote', 'pickers and pick leads', 3], ['Pack-out stations and compliance labels', 'packers', 2],
  ['Shipping, load building and dock appointments', 'shipping clerks and yard drivers', 4], ['Returns grading and disposition', 'returns associates', 3],
  ['Exception handling and short picks', 'shift leads', 2], ['Reports and dashboards', 'Site managers and analysts', 2],
  ['Printer and device first-line support', 'Site IT coordinators', 3], ['Conveyor and sorter fault recovery', 'maintenance technicians', 4],
];

/// The first year's releases: month and scope.
const RELEASES = [
  ['April', 'Central region configuration baseline and EDI 940/945 for the first three retailers'],
  ['May', 'cycle count redesign and nightly inventory reconciliation'],
  ['June', 'Southeast region rollout, wave one'], ['July', 'Southeast region rollout, wave two, and returns grading'],
  ['August', 'Peak Season capacity changes and printer fleet refresh'], ['September', 'freeze rehearsal and disaster recovery failover test'],
  ['October to December', 'no releases (Peak Season) except emergency fixes'], ['January', 'post-peak defect backlog and dashboard changes'],
  ['February', 'South Central region rollout'], ['March', 'Western region design and goods-to-person integration pilot'],
];

/// Site readiness checks and who owns them.
const READINESS = [
  ['Wireless survey and access point placement', 'Contractor'], ['Handheld and vehicle terminal inventory', 'Site IT'],
  ['Printer fleet and label stock', 'Site IT'], ['Item and location master data load', 'Client'], ['Carrier and dock door set-up', 'Client'],
  ['EDI partners certified for the Site', 'Contractor'], ['Conveyor and sorter interface tests', 'Contractor'], ['User roles and accounts', 'Client'],
  ['Operator training completed', 'Site trainer'], ['Cutover rehearsal', 'Contractor'],
];

/// Exit Assistance, step by step.
const EXIT_STEPS = [
  'deliver a transition plan within thirty days, with owners, dates and acceptance criteria for each service;',
  'hold knowledge transfer sessions for each region, recorded and indexed by topic;',
  'deliver every configuration, interface map, script, runbook and open ticket in a format the successor can load;',
  'run the services in parallel with the successor until it accepts each service in writing;',
  'transfer the integration server images and message archives, then certify deletion of Client Data from its own systems;',
  'answer the successor\'s questions for ninety days after the last service transfers, at the rates in Schedule C;',
  'cooperate with Client\'s auditors on any review of the transition;',
  'return badges, devices and documentation at each Site within ten Business Days of leaving it.',
];

/// Governance reports: name, audience, cadence.
const REPORTS = [
  ['Service Level report with credits earned', 'the steering committee', 'by the tenth Business Day of each month'],
  ['Regional incident and problem review', 'each regional operations review', 'every two weeks'],
  ['Release notes and test summary', 'the change board', 'before each monthly release window'],
  ['Security control evidence pack', 'Client\'s information security office', 'each quarter'],
  ['Capacity forecast for Peak Season', 'the vice president of distribution systems', 'by the first Monday in August'],
  ['Subcontractor performance summary', 'the steering committee', 'twice a year'],
  ['Roadmap of Client-requested enhancements', 'the steering committee', 'each quarter'],
  ['Training completion by Site', 'Site general managers', 'monthly during any rollout'],
];

export function masterServicesAgreement12() {
  const id = 'msa-effective-date-in-definitions-12p';
  const rng = Rng.from(id);
  const effective = '2026-03-02';
  const rfp = '2025-09-30';
  const proposal = '2025-11-18';
  const signedContractor = '2026-03-09';
  const signedClient = '2026-03-11';
  const sowStart = '2026-04-06';
  const termEnd = '2029-03-01';
  const flow = new Flow({
    face: 'serif', fontSize: 10.5, leading: 1.35, margins: { top: 64, bottom: 66 }, keep: [CLIENT, CONTRACTOR, ...SUBCONTRACTORS.map(([name]) => name)],
    header: (page, { number }) => {
      if (number > 1) page.text(72, 44, `Master Services Agreement No. ${AGREEMENT_NUMBER}`, { face: 'sans', size: 7.5, grey: 0.4 });
    },
    footer: (page, { number, total }) => page.textRight(540, 760, `Page ${number} of ${total}`, { face: 'sans', size: 7.5, grey: 0.4 }),
  });

  flow.heading('MASTER SERVICES AGREEMENT', { level: 1, align: 'center', size: 16 });
  flow.paragraph(`Agreement No. ${AGREEMENT_NUMBER}`, { face: 'sans', size: 9.5, align: 'center', after: 12 });
  flow.paragraph(`This Master Services Agreement (this "Agreement") is entered into as of the Effective Date (as defined in Schedule D) by and between ${CLIENT}, a Delaware corporation with offices at 4100 Alum Creek Drive, Columbus, Ohio 43207 ("Client"), and ${CONTRACTOR}, an Ohio limited liability company with offices at 77 Brushy Run Road, Dublin, Ohio 43017 ("Contractor"). Client and Contractor are each a "Party" and together the "Parties".`);
  flow.paragraph('RECITALS', { face: 'sans-bold', size: 10, after: 4 });
  flow.paragraph(`A. Client operates a network of distribution, fulfillment and cross-dock facilities and issued a request for proposal for warehouse systems integration services on ${longDate(rfp)} (the "RFP").`);
  flow.paragraph(`B. Contractor responded with its proposal dated ${longDate(proposal)} (the "Proposal"), which Client selected after demonstrations at three of its facilities.`);
  flow.paragraph('C. The Parties wish to set out the terms on which Contractor will provide the Services described in Schedule A and in Statements of Work entered into under this Agreement. The Proposal is not part of this Agreement except as a Statement of Work incorporates a part of it by reference.');
  flow.paragraph('NOW, THEREFORE, in consideration of the mutual promises in this Agreement, the Parties agree as follows:');

  const articles = [
    ['Structure', [
      'This Agreement sets out the general terms on which Contractor provides Services. Each project or recurring service is described in a Statement of Work signed by both Parties, which states its scope, deliverables, Sites, milestones, acceptance criteria, fees and any terms that differ from this Agreement. If a Statement of Work conflicts with this Agreement, this Agreement controls unless the Statement of Work expressly identifies the section it overrides.',
      'Schedules A through F form part of this Agreement. Capitalized terms have the meanings given in Schedule D or where they are first defined in the body of this Agreement.',
    ]],
    ['Services and Personnel', [
      'Contractor will perform the Services with personnel who have the skills and certifications stated in the applicable Statement of Work, in a professional manner consistent with good industry practice for warehouse systems integration. Contractor will assign the Key Personnel listed in Schedule F and will not remove or replace any of them during the first twelve months of their assignment without Client\'s consent, except for illness, departure from Contractor, or termination for cause.',
      'Contractor\'s personnel working at a Site will follow the Site\'s safety rules, including powered industrial truck traffic plans, high-visibility clothing, lock-out procedures for conveyor work, and food-grade hygiene rules at temperature-controlled Sites. Client may require the removal of any individual who breaches those rules.',
    ]],
    ['Change Control', [
      'Either Party may request a change to a Statement of Work by a written change request describing the change and its reason. Within ten business days Contractor will provide an impact assessment covering scope, schedule, fees, Service Levels and risks. No change is binding until both Parties sign a change order. Contractor will not reduce testing or security activities to absorb a change without Client\'s written approval.',
    ]],
    ['Fees and Invoices', [
      'Client will pay the fees in each Statement of Work. Time-and-materials work is charged at the rates in Schedule C, which are fixed for the first two Contract Years and may then increase once each Contract Year by no more than the lesser of three percent and the change in the Employment Cost Index for private industry professional and technical services. Fixed-fee work is invoiced on the milestones in the Statement of Work.',
      'Contractor will invoice monthly in arrears, itemizing hours by role, Site and Statement of Work, with approved expenses supported by receipts. Travel is reimbursed under the Client travel policy and only when pre-approved. Undisputed amounts are due forty-five days after Client receives a correct invoice. Client may withhold disputed amounts in good faith and will pay any amount found due within fifteen days after the dispute is resolved.',
    ]],
    ['Service Levels and Credits', [
      'Contractor will meet the Service Levels in Schedule B. If Contractor misses a Service Level in a month, it will provide a root-cause analysis within ten business days and a remediation plan. Service credits are calculated as set out in Schedule B, are capped at twelve percent of the monthly recurring fees, and are Client\'s sole monetary remedy for the miss itself, without limiting Client\'s right to terminate for repeated failures.',
    ]],
    ['Client Responsibilities', [
      'Client will provide timely access to its Sites, systems, data and subject-matter experts, make decisions within the times stated in a Statement of Work, and maintain licenses for the third-party software Contractor configures. Contractor is excused from a delay to the extent caused by Client\'s failure to perform these responsibilities, if Contractor gives prompt written notice of the failure and its expected effect.',
    ]],
    ['Intellectual Property', [
      'Deliverables created specifically for Client, including configurations, interface maps, reports and documentation, are owned by Client on payment. Contractor keeps ownership of its pre-existing tools, accelerators and know-how and grants Client a perpetual, royalty-free license to use any of them that are embedded in a Deliverable for Client\'s internal business.',
    ]],
    ['Confidentiality and Data', [
      'Each Party will protect the other\'s Confidential Information with at least reasonable care and use it only to perform or receive the Services. Client Data remains Client\'s property. Contractor will process Client Data only on Client\'s instructions, in compliance with the security controls in Schedule E, and will return or destroy it at the end of the Term, certifying destruction in writing.',
    ]],
    ['Warranties', [
      'Contractor warrants that the Services will be performed in a professional manner and that each Deliverable will conform to its specification for ninety days after acceptance. Contractor will correct non-conforming Deliverables at no charge. These warranties are exclusive and replace all implied warranties, including merchantability and fitness for a particular purpose.',
    ]],
    ['Indemnities', [
      'Contractor will defend and indemnify Client against third-party claims that a Deliverable infringes intellectual property rights, and against claims for bodily injury or property damage caused by Contractor\'s personnel at a Site. Client will defend and indemnify Contractor against claims arising from materials Client provides. The indemnified Party must give prompt notice and reasonable cooperation.',
    ]],
    ['Limitation of Liability', [
      'Except for breaches of confidentiality, indemnity obligations, and gross negligence or willful misconduct, neither Party is liable for indirect, consequential or punitive damages, and each Party\'s total liability in any Contract Year is limited to the fees paid or payable under this Agreement in the twelve months before the event giving rise to the claim.',
    ]],
    ['Term and Termination', [
      'This Agreement begins on the Effective Date and continues for the Initial Term, after which it renews for successive one-year Renewal Terms unless either Party gives notice of non-renewal at least ninety days before the end of the then-current term. Client may terminate this Agreement or any Statement of Work for convenience on sixty days\' notice, paying for Services performed through termination and any committed non-cancellable costs.',
      'Either Party may terminate for material breach not cured within thirty days after notice. On expiry or termination Contractor will provide the Exit Assistance described in Schedule F for up to six months at the rates in Schedule C.',
    ]],
    ['Insurance', [
      'Contractor will maintain commercial general liability insurance of at least $2,000,000 per occurrence, workers\' compensation as required by law, automobile liability of $1,000,000, professional and technology errors and omissions insurance of $5,000,000, and cyber liability insurance of $5,000,000, each with insurers rated A- or better, and will provide certificates on request.',
    ]],
    ['General', [
      'This Agreement is governed by the laws of the State of Ohio. Notices must be in writing and delivered to the addresses on the signature page. Neither Party may assign this Agreement without the other\'s consent, except to a successor of its business. This Agreement, its Schedules and the Statements of Work are the entire agreement on their subject and may be amended only in a writing signed by both Parties.',
    ]],
  ];
  articles.forEach(([title, paragraphs], index) => {
    flow.paragraph(`${index + 1}. ${title.toUpperCase()}`, { face: 'sans-bold', size: 10, before: 4, after: 3, keepWithNext: 30 });
    paragraphs.forEach((text, item) => flow.paragraph(`${index + 1}.${item + 1} ${text}`));
  });

  // Schedules A to C: as many blocks as it takes to fill pages up to 8.
  const pick = rng.fork('schedules');
  const windows = ['24x7', 'Mon-Sat 05:00-23:00 Site time', 'Mon-Fri 07:00-19:00 ET', 'Site operating shifts'];
  const catalogue = blocks([...SERVICES, null], (service, index) => {
    if (index === 0) {
      flow.heading('SCHEDULE A - SERVICE CATALOGUE', { level: 2 });
      flow.paragraph('Each service below may be ordered for one or more Sites in a Statement of Work. The service window is when Contractor staffs the service; requests outside it are handled at the next start of the window unless they are Priority 1. Each service carries the minimum commitment, in hours per quarter across all Sites, shown in the table that closes this Schedule.', { size: 9.5 });
    }
    if (service) {
      const [code, name, description] = service;
      flow.paragraph([{ text: `A.${index + 1} ${code} - ${name}. `, face: 'sans-bold' }, { text: description }], { size: 9.5, after: 4 });
      return;
    }
    flow.table([{ header: 'Service', width: 0.16 }, { header: 'Window', width: 0.44 }, { header: 'Minimum', width: 0.2, align: 'right' }, { header: 'Lead', width: 0.2 }],
      SERVICES.map(([code]) => [code, windows[pick.int(0, windows.length - 1)], `${pick.int(2, 12) * 10} h`, pick.pick(['Contractor PMO', 'Site IT', 'Regional lead', 'Solution architect'])]), { size: 8.5 });
  });
  const levels = blocks([0, 1], (part) => {
    if (part === 0) {
      flow.heading('SCHEDULE B - SITES AND SERVICE LEVELS', { level: 2 });
      flow.paragraph('Priority 1 is a stop to shipping or receiving at a Site; Priority 2 is a material degradation with a workaround; Priority 3 is any other incident. Response is measured from ticket creation to an engineer working the incident; restoration to the Site confirming normal operation. Priority 2 response is one hour at every Site, and Priority 2 restoration four times the Priority 1 restoration. Each miss earns a credit of 2% of the Site\'s monthly fee, doubled for a second miss of the same Service Level in a quarter.', { size: 9.5 });
      flow.table([{ header: 'Site', width: 0.1 }, { header: 'Region', width: 0.13 }, { header: 'Location', width: 0.2 }, { header: 'Building', width: 0.3 }, { header: 'Shifts', width: 0.08, align: 'right' }],
        SITES.map(([code, region, city, building, shifts]) => [code, region, city, building, String(shifts)]), { size: 8.5 });
      return;
    }
    flow.table([{ header: 'Site', width: 0.1 }, { header: 'P1 response', width: 0.15, align: 'right' }, { header: 'P1 restore', width: 0.14, align: 'right' }, { header: 'Availability', width: 0.15, align: 'right' }, { header: 'Resident technician', width: 0.26 }, { header: 'Monthly fee', width: 0.2, align: 'right' }],
      SITES.map(([code, , , , shifts]) => [code, `${[15, 20, 30][pick.int(0, 2)]} min`, `${[2, 3, 4][pick.int(0, 2)]} h`, ['99.5%', '99.7%', '99.9%'][pick.int(0, 2)], shifts === 3 ? 'every shift' : shifts === 2 ? 'both shifts' : 'day shift', money(pick.int(18, 64) * 50000)]), { size: 8.5 });
  });
  const dependencies = blocks(INPUTS, ([codes, inputs], index) => {
    if (index === 0) {
      flow.heading('SCHEDULE A-1 - CLIENT INPUTS BY SERVICE', { level: 2 });
      flow.paragraph('What Client provides before Contractor can start a service at a Site, at least the number of Business Days shown before the start date in the Statement of Work. A missing input excuses the affected Service Levels until it is provided.', { size: 9.5 });
    }
    flow.paragraph([{ text: `${codes}: `, face: 'sans-bold' }, { text: `${inputs} (${pick.int(2, 8)} Business Days).` }], { size: 9, after: 3 });
  });
  const rates = blocks([0], () => {
    flow.heading('SCHEDULE C - RATE CARD', { level: 2 });
    flow.paragraph('Hourly rates for time-and-materials work, fixed for the first two Contract Years. Overtime approved in advance is charged at 1.5 times the rate; work on Client-designated peak days at 1.25 times.', { size: 9.5 });
    flow.table([{ header: 'Role', width: 0.46 }, { header: 'Onshore', width: 0.18, align: 'right' }, { header: 'Nearshore', width: 0.18, align: 'right' }, { header: 'Minimum', width: 0.18, align: 'right' }],
      ROLES.map(([role, cents]) => [role, money(cents), money(Math.round(cents * 0.62 / 100) * 100), `${pick.int(1, 4) * 4} hours`]), { size: 8.5 });
  });
  const peak = blocks([0], () => {
    flow.heading('SCHEDULE B-1 - PEAK SEASON COVERAGE', { level: 2 });
    flow.paragraph('During Peak Season Contractor adds the coverage below at each Site, at the rates in Schedule C, freezes releases unless the Site general manager approves an emergency change, runs a war room at each Site in the two weeks around Cyber Monday, and holds a 06:30 readiness call with the Site leads every day.', { size: 9.5 });
    flow.table([{ header: 'Site', width: 0.12 }, { header: 'Extra technicians', width: 0.2, align: 'right' }, { header: 'Weekends and holidays', width: 0.44 }, { header: 'War room', width: 0.24, align: 'right' }],
      SITES.map(([code]) => [code, String(pick.int(1, 4)), pick.pick(['on-call engineer on site', 'remote cover from the Dublin service desk', 'on-call engineer and remote cover']), `${pick.int(2, 6)} people`]), { size: 8.5 });
  });
  const expenses = blocks(EXPENSES, (rule, index) => {
    if (index === 0) {
      flow.heading('SCHEDULE C-1 - EXPENSES', { level: 2 });
      flow.paragraph('Reimbursable expenses are charged at cost, without markup, under the rules below and the Client travel policy.', { size: 9.5 });
    }
    flow.paragraph(`C-1.${index + 1} ${rule}`, { size: 9, indent: 12, after: 3 });
  });
  // Site readiness, Site by Site and check by check: as much as fills page 8.
  const readinessRows = SITES.flatMap(([code]) => READINESS.map(([check, owner]) => {
    const count = pick.int(3, 140);
    const status = pick.pick(['complete', 'complete', 'in progress', 'not started', 'complete, retest due']);
    return [code, check, String(count), status, owner];
  }));
  const readiness = tableBlocks(flow, [{ header: 'Site', width: 0.1 }, { header: 'Readiness check', width: 0.42 }, { header: 'Items', width: 0.1, align: 'right' }, { header: 'Status at signing', width: 0.22 }, { header: 'Owner', width: 0.16 }], readinessRows,
    { title: 'SCHEDULE C-2 - SITE READINESS', intro: 'The readiness of each Site for its first Statement of Work as assessed during the Proposal demonstrations; items are the count of devices, interfaces, labels or users the check covers. Contractor will update this Schedule at each steering committee.' });
  fillTo(flow, 9, chain(catalogue, rates, levels, peak, expenses, dependencies, readiness), { id });

  // Schedule D: the definitions, on page 9.
  flow.heading('SCHEDULE D - DEFINITIONS', { level: 2, before: 4 });
  for (const [term, meaning] of [
    ['Acceptance', 'Client\'s written confirmation that a Deliverable meets its acceptance criteria, or the passing of ten business days after delivery without a written rejection stating the criteria not met'],
    ['Business Day', 'a day other than a Saturday, Sunday or a day on which banks in Columbus, Ohio are closed'],
    ['Client Data', 'all data Client or its customers provide or that the Services generate about Client\'s inventory, orders, shipments, employees and customers'],
    ['Contract Year', 'each period of twelve months beginning on the Effective Date or an anniversary of it'],
    ['Deliverable', 'anything Contractor delivers under a Statement of Work, including configurations, interface maps, code, reports and documentation'],
    ['Effective Date', `${longDate(effective)}`],
    ['Exit Assistance', 'the transition services in Schedule F that move the Services to Client or a successor provider'],
    ['Go-Live', 'the first day a Site processes live orders on a configuration delivered by Contractor'],
    ['Initial Term', `the period beginning on the Effective Date and ending on ${longDate(termEnd)}`],
    ['Key Personnel', 'the individuals named as such in Schedule F and their approved replacements'],
    ['Peak Season', 'the period from the second Monday in October to the second Friday in January, during which Contractor will not deploy releases at a Site without the Site general manager\'s approval'],
    ['Renewal Term', 'each one-year period after the Initial Term for which this Agreement renews'],
    ['Service Levels', 'the measures of performance in Schedule B, and any additional measures in a Statement of Work'],
    ['Services', 'the services in Schedule A and in each Statement of Work, and anything reasonably necessary to perform them'],
    ['Site', 'each facility listed in Schedule B and any facility added to it by a Statement of Work'],
    ['Statement of Work', `a document in the form agreed by the Parties describing a project or recurring service; the first, for the Central region, is to start on ${longDate(sowStart)}`],
    ['Term', 'the Initial Term and any Renewal Terms'],
  ]) flow.paragraph([{ text: `"${term}" `, face: 'sans-bold' }, { text: `means ${meaning}.` }], { size: 9.5, indent: 12, after: 3 });

  // Schedules E and F, then the signature page as page 12.
  const team = people(rng.fork('personnel'), 12);
  const governance = blocks([0], () => {
    flow.heading('SCHEDULE F - GOVERNANCE, KEY PERSONNEL AND SUBCONTRACTORS', { level: 2 });
    flow.paragraph('A steering committee of the Client vice president of distribution systems and Contractor\'s program director meets monthly; an operations review of each region meets every two weeks; a change board meets weekly during any project and monthly otherwise.', { size: 9.5 });
    flow.table([{ header: 'Key person', width: 0.34 }, { header: 'Role', width: 0.4 }, { header: 'Region', width: 0.26 }],
      [['Program director', 'Central'], ['Solution architect', 'All regions'], ['Integration architect', 'All regions'], ['Automation controls engineer', 'Southeast'], ['Senior WMS consultant', 'Western'], ['Senior WMS consultant', 'Northeast'], ['Test lead', 'All regions'], ['Engagement manager', 'South Central']]
        .map(([role, region], index) => [team[index], role, region]), { size: 8.5 });
    flow.paragraph('Approved subcontractors. Contractor remains responsible for the work of each:', { size: 9.5, after: 3 });
    flow.table([{ header: 'Subcontractor', width: 0.38 }, { header: 'Scope', width: 0.62 }], SUBCONTRACTORS, { size: 8.5 });
  });
  const controls = blocks(CONTROLS, ([control, text, evidence, frequency], index) => {
    if (index === 0) {
      flow.heading('SCHEDULE E - SECURITY CONTROLS', { level: 2 });
      flow.paragraph('Contractor will operate the controls below for every system it administers for Client and will provide the evidence listed at the frequency shown. A control failure is reported under IR-1.', { size: 9.5 });
    }
    flow.paragraph([{ text: `${control} `, face: 'sans-bold' }, { text: `${text}. Evidence: ${evidence.toLowerCase()}; ${frequency.toLowerCase()}.` }], { size: 9, after: 3 });
  });
  const exit = blocks(EXIT_STEPS, (step, index) => {
    if (index === 0) flow.paragraph('Exit Assistance. On notice of expiry or termination Contractor will:', { face: 'sans-bold', size: 9.5, before: 4, after: 3 });
    flow.paragraph(`F.${index + 1} ${step}`, { size: 9, indent: 12, after: 3 });
  });
  const reports = blocks(REPORTS, ([name, audience, cadence], index) => {
    if (index === 0) flow.paragraph('Governance reports Contractor will deliver:', { face: 'sans-bold', size: 9.5, before: 4, after: 3 });
    flow.paragraph(`${name}, for ${audience}, ${cadence}.`, { size: 9, indent: 12, after: 3 });
  });
  const curriculum = blocks(TRAINING, ([module, audience, hours], index) => {
    if (index === 0) flow.paragraph('Training curriculum Contractor will deliver at each Site before Go-Live:', { face: 'sans-bold', size: 9.5, before: 4, after: 3 });
    const size = pick.int(6, 14);
    flow.paragraph(`${module} - ${audience}; ${hours} hours; classes of up to ${size}; competency check by the Site trainer.`, { size: 9, indent: 12, after: 2 });
  });
  const releases = blocks(RELEASES, ([month, scope], index) => {
    if (index === 0) flow.paragraph('Release calendar for the first Contract Year (deployments on the third Tuesday of the month, 22:00 to 02:00 Site time):', { face: 'sans-bold', size: 9.5, before: 4, after: 3 });
    flow.paragraph(`${month}: ${scope}.`, { size: 9, indent: 12, after: 2 });
  });
  const auditRows = SITES.flatMap(([code]) => ['AC-2', 'AC-4', 'CM-1', 'SI-1', 'CP-1'].map((control) => [code, control, pick.pick(['Q2', 'Q3', 'Q4', 'Q1']), pick.pick(['Client internal audit', 'Contractor quality office', 'external auditor']), `${pick.int(5, 40)} samples`]));
  const audits = tableBlocks(flow, [{ header: 'Site', width: 0.12 }, { header: 'Control', width: 0.14 }, { header: 'Quarter', width: 0.12 }, { header: 'Tested by', width: 0.38 }, { header: 'Sample', width: 0.24, align: 'right' }], auditRows,
    { title: 'SCHEDULE F-1 - CONTROL TESTING PLAN', intro: 'The controls in Schedule E tested at each Site in the first Contract Year, by whom and on what sample.' });
  fillTo(flow, 12, chain(controls, governance, exit, reports, curriculum, releases, audits), { id });

  flow.paragraph('IN WITNESS WHEREOF, the Parties have signed this Agreement by their authorized representatives.', { before: 6, after: 18 });
  const signers = people(rng.fork('signers'), 2, { exclude: team });
  const top = flow.y;
  [[CLIENT, signers[0], 'Senior Vice President, Supply Chain', signedClient], [CONTRACTOR, signers[1], 'Managing Partner', signedContractor]].forEach(([party, person, title, date], column) => {
    const x = 72 + column * 240;
    [[party, 'sans-bold', 9], [`By: /s/ ${person}`, 'serif', 10], [`Name: ${person}`, 'serif', 10], [`Title: ${title}`, 'serif', 10], [`Date: ${longDate(date)}`, 'serif', 10]]
      .forEach(([text, face, size], line) => flow.page.text(x, top + 12 + line * 16, text, { face, size }));
  });
  flow.y = top + 100;
  flow.paragraph(`Notices to Client: General Counsel, 4100 Alum Creek Drive, Columbus, Ohio 43207. Notices to Contractor: Contracts Director, 77 Brushy Run Road, Dublin, Ohio 43017. Statements of Work under this Agreement cite Agreement No. ${AGREEMENT_NUMBER}.`, { size: 9.5 });

  const pages = flow.finish();
  if (pages.length !== 12) throw new Error(`${id}: expected 12 pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  expectPages(id, text, longDate(effective), [9]);
  expectPages(id, text, CLIENT, [1, 12]);
  expectPages(id, text, CONTRACTOR, [1, 12]);
  const lines = text.flatMap((page) => page.split('\n'));
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Twelve-page master services agreement dated only in its definitions schedule',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_10', 'contract', 'middle_fact', 'competing_dates', 'irrelevant_names', 'information_dense', 'table'],
    notes: `The first page says the agreement is entered into "as of the Effective Date (as defined in Schedule D)"; the Effective Date (${longDate(effective)}) is stated once, in the definitions on page 9, beside the end of the Initial Term (${longDate(termEnd)}) and the start of the first statement of work (${longDate(sowStart)}). Page 1 dates the RFP (${longDate(rfp)}) and the Proposal (${longDate(proposal)}); the signature page on page 12 dates the signatures (${longDate(signedContractor)} and ${longDate(signedClient)}). The parties are named only on pages 1 and 12; Schedule F names five subcontractors that are not parties. Pages 5 to 8 and 10 to 11 are schedules of services, Sites, rates and controls.`,
    structure: structure({
      readingOrder: readingSnippets(lines, 6),
      keyValues: [['Effective Date', longDate(effective)]],
    }),
    gold: gold({
      type: 'Master Services Agreement',
      acceptableTypes: ['Services Agreement'],
      date: effective,
      role: 'effective',
      forbiddenDates: [[rfp, 'request for proposal issued'], [proposal, 'contractor proposal date'], [signedContractor, 'contractor signature date'], [signedClient, 'client signature date'], [sowStart, 'first statement of work start'], [termEnd, 'end of the initial term']],
      parties: [CLIENT, CONTRACTOR],
      relation: 'between',
      roles: [[CLIENT, 'client'], [CLIENT, 'customer'], [CONTRACTOR, 'contractor'], [CONTRACTOR, 'vendor'], [CONTRACTOR, 'seller']],
      forbiddenParties: SUBCONTRACTORS.slice(0, 3).map(([name]) => [name, 'approved subcontractor listed in Schedule F']),
      facts: [['systems integration', 'systems integrator'], ['warehouse']],
      subjectTerms: ['master services agreement', 'warehouse', 'service levels'],
      readiness: 'ready',
      dateText: [longDate(effective)],
      dateAnchor: '"Effective Date" means',
      typeText: ['MASTER SERVICES AGREEMENT'],
      identifierText: [AGREEMENT_NUMBER],
    }),
  });
}
