/// A one-hundred-page annual report of a farmers' cooperative. The cover
/// says only "Fiscal Year 2026"; the report is dated once, on the
/// independent auditors' report in the middle of the book. Every page is
/// built from a data model of divisions, locations, members, and accounts,
/// so no two pages say the same thing.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { amount, decimal, grouped, longDate, money, percent } from '../lib/format.mjs';
import { people } from '../lib/names.mjs';
import { digitalPdf, result } from './common.mjs';
import { pageContaining } from './contracts.mjs';

const COOP = 'Tamsin Valley Farmers Cooperative';
const AUDITOR = 'Wainwright Lindqvist LLP';
const MONTHS = ['Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec', 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun'];

const DIVISIONS = [
  { key: 'grain', name: 'Grain Marketing', unit: 'thousand bushels', products: ['corn', 'wheat', 'sorghum', 'soybeans'], base: 4200, margin: 610 },
  { key: 'agronomy', name: 'Agronomy', unit: 'tons of fertilizer', products: ['anhydrous ammonia', 'urea', 'potash', 'crop protection'], base: 610, margin: 1480 },
  { key: 'energy', name: 'Energy', unit: 'thousand gallons', products: ['diesel', 'gasoline', 'propane', 'lubricants'], base: 2380, margin: 920 },
  { key: 'feed', name: 'Animal Nutrition', unit: 'tons of feed', products: ['dairy rations', 'beef supplements', 'mineral', 'pet and equine'], base: 5400, margin: 1150 },
  { key: 'dairy', name: 'Dairy Processing', unit: 'thousand pounds of milk', products: ['fluid milk', 'cheese', 'butter', 'whey protein'], base: 31000, margin: 830 },
  { key: 'retail', name: 'Farm and Ranch Retail', unit: 'thousand transactions', products: ['hardware', 'fencing', 'animal health', 'workwear'], base: 38, margin: 2650 },
];

/// Each location with its role, its largest capital project, and what
/// marked its year - consistent with the division reviews and the year in
/// review elsewhere in the book.
const LOCATIONS = [
  ['Tamsin Valley', 'NM', 'Headquarters, grain terminal, agronomy center, retail store', 'dryer burner upgrades on both grain dryers', 'a single-day record of 412,000 bushels received in July'],
  ['Copper Flats', 'AZ', 'Grain elevator, energy cardlock', 'a card reader and canopy at the fuel island', 'drought that cut local sorghum yields by a third'],
  ['Silverlode', 'NV', 'Feed mill, retail store', 'a pellet mill rebuild', 'a new customer, a 4,000-head feedlot'],
  ['Kestrel Ridge', 'UT', 'Dairy processing plant', 'a cheese brine system upgrade', 'a successful food safety audit with no findings'],
  ['Marrow Springs', 'CO', 'Agronomy center, propane plant', 'a second anhydrous nurse-tank bay', 'a January cold snap that doubled propane deliveries for nine days'],
  ['Fallow Creek', 'TX', 'Grain elevator, agronomy center', 'a new dryer burner and truck probe', 'a 9% increase in custom application acres'],
  ['Dunmore Lake', 'ID', 'Feed mill, energy cardlock', 'dust collection replacement', 'the first full year under its new manager'],
  ['Oriel Junction', 'MO', 'Grain shuttle loader', 'rail spur rehabilitation ahead of the loop track expansion', 'a new rail contract with two shuttle trains a month'],
  ['Quarry Bend', 'TN', 'Retail store, propane plant', 'a 30,000-gallon propane storage tank', 'lower volume after a competitor opened 12 miles away'],
  ['Bellmoor', 'WI', 'Cheese plant (joint venture)', 'a whey evaporator overhaul', 'zero lost-time injuries for the third consecutive year'],
  ['Linden Cross', 'PA', 'Energy terminal', 'solar panels on the warehouse roof', 'the transition to automated delivery dispatch'],
  ['Briarport', 'NC', 'Animal health distribution center', 'a pharmacy cold room', 'same-day shipping on orders placed by noon'],
];

/// Which divisions a location's role puts it in.
const ROLE_PATTERNS = { grain: /grain|elevator|shuttle|terminal/i, agronomy: /agronomy/i, energy: /energy|propane|cardlock/i, feed: /feed/i, dairy: /dairy|cheese/i, retail: /retail|animal health/i };

const COUNTIES = ['Alamosa Bend', 'Cibola Flats', 'Guadarama', 'Harrow', 'Kettle', 'Los Pinos', 'Mesa Roja', 'Sandoval Hills', 'Torrance Ridge', 'Valencia Wells'];

/// What distinguishes each county's membership; assigned one to a county.
const COUNTY_TRAITS = [
  ['delivered the most grain of any county', 'Two new members joined after a competitor\'s elevator closed.'],
  ['bought more fertilizer per acre than the cooperative average', 'A member meeting in March drew 140 people.'],
  ['shipped milk from 14 dairies to Kestrel Ridge', 'Three of those dairies added robotic milking during the year.'],
  ['run mostly cow-calf operations on rangeland', 'Mineral program enrollment doubled after the dry summer.'],
  ['grow the most irrigated acres in the trade area', 'Several long-time members retired and leased their land to younger members.'],
  ['were hit hardest by the spring drought', 'The board approved a deferred payment plan for 31 affected members.'],
  ['include the cooperative\'s two largest pecan growers', 'The county\'s 4-H program received a cooperative grant for a livestock barn.'],
  ['use more propane for crop drying than any other county', 'Membership has grown every year since 2019.'],
  ['raise most of the chile peppers the cooperative markets', 'A new grower group formed to bargain with processors.'],
  ['have the youngest average age of any county, at 46', 'Nine members completed the Young Producers Council program.'],
];

/// Thousands of dollars, printed with grouping.
const k = (value) => grouped(Math.round(value));

export function annualReport() {
  const id = 'annual-report-100p';
  const rng = Rng.from(id);
  const yearEnd = '2026-06-30';
  const priorYearEnd = '2025-06-30';
  const reportDate = '2026-09-18';
  const meeting = '2026-12-03';
  const patronageDate = '2026-12-15';
  const managers = people(rng.fork('managers'), LOCATIONS.length);
  const directors = people(rng.fork('directors'), 9, { exclude: managers });
  const executives = people(rng.fork('executives'), 7, { exclude: [...managers, ...directors] });

  // The data model. Revenue and margin by division for five years, in
  // thousands of dollars; volumes by month; locations with their own facts.
  const data = rng.fork('model');
  const years = ['2022', '2023', '2024', '2025', '2026'];
  const divisions = DIVISIONS.map((division) => {
    const revenue = [];
    let value = division.base * data.int(38, 62);
    for (let year = 0; year < 5; year += 1) {
      revenue.push(Math.round(value));
      value *= 1 + data.int(-60, 140) / 1000;
    }
    const margins = revenue.map((sales, index) => Math.round((sales * (division.margin + data.int(-120, 120) + index * 8)) / 10000));
    const monthly = MONTHS.map(() => Math.round(division.base * (0.6 + data.float() * 0.9)));
    return { ...division, revenue, margins, monthly, employees: data.int(60, 420), capex: data.int(900, 9800) };
  });
  const totalRevenue = years.map((_, index) => divisions.reduce((sum, division) => sum + division.revenue[index], 0));
  const totalMargin = years.map((_, index) => divisions.reduce((sum, division) => sum + division.margins[index], 0));
  const operatingExpense = totalMargin.map((margin) => Math.round(margin * (0.62 + data.float() * 0.06)));
  const localSavings = totalMargin.map((margin, index) => margin - operatingExpense[index]);
  const patronageIncome = localSavings.map((value) => Math.round(value * 0.88));
  const members = COUNTIES.map((county) => ({ county, active: data.int(240, 1180), new: data.int(8, 64), equity: data.int(2600, 21400) }));
  const locations = LOCATIONS.map(([town, state, role, project, note], index) => ({
    town, state, role, project, note, manager: managers[index],
    acres: data.int(6, 140), employees: data.int(9, 140), storage: data.int(0, 14) * 250, built: data.int(1948, 2019),
    volume: data.int(800, 24000), safetyDays: data.int(40, 2100), recordables: data.int(0, 6), capex: data.int(80, 6400),
    divisions: DIVISIONS.filter((division) => ROLE_PATTERNS[division.key].test(role)),
  }));

  // Five-year history per location, shared by the location pages and the
  // five-year statistics table so the two agree.
  const trends = rng.fork('location-history');
  for (const location of locations) {
    location.trend = years.map((_, index) => (index === 4 ? location.volume : Math.round(location.volume * (0.7 + index * 0.06 + trends.float() * 0.1))));
    location.staff = years.map((_, index) => (index === 4 ? location.employees : Math.max(4, location.employees + trends.int(-12, 12))));
  }

  const flow = new Flow({
    face: 'serif', fontSize: 10.5, leading: 1.35, margins: { top: 70, bottom: 70 }, keep: [COOP, AUDITOR],
    header: (page, { number }) => {
      if (number > 2) page.text(72, 46, `${COOP} - Annual Report, Fiscal Year 2026`, { face: 'sans', size: 7.5, grey: 0.4 });
    },
    footer: (page, { number }) => {
      if (number > 1) page.textCenter(306, 752, String(number), { face: 'sans', size: 8, grey: 0.4 });
    },
  });
  const cover = flow.page;
  cover.rect(0, 0, 612, 792, { fill: 0.9, stroke: null });
  cover.text(72, 300, COOP, { face: 'sans-bold', size: 24 });
  cover.text(72, 345, 'Annual Report', { face: 'sans-bold', size: 36 });
  cover.text(72, 380, 'Fiscal Year 2026', { face: 'serif', size: 20 });
  cover.text(72, 690, 'Owned by the farmers and ranchers it serves since 1937', { face: 'sans', size: 10, grey: 0.25 });
  flow.pageBreak();

  // Sections are registered with the part of the book they belong to and
  // drawn part by part: front matter, divisions, the year, the audited
  // statements, locations and counties, back matter.
  const sections = [];
  let group = 1;
  const section = (title, draw) => sections.push([title, draw, group]);

  section('At a Glance', () => {
    flow.heading('At a Glance', { level: 1, size: 18 });
    flow.paragraph(`${COOP} is a farmer-owned supply and marketing cooperative serving ${grouped(members.reduce((sum, member) => sum + member.active, 0))} active members across ten counties from twelve locations in eleven states. Members own the cooperative, elect its board, and share in its earnings through patronage refunds based on the business they do with it.`);
    flow.table([{ header: 'Fiscal year 2026', width: 0.6 }, { header: 'Amount', width: 0.4, align: 'right' }], [
      ['Total revenue (thousands)', `$${k(totalRevenue[4])}`],
      ['Gross margin (thousands)', `$${k(totalMargin[4])}`],
      ['Local savings before income taxes (thousands)', `$${k(localSavings[4])}`],
      ['Patronage allocated to members (thousands)', `$${k(patronageIncome[4])}`],
      ['Cash patronage to be paid in December', '40% of allocation'],
      ['Members\' equity (thousands)', `$${k(members.reduce((sum, member) => sum + member.equity, 0) * 9)}`],
      ['Employees', grouped(divisions.reduce((sum, division) => sum + division.employees, 0))],
      ['Locations', '12'],
    ], { size: 9.5 });
    flow.paragraph('Revenue by division, fiscal 2026 (thousands of dollars):', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Division', width: 0.4 }, { header: 'Revenue', width: 0.2, align: 'right' }, { header: 'Share', width: 0.2, align: 'right' }, { header: 'Gross margin', width: 0.2, align: 'right' }], divisions.map((division) => [division.name, k(division.revenue[4]), percent(Math.round((division.revenue[4] * 10000) / totalRevenue[4]), 1), k(division.margins[4])]), { size: 9.5 });
  });

  section('Letter to Members', () => {
    flow.heading('To Our Members', { level: 1, size: 18 });
    const change = Math.round(((totalRevenue[4] - totalRevenue[3]) * 10000) / totalRevenue[3]);
    flow.paragraph(`Fiscal 2026 tested every part of the cooperative. Revenue ${change >= 0 ? 'rose' : 'fell'} ${percent(Math.abs(change), 1)} to $${k(totalRevenue[4])} thousand, while gross margin ${totalMargin[4] >= totalMargin[3] ? 'grew' : 'declined'} to $${k(totalMargin[4])} thousand. Grain prices spent most of the year below the cost of production for many of you, fertilizer prices eased only in the spring, and drought in the southern counties cut sorghum yields by as much as a third. Through it all, your cooperative kept its elevators open longer during harvest, extended input financing to 412 members, and delivered fuel and propane through the coldest January in eleven years.`);
    flow.paragraph(`Local savings before income taxes were $${k(localSavings[4])} thousand, of which $${k(patronageIncome[4])} thousand is patronage-sourced and will be allocated to members. The board has voted to pay 40% of the allocation in cash in December, the highest cash percentage since fiscal 2014, and to retire the remaining 2011 and 2012 allocated equity of members who have reached age 70 or whose estates have asked for it.`);
    flow.paragraph('We invested in the places that will matter for the next twenty years: a 1.2 million bushel concrete bin at Tamsin Valley, a pellet mill rebuild at Silverlode, and the brine system at the Kestrel Ridge dairy plant. Our employees worked 1.9 million hours with a recordable injury rate of 1.6, the best in the cooperative\'s history, and we thank them for it.');
    flow.paragraph('Next year we will finish the shuttle loader expansion at Oriel Junction, bring precision agronomy services to every agronomy location, and begin a review of our energy business, where the shift to renewable diesel is changing what members need from us. We will continue to report to you plainly about what we do and how it turns out.');
    flow.paragraph(`Respectfully, ${directors[0]}, Chair of the Board, and ${executives[0]}, President and Chief Executive Officer`, { face: 'sans-bold', size: 10 });
  });

  section('Strategic Plan Progress', () => {
    flow.heading('Strategic Plan 2024-2028: Progress', { level: 1, size: 16 });
    const plan = rng.fork('plan');
    flow.paragraph('In 2023 the board adopted a five-year plan with eight goals. This table reports where each stands after its third year.');
    flow.table([{ header: 'Goal', width: 0.46 }, { header: 'Target by FY2028', width: 0.22 }, { header: 'FY2026', width: 0.16 }, { header: 'Status', width: 0.16 }], [
      ['Raise grain handling capacity', '70 million bu./yr', `${plan.int(52, 66)} million`, 'On track'],
      ['Precision agronomy on member acres', '40% of acres', `${plan.int(24, 33)}%`, 'On track'],
      ['Recordable injury rate', 'Below 2.0', '1.6', 'Achieved'],
      ['Cash patronage percentage', '35% or more', '40%', 'Achieved'],
      ['Equity retirement cycle', '15 years', `${plan.int(16, 19)} years`, 'Behind'],
      ['Members\' equity to assets', '60%', `${plan.int(55, 61)}%`, 'On track'],
      ['Dairy plant utilization', '90%', `${plan.int(80, 89)}%`, 'Behind'],
      ['Scope 1 and 2 emissions', '-20% vs FY2022', `-${plan.int(8, 15)}%`, 'On track'],
    ], { size: 9, border: 'rules' });
    flow.paragraph(`The two goals behind schedule share a cause: weaker earnings in fiscal 2024 slowed equity retirement, and a competitor's new cheese plant drew milk away from Kestrel Ridge. The board will revisit both targets when it updates the plan in fiscal 2027. Capital spent under the plan so far totals $${k(plan.int(48000, 66000))} thousand of the $${k(plan.int(90000, 110000))} thousand budgeted for five years.`);
  });
  section('Market Review', () => {
    flow.heading('Market Review', { level: 1, size: 16 });
    const market = rng.fork('market');
    flow.paragraph('Monthly average local prices paid or charged by the cooperative during fiscal 2026. Grain prices are cash bids at the Tamsin Valley terminal; input prices are delivered retail prices.');
    const series = [['Corn ($/bu)', 380, 470, 2], ['Wheat ($/bu)', 510, 640, 2], ['Sorghum ($/bu)', 340, 450, 2], ['Soybeans ($/bu)', 960, 1130, 2], ['Anhydrous ammonia ($/ton)', 640, 820, 0], ['Urea ($/ton)', 470, 610, 0], ['Diesel ($/gal)', 318, 412, 2], ['Propane ($/gal)', 164, 249, 2], ['Class III milk ($/cwt)', 1620, 2140, 2], ['Feeder steers ($/cwt)', 24800, 31200, 2]];
    flow.table([{ header: 'Jul - Dec 2025', width: 0.28 }, ...MONTHS.slice(0, 6).map((month) => ({ header: month, width: 0.12, align: 'right' }))], series.map(([label, low, high, places]) => [label, ...MONTHS.slice(0, 6).map(() => decimal(market.int(low, high), places))]), { size: 8.5, border: 'rules' });
    flow.table([{ header: 'Jan - Jun 2026', width: 0.28 }, ...MONTHS.slice(6).map((month) => ({ header: month, width: 0.12, align: 'right' }))], series.map(([label, low, high, places]) => [label, ...MONTHS.slice(6).map(() => decimal(market.int(low, high), places))]), { size: 8.5, border: 'rules' });
    flow.paragraph(`Rainfall in the trade area was ${market.int(58, 86)}% of normal from April through June, and growing degree days were ${market.int(2, 9)}% above the ten-year average. The local basis for corn averaged ${market.int(18, 46)} cents under the nearby futures contract, ${market.int(3, 12)} cents weaker than fiscal 2025, mainly because rail freight to export terminals rose. Fertilizer prices declined steadily after the fall application season as world nitrogen supplies recovered.`);
  });
  section('Five-Year Financial Summary', () => {
    flow.heading('Five-Year Financial Summary', { level: 1, size: 16 });
    flow.paragraph('In thousands of dollars except per-member data; fiscal years end June 30.', { size: 9, grey: 0.3 });
    const row = (label, values) => [label, ...values.map((value) => k(value))];
    flow.table([{ header: '', width: 0.35 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.13, align: 'right' }))], [
      row('Revenue', totalRevenue),
      row('Gross margin', totalMargin),
      row('Operating expenses', operatingExpense),
      row('Local savings', localSavings),
      row('Patronage-sourced income', patronageIncome),
      row('Non-patronage income', localSavings.map((value, index) => value - patronageIncome[index])),
      row('Capital expenditures', years.map(() => data.int(9000, 26000))),
      row('Depreciation', years.map(() => data.int(8000, 14000))),
      row('Working capital', years.map(() => data.int(42000, 91000))),
      row('Long-term debt', years.map(() => data.int(38000, 72000))),
      row('Members\' equity', years.map(() => data.int(180000, 260000))),
    ], { size: 9, border: 'rules' });
    flow.paragraph('Gross margin by division (thousands of dollars):', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Division', width: 0.35 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.13, align: 'right' }))], divisions.map((division) => row(division.name, division.margins)), { size: 9, border: 'rules' });
    flow.paragraph('Ratios:', { face: 'sans-bold', size: 10 });
    flow.table([{ header: '', width: 0.35 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.13, align: 'right' }))], [
      ['Current ratio', ...years.map(() => decimal(data.int(130, 210), 2))],
      ['Equity to total assets', ...years.map(() => percent(data.int(5200, 6800), 1))],
      ['Return on local equity', ...years.map(() => percent(data.int(450, 1400), 1))],
      ['Operating expense / gross margin', ...operatingExpense.map((value, index) => percent(Math.round((value * 10000) / totalMargin[index]), 1))],
    ], { size: 9, border: 'rules' });
  });

  section('Membership', () => {
    flow.heading('Our Members', { level: 1, size: 16 });
    flow.paragraph('Membership is open to any agricultural producer who does at least $2,500 of business with the cooperative in a fiscal year and buys one share of common stock. Members who are inactive for three consecutive years are moved to inactive status but keep their allocated equity.');
    flow.table([{ header: 'County', width: 0.3 }, { header: 'Active members', width: 0.18, align: 'right' }, { header: 'New in FY2026', width: 0.16, align: 'right' }, { header: 'Allocated equity ($000)', width: 0.2, align: 'right' }, { header: 'Directors', width: 0.16, align: 'right' }], members.map((member, index) => [member.county, grouped(member.active), String(member.new), k(member.equity), String(index < 9 ? 1 : 0)]), { size: 9 });
    flow.paragraph(`Patronage history: allocations of $${k(patronageIncome[0])} thousand (FY2022), $${k(patronageIncome[1])} thousand (FY2023), $${k(patronageIncome[2])} thousand (FY2024), and $${k(patronageIncome[3])} thousand (FY2025) preceded this year's $${k(patronageIncome[4])} thousand. Equity retirements paid to members totaled $${k(data.int(2100, 4800))} thousand during the year, including estates of 61 members.`);
    flow.paragraph('Member age distribution: under 35, 14%; 35 to 54, 38%; 55 to 69, 33%; 70 and over, 15%. Women are principal operators on 27% of member farms. The Young Producers Council, now in its ninth year, had 118 participants.');
  });

  group = 2;
  divisions.forEach((division, index) => {
    section(`${division.name} Division`, () => {
      flow.heading(`${division.name}`, { level: 1, size: 16 });
      const growth = Math.round(((division.revenue[4] - division.revenue[3]) * 10000) / division.revenue[3]);
      const variant = rng.fork(`division-${division.key}`);
      flow.paragraph(`${division.name} handles ${division.products.slice(0, -1).join(', ')}, and ${division.products[division.products.length - 1]} for members and other customers. Revenue was $${k(division.revenue[4])} thousand in fiscal 2026, ${growth >= 0 ? 'up' : 'down'} ${percent(Math.abs(growth), 1)} from fiscal 2025, and gross margin was $${k(division.margins[4])} thousand. The division employs ${division.employees} people and invested $${k(division.capex)} thousand in capital projects during the year.`);
      flow.paragraph(variant.pick([
        `Volume peaked in ${MONTHS[division.monthly.indexOf(Math.max(...division.monthly))]} at ${grouped(Math.max(...division.monthly))} ${division.unit}, and the slowest month was ${MONTHS[division.monthly.indexOf(Math.min(...division.monthly))]} at ${grouped(Math.min(...division.monthly))}.`,
        `Monthly volume ranged from ${grouped(Math.min(...division.monthly))} to ${grouped(Math.max(...division.monthly))} ${division.unit}, a wider swing than in fiscal 2025 because of the late harvest and the January cold snap.`,
      ]));
      flow.table([{ header: `Monthly volume (${division.unit})`, width: 0.28 }, ...MONTHS.slice(0, 6).map((month) => ({ header: month, width: 0.12, align: 'right' }))], [
        ['Jul - Dec', ...division.monthly.slice(0, 6).map((value) => grouped(value))],
      ], { size: 8.5 });
      flow.table([{ header: `Monthly volume (${division.unit})`, width: 0.28 }, ...MONTHS.slice(6).map((month) => ({ header: month, width: 0.12, align: 'right' }))], [
        ['Jan - Jun', ...division.monthly.slice(6).map((value) => grouped(value))],
      ], { size: 8.5 });
      const productShares = division.products.map(() => variant.int(10, 45));
      const shareTotal = productShares.reduce((a, b) => a + b, 0);
      flow.table([{ header: 'Product line', width: 0.4 }, { header: 'Share of revenue', width: 0.2, align: 'right' }, { header: 'Revenue ($000)', width: 0.2, align: 'right' }, { header: 'Change vs FY2025', width: 0.2, align: 'right' }], division.products.map((product, productIndex) => [product, percent(Math.round((productShares[productIndex] * 10000) / shareTotal), 1), k((division.revenue[4] * productShares[productIndex]) / shareTotal), `${variant.chance(0.6) ? '+' : '-'}${decimal(variant.int(5, 190), 1)}%`]), { size: 9 });
      const commentary = [
        ['Receipts at harvest were concentrated in a six-week window, and the terminal at Tamsin Valley ran two shifts for 41 days. Basis was weaker than normal through December as rail freight to the Gulf rose; we stored grain under our deferred-price program for 640 members rather than sell into it.', 'Grain marketing margins depend on carry in the futures market, basis appreciation, and drying and storage income. All three were below the five-year average this year.'],
        ['Fertilizer prices fell 18% from their fiscal 2025 peak, which lowered revenue but restored margin on fall-applied nitrogen. Custom application acres grew to 212,000, and the variable-rate prescriptions written by our agronomists covered 31% of those acres, up from 22%.', 'Crop protection product returns were the lowest in five years because our agronomists booked products against field-by-field plans instead of whole-farm estimates.'],
        ['Diesel gallons rose with the late harvest and propane demand jumped during the January cold snap, when the energy team made 2,840 deliveries in nine days. Renewable diesel blends reached 8% of on-road diesel sales.', 'Lubricant margins improved after we moved to a single supplier with a three-year price agreement. Propane contract enrollment for the coming winter was 61% of expected gallons by August.'],
        ['Feed tons rose as dairy herds in the northern counties expanded and beef supplement sales recovered after two dry years. The pellet mill rebuild at Silverlode cut energy use per ton by 14%.', 'Mineral sales grew fastest, helped by a new custom-mix program for cow-calf operations that now serves 311 members.'],
        ['The Kestrel Ridge plant processed more milk than in any previous year, and cheese yield improved after the brine system upgrade. Whey protein concentrate sold at prices well above fiscal 2025, offsetting weaker butter prices.', 'Member milk was paid at an average of $19.84 per hundredweight, including a quality premium averaged across all shipping members of $0.41.'],
        ['Retail transactions increased at all three stores, and online orders for in-store pickup reached 11% of sales. Fencing and livestock handling equipment led growth; workwear and hardware were flat.', 'Inventory turns improved to 4.1 from 3.6 after a review that removed 2,300 slow-moving items.'],
      ][index];
      flow.paragraph(commentary[0]);
      flow.paragraph(commentary[1]);
      flow.paragraph(`Outlook: ${variant.pick(['we expect volumes close to this year\'s, with margin depending on weather and commodity prices', 'the budget assumes modest volume growth and stable margins', 'we expect lower revenue on lower prices but similar margin per unit', 'the division will spend more on equipment replacement than in any of the last five years'])}. Capital projects approved for fiscal 2027 total $${k(variant.int(800, 7200))} thousand.`);
    });
    section(`${division.name} by Location`, () => {
      const variant = rng.fork(`division-locations-${division.key}`);
      flow.heading(`${division.name}: Locations and Markets`, { level: 1, size: 15 });
      const served = locations.filter((location) => ROLE_PATTERNS[division.key].test(location.role));
      flow.table([{ header: 'Location', width: 0.3 }, { header: `Volume (${division.unit})`, width: 0.22, align: 'right' }, { header: 'Revenue ($000)', width: 0.16, align: 'right' }, { header: 'Margin ($000)', width: 0.16, align: 'right' }, { header: 'Staff', width: 0.16, align: 'right' }], served.map((location) => {
        const volume = variant.int(Math.round(division.base * 0.4), division.base * 3);
        return [`${location.town}, ${location.state}`, grouped(volume), k(division.revenue[4] / served.length * (0.6 + variant.float() * 0.8)), k(division.margins[4] / served.length * (0.6 + variant.float() * 0.8)), String(variant.int(4, 60))];
      }), { size: 8.5, border: 'rules' });
      const product = division.products[0];
      const prices = MONTHS.map(() => variant.int(80, 140));
      flow.paragraph(`Average monthly price index for ${product} (fiscal 2025 average = 100):`, { face: 'sans-bold', size: 10 });
      flow.table([{ header: 'Half', width: 0.16 }, ...MONTHS.slice(0, 6).map((month) => ({ header: month, width: 0.14, align: 'right' }))], [['Jul - Dec', ...prices.slice(0, 6).map(String)], ['Jan - Jun', ...prices.slice(6).map(String)]], { size: 8.5, border: 'rules', header: true });
      flow.paragraph(`Market conditions: ${variant.pick(['prices fell through the first half on large world supplies and recovered only partly after the spring planting report', 'prices rose early on tight local supplies, then eased when imports arrived', 'a strong dollar weighed on export demand for most of the year', 'freight costs, not commodity prices, drove most of the change in delivered cost'])}. Our average margin per unit was ${decimal(variant.int(40, 380), 2)} dollars, compared with ${decimal(variant.int(40, 380), 2)} dollars in fiscal 2025, and ${variant.int(55, 92)}% of volume was handled for members rather than non-member customers.`);
      flow.paragraph(`Competition: ${variant.int(2, 7)} competing facilities operate within 30 miles of at least one of our ${division.name.toLowerCase()} locations. We estimate our share of the trade area at ${variant.int(22, 61)}%, ${variant.pick(['unchanged from last year', 'up two points', 'down one point', 'up four points after a competitor closed'])}.`);
    });
    section(`${division.name} Programs and Projects`, () => {
      const variant = rng.fork(`division-programs-${division.key}`);
      flow.heading(`${division.name}: Programs and Projects`, { level: 1, size: 15 });
      const programs = {
        grain: ['Deferred-price contracts', 'Minimum-price contracts', 'Grain bank storage', 'Basis contracts', 'Drying and conditioning'],
        agronomy: ['Variable-rate prescriptions', 'Soil sampling (2.5-acre grids)', 'Custom application', 'Nitrogen stabilizer program', 'Seed treatment'],
        energy: ['Propane prepay contracts', 'Propane budget billing', 'Fleet fueling cards', 'Tank monitoring', 'Lubricant analysis'],
        feed: ['Custom dairy rations', 'Cow-calf mineral program', 'Bulk delivery routes', 'Forage testing', 'Starter feed bookings'],
        dairy: ['Quality premium program', 'Farm sustainability audits', 'Component pricing', 'Hauling assistance', 'New producer onboarding'],
        retail: ['Online order and pickup', 'Ranch account billing', 'Fencing design service', 'Animal health club', 'Small engine repair'],
      }[division.key];
      flow.table([{ header: 'Program', width: 0.4 }, { header: 'Participants', width: 0.2, align: 'right' }, { header: 'Volume or acres', width: 0.2, align: 'right' }, { header: 'Change', width: 0.2, align: 'right' }], programs.map((program) => [program, grouped(variant.int(30, 1400)), grouped(variant.int(1000, 240000)), `${variant.chance(0.7) ? '+' : '-'}${variant.int(1, 34)}%`]), { size: 9, border: 'rules' });
      flow.paragraph('Capital projects completed or under way:', { face: 'sans-bold', size: 10 });
      flow.table([{ header: 'Project', width: 0.44 }, { header: 'Location', width: 0.24 }, { header: 'Cost ($000)', width: 0.16, align: 'right' }, { header: 'Status', width: 0.16 }], [0, 1, 2, 3].map(() => [variant.pick(['Scale house replacement', 'Conveyor and leg upgrade', 'Tank farm containment', 'Pellet die replacement', 'Delivery truck fleet (4 units)', 'Bulk bin expansion', 'Cold storage addition', 'Office remodel', 'Fire suppression upgrade', 'Rail track repair', 'Software implementation', 'Boiler replacement']), `${variant.pick(locations).town}`, k(variant.int(90, 3800)), variant.pick(['Completed', 'In progress', 'Approved', 'Completed under budget'])]), { size: 9, border: 'rules' });
      flow.paragraph(`During the year the division ${variant.pick(['added a second shift in the busiest season', 'trained every driver in defensive winter driving', 'reorganized its sales team by geography', 'moved dispatch to a single center at Tamsin Valley', 'signed three new supply agreements with regional manufacturers'])}, and ${variant.pick(['member satisfaction scores rose to 87 from 81', 'on-time delivery reached 96%', 'average response time to service calls fell to 3.1 hours', 'inventory write-downs fell by half', 'the number of active accounts grew by 6%'])}.`);
    });
  });

  group = 5;
  locations.forEach((location, index) => {
    section(`Location: ${location.town}`, () => {
      const variant = rng.fork(`location-${index}`);
      flow.heading(`${location.town}, ${location.state}`, { level: 1, size: 15 });
      flow.paragraph(location.role, { face: 'sans', size: 10, grey: 0.3 });
      flow.table([{ header: 'Facility profile', width: 0.45 }, { header: '', width: 0.55 }], [
        ['Location manager', location.manager],
        ['Site area', `${location.acres} acres`],
        ['Employees (full-time equivalent)', String(location.employees)],
        ['Storage capacity', location.storage ? `${grouped(location.storage * 1000)} bushels` : 'None (no grain storage)'],
        ['Original facility built', String(location.built)],
        ['Fiscal 2026 throughput', `${grouped(location.volume)} units of primary product`],
        ['Days since last lost-time injury', grouped(location.safetyDays)],
        ['OSHA recordable injuries in fiscal 2026', String(location.recordables)],
        ['Capital invested in fiscal 2026', `$${k(location.capex)} thousand`],
      ], { size: 9 });
      flow.paragraph(`The year at ${location.town} was marked by ${location.note}. The largest capital project was ${location.project}, completed ${variant.pick(['ahead of schedule', 'on budget', 'three weeks late after a steel delivery delay', 'in time for harvest', 'during the spring shutdown'])}. ${location.note.includes('new manager') ? `${location.manager} took over the location in the summer of 2025` : `${location.manager} ${variant.pick(['joined the cooperative in', 'has managed the site since', 'was promoted to manager in', 'took over the location in'])} ${variant.int(2005, 2024)}`}.`);
      const weights = [1, 2, 3, 4].map(() => variant.int(18, 32));
      const quarterly = weights.map((weight) => Math.round((location.volume * weight) / weights.reduce((a, b) => a + b, 0)));
      quarterly[3] = location.volume - quarterly[0] - quarterly[1] - quarterly[2];
      const price = variant.int(40, 260);
      const marginRate = variant.int(6, 22);
      flow.table([{ header: 'Quarter', width: 0.25 }, { header: 'Throughput', width: 0.25, align: 'right' }, { header: 'Revenue ($000)', width: 0.25, align: 'right' }, { header: 'Margin ($000)', width: 0.25, align: 'right' }], quarterly.map((volume, quarter) => [`Q${quarter + 1} (${['Jul-Sep', 'Oct-Dec', 'Jan-Mar', 'Apr-Jun'][quarter]} ${quarter < 2 ? 2025 : 2026})`, grouped(volume), k((volume * price * variant.int(90, 110)) / 10000), k((volume * price * marginRate * variant.int(85, 115)) / 1000000)]), { size: 9 });
      flow.paragraph(variant.pick([
        `Members within 25 miles of ${location.town}: ${grouped(variant.int(140, 980))}. Share of local market (estimated): ${variant.int(18, 64)}%.`,
        `Trucks received or loaded per day at peak: ${variant.int(40, 420)}. Average wait at peak: ${variant.int(4, 55)} minutes.`,
        `Energy use per unit fell ${variant.int(2, 19)}% after equipment upgrades; water use was ${grouped(variant.int(400, 9200))} thousand gallons.`,
      ]), { size: 10 });
      flow.paragraph(`Community: employees volunteered ${grouped(variant.int(120, 1400))} hours, and the location sponsored ${variant.pick(['the county 4-H livestock sale', 'a high school FFA chapter', 'the volunteer fire department\'s grain bin rescue training', 'a farmers market', 'the county fair'])}.`, { size: 10 });
      flow.table([{ header: 'Five-year record', width: 0.3 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.14, align: 'right' }))], [
        ['Throughput', ...location.trend.map(grouped)],
        ['Employees at year end', ...location.staff.map(String)],
        ['Capital invested ($000)', ...years.map((_, year) => k(year === 4 ? location.capex : variant.int(40, 5200)))],
      ], { size: 8.5, border: 'rules' });
      const change = Math.round(((location.trend[4] - location.trend[3]) * 1000) / location.trend[3]);
      flow.paragraph(`Throughput ${change >= 0 ? 'rose' : 'fell'} ${decimal(Math.abs(change), 1)}% from fiscal 2025. ${variant.pick([
        `The location expects to handle about ${grouped(Math.round(location.trend[4] * (0.95 + variant.float() * 0.12)))} units in fiscal 2027`,
        `Its fiscal 2027 budget assumes ${variant.int(1, 6)}% more volume and ${variant.int(1, 4)} fewer overtime hours per employee per month`,
        `Management will review whether to add ${variant.pick(['a second shift', 'weekend hours', 'a seasonal crew', 'a satellite site'])} in fiscal 2027`,
      ])}, and ${variant.pick(['two employees completed the cooperative\'s supervisor training', 'the site will host the regional safety day in May', 'a new member advisory group meets quarterly', 'the scale house software will be replaced', 'a retiring manager will be succeeded from within'])}.`, { size: 10 });
    });
    section(`Location detail: ${location.town}`, () => {
      const variant = rng.fork(`location-detail-${index}`);
      flow.heading(`${location.town}: Operations Detail`, { level: 2, size: 13 });
      const monthly = MONTHS.map(() => variant.int(Math.round(location.volume / 20), Math.round(location.volume / 8)));
      flow.table([{ header: 'Monthly throughput', width: 0.16 }, ...MONTHS.slice(0, 6).map((month) => ({ header: month, width: 0.14, align: 'right' }))], [['Jul - Dec', ...monthly.slice(0, 6).map(grouped)], ['Jan - Jun', ...monthly.slice(6).map(grouped)]], { size: 8.5, border: 'rules' });
      flow.table([{ header: 'Staff by role', width: 0.5 }, { header: 'Full-time', width: 0.25, align: 'right' }, { header: 'Seasonal', width: 0.25, align: 'right' }], ['Operations', 'Drivers', 'Sales and agronomy', 'Maintenance', 'Office'].map((role) => [role, String(variant.int(1, 30)), String(variant.int(0, 18))]), { size: 8.5, border: 'rules' });
      const kit = [
        [/grain|elevator|shuttle|terminal/i, ['Truck scale, 120 ft', 'Grain dryer, 3,000 bu/hr', 'Bucket elevator leg', 'Rail car mover']],
        [/agronomy/i, ['Fertilizer blender', 'Floater applicator', 'Anhydrous nurse tanks (40)', 'Chemical induction system']],
        [/feed/i, ['Pellet mill, 300 hp', 'Batch mixer, 4 ton', 'Bulk feed truck', 'Hammer mill']],
        [/energy|propane|cardlock/i, ['Propane transport', 'Fuel dispensers (4)', 'Bobtail delivery truck', 'Storage tank, 30,000 gal']],
        [/dairy|cheese/i, ['Cheese vat, 50,000 lb', 'HTST pasteurizer', 'Whey evaporator', 'Clean-in-place system']],
        [/retail|animal health/i, ['Forklift', 'Pharmacy cold room', 'Delivery van', 'Point-of-sale system']],
      ].filter(([pattern]) => pattern.test(location.role)).flatMap(([, items]) => items);
      flow.table([{ header: 'Major equipment', width: 0.5 }, { header: 'Year', width: 0.2, align: 'right' }, { header: 'Condition', width: 0.3 }], variant.sample(kit, Math.min(4, kit.length)).map((item) => [item, String(variant.int(1978, 2025)), variant.pick(['Good', 'Fair', 'Scheduled for replacement', 'New', 'Rebuilt this year'])]), { size: 8.5, border: 'rules' });
      flow.table([{ header: 'Product line', width: 0.36 }, { header: 'Division', width: 0.28 }, { header: 'Volume', width: 0.18, align: 'right' }, { header: 'vs FY2025', width: 0.18, align: 'right' }], location.divisions.flatMap((division) => division.products.slice(0, location.divisions.length > 1 ? 2 : 4).map((product) => [product, division.name, grouped(variant.int(Math.round(division.base / 20), division.base)), `${variant.chance(0.65) ? '+' : '-'}${decimal(variant.int(4, 230), 1)}%`])), { size: 8.5, border: 'rules' });
      flow.table([{ header: 'Fiscal 2027 capital plan', width: 0.5 }, { header: 'Budget ($000)', width: 0.25, align: 'right' }, { header: 'Timing', width: 0.25 }], variant.sample(['Roof replacement', 'Truck scale recertification and deck', 'Loader replacement', 'Office HVAC', 'Paving and drainage', 'Security cameras and gates', 'Spill containment curbing', 'Delivery truck', 'Bin sweep augers', 'Backup generator'], 3).map((project) => [project, k(variant.int(30, 900)), variant.pick(['Summer 2026', 'Fall 2026', 'Winter 2026-27', 'Spring 2027'])]), { size: 8.5, border: 'rules' });
      flow.paragraph(`Customers served: ${grouped(variant.int(90, 1900))} accounts, of which ${variant.int(55, 94)}% are members. The largest ten accounts provided ${variant.int(18, 52)}% of the location's margin. Average days sales outstanding were ${variant.int(18, 47)}. The location's budget for fiscal 2027 calls for ${variant.pick(['flat volume and a 2% expense reduction', 'a 5% volume increase', 'the replacement of its oldest delivery truck', 'two additional seasonal employees in the fall', 'completion of the safety railing project'])}.`, { size: 10 });
    });
  });
  const countyOrder = rng.fork('county-traits').shuffle(COUNTIES.map((_, index) => index));
  members.forEach((member, index) => {
    section(`Members in ${member.county} County`, () => {
      const variant = rng.fork(`county-${index}`);
      flow.heading(`${member.county} County`, { level: 1, size: 15 });
      flow.table([{ header: 'Membership profile', width: 0.6 }, { header: '', width: 0.4, align: 'right' }], [
        ['Active members', grouped(member.active)],
        ['New members in fiscal 2026', String(member.new)],
        ['Allocated equity ($000)', k(member.equity)],
        ['Average patronage per active member', `$${grouped(variant.int(900, 6400))}`],
        ['Members age 70 and over', `${variant.int(9, 24)}%`],
        ['Director', index < 9 ? directors[index] : 'At-large seat (vacant)'],
      ], { size: 9 });
      const crops = variant.sample(['Corn', 'Wheat', 'Sorghum', 'Alfalfa', 'Cotton', 'Chile peppers', 'Pecans', 'Onions', 'Pasture and rangeland', 'Soybeans', 'Potatoes', 'Barley'], 5);
      const acres = crops.map(() => variant.int(4, 160));
      flow.table([{ header: 'Crop mix of member acres', width: 0.5 }, { header: 'Thousand acres', width: 0.25, align: 'right' }, { header: 'Share', width: 0.25, align: 'right' }], crops.map((crop, cropIndex) => [crop, String(acres[cropIndex]), percent(Math.round((acres[cropIndex] * 10000) / acres.reduce((a, b) => a + b, 0)), 1)]), { size: 9, border: 'rules' });
      flow.table([{ header: 'Patronage allocated ($000)', width: 0.35 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.13, align: 'right' }))], [[member.county, ...years.map(() => k(variant.int(200, 3600)))]], { size: 9, border: 'rules' });
      flow.table([{ header: 'Cooperative programs used', width: 0.46 }, { header: 'Members enrolled', width: 0.27, align: 'right' }, { header: 'Volume', width: 0.27, align: 'right' }], variant.sample([
        ['Propane prepay and budget billing', 'gallons'], ['Deferred-price grain contracts', 'bushels'], ['Cow-calf mineral program', 'tons'], ['Ranch account billing', 'dollars'],
        ['Grain bank storage', 'bushels'], ['Fleet fueling cards', 'gallons'], ['Custom dairy rations', 'tons'], ['Tank monitoring', 'tanks'], ['Seed treatment', 'units'],
      ], 4).map(([program, unit]) => [program, grouped(Math.max(3, Math.round(member.active * variant.int(4, 38) / 100))), `${grouped(variant.int(2, 900) * (unit === 'tanks' ? 1 : 100))} ${unit}`]), { size: 9, border: 'rules' });
      flow.paragraph(`Equity retired to ${member.county} County members during the year totaled $${k(variant.int(60, 640))} thousand, including $${k(variant.int(10, 180))} thousand paid to the estates of ${variant.int(2, 11)} members. ${variant.int(18, 71)} members attended the county's winter meeting, and ${variant.int(3, 19)} serve on cooperative advisory committees.`);
      flow.paragraph(`${member.county} County members ${COUNTY_TRAITS[countyOrder[index]][0]}. ${COUNTY_TRAITS[countyOrder[index]][1]}`);
    });
  });

  group = 3;
  section('The Year in Review', () => {
    flow.heading('The Year in Review', { level: 1, size: 16 });
    const events = [
      ['July', 'Wheat harvest began two weeks early; the Tamsin Valley terminal received a single-day record of 412,000 bushels.'],
      ['August', 'The board approved the Oriel Junction shuttle loader expansion and a $24.6 million capital plan.'],
      ['September', 'Fall fertilizer bookings opened at prices 18% below the prior year; 71% of expected tons were booked by month end.'],
      ['October', 'A belt failure on the main leg at Copper Flats stopped receiving for 14 days during corn harvest.'],
      ['November', 'Seasonal borrowings peaked; the energy division began winter fill of member propane tanks.'],
      ['December', 'Cash patronage of $7.9 million for fiscal 2025 was paid to members.'],
      ['January', 'The coldest week in eleven years brought 2,840 propane deliveries in nine days.'],
      ['February', 'The Silverlode pellet mill rebuild was completed during a planned two-week shutdown.'],
      ['March', 'Members in ten counties attended spring meetings; the Young Producers Council toured the dairy plant.'],
      ['April', 'A hailstorm damaged roofs at two locations; the cooperative donated feed and fuel to affected members.'],
      ['May', 'Custom application crews covered 41,000 acres in the last two weeks of the month.'],
      ['June', 'The fiscal year closed with grain inventories hedged within board limits and no covenant exceptions.'],
    ];
    flow.table([{ header: 'Month', width: 0.15 }, { header: 'What happened', width: 0.85 }], events, { size: 9.5, border: 'rules' });
    flow.paragraph('None of the events above changed the cooperative\'s strategy, but several will shape next year\'s capital plan: the leg failure at Copper Flats accelerated a replacement already planned for fiscal 2028, and the hail damage prompted a review of roof condition at every location.');
  });
  section('Member Programs', () => {
    flow.heading('Member Programs', { level: 1, size: 16 });
    const variant = rng.fork('member-programs');
    flow.table([{ header: 'Program', width: 0.36 }, { header: 'Members enrolled', width: 0.2, align: 'right' }, { header: 'Benefit to members', width: 0.44 }], [
      ['Input financing (seasonal)', grouped(variant.int(300, 520)), 'Prime + 1.25% until December 1'],
      ['Grain marketing education', grouped(variant.int(80, 240)), 'Monthly workshops on hedging and contracts'],
      ['Young Producers Council', '118', 'Leadership training, plant tours, board observer seat'],
      ['Scholarship program', String(variant.int(24, 61)), '$1,500 per student for agriculture-related study'],
      ['Equity retirement for age 70', grouped(variant.int(140, 320)), 'Early retirement of allocated equity'],
      ['Farm safety training', grouped(variant.int(200, 700)), 'Grain bin, anhydrous, and tractor safety sessions'],
      ['Drought assistance', grouped(variant.int(60, 190)), 'Feed price discount and deferred payment terms'],
    ], { size: 9, border: 'rules' });
    flow.paragraph(`Member surveys: ${grouped(variant.int(900, 2100))} members responded to this year's survey. ${variant.int(78, 91)}% said they were satisfied or very satisfied with the cooperative, ${variant.int(60, 80)}% said they would recommend membership to a neighbor, and the most requested improvement was longer receiving hours during harvest, which we will pilot at three elevators this fall.`);
  });
  section('Our People', () => {
    flow.heading('Our People', { level: 1, size: 16 });
    const staff = rng.fork('people');
    flow.table([{ header: 'Division', width: 0.3 }, { header: 'Full-time', width: 0.14, align: 'right' }, { header: 'Seasonal', width: 0.14, align: 'right' }, { header: 'Turnover', width: 0.14, align: 'right' }, { header: 'Training hours', width: 0.14, align: 'right' }, { header: 'Avg. tenure', width: 0.14, align: 'right' }], divisions.map((division) => [division.name, String(division.employees), String(staff.int(5, 90)), percent(staff.int(600, 2400), 1), grouped(staff.int(800, 9000)), `${decimal(staff.int(40, 160), 1)} yrs`]), { size: 9, border: 'rules' });
    flow.paragraph(`Wages and benefits totaled $${k(staff.int(61000, 82000))} thousand. The general wage increase was 4%, and commercial drivers received an additional market adjustment of $1.25 an hour. The cooperative pays 85% of employee health premiums and 70% of family premiums, and contributes up to 5% of pay to the 401(k) plan. ${staff.int(31, 58)} employees completed the supervisor development program, and ${staff.int(12, 30)} interns worked at locations and the dairy plant last summer, ${staff.int(4, 11)} of whom accepted full-time offers.`);
    flow.paragraph(`Safety training: every employee who enters grain bins completed hands-on rescue training; ${staff.int(80, 140)} drivers completed winter driving refreshers; and the cooperative's safety committee conducted ${staff.int(40, 90)} location audits, closing ${staff.int(85, 99)}% of findings within 30 days.`);
  });
  section('Quarterly Results', () => {
    flow.heading('Quarterly Results (unaudited)', { level: 1, size: 16 });
    const quarters = rng.fork('quarters');
    const shares = [0.31, 0.24, 0.18, 0.27];
    flow.table([{ header: 'In thousands of dollars', width: 0.36 }, { header: 'Q1 (Jul-Sep)', width: 0.16, align: 'right' }, { header: 'Q2 (Oct-Dec)', width: 0.16, align: 'right' }, { header: 'Q3 (Jan-Mar)', width: 0.16, align: 'right' }, { header: 'Q4 (Apr-Jun)', width: 0.16, align: 'right' }], [
      ['Revenue', ...shares.map((share) => k(totalRevenue[4] * share))],
      ['Gross margin', ...shares.map((share) => k(totalMargin[4] * (share + quarters.int(-20, 20) / 1000)))],
      ['Operating expenses', ...shares.map(() => k(operatingExpense[4] / 4 * (0.9 + quarters.float() * 0.2)))],
      ['Local savings', ...shares.map((share) => k(localSavings[4] * (share + quarters.int(-40, 40) / 1000)))],
      ['Seasonal borrowings at quarter end', ...shares.map(() => k(quarters.int(20000, 110000)))],
    ], { size: 9, border: 'rules' });
    flow.paragraph('The first and second quarters carry most of the year\'s grain and fall fertilizer volume; the third quarter is the energy division\'s busiest; and the fourth quarter includes spring agronomy. Quarterly results are not audited and may not sum to the annual totals because of rounding and year-end adjustments.');
  });
  section('Sensitivity Analysis', () => {
    flow.heading('Sensitivity of Local Savings', { level: 1, size: 16 });
    const sensitivity = rng.fork('sensitivity');
    flow.paragraph('The following estimates show how fiscal 2026 local savings would have changed if one factor had been different and everything else had stayed the same. They are illustrations, not forecasts.');
    flow.table([{ header: 'Change in', width: 0.56 }, { header: 'Effect on local savings ($000)', width: 0.44, align: 'right' }], [
      ['Grain volume handled, 10% higher', `+${k(sensitivity.int(1200, 2600))}`],
      ['Corn basis, 5 cents per bushel stronger', `+${k(sensitivity.int(600, 1400))}`],
      ['Fertilizer margin, $10 per ton lower', `-${k(sensitivity.int(400, 1100))}`],
      ['Propane gallons, 15% lower (warm winter)', `-${k(sensitivity.int(700, 1800))}`],
      ['Class III milk price, $1 per cwt higher', `+${k(sensitivity.int(300, 900))}`],
      ['Interest rates, 1 percentage point higher', `-${k(sensitivity.int(500, 1300))}`],
      ['Wages, 1% higher across all divisions', `-${k(sensitivity.int(500, 900))}`],
      ['Bad debt expense doubled', `-${k(sensitivity.int(300, 800))}`],
    ], { size: 9.5, border: 'rules' });
    flow.paragraph('Grain volume and basis remain the factors with the greatest effect, which is why the board continues to invest in handling capacity and rail access rather than in new lines of business.');
  });
  const risks = riskFactors();
  section('Risk Factors', () => {
    flow.heading('Risk Factors', { level: 1, size: 16 });
    flow.paragraph('The following risks could materially affect the cooperative\'s results, financial condition, or ability to return equity to members. They are not the only risks we face.');
    risks.forEach(([title, body]) => flow.paragraph([{ text: `${title}. `, face: 'sans-bold' }, { text: body }]));
  });

  section('Management\'s Discussion and Analysis', () => {
    flow.heading('Management\'s Discussion and Analysis', { level: 1, size: 16 });
    flow.paragraph(`Revenue for fiscal 2026 was $${k(totalRevenue[4])} thousand compared with $${k(totalRevenue[3])} thousand in fiscal 2025. Changes in commodity prices explain most of the movement in revenue from year to year; gross margin, which is revenue less the cost of goods sold, is the better measure of how much business members did with the cooperative and how profitably it was handled.`);
    flow.paragraph(`Gross margin was $${k(totalMargin[4])} thousand. Operating expenses were $${k(operatingExpense[4])} thousand, or ${percent(Math.round((operatingExpense[4] * 10000) / totalMargin[4]), 1)} of gross margin. Wages and benefits rose 5.8% with a 4% general increase and higher health plan costs; repairs rose with the age of our elevators; and insurance premiums rose 11% after a hard market renewal.`);
    flow.paragraph(`Liquidity: the cooperative ended the year with working capital of $${k(data.int(52000, 88000))} thousand and $${k(data.int(40000, 70000))} thousand available under its seasonal revolving credit facility with Kingsfold Farm Credit Bank. Seasonal borrowings peaked in November at $${k(data.int(60000, 110000))} thousand, when grain and fertilizer inventories are highest. The cooperative was in compliance with all loan covenants at year end.`);
    flow.paragraph('Capital resources: the board approved a capital budget of $24.6 million for fiscal 2027, to be funded from operating cash flow and a term loan advance for the Oriel Junction shuttle loader. Equity retirement is planned at $4.0 million.');
    flow.paragraph('Critical accounting estimates: inventories of grain and grain-related contracts are carried at net realizable value; the allowance for doubtful accounts reflects specific reserves on 23 accounts and a general reserve based on aging; pension obligations depend on a discount rate that rose to 5.4% from 5.1%.');
  });

  section('Corporate Governance', () => {
    flow.heading('Corporate Governance', { level: 1, size: 16 });
    const governance = rng.fork('committees');
    flow.paragraph('The board sets policy, approves the budget and capital plan, hires and evaluates the chief executive, and decides each year how much of the cooperative\'s earnings to return to members in cash. Directors are elected by members of their county for three-year terms and may serve four terms.');
    flow.table([{ header: 'Director', width: 0.32 }, { header: 'Board meetings attended', width: 0.24, align: 'right' }, { header: 'Committee meetings attended', width: 0.24, align: 'right' }, { header: 'Committees', width: 0.2 }], directors.map((name) => [name, `${governance.int(9, 11)} of 11`, `${governance.int(3, 6)} of ${governance.int(6, 7)}`, governance.pick(['Audit', 'Governance', 'Member Relations', 'Audit, Governance', 'Capital Planning'])]), { size: 9, border: 'rules' });
    flow.paragraph(`The board completed its biennial self-evaluation in February, adopted a revised conflict-of-interest policy requiring directors to disclose any business with competitors, and held a joint session with the Young Producers Council. Director compensation totaled $${k(governance.int(90, 160))} thousand.`);
  });
  section('Report of the Audit Committee', () => {
    flow.heading('Report of the Audit Committee', { level: 1, size: 16 });
    flow.paragraph(`The Audit Committee consists of three directors, none of whom is an employee. It met six times during fiscal 2026, including two sessions with the independent auditors without management present. The committee reviewed the audited financial statements with management and with ${AUDITOR}, discussed the matters the auditors are required to communicate, including the valuation of grain inventories and derivatives and the pension obligation, and received the auditors' written statement of independence.`);
    flow.paragraph('The committee also reviewed the internal audit plan and the results of the eleven internal audits completed during the year, which covered grain settlements, propane billing, seasonal input financing, payroll, and information security. Management resolved all findings rated high within 60 days. Based on these reviews, the committee recommended to the board that the audited financial statements be included in this annual report, and the board approved the recommendation.');
    flow.paragraph(`Members of the Audit Committee: ${directors[3]} (chair), ${directors[5]}, and ${directors[7]}.`, { face: 'sans-bold', size: 10 });
  });
  section('Community Investment', () => {
    flow.heading('Community Investment', { level: 1, size: 16 });
    const giving = rng.fork('giving');
    flow.table([{ header: 'Recipient', width: 0.46 }, { header: 'Purpose', width: 0.36 }, { header: 'Amount', width: 0.18, align: 'right' }], [
      ['Tamsin Valley Volunteer Fire Department', 'Grain bin rescue tube and training', `$${grouped(giving.int(4, 12) * 1000)}`],
      ['County 4-H and FFA livestock sales (10 counties)', 'Premiums and buyer support', `$${grouped(giving.int(40, 90) * 1000)}`],
      ['Rural health clinic, Copper Flats', 'Mobile screening van fuel', `$${grouped(giving.int(5, 15) * 1000)}`],
      ['Agricultural scholarships', `${giving.int(24, 61)} students`, `$${grouped(giving.int(36, 91) * 1000)}`],
      ['Food banks in the trade area', 'Milk and cheese donations (retail value)', `$${grouped(giving.int(60, 140) * 1000)}`],
      ['Hailstorm relief (April)', 'Feed and fuel for affected members', `$${grouped(giving.int(20, 70) * 1000)}`],
      ['Rural broadband coalition', 'Matching grant for tower study', `$${grouped(giving.int(10, 30) * 1000)}`],
    ], { size: 9, border: 'rules' });
    flow.paragraph(`Employees volunteered ${grouped(giving.int(4000, 9000))} hours on cooperative time, and the cooperative matched ${grouped(giving.int(120, 260))} employee gifts to local charities.`);
  });
  group = 4;
  section('Report of Independent Auditors', () => {
    flow.pageBreak();
    const bodySize = flow.fontSize;
    flow.fontSize = 9.5;
    flow.heading('Report of Independent Auditors', { level: 1, size: 16 });
    flow.paragraph(`To the Board of Directors and Members of ${COOP}`, { face: 'sans-bold', size: 10.5 });
    flow.paragraph('Opinion', { face: 'sans-bold', size: 10.5, after: 2 });
    flow.paragraph(`We have audited the consolidated financial statements of ${COOP} and its subsidiaries (the Cooperative), which comprise the consolidated balance sheets as of ${longDate(yearEnd)} and ${longDate(priorYearEnd)}, and the related consolidated statements of operations, comprehensive income, changes in members' equity, and cash flows for the years then ended, and the related notes to the consolidated financial statements. In our opinion, the accompanying consolidated financial statements present fairly, in all material respects, the financial position of the Cooperative as of ${longDate(yearEnd)} and ${longDate(priorYearEnd)}, and the results of its operations and its cash flows for the years then ended in accordance with accounting principles generally accepted in the United States of America.`);
    flow.paragraph('Basis for Opinion', { face: 'sans-bold', size: 10.5, after: 2 });
    flow.paragraph('We conducted our audits in accordance with auditing standards generally accepted in the United States of America. Our responsibilities under those standards are further described in the Auditor\'s Responsibilities section of our report. We are required to be independent of the Cooperative and to meet our other ethical responsibilities in accordance with the relevant ethical requirements relating to our audits. We believe that the audit evidence we have obtained is sufficient and appropriate to provide a basis for our audit opinion.');
    flow.paragraph('Responsibilities of Management for the Financial Statements', { face: 'sans-bold', size: 10.5, after: 2 });
    flow.paragraph('Management is responsible for the preparation and fair presentation of the consolidated financial statements in accordance with accounting principles generally accepted in the United States of America, and for the design, implementation, and maintenance of internal control relevant to the preparation and fair presentation of financial statements that are free from material misstatement, whether due to fraud or error. Management is also required to evaluate whether there are conditions or events that raise substantial doubt about the Cooperative\'s ability to continue as a going concern within one year after the date that the financial statements are available to be issued.');
    flow.paragraph('Auditor\'s Responsibilities for the Audit of the Financial Statements', { face: 'sans-bold', size: 10.5, after: 2 });
    flow.paragraph('Our objectives are to obtain reasonable assurance about whether the consolidated financial statements as a whole are free from material misstatement, whether due to fraud or error, and to issue an auditor\'s report that includes our opinion. Reasonable assurance is a high level of assurance but is not absolute assurance. In performing an audit, we exercise professional judgment, identify and assess the risks of material misstatement, obtain an understanding of internal control relevant to the audit, evaluate the appropriateness of accounting policies used and the reasonableness of significant accounting estimates, and conclude whether there are conditions or events that raise substantial doubt about the Cooperative\'s ability to continue as a going concern for a reasonable period of time.');
    flow.paragraph('Supplementary Information', { face: 'sans-bold', size: 10.5, after: 2 });
    flow.paragraph('Our audits were conducted for the purpose of forming an opinion on the consolidated financial statements as a whole. The consolidating information and the division and location schedules that follow the notes are presented for purposes of additional analysis and are not a required part of the financial statements. Such information has been subjected to the auditing procedures applied in the audit of the financial statements and, in our opinion, is fairly stated in all material respects in relation to the consolidated financial statements as a whole.');
    flow.paragraph(`/s/ ${AUDITOR}`, { before: 6, after: 0 });
    flow.paragraph('Kestrel Ridge, Utah', { after: 0 });
    flow.paragraph(longDate(reportDate), { after: 8 });
    flow.fontSize = bodySize;
  });

  section('Consolidated Financial Statements', () => {
    flow.pageBreak();
    flow.heading('Consolidated Balance Sheets', { level: 1, size: 15 });
    flow.paragraph(`As of ${longDate(yearEnd)} and ${longDate(priorYearEnd)} (in thousands of dollars)`, { size: 9, grey: 0.3 });
    const sheet = rng.fork('balance');
    const assetLines = ['Cash and cash equivalents', 'Receivables, net of allowance', 'Inventories - grain', 'Inventories - supplies and finished goods', 'Derivative assets', 'Margin deposits', 'Prepaid expenses', 'Investments in other cooperatives', 'Property, plant and equipment, net', 'Operating lease right-of-use assets', 'Other assets'];
    const liabilityLines = ['Seasonal notes payable', 'Current maturities of long-term debt', 'Accounts payable', 'Customer credit balances and prepayments', 'Derivative liabilities', 'Accrued expenses', 'Patronage payable in cash', 'Long-term debt, less current maturities', 'Operating lease liabilities', 'Pension and other postretirement obligations', 'Deferred income taxes'];
    const equityLines = ['Common stock', 'Allocated equities', 'Unallocated retained earnings', 'Accumulated other comprehensive loss'];
    const column = (lines, low, high) => lines.map((line) => [line, sheet.int(low, high), sheet.int(low, high)]);
    const assetRows = column(assetLines, 2400, 96000);
    const liabilityRows = column(liabilityLines, 1100, 52000);
    const equityRows = column(equityLines, 900, 120000);
    const sum = (rows, index) => rows.reduce((total, row) => total + row[index], 0);
    const tableRows = (rows) => rows.map(([label, current, prior]) => [label, k(current), k(prior)]);
    flow.table([{ header: 'Assets', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], [...tableRows(assetRows), [{ text: 'Total assets', face: 'sans-bold' }, k(sum(assetRows, 1)), k(sum(assetRows, 2))]], { size: 9, border: 'rules' });
    flow.table([{ header: 'Liabilities and members\' equity', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], [...tableRows(liabilityRows), [{ text: 'Total liabilities', face: 'sans-bold' }, k(sum(liabilityRows, 1)), k(sum(liabilityRows, 2))], ...tableRows(equityRows), [{ text: 'Total members\' equity', face: 'sans-bold' }, k(sum(equityRows, 1)), k(sum(equityRows, 2))]], { size: 9, border: 'rules' });
    flow.heading('Consolidated Statements of Operations', { level: 1, size: 15 });
    flow.paragraph(`Years ended ${longDate(yearEnd)} and ${longDate(priorYearEnd)} (in thousands of dollars)`, { size: 9, grey: 0.3 });
    flow.table([{ header: '', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], [
      ['Revenue', k(totalRevenue[4]), k(totalRevenue[3])],
      ['Cost of goods sold', k(totalRevenue[4] - totalMargin[4]), k(totalRevenue[3] - totalMargin[3])],
      ['Gross margin', k(totalMargin[4]), k(totalMargin[3])],
      ['Service and other income', k(sheet.int(3000, 9000)), k(sheet.int(3000, 9000))],
      ['Operating expenses', k(operatingExpense[4]), k(operatingExpense[3])],
      ['Interest expense', k(sheet.int(2400, 6800)), k(sheet.int(2400, 6800))],
      ['Patronage from other cooperatives', k(sheet.int(900, 4200)), k(sheet.int(900, 4200))],
      ['Local savings before income taxes', k(localSavings[4]), k(localSavings[3])],
      ['Income tax expense', k(sheet.int(200, 1400)), k(sheet.int(200, 1400))],
      ['Net savings', k(localSavings[4] - 700), k(localSavings[3] - 650)],
    ], { size: 9, border: 'rules' });
    flow.heading('Consolidated Statements of Cash Flows (summary)', { level: 1, size: 15 });
    flow.table([{ header: '', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], [
      ['Net cash provided by operating activities', k(sheet.int(18000, 42000)), k(sheet.int(18000, 42000))],
      ['Capital expenditures', `(${k(sheet.int(9000, 26000))})`, `(${k(sheet.int(9000, 26000))})`],
      ['Proceeds from sale of property', k(sheet.int(100, 2000)), k(sheet.int(100, 2000))],
      ['Net change in seasonal borrowings', k(sheet.int(-8000, 9000)), k(sheet.int(-8000, 9000))],
      ['Payments on long-term debt', `(${k(sheet.int(3000, 9000))})`, `(${k(sheet.int(3000, 9000))})`],
      ['Cash patronage and equity retirements paid', `(${k(sheet.int(5000, 12000))})`, `(${k(sheet.int(5000, 12000))})`],
      ['Net change in cash', k(sheet.int(-3000, 6000)), k(sheet.int(-3000, 6000))],
    ], { size: 9, border: 'rules' });
  });

  const notes = financialNotes(rng.fork('notes'), { yearEnd, reportDate, divisions });
  section('Notes to Consolidated Financial Statements', () => {
    flow.heading('Notes to Consolidated Financial Statements', { level: 1, size: 15 });
    notes.forEach(([title, paragraphs, table], index) => {
      flow.paragraph(`Note ${index + 1} - ${title}`, { face: 'sans-bold', size: 10.5, after: 3, keepWithNext: 40 });
      paragraphs.forEach((text) => flow.paragraph(text));
      if (table) flow.table(table.columns, table.rows, { size: 8.5, border: 'rules' });
    });
  });

  section('Supplementary Schedules', () => {
    flow.pageBreak();
    flow.heading('Supplementary Schedules', { level: 1, size: 15 });
    flow.paragraph('Consolidating information by division, fiscal 2026 (in thousands of dollars).', { size: 9, grey: 0.3 });
    const supplementary = rng.fork('supplementary');
    flow.table([{ header: 'Division', width: 0.28 }, { header: 'Revenue', width: 0.12, align: 'right' }, { header: 'Margin', width: 0.12, align: 'right' }, { header: 'Expenses', width: 0.12, align: 'right' }, { header: 'Savings', width: 0.12, align: 'right' }, { header: 'Assets', width: 0.12, align: 'right' }, { header: 'Capex', width: 0.12, align: 'right' }], divisions.map((division) => {
      const expenses = Math.round(division.margins[4] * (0.55 + supplementary.float() * 0.2));
      return [division.name, k(division.revenue[4]), k(division.margins[4]), k(expenses), k(division.margins[4] - expenses), k(supplementary.int(9000, 88000)), k(division.capex)];
    }), { size: 8.5, border: 'rules' });
    flow.paragraph('Patronage allocation by division (fiscal 2026):', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Division', width: 0.34 }, { header: 'Patronage basis', width: 0.26 }, { header: 'Rate', width: 0.2, align: 'right' }, { header: 'Allocated ($000)', width: 0.2, align: 'right' }], divisions.map((division) => [division.name, { grain: 'per bushel delivered', agronomy: 'per dollar of purchases', energy: 'per gallon purchased', feed: 'per ton purchased', dairy: 'per hundredweight shipped', retail: 'per dollar of purchases' }[division.key], { grain: `$${decimal(supplementary.int(2, 9), 3)}`, agronomy: percent(supplementary.int(150, 600), 1), energy: `$${decimal(supplementary.int(3, 14), 3)}`, feed: `$${decimal(supplementary.int(120, 640), 2)}`, dairy: `$${decimal(supplementary.int(9, 41), 2)}`, retail: percent(supplementary.int(80, 300), 1) }[division.key], k(patronageIncome[4] * division.margins[4] / totalMargin[4])]), { size: 8.5, border: 'rules' });
    flow.paragraph('Consolidating balance sheet by entity, June 30, 2026 (in thousands of dollars):', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Entity', width: 0.34 }, { header: 'Assets', width: 0.16, align: 'right' }, { header: 'Liabilities', width: 0.16, align: 'right' }, { header: 'Equity', width: 0.17, align: 'right' }, { header: 'Savings', width: 0.17, align: 'right' }], [COOP, 'Tamsin Valley Energy LLC', 'TVFC Dairy Products Inc.', 'TVFC Agronomy Services LLC', 'Granary Road Insurance Co.'].map((entity) => {
      const assets = supplementary.int(4000, 160000);
      const liabilities = Math.round(assets * (0.3 + supplementary.float() * 0.3));
      return [entity, k(assets), k(liabilities), k(assets - liabilities), k(supplementary.int(100, 9000))];
    }), { size: 8.5, border: 'rules' });
    flow.paragraph('Capital expenditures by location, five years (in thousands of dollars):', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Location', width: 0.3 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.14, align: 'right' }))], locations.map((location) => [`${location.town}, ${location.state}`, ...years.map(() => k(supplementary.int(40, 6800)))]), { size: 8.5, border: 'rules' });
    flow.paragraph('Patronage by member class, fiscal 2026:', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Class', width: 0.4 }, { header: 'Members', width: 0.2, align: 'right' }, { header: 'Allocated ($000)', width: 0.2, align: 'right' }, { header: 'Cash portion ($000)', width: 0.2, align: 'right' }], [['Individual producers', 'Under $25,000 of business', '$25,000 - $250,000', 'Over $250,000'][0], 'Partnerships and family corporations', 'Under $25,000 of business', '$25,000 - $250,000 of business', 'Over $250,000 of business'].map((label) => {
      const allocated = supplementary.int(400, 9000);
      return [label, grouped(supplementary.int(80, 3000)), k(allocated), k(allocated * 0.4)];
    }), { size: 8.5, border: 'rules' });
    flow.paragraph('Property, plant and equipment by location (in thousands of dollars):', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Location', width: 0.3 }, { header: 'Land', width: 0.14, align: 'right' }, { header: 'Buildings', width: 0.14, align: 'right' }, { header: 'Equipment', width: 0.14, align: 'right' }, { header: 'Accum. depr.', width: 0.14, align: 'right' }, { header: 'Net', width: 0.14, align: 'right' }], locations.map((location) => {
      const land = supplementary.int(80, 2400);
      const buildings = supplementary.int(900, 26000);
      const equipment = supplementary.int(700, 31000);
      const depreciation = Math.round((buildings + equipment) * (0.3 + supplementary.float() * 0.4));
      return [`${location.town}, ${location.state}`, k(land), k(buildings), k(equipment), `(${k(depreciation)})`, k(land + buildings + equipment - depreciation)];
    }), { size: 8.5, border: 'rules' });
  });

  group = 3;
  section('Safety and Stewardship', () => {
    flow.heading('Safety and Stewardship', { level: 1, size: 15 });
    const stewardship = rng.fork('stewardship');
    flow.paragraph('We report safety and environmental measures for every location, because the work our employees do - entering grain bins, handling anhydrous ammonia, delivering propane on icy roads - is among the most hazardous in agriculture.');
    flow.table([{ header: 'Location', width: 0.3 }, { header: 'Hours worked', width: 0.16, align: 'right' }, { header: 'Recordables', width: 0.14, align: 'right' }, { header: 'TRIR', width: 0.12, align: 'right' }, { header: 'Bin entries (permitted)', width: 0.14, align: 'right' }, { header: 'Electricity (MWh)', width: 0.14, align: 'right' }], locations.map((location) => {
      const hours = location.employees * stewardship.int(1850, 2250);
      return [`${location.town}, ${location.state}`, grouped(hours), String(location.recordables), decimal(Math.round((location.recordables * 200000 * 100) / hours), 2), String(stewardship.int(0, 160)), grouped(stewardship.int(80, 9400))];
    }), { size: 8.5, border: 'rules' });
    flow.paragraph(`Total greenhouse gas emissions from our own operations (scopes 1 and 2) were ${grouped(stewardship.int(31000, 52000))} metric tons of carbon dioxide equivalent, ${stewardship.int(2, 9)}% lower than fiscal 2025, mainly from solar arrays at three locations and the dryer burner upgrades at Tamsin Valley and Fallow Creek. Our agronomists wrote nutrient management plans for ${grouped(stewardship.int(140, 380))} thousand acres, and ${stewardship.int(18, 41)}% of nitrogen sold was applied with a stabilizer or in split applications.`);
    flow.paragraph(`Community investment totaled $${k(stewardship.int(380, 910))} thousand, including scholarships for ${stewardship.int(24, 61)} students, grain bin rescue tubes and training for ${stewardship.int(8, 19)} rural fire departments, and donations of feed and fuel after the April hailstorm.`);
  });

  section('Governance', () => {
    flow.heading('Board of Directors', { level: 1, size: 15 });
    const governance = rng.fork('governance');
    flow.table([{ header: 'Director', width: 0.26 }, { header: 'County', width: 0.2 }, { header: 'Operation', width: 0.36 }, { header: 'Term ends', width: 0.18, align: 'right' }], directors.map((name, index) => [name, COUNTIES[index], governance.pick(['Irrigated corn and alfalfa', 'Dryland wheat and sorghum', 'Cow-calf and hay', '600-cow dairy', 'Feedlot and farming', 'Chile peppers and onions', 'Pecans and cotton', 'Seed potatoes', 'Sheep and wool']), `${2026 + (index % 3)}`]), { size: 9 });
    flow.paragraph(`${directors[0]} serves as Chair, ${directors[1]} as Vice Chair, and ${directors[2]} as Secretary-Treasurer. The board met 11 times during the year; its Audit, Governance, and Member Relations committees met 14 times in total. Directors are paid $400 per meeting day plus mileage.`);
    directors.forEach((name, index) => {
      const bio = rng.fork(`bio-${index}`);
      flow.paragraph(`${name} ${bio.pick(['farms', 'ranches', 'operates a dairy', 'raises cattle and hay'])} near ${bio.pick(locations).town} with ${bio.pick(['a sister and two nephews', 'a spouse and son', 'three cousins', 'a business partner', 'a daughter who returned home in 2019'])}. Elected in ${bio.int(2009, 2024)}, ${name.split(' ')[0]} ${bio.pick(['chairs the Audit Committee', 'serves on the Governance Committee', 'chairs Member Relations', 'represents the cooperative on the regional supply cooperative board', 'leads the board\'s capital planning work'])} and ${bio.pick(['studied agricultural economics', 'is a past county Farm Bureau president', 'served on the local school board', 'is a licensed pilot', 'teaches a farm safety course'])}.`, { size: 10 });
    });
    flow.heading('Management', { level: 2 });
    flow.table([{ header: 'Name', width: 0.4 }, { header: 'Title', width: 0.6 }], executives.map((name, index) => [name, ['President and Chief Executive Officer', 'Chief Financial Officer', 'Vice President, Grain and Energy', 'Vice President, Agronomy and Animal Nutrition', 'Vice President, Dairy Processing', 'Vice President, Human Resources and Safety', 'Director of Member Services'][index]]), { size: 9 });
    flow.heading('Annual Meeting', { level: 2 });
    flow.paragraph(`The annual meeting of members will be held on ${longDate(meeting)} at the Tamsin Valley Fairgrounds. Directors will be elected for the Harrow, Kettle, and Los Pinos counties. Cash patronage checks for fiscal 2026 will be mailed on ${longDate(patronageDate)}.`);
  });

  group = 6;
  group = 5;
  section('Grain Receipts by Location', () => {
    flow.heading('Grain Receipts by Location and Crop', { level: 1, size: 16 });
    const receipts = rng.fork('receipts');
    const grainSites = locations.filter((location) => /grain|shuttle|terminal|elevator/i.test(location.role));
    flow.table([{ header: 'Location (thousand bushels)', width: 0.28 }, { header: 'Corn', width: 0.12, align: 'right' }, { header: 'Wheat', width: 0.12, align: 'right' }, { header: 'Sorghum', width: 0.12, align: 'right' }, { header: 'Soybeans', width: 0.12, align: 'right' }, { header: 'Total', width: 0.12, align: 'right' }, { header: 'Shipped by rail', width: 0.12, align: 'right' }], grainSites.map((location) => {
      const crops = [receipts.int(2000, 19000), receipts.int(800, 9000), receipts.int(300, 6000), receipts.int(100, 4000)];
      return [`${location.town}, ${location.state}`, ...crops.map(grouped), grouped(crops.reduce((a, b) => a + b, 0)), percent(receipts.int(2000, 9400), 1)];
    }), { size: 8.5, border: 'rules' });
    flow.paragraph(`Moisture and quality: ${percent(receipts.int(800, 3100), 1)} of corn receipts required drying, and ${percent(receipts.int(100, 900), 1)} of wheat receipts were discounted for test weight. Average corn moisture at delivery was ${decimal(receipts.int(150, 190), 1)}%.`);
    flow.paragraph(`Rail shipments: ${receipts.int(28, 61)} shuttle trains were loaded at Oriel Junction and Tamsin Valley, averaging ${receipts.int(10, 15)} hours per train against the railroad's 15-hour standard; ${receipts.int(1, 6)} trains were loaded late because of weather.`);
  });
  section('Energy Deliveries', () => {
    flow.heading('Energy Deliveries by Route', { level: 1, size: 16 });
    const routes = rng.fork('routes');
    flow.table([{ header: 'Route', width: 0.24 }, { header: 'Base', width: 0.2 }, { header: 'Propane gal (000)', width: 0.14, align: 'right' }, { header: 'Diesel gal (000)', width: 0.14, align: 'right' }, { header: 'Stops', width: 0.14, align: 'right' }, { header: 'Miles (000)', width: 0.14, align: 'right' }], Array.from({ length: 12 }, (_, index) => [`Route ${String(index + 1).padStart(2, '0')}`, routes.pick(locations).town, grouped(routes.int(200, 2400)), grouped(routes.int(100, 3100)), grouped(routes.int(900, 7800)), grouped(routes.int(18, 96))]), { size: 8.5, border: 'rules' });
    flow.paragraph(`Tank monitors were installed on ${grouped(routes.int(900, 2400))} member propane tanks, allowing deliveries to be scheduled by measured level rather than by calendar; runouts fell to ${routes.int(3, 19)} from ${routes.int(20, 44)} the year before.`);
  });
  section('Feed Production', () => {
    flow.heading('Feed Production by Mill', { level: 1, size: 16 });
    const mills = rng.fork('mills');
    flow.table([{ header: 'Mill', width: 0.26 }, { header: 'Tons produced', width: 0.15, align: 'right' }, { header: 'Pelleted', width: 0.14, align: 'right' }, { header: 'Formulas', width: 0.14, align: 'right' }, { header: 'kWh per ton', width: 0.15, align: 'right' }, { header: 'Downtime hrs', width: 0.16, align: 'right' }], ['Silverlode, NV', 'Dunmore Lake, ID'].map((mill) => [mill, grouped(mills.int(40000, 120000)), percent(mills.int(3000, 7800), 1), String(mills.int(140, 420)), decimal(mills.int(90, 240), 1), String(mills.int(40, 300))]), { size: 9, border: 'rules' });
    flow.table([{ header: 'Top ingredients purchased', width: 0.5 }, { header: 'Tons', width: 0.25, align: 'right' }, { header: 'Avg. cost/ton', width: 0.25, align: 'right' }], ['Corn (ground)', 'Soybean meal', 'Distillers grains', 'Wheat midds', 'Alfalfa pellets', 'Molasses', 'Limestone', 'Salt'].map((ingredient) => [ingredient, grouped(mills.int(1200, 40000)), `$${grouped(mills.int(60, 520))}`]), { size: 9, border: 'rules' });
  });
  section('Dairy Plant Production', () => {
    flow.heading('Dairy Plant Production', { level: 1, size: 16 });
    const plant = rng.fork('plant');
    flow.table([{ header: 'Product (thousand lb.)', width: 0.28 }, ...MONTHS.slice(0, 6).map((month) => ({ header: month, width: 0.12, align: 'right' }))], ['Cheese (cheddar)', 'Cheese (Monterey Jack)', 'Butter', 'Whey protein concentrate'].map((product) => [product, ...MONTHS.slice(0, 6).map(() => grouped(plant.int(300, 2600)))]), { size: 8.5, border: 'rules' });
    flow.table([{ header: 'Product (thousand lb.)', width: 0.28 }, ...MONTHS.slice(6).map((month) => ({ header: month, width: 0.12, align: 'right' }))], ['Cheese (cheddar)', 'Cheese (Monterey Jack)', 'Butter', 'Whey protein concentrate'].map((product) => [product, ...MONTHS.slice(6).map(() => grouped(plant.int(300, 2600)))]), { size: 8.5, border: 'rules' });
    flow.paragraph(`Milk received from ${plant.int(60, 140)} member dairies averaged a somatic cell count of ${grouped(plant.int(140, 230))} thousand cells per milliliter. Cheese yield improved to ${decimal(plant.int(1000, 1080), 2)} pounds per hundredweight after the brine system upgrade, and the plant passed its food safety certification audit with a score of ${decimal(plant.int(940, 990), 1)}.`);
  });
  section('Retail Sales by Department', () => {
    flow.heading('Retail Sales by Department', { level: 1, size: 16 });
    const store = rng.fork('store');
    flow.table([{ header: 'Department', width: 0.3 }, { header: 'Tamsin Valley', width: 0.175, align: 'right' }, { header: 'Silverlode', width: 0.175, align: 'right' }, { header: 'Quarry Bend', width: 0.175, align: 'right' }, { header: 'Change', width: 0.175, align: 'right' }], ['Fencing and gates', 'Livestock handling', 'Animal health', 'Feed (bagged)', 'Hardware and tools', 'Workwear and boots', 'Lawn and garden', 'Pet supplies', 'Online pickup orders'].map((department) => [department, k(store.int(120, 2400)), k(store.int(80, 1800)), k(store.int(60, 1500)), `${store.chance(0.7) ? '+' : '-'}${decimal(store.int(5, 190), 1)}%`]), { size: 9, border: 'rules' });
    flow.paragraph(`Sales figures are in thousands of dollars. Gross margin percentage for retail was ${decimal(store.int(240, 330), 1)}%, and shrink was ${decimal(store.int(6, 19), 1)}% of sales.`);
  });
  section('Location Statistics, Five Years', () => {
    flow.heading('Location Statistics, Five Years', { level: 1, size: 16 });
    flow.paragraph('Throughput of each location\'s primary product (thousand units):', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Location', width: 0.3 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.14, align: 'right' }))], locations.map((location) => [`${location.town}, ${location.state}`, ...location.trend.map(grouped)]), { size: 8.5, border: 'rules' });
    flow.paragraph('Employees at year end:', { face: 'sans-bold', size: 10 });
    flow.table([{ header: 'Location', width: 0.3 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.14, align: 'right' }))], locations.map((location) => [`${location.town}, ${location.state}`, ...location.staff.map(String)]), { size: 8.5, border: 'rules' });
    const growth = locations.map((location) => ({ location, change: Math.round(((location.trend[4] - location.trend[0]) * 1000) / location.trend[0]) })).sort((a, b) => b.change - a.change);
    flow.paragraph(`Over five years throughput grew fastest at ${growth[0].location.town} (${decimal(growth[0].change, 1)}%) and ${growth[1].location.town} (${decimal(growth[1].change, 1)}%), and slowest at ${growth[growth.length - 1].location.town} (${decimal(growth[growth.length - 1].change, 1)}%). Headcount moved by no more than a dozen people at any location; seasonal staff are not included.`);
  });
  section('Ten-Year History', () => {
    flow.heading('Ten-Year History', { level: 1, size: 16 });
    const decade = rng.fork('decade');
    const fiscal = ['2017', '2018', '2019', '2020', '2021', '2022', '2023', '2024', '2025', '2026'];
    flow.table([{ header: 'Fiscal year', width: 0.16 }, { header: 'Revenue ($000)', width: 0.16, align: 'right' }, { header: 'Margin ($000)', width: 0.16, align: 'right' }, { header: 'Savings ($000)', width: 0.16, align: 'right' }, { header: 'Cash patronage', width: 0.18, align: 'right' }, { header: 'Members', width: 0.18, align: 'right' }], fiscal.map((year, index) => [year, k(index >= 5 ? totalRevenue[index - 5] : decade.int(310000, 520000)), k(index >= 5 ? totalMargin[index - 5] : decade.int(52000, 81000)), k(index >= 5 ? localSavings[index - 5] : decade.int(6000, 21000)), `${decade.int(20, 40)}%`, grouped(decade.int(7400, 8900))]), { size: 9, border: 'rules' });
    flow.paragraph('Fiscal years 2017 through 2021 are restated for the 2021 merger with the Dunmore Lake feed cooperative. Cash patronage is the percentage of the year\'s patronage allocation paid in cash.');
  });
  section('Equity Retirement Schedule', () => {
    flow.heading('Equity Retirement Schedule', { level: 1, size: 16 });
    const retirement = rng.fork('retirement');
    flow.paragraph('Allocated equity outstanding by year of allocation, and the board\'s current plan for retiring it. Retirements of estates and of members age 70 or older are paid ahead of this schedule.');
    flow.table([{ header: 'Allocation year', width: 0.22 }, { header: 'Originally allocated ($000)', width: 0.26, align: 'right' }, { header: 'Outstanding ($000)', width: 0.26, align: 'right' }, { header: 'Planned retirement', width: 0.26 }], Array.from({ length: 14 }, (_, index) => {
      const year = 2012 + index;
      const original = retirement.int(3000, 12000);
      return [String(year), k(original), k(original * (index < 2 ? 0.1 : 0.4 + retirement.float() * 0.6)), year <= 2013 ? 'FY2027' : year <= 2016 ? `FY${2014 + index}` : 'Board discretion'];
    }), { size: 9, border: 'rules' });
  });
  section('Members by Patronage Size', () => {
    flow.heading('Members by Patronage Size', { level: 1, size: 16 });
    const size = rng.fork('size');
    flow.table([{ header: 'Annual business with the cooperative', width: 0.4 }, { header: 'Members', width: 0.2, align: 'right' }, { header: 'Share of patronage', width: 0.2, align: 'right' }, { header: 'Average age', width: 0.2, align: 'right' }], ['Under $10,000', '$10,000 - $50,000', '$50,000 - $250,000', '$250,000 - $1,000,000', 'Over $1,000,000'].map((band) => [band, grouped(size.int(300, 3400)), percent(size.int(200, 3400), 1), String(size.int(44, 67))]), { size: 9, border: 'rules' });
    flow.paragraph('The 200 largest members account for about 46% of gross margin; the 3,000 smallest account for less than 5%. The board considers both groups essential: large members provide most of the volume that keeps facilities efficient, and small members are often the next generation of large ones.');
  });
  section('Benefit Plans and Insurance', () => {
    flow.heading('Benefit Plans and Insurance Program', { level: 1, size: 16 });
    const insurance = rng.fork('insurance');
    flow.table([{ header: 'Coverage', width: 0.36 }, { header: 'Insurer or structure', width: 0.38 }, { header: 'Limit', width: 0.26, align: 'right' }], [
      ['Property and business interruption', 'Highmeadow Casualty Company', `$${grouped(insurance.int(180, 320))} million`],
      ['General and products liability', 'Northfell Mutual Insurance Company', '$2 million / $4 million'],
      ['Umbrella', 'Two-layer tower', `$${insurance.int(25, 60)} million`],
      ['Auto and hazardous materials transport', 'Saltash Point Indemnity Co.', '$5 million'],
      ['Workers\' compensation', 'Captive retention $500,000; excess insurer', 'Statutory'],
      ['Grain warehouse bonds', 'Surety', 'As required by state law'],
      ['Cyber liability', 'Specialty insurer', '$10 million'],
    ], { size: 9, border: 'rules' });
    flow.table([{ header: 'Employee plan', width: 0.4 }, { header: 'Participants', width: 0.2, align: 'right' }, { header: 'Cooperative cost ($000)', width: 0.4, align: 'right' }], ['Medical (self-insured)', 'Dental', 'Life and disability', '401(k) match', 'Pension (frozen)'].map((plan) => [plan, grouped(insurance.int(300, 1200)), k(insurance.int(400, 9000))]), { size: 9, border: 'rules' });
  });
  section('Debt and Covenants', () => {
    flow.heading('Debt Maturities and Loan Covenants', { level: 1, size: 16 });
    const debt = rng.fork('debt');
    flow.table([{ header: 'Fiscal year', width: 0.2 }, { header: 'Term loan A', width: 0.2, align: 'right' }, { header: 'Term loan B', width: 0.2, align: 'right' }, { header: 'Term loan C', width: 0.2, align: 'right' }, { header: 'Total ($000)', width: 0.2, align: 'right' }], ['2027', '2028', '2029', '2030', '2031', '2032 and later'].map((year) => {
      const parts = [debt.int(900, 4200), debt.int(800, 3600), debt.int(0, 2400)];
      return [year, ...parts.map(k), k(parts.reduce((a, b) => a + b, 0))];
    }), { size: 9, border: 'rules' });
    flow.table([{ header: 'Covenant', width: 0.5 }, { header: 'Requirement', width: 0.25, align: 'right' }, { header: 'June 30, 2026', width: 0.25, align: 'right' }], [
      ['Working capital', 'at least $40,000 thousand', `$${k(debt.int(52000, 88000))} thousand`],
      ['Members\' equity to total assets', 'at least 45%', percent(debt.int(5400, 6200), 1)],
      ['Debt service coverage', 'at least 1.35', decimal(debt.int(180, 290), 2)],
      ['Capital expenditures without lender consent', 'up to $30,000 thousand', `$${k(debt.int(14000, 26000))} thousand`],
    ], { size: 9, border: 'rules' });
    flow.paragraph(`Interest rate risk: ${debt.int(55, 78)}% of term debt bears a fixed rate. A one percentage point increase in short-term rates would raise annual interest on the seasonal facility and the variable term loan by about $${k(debt.int(500, 1300))} thousand at average fiscal 2026 balances.`);
  });
  section('Agronomy Acres by County', () => {
    flow.heading('Agronomy Services by County', { level: 1, size: 16 });
    const acres = rng.fork('acres');
    flow.table([{ header: 'County', width: 0.24 }, { header: 'Soil-sampled acres', width: 0.19, align: 'right' }, { header: 'Custom-applied acres', width: 0.19, align: 'right' }, { header: 'Variable-rate acres', width: 0.19, align: 'right' }, { header: 'Agronomists', width: 0.19, align: 'right' }], COUNTIES.map((county) => [county, grouped(acres.int(2000, 40000)), grouped(acres.int(5000, 52000)), grouped(acres.int(1000, 21000)), String(acres.int(1, 6))]), { size: 9, border: 'rules' });
    flow.paragraph(`Agronomists wrote ${grouped(acres.int(900, 2100))} field plans this year, an average of ${acres.int(120, 260)} acres each. Yield-monitor data from ${acres.int(140, 380)} members was used to calibrate variable-rate nitrogen recommendations, and the side-by-side trials on ${acres.int(30, 80)} farms showed an average return of $${acres.int(9, 31)} per acre over uniform application.`);
  });
  section('Rail and Logistics', () => {
    flow.heading('Rail and Logistics', { level: 1, size: 16 });
    const logistics = rng.fork('logistics');
    flow.table([{ header: 'Measure', width: 0.6 }, { header: 'FY2025', width: 0.2, align: 'right' }, { header: 'FY2026', width: 0.2, align: 'right' }], [
      ['Shuttle trains loaded', String(logistics.int(24, 50)), String(logistics.int(28, 61))],
      ['Average hours to load a shuttle train', decimal(logistics.int(100, 150), 1), decimal(logistics.int(95, 145), 1)],
      ['Rail cars leased', String(logistics.int(60, 140)), String(logistics.int(60, 140))],
      ['Trucks in fleet (all divisions)', String(logistics.int(140, 220)), String(logistics.int(140, 220))],
      ['Fleet miles (millions)', decimal(logistics.int(48, 72), 1), decimal(logistics.int(48, 72), 1)],
      ['Preventable accidents per million miles', decimal(logistics.int(40, 120), 2), decimal(logistics.int(30, 110), 2)],
      ['Freight cost per bushel to export terminal (cents)', String(logistics.int(48, 66)), String(logistics.int(52, 72))],
    ], { size: 9, border: 'rules' });
    flow.paragraph('The Oriel Junction expansion will add a loop track able to hold a second 110-car train, which the railroad has agreed to serve with dedicated crews during harvest. Construction is scheduled to finish before the fall 2027 harvest.');
  });
  group = 6;
  section('Five-Year Operating Statistics', () => {
    flow.heading('Five-Year Operating Statistics', { level: 1, size: 15 });
    const statistics = rng.fork('statistics');
    flow.table([{ header: '', width: 0.35 }, ...years.map((year) => ({ header: `FY${year}`, width: 0.13, align: 'right' }))], [
      ['Grain handled (million bu.)', ...years.map(() => decimal(statistics.int(380, 690), 1))],
      ['Fertilizer sold (thousand tons)', ...years.map(() => decimal(statistics.int(2100, 3400), 1))],
      ['Fuel and propane (million gal.)', ...years.map(() => decimal(statistics.int(410, 620), 1))],
      ['Feed sold (thousand tons)', ...years.map(() => decimal(statistics.int(1400, 2300), 1))],
      ['Milk processed (million lb.)', ...years.map(() => decimal(statistics.int(3300, 4100), 1))],
      ['Retail transactions (thousands)', ...years.map(() => grouped(statistics.int(380, 520)))],
      ['Active members', ...years.map(() => grouped(statistics.int(7600, 8900)))],
      ['Employees at year end', ...years.map(() => grouped(statistics.int(1010, 1240)))],
    ], { size: 9, border: 'rules' });
    flow.heading('Glossary', { level: 2 });
    for (const [term, meaning] of [
      ['Basis', 'the difference between a local cash price and a futures price.'],
      ['Local savings', 'the cooperative\'s earnings before income taxes, from business with members and others.'],
      ['Patronage refund', 'a distribution of patronage-sourced earnings to members in proportion to their business with the cooperative, paid partly in cash and partly as allocated equity.'],
      ['Allocated equity', 'patronage refunds retained by the cooperative and credited to members\' accounts, to be retired in later years.'],
      ['TRIR', 'total recordable incident rate: recordable injuries per 200,000 hours worked.'],
      ['Shuttle loader', 'a rail facility able to load a 110-car train within 15 hours.'],
    ]) flow.paragraph([{ text: `${term}: `, face: 'sans-bold' }, { text: meaning }], { size: 10 });
    flow.paragraph(`${COOP} - 400 Granary Road, Tamsin Valley, NM 87501 - (575) 555-0119 - members@tamsinvalley-coop.example`, { face: 'sans', size: 9, grey: 0.3, before: 8 });
  });
  section('Service Recognition', () => {
    flow.heading('Service Recognition', { level: 1, size: 16 });
    flow.paragraph('We thank the employees who reached a service milestone during fiscal 2026.');
    const honorees = people(rng.fork('honorees'), 24, { exclude: [...managers, ...directors, ...executives] });
    const service = rng.fork('service');
    flow.table([{ header: 'Employee', width: 0.34 }, { header: 'Location', width: 0.26 }, { header: 'Role', width: 0.26 }, { header: 'Years', width: 0.14, align: 'right' }], honorees.map((name, index) => [name, service.pick(locations).town, service.pick(['Elevator operator', 'Propane driver', 'Agronomist', 'Feed mill operator', 'Cheesemaker', 'Store clerk', 'Mechanic', 'Accountant', 'Dispatcher', 'Applicator']), String([10, 10, 10, 15, 15, 20, 20, 25, 30, 35, 40][index % 11])]), { size: 9, border: 'rules' });
  });
  section('Directory', () => {
    flow.heading('Directory of Locations', { level: 1, size: 16 });
    const directory = rng.fork('directory');
    flow.table([{ header: 'Location', width: 0.24 }, { header: 'Address', width: 0.36 }, { header: 'Telephone', width: 0.2 }, { header: 'Manager', width: 0.2 }], locations.map((location) => [`${location.town}, ${location.state}`, `${directory.int(10, 9800)} ${directory.pick(['Granary Road', 'Depot Street', 'County Road 14', 'Elevator Lane', 'Highway 60', 'Rail Avenue'])}`, `(${directory.int(201, 989)}) 555-01${directory.int(10, 99)}`, location.manager]), { size: 9, border: 'rules' });
    flow.paragraph('Member services: members@tamsinvalley-coop.example. Grain bids are posted at 7:00 a.m. and 1:30 p.m. each business day. Propane emergency line: (575) 555-0177, 24 hours.', { size: 10 });
  });

  // Contents page listing every section, then the sections themselves.
  flow.heading('Contents', { level: 1, size: 18 });
  const ordered = sections.map((entry, index) => ({ entry, index })).sort((a, b) => a.entry[2] - b.entry[2] || a.index - b.index).map(({ entry }) => entry);
  flow.startColumns(2, 24);
  ordered.forEach(([title]) => flow.paragraph(title, { size: 9, after: 0, face: 'sans' }));
  flow.endColumns();
  flow.pageBreak();
  ordered.forEach(([title, draw], index) => {
    if (index > 0 && !['Report of Independent Auditors', 'Consolidated Financial Statements', 'Supplementary Schedules'].includes(title)) flow.pageBreak();
    draw();
  });

  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  const datedPage = pageContaining(text, `Kestrel Ridge, Utah ${longDate(reportDate)}`.replace(/\s+/g, ' '));
  if (datedPage < 40 || datedPage > 60) throw new Error(`${id}: the report date must fall on pages 40-60, not ${datedPage}`);
  if (pages.length !== 100) throw new Error(`${id}: expected 100 pages, laid out ${pages.length}`);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'One-hundred-page annual report of a farmers\' cooperative',
    kind: 'annual_report',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_100', 'information_dense', 'middle_fact', 'financial', 'table', 'competing_dates', 'irrelevant_names'],
    notes: `The cover says only "Fiscal Year 2026" and the members' letter is undated. The report is dated once, under the independent auditors' signature on page ${datedPage}: ${longDate(reportDate)}. The fiscal year end (${longDate(yearEnd)}) is stated throughout and is accepted. The prior year end, the annual meeting (${longDate(meeting)}), and the patronage payment date (${longDate(patronageDate)}) are traps. Every page carries its own figures: three pages for each of six divisions, two for each of twelve locations, a page for each of ten member counties, ${risks.length} risk factors, the audited statements, and ${notes.length} notes. Directors, managers, and the auditors are named; the report is the cooperative's.`,
    gold: gold({
      type: 'Annual Report',
      acceptableTypes: ['Annual Report Fiscal Year 2026'],
      date: reportDate,
      acceptableDates: [yearEnd],
      role: 'issuance',
      forbiddenDates: [[priorYearEnd, 'prior fiscal year end'], [meeting, 'annual meeting date'], [patronageDate, 'cash patronage payment date']],
      parties: [COOP],
      relation: 'from',
      acceptablePartySets: [{ parties: [COOP], relation: 'for' }],
      roles: [[COOP, 'issuer'], [COOP, 'subject']],
      forbiddenParties: [[AUDITOR, 'independent auditors'], [directors[0], 'board chair'], [executives[0], 'chief executive']],
      facts: [['Fiscal Year 2026', 'fiscal 2026', 'FY2026'], ['cooperative', 'Cooperative'], [`$${k(totalRevenue[4])}`, k(totalRevenue[4]), 'patronage']],
      subjectTerms: ['annual report', 'cooperative', 'patronage', 'grain', 'members'],
      readiness: 'ready',
      dateText: [longDate(reportDate)],
    }),
  });
}

function riskFactors() {
  return [
    ['Commodity prices', 'Most of our revenue comes from buying and selling grain, fertilizer, fuel, feed ingredients, and milk, whose prices move with world markets. We hedge grain and some energy positions with exchange-traded futures, but basis, spreads, and the timing of member pricing decisions remain exposed, and a sharp move can turn a profitable position into a loss within days.'],
    ['Weather and crop size', 'Drought, hail, flooding, and early frost change how much grain members harvest and how much fertilizer and crop protection they buy. A short crop in our trade area reduces grain handling and drying income, and a wet spring can compress the application season into a few weeks and leave product unsold.'],
    ['Member concentration', 'Our 200 largest members account for about 46% of gross margin. If several of them retire, sell their operations, or move business to a competitor, our volumes and margins would fall more than the number of members lost would suggest.'],
    ['Competition from investor-owned companies', 'Regional and national grain companies, input retailers, and fuel distributors compete for every bushel and every gallon in our trade area. Several have greater financial resources and can sustain lower margins for longer than we can.'],
    ['Counterparty and credit risk', 'We extend seasonal credit to members for inputs and hold deferred-price contracts on their behalf. A widespread farm income downturn would raise bad debts. We also depend on futures commission merchants and grain buyers to perform their contracts.'],
    ['Liquidity during seasonal peaks', 'Our borrowing needs peak in the fall, when grain and fertilizer inventories and margin calls are highest. If our lender reduced the seasonal facility or margin calls exceeded our available credit, we might have to sell inventory or close positions at unfavorable prices.'],
    ['Interest rates', 'Higher interest rates raise the cost of carrying inventory and of our term debt, and they reduce the carry that makes grain storage profitable.'],
    ['Dependence on rail service', 'Our shuttle loaders depend on two railroads for car supply. Service disruptions during harvest can fill our storage and force us to stop receiving grain or to pay members less for it.'],
    ['Hazardous materials', 'We store and transport anhydrous ammonia, propane, and fuels. A release or explosion could injure employees or neighbors, damage property, lead to fines, and harm the cooperative\'s reputation.'],
    ['Grain dust and confined spaces', 'Grain elevators present risks of dust explosions and engulfment. Despite our training and permit-required entry procedures, a serious accident could cause loss of life and long shutdowns.'],
    ['Food safety in dairy processing', 'Contamination of milk or cheese could require a recall, harm consumers, and damage our relationships with the retailers and food companies that buy our products.'],
    ['Environmental regulation', 'Changes in rules on nutrient runoff, air emissions from dryers and feed mills, and wastewater from the dairy plant could require capital spending that does not increase revenue.'],
    ['Renewable fuels policy', 'Changes in federal and state renewable fuel standards and tax credits affect the value of the corn and soybean oil we market and the economics of renewable diesel we sell.'],
    ['Trade policy', 'Tariffs and export restrictions imposed by the United States or its trading partners can reduce export demand for grain and lower local prices, as happened in fiscal 2019.'],
    ['Labor availability', 'We compete for drivers holding commercial licenses with hazardous materials endorsements, certified applicators, and plant mechanics. Shortages raise wages and can delay deliveries at the busiest times of year.'],
    ['Aging facilities', 'Several of our elevators and the oldest feed mill were built before 1970. Maintaining them requires steady capital spending, and an unexpected structural failure could close a location during harvest.'],
    ['Information technology and cybersecurity', 'We rely on our grain accounting, dispatch, and payment systems to serve members. A ransomware attack or system failure during harvest could stop scales and settlements for days.'],
    ['Joint venture performance', 'Our share of earnings from the Bellmoor cheese joint venture depends on decisions we do not control alone and on cheese markets that have been volatile.'],
    ['Pension obligations', 'The frozen defined benefit pension plan remains underfunded. Lower discount rates or poor investment returns would increase required contributions.'],
    ['Equity retirement expectations', 'Members expect allocated equity to be retired on a regular cycle. If earnings fall, the board may slow retirements, which could weaken member loyalty.'],
    ['Tax treatment of patronage', 'The cooperative\'s income tax position depends on its qualifying as a cooperative under federal tax law. A change in that treatment would reduce the amount available for patronage refunds.'],
    ['Climate change', 'Longer droughts and more intense storms could change what crops our members grow and where, reduce yields over time, and increase the cost of insurance for our facilities.'],
    ['Consolidation of farms', 'Fewer, larger farms buy more directly from manufacturers and market grain to end users, bypassing local cooperatives. This trend could reduce our share of the business in our trade area.'],
    ['Governance and succession', 'The cooperative depends on a small group of senior managers and on directors who are working farmers. The loss of key people without a successor could disrupt operations.'],
  ];
}

function financialNotes(rng, { yearEnd, reportDate, divisions }) {
  const thousands = (low, high) => k(rng.int(low, high));
  return [
    ['Nature of Operations', [`${COOP} is an agricultural cooperative organized under the laws of New Mexico. It markets grain and milk for its members and supplies them with fertilizer, crop protection products, fuel, propane, feed, and farm and ranch merchandise from twelve locations. The consolidated financial statements include the accounts of the Cooperative and its wholly owned subsidiaries; its 50% interest in the Bellmoor cheese joint venture is accounted for using the equity method.`]],
    ['Summary of Significant Accounting Policies', ['Grain inventories, forward purchase and sale contracts, and exchange-traded futures and options are valued at net realizable value, with changes recognized in cost of goods sold. Other inventories are valued at the lower of cost (first-in, first-out) and net realizable value. Revenue is recognized when control of goods passes to the customer, generally on delivery or when the member prices grain. Patronage refunds received from other cooperatives are recorded when notification is received.', 'Property, plant and equipment is depreciated on the straight-line method over estimated useful lives of 10 to 40 years for buildings and 3 to 15 years for equipment.']],
    ['Receivables', ['Receivables are stated net of an allowance for credit losses. Members may finance input purchases through the seasonal program at a rate of prime plus 1.25%, due December 1.'], { columns: [{ header: '', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], rows: [['Trade receivables', thousands(31000, 52000), thousands(31000, 52000)], ['Seasonal input financing', thousands(9000, 24000), thousands(9000, 24000)], ['Other', thousands(400, 3000), thousands(400, 3000)], ['Allowance for credit losses', `(${thousands(600, 1900)})`, `(${thousands(600, 1900)})`]] }],
    ['Inventories', ['Grain inventories are hedged to the extent practical; open positions are within limits set by the board\'s risk committee.'], { columns: [{ header: '', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], rows: [['Grain', thousands(26000, 61000), thousands(26000, 61000)], ['Fertilizer and crop protection', thousands(14000, 33000), thousands(14000, 33000)], ['Energy', thousands(3000, 9000), thousands(3000, 9000)], ['Feed and ingredients', thousands(4000, 12000), thousands(4000, 12000)], ['Dairy products', thousands(5000, 15000), thousands(5000, 15000)], ['Retail merchandise', thousands(6000, 14000), thousands(6000, 14000)]] }],
    ['Derivative Instruments', ['The Cooperative uses exchange-traded futures and options and over-the-counter swaps to manage price risk on grain, energy, and dairy commodities. These instruments are not designated as hedges for accounting purposes. Margin deposits with futures commission merchants were restricted at year end.'], { columns: [{ header: 'Contract type', width: 0.4 }, { header: 'Notional (thousand units)', width: 0.3, align: 'right' }, { header: 'Fair value ($000)', width: 0.3, align: 'right' }], rows: [['Corn futures (bushels)', thousands(4000, 19000), thousands(-900, 1600)], ['Wheat futures (bushels)', thousands(1000, 7000), thousands(-500, 900)], ['Soybean futures (bushels)', thousands(500, 4000), thousands(-300, 700)], ['Diesel swaps (gallons)', thousands(1000, 6000), thousands(-400, 500)], ['Class III milk futures (pounds)', thousands(20000, 90000), thousands(-600, 800)]] }],
    ['Investments in Other Cooperatives', ['Investments in other cooperatives are carried at cost plus allocated patronage, less cash redemptions.'], { columns: [{ header: 'Cooperative', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], rows: [['Kingsfold Farm Credit Bank', thousands(4000, 9000), thousands(4000, 9000)], ['Regional supply cooperative (federated)', thousands(12000, 29000), thousands(12000, 29000)], ['Grain marketing cooperative', thousands(3000, 8000), thousands(3000, 8000)], ['Bellmoor cheese joint venture (equity method)', thousands(6000, 15000), thousands(6000, 15000)], ['Other', thousands(200, 1500), thousands(200, 1500)]] }],
    ['Property, Plant and Equipment', [`Depreciation expense was $${thousands(8000, 14000)} thousand in fiscal 2026. Construction in progress at ${longDate(yearEnd)} related mainly to the Oriel Junction shuttle loader, with remaining commitments of $${thousands(3000, 9000)} thousand.`]],
    ['Leases', ['The Cooperative leases rail cars, delivery trucks, and the Briarport distribution center under operating leases. Maturities of lease liabilities:'], { columns: [{ header: 'Fiscal year', width: 0.6 }, { header: 'Amount ($000)', width: 0.4, align: 'right' }], rows: [['2027', thousands(2400, 4200)], ['2028', thousands(2000, 3800)], ['2029', thousands(1500, 3000)], ['2030', thousands(900, 2400)], ['2031', thousands(500, 1800)], ['Thereafter', thousands(800, 4000)]] }],
    ['Seasonal Notes Payable', ['The Cooperative has a seasonal revolving credit facility with Kingsfold Farm Credit Bank with a maximum commitment that varies by month, from $60 million in July to $140 million in November. Borrowings bear interest at SOFR plus 2.00%.']],
    ['Long-Term Debt', ['Long-term debt consists of term loans from Kingsfold Farm Credit Bank secured by substantially all property, plant and equipment. The loan agreement requires minimum working capital of $40 million, a minimum ratio of members\' equity to total assets of 45%, and a minimum debt service coverage ratio of 1.35.'], { columns: [{ header: 'Loan', width: 0.4 }, { header: 'Rate', width: 0.2, align: 'right' }, { header: 'Maturity', width: 0.2, align: 'right' }, { header: 'Balance ($000)', width: 0.2, align: 'right' }], rows: [['Term loan A', '4.15%', '2029', thousands(9000, 18000)], ['Term loan B (dairy plant)', '5.05%', '2033', thousands(12000, 26000)], ['Term loan C (shuttle loader)', 'SOFR + 2.25%', '2036', thousands(5000, 14000)], ['Equipment notes', '3.90% - 6.40%', '2027 - 2031', thousands(1000, 4000)]] }],
    ['Income Taxes', ['The Cooperative is subject to income taxes on non-patronage income and on patronage income not allocated to members. Deferred taxes relate mainly to depreciation and to the difference between book and tax treatment of non-qualified allocations.']],
    ['Retirement Plans', ['The Cooperative sponsors a 401(k) plan with a matching contribution of 100% of the first 5% of pay, and a defined benefit pension plan frozen to new participants and benefit accruals since 2013.'], { columns: [{ header: 'Pension plan', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], rows: [['Projected benefit obligation', thousands(31000, 42000), thousands(31000, 42000)], ['Fair value of plan assets', thousands(24000, 36000), thousands(24000, 36000)], ['Discount rate', '5.40%', '5.10%'], ['Expected return on assets', '6.25%', '6.25%'], ['Employer contributions', thousands(900, 2400), thousands(900, 2400)]] }],
    ['Members\' Equity', ['Members\' equity consists of common stock ($1,000 per share), allocated equities, and unallocated retained earnings. The board determines each year the cash portion of patronage refunds and the equity to be retired.'], { columns: [{ header: 'Allocation year', width: 0.4 }, { header: 'Outstanding ($000)', width: 0.3, align: 'right' }, { header: 'Planned retirement', width: 0.3, align: 'right' }], rows: [['2012 and prior', thousands(800, 3000), 'FY2027'], ['2013 - 2016', thousands(9000, 21000), 'FY2028 - FY2031'], ['2017 - 2020', thousands(14000, 29000), 'Board discretion'], ['2021 - 2025', thousands(22000, 46000), 'Board discretion'], ['2026 (this allocation)', thousands(6000, 15000), 'Board discretion']] }],
    ['Patronage Refunds', [`Patronage-sourced savings for fiscal 2026 will be allocated to members in proportion to their business by division, as shown in the supplementary schedules. The board has approved payment of 40% in cash in December 2026 and retention of 60% as qualified allocated equity.`]],
    ['Segment Information', ['The Cooperative manages its business through six divisions, which are its reportable segments.'], { columns: [{ header: 'Segment', width: 0.4 }, { header: 'Revenue ($000)', width: 0.3, align: 'right' }, { header: 'Gross margin ($000)', width: 0.3, align: 'right' }], rows: divisions.map((division) => [division.name, k(division.revenue[4]), k(division.margins[4])]) }],
    ['Concentrations', [`Sales to the five largest grain buyers accounted for ${rng.int(28, 44)}% of grain revenue, and the two railroads serving our shuttle loaders handled ${rng.int(51, 79)}% of grain shipped. Approximately ${rng.int(30, 55)}% of dairy plant output is sold to one national retailer under an agreement renewed through 2028.`]],
    ['Commitments', [`At ${longDate(yearEnd)} the Cooperative had commitments to purchase ${grouped(rng.int(120, 640))} thousand bushels of grain from members at prices to be set, fixed-price fertilizer purchase commitments of $${thousands(4000, 15000)} thousand, and equipment purchase commitments of $${thousands(2000, 9000)} thousand.`]],
    ['Contingencies', ['The Cooperative is a defendant in a lawsuit alleging that grain dust from the Fallow Creek elevator damaged a neighboring orchard. Management believes the claim is without merit and is covered by insurance. The Cooperative is also subject to routine claims arising in the ordinary course of business, none of which management expects to have a material effect.']],
    ['Related Party Transactions', ['Directors and their farming operations do business with the Cooperative on the same terms as other members. Purchases from directors\' operations were $' + thousands(1200, 4200) + ' thousand and sales to them $' + thousands(2600, 7800) + ' thousand during fiscal 2026.']],
    ['Fair Value Measurements', ['Assets and liabilities measured at fair value are classified by the level of inputs used. Grain inventories and forward contracts are Level 2, valued from exchange prices adjusted for local basis; futures and options are Level 1; swaps are Level 2.'], { columns: [{ header: 'June 30, 2026 ($000)', width: 0.4 }, { header: 'Level 1', width: 0.2, align: 'right' }, { header: 'Level 2', width: 0.2, align: 'right' }, { header: 'Total', width: 0.2, align: 'right' }], rows: [['Grain inventories', '-', thousands(26000, 61000), thousands(26000, 61000)], ['Forward purchase contracts', '-', thousands(800, 4200), thousands(800, 4200)], ['Forward sale contracts', '-', thousands(600, 3800), thousands(600, 3800)], ['Exchange-traded derivatives', thousands(200, 2600), '-', thousands(200, 2600)], ['Swaps', '-', thousands(50, 900), thousands(50, 900)]] }],
    ['Intangible Assets and Goodwill', ['Intangible assets arose from the 2019 acquisition of the Dunmore Lake feed business and consist of customer relationships amortized over 12 years and a trade name amortized over 5 years. Goodwill of $' + thousands(1200, 3400) + ' thousand was tested for impairment at year end; no impairment was recorded.']],
    ['Accrued Expenses', ['Accrued expenses consisted of the following at year end.'], { columns: [{ header: '', width: 0.6 }, { header: '2026', width: 0.2, align: 'right' }, { header: '2025', width: 0.2, align: 'right' }], rows: [['Wages, bonuses, and vacation', thousands(4200, 7800), thousands(4200, 7800)], ['Health plan claims incurred but not reported', thousands(600, 1400), thousands(600, 1400)], ['Property taxes', thousands(900, 2100), thousands(900, 2100)], ['Interest', thousands(200, 900), thousands(200, 900)], ['Other', thousands(700, 2600), thousands(700, 2600)]] }],
    ['Self-Insurance', ['The Cooperative is self-insured for employee health benefits up to $175,000 per participant per year and for workers\' compensation up to $500,000 per claim through its captive insurance subsidiary, with stop-loss coverage above those amounts. Reserves are based on actuarial estimates.']],
    ['Revenue Disaggregation', ['Revenue by type for fiscal 2026 (in thousands of dollars).'], { columns: [{ header: 'Type', width: 0.6 }, { header: 'Amount', width: 0.4, align: 'right' }], rows: [['Grain sales', k(divisions[0].revenue[4])], ['Agronomy products and services', k(divisions[1].revenue[4])], ['Fuel, propane, and lubricants', k(divisions[2].revenue[4])], ['Feed and nutrition products', k(divisions[3].revenue[4])], ['Dairy products', k(divisions[4].revenue[4])], ['Retail merchandise', k(divisions[5].revenue[4])], ['Storage, drying, and service income', thousands(3000, 9000)]] }],
    ['Joint Venture', ['The Cooperative owns 50% of a cheese manufacturing joint venture at Bellmoor, Wisconsin. Summarized financial information of the joint venture (100%) for its year ended June 30, 2026: revenue $' + thousands(60000, 98000) + ' thousand; net income $' + thousands(1200, 6400) + ' thousand; total assets $' + thousands(40000, 72000) + ' thousand. The Cooperative supplies about 40% of the joint venture\'s milk.']],
    ['Asset Retirement Obligations', ['The Cooperative has recorded obligations to remove underground fuel storage tanks and to restore leased rail sidings at the end of their leases, measured at the present value of estimated costs. The liability was $' + thousands(400, 1300) + ' thousand at year end.']],
    ['Subsequent Events', [`Management evaluated subsequent events through ${longDate(reportDate)}, the date the financial statements were available to be issued. In August the board approved the purchase of the assets of an independent grain elevator near Oriel Junction for $${thousands(2600, 5400)} thousand, expected to close in the second quarter of fiscal 2027.`]],
  ];
}
