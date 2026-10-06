/// Records: board minutes crowded with names, a museum loan condition report,
/// and an aircraft maintenance record. The last two are unusual documents a
/// filing assistant still has to name sensibly.
import { Flow } from '../lib/layout.mjs';
import { gold } from '../lib/gold.mjs';
import { longDate, numericDate } from '../lib/format.mjs';
import { digitalPdf, letterhead, result } from './common.mjs';

export function boardMinutes() {
  const id = 'board-minutes';
  const org = 'Cairnfield Cooperative Grocers';
  const meeting = '2025-09-17';
  const prior = '2025-08-20';
  const next = '2025-10-15';
  const ballots = '2025-11-14';
  const directors = ['Beckett Halloran (President)', 'Mabel Trevelyan (Vice President)', 'Soren Achterberg (Treasurer)', 'Yara Delacroix-Hale (Secretary)', 'Casimir Boateng', 'Nell Fitzgerald-Moss', 'Isidore Vanterpool', 'Rosalind Kilbride', 'Ezra Lindqvist'];
  const absent = ['Celeste Marchetti'];
  const staff = ['Dashiell Pemberton, General Manager', 'Farida Okonkwo, Finance Director', 'Hollis Abernathy, Store Manager - Elm Street', 'Odalys Varga, Store Manager - Wrenfield', 'Lionel Saltonstall, Member Services Coordinator'];
  const guests = ['Tove Nakashima', 'Gideon Ravenscroft', 'Paloma Inglewood', 'Kofi Wexley', 'Greer Somerled'];
  const flow = new Flow({ face: 'serif', fontSize: 10.5, margins: { top: 60, bottom: 64 }, footer: (page, { number, total }) => page.textCenter(306, 760, `Board minutes - ${longDate(meeting)} - page ${number} of ${total}`, { face: 'sans', size: 7.5, grey: 0.35 }), keep: [org] });
  flow.heading(org.toUpperCase(), { level: 1, align: 'center', size: 13, after: 2 });
  flow.paragraph('Minutes of the Regular Meeting of the Board of Directors', { align: 'center', face: 'sans-bold', size: 11.5, after: 2 });
  flow.paragraph(`${longDate(meeting)} - Community Room, Elm Street Store, 412 Elm Street, Bellmoor, WI`, { align: 'center', face: 'sans', size: 9.5, after: 12 });
  flow.fields([
    ['Directors present:', directors.join(', ')],
    ['Directors absent:', `${absent.join(', ')} (excused)`],
    ['Staff present:', staff.join('; ')],
    ['Member-owners present:', guests.join(', ')],
    ['Recording secretary:', 'Lionel Saltonstall'],
  ], { labelWidth: 120, size: 9.5 });
  const items = [
    ['1. Call to order', `President Beckett Halloran called the regular meeting of the Board of Directors of ${org} to order at 6:32 p.m. A quorum of nine directors was present.`],
    ['2. Approval of agenda', 'The agenda was approved as circulated, with the patronage dividend discussion moved ahead of committee reports. (Kilbride / Boateng; approved unanimously.)'],
    ['3. Approval of minutes', `The minutes of the regular meeting of ${longDate(prior)} were approved with one correction: the August sales figure for the Wrenfield store should read $612,480, not $621,480. (Trevelyan / Vanterpool; approved, Lindqvist abstaining.)`],
    ['4. General Manager\'s report', 'Dashiell Pemberton reported that combined sales for August were $1,904,215, up 6.2% from August 2024, with produce and bulk leading growth. Transactions rose 3.8% and the average basket 2.3%. Member-owner sales were 71% of the total. Labor cost was 13.9% of sales against a budget of 14.2%. The Wrenfield store\'s new deli counter opened on September 2 and averaged $1,140 a day in its first two weeks.'],
    ['5. Finance report', 'Farida Okonkwo presented the unaudited results for the first eight months of the fiscal year: net sales of $14.31 million, gross margin of 35.6%, and net income of $186,400, compared with a budgeted $152,000. Cash on hand was $1.27 million. The line of credit with Kingsfold Community Bank remains undrawn. The board accepted the finance report. (Achterberg / Fitzgerald-Moss; approved unanimously.)'],
    ['6. Patronage dividend', 'The board discussed the Finance Committee\'s recommendation to allocate 70% of the fiscal year\'s patronage-sourced income to member-owners, with 20% paid in cash and 80% retained as equity, and to revisit the cash percentage in March once the audit is complete. After discussion of the Wrenfield expansion\'s capital needs, the board adopted the recommendation. (Achterberg / Trevelyan; approved 8-1, Boateng opposed.)'],
    ['7. Committee reports', `Board Development: three member-owners have submitted candidacy forms for the three open seats; the committee will hold a candidate forum on October 28. Ballots are due ${longDate(ballots)}. Member Engagement: 214 new member-owners joined in August, the most in any month since 2021. Policy: the committee will bring revised board expense reimbursement guidelines to the October meeting.`],
    ['8. Old business - Wrenfield expansion', 'The General Manager reported that negotiations with the landlord of the adjacent space at 1188 Pennant Street are continuing. The landlord has proposed a ten-year lease at $18.50 per square foot with a tenant improvement allowance of $35 per square foot. The board directed the General Manager to continue negotiations within the parameters set in executive session in July.'],
    ['9. New business', 'The board approved a donation of $2,500 to the Bellmoor Community Food Pantry\'s winter drive. (Kilbride / Halloran; approved unanimously.) The board asked staff to prepare options for a second electric vehicle charging station at the Elm Street store.'],
    ['10. Member-owner comments', 'Paloma Inglewood asked about extended hours at the Wrenfield store during the holiday season; the General Manager will report back in October. Kofi Wexley thanked the board for the bulk department expansion.'],
    ['11. Executive session', 'The board went into executive session at 8:21 p.m. to discuss the General Manager\'s annual performance review, and returned to open session at 8:44 p.m. No action was taken.'],
    ['12. Adjournment', `The meeting was adjourned at 8:47 p.m. The next regular meeting will be held on ${longDate(next)}.`],
  ];
  for (const [heading, body] of items) {
    flow.paragraph(heading, { face: 'sans-bold', size: 10, after: 2, keepWithNext: 24 });
    flow.paragraph(body, { indent: 14 });
  }
  flow.paragraph('Respectfully submitted,', { before: 6, after: 12 });
  flow.paragraph('/s/ Yara Delacroix-Hale, Secretary', { after: 4 });
  flow.paragraph(`Approved by the Board of Directors on ${longDate(next)}.`, { size: 9, grey: 0.3 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Grocery cooperative board minutes with twenty named attendees',
    kind: 'minutes',
    textLayer: 'native',
    pages: pages.length,
    categories: ['irrelevant_names', 'competing_dates'],
    notes: `The meeting date (${longDate(meeting)}) is under the title and in every footer. Twenty people are named as directors, staff, and guests, and a bank and a food pantry appear in the business - none is a party; the minutes are the cooperative's. The approved minutes of the prior meeting (${longDate(prior)}), the ballot deadline (${longDate(ballots)}), and the next meeting (${longDate(next)}, which is also when these minutes were approved) are traps. Filing with no party at all is accepted.`,
    gold: gold({
      type: 'Board Meeting Minutes',
      acceptableTypes: ['Minutes of the Regular Meeting of the Board of Directors', 'Meeting Minutes', 'Board Minutes'],
      date: meeting,
      role: 'issuance',
      forbiddenDates: [[prior, 'date of the prior meeting whose minutes were approved'], [next, 'next meeting date and approval date'], [ballots, 'board election ballot deadline']],
      parties: [org],
      relation: 'for',
      acceptablePartySets: [{ parties: [], relation: 'none' }, { parties: [org], relation: 'from' }],
      roles: [[org, 'subject'], [org, 'issuer']],
      forbiddenParties: [['Beckett Halloran', 'board president'], ['Dashiell Pemberton', 'general manager'], ['Kingsfold Community Bank', 'lender mentioned in the finance report'], ['Paloma Inglewood', 'member-owner guest']],
      facts: [[org, 'Cairnfield'], ['patronage dividend', 'patronage'], ['Wrenfield']],
      subjectTerms: ['board', 'patronage dividend', 'Wrenfield expansion', 'minutes'],
      readiness: 'ready',
      dateText: [longDate(meeting)],
    }),
  });
}

export function fieldConditionReport() {
  const id = 'field-condition-report';
  const museum = 'Saltash Point Museum of Art';
  const lender = 'Haverford Family Collection';
  const conservator = 'Delphine Moncrieff';
  const registrar = 'Anselm Greaves';
  const exam = '2025-10-02';
  const previous = '2019-03-14';
  const opens = '2025-11-07';
  const closes = '2026-02-22';
  const flow = new Flow({ face: 'sans', fontSize: 9.5, margins: { top: 48, bottom: 56, left: 54, right: 54 }, keep: [museum, lender] });
  flow.y = letterhead(flow.page, { name: museum, lines: ['Registration and Conservation  -  1 Gallery Walk, Saltash Point, CA 94923'], x: 54, width: 504, size: 14 });
  flow.heading('LOAN CONDITION REPORT', { level: 1, size: 14, after: 2 });
  flow.paragraph('Outgoing examination at lender\'s premises', { size: 9.5, grey: 0.3, after: 8 });
  flow.table([{ header: 'Field', width: 0.3 }, { header: 'Entry', width: 0.7 }], [
    ['Date of examination', longDate(exam)],
    ['Examined by', `${conservator}, Associate Conservator of Paintings`],
    ['Location', 'Lender\'s residence, Haverford House, 2 Fennel Court, Pemberly Falls, NY'],
    ['Lender', lender],
    ['Lender inventory no.', 'HFC-0447'],
    ['Exhibition', `Tides of Light: Coastal Painting 1880-1920 (${longDate(opens)} - ${longDate(closes)})`],
    ['Artist', 'Elias Brandvold (1871-1934)'],
    ['Title / date', 'Harbor at Dusk, Gilchrist, 1906'],
    ['Medium / support', 'Oil on canvas, lined; original stretcher replaced'],
    ['Dimensions', 'Canvas 24 x 36 in. (61 x 91.4 cm); framed 31 1/2 x 43 1/2 x 3 in.'],
    ['Previous report', `Lender's insurer survey dated ${longDate(previous)}`],
  ], { size: 9 });
  flow.heading('Condition by area', { level: 3 });
  flow.table([{ header: 'Area', width: 0.2 }, { header: 'Condition', width: 0.16 }, { header: 'Observations', width: 0.64 }], [
    ['Support', 'Good', 'Glue-paste lining (2011, Penhaligon Conservation Studio) is sound; slight draw at upper left corner; tension even.'],
    ['Ground', 'Good', 'Commercially prepared off-white ground visible at tacking margins; no losses.'],
    ['Paint layer', 'Fair', 'Fine age craquelure overall. Area A: 2 cm tented cleavage in the dark water at lower left, stable on examination. Area B: three pinpoint losses in the sky, upper right, previously inpainted.'],
    ['Varnish', 'Fair', 'Natural resin varnish, moderately discolored (yellow); slight blanching along the lower edge.'],
    ['Frame', 'Good', 'Period gilt frame; minor gilding losses at the lower right corner; ornament secure; rabbet lined with felt.'],
    ['Glazing / backing', 'Good', 'Low-reflection acrylic glazing; corrugated board backing to be replaced by borrower before travel.'],
    ['Hanging hardware', 'Good', 'Two D-rings with security hangers; wire removed at examination.'],
  ], { size: 8.5 });
  flow.heading('Requirements for loan', { level: 3 });
  for (const item of [
    'Environment: 70 F +/- 2 F; relative humidity 50% +/- 5%; illumination not to exceed 150 lux; no direct daylight.',
    'Area A must be checked by a conservator on arrival and again on deinstallation; the borrower may not consolidate it without the lender\'s written approval.',
    'Packing: travel frame inside a double-walled crate with 2-inch polyethylene foam; crate to acclimatize 24 hours before unpacking.',
    'Courier: a borrower courier accompanies the work in both directions; no overnight stops in transit.',
    'Photography: images 01-14 taken at examination, raking light on Area A (images 06-08).',
  ]) flow.paragraph(`- ${item}`, { indent: 10 });
  flow.paragraph(`Lender's representative: ____________________     Borrower's registrar: /s/ ${registrar}`, { before: 10, size: 9.5 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Museum loan condition report on a lent painting',
    kind: 'condition_report',
    textLayer: 'native',
    pages: pages.length,
    categories: ['unusual', 'table', 'competing_dates'],
    notes: `An unusual document: a conservator's condition check of a painting before it travels for an exhibition. The examination date (${longDate(exam)}) is the report's date; the exhibition run (${longDate(opens)} - ${longDate(closes)}) and the prior survey (${longDate(previous)}) are traps, and 1906, 2011, 1871-1934 are years, not dates. Who it is filed under is a judgement: the museum that prepared it is the gold, the lender is accepted, and so is no party.`,
    gold: gold({
      type: 'Loan Condition Report',
      acceptableTypes: ['Condition Report'],
      date: exam,
      role: 'issuance',
      forbiddenDates: [[previous, 'date of the previous condition survey'], [opens, 'exhibition opening'], [closes, 'exhibition closing']],
      parties: [museum],
      relation: 'from',
      acceptablePartySets: [{ parties: [lender], relation: 'for' }, { parties: [museum, lender], relation: 'between' }, { parties: [], relation: 'none' }],
      roles: [[museum, 'issuer'], [lender, 'subject'], [lender, 'client']],
      forbiddenParties: [[conservator, 'conservator who examined the work'], ['Penhaligon Conservation Studio', 'studio that lined the canvas in 2011'], ['Elias Brandvold', 'the artist']],
      facts: [['Harbor at Dusk'], ['Brandvold'], ['Tides of Light', 'exhibition', 'loan']],
      subjectTerms: ['condition', 'painting', 'loan', 'Harbor at Dusk'],
      readiness: 'either',
      dateText: [longDate(exam)],
    }),
  });
}

export function aircraftMaintenanceLog() {
  const id = 'aircraft-maintenance-log';
  const operator = 'Highmeadow Air Charter LLC';
  const station = 'Kestrel Ridge Aero Services';
  const inspector = 'Arlo Sinclair-Ray';
  const returned = '2026-08-14';
  const previousAnnual = '2025-08-09';
  const nextDue = '2027-08-31';
  const elt = '2027-02-28';
  const entries = [
    ['2026-08-10', '4,812.6', 'Aircraft received for annual / 100-hour inspection. Compression check (cyl 1-4): 76/80, 74/80, 77/80, 75/80.', 'AS'],
    ['2026-08-10', '4,812.6', 'Removed and cleaned spark plugs; rotated top and bottom; gapped 0.018 in. Oil and filter changed, 8 qt. Filter cut open, no metal.', 'JM'],
    ['2026-08-11', '4,812.6', 'Left main gear brake linings below limits; replaced linings both sides, P/N HB-66-105. Bled brakes, ops check good.', 'JM'],
    ['2026-08-11', '4,812.6', 'Found cracked exhaust gasket at cylinder 3; replaced gasket, torqued per manual. Leak check good.', 'AS'],
    ['2026-08-12', '4,812.6', 'Complied with AD 2011-10-09 (fuel selector valve inspection) - no defects. Recurring at 100 hours.', 'AS'],
    ['2026-08-12', '4,812.6', 'Complied with AD 2020-18-06 (seat rail inspection) - rails within limits. Recurring at annual.', 'AS'],
    ['2026-08-13', '4,812.9', 'Ground run and post-maintenance taxi check; magneto drop L 75 / R 100 rpm; static rpm 2,310.', 'AS'],
  ];
  const flow = new Flow({ face: 'sans', fontSize: 9, margins: { top: 48, bottom: 56, left: 50, right: 50 }, keep: [operator, station] });
  const page = flow.page;
  page.text(50, 62, 'AIRCRAFT MAINTENANCE RECORD', { face: 'sans-bold', size: 15 });
  page.text(50, 76, `${station}  -  Repair Station Certificate KRAS-417R  -  Hangar 6, Kestrel Ridge Municipal Airport, UT`, { size: 8, grey: 0.3 });
  flow.y = 90;
  flow.table([{ header: 'Registration', width: 0.14 }, { header: 'Make / model', width: 0.24 }, { header: 'Serial no.', width: 0.14 }, { header: 'Engine', width: 0.26 }, { header: 'Owner / operator', width: 0.22 }], [
    ['N0612X', 'Halden Aero Works HA-180', 'HA180-01977', 'Tamarack Engines TE-320-D2, S/N T-4419-36', operator],
  ], { size: 8.5 });
  flow.table([{ header: 'Airframe total time', width: 0.25 }, { header: 'Tach at inspection', width: 0.25 }, { header: 'Engine time since overhaul', width: 0.25 }, { header: 'Work order', width: 0.25 }], [['6,204.3 hrs', '4,812.6', '1,388.4 hrs', 'WO-26-0813']], { size: 8.5 });
  flow.heading('Maintenance entries', { level: 3, size: 10 });
  flow.table([
    { header: 'Date', width: 0.12 },
    { header: 'Tach', width: 0.09 },
    { header: 'Work performed', width: 0.69 },
    { header: 'Mech.', width: 0.1 },
  ], entries.map(([date, tach, work, initials]) => [numericDate(date), tach, work, initials]), { size: 8.5 });
  flow.heading('Inspection and return to service', { level: 3, size: 10 });
  flow.paragraph(`I certify that this aircraft has been inspected on ${numericDate(returned)} in accordance with an annual inspection and a 100-hour inspection and was determined to be in airworthy condition. Previous annual inspection: ${numericDate(previousAnnual)}.`, { face: 'serif', size: 10 });
  flow.table([{ header: 'Signature', width: 0.3 }, { header: 'Certificate no.', width: 0.28 }, { header: 'Date of return to service', width: 0.24 }, { header: 'Tach', width: 0.18 }], [[`/s/ ${inspector}`, 'A&P / IA 3318492', numericDate(returned), '4,812.9']], { size: 9 });
  flow.heading('Items due', { level: 3, size: 10 });
  flow.table([{ header: 'Item', width: 0.5 }, { header: 'Due', width: 0.5 }], [
    ['Next annual inspection', `End of month, ${numericDate(nextDue)}`],
    ['Next 100-hour inspection', 'Tach 4,912.9'],
    ['ELT battery replacement', numericDate(elt)],
    ['AD 2011-10-09 recurring', 'Tach 4,912.9'],
    ['Transponder / altimeter check (24 months)', '06/2027'],
  ], { size: 8.5 });
  flow.paragraph('Mechanics: AS - Arlo Sinclair-Ray (A&P/IA); JM - Joaquin Moncrieff (A&P 4401873). Parts traceability records retained with the work order.', { size: 8, grey: 0.3 });
  const pages = flow.finish();
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Light aircraft annual inspection and maintenance record',
    kind: 'maintenance_record',
    textLayer: 'native',
    pages: pages.length,
    categories: ['unusual', 'table', 'competing_dates'],
    notes: `Seven dated entries over four days lead up to the return to service on ${numericDate(returned)}, which is the record's date. The previous annual (${numericDate(previousAnnual)}), the next annual due (${numericDate(nextDue)}), the ELT battery due date, and two airworthiness directive numbers that look exactly like ISO dates (AD 2011-10-09, AD 2020-18-06) are traps. Filed for the aircraft's operator; from the repair station is also accepted.`,
    gold: gold({
      type: 'Aircraft Maintenance Record',
      acceptableTypes: ['Maintenance Record', 'Aircraft Maintenance Log', 'Annual Inspection Record'],
      date: returned,
      role: 'issuance',
      forbiddenDates: [[previousAnnual, 'previous annual inspection'], [nextDue, 'next annual inspection due'], [elt, 'ELT battery due'], ['2011-10-09', 'airworthiness directive number that reads like a date'], ['2026-08-10', 'first maintenance entry']],
      parties: [operator],
      relation: 'for',
      acceptablePartySets: [{ parties: [station], relation: 'from' }],
      roles: [[operator, 'subject'], [operator, 'client'], [station, 'issuer'], [station, 'provider']],
      forbiddenParties: [[inspector, 'inspector who signed the return to service'], ['Halden Aero Works', 'aircraft manufacturer'], ['Tamarack Engines', 'engine manufacturer']],
      facts: [['N0612X'], ['annual inspection', 'annual'], ['brake', 'exhaust gasket']],
      subjectTerms: ['annual inspection', 'aircraft', 'airworthy', 'N0612X'],
      readiness: 'either',
      dateText: [numericDate(returned)],
    }),
  });
}
