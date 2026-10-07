/// A forty-page term loan agreement for a cold-storage company. The lender
/// and the borrower are named twice: in the preamble on page 1 and on the
/// signature page, page 40. Everything between says only "Lender" and
/// "Borrower", while the schedules name the borrower's customers, the
/// appraiser, the title insurer and the bank whose loan this one repays.
/// The agreement is dated in its preamble; the maturity date in the
/// definitions shares its day and month.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { MONTHS, addDays, addMonths, grouped, longDate, money, numericDate } from '../lib/format.mjs';
import { people } from '../lib/names.mjs';
import { digitalPdf, readingSnippets, result } from './common.mjs';
import { blocks, chain, expectPages, fillTo, tableBlocks } from './dense.mjs';

const LENDER = 'Oakhurst Mutual Savings Bank';
const BORROWER = 'Saltbrook Cold Storage Partners LP';
const GENERAL_PARTNER = 'Saltbrook Holdings GP LLC';
const PRIOR_LENDER = 'Kingsbarn Federal Savings Bank';
const APPRAISER = 'Greywether Valuation Group';
const TITLE = 'Lockhaven Title Insurance Company';
const LOAN_NUMBER = 'OMSB-CRE-26-0817';
const APPRAISAL_DATE = '2026-06-22';
const ENVIRONMENTAL_DATE = '2026-05-30';
const PRIOR_LOAN_DATE = '2019-05-03';

/// The mortgaged facilities: code, town, address, square feet, pallet
/// positions, temperature zones, appraised value in cents.
const FACILITIES = [
  ['SB-1', 'Saltbrook, Delaware', '1200 Marsh Harbor Road', 286000, 41200, 'frozen -10 F, cooler 34 F', 4810000000],
  ['SB-2', 'Milford, Delaware', '77 Cedar Neck Industrial Drive', 174500, 24600, 'frozen -10 F, blast -25 F', 2925000000],
  ['SB-3', 'Salisbury, Maryland', '4410 Northwood Logistics Way', 212800, 30100, 'frozen -5 F, cooler 36 F, ambient', 3390000000],
  ['SB-4', 'Easton, Maryland', '19 Port Street Extension', 98400, 13800, 'cooler 34 F, ripening rooms', 1468000000],
  ['SB-5', 'Seaford, Delaware', '600 Nanticoke Commerce Park', 156200, 22300, 'frozen -10 F', 2440000000],
  ['SB-6', 'Dover, Delaware', '3 Horsepond Business Center', 121700, 17400, 'frozen -10 F, cooler 34 F', 1975000000],
  ['SB-7', 'Laurel, Delaware', '25 Broad Creek Distribution Drive', 138900, 19700, 'frozen -10 F, cooler 36 F', 2150000000],
  ['SB-8', 'Cambridge, Maryland', '1800 Bayly Road Industrial Park', 109300, 15200, 'cooler 34 F, controlled atmosphere', 1720000000],
  ['SB-9', 'Harrington, Delaware', '410 Fairgrounds Logistics Lane', 192600, 27500, 'frozen -10 F, blast -25 F, cooler 34 F', 3010000000],
];

/// Each facility's county, the year it was built and the year it was last
/// expanded, its dock doors, and whether a rail siding serves it.
const SITES = {
  'SB-1': ['Sussex County', 1988, 2017, 46, true], 'SB-2': ['Kent County', 2003, 2021, 28, false], 'SB-3': ['Wicomico County', 1996, 2019, 34, true],
  'SB-4': ['Talbot County', 1979, 2008, 14, false], 'SB-5': ['Sussex County', 2006, 2023, 24, false], 'SB-6': ['Kent County', 1992, 2014, 20, true],
  'SB-7': ['Sussex County', 2011, 2024, 22, false], 'SB-8': ['Dorchester County', 1984, 2012, 16, false], 'SB-9': ['Kent County', 2015, 2025, 32, true],
};

/// What each customer stores, by its place in CUSTOMERS.
const COMMODITIES = ['fresh and frozen poultry', 'frozen seafood', 'fresh produce', 'frozen desserts', 'frozen vegetables', 'frozen dough and bakery goods', 'prepared frozen foods', 'butter and cheese',
  'ice cream', 'frozen beef and pork', 'frozen appetizers', 'frozen berries', 'frozen pet food', 'fresh fish', 'chilled prepared meals', 'mixed grocery'];

/// Recorded matters each title policy excepts, and who benefits from them.
const EXCEPTIONS = [
  ['Electric distribution easement', 'Tidemark Electric Cooperative'], ['Gas service easement', 'Bayshore Gas Transmission Company'], ['Water main easement', 'Nanticoke Valley Water Company'],
  ['Fiber optic easement', 'Peninsula Fiber Networks LLC'], ['Drainage and stormwater easement', 'the county'], ['Declaration of covenants, conditions and restrictions', 'the industrial park association'],
  ['Sidetrack agreement', 'Marshland Short Line Railroad'], ['Ingress and egress easement', 'the adjoining owner'], ['Right-of-way widening deed', 'the State Department of Transportation'],
  ['Memorandum of solar rooftop lease', 'Brightfield Rooftop Solar LLC'], ['Wetlands conservation easement', 'Saltmeadow Land Trust'], ['Billboard lease', 'Roadside Outdoor Media Inc.'],
];

/// Capital projects by kind and typical cost in dollars.
const PROJECTS = [
  ['Roof replacement over the freezer', 1800000], ['Convert refrigeration to low-charge ammonia', 4200000], ['Replace dock levelers and seals', 380000], ['Add blast cells', 2600000],
  ['Rooftop solar array', 1900000], ['LED high-bay lighting retrofit', 260000], ['Replace underfloor heating glycol loop', 950000], ['Resurface truck court', 540000],
  ['Add 4,000 pallet positions of mobile racking', 1450000], ['Replace evaporator coils', 720000], ['Upgrade fire suppression to ESFR', 880000], ['Standby generator replacement', 1100000],
];

/// Customers whose storage contracts are assigned to Lender.
const CUSTOMERS = [
  'Bayfront Poultry Processors Inc.', 'Chesapeake Tide Seafood LLC', 'Greenmarsh Produce Cooperative', 'Holloway Frozen Desserts Co.',
  'Indian River Vegetable Growers', 'Juniper Lane Bakeries Inc.', 'Kettle Cove Foods LLC', 'Lowland Dairy Distributors',
  'Marlin Point Ice Cream Company', 'Northfork Meat Packers Inc.', 'Oyster Bay Frozen Foods LLC', 'Pinewood Berry Farms',
  'Quarry Hill Pet Foods Inc.', 'Rivermouth Fish Company', 'Sandpiper Prepared Meals LLC', 'Tidewater Grocery Wholesale Inc.',
];

/// Refrigeration and handling equipment by type and maker.
const EQUIPMENT = [
  ['Screw compressor, 300 hp', 'Arden Compression'], ['Screw compressor, 450 hp', 'Arden Compression'], ['Evaporative condenser', 'Patapsco Coil Works'],
  ['Penthouse evaporator, 18 ton', 'Kestle Thermal'], ['Ceiling-hung evaporator, 9 ton', 'Kestle Thermal'], ['Ammonia receiver, 2,000 gal', 'Hollis Vessel Works'],
  ['Ammonia recirculator package', 'Hollis Vessel Works'], ['Blast freezer tunnel', 'Polar Tunnel Systems'], ['Rapid-roll freezer door', 'Rapidlane Doors'],
  ['Dock leveler with insulated seal', 'Marshdock Equipment'], ['Electric reach truck, cold-store rated', 'Ridgeline Lift Trucks'], ['Electric pallet jack', 'Ridgeline Lift Trucks'],
  ['Mobile racking carriage set', 'Storewell'], ['Standby generator, 1,250 kW', 'Peninsula Power Systems'], ['Underfloor heating glycol pump set', 'Grundell'],
  ['Refrigeration control system', 'Logix Cold Controls'],
];

const DEFINITIONS = [
  ['Account Control Agreement', 'each agreement among Borrower, Lender and a depositary bank giving Lender control of a deposit account of Borrower'],
  ['Amortization Schedule', 'the schedule of principal payments in Schedule 1'],
  ['Applicable Rate', 'a fixed rate of 6.42 percent per annum'],
  ['Appraisal', `the appraisal of the Mortgaged Properties prepared by ${APPRAISER} with a valuation date of ${longDate(APPRAISAL_DATE)}`],
  ['Assignment of Contracts', 'the assignment to Lender of the Customer Contracts listed in Schedule 4'],
  ['Business Day', 'a day on which Lender is open for business in Wilmington, Delaware'],
  ['Cash Management Period', 'any period that begins when the Debt Service Coverage Ratio for a Test Period is below 1.25 to 1.00 and ends when it has been at least 1.35 to 1.00 for two consecutive Test Periods'],
  ['Collateral', 'all property in which Borrower grants Lender a lien under the Loan Documents'],
  ['Customer Contracts', 'Borrower\'s warehousing agreements with its customers, including those listed in Schedule 4'],
  ['Debt Service', 'for any Test Period, the scheduled principal and interest payable on the Loan during it'],
  ['Debt Service Coverage Ratio', 'for any Test Period, Net Operating Income divided by Debt Service'],
  ['Default Rate', 'the Applicable Rate plus four percent per annum'],
  ['Environmental Reports', `the reports listed in Schedule 6, the Phase I assessments among them each dated ${longDate(ENVIRONMENTAL_DATE)}`],
  ['Equipment', 'the refrigeration, material handling and power equipment listed in Schedule 3 and all replacements'],
  ['Event of Default', 'any of the events listed in Section 10.1'],
  ['Existing Loan', `the loan made to Borrower by ${PRIOR_LENDER} under a loan agreement dated ${longDate(PRIOR_LOAN_DATE)}, which the Loan repays in full`],
  ['First Payment Date', `the first day of the second full calendar month after the date of this Agreement`],
  ['Loan', 'the term loan made under Section 2.1'],
  ['Loan Amount', 'one hundred fifty-two million dollars ($152,000,000)'],
  ['Loan Documents', 'this Agreement, the Note, the Mortgages, the Assignment of Contracts, the Account Control Agreements and every other document securing or relating to the Loan'],
  ['Loan-to-Value Ratio', 'the outstanding principal of the Loan divided by the appraised value of the Mortgaged Properties'],
  ['Maturity Date', `${longDate('2033-08-14')}`],
  ['Mortgaged Properties', 'the nine cold-storage facilities listed in Schedule 2'],
  ['Mortgages', 'the mortgages and security agreements on each Mortgaged Property'],
  ['Net Operating Income', 'revenue from the Mortgaged Properties less operating expenses, excluding depreciation, debt service and non-cash items, adjusted for a management fee of at least three percent'],
  ['Permitted Liens', 'liens for taxes not yet due, the matters listed in the Title Policies, and purchase-money liens on equipment securing not more than $2,500,000 in total'],
  ['Replacement Reserve', 'the reserve established under Section 5.2'],
  ['Test Period', 'each period of four consecutive fiscal quarters'],
  ['Title Policies', `the mortgagee title insurance policies issued by ${TITLE}`],
];

const ARTICLES = [
  ['The Loan', [
    'Lender agrees to make the Loan to Borrower in a single advance on the date of this Agreement, in the Loan Amount, on the terms and subject to the conditions in this Agreement. Amounts repaid may not be reborrowed.',
    'Borrower will use the proceeds of the Loan to repay the Existing Loan in full, to pay the costs of closing, to fund the Replacement Reserve and the Tax and Insurance Reserve, and, as to the balance, for general working capital of the Mortgaged Properties.',
  ]],
  ['Interest and Payments', [
    'The Loan bears interest at the Applicable Rate, computed on the basis of a 360-day year and the actual number of days elapsed. Beginning on the First Payment Date and on the first day of each month after it, Borrower will pay accrued interest and the principal installment shown in the Amortization Schedule. The entire outstanding principal, with accrued interest and all other amounts due, is payable on the Maturity Date.',
    'If any payment is not made within ten days after its due date, Borrower will pay a late charge of four percent of the amount overdue. After an Event of Default and while it continues, the Loan bears interest at the Default Rate.',
    'All payments are made without setoff or counterclaim, in immediately available funds, by 2:00 p.m. Wilmington time, to the account Lender designates. A payment due on a day that is not a Business Day is due on the next Business Day.',
  ]],
  ['Prepayment', [
    'Borrower may prepay the Loan in whole, but not in part, on any payment date on thirty days\' notice, with a prepayment premium of three percent of the amount prepaid in the first two years of the Loan, two percent in the third and fourth years, one percent in the fifth year, and none after that.',
    'If a Mortgaged Property is sold with Lender\'s consent, Borrower will prepay the release price for it, which is one hundred twenty percent of the Loan Amount allocated to it in Schedule 2, with the prepayment premium then applicable.',
  ]],
  ['Conditions of Closing', [
    'Lender\'s obligation to make the Loan is subject to its receipt of the documents and evidence listed in the Closing Checklist in Schedule 7, each in form and substance satisfactory to Lender, and to the payoff of the Existing Loan from the proceeds.',
  ]],
  ['Reserves', [
    'Borrower will deposit with Lender monthly one-twelfth of the annual taxes and insurance premiums for the Mortgaged Properties (the "Tax and Insurance Reserve"), and $0.25 per square foot per year of the Mortgaged Properties for capital replacements of roofs, refrigeration and paving (the "Replacement Reserve"). Lender will disburse reserves against invoices for the purposes for which they are held.',
    'During a Cash Management Period all revenue of the Mortgaged Properties will be deposited in an account controlled by Lender and applied first to reserves, then to Debt Service, then to approved operating expenses, and the balance held as additional collateral until the Cash Management Period ends.',
  ]],
  ['Representations and Warranties', [
    'Borrower represents and warrants that it is duly organized and in good standing; that the Loan Documents have been duly authorized and are its binding obligations; that its financial statements fairly present its condition; that it owns each Mortgaged Property in fee simple subject only to Permitted Liens; that the Mortgaged Properties comply in all material respects with zoning, building, fire and environmental laws; that the Customer Contracts listed in Schedule 4 are in full force and no customer is in default beyond any grace period; and that there is no litigation that could reasonably be expected to have a material adverse effect on it.',
    'Borrower further represents that, except as disclosed in the Environmental Reports, no hazardous material has been released at any Mortgaged Property, and that each ammonia refrigeration system is operated under a risk management plan filed with the Environmental Protection Agency and a process safety management program.',
  ]],
  ['Affirmative Covenants', [
    'Borrower will deliver quarterly financial statements within forty-five days after each quarter and audited annual statements within one hundred twenty days after each year, with a compliance certificate in the form of Exhibit A; maintain the Mortgaged Properties and the Equipment in good repair; maintain the insurance in Schedule 5; pay its taxes when due; keep its ammonia systems in compliance with the process safety management standard; and notify Lender promptly of any default, litigation, environmental release or loss of a customer accounting for more than five percent of revenue.',
  ]],
  ['Negative Covenants', [
    'Borrower will not incur indebtedness other than the Loan and purchase-money equipment financing within the Permitted Liens; grant liens other than Permitted Liens; sell or lease any Mortgaged Property except storage agreements in the ordinary course; merge or change its form; change its general partner; amend a Customer Contract in a way that reduces its term or minimum charges; or make distributions during a Cash Management Period.',
  ]],
  ['Financial Covenants', [
    'Borrower will maintain a Debt Service Coverage Ratio of at least 1.20 to 1.00 for each Test Period, and a Loan-to-Value Ratio of not more than seventy percent, tested on any new appraisal Lender orders at Borrower\'s cost not more than once a year.',
  ]],
  ['Events of Default and Remedies', [
    'Each of the following is an Event of Default: failure to pay principal or interest when due, or any other amount within ten days after notice; a breach of a financial covenant; a breach of any other covenant not cured within thirty days after notice; a representation that proves materially false; a default under other indebtedness over $1,000,000; a judgment over $1,000,000 not paid or stayed within sixty days; bankruptcy or insolvency; a change of control of Borrower or its general partner; or the loss of a Mortgaged Property\'s certificate of occupancy for more than ninety days.',
    'On an Event of Default Lender may declare the Loan immediately due, foreclose the Mortgages, collect the Customer Contracts directly, apply the reserves, and exercise any other remedy under the Loan Documents or law, all of which are cumulative.',
  ]],
  ['Miscellaneous', [
    'This Agreement is governed by the laws of the State of Delaware. Borrower will pay Lender\'s reasonable costs of closing, administration and enforcement. Lender may sell participations in the Loan and assign it to an affiliate or, after an Event of Default, to any person. Notices are given to the addresses on the signature page. EACH PARTY WAIVES TRIAL BY JURY IN ANY ACTION RELATING TO THE LOAN DOCUMENTS.',
  ]],
];

/// What the closing checklist asks for, and who delivers it.
const CHECKLIST = [
  ['Executed Term Loan Agreement and Note', 'Lender\'s counsel'], ['Mortgage for each Mortgaged Property, recorded', 'Title company'],
  ['UCC-1 financing statements filed with the Delaware Secretary of State', 'Lender\'s counsel'], ['Assignment of Contracts with customer acknowledgments', 'Borrower'],
  ['Account Control Agreements for the operating and reserve accounts', 'Borrower'], ['Title Policies with extended coverage endorsements', 'Title company'],
  ['ALTA surveys of each Mortgaged Property', 'Surveyor'], ['Zoning reports for each Mortgaged Property', 'Zoning consultant'],
  ['Property condition reports', 'Engineer'], ['Environmental Reports with reliance letters', 'Environmental consultant'],
  ['Appraisal with reliance letter', 'Appraiser'], ['Evidence of insurance meeting Schedule 5, with lender loss payee endorsements', 'Insurance broker'],
  ['Flood zone determinations', 'Lender'], ['Certificate of limited partnership and partnership agreement', 'Borrower'],
  ['Certificate of formation and operating agreement of the general partner', 'Borrower'], ['Good standing certificates', 'Borrower\'s counsel'],
  ['Incumbency and authorizing resolutions', 'Borrower'], ['Opinion of Borrower\'s counsel', 'Borrower\'s counsel'],
  ['Payoff letter for the Existing Loan', 'Borrower'], ['Release of the Existing Loan mortgages and UCC filings', 'Title company'],
  ['Risk management plans for the ammonia systems', 'Borrower'], ['Process safety management audits, last three years', 'Borrower'],
  ['Rent roll and customer contract abstracts', 'Borrower'], ['Trailing twelve-month operating statements by facility', 'Borrower'],
  ['Capital expenditure plan for three years', 'Borrower'], ['Know-your-customer documentation for each owner over 10%', 'Borrower'],
  ['Funds flow memorandum', 'Lender'], ['Settlement statement', 'Title company'],
];

export function termLoan40() {
  const id = 'term-loan-parties-apart-40p';
  const rng = Rng.from(id);
  const dated = '2026-08-14';
  const maturity = '2033-08-14';
  const firstPayment = '2026-10-01';
  const appraisal = APPRAISAL_DATE;
  const environmental = ENVIRONMENTAL_DATE;
  const priorLoan = PRIOR_LOAN_DATE;
  const flow = new Flow({
    face: 'serif', fontSize: 10.5, leading: 1.35, margins: { top: 64, bottom: 66 }, keep: [LENDER, BORROWER, GENERAL_PARTNER, PRIOR_LENDER, APPRAISER, TITLE, ...CUSTOMERS],
    header: (page, { number }) => {
      if (number > 1) page.text(72, 44, `Term Loan Agreement - Loan No. ${LOAN_NUMBER}`, { face: 'sans', size: 7.5, grey: 0.4 });
    },
    footer: (page, { number }) => page.textCenter(306, 760, `- ${number} -`, { face: 'sans', size: 7.5, grey: 0.4 }),
  });

  flow.heading('TERM LOAN AGREEMENT', { level: 1, align: 'center', size: 16 });
  flow.paragraph(`Loan No. ${LOAN_NUMBER}`, { face: 'sans', size: 9.5, align: 'center', after: 12 });
  flow.paragraph(`This TERM LOAN AGREEMENT is dated as of ${longDate(dated)} and is made between ${LENDER}, a Delaware savings bank ("Lender"), and ${BORROWER}, a Delaware limited partnership ("Borrower").`);
  flow.paragraph('Borrower owns and operates the nine public refrigerated warehouses described in Schedule 2 and has asked Lender to make a term loan secured by them, by their equipment and by Borrower\'s customer contracts, to refinance the Existing Loan and for working capital. Lender is willing to do so on the terms of this Agreement. The parties therefore agree as follows:');
  flow.paragraph('ARTICLE 1 - DEFINITIONS', { face: 'sans-bold', size: 10, before: 4, after: 3 });
  flow.paragraph('1.1 In this Agreement the following terms have the meanings given below; other capitalized terms are defined where they first appear.', { after: 4 });
  for (const [term, meaning] of DEFINITIONS) flow.paragraph([{ text: `"${term}" `, face: 'sans-bold' }, { text: `means ${meaning}.` }], { indent: 12, size: 10, after: 3 });
  ARTICLES.forEach(([title, paragraphs], index) => {
    const number = index + 2;
    flow.paragraph(`ARTICLE ${number} - ${title.toUpperCase()}`, { face: 'sans-bold', size: 10, before: 4, after: 3, keepWithNext: 30 });
    paragraphs.forEach((text, item) => flow.paragraph(`${number}.${item + 1} ${text}`));
  });

  const pick = rng.fork('schedules');
  // Schedule 1: the amortization.
  const amortization = [];
  let balance = 15200000000;
  const monthlyRate = 0.0642 / 12;
  for (let month = 0; month < 84; month += 1) {
    const date = addMonths(firstPayment, month);
    const interest = Math.round(balance * monthlyRate);
    const principal = Math.round(balance / (300 - month) + month * 1200);
    balance -= principal;
    amortization.push([String(month + 1), numericDate(date), money(principal), money(interest), money(principal + interest), money(balance)]);
  }
  amortization[83][2] = `${amortization[83][2]} plus balloon`;
  const schedule1 = tableBlocks(flow, [{ header: 'No.', width: 0.08, align: 'right' }, { header: 'Payment date', width: 0.16 }, { header: 'Principal', width: 0.2, align: 'right' }, { header: 'Interest', width: 0.17, align: 'right' }, { header: 'Total', width: 0.19, align: 'right' }, { header: 'Balance after', width: 0.2, align: 'right' }], amortization,
    { title: 'SCHEDULE 1 - AMORTIZATION SCHEDULE', intro: 'Scheduled payments of principal and interest at the Applicable Rate, assuming no prepayment. Interest shown is for a 30-day month; actual interest is computed on the actual days elapsed. The unpaid balance is due on the Maturity Date.', chunk: 6 });
  // Schedule 2: the properties, then each one's legal description.
  // The Loan Amount spread over the facilities by appraised value, the
  // rounding left with the last.
  const appraised = FACILITIES.reduce((sum, facility) => sum + facility[6], 0);
  const allocated = FACILITIES.map(([, , , , , , value]) => Math.round(15200000000 * value / appraised / 10000000) * 10000000);
  allocated[allocated.length - 1] += 15200000000 - allocated.reduce((sum, value) => sum + value, 0);
  const schedule2 = tableBlocks(flow, [{ header: 'Facility', width: 0.09 }, { header: 'Location', width: 0.33 }, { header: 'Sq ft', width: 0.1, align: 'right' }, { header: 'Pallets', width: 0.1, align: 'right' }, { header: 'Zones', width: 0.18 }, { header: 'Allocated Loan Amount', width: 0.2, align: 'right' }],
    FACILITIES.map(([code, town, street, area, pallets, zones], index) => [code, `${street}, ${town}`, grouped(area), grouped(pallets), zones, money(allocated[index])]),
    { title: 'SCHEDULE 2 - MORTGAGED PROPERTIES', intro: 'Each facility, its storage capacity, and the part of the Loan Amount allocated to it for releases under Section 4.2. Appraised values are those in the Appraisal.', chunk: 6 });
  const legal = chain(...FACILITIES.map(([code, town]) => tableBlocks(flow,
    [{ header: 'Course', width: 0.12, align: 'right' }, { header: 'Bearing', width: 0.3 }, { header: 'Distance (ft)', width: 0.2, align: 'right' }, { header: 'To', width: 0.38 }],
    Array.from({ length: pick.int(14, 22) }, (_, index) => [String(index + 1), `${pick.pick(['N', 'S'])} ${pick.int(0, 89)}°${String(pick.int(0, 59)).padStart(2, '0')}'${String(pick.int(0, 59)).padStart(2, '0')}" ${pick.pick(['E', 'W'])}`, (pick.int(3000, 120000) / 100).toFixed(2), pick.pick(['iron pipe found', 'capped rebar set', 'concrete monument', 'right-of-way line', 'centerline of ditch', 'fence corner post'])]),
    { intro: `Legal description of ${code} (${town}): beginning at the point of beginning shown on the ALTA survey and running by the courses below, containing ${(pick.int(800, 3200) / 100).toFixed(2)} acres.`, chunk: 6 })));
  // Schedule 3: the equipment, facility by facility.
  const equipmentRows = FACILITIES.flatMap(([code]) => EQUIPMENT.filter(() => pick.chance(0.75)).map(([kind, maker]) => [code, kind, maker, `${maker.slice(0, 2).toUpperCase()}${pick.int(100000, 999999)}`, String(pick.int(2004, 2025)), money(pick.int(12, 900) * 100000)]));
  const schedule3 = tableBlocks(flow, [{ header: 'Facility', width: 0.09 }, { header: 'Equipment', width: 0.33 }, { header: 'Maker', width: 0.17 }, { header: 'Serial', width: 0.14 }, { header: 'Year', width: 0.09, align: 'right' }, { header: 'Book value', width: 0.18, align: 'right' }], equipmentRows,
    { title: 'SCHEDULE 3 - EQUIPMENT', intro: 'Equipment subject to Lender\'s security interest. Replacements become Collateral when installed; Borrower will update this Schedule with each annual compliance certificate.', chunk: 6 });
  // Schedule 4: the customer contracts.
  // Each customer stores at one or two facilities.
  const contracts = CUSTOMERS.flatMap((customer, index) => {
    const at = FACILITIES.filter(() => pick.chance(0.2)).slice(0, 2);
    if (!at.length) at.push(FACILITIES[pick.int(0, FACILITIES.length - 1)]);
    return at.map(([code, , , , , zones]) => ({ customer, code, commodity: COMMODITIES[index], zone: zones.split(', ')[0], minimum: pick.int(400, 9000), years: pick.int(1, 7), charges: pick.int(80, 2400) * 100000, renewals: pick.pick(['none', 'one-year renewals', 'two three-year renewals']) }));
  });
  const contractRows = contracts.map((contract) => [contract.customer, contract.code, `${grouped(contract.minimum)} pallets`, `${contract.years} ${contract.years > 1 ? 'years' : 'year'}`, money(contract.charges), contract.renewals]);
  const schedule4 = tableBlocks(flow, [{ header: 'Customer', width: 0.3 }, { header: 'Facility', width: 0.09 }, { header: 'Minimum', width: 0.14 }, { header: 'Remaining term', width: 0.13 }, { header: 'Annual minimum charges', width: 0.18, align: 'right' }, { header: 'Renewals', width: 0.16 }], contractRows,
    { title: 'SCHEDULE 4 - CUSTOMER CONTRACTS', intro: 'The warehousing agreements assigned to Lender. None of the customers is a party to this Agreement. Minimums are reserved pallet positions.', chunk: 5 });
  const insurance = blocks(INSURANCE, (text, index) => {
    if (index === 0) flow.heading('SCHEDULE 5 - INSURANCE REQUIREMENTS', { level: 2 });
    flow.paragraph(`5.${index + 1} ${text}`, { size: 9.5 });
  });
  const reports = tableBlocks(flow, [{ header: 'Facility', width: 0.1 }, { header: 'Report', width: 0.42 }, { header: 'Consultant', width: 0.28 }, { header: 'Findings', width: 0.2 }],
    FACILITIES.flatMap(([code]) => [['Phase I environmental site assessment', 'Marlowe Environmental Services LLC'], ['Property condition report', 'Brightline Building Diagnostics'], ['Ammonia system mechanical integrity inspection', 'Coldline Process Safety LLC']].map(([report, firm]) => [code, report, firm, pick.pick(['no recognized environmental conditions', 'historical fuel tank; no further action', 'deferred maintenance of $120,000', 'two relief valves overdue for replacement', 'no material findings'])])),
    { title: 'SCHEDULE 6 - ENVIRONMENTAL AND PROPERTY REPORTS', intro: 'Reports delivered before closing, on which Lender may rely.', chunk: 6 });
  const checklist = tableBlocks(flow, [{ header: 'No.', width: 0.08, align: 'right' }, { header: 'Item', width: 0.62 }, { header: 'Responsible', width: 0.3 }], CHECKLIST.map(([item, who], index) => [String(index + 1), item, who]),
    { title: 'SCHEDULE 7 - CLOSING CHECKLIST', intro: 'The items Lender requires before making the Loan under Article 5.', chunk: 6 });
  const certificate = blocks(CERTIFICATE, (text, index) => {
    if (index === 0) flow.heading('EXHIBIT A - FORM OF COMPLIANCE CERTIFICATE', { level: 2 });
    flow.paragraph(text, { size: 9.5 });
  });
  const covenantRows = FACILITIES.flatMap(([code]) => ['Revenue', 'Power cost', 'Labor', 'Repairs and maintenance', 'Real estate taxes', 'Insurance', 'Management fee', 'Net Operating Income'].map((line) => [code, line, money(pick.int(20, 4200) * 100000), money(pick.int(20, 4200) * 100000)]));
  const worksheet = tableBlocks(flow, [{ header: 'Facility', width: 0.12 }, { header: 'Line', width: 0.4 }, { header: 'Test Period', width: 0.24, align: 'right' }, { header: 'Prior Test Period', width: 0.24, align: 'right' }], covenantRows,
    { title: 'Annex to Exhibit A - Facility Operating Statement', intro: 'To be completed for each Test Period from Borrower\'s books; the illustrative figures below are from the trailing twelve months delivered at closing.', chunk: 8 });
  // The due diligence appendices, most of them by facility; the customer
  // activity, the longest, comes last.
  const month = (iso) => { const [y, m] = iso.split('-'); return `${MONTHS[Number(m) - 1].slice(0, 3)} ${y}`; };
  const history = chain(...FACILITIES.map(([code, town, street, area, pallets, zones], index) => {
    const [county, built, expanded, doors, rail] = SITES[code];
    const base = pick.int(68, 86);
    const months = Array.from({ length: 24 }, (_, at) => {
      const occupancy = Math.min(98, Math.max(52, Math.round(base + 9 * Math.sin(((at + pick.int(0, 2)) / 12) * 2 * Math.PI) + pick.int(-4, 4))));
      const occupied = Math.round(pallets * occupancy / 100);
      const storage = occupied * pick.int(2150, 2650);
      const handling = Math.round(occupied * pick.int(35, 70) / 100) * pick.int(1100, 1500);
      const megawatts = Math.round(area * pick.int(26, 44) / 10000);
      const power = megawatts * pick.int(9800, 13800);
      return [month(addMonths('2024-07-01', at)), grouped(occupied), `${occupancy}%`, money(storage), money(handling), grouped(megawatts), money(power), money(Math.round((storage + handling) * pick.int(28, 41) / 100) - power)];
    });
    const peak = months.reduce((best, row) => (Number(row[2].slice(0, -1)) > Number(best[2].slice(0, -1)) ? row : best));
    return tableBlocks(flow, [{ header: 'Month', width: 0.11 }, { header: 'Pallets held', width: 0.12, align: 'right' }, { header: 'Occupancy', width: 0.1, align: 'right' }, { header: 'Storage revenue', width: 0.15, align: 'right' }, { header: 'Handling revenue', width: 0.14, align: 'right' }, { header: 'Power (MWh)', width: 0.1, align: 'right' }, { header: 'Power cost', width: 0.13, align: 'right' }, { header: 'NOI', width: 0.15, align: 'right' }], months,
      { title: index === 0 ? 'APPENDIX 1 - FACILITY OPERATING HISTORY' : null, intro: `${index === 0 ? 'Monthly results of each Mortgaged Property for the twenty-four months before closing, from Borrower\'s books, unaudited. ' : ''}${code}, ${street}, ${town} (${county}): built ${built}${expanded > built ? ` and expanded ${expanded}` : ''}; ${grouped(area)} square feet and ${grouped(pallets)} pallet positions (${zones}); ${doors} dock doors${rail ? ' and a rail siding' : ''}. Occupancy peaked at ${peak[2]} in ${peak[0]}.`, chunk: 6, size: 8 });
  }));
  const titleRows = (code) => Array.from({ length: pick.int(5, 9) }, () => EXCEPTIONS[pick.int(0, EXCEPTIONS.length - 1)]).map(([instrument, holder], index) => {
    const recorded = addDays('1958-01-01', pick.int(0, 24000));
    return [String(index + 1), instrument, holder === 'the county' ? SITES[code][0] : holder, numericDate(recorded), pick.chance(0.5) ? `Book ${pick.int(180, 4800)}, Page ${pick.int(1, 640)}` : `Instr. ${recorded.slice(0, 4)}${String(pick.int(1, 99999)).padStart(6, '0')}`];
  });
  const title = chain(...FACILITIES.map(([code, town], index) => tableBlocks(flow, [{ header: 'No.', width: 0.07, align: 'right' }, { header: 'Recorded matter', width: 0.37 }, { header: 'In favor of', width: 0.3 }, { header: 'Recorded', width: 0.11 }, { header: 'Reference', width: 0.15 }], titleRows(code),
    { title: index === 0 ? 'APPENDIX 2 - TITLE EXCEPTIONS' : null, intro: `${index === 0 ? 'The matters each Title Policy excepts from coverage, all of them Permitted Liens. ' : ''}${code} (${town}), Title Policy No. LT-${pick.int(100000, 999999)}:`, chunk: 5, size: 8 })));
  const parcels = FACILITIES.flatMap(([code, , , , , , value]) => Array.from({ length: pick.int(1, 3) }, (_, index, all) => {
    const land = Math.round(value * pick.int(8, 18) / 100 / 1000) * 1000;
    const improvements = Math.round(value * pick.int(40, 70) / 100 / 1000) * 1000;
    const tax = Math.round((land + improvements) * pick.int(55, 140) / 100000);
    return [code, `${pick.int(1, 6)}-${pick.int(10, 99)}-${pick.int(100, 999)}.${String(pick.int(0, 99)).padStart(2, '0')}`, SITES[code][0], money(land), money(improvements), money(tax), money(Math.round(tax * pick.int(102, 109) / 100))];
  }));
  const taxes = tableBlocks(flow, [{ header: 'Facility', width: 0.09 }, { header: 'Parcel', width: 0.15 }, { header: 'County', width: 0.17 }, { header: 'Assessed land', width: 0.14, align: 'right' }, { header: 'Assessed improvements', width: 0.17, align: 'right' }, { header: '2025 tax', width: 0.13, align: 'right' }, { header: '2026 estimate', width: 0.15, align: 'right' }], parcels,
    { title: 'APPENDIX 3 - REAL ESTATE TAX PARCELS', intro: 'The tax parcels of each Mortgaged Property, from which the Tax and Insurance Reserve is sized.', chunk: 6, size: 8 });
  const systems = FACILITIES.flatMap(([code, , , area]) => Array.from({ length: area > 150000 ? 2 : 1 }, (_, index) => [`${code}-R${index + 1}`, `${grouped(Math.round(area * pick.int(30, 90) / 10000) * 10)} lb`, String(pick.int(2, 7)), String(pick.int(8, 46)), pick.pick(['Program 3', 'Program 3', 'Program 2']), month(addMonths('2023-01-01', pick.int(0, 30))), month(addMonths('2024-01-01', pick.int(0, 20))), pick.pick(['none', 'none', 'two relief valves', 'labeling of piping', 'update of the emergency response plan', 'eyewash station retest'])]));
  const ammonia = tableBlocks(flow, [{ header: 'System', width: 0.1 }, { header: 'Ammonia charge', width: 0.13, align: 'right' }, { header: 'Compressors', width: 0.11, align: 'right' }, { header: 'Evaporators', width: 0.11, align: 'right' }, { header: 'RMP', width: 0.1 }, { header: 'Last PSM audit', width: 0.12 }, { header: 'Last integrity inspection', width: 0.14 }, { header: 'Open findings', width: 0.19 }], systems,
    { title: 'APPENDIX 4 - AMMONIA REFRIGERATION SYSTEMS', intro: 'Each system subject to the process safety management standard and to a risk management plan, with its most recent audit and inspection.', chunk: 6, size: 8 });
  const capital = FACILITIES.flatMap(([code]) => [2027, 2028, 2029].flatMap((year) => PROJECTS.filter(() => pick.chance(0.18)).map(([project, cost]) => [code, String(year), project, money(Math.round(cost * pick.int(80, 125) / 100 / 1000) * 100000), pick.pick(['Replacement Reserve', 'operating cash', 'equipment financing', 'Replacement Reserve and operating cash'])])));
  const capex = tableBlocks(flow, [{ header: 'Facility', width: 0.09 }, { header: 'Year', width: 0.08 }, { header: 'Project', width: 0.45 }, { header: 'Budget', width: 0.14, align: 'right' }, { header: 'Funded from', width: 0.24 }], capital,
    { title: 'APPENDIX 5 - CAPITAL EXPENDITURE PLAN', intro: 'The three-year plan delivered under the Closing Checklist. Projects over $1,000,000 need Lender\'s approval of their budget before they start.', chunk: 6, size: 8 });
  const activity = chain(...contracts.map((contract, index) => {
    const storageRate = pick.int(1650, 2850);
    const handlingRate = pick.int(950, 1650);
    let held = Math.round(contract.minimum * pick.int(80, 120) / 100);
    const rows = Array.from({ length: 12 }, (_, at) => {
      const received = Math.round(held * pick.int(18, 55) / 100);
      const shipped = Math.max(0, Math.round(received * pick.int(80, 118) / 100));
      held = Math.max(Math.round(contract.minimum * 0.5), held + received - shipped);
      const billed = Math.max(held, contract.minimum);
      return [month(addMonths('2025-07-01', at)), grouped(received), grouped(shipped), grouped(held), money(billed * storageRate), money((received + shipped) * handlingRate), money(pick.int(0, 60) * 2500)];
    });
    return tableBlocks(flow, [{ header: 'Month', width: 0.12 }, { header: 'Pallets in', width: 0.12, align: 'right' }, { header: 'Pallets out', width: 0.12, align: 'right' }, { header: 'Held at month end', width: 0.16, align: 'right' }, { header: 'Storage billed', width: 0.17, align: 'right' }, { header: 'Handling billed', width: 0.16, align: 'right' }, { header: 'Accessorials', width: 0.15, align: 'right' }], rows,
      { title: index === 0 ? 'APPENDIX 6 - CUSTOMER ACTIVITY' : null, intro: `${index === 0 ? 'Pallet movements and billings under each Customer Contract for the twelve months before closing. ' : ''}${contract.customer} at ${contract.code}: ${contract.commodity}, ${contract.zone}; ${grouped(contract.minimum)} pallet positions reserved at ${money(storageRate)} a month each, handling ${money(handlingRate)} a pallet in or out.`, chunk: 6, size: 8 });
  }));
  fillTo(flow, 40, chain(schedule1, schedule2, legal, schedule3, schedule4, insurance, reports, checklist, certificate, worksheet, history, title, taxes, ammonia, capex, activity), { id, room: 160 });

  flow.paragraph('IN WITNESS WHEREOF, the parties have signed this Term Loan Agreement as of the date first written above.', { before: 6, after: 18 });
  const [lenderSigner, partnerSigner] = people(rng.fork('signers'), 2);
  const top = flow.y;
  [[LENDER, '', lenderSigner, 'Senior Vice President, Commercial Real Estate'], [BORROWER, `By: ${GENERAL_PARTNER}, its general partner`, partnerSigner, 'Managing Member']].forEach(([party, through, person, title], column) => {
    const x = 72 + column * 240;
    [[party, 'sans-bold', 9], [through, 'serif', 9], [`By: /s/ ${person}`, 'serif', 10], [`Name: ${person}`, 'serif', 10], [`Title: ${title}`, 'serif', 10]]
      .forEach(([text, face, size], line) => { if (text) flow.page.text(x, top + 12 + line * 16, text, { face, size }); });
  });
  flow.y = top + 100;
  flow.paragraph('Address for notices to Lender: Commercial Real Estate Lending, 1 Rodney Square, Wilmington, Delaware 19801. Address for notices to Borrower: 1200 Marsh Harbor Road, Saltbrook, Delaware 19960, with a copy to its counsel.', { size: 9.5 });

  const pages = flow.finish();
  if (pages.length !== 40) throw new Error(`${id}: expected 40 pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  expectPages(id, text, longDate(dated), [1]);
  expectPages(id, text, LENDER, [1, 40]);
  expectPages(id, text, BORROWER, [1, 40]);
  const lines = text.flatMap((page) => page.split('\n'));
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Forty-page term loan agreement naming its parties only on the first and last pages',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_25', 'contract', 'financial', 'irrelevant_names', 'competing_dates', 'information_dense', 'table'],
    notes: `The lender (${LENDER}) and the borrower (${BORROWER}) are named only in the preamble on page 1 and on the signature page, page 40, thirty-nine pages apart; every page between says "Lender" and "Borrower". The agreement is dated in the preamble (${longDate(dated)}); the definitions give the Maturity Date (${longDate(maturity)}), the same day and month seven years on. The schedules carry the amortization from the first payment (${numericDate(firstPayment)}), and name the bank whose loan this one repays, the appraiser, the title insurer and sixteen customers, none of them parties. The definitions also date the appraisal (${longDate(appraisal)}), the environmental reports (${longDate(environmental)}) and the loan agreement being repaid (${longDate(priorLoan)}).`,
    structure: structure({
      readingOrder: readingSnippets(lines, 8),
      keyValues: [['Maturity Date', longDate(maturity)]],
    }),
    recording: 'pending',
    gold: gold({
      type: 'Term Loan Agreement',
      acceptableTypes: ['Loan Agreement'],
      date: dated,
      role: 'effective',
      forbiddenDates: [[maturity, 'maturity date'], [firstPayment, 'first payment date'], [appraisal, 'appraisal valuation date'], [environmental, 'environmental report date'], [priorLoan, 'date of the loan being repaid']],
      parties: [LENDER, BORROWER],
      relation: 'between',
      roles: [[LENDER, 'lender'], [BORROWER, 'borrower']],
      forbiddenParties: [[PRIOR_LENDER, 'lender of the loan being repaid'], [APPRAISER, 'appraiser'], [TITLE, 'title insurer'], [CUSTOMERS[0], 'customer whose contract is assigned'], [GENERAL_PARTNER, 'borrower\'s general partner, signing for it']],
      facts: [['$152,000,000', 'one hundred fifty-two million dollars'], ['cold storage', 'cold-storage', 'refrigerated warehouses']],
      subjectTerms: ['term loan', 'cold storage', 'amortization'],
      readiness: 'ready',
      dateText: [longDate(dated)],
      typeText: ['TERM LOAN AGREEMENT'],
      identifierText: [LOAN_NUMBER],
    }),
  });
}

const INSURANCE = [
  'Property insurance on each Mortgaged Property and the Equipment on a special form basis for full replacement cost, with an agreed amount endorsement, a deductible of not more than $100,000 per occurrence, and Lender named as mortgagee and lender loss payee.',
  'Spoilage and refrigeration breakdown coverage for goods in Borrower\'s care, custody or control of at least $25,000,000 per occurrence, and equipment breakdown coverage for the ammonia systems with ammonia contamination coverage of at least $1,000,000.',
  'Business income insurance covering at least eighteen months of Net Operating Income, with an extended period of indemnity of one hundred eighty days.',
  'Warehouse legal liability of at least $10,000,000 per location, commercial general liability of $2,000,000 per occurrence with an umbrella of $25,000,000, and pollution legal liability of $10,000,000 covering ammonia releases.',
  'Flood insurance for any Mortgaged Property in a special flood hazard area, for the maximum available under the National Flood Insurance Program and excess flood coverage to replacement cost.',
  'Insurers must be rated A- VIII or better by A.M. Best. Policies must provide thirty days\' notice to Lender of cancellation or material change, and Borrower will deliver renewal certificates before each expiry.',
];

const CERTIFICATE = [
  'To Lender: This certificate is delivered under Section 8.1 of the Term Loan Agreement. Capitalized terms have the meanings given in the Agreement. The undersigned officer of Borrower\'s general partner certifies, on behalf of Borrower and not personally, that:',
  '1. The attached financial statements fairly present Borrower\'s financial condition and results of operations for the period they cover, in accordance with generally accepted accounting principles consistently applied, subject in the case of quarterly statements to year-end adjustments.',
  '2. The Debt Service Coverage Ratio for the Test Period ended on the last day of the period covered by this certificate was ____ to 1.00, as calculated in the annexed worksheet, and the minimum required is 1.20 to 1.00.',
  '3. No Event of Default has occurred and is continuing, and no event has occurred that with notice or lapse of time would become one, except as described in an attachment to this certificate with the steps Borrower is taking about it.',
  '4. Schedule 3 and Schedule 4, as updated in the attachments, list all Equipment and Customer Contracts as of the last day of the period, and no customer accounting for more than five percent of revenue has given notice of termination or non-renewal.',
  '5. Each ammonia refrigeration system was operated during the period under its risk management plan, and no reportable release occurred except as described in an attachment.',
];
