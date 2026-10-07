/// A sixty-page commercial property policy for a brewing cooperative,
/// assembled as the insurer's renewal packet: the coverage forms and the
/// location schedules first, the Common Policy Declarations in the middle
/// (page 30), more schedules after. The policy period - the date the
/// policy takes effect - is stated only in the declarations table. The
/// declarations also carry the expiration and the countersignature date;
/// the schedules carry the inspection and application dates, past losses
/// and the banks with an interest in the buildings.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { addDays, grouped, money, numericDate } from '../lib/format.mjs';
import { people, phone } from '../lib/names.mjs';
import { digitalPdf, readingSnippets, result } from './common.mjs';
import { blocks, chain, expectPages, fillTo, tableBlocks } from './dense.mjs';

const INSURER = 'Gannet Rock Mutual Insurance Company';
const INSURED = 'Hollowmere Craft Brewing Cooperative';
const BROKER = 'Westbourne Risk Partners LLC';
const POLICY_NUMBER = 'GRM-CPP-4417305';

/// The insured's locations: name, street, town, what happens there.
const LOCATIONS = [
  ['Hollowmere Brewhouse', '410 Millrace Road', 'Hollowmere, Vermont', 'production brewery, packaging hall and offices'],
  ['Millrace Taproom', '412 Millrace Road', 'Hollowmere, Vermont', 'taproom and beer garden'],
  ['North Fork Cellars', '88 Quarry Lane', 'Barre, Vermont', 'barrel-aging cellar'],
  ['Kettle Hill Malthouse', '15 Kettle Hill Road', 'Randolph, Vermont', 'floor malting and grain storage'],
  ['Lakeside Canning', '2200 Shelburne Road', 'South Burlington, Vermont', 'canning line and cold storage'],
  ['Burlington Taproom', '160 Pine Street', 'Burlington, Vermont', 'taproom and small-batch pilot brewery'],
  ['Stowe Tasting Room', '1041 Mountain Road', 'Stowe, Vermont', 'tasting room and retail'],
  ['Montpelier Pub', '27 State Street', 'Montpelier, Vermont', 'brewpub with kitchen'],
  ['Rutland Distribution', '5 Quality Lane', 'Rutland, Vermont', 'distribution warehouse and fleet garage'],
  ['Brattleboro Taproom', '73 Flat Street', 'Brattleboro, Vermont', 'taproom'],
  ['Hanover Crossing Store', '12 Lebanon Street', 'Hanover, New Hampshire', 'retail store'],
  ['Keene Depot', '300 Marlboro Street', 'Keene, New Hampshire', 'distribution warehouse'],
  ['Saratoga Barrel House', '46 Excelsior Avenue', 'Saratoga Springs, New York', 'barrel house and event space'],
  ['Plattsburgh Depot', '9 Banker Road', 'Plattsburgh, New York', 'distribution warehouse'],
  ['Hop Yard Barn', '700 River Road', 'Bethel, Vermont', 'hop yard, drying kiln and barn'],
  ['Cider Annex', '414 Millrace Road', 'Hollowmere, Vermont', 'cider press and fermentation room'],
  ['Hollowmere Visitor Center', '416 Millrace Road', 'Hollowmere, Vermont', 'visitor center, gift shop and tours'],
  ['Waterbury Bottle Shop', '51 Main Street', 'Waterbury, Vermont', 'retail store'],
  ['Middlebury Taproom', '3 Maple Street', 'Middlebury, Vermont', 'taproom'],
  ['St. Albans Depot', '140 Lake Road', 'St. Albans, Vermont', 'distribution warehouse'],
  ['Lebanon Pilot Kitchen', '26 Hanover Street', 'Lebanon, New Hampshire', 'test kitchen and offices'],
  ['Quechee Gorge Stand', '5967 Woodstock Road', 'Quechee, Vermont', 'seasonal tasting stand'],
];

/// Further locations: town, state, and the kind of site.
const MORE = [
  ['Woodstock', 'Vermont', 'Taproom', 'taproom'], ['Ludlow', 'Vermont', 'Bottle Shop', 'retail store'], ['Manchester', 'Vermont', 'Tasting Room', 'tasting room and retail'],
  ['Bennington', 'Vermont', 'Depot', 'distribution warehouse'], ['Springfield', 'Vermont', 'Taproom', 'taproom with kitchen'], ['Newport', 'Vermont', 'Depot', 'distribution warehouse'],
  ['St. Johnsbury', 'Vermont', 'Taproom', 'taproom'], ['Morrisville', 'Vermont', 'Hop Store', 'hop and grain storage barn'], ['Vergennes', 'Vermont', 'Cider House', 'cider house and orchard store'],
  ['Claremont', 'New Hampshire', 'Depot', 'distribution warehouse'], ['Concord', 'New Hampshire', 'Taproom', 'taproom'], ['Portsmouth', 'New Hampshire', 'Barrel Room', 'barrel room and events'],
  ['Littleton', 'New Hampshire', 'Bottle Shop', 'retail store'], ['Glens Falls', 'New York', 'Taproom', 'taproom'], ['Lake Placid', 'New York', 'Tasting Room', 'seasonal tasting room'],
  ['Albany', 'New York', 'Depot', 'distribution warehouse and fleet garage'], ['Ticonderoga', 'New York', 'Bottle Shop', 'retail store'], ['Greenfield', 'Massachusetts', 'Depot', 'distribution warehouse'],
];
const STREET_NAMES = ['Main Street', 'Depot Street', 'Elm Street', 'Mill Street', 'River Road', 'Railroad Street', 'Union Street', 'Bridge Street', 'Park Street', 'Church Street', 'Pleasant Street', 'Water Street'];

const CONSTRUCTION = ['frame', 'joisted masonry', 'non-combustible', 'masonry non-combustible', 'modified fire-resistive'];
const ROOFS = ['standing-seam metal', 'EPDM membrane', 'TPO membrane', 'asphalt shingle', 'slate'];

/// Equipment breakdown objects at a brewery, with a typical value in
/// cents.
const OBJECTS = [
  ['Brewhouse, 30 bbl four-vessel', 180000000], ['Fermentation tank, 60 bbl', 9800000], ['Bright tank, 60 bbl', 8600000],
  ['Glycol chiller, 40 ton', 21000000], ['Steam boiler, 150 hp', 16500000], ['Air compressor, 75 hp', 4400000],
  ['Canning line, 120 cans per minute', 145000000], ['Keg washer and filler', 38000000], ['Centrifuge', 52000000],
  ['Walk-in cooler refrigeration unit', 3900000], ['Grain mill and auger system', 6700000], ['Wastewater pretreatment skid', 41000000],
  ['Kitchen hood and fire suppression', 2800000], ['Draft system and glycol trunk line', 3100000], ['Forklift charger bank', 1900000],
  ['Barrel rack system', 2400000], ['Cider press, hydraulic', 7600000], ['Malt kiln burner', 12500000], ['Standby generator, 300 kW', 19000000],
];

/// Loss control recommendations, by subject.
const RECOMMENDATIONS = [
  'Replace the remaining extension cords feeding the fermentation tank heaters with hard-wired receptacles.',
  'Test the sprinkler system\'s main drain and record the result quarterly.',
  'Install a CO2 monitor with an audible alarm in the cellar and calibrate it every six months.',
  'Clean the kitchen hood ducts by a certified contractor every three months and keep the reports.',
  'Relocate combustible packaging stored within eighteen inches of the sprinkler heads.',
  'Add a low-temperature alarm on the glycol loop that calls the on-call brewer.',
  'Provide guards on the grain auger drive and lock out the auger during cleaning.',
  'Secure the propane cylinders for the forklifts in a ventilated cage outside the building.',
  'Repair the roof drains clogged with debris above the packaging hall.',
  'Post and enforce a hot work permit program for welding on tanks and lines.',
  'Install backflow prevention on the hose bibs used for tank cleaning.',
  'Train taproom staff annually on the use of portable fire extinguishers.',
  'Keep a written inventory of barrels with their contents and fill dates, stored off site.',
  'Brace the bright tanks against seismic movement as the manufacturer recommends.',
  'Install a monitored intrusion alarm covering the retail area and the office.',
  'Inspect the boiler annually by a licensed inspector and post the certificate.',
  'Keep exit paths clear of kegs and pallets and test emergency lighting monthly.',
  'Elevate the electrical panels in the basement above the 100-year flood level.',
];

/// Claims in the five prior policy periods.
const CAUSES = ['water damage from a burst glycol line', 'roof leak during snowmelt', 'kitchen grease fire', 'theft of kegs from the loading dock', 'power surge damaging the canning line controls', 'freezing of a sprinkler branch line', 'vehicle impact on the overhead door', 'spoilage after a chiller failure', 'wind damage to the beer garden pergola', 'vandalism to the taproom windows', 'boiler tube failure', 'flood of the basement storage'];

/// Banks with an interest in the insured's buildings.
const MORTGAGEES = [
  'Wrenhollow Farm Credit, ACA', 'Lakemoor Valley Savings Bank', 'Brewstead Equipment Finance LLC', 'Fernbrook River Community Bank',
  'Pinecastle Trust and Savings', 'Ashgrove Federal Credit Union', 'Quillfeather Mutual Savings Bank', 'Whitlow River Business Lenders LLC',
];

const FORMS = [
  ['IL 00 17', 'Common Policy Conditions'], ['IL 00 21', 'Nuclear Energy Liability Exclusion Endorsement'], ['CP 00 10', 'Building and Personal Property Coverage Form'],
  ['CP 00 30', 'Business Income (and Extra Expense) Coverage Form'], ['CP 10 30', 'Causes of Loss - Special Form'], ['CP 00 90', 'Commercial Property Conditions'],
  ['CP 04 05', 'Ordinance or Law Coverage'], ['CP 04 11', 'Protective Safeguards'], ['CP 12 18', 'Loss Payable Provisions'],
  ['GR 41 02', 'Brewers Spoilage and Contamination Coverage'], ['GR 41 07', 'Tank Collapse and Leakage Coverage'], ['GR 41 12', 'Equipment Breakdown Coverage'],
  ['GR 41 20', 'Utility Services - Direct Damage and Time Element'], ['GR 41 33', 'Spoilage of Barrel-Aged Inventory - Agreed Value'],
];

const COVERAGE = [
  ['A. Coverage', 'We will pay for direct physical loss of or damage to Covered Property at the premises described in the Schedule of Locations caused by or resulting from any Covered Cause of Loss. Covered Property means the following types of property for which a Limit of Insurance is shown in the Declarations: your Building; your Business Personal Property; and Personal Property of Others in your care, custody or control.'],
  ['A.1 Building', 'Building means the buildings and structures at the premises, including completed additions; fixtures, including outdoor fixtures; permanently installed machinery and equipment, including brewing vessels, fermentation and bright tanks, and their piping, valves and controls; and personal property owned by you that is used to maintain or service the buildings, including fire-extinguishing equipment and outdoor furniture.'],
  ['A.2 Business Personal Property', 'Business Personal Property means property you own that is used in your business, located in or on the buildings or within 1,000 feet of the premises, including furniture, machinery and equipment not permanently installed, stock (raw materials such as malt, hops and yeast, goods in process including beer in fermentation and conditioning, and finished goods including packaged beer, cider and merchandise), labor, materials or services furnished on personal property of others, and your use interest as tenant in improvements and betterments.'],
  ['A.3 Property Not Covered', 'Covered Property does not include accounts, bills, currency or money; animals; automobiles held for sale; bridges, roadways and walks; contraband; land, water or growing crops, including the hops growing at the Hop Yard Barn; the cost of excavations or underground pipes; retaining walls that are not part of a building; or vehicles that are licensed for use on public roads.'],
  ['A.4 Additional Coverages', 'Subject to the limits in the Schedule of Sublimits, we will also pay for debris removal; the preservation of property moved to protect it from a covered loss; fire department service charges; pollutant clean-up and removal from land or water at the premises; increased cost of construction required by ordinance; electronic data restoration; and the cost of restoring beer, cider and wort lost from tanks, lines and kegs as a result of a covered loss.'],
  ['A.5 Coverage Extensions', 'You may extend the insurance to newly acquired or constructed property for up to 180 days; to personal effects of officers, partners, members and employees; to valuable papers and records, including brewing logs and recipes, for the cost to research and restore them; to property off premises, including kegs at customers\' premises and products at festivals; and to outdoor property such as signs, fences and the beer garden furnishings.'],
  ['B. Exclusions', 'We will not pay for loss or damage caused directly or indirectly by ordinance or law except as Coverage Form CP 04 05 provides; earth movement, unless fire or explosion results; governmental action; nuclear hazard; utility services failure originating away from the premises, except as form GR 41 20 provides; war and military action; water, meaning flood, surface water, waves, mudslide or water backing up from a sewer or drain; fungus, wet rot or dry rot except as limited coverage provides; or virus or bacteria, except bacterial contamination of product covered by form GR 41 02.'],
  ['B.1 Other Exclusions', 'We will also not pay for loss caused by wear and tear, rust, corrosion or decay; settling or cracking of foundations or tanks; mechanical breakdown, except as form GR 41 12 provides; dishonest acts of you or your employees; voluntary parting with property induced by fraud; rain, snow or ice to personal property in the open; or the neglect of an insured to use all reasonable means to save and preserve property at and after the time of loss.'],
  ['C. Limits of Insurance', 'The most we will pay for loss or damage in any one occurrence is the applicable Limit of Insurance shown in the Declarations and the Statement of Values. Blanket limits apply across all locations for which the Statement of Values shows a value, subject to the 90 percent margin clause: we will not pay more at any location than 110 percent of the value reported for it.'],
  ['D. Deductible', 'We will not pay for loss or damage in any one occurrence until the amount of loss exceeds the Deductible shown in the Declarations, after which we pay the amount of loss in excess of the Deductible up to the Limit of Insurance. Separate deductibles apply to equipment breakdown, spoilage and named storm, as the Schedule of Sublimits and Deductibles shows.'],
  ['E. Loss Conditions', 'In the event of loss or damage you must notify us or the agent promptly, give a description of how, when and where it occurred, take all reasonable steps to protect the property from further damage, give us an inventory of damaged and undamaged property with quantities, costs and values, permit us to inspect the property and records, send us a signed sworn proof of loss within sixty days of our request, and cooperate with us in the investigation and settlement of the claim.'],
  ['E.1 Valuation', 'We will determine the value of Covered Property at replacement cost without deduction for depreciation, except that finished stock is valued at its selling price less discounts and expenses you would have had; barrel-aged inventory at the agreed values in form GR 41 33; and property you have not repaired or replaced within two years after the loss at actual cash value.'],
  ['E.2 Appraisal', 'If we and you disagree on the value of the property or the amount of loss, either may make written demand for an appraisal of the loss. Each party will select a competent and impartial appraiser, and the two appraisers will select an umpire. A decision agreed to by any two will be binding. We retain our right to deny the claim.'],
  ['F. Additional Conditions', 'The coinsurance condition does not apply to property insured on a blanket basis with a Statement of Values on file. The mortgageholders shown in the Schedule of Mortgagees and Loss Payees will receive loss payments as their interests appear, and we will give them thirty days\' notice of cancellation or non-renewal.'],
  ['G. Optional Coverages', 'Agreed Value, Inflation Guard of four percent annually, and Replacement Cost including stock apply as the Declarations indicate. Extended Business Income applies for 180 days after operations resume.'],
  ['H. Spoilage and Contamination', 'Form GR 41 02 covers loss of beer, cider, wort and ingredients spoiled by a change in temperature or humidity from a breakdown of refrigeration or glycol equipment, a power outage at the premises, or contamination by a microorganism introduced accidentally into the brewing process, subject to a deductible of $5,000 and the sublimit shown, and provided the equipment is maintained under a written maintenance program.'],
  ['I. Tank Collapse and Leakage', 'Form GR 41 07 covers the sudden collapse, rupture or leakage of a brewing, fermentation, bright or storage tank, the product lost from it, and the resulting damage to other covered property, but not leakage that is gradual or continues for more than fourteen days before discovery.'],
];

export function propertyPolicy60() {
  const id = 'property-policy-declarations-mid-60p';
  const rng = Rng.from(id);
  const effective = '2026-07-01';
  const expiration = '2027-07-01';
  const prior = '2025-07-01';
  const countersigned = '2026-06-24';
  const application = '2026-05-12';
  const inspection = '2026-03-18';
  const flow = new Flow({
    face: 'serif', fontSize: 10, leading: 1.35, margins: { top: 64, bottom: 66 }, keep: [INSURER, INSURED, BROKER, ...MORTGAGEES],
    header: (page, { number }) => {
      if (number > 1) page.text(72, 44, `Policy ${POLICY_NUMBER}`, { face: 'sans', size: 7.5, grey: 0.4 });
    },
    footer: (page, { number, total }) => page.textRight(540, 760, `Page ${number} of ${total}`, { face: 'sans', size: 7.5, grey: 0.4 }),
  });
  const pick = rng.fork('schedules');

  flow.heading(INSURER.toUpperCase(), { level: 1, align: 'center', size: 14 });
  flow.paragraph('A Mutual Insurance Company - Home Office: 1 Breakwater Plaza, Portland, Maine 04101', { face: 'sans', size: 8.5, align: 'center', after: 14 });
  flow.heading('COMMERCIAL PROPERTY POLICY', { level: 1, align: 'center', size: 16 });
  flow.paragraph(`Policy No. ${POLICY_NUMBER} issued to ${INSURED} through ${BROKER}`, { face: 'sans', size: 9.5, align: 'center', after: 12 });
  flow.paragraph(`In return for the payment of the premium, and subject to all the terms of this policy, ${INSURER} agrees with the Named Insured to provide the insurance as stated in this policy. This policy consists of the coverage forms and endorsements listed below, the Schedules, and the Common Policy Declarations, which in this renewal packet follow the Schedule of Protective Safeguards. The Declarations state the policy period, the limits and the premium.`);
  flow.table([{ header: 'Form', width: 0.18 }, { header: 'Title', width: 0.64 }, { header: 'Edition', width: 0.18 }], FORMS.map(([form, title]) => [form, title, `${String(pick.int(1, 12)).padStart(2, '0')} ${pick.pick(['12', '16', '19', '22'])}`]), { size: 8.5 });
  flow.paragraph('IN WITNESS WHEREOF, we have caused this policy to be executed and attested, but it is not valid unless countersigned on the Declarations by our authorized representative.', { size: 9.5 });
  flow.pageBreak();
  flow.heading('BUILDING AND PERSONAL PROPERTY COVERAGE FORM', { level: 2 });
  flow.paragraph('Throughout this policy the words "you" and "your" refer to the Named Insured shown in the Declarations. The words "we", "us" and "our" refer to the company providing this insurance. Other words and phrases that appear in quotation marks have special meaning; refer to Section H, Definitions.', { size: 9.5 });
  for (const [title, text] of COVERAGE) flow.paragraph([{ text: `${title}. `, face: 'sans-bold' }, { text }], { size: 9.5 });

  // The schedules before the declarations.
  const sites = [...LOCATIONS, ...MORE.map(([town, state, kind, use], index) => [`${town} ${kind}`, `${pick.int(2, 480)} ${STREET_NAMES[index % STREET_NAMES.length]}`, `${town}, ${state}`, use])];
  const locations = sites.map(([name, street, town, use], index) => {
    const buildings = Array.from({ length: pick.int(1, 4) }, (_, building) => {
      const year = pick.int(1890, 2022);
      const area = pick.int(18, 640) * 100;
      return {
        number: building + 1,
        construction: CONSTRUCTION[pick.int(0, CONSTRUCTION.length - 1)],
        roof: ROOFS[pick.int(0, ROOFS.length - 1)],
        year,
        area,
        stories: pick.int(1, 3),
        sprinklered: pick.chance(0.6),
        alarm: pick.pick(['central fire', 'local fire', 'central fire and burglar', 'none']),
        building: area * pick.int(140, 420) * 100,
        contents: area * pick.int(30, 260) * 100,
        heating: pick.pick(['gas forced air', 'oil-fired hot water', 'steam from the process boiler', 'heat pumps', 'propane unit heaters']),
        wiring: pick.int(1975, 2024),
        plumbing: pick.pick(['copper', 'PEX', 'galvanized, partly replaced', 'copper and PVC']),
        exposures: ['north', 'east', 'south', 'west'].map(() => `${pick.pick(['open lot', 'street', 'parking', 'brick store', 'frame dwelling', 'river', 'rail siding', 'woods', 'insured building'])}, ${pick.int(10, 300)} ft`),
      };
    });
    return { number: index + 1, name, street, town, use, protection: pick.int(2, 9), buildings };
  });
  const scheduleA = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Location', width: 0.27 }, { header: 'Address', width: 0.36 }, { header: 'Occupancy', width: 0.3 }],
    locations.map((location) => [String(location.number), location.name, `${location.street}, ${location.town}`, location.use]),
    { title: 'SCHEDULE OF LOCATIONS', intro: 'The premises at which the insurance applies. Newly acquired locations are covered under Coverage Extension A.5 until reported.', chunk: 6 });
  const values = locations.flatMap((location) => location.buildings.map((building) => [`${location.number}-${building.number}`, grouped(building.area), String(building.stories), String(building.year), building.construction, building.roof, money(building.building), money(building.contents)]));
  const scheduleB = tableBlocks(flow, [{ header: 'Bldg', width: 0.07 }, { header: 'Sq ft', width: 0.09, align: 'right' }, { header: 'Floors', width: 0.07, align: 'right' }, { header: 'Built', width: 0.07, align: 'right' }, { header: 'Construction', width: 0.2 }, { header: 'Roof', width: 0.16 }, { header: 'Building value', width: 0.17, align: 'right' }, { header: 'Contents', width: 0.17, align: 'right' }], values,
    { title: 'STATEMENT OF VALUES', intro: 'Replacement cost values reported by the Named Insured for the blanket limits; the margin clause in Section C limits recovery at a location to 110 percent of its reported values.', chunk: 6 });
  const cope = locations.flatMap((location) => location.buildings.map((building) => [`${location.number}-${building.number}`, `class ${location.protection}`, building.sprinklered ? 'wet system, full coverage' : 'unsprinklered', building.alarm, `${pick.int(1, 9)} mi to station; hydrant ${pick.int(50, 1200)} ft`]));
  const scheduleC = tableBlocks(flow, [{ header: 'Bldg', width: 0.08 }, { header: 'Protection', width: 0.12 }, { header: 'Sprinklers', width: 0.22 }, { header: 'Alarm', width: 0.3 }, { header: 'Fire service', width: 0.28 }], cope,
    { title: 'SCHEDULE OF OCCUPANCY AND PROTECTION', intro: 'Construction, occupancy, protection and exposure as surveyed by our loss control representative.', chunk: 6 });
  const objects = locations.flatMap((location) => OBJECTS.filter((_, index) => (index + location.number) % 3 !== 0 && pick.chance(0.55)).map(([object, value]) => [`${location.number}`, object, `${object.slice(0, 3).toUpperCase()}-${pick.int(10000, 99999)}`, String(pick.int(1998, 2025)), money(Math.round(value * pick.int(80, 125) / 100))]));
  const scheduleD = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Object', width: 0.43 }, { header: 'Serial', width: 0.15 }, { header: 'Year', width: 0.1, align: 'right' }, { header: 'Replacement value', width: 0.25, align: 'right' }], objects,
    { title: 'SCHEDULE OF EQUIPMENT BREAKDOWN OBJECTS', intro: 'Objects covered under form GR 41 12. Objects not listed are covered if of a type listed and acquired during the policy period.', chunk: 6 });
  const safeguards = locations.flatMap((location) => location.buildings.filter((building) => building.sprinklered || building.alarm !== 'none').map((building) => [`${location.number}-${building.number}`, building.sprinklered ? 'P-1 automatic sprinkler system' : 'P-2 automatic fire alarm', building.alarm === 'none' ? '-' : building.alarm, pick.pick(['P-4 hood suppression in kitchen', 'none', 'P-3 security service', 'none'])]));
  const scheduleE = tableBlocks(flow, [{ header: 'Bldg', width: 0.08 }, { header: 'Safeguard', width: 0.34 }, { header: 'Alarm service', width: 0.34 }, { header: 'Other', width: 0.24 }], safeguards,
    { title: 'SCHEDULE OF PROTECTIVE SAFEGUARDS', intro: 'Under form CP 04 11 you must maintain the safeguards below in complete working order. We will not pay for fire loss at a building if you failed to keep a listed safeguard in working order and knew of the suspension.', chunk: 6 });
  const systems = tableBlocks(flow, [{ header: 'Bldg', width: 0.07 }, { header: 'Heating', width: 0.17 }, { header: 'Wiring', width: 0.08, align: 'right' }, { header: 'Plumbing', width: 0.12 }, { header: 'North', width: 0.14 }, { header: 'East', width: 0.14 }, { header: 'South', width: 0.14 }, { header: 'West', width: 0.14 }],
    locations.flatMap((location) => location.buildings.map((building) => [`${location.number}-${building.number}`, building.heating, String(building.wiring), building.plumbing, ...building.exposures])),
    { title: 'SCHEDULE OF BUILDING SYSTEMS AND EXPOSURES', intro: 'As surveyed by our loss control representative and reported in the application: the heating, the year the wiring was last updated, the plumbing, and what lies on each side of the building and how far away. A change must be reported within thirty days.', chunk: 6, size: 8 });
  // Vessels covered by form GR 41 07, at the sites that brew, ferment or age.
  const tanks = locations.filter((location) => /brew|cellar|cider|canning|barrel|malt/i.test(`${location.name} ${location.use}`)).flatMap((location) => Array.from({ length: pick.int(4, 12) }, (_, index) => {
    const kind = pick.pick([['Fermenter', 30, 120], ['Bright tank', 30, 90], ['Mash tun', 15, 40], ['Lauter tun', 15, 40], ['Hot liquor tank', 20, 60], ['Cold liquor tank', 20, 60], ['Unitank', 15, 60], ['Foeder', 20, 80], ['Cider fermenter', 10, 40], ['Wastewater equalization tank', 40, 160]]);
    const barrels = pick.int(kind[1], kind[2]);
    return [String(location.number), `${kind[0]} ${index + 1}`, `${barrels} bbl`, pick.pick(['304 stainless, jacketed', '316 stainless, jacketed', 'oak, French', 'oak, American', 'stainless, single wall', 'HDPE']), String(pick.int(1996, 2025)), money(barrels * pick.int(310, 820) * 100), money(barrels * pick.int(180, 1200) * 100)];
  }));
  const scheduleT = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Vessel', width: 0.22 }, { header: 'Capacity', width: 0.11, align: 'right' }, { header: 'Construction', width: 0.2 }, { header: 'Year', width: 0.08, align: 'right' }, { header: 'Vessel value', width: 0.16, align: 'right' }, { header: 'Typical contents', width: 0.16, align: 'right' }], tanks,
    { title: 'SCHEDULE OF TANKS AND VESSELS', intro: 'Vessels covered under form GR 41 07, with their replacement value and the value of the product they typically hold. A vessel installed during the policy period is covered from installation if reported within ninety days.', chunk: 6 });
  const OUTDOOR = ['Illuminated pylon sign', 'Wall sign, channel letters', 'Beer garden pergola', 'Outdoor seating and patio heaters', 'Perimeter fencing', 'Painted mural', 'Patio string lighting', 'Gas fire pit', 'Hop trellis system', 'Grain silo, 40-ton', 'Delivery van wrap and signage'];
  const outdoor = locations.flatMap((location) => OUTDOOR.filter(() => pick.chance(0.17)).map((item) => [String(location.number), item, pick.pick(['attached to building', 'freestanding', 'seasonal, stored in winter', 'on leased land']), money(pick.int(15, 900) * 10000)]));
  const scheduleO = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Item', width: 0.43 }, { header: 'Installation', width: 0.28 }, { header: 'Value', width: 0.22, align: 'right' }], outdoor,
    { title: 'SCHEDULE OF OUTDOOR PROPERTY', intro: 'Signs, structures and furnishings insured under the outdoor property extension; trees, shrubs and plants are subject to the per-item limit.', chunk: 8 });
  const rating = locations.flatMap((location) => location.buildings.map((building) => {
    const base = pick.int(18, 64) / 1000;
    const credit = building.sprinklered ? pick.int(25, 45) : 0;
    return [`${location.number}-${building.number}`, pick.pick(['0931 brewery', '0932 distillery and winery', '0570 restaurant', '0702 tavern', '1150 warehouse', '0567 retail', '0311 farm building']), `${location.protection}`, building.construction, base.toFixed(3), credit ? `-${credit}%` : 'none', (base * (100 - credit) / 100 + (building.year < 1950 ? 0.012 : 0)).toFixed(3)];
  }));
  const scheduleN = tableBlocks(flow, [{ header: 'Bldg', width: 0.08 }, { header: 'Class', width: 0.24 }, { header: 'PC', width: 0.06, align: 'right' }, { header: 'Construction', width: 0.22 }, { header: 'Base rate', width: 0.12, align: 'right' }, { header: 'Sprinkler credit', width: 0.14, align: 'right' }, { header: 'Final rate', width: 0.14, align: 'right' }], rating,
    { title: 'RATING WORKSHEET', intro: 'Building rates per $100 of value: the class and protection class base rate, the credit for an approved sprinkler system, and a load for buildings built before 1950.', chunk: 8 });
  fillTo(flow, 30, chain(scheduleA, scheduleB, scheduleC, scheduleD, scheduleE, systems, scheduleN, scheduleO, scheduleT), { id });

  // The declarations, on page 30.
  const [representative] = people(rng.fork('signers'), 1);
  flow.heading('COMMON POLICY DECLARATIONS', { level: 1, align: 'center', size: 13, before: 4 });
  const total = locations.reduce((sum, location) => sum + location.buildings.reduce((inner, building) => inner + building.building + building.contents, 0), 0);
  const premium = Math.round(total * 0.0021 / 100) * 100;
  flow.table([{ header: 'Item', width: 0.3 }, { header: 'Declaration', width: 0.7 }], [
    ['Policy Number', POLICY_NUMBER],
    ['Named Insured', INSURED],
    ['Mailing Address', '410 Millrace Road, Hollowmere, Vermont 05751'],
    ['Policy Period', `From ${numericDate(effective)} to ${numericDate(expiration)} at 12:01 A.M. standard time at the mailing address of the Named Insured`],
    ['Form of Business', 'Cooperative association'],
    ['Producer', `${BROKER}, 90 Bank Street, Burlington, Vermont 05401`],
    ['Business Description', 'Brewing, packaging and distribution of beer and cider; taprooms and retail'],
    ['Building - blanket limit', money(locations.reduce((sum, location) => sum + location.buildings.reduce((inner, building) => inner + building.building, 0), 0))],
    ['Business Personal Property - blanket limit', money(locations.reduce((sum, location) => sum + location.buildings.reduce((inner, building) => inner + building.contents, 0), 0))],
    ['Business Income and Extra Expense', '$9,500,000, 12 months actual loss sustained'],
    ['Deductible', '$10,000 per occurrence; $25,000 equipment breakdown; 2% named storm'],
    ['Coinsurance', 'Not applicable (agreed value, Statement of Values on file)'],
    ['Total annual premium', money(premium)],
    ['Renewal of', 'Policy GRM-CPP-4417304'],
    ['Countersigned', `${numericDate(countersigned)} by ${representative}, authorized representative`],
  ], { size: 9.5 });
  flow.paragraph('These Declarations, together with the Common Policy Conditions, the coverage forms and the endorsements listed on page 1, complete the policy. Premium is payable in four installments; see the Schedule of Premium by Location.', { size: 9.5 });

  // The schedules after the declarations, to page 60.
  const premiumRows = locations.flatMap((location) => ['Building', 'Business Personal Property', 'Business Income', 'Equipment Breakdown'].map((coverage) => [String(location.number), location.name, coverage, `${(pick.int(80, 420) / 1000).toFixed(3)}`, money(pick.int(4, 280) * 10000)]));
  const scheduleF = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Location', width: 0.3 }, { header: 'Coverage', width: 0.3 }, { header: 'Rate', width: 0.11, align: 'right' }, { header: 'Premium', width: 0.22, align: 'right' }], premiumRows,
    { title: 'SCHEDULE OF PREMIUM BY LOCATION', intro: 'Rates are per $100 of insured value. The premium is subject to audit of the values reported at expiration.', chunk: 8 });
  const mortgageeRows = MORTGAGEES.flatMap((bank) => locations.filter(() => pick.chance(0.2)).slice(0, 3).map((location) => [bank, `${location.number}-${pick.int(1, location.buildings.length)}`, pick.pick(['Mortgageholder', 'Loss Payee', 'Lender\'s Loss Payable', 'Contract of Sale']), `Loan ${pick.int(100000, 999999)}`]));
  const scheduleG = tableBlocks(flow, [{ header: 'Name', width: 0.42 }, { header: 'Bldg', width: 0.1 }, { header: 'Interest', width: 0.26 }, { header: 'Reference', width: 0.22 }], mortgageeRows,
    { title: 'SCHEDULE OF MORTGAGEES AND LOSS PAYEES', intro: 'Persons with an interest in Covered Property under form CP 12 18. They are not insureds and are not parties to this policy.', chunk: 6 });
  const control = locations.flatMap((location) => RECOMMENDATIONS.filter(() => pick.chance(0.22)).map((text) => [String(location.number), `${String(location.number).padStart(2, '0')}-${pick.int(10, 99)}`, text, pick.pick(['Critical', 'Important', 'Advisory'])]));
  const scheduleH = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Ref.', width: 0.09 }, { header: 'Recommendation', width: 0.66 }, { header: 'Priority', width: 0.18 }], control,
    { title: 'LOSS CONTROL RECOMMENDATIONS', intro: `From the survey of ${numericDate(inspection)}. Critical recommendations must be completed within ninety days; we may adjust the premium or the terms if they are not.`, chunk: 5 });
  const history = Array.from({ length: 34 }, (_, index) => {
    const location = locations[pick.int(0, locations.length - 1)];
    const year = 2021 + Math.floor(index / 7);
    const date = addDays(`${year}-07-15`, pick.int(0, 340));
    const paid = pick.int(0, 3600) * 10000;
    return [`C${year % 100}-${String(pick.int(1000, 9999))}`, numericDate(date), `${location.number}`, CAUSES[pick.int(0, CAUSES.length - 1)], paid ? money(paid) : 'closed without payment'];
  });
  const scheduleI = tableBlocks(flow, [{ header: 'Claim', width: 0.13 }, { header: 'Date of loss', width: 0.14 }, { header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Cause', width: 0.42 }, { header: 'Paid', width: 0.24, align: 'right' }], history,
    { title: 'FIVE-YEAR LOSS HISTORY', intro: `Losses reported under this insurer's policies for the Named Insured in the five policy periods before this one, the last of which began ${numericDate(prior)}, as stated in the application signed ${numericDate(application)}.`, chunk: 6 });
  const sublimits = [
    ['Debris removal', '25% of loss plus $250,000'], ['Pollutant clean-up', '$100,000 annual aggregate'], ['Electronic data', '$250,000'],
    ['Valuable papers and records', '$500,000'], ['Accounts receivable', '$750,000'], ['Newly acquired buildings', '$5,000,000 for 180 days'],
    ['Property off premises, including kegs at customers', '$1,000,000'], ['Property in transit', '$500,000'], ['Outdoor property', '$250,000; $5,000 per tree'],
    ['Spoilage and contamination', '$2,500,000; $5,000 deductible'], ['Tank collapse and leakage', '$3,000,000'], ['Barrel-aged inventory', 'agreed values per Schedule J'],
    ['Utility services', '$1,000,000; 24-hour waiting period'], ['Ordinance or law, coverage B and C', '$2,000,000'], ['Flood', '$2,000,000 aggregate; excluded at Hop Yard Barn'],
    ['Earth movement', '$2,000,000 aggregate'], ['Named storm', '2% of values at the location, minimum $25,000'], ['Fine arts in taprooms', '$100,000'],
    ['Employee theft', '$50,000'], ['Brand and labels', 'included'], ['Civil authority', '30 days'], ['Extended period of indemnity', '180 days'],
  ];
  const scheduleJ = tableBlocks(flow, [{ header: 'Coverage', width: 0.5 }, { header: 'Sublimit and deductible', width: 0.5 }], sublimits,
    { title: 'SCHEDULE OF SUBLIMITS AND DEDUCTIBLES', intro: 'Sublimits are part of, not in addition to, the blanket limits, unless stated otherwise.', chunk: 6 });
  const barrels = locations.filter((location) => /barrel|cellar|Cider/.test(`${location.name} ${location.use}`)).flatMap((location) => Array.from({ length: 18 }, (_, index) => [String(location.number), `${pick.pick(['Bourbon', 'Rye', 'Port', 'Red wine', 'Maple syrup', 'Calvados'])} barrel lot ${pick.int(100, 999)}`, `${pick.int(4, 60)} barrels`, `${pick.int(6, 36)} months`, money(pick.int(30, 900) * 100000)]));
  const scheduleK = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Lot', width: 0.38 }, { header: 'Quantity', width: 0.16 }, { header: 'Age', width: 0.14 }, { header: 'Agreed value', width: 0.25, align: 'right' }], barrels,
    { title: 'SCHEDULE J - BARREL-AGED INVENTORY, AGREED VALUES', intro: 'Agreed values under form GR 41 33, reported quarterly. A lot moved between the listed locations keeps its agreed value.', chunk: 6 });
  const income = locations.flatMap((location) => ['Net sales', 'Cost of goods sold', 'Payroll, ordinary', 'Continuing expenses', 'Extra expense estimate'].map((line) => [String(location.number), location.name, line, money(pick.int(5, 900) * 1000000)]));
  const scheduleL = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Location', width: 0.33 }, { header: 'Line', width: 0.34 }, { header: 'Annual amount', width: 0.26, align: 'right' }], income,
    { title: 'BUSINESS INCOME WORKSHEET', intro: 'Annual figures reported on the worksheet supporting the Business Income limit. They do not limit recovery, which is the actual loss sustained.', chunk: 8 });
  const contacts = blocks([0], () => {
    flow.heading('CLAIMS AND SERVICE', { level: 2 });
    flow.paragraph(`Report a loss at any hour to the claims center at ${phone(pick)} or through your producer, ${BROKER}. For equipment breakdown, call the inspection service first so that the object can be inspected before repair.`, { size: 9.5 });
  });
  const maintenance = locations.flatMap((location) => OBJECTS.filter((_, index) => (index * 7 + location.number) % 4 === 0).map(([object]) => [String(location.number), object, pick.pick(['annual inspection', 'oil analysis', 'vibration analysis', 'pressure test', 'infrared scan', 'safety valve test']), `${2024 + pick.int(0, 1)}-${String(pick.int(1, 12)).padStart(2, '0')}`, pick.pick(['satisfactory', 'satisfactory', 'minor repairs completed', 'recommendation issued', 'deferred to spring'])]));
  const scheduleM = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Object', width: 0.36 }, { header: 'Maintenance', width: 0.2 }, { header: 'Month', width: 0.12 }, { header: 'Result', width: 0.25 }], maintenance,
    { title: 'EQUIPMENT BREAKDOWN MAINTENANCE RECORDS', intro: 'The maintenance the Named Insured reported for the objects in the equipment breakdown schedule, on which form GR 41 02 depends. Objects without a record within fifteen months are subject to the inspection condition.', chunk: 8 });
  const PLACES = ['Copper', 'Lantern', 'Ferry', 'Old Mill', 'Granite', 'Kingfisher', 'Hearth', 'Maple Leaf', 'Riverbend', 'Two Owls', 'Blue Heron', 'Ledge', 'Snowshoe', 'Covered Bridge', 'Iron Horse', 'Birch Hollow'];
  const VENUES = ['Tavern', 'Pub', 'Bistro', 'Grill', 'Inn', 'Ale House', 'Market', 'Kitchen'];
  const accounts = [...new Set(Array.from({ length: 60 }, () => `${pick.pick(PLACES)} ${pick.pick(VENUES)}`))].slice(0, 36);
  const kegs = accounts.map((account) => {
    const half = pick.int(0, 24);
    const sixth = pick.int(0, 40);
    return [account, sites[pick.int(0, sites.length - 1)][2], String(half), String(sixth), money(half * 19000 + sixth * 11000)];
  });
  const scheduleP = tableBlocks(flow, [{ header: 'Account', width: 0.34 }, { header: 'Town', width: 0.3 }, { header: '1/2 bbl', width: 0.1, align: 'right' }, { header: '1/6 bbl', width: 0.1, align: 'right' }, { header: 'Keg value', width: 0.16, align: 'right' }], kegs,
    { title: 'SCHEDULE OF KEGS AT CUSTOMERS\' PREMISES', intro: 'Kegs owned by the Named Insured and held by accounts at the last quarterly count, covered under the property off premises extension.', chunk: 8 });
  const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
  const monthly = locations.flatMap((location) => MONTH_NAMES.map((name, index) => {
    const season = 1 + 0.35 * Math.sin(((index - 3) / 12) * 2 * Math.PI);
    const sales = Math.round(pick.int(40, 900) * season) * 100000;
    return [String(location.number), `${name} 2025`, money(sales), money(Math.round(sales * pick.int(28, 46) / 100)), money(Math.round(sales * pick.int(18, 32) / 100)), money(Math.round(sales * pick.int(9, 21) / 100))];
  }));
  const scheduleQ = tableBlocks(flow, [{ header: 'Loc.', width: 0.07, align: 'right' }, { header: 'Month', width: 0.13 }, { header: 'Net sales', width: 0.2, align: 'right' }, { header: 'Cost of goods', width: 0.2, align: 'right' }, { header: 'Payroll', width: 0.2, align: 'right' }, { header: 'Continuing', width: 0.2, align: 'right' }], monthly,
    { title: 'BUSINESS INCOME MONTHLY REPORT', intro: 'Monthly figures supporting the Business Income Worksheet, location by location, from the Named Insured\'s 2025 accounts. An extract; the full report is on file with the producer.', chunk: 8 });
  // The monthly report, the longest, comes last.
  fillTo(flow, 60, chain(scheduleF, scheduleG, scheduleH, scheduleI, scheduleJ, scheduleK, scheduleL, contacts, scheduleM, scheduleP, scheduleQ), { id, room: 120 });
  flow.paragraph('NOTICE TO POLICYHOLDERS: This policy is issued by a mutual company. You are a member of the company while this policy is in force and are entitled to vote at its annual meeting and to participate in any distribution of surplus declared by its board of directors. This policy is non-assessable.', { size: 9, before: 6 });

  const pages = flow.finish();
  if (pages.length !== 60) throw new Error(`${id}: expected 60 pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  expectPages(id, text, numericDate(effective), [30]);
  const lines = text.flatMap((page) => page.split('\n'));
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Sixty-page commercial property policy with its declarations in the middle',
    kind: 'insurance',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_50', 'financial', 'date_in_table', 'middle_fact', 'irrelevant_names', 'competing_dates', 'information_dense', 'table'],
    notes: `A renewal packet with the coverage forms and location schedules first: the Common Policy Declarations are on page 30, and the Policy Period there (from ${numericDate(effective)} to ${numericDate(expiration)}) is the only place the effective date appears. The same table gives the countersignature date (${numericDate(countersigned)}); the loss history dates the previous policy period (${numericDate(prior)}) and the application (${numericDate(application)}); the loss control schedule dates the survey (${numericDate(inspection)}). The producer and eight banks with an interest in the buildings are named but are not parties.`,
    structure: structure({
      readingOrder: readingSnippets(lines, 8),
      keyValues: [['Policy Number', POLICY_NUMBER], ['Named Insured', INSURED]],
    }),
    recording: 'pending',
    gold: gold({
      type: 'Commercial Property Policy',
      acceptableTypes: ['Commercial Property Insurance Policy', 'Property Insurance Policy', 'Insurance Policy', 'Commercial Property Policy Declarations'],
      date: effective,
      role: 'effective',
      forbiddenDates: [[expiration, 'policy expiration'], [countersigned, 'countersignature date'], [prior, 'start of the previous policy period'], [application, 'application date'], [inspection, 'loss control survey date']],
      parties: [INSURED],
      relation: 'for',
      acceptablePartySets: [{ parties: [INSURER], relation: 'from' }],
      roles: [[INSURED, 'subject'], [INSURED, 'customer'], [INSURER, 'issuer'], [INSURER, 'provider'], [INSURER, 'vendor']],
      forbiddenParties: [[BROKER, 'producer'], [MORTGAGEES[0], 'mortgageholder'], [MORTGAGEES[1], 'mortgageholder']],
      facts: [['brewing', 'brewer', 'beer'], ['Vermont']],
      subjectTerms: ['commercial property', 'brewery', 'policy period'],
      readiness: 'ready',
      dateText: [numericDate(effective)],
      dateAnchor: 'Policy Period',
      typeText: ['COMMERCIAL PROPERTY POLICY'],
      identifierText: [POLICY_NUMBER],
    }),
  });
}
