/// A twenty-five-page industrial lease whose own date appears once: in the
/// Lease Particulars of Schedule 1, on page 23, beside the commencement,
/// rent commencement and expiration dates. Page 1 says only that the lease
/// is "dated as of the date stated in Schedule 1", and names the letter of
/// intent and the lease of the tenant's old building it replaces, each
/// with its own date. Between them are twenty pages of articles, a legal
/// description, dock and building schedules, a rent schedule, an
/// environmental baseline and a work letter.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { addMonths, longDate, money } from '../lib/format.mjs';
import { people, phone } from '../lib/names.mjs';
import { digitalPdf, readingSnippets, result } from './common.mjs';
import { blocks, chain, expectPages, fillTo, tableBlocks } from './dense.mjs';

const LANDLORD = 'Ardent Quay Industrial Properties LP';
const TENANT = 'Moss & Lanyard Distribution Inc.';
const GUARANTOR = 'Moss & Lanyard Holdings LLC';
const BROKER = 'Pelham Crane Realty Advisors';
const CONSULTANT = 'Cinderford Environmental Group';
const BUILDING = 'Building 7, Ardent Quay Logistics Park';

const ARTICLES = [
  ['Premises', [
    `Landlord leases to Tenant the building known as ${BUILDING}, 2200 Wharfside Parkway, Port Ellery, Maryland 21226, containing approximately 312,480 square feet of floor area, together with the truck court, trailer parking and car parking areas shown on Exhibit B, on the land described in Exhibit A (together, the "Premises"). Tenant may use in common with other tenants of the Park the roads, rail spur, stormwater facilities and landscaped areas (the "Common Areas").`,
    'Tenant has inspected the Premises and accepts them in their condition on the Commencement Date, subject to completion of the Landlord Work in Exhibit G and to Landlord\'s warranty that the roof, the structure and the building systems listed in Exhibit E will be in good working order on that date. Tenant will give Landlord a punch list within thirty days after the Commencement Date, and Landlord will complete the listed items within sixty days.',
  ]],
  ['Term and Options', [
    'The Term begins on the Commencement Date and ends on the Expiration Date stated in Schedule 1, unless sooner terminated. Tenant has two options to extend the Term for five years each, exercisable by notice given not more than eighteen nor less than twelve months before the then-current Expiration Date, at ninety-five percent of the fair market rent determined under Section 3.5.',
    'If Landlord does not deliver the Premises with the Landlord Work substantially complete by the Commencement Date, the Commencement Date and the Rent Commencement Date will each be postponed day for day; if delivery has not occurred within one hundred twenty days, Tenant may terminate this Lease by notice before delivery and recover its deposit.',
  ]],
  ['Rent', [
    'Tenant will pay Base Rent in the monthly amounts in Exhibit C, in advance on the first day of each month from the Rent Commencement Date, by electronic funds transfer, without deduction or setoff except as this Lease expressly allows. Base Rent increases by three percent on each anniversary of the Rent Commencement Date.',
    'In addition Tenant will pay its share of Operating Expenses, Taxes and Insurance Costs for the Park, which on the Commencement Date is 18.6 percent, adjusted if the floor area of the Park changes. Landlord will estimate these amounts before each calendar year and reconcile them within one hundred twenty days after its end; Tenant may audit Landlord\'s records once a year at its own cost, and Landlord will pay for the audit if it shows an overcharge of more than four percent.',
    'Controllable Operating Expenses, which exclude Taxes, insurance premiums, utilities and snow and ice removal, will not increase by more than five percent a year over the prior year\'s amount on a non-cumulative basis.',
  ]],
  ['Security Deposit and Guaranty', [
    `Tenant will deliver a security deposit of ${money(31248000)} in cash or an irrevocable standby letter of credit from a bank with an office in Baltimore. Landlord may draw on it to cure a default, and Tenant will restore it within ten days. If Tenant is not in default during the first five Lease Years, the deposit will be reduced by half. ${GUARANTOR}, Tenant's parent, will guarantee Tenant's obligations in the form attached as Exhibit H.`,
  ]],
  ['Use', [
    'Tenant may use the Premises for the warehousing and distribution of packaged consumer goods, light assembly and kitting, packaging, returns processing and ancillary offices, and for no other use without Landlord\'s consent. Tenant will not store goods classified as high-piled combustible storage above the heights its fire protection permits, and will not store hazardous materials except those used in ordinary warehouse operations in quantities below the reporting thresholds of the county fire code.',
    'Tenant may operate the Premises twenty-four hours a day, seven days a week, and may park trailers in the trailer stalls shown on Exhibit B but not in the truck court lanes or on the Park roads.',
  ]],
  ['Environmental Matters', [
    `Tenant will comply with all environmental laws applicable to its use of the Premises and will not release any hazardous material on or from them. Exhibit D summarizes the baseline environmental assessment that ${CONSULTANT} performed for Landlord before the Commencement Date. Tenant is responsible for contamination caused by Tenant or its contractors after the Commencement Date and is not responsible for any condition shown in Exhibit D or caused by others.`,
    'At the end of the Term Tenant will remove its hazardous materials and storage equipment and, if Landlord reasonably requests on evidence of a release during the Term, will provide a Phase I environmental assessment of the Premises at its cost.',
  ]],
  ['Loading, Rail and Truck Court', [
    'Tenant has the exclusive use of the dock doors, levelers and seals listed in Exhibit B, the truck court in front of them, and the trailer stalls assigned to the Premises. Tenant may use the rail spur on the schedule the Park\'s rail coordinator publishes, paying its share of the spur\'s maintenance in proportion to the railcars it receives.',
    'Tenant will keep the truck court free of debris and stored materials, will not perform vehicle maintenance on it other than emergency repairs, and will require its carriers to observe the Park\'s speed limits, idling restrictions and traffic plan.',
  ]],
  ['Maintenance and Repairs', [
    'Landlord will maintain, repair and replace the roof membrane and structure, the foundation, the exterior walls, the slab (except damage caused by Tenant\'s equipment), the fire sprinkler mains and the Common Areas, the cost of which is an Operating Expense to the extent this Lease allows. Tenant will maintain the interior of the Premises, the dock equipment, the heating and ventilation units, the lighting and the plumbing fixtures, under service contracts with contractors reasonably acceptable to Landlord.',
    'Tenant will repair floor damage from forklifts and racking within thirty days, and will patch anchor holes and remove floor striping it installs at the end of the Term.',
  ]],
  ['Alterations and Racking', [
    'Tenant may make non-structural alterations costing less than $150,000 each without consent on notice to Landlord. Other alterations need Landlord\'s consent, not unreasonably withheld. Tenant may install storage racking, conveyors and mezzanines under permits it obtains, designed for the slab\'s rated load of 6,000 pounds per rack post, and will remove them at the end of the Term unless Landlord agrees otherwise.',
  ]],
  ['Signs', [
    'Tenant may place its name on the building\'s monument sign panel and on the building fascia above the office entrance, in sizes and materials Landlord approves and the county permits. Tenant will remove its signs at the end of the Term and repair any damage.',
  ]],
  ['Utilities', [
    'Tenant will contract directly for and pay for electricity, natural gas, telephone and data services to the Premises. Water and sewer are submetered and billed by Landlord at cost. Landlord is not liable for an interruption of utilities unless caused by its negligence, but if an interruption within Landlord\'s control continues for more than three business days and Tenant cannot reasonably operate, Base Rent abates from the fourth day until service is restored.',
  ]],
  ['Insurance', [
    'Landlord will insure the building for its full replacement cost against special form causes of loss, with rent loss coverage for twelve months. Tenant will maintain commercial general liability insurance of $5,000,000 per occurrence, property insurance on its inventory and equipment for their replacement cost, business interruption insurance, automobile liability of $2,000,000 and workers\' compensation. Each party waives claims against the other for losses covered by property insurance it is required to carry.',
  ]],
  ['Indemnity', [
    'Tenant will indemnify Landlord against claims for injury or damage occurring in the Premises, except to the extent caused by Landlord\'s negligence or willful misconduct. Landlord will indemnify Tenant against claims for injury or damage occurring in the Common Areas, except to the extent caused by Tenant\'s negligence or willful misconduct.',
  ]],
  ['Casualty and Condemnation', [
    'If the building is damaged by fire or other casualty, Landlord will restore it within two hundred forty days unless the damage occurs in the last eighteen months of the Term, in which case either party may terminate. Rent abates in proportion to the area Tenant cannot use. If more than twenty percent of the building or a third of the truck court is taken by condemnation, either party may terminate this Lease.',
  ]],
  ['Assignment and Subletting', [
    'Tenant may not assign this Lease or sublet any part of the Premises without Landlord\'s consent, which Landlord will not unreasonably withhold, condition or delay. Consent is not required for an assignment to an affiliate or to a successor by merger with a tangible net worth at least equal to Tenant\'s on the date of this Lease. Landlord will receive half of any sublease profit after Tenant recovers its costs.',
  ]],
  ['Default and Remedies', [
    'It is an event of default if Tenant fails to pay rent within five business days after notice, fails to perform any other obligation within thirty days after notice (or longer if the failure cannot be cured in thirty days and Tenant diligently pursues the cure), or becomes insolvent. On an event of default Landlord may terminate this Lease or Tenant\'s right of possession and recover its damages, which Landlord will mitigate.',
    'If Landlord fails to perform an obligation within thirty days after notice, Tenant may perform it and recover the reasonable cost from Landlord, with interest at the prime rate plus two percent.',
  ]],
  ['Subordination and Estoppel', [
    'This Lease is subordinate to any mortgage on the Park only if its holder agrees not to disturb Tenant\'s possession while Tenant is not in default. Each party will deliver an estoppel certificate within fifteen days after the other\'s request, not more than three times a year.',
  ]],
  ['Surrender and Holdover', [
    'At the end of the Term Tenant will surrender the Premises broom clean and in good repair, ordinary wear and casualty excepted, with its racking, conveyors and signs removed. If Tenant holds over without consent it will pay one hundred fifty percent of the last month\'s Base Rent for each month or part of a month, and after sixty days will be liable for Landlord\'s damages from a lost replacement tenant.',
  ]],
  ['Brokers and Notices', [
    `Each party represents that it dealt with no broker other than ${BROKER}, whose commission Landlord will pay under a separate agreement. Notices must be in writing and sent to the addresses in Schedule 1 by hand, nationally recognized overnight courier or certified mail, and are effective on receipt or refusal.`,
  ]],
  ['Miscellaneous', [
    'This Lease is governed by the laws of the State of Maryland. Time is of the essence. If any provision is unenforceable, the rest remains in effect. This Lease, its Schedules and Exhibits are the entire agreement of the parties about the Premises and supersede the letter of intent and all earlier proposals. Neither party may record this Lease, but either may record a memorandum of it in the form Landlord provides.',
  ]],
];

/// Dock doors: type, size, leveler, seal, restraint.
const DOOR_TYPES = [['dock-high', "9' x 10'"], ['dock-high', "9' x 10'"], ['dock-high', "9' x 10'"], ['dock-high', "10' x 12'"], ['drive-in', "12' x 14'"]];
const LEVELERS = ['35,000 lb hydraulic leveler', '40,000 lb hydraulic leveler', '30,000 lb mechanical leveler', 'vertical-storing hydraulic leveler'];
const SEALS = ['foam pad seal', 'inflatable shelter', 'curtain shelter', 'foam head and side pads'];
const RESTRAINTS = ['rotating hook restraint', 'wheel-based restraint', 'no restraint (drive-in)'];

/// Building systems: tag prefix, description, maker.
const SYSTEMS = [
  ['RTU', 'Rooftop unit serving the office area, 15 tons', 'Hartwell Climate'], ['UH', 'Gas-fired unit heater, 250 MBH', 'Corbin Thermal'],
  ['AHU', 'Make-up air unit, high bay, 12,000 cfm', 'Hartwell Climate'], ['EF', 'Roof exhaust fan, 24 in.', 'Brennan Air Moving'],
  ['FP', 'Fire pump, 1,500 gpm diesel', 'Kessler Fire Systems'], ['RISER', 'ESFR sprinkler riser', 'Kessler Fire Systems'],
  ['SWG', 'Main switchgear, 3,000 A, 480 V', 'Danbury Electric'], ['XFMR', 'Dry-type transformer, 225 kVA', 'Danbury Electric'],
  ['LP', 'Lighting panel', 'Danbury Electric'], ['LED', 'High-bay LED fixtures with occupancy sensors (bank)', 'Lumen Ridge'],
  ['WH', 'Electric water heater, 80 gal', 'Ostrander Plumbing Supply'], ['BFP', 'Backflow preventer, 6 in.', 'Ostrander Plumbing Supply'],
  ['GEN', 'Life-safety generator, 150 kW natural gas', 'Danbury Electric'], ['FA', 'Fire alarm control panel', 'Kessler Fire Systems'],
  ['ROOF', 'TPO roof membrane, 60 mil, mechanically fastened (section)', 'Calloway Roofing'], ['DOOR', 'Dock door operator', 'Wainscott Door'],
];

/// Environmental baseline: sample location descriptions.
const SAMPLE_PLACES = [
  'truck court, north of dock door', 'former rail siding, east end', 'stormwater inlet', 'transformer pad', 'fire pump room floor drain',
  'trailer parking row', 'landscaped swale, south property line', 'fuel island footprint of the previous user', 'office parking lot',
  'slab penetration near column line', 'oil-water separator outlet', 'perimeter monitoring well', 'rail spur ballast',
];

/// The Landlord Work and its budget.
const WORK = [
  'Restripe truck court and trailer stalls', 'Install 18 additional dock levelers and seals', 'Add ESFR sprinkler heads at new mezzanine',
  'Upgrade high-bay lighting to 30 foot-candles', 'Build out 6,200 sq ft office and break room', 'Add two ADA restrooms at the shipping office',
  'Replace damaged slab panels at column lines 12-14', 'Install battery charging room ventilation', 'Seal and coat the exterior tilt-up panels',
  'Add 120 car parking spaces on the west lot', 'Install perimeter security fencing and two guard booths', 'Add 2,000 A electrical service for automation',
  'Install trench drains at the drive-in door', 'Replace roof drains and overflow scuppers', 'Add data cabling pathways to the mezzanine',
  'Install an emergency eyewash station in the battery room', 'Paint the warehouse interior walls to 12 feet', 'Commission the life-safety generator',
];

/// A metes-and-bounds course as a table row: number, bearing, distance,
/// and the monument it runs to.
function course(rng, index) {
  const ns = rng.pick(['N', 'S']);
  const ew = rng.pick(['E', 'W']);
  const degrees = rng.int(0, 89);
  const minutes = String(rng.int(0, 59)).padStart(2, '0');
  const seconds = String(rng.int(0, 59)).padStart(2, '0');
  const feet = (rng.int(4000, 98000) / 100).toFixed(2);
  const to = rng.pick(['iron pipe found', 'capped rebar set', 'concrete monument found', 'center of drainage swale', 'nail set in pavement', 'right-of-way line, Wharfside Parkway', 'rail spur centerline']);
  return [String(index + 1), `${ns} ${degrees}°${minutes}'${seconds}" ${ew}`, feet, to];
}

export function industrialLease25() {
  const id = 'industrial-lease-dated-in-schedule-25p';
  const rng = Rng.from(id);
  const dated = '2026-04-14';
  const loi = '2026-01-09';
  const oldLease = '2016-05-01';
  const commencement = '2026-07-01';
  const rentCommencement = '2026-10-01';
  const expiration = '2036-06-30';
  const tenantSigned = '2026-04-16';
  const landlordSigned = '2026-04-20';
  const flow = new Flow({
    face: 'serif', fontSize: 10.5, leading: 1.35, margins: { top: 64, bottom: 66 }, keep: [LANDLORD, TENANT, GUARANTOR, BROKER, CONSULTANT],
    header: (page, { number }) => {
      if (number > 1) page.text(72, 44, 'Industrial Lease - Building 7, Ardent Quay Logistics Park', { face: 'sans', size: 7.5, grey: 0.4 });
    },
    footer: (page, { number, total }) => page.textRight(540, 760, `${number} / ${total}`, { face: 'sans', size: 7.5, grey: 0.4 }),
  });

  flow.heading('INDUSTRIAL LEASE', { level: 1, align: 'center', size: 16 });
  flow.paragraph(BUILDING, { face: 'sans', size: 10, align: 'center', after: 12 });
  flow.paragraph(`THIS INDUSTRIAL LEASE is dated as of the date stated in Schedule 1 and is made between ${LANDLORD}, a Delaware limited partnership ("Landlord"), and ${TENANT}, a Maryland corporation ("Tenant"). The Lease Particulars in Schedule 1 are part of this Lease, and capitalized terms used without definition have the meanings given there.`);
  flow.paragraph(`This Lease follows the letter of intent between Landlord and Tenant dated ${longDate(loi)}, which it supersedes. On the Commencement Date it also replaces Tenant's lease of Building 4 in the Park dated ${longDate(oldLease)}, which the parties agree will end when Tenant has moved its operations into the Premises, and in any event ninety days after the Commencement Date.`);
  ARTICLES.forEach(([title, paragraphs], index) => {
    flow.paragraph(`ARTICLE ${index + 1} - ${title.toUpperCase()}`, { face: 'sans-bold', size: 10, before: 4, after: 3, keepWithNext: 30 });
    paragraphs.forEach((text, item) => flow.paragraph(`${index + 1}.${item + 1} ${text}`));
  });

  const pick = rng.fork('exhibits');
  const courseColumns = [{ header: 'Course', width: 0.12, align: 'right' }, { header: 'Bearing', width: 0.3 }, { header: 'Distance (ft)', width: 0.2, align: 'right' }, { header: 'To', width: 0.38 }];
  const parcel = (title, intro, start, count) => tableBlocks(flow, courseColumns, Array.from({ length: count }, (_, index) => course(pick, start + index)), { title, intro, chunk: 6, size: 8.5 });
  const legal = chain(
    parcel('EXHIBIT A - LEGAL DESCRIPTION', 'Parcel 1 (fee): all that tract of land in the Fifteenth Election District of the county, being Lot 7 of the Ardent Quay Logistics Park subdivision plat, beginning at a concrete monument found at the southeast corner of the intersection of Wharfside Parkway and Lanyard Court and running by the courses below back to the place of beginning, containing 21.884 acres.', 0, 64),
    parcel(null, 'Parcel 2 (non-exclusive access easement over the Park roads appurtenant to Parcel 1): beginning at a point on the east right-of-way line of Lanyard Court distant 412.18 feet from the beginning point of Parcel 1, and running by the courses below.', 64, 30),
    parcel(null, 'Parcel 3 (utility and stormwater easement along the rail spur): beginning at a capped rebar set on the north line of Parcel 1 at the centerline of track 3, and running by the courses below.', 94, 26),
  );
  const doors = blocks(Array.from({ length: 48 }, (_, index) => index), (index) => {
    if (index === 0) {
      flow.heading('EXHIBIT B - DOCKS, TRUCK COURT AND PARKING', { level: 2 });
      flow.paragraph('The building has 48 loading positions on the east elevation, a 185-foot-deep truck court, 92 trailer stalls and 310 car parking spaces. Each position is listed with its equipment; positions marked for Landlord Work receive new equipment before the Commencement Date.', { size: 9.5 });
    }
    const [type, size] = DOOR_TYPES[pick.int(0, DOOR_TYPES.length - 1)];
    const leveler = type === 'drive-in' ? 'ramp to grade, no leveler' : LEVELERS[pick.int(0, LEVELERS.length - 1)];
    const seal = SEALS[pick.int(0, SEALS.length - 1)];
    const restraint = type === 'drive-in' ? RESTRAINTS[2] : RESTRAINTS[pick.int(0, 1)];
    const work = pick.chance(0.35) ? '; Landlord Work' : '';
    const light = pick.chance(0.5) ? 'LED dock light' : 'swing-arm dock light';
    flow.paragraph(`Door E-${String(index + 1).padStart(2, '0')} at column line ${String.fromCharCode(65 + (index % 12))}-${Math.floor(index / 12) + 4}: ${type}, ${size}, ${leveler}, ${seal}, ${restraint}, ${light}${work}.`, { size: 9, after: 1 });
  });
  // Exhibit C: monthly for the first two years, then by Lease Year.
  // $14.00 a square foot a year on 312,480 square feet, in cents a month.
  const monthly = 36456000;
  const rent = blocks([0, 1, 2], (part) => {
    if (part === 0) {
      flow.heading('EXHIBIT C - BASE RENT', { level: 2 });
      flow.paragraph('Base Rent is $14.00 per square foot of floor area per year for the first Lease Year, increasing by three percent on each anniversary of the Rent Commencement Date. The first two Lease Years are shown month by month, with the free rent period before the Rent Commencement Date.', { size: 9.5 });
      const rows = Array.from({ length: 27 }, (_, index) => {
        const month = addMonths(commencement, index);
        const free = month < rentCommencement;
        const year = index < 3 ? 0 : Math.floor((index - 3) / 12);
        const amount = free ? 0 : Math.round(monthly * 1.03 ** year);
        return [String(index + 1), month.slice(0, 7), free ? 'Abated' : money(amount), free ? '-' : money(Math.round((amount * 12) / 312480))];
      });
      flow.table([{ header: 'Month', width: 0.14 }, { header: 'Period', width: 0.24 }, { header: 'Monthly Base Rent', width: 0.32, align: 'right' }, { header: 'Annual rate per sq ft', width: 0.3, align: 'right' }], rows, { size: 8.5 });
    } else if (part === 1) {
      const rows = Array.from({ length: 10 }, (_, index) => {
        const amount = Math.round(monthly * 1.03 ** index);
        return [`Lease Year ${index + 1}`, money(amount), money(amount * 12), `$${(14 * 1.03 ** index).toFixed(2)}`];
      });
      flow.paragraph('By Lease Year:', { face: 'sans-bold', size: 9.5, after: 3 });
      flow.table([{ header: 'Period', width: 0.25 }, { header: 'Monthly', width: 0.25, align: 'right' }, { header: 'Annual', width: 0.25, align: 'right' }, { header: 'Per sq ft', width: 0.25, align: 'right' }], rows, { size: 8.5 });
    } else {
      flow.paragraph('Base Rent during each extension term is ninety-five percent of the fair market rent for comparable bulk distribution buildings in the Port Ellery submarket, determined by agreement or, failing agreement within thirty days, by three appraisers each with at least ten years of industrial leasing experience in the Baltimore region.', { size: 9.5 });
    }
  });
  const detect = (limit) => (pick.chance(0.4) ? (limit * pick.int(2, 70) / 100).toFixed(limit < 10 ? 2 : 0) : 'ND');
  const borings = tableBlocks(flow, [{ header: 'Boring', width: 0.09 }, { header: 'Location', width: 0.27 }, { header: 'Depth (ft)', width: 0.08, align: 'right' }, { header: 'Fill / soil', width: 0.2 }, { header: 'PID', width: 0.07, align: 'right' }, { header: 'TPH-DRO', width: 0.09, align: 'right' }, { header: 'Pb', width: 0.07, align: 'right' }, { header: 'As', width: 0.07, align: 'right' }, { header: 'PCB', width: 0.06, align: 'right' }],
    Array.from({ length: 40 }, (_, index) => [`SB-${String(index + 1).padStart(2, '0')}`, SAMPLE_PLACES[index % SAMPLE_PLACES.length], String(pick.int(1, 12)), pick.pick(['silty clay fill', 'gravel fill', 'fill with brick', 'native silty sand', 'organic clay']), (pick.int(0, 48) / 10).toFixed(1), detect(230), detect(400), detect(3.0), detect(1.0)]),
    { title: 'EXHIBIT D - ENVIRONMENTAL BASELINE', intro: `Summary of the baseline assessment prepared by ${CONSULTANT}. Soil results in mg/kg against the state's non-residential standards (TPH-DRO 230, lead 400, arsenic 3.0, PCBs 1.0); groundwater in ug/L against 5 for each solvent; PID in ppm. "ND" means not detected above the reporting limit. The full report, including boring logs and laboratory reports, is incorporated by reference.`, chunk: 5 });
  const wells = tableBlocks(flow, [{ header: 'Well', width: 0.12 }, { header: 'Depth to water (ft)', width: 0.2, align: 'right' }, { header: 'Screen (ft)', width: 0.2 }, { header: 'PCE', width: 0.12, align: 'right' }, { header: 'TCE', width: 0.12, align: 'right' }, { header: 'Dissolved O2 (mg/L)', width: 0.24, align: 'right' }],
    Array.from({ length: 12 }, (_, index) => [`MW-${index + 1}`, (pick.int(60, 190) / 10).toFixed(1), `${pick.int(8, 14)}-${pick.int(18, 26)}`, detect(5), detect(5), (pick.int(5, 62) / 10).toFixed(1)]), { chunk: 6 });
  const baseline = chain(borings, wells);
  const systems = blocks(Array.from({ length: 64 }, (_, index) => index), (index) => {
    if (index === 0) {
      flow.heading('EXHIBIT E - BUILDING SYSTEMS', { level: 2 });
      flow.paragraph('Systems Landlord warrants to be in good working order on the Commencement Date. "L" means Landlord maintains and replaces; "T" means Tenant maintains under a service contract and Landlord replaces at the end of its useful life.', { size: 9.5 });
    }
    const [prefix, description, maker] = SYSTEMS[index % SYSTEMS.length];
    const installed = pick.int(2008, 2025);
    const condition = installed > 2019 ? 'good' : pick.pick(['fair', 'good', 'fair, scheduled for replacement in Lease Year 3']);
    const owner = ['ROOF', 'RISER', 'FP', 'SWG', 'XFMR', 'GEN', 'FA'].includes(prefix) ? 'L' : 'T';
    flow.paragraph(`${prefix}-${String(Math.floor(index / SYSTEMS.length) + 1)}${String(index % 9 + 1)}: ${description}, ${maker}, installed ${installed}, ${condition}; responsibility ${owner}.`, { size: 9, after: 1 });
  });
  const work = blocks(WORK, (item, index) => {
    if (index === 0) {
      flow.heading('EXHIBIT G - WORK LETTER', { level: 2 });
      flow.paragraph('Landlord will perform the Landlord Work below at its cost using building-standard materials, under plans Tenant approves, and will deliver it substantially complete by the Commencement Date. Tenant may request changes at its cost; Landlord\'s tenant improvement allowance for Tenant\'s own work is $8.50 per square foot.', { size: 9.5 });
    }
    const budget = money(pick.int(18, 640) * 250000);
    const weeks = pick.int(2, 14);
    flow.paragraph(`G-${index + 1} ${item}: budget ${budget}; ${weeks} weeks; ${pick.pick(['Landlord\'s general contractor', 'specialty subcontractor', 'Landlord\'s general contractor with Tenant\'s automation vendor'])}.`, { size: 9, after: 1 });
  });
  const flatness = blocks(Array.from({ length: 54 }, (_, index) => index), (index) => {
    if (index === 0) {
      flow.heading('EXHIBIT E-1 - SLAB FLATNESS AND LOAD TESTS', { level: 2 });
      flow.paragraph('Floor flatness (FF) and levelness (FL) of each pour strip measured under ASTM E1155 within seventy-two hours of placement; the specification requires FF 50 and FL 35 overall and FF 35 and FL 25 locally. Joint fill and spall repairs listed are part of the Landlord Work.', { size: 9.5 });
    }
    const line = String.fromCharCode(65 + (index % 13));
    const strip = `${line}-${String.fromCharCode(66 + (index % 13))}/${Math.floor(index / 13) * 4 + 2}-${Math.floor(index / 13) * 4 + 6}`;
    const ff = (pick.int(420, 690) / 10).toFixed(1);
    const fl = (pick.int(280, 520) / 10).toFixed(1);
    const strength = pick.int(4100, 5600);
    const repair = Number(ff) < 50 ? 'grind high spots and retest' : pick.pick(['no action', 'refill joints with semi-rigid epoxy', 'repair two spalls at the construction joint', 'no action']);
    flow.paragraph(`Pour strip ${index + 1} (column lines ${strip}): FF ${ff}, FL ${fl}, 28-day cylinder strength ${strength} psi; ${repair}.`, { size: 9, after: 1 });
  });
  const zones = blocks(ZONES, ([zone, commodity, height, design], index) => {
    if (index === 0) {
      flow.heading('EXHIBIT E-2 - FIRE PROTECTION DESIGN BASIS', { level: 2 });
      flow.paragraph('The building is protected by ESFR sprinklers with K-25.2 heads at 52 psi, supplied by the diesel fire pump. Tenant will keep its storage within the commodity class and height for each zone, and will not install in-rack sprinklers or solid shelving without Landlord\'s approval of revised hydraulic calculations.', { size: 9.5 });
    }
    const aisle = pick.int(8, 13);
    flow.paragraph(`${zone}: ${commodity}, stored to ${height} ft under a 40 ft clear height; ${design}; minimum aisle ${aisle} ft; flue spaces of 6 in. maintained between back-to-back racks.`, { size: 9, after: 2 });
  });
  const roof = tableBlocks(flow, [{ header: 'Section', width: 0.11 }, { header: 'Bay', width: 0.11 }, { header: 'Drains', width: 0.1, align: 'right' }, { header: 'Wet insulation (sq ft)', width: 0.16, align: 'right' }, { header: 'Other findings', width: 0.42 }, { header: 'RTUs', width: 0.1, align: 'right' }],
    Array.from({ length: 24 }, (_, index) => [`R-${String(index + 1).padStart(2, '0')}`, pick.pick(['north', 'south', 'east', 'west', 'central']), String(pick.int(2, 9)), String(pick.int(0, 220)), pick.pick(['seams sound', 'two open laps at the ridge to heat-weld', 'ponding near a drain; add tapered insulation', 'pitch pans to refill', 'skylight curb flashing loose', 'walkway pads worn']), String(pick.int(1, 7))]),
    { title: 'EXHIBIT E-3 - ROOF CONDITION SURVEY', intro: 'Results of the infrared moisture survey and visual inspection of the roof, by section of approximately 13,000 square feet; wet insulation is replaced as Landlord Work. The manufacturer\'s warranty runs for twenty years from substantial completion of the roof and is assignable to a purchaser of the building.' });
  const budget = blocks(BUDGET, ([item, base], index) => {
    if (index === 0) {
      flow.heading('EXHIBIT J - PARK OPERATING BUDGET', { level: 2 });
      flow.paragraph('Landlord\'s budget of Operating Expenses, Taxes and Insurance Costs for the Park for the first calendar year of the Term, from which Tenant\'s monthly estimate is set. Amounts are for the whole Park; Tenant\'s share is 18.6 percent.', { size: 9.5 });
    }
    const total = base * pick.int(90, 112) * 10;
    flow.paragraph(`${item}: ${money(total)} for the Park, ${money(Math.round(total * 0.186))} Tenant's share, ${(total / 1680000 / 100).toFixed(3)} dollars per square foot.`, { size: 9, after: 1 });
  });
  const directory = blocks(PARK, ([building, occupant, area, use], index) => {
    if (index === 0) {
      flow.heading('EXHIBIT I - PARK DIRECTORY', { level: 2 });
      flow.paragraph('The buildings of the Park, their occupants on the date the letter of intent was signed, and their uses, for the allocation of Common Area costs and rail spur time.', { size: 9.5 });
    }
    flow.paragraph(`${building}: ${occupant}; ${area} sq ft; ${use}.`, { size: 9, after: 1 });
  });
  const guaranty = blocks(GUARANTY, (paragraph, index) => {
    if (index === 0) flow.heading('EXHIBIT H - FORM OF GUARANTY', { level: 2 });
    flow.paragraph(paragraph, { size: 9.5 });
  });
  const rules = blocks(RULES, (rule, index) => {
    if (index === 0) {
      flow.heading('EXHIBIT F - PARK RULES', { level: 2 });
    }
    flow.paragraph(`F-${index + 1} ${rule}`, { size: 9, after: 2 });
  });
  // The racking Tenant may install, bay by bay: as much as fills page 22.
  const rackRows = [];
  for (let bay = 0; bay < 26; bay += 1) {
    for (let run = 0; run < 6; run += 1) {
      const kind = pick.pick(['selective, single-deep', 'selective, double-deep', 'pallet flow', 'push-back, 4 deep', 'carton flow with pick modules']);
      const levels = pick.int(4, 7);
      rackRows.push([`${String.fromCharCode(65 + (bay % 13))}${Math.floor(bay / 13) + 1}-${run + 1}`, kind, String(levels), `${pick.int(18, 42) * 100} lb`, `${pick.int(96, 144)} in.`, pick.pick(['yes', 'no', 'yes, with column guards'])]);
    }
  }
  const racking = tableBlocks(flow, [{ header: 'Run', width: 0.1 }, { header: 'Rack type', width: 0.34 }, { header: 'Beam levels', width: 0.13, align: 'right' }, { header: 'Beam load', width: 0.14, align: 'right' }, { header: 'Beam span', width: 0.13, align: 'right' }, { header: 'Anchored', width: 0.16 }], rackRows,
    { title: 'EXHIBIT L - APPROVED RACKING LAYOUT', intro: 'The rack runs Landlord approves under Article 9, by bay and run. Beam loads are per pair of beams; every run is anchored to the slab with two anchors per post unless marked otherwise, and the anchors are removed and the holes patched at the end of the Term.' });
  fillTo(flow, 23, chain(legal, doors, rent, baseline, systems, flatness, zones, roof, work, rules, guaranty, budget, directory, racking), { id });

  // Schedule 1, the only place the lease is dated.
  flow.heading('SCHEDULE 1 - LEASE PARTICULARS', { level: 2, before: 4 });
  flow.table([{ header: 'Item', width: 0.36 }, { header: 'Particulars', width: 0.64 }], [
    ['Date of this Lease', longDate(dated)],
    ['Landlord', LANDLORD],
    ['Tenant', TENANT],
    ['Guarantor', GUARANTOR],
    ['Premises', `${BUILDING}, 2200 Wharfside Parkway, Port Ellery, Maryland 21226; approximately 312,480 square feet`],
    ['Commencement Date', longDate(commencement)],
    ['Rent Commencement Date', longDate(rentCommencement)],
    ['Expiration Date', longDate(expiration)],
    ['Initial Base Rent', '$14.00 per square foot per year, $364,560.00 per month'],
    ['Tenant\'s Share', '18.6 percent'],
    ['Security Deposit', money(31248000)],
    ['Permitted Use', 'Warehousing and distribution of packaged consumer goods, kitting, packaging and returns'],
    ['Extension Options', 'Two options of five years each'],
    ['Landlord\'s address for notices', 'c/o Ardent Quay Asset Management, 400 Harborview Drive, Suite 1100, Baltimore, Maryland 21230'],
    ['Tenant\'s address for notices', 'Attention: General Counsel, 1850 Sparrows Point Boulevard, Dundalk, Maryland 21222'],
  ], { size: 9 });
  const notices = blocks(SCHEDULE_NOTES, (note, index) => {
    if (index === 0) flow.paragraph('Notes to the Particulars:', { face: 'sans-bold', size: 9.5, after: 3 });
    flow.paragraph(`${index + 1}. ${note}`, { size: 9, indent: 12, after: 2 });
  });
  const contractorRows = SERVICE_TRADES.flatMap(([trade, firms]) => firms.map((firm) => [trade, firm, phone(pick), pick.pick(['24-hour emergency', 'business hours', 'business hours; emergency by arrangement']), `${pick.int(1, 6)} hours`]));
  const contractors = tableBlocks(flow, [{ header: 'Trade', width: 0.2 }, { header: 'Approved contractor', width: 0.32 }, { header: 'Telephone', width: 0.17 }, { header: 'Availability', width: 0.2 }, { header: 'Response', width: 0.11, align: 'right' }], contractorRows,
    { title: 'SCHEDULE 2 - APPROVED SERVICE CONTRACTORS', intro: 'Contractors Landlord approves for the work Tenant performs under Articles 8 and 9. Tenant may use others with Landlord\'s consent, not unreasonably withheld. None of them is a party to this Lease.' });
  const moveRows = MOVE_IN.map(([week, task, owner]) => [week, task, owner, pick.pick(['Park manager', 'Tenant project manager', 'Landlord construction manager', 'Tenant facilities lead'])]);
  const moveIn = tableBlocks(flow, [{ header: 'When', width: 0.16 }, { header: 'Task', width: 0.46 }, { header: 'Responsible', width: 0.19 }, { header: 'Coordinator', width: 0.19 }], moveRows,
    { title: 'SCHEDULE 3 - MOVE-IN COORDINATION', intro: 'Weeks are counted from the Commencement Date (week 0). The plan coordinates Tenant\'s move from Building 4 with the completion of the Landlord Work.' });
  fillTo(flow, 25, chain(notices, contractors, moveIn), { id, room: 90 });

  flow.paragraph('IN WITNESS WHEREOF, Landlord and Tenant have signed this Lease, which takes effect on the date stated in Schedule 1 whatever the dates of signature below.', { before: 4, after: 16 });
  const [landlordSigner, tenantSigner, notaryOne, notaryTwo] = people(rng.fork('signers'), 4);
  const top = flow.y;
  [[LANDLORD, `By: Ardent Quay GP LLC, its general partner`, landlordSigner, 'Vice President', landlordSigned], [TENANT, '', tenantSigner, 'Chief Operating Officer', tenantSigned]].forEach(([party, through, person, title, date], column) => {
    const x = 72 + column * 240;
    [[party, 'sans-bold', 9], [through, 'serif', 9], [`By: /s/ ${person}`, 'serif', 10], [`Name: ${person}`, 'serif', 10], [`Title: ${title}`, 'serif', 10], [`Signed: ${longDate(date)}`, 'serif', 10]]
      .forEach(([text, face, size], line) => { if (text) flow.page.text(x, top + 12 + line * 16, text, { face, size }); });
  });
  flow.y = top + 116;
  flow.paragraph(`State of Maryland, County of Baltimore. On ${longDate(tenantSigned)} before me, ${notaryOne}, a notary public, personally appeared ${tenantSigner}, who acknowledged signing this Lease for Tenant. My commission expires on the date on my seal.`, { size: 9 });
  flow.paragraph(`State of Maryland, City of Baltimore. On ${longDate(landlordSigned)} before me, ${notaryTwo}, a notary public, personally appeared ${landlordSigner}, who acknowledged signing this Lease for Landlord's general partner.`, { size: 9 });

  const pages = flow.finish();
  if (pages.length !== 25) throw new Error(`${id}: expected 25 pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  expectPages(id, text, longDate(dated), [23]);
  const lines = text.flatMap((page) => page.split('\n'));
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Twenty-five-page industrial lease dated only in its Lease Particulars near the end',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_25', 'contract', 'date_in_table', 'competing_dates', 'irrelevant_names', 'information_dense', 'table'],
    notes: `Page 1 says the lease is "dated as of the date stated in Schedule 1" and names two earlier documents with their dates: the letter of intent (${longDate(loi)}) and the lease of the tenant's old building (${longDate(oldLease)}). The lease's own date (${longDate(dated)}) is printed once, in the Lease Particulars table on page 23, beside the Commencement Date (${longDate(commencement)}), Rent Commencement Date (${longDate(rentCommencement)}) and Expiration Date (${longDate(expiration)}). The signatures and notary acknowledgments on page 25 carry ${longDate(tenantSigned)} and ${longDate(landlordSigned)}. The guarantor, the broker and the environmental consultant are named but are not parties.`,
    structure: structure({
      readingOrder: readingSnippets(lines, 8),
      keyValues: [['Date of this Lease', longDate(dated)], ['Commencement Date', longDate(commencement)], ['Expiration Date', longDate(expiration)]],
    }),
    recording: 'pending',
    gold: gold({
      type: 'Industrial Lease',
      acceptableTypes: ['Lease', 'Lease Agreement', 'Industrial Lease Agreement'],
      date: dated,
      role: 'effective',
      forbiddenDates: [[loi, 'letter of intent the lease supersedes'], [oldLease, 'lease of the old building it replaces'], [commencement, 'commencement date'], [rentCommencement, 'rent commencement date'], [expiration, 'expiration date'], [tenantSigned, 'tenant signature date'], [landlordSigned, 'landlord signature date']],
      parties: [LANDLORD, TENANT],
      relation: 'between',
      roles: [[LANDLORD, 'landlord'], [TENANT, 'tenant']],
      forbiddenParties: [[GUARANTOR, 'guarantor, not a party to the lease'], [BROKER, 'broker'], [CONSULTANT, 'environmental consultant']],
      facts: [['Ardent Quay Logistics Park', 'Ardent Quay'], ['312,480 square feet', '312,480']],
      subjectTerms: ['industrial lease', 'warehouse', 'dock'],
      readiness: 'ready',
      dateText: [longDate(dated)],
      dateAnchor: 'Date of this Lease',
      typeText: ['INDUSTRIAL LEASE'],
    }),
  });
}

/// Fire protection zones: zone, commodity, height, design.
const ZONES = [
  ['Zone 1, column lines A-D', 'Class I-IV commodities in cartons on wood pallets', 35, 'ESFR at 52 psi, 12 heads, 60 minutes'],
  ['Zone 2, column lines D-G', 'cartoned unexpanded Group A plastics', 30, 'ESFR at 52 psi, 12 heads, 60 minutes'],
  ['Zone 3, column lines G-J', 'packaged lithium-ion batteries in retail cartons, limited quantities', 20, 'ESFR at 60 psi, 12 heads, with segregation by 25 ft'],
  ['Zone 4, column lines J-M', 'Class I-IV commodities in cartons, double-deep racks', 35, 'ESFR at 52 psi, 12 heads'],
  ['Zone 5, mezzanine', 'kitting materials and packaging supplies', 12, 'light hazard at the mezzanine underside'],
  ['Zone 6, returns area', 'mixed commodities in gaylord boxes', 15, 'ESFR at 52 psi; gaylords no more than three high'],
  ['Zone 7, battery charging room', 'forklift batteries on charge', 0, 'ordinary hazard group 2 with ventilation interlock'],
  ['Zone 8, aerosols cage', 'Level 1 aerosols only', 10, 'chain-link enclosure with in-rack heads'],
  ['Zone 9, dock staging', 'outbound pallets for no more than 48 hours', 8, 'ESFR at 52 psi'],
  ['Zone 10, pallet storage', 'idle wood pallets', 6, 'outdoor storage only, at least 50 ft from the building'],
];

/// The operating budget: line item and base amount.
const BUDGET = [
  ['Real estate taxes', 41000], ['Property insurance', 18600], ['Liability insurance', 2900], ['Common Area landscaping', 3600],
  ['Snow and ice removal', 4800], ['Road and truck court repairs', 3100], ['Parking lot sweeping', 1200], ['Stormwater pond maintenance', 900],
  ['Rail spur inspection and maintenance', 2700], ['Site lighting electricity', 1500], ['Security patrol', 6200], ['Gatehouse staffing', 5400],
  ['Park signage', 300], ['Fire lane striping', 400], ['Common Area utilities', 1100], ['Property management fee', 7900],
  ['Roof inspections', 800], ['Backflow and fire pump testing', 1300], ['Pest control', 600], ['Trash removal from Common Areas', 700],
  ['Environmental compliance reporting', 950], ['Owners association dues', 1800],
];

/// The Park's buildings: building, occupant, area, use.
const PARK = [
  ['Building 1', 'Kestrel Bay Foods LLC', '420,000', 'temperature-controlled distribution'], ['Building 2', 'Larchmont Appliance Logistics Inc.', '265,000', 'appliance cross-dock'],
  ['Building 3', 'vacant, available for lease', '180,000', 'speculative warehouse'], ['Building 4', TENANT, '154,000', 'consumer goods distribution until the Commencement Date'],
  ['Building 5', 'Tidebrook Paper Products Co.', '238,000', 'paper and packaging warehouse'], ['Building 6', 'Halloran Freight Systems Inc.', '310,520', 'third-party logistics'],
  ['Building 7', 'the Premises', '312,480', 'consumer goods distribution'],
];

/// The form of guaranty.
const GUARANTY = [
  `For value received, and to induce Landlord to enter into the Lease, ${GUARANTOR} ("Guarantor") unconditionally guarantees to Landlord the full and punctual payment of all rent and other sums due under the Lease and the performance of all of Tenant's obligations under it.`,
  'This is a guaranty of payment and not of collection. Landlord may enforce it without first proceeding against Tenant, the Security Deposit or any other security. Guarantor waives notice of acceptance, presentment, demand, protest and notice of default, except a notice of default Landlord is required to give Tenant under the Lease.',
  "Guarantor's liability is not affected by any extension, amendment, assignment or subletting of the Lease, by Tenant's bankruptcy or the rejection of the Lease in it, or by any release of any other guarantor or security.",
  'Guarantor will deliver its audited annual financial statements to Landlord within one hundred twenty days after the end of each fiscal year, and will maintain a tangible net worth of at least $75,000,000 while this guaranty is in effect.',
  'This guaranty ends when Tenant has paid and performed all its obligations through the end of the Term and surrendered the Premises, except for obligations that survive the Lease. It is governed by the laws of the State of Maryland, and Guarantor submits to the courts of Baltimore City.',
];

/// Move-in tasks: week, task, responsible party.
const MOVE_IN = [
  ['Week -10', 'Final racking drawings submitted for the county permit', 'Tenant'], ['Week -9', 'Landlord Work punch walk of the office area', 'Landlord'],
  ['Week -8', 'Utility accounts opened in Tenant\'s name', 'Tenant'], ['Week -8', 'Fire marshal review of the high-piled storage plan', 'Tenant'],
  ['Week -7', 'Dock levelers and seals commissioned', 'Landlord'], ['Week -6', 'Racking installation begins in bays A to D', 'Tenant'],
  ['Week -6', 'Data cabling and wireless access points installed', 'Tenant'], ['Week -5', 'Security fencing and guard booths complete', 'Landlord'],
  ['Week -4', 'Sprinkler acceptance test witnessed by the fire marshal', 'Landlord'], ['Week -4', 'Conveyor installation begins at the mezzanine', 'Tenant'],
  ['Week -3', 'Battery charging room ventilation interlock tested', 'Landlord'], ['Week -3', 'Use and occupancy permit application filed', 'Tenant'],
  ['Week -2', 'Warehouse management system go-live rehearsal', 'Tenant'], ['Week -2', 'Truck court restriping and trailer stall numbering', 'Landlord'],
  ['Week -1', 'Keys, access cards and alarm codes handed over', 'Landlord'], ['Week 0', 'Possession delivered; punch list window opens', 'Landlord'],
  ['Week 1', 'First inbound trailers received from Building 4', 'Tenant'], ['Week 2', 'Outbound shipping starts from the new docks', 'Tenant'],
  ['Week 4', 'Punch list delivered to Landlord', 'Tenant'], ['Week 6', 'Building 4 vacated and surrendered', 'Tenant'],
  ['Week 8', 'Racking removal in Building 4 complete', 'Tenant'], ['Week 12', 'Punch list items complete', 'Landlord'],
];

/// Approved contractors by trade.
const SERVICE_TRADES = [
  ['Dock equipment', ['Wainscott Door Service Co.', 'Harborline Dock Systems LLC']],
  ['Fire protection', ['Kessler Fire Systems Inc.', 'Redfern Sprinkler Company']],
  ['Electrical', ['Danbury Electric Contractors Inc.', 'Pole Star Electrical LLC']],
  ['HVAC', ['Hartwell Climate Services', 'Brennan Mechanical Inc.']],
  ['Roofing', ['Calloway Roofing Co.', 'Sterling Ridge Roofing LLC']],
  ['Racking and installation', ['Storewell Rack Installers LLC', 'Tidemark Material Handling Inc.']],
  ['Plumbing', ['Ostrander Plumbing Services', 'Quayside Plumbing & Drain LLC']],
  ['Paving and striping', ['Hollins Paving Co.', 'Linemark Striping LLC']],
  ['Security systems', ['Gatewatch Security Integration LLC', 'Sentry Point Systems Inc.']],
  ['Environmental', [CONSULTANT, 'Marlowe Environmental Services LLC']],
];

/// Park rules.
const RULES = [
  'Trucks will use only the Park entrance on Wharfside Parkway and will not queue on public streets.',
  'Speed in the Park is limited to 15 miles per hour and to 5 miles per hour in truck courts.',
  'Trucks and yard tractors will not idle for more than five minutes except while loading refrigerated trailers.',
  'Trailers will be parked only in marked stalls and never on landscaped areas or fire lanes.',
  'Pallets, dunnage and waste will be kept inside the building or in enclosed containers.',
  'Smoking is permitted only in the designated shelter at the west parking lot.',
  'Tenant\'s employees will park in the car lot assigned to the Premises and not in another tenant\'s lot.',
  'Snow will be plowed to the stockpile areas Landlord designates and never against dock doors.',
  'Hazardous materials deliveries will be scheduled with the Park manager twenty-four hours in advance.',
  'Rail switching will be coordinated with the Park rail coordinator, and no railcar will be left on the main spur.',
  'Contractors working on the roof will sign in at the Park office and use only the roof access ladder at the north stair.',
  'Exterior lighting will remain on from dusk to dawn, and Tenant will report outages within one business day.',
];

/// Notes to the Lease Particulars.
const SCHEDULE_NOTES = [
  'Where Article 3 refers to a Lease Year, the first Lease Year ends on the last day of the twelfth full month after the Rent Commencement Date.',
  'The floor area was measured by Landlord\'s architect from the outside face of the exterior walls and is not subject to remeasurement.',
  'Tenant\'s Share is the floor area of the building divided by the floor area of all buildings in the Park, 1,680,000 square feet.',
  'The Security Deposit may be a letter of credit as Article 4 allows, in the form attached to the letter of intent.',
  'The Permitted Use includes storage of lithium-ion batteries packaged for retail sale, in the quantities the fire marshal approved for the building.',
  'Addresses for notices may be changed by notice given as Article 19 requires.',
  'The extension options are personal to Tenant and its affiliates and may not be exercised while an event of default continues.',
  'If a date in these Particulars falls on a day that is not a business day, the obligation it fixes falls due on the next business day.',
  'The trailer stalls assigned to the Premises are stalls 101 through 192 on the plan in Exhibit B.',
  'The rail spur allocation is two railcar spots on track 3, shared with Building 6.',
];
