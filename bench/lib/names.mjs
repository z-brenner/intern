/// Fictional people, organisations, places, and identifiers.
///
/// Every name InternBench prints comes from these pools or from a builder's
/// own hand-written cast. Company names are built from pairs of unusual
/// words so that they read like real firms without being any real firm;
/// places are invented towns with real state abbreviations and plausible ZIP
/// codes. Nothing here may be a real company (and not the stock placeholder
/// names either: no Contoso, Acme, or Northwind).

export const FIRST_NAMES = [
  'Adaeze', 'Alden', 'Amara', 'Anika', 'Anselm', 'Arlo', 'Beatrix', 'Bram', 'Calla', 'Casimir',
  'Celeste', 'Corwin', 'Dalia', 'Darius', 'Delphine', 'Desmond', 'Elodie', 'Emeric', 'Esme', 'Ezra',
  'Farida', 'Fenwick', 'Gideon', 'Greer', 'Hana', 'Hollis', 'Ilse', 'Imogen', 'Ines', 'Isidore',
  'Jaya', 'Joaquin', 'Juniper', 'Kasimir', 'Keziah', 'Kofi', 'Lark', 'Leopold', 'Linnea', 'Lucan',
  'Mabel', 'Mateo', 'Meridel', 'Mireille', 'Nadia', 'Nikolai', 'Noor', 'Odalys', 'Orrin', 'Oswin',
  'Paloma', 'Priya', 'Quentin', 'Rafferty', 'Renata', 'Rhiannon', 'Rosalind', 'Rufus', 'Saoirse', 'Saul',
  'Selby', 'Signe', 'Soren', 'Tamsin', 'Tariq', 'Thaddeus', 'Tove', 'Ulrich', 'Valentina', 'Vesna',
  'Wendell', 'Wilhelmina', 'Xavier', 'Yara', 'Yusuf', 'Zelda', 'Zoltan', 'Beckett', 'Corinne', 'Dashiell',
  'Elspeth', 'Florian', 'Gwendolyn', 'Hugo', 'Ione', 'Jasper', 'Kaia', 'Lionel', 'Marisol', 'Nell',
];

export const LAST_NAMES = [
  'Abernathy', 'Achterberg', 'Adeyemi', 'Albrecht', 'Ambrose', 'Arkwright', 'Ashdown', 'Baptiste', 'Barlowe', 'Bellweather',
  'Birkett', 'Blackwood', 'Boateng', 'Brandvold', 'Brightwater', 'Calloway', 'Castellan', 'Chakrabarti', 'Coldwell', 'Corrigan',
  'Dahlquist', 'Delacroix-Hale', 'Devereaux', 'Dunmore', 'Eckhardt', 'Ellingsen', 'Escobedo', 'Fairbanks', 'Falkenrath', 'Fennimore',
  'Fitzgerald-Moss', 'Galbraith', 'Garroway', 'Gillespie', 'Greaves', 'Halloran', 'Haverford', 'Hollingsworth', 'Iwasaki', 'Jaramillo',
  'Kavanagh', 'Kettleborough', 'Kowalczyk', 'Lachance', 'Lindqvist', 'Lockridge', 'Maddox', 'Marchetti', 'Mbeki', 'McAllister',
  'Merriweather', 'Moncrieff', 'Nakashima', 'Nygaard', 'Oduya', 'Okonkwo', 'Ollerton', 'Ostrowski', 'Pemberton', 'Penhaligon',
  'Quarles', 'Radcliffe', 'Ramasamy', 'Ravenscroft', 'Rehnquist', 'Rosenthal', 'Saltonstall', 'Sandoval', 'Seabrook', 'Sherwood',
  'Sinclair-Ray', 'Somerled', 'Strickland', 'Szabo', 'Tavistock', 'Thorne', 'Trevelyan', 'Underhill', 'Vanterpool', 'Varga',
  'Velasquez', 'Wainwright', 'Wexley', 'Whitcombe', 'Winterbourne', 'Yamashiro', 'Yardley', 'Zielinski', 'Zubiri', 'Oyelaran',
  'Brennagh', 'Castellanos', 'Driscoll', 'Esterhazy', 'Fontaine', 'Gutierrez-Lam', 'Holmqvist', 'Inglewood', 'Jessup', 'Kilbride',
];

const COMPANY_FIRST = [
  'Alderwick', 'Amberline', 'Ashgrove', 'Basalt', 'Bellhollow', 'Birchwater', 'Blue Heron', 'Bramblecrest', 'Brindle', 'Cairnfield',
  'Calderwood', 'Cedarmark', 'Cinderpath', 'Clearbrook', 'Copperleaf', 'Corvid', 'Cresthaven', 'Driftmoor', 'Duskwater', 'Eastwick',
  'Elmstead', 'Emberglow', 'Fallowmere', 'Fernvale', 'Flintridge', 'Foxhallow', 'Gildersleeve', 'Glasswing', 'Granite Bay', 'Greyfriar',
  'Harrowgate', 'Hartwell', 'Hazelmoor', 'Highmeadow', 'Hollowell', 'Ironvale', 'Juniper Hill', 'Kestrel', 'Kingsfold', 'Lamplighter',
  'Larkspur', 'Lindenhall', 'Lowmarsh', 'Marrowfield', 'Meadowlark', 'Millbrook', 'Moonrake', 'Mossgiel', 'Northfell', 'Oakhaven',
  'Oldcastle', 'Orchardine', 'Palisade', 'Pennywhistle', 'Pinehollow', 'Quarrystone', 'Quillfeather', 'Ravensmoor', 'Redfern', 'Ridgewell',
  'Rookwood', 'Rowanbrae', 'Saltmarsh', 'Sandpiper', 'Silverbirch', 'Skylark', 'Sorrel', 'Starling', 'Stonebridge', 'Summerhill',
  'Tamarack', 'Thistledown', 'Thornbury', 'Tidewater', 'Umberlee', 'Vantage Peak', 'Wexcombe', 'Whinstone', 'Willowmere', 'Wrenfield',
];

const COMPANY_SECOND = [
  'Analytics', 'Freight', 'Logistics', 'Fabrication', 'Instruments', 'Bioworks', 'Textiles', 'Software', 'Data Systems', 'Engineering',
  'Packaging', 'Hospitality', 'Environmental', 'Mechanical', 'Robotics', 'Supply', 'Coatings', 'Ceramics', 'Cartography', 'Telemetry',
  'Holdings', 'Partners', 'Consulting', 'Labs', 'Foods', 'Medical Devices', 'Outfitters', 'Print Works', 'Glassworks', 'Metals',
  'Energy', 'Water Systems', 'Imaging', 'Payroll Services', 'Cloud Services', 'Security', 'Staffing', 'Aviation', 'Marine', 'Health',
];

const COMPANY_SUFFIX = ['LLC', 'Inc.', 'Co.', 'Corporation', 'Ltd.', 'LLC', 'Inc.', 'LLP', 'Group, Inc.', 'Company'];

export const TOWNS = [
  ['Port Alder', 'OR', '973'], ['Wrenfield', 'OH', '440'], ['East Calloway', 'MN', '553'], ['Marrow Springs', 'CO', '805'],
  ['Halden Bay', 'WA', '982'], ['Fallow Creek', 'TX', '761'], ['Linden Cross', 'PA', '190'], ['Briarport', 'NC', '275'],
  ['Copper Flats', 'AZ', '852'], ['North Easton Mills', 'MA', '017'], ['Gilchrist Harbor', 'ME', '041'], ['Saltash Point', 'CA', '949'],
  ['Kestrel Ridge', 'UT', '840'], ['Pemberly Falls', 'NY', '128'], ['Ashworth', 'IL', '604'], ['Tamsin Valley', 'NM', '875'],
  ['Larchmont Heights', 'GA', '303'], ['Oriel Junction', 'MO', '641'], ['Bellmoor', 'WI', '535'], ['Cobble Hill Station', 'VT', '054'],
  ['Westharrow', 'MI', '491'], ['Dunmore Lake', 'ID', '838'], ['Quarry Bend', 'TN', '372'], ['Silverlode', 'NV', '894'],
];

export const STREETS = [
  'Brackenridge Avenue', 'Ostrander Road', 'Quillon Street', 'Harrow Lane', 'Tidewater Parkway', 'Calder Way', 'Fennel Court',
  'Wexley Boulevard', 'Old Mill Road', 'Juniper Loop', 'Lamplighter Drive', 'Saltmarsh Road', 'Cinder Street', 'Orchard Row',
  'Gristmill Lane', 'Basalt Drive', 'Kingsfold Avenue', 'Marrowfield Road', 'Pennant Street', 'Ridgewell Terrace',
  'Starling Way', 'Thornbury Place', 'Umber Street', 'Vantage Drive', 'Whinstone Road', 'Alder Court', 'Ferry Landing Road',
];

export function personName(rng, { middleInitial = false } = {}) {
  const first = rng.pick(FIRST_NAMES);
  const last = rng.pick(LAST_NAMES);
  return middleInitial ? `${first} ${String.fromCharCode(65 + rng.int(0, 25))}. ${last}` : `${first} ${last}`;
}

/// `count` distinct people.
export function people(rng, count, { exclude = [] } = {}) {
  const seen = new Set(exclude);
  const out = [];
  while (out.length < count) {
    const name = personName(rng);
    if (seen.has(name)) continue;
    seen.add(name);
    out.push(name);
  }
  return out;
}

export function companyName(rng, { suffix = true, second = null } = {}) {
  const base = `${rng.pick(COMPANY_FIRST)} ${second ?? rng.pick(COMPANY_SECOND)}`;
  return suffix ? `${base} ${rng.pick(COMPANY_SUFFIX)}` : base;
}

/// `count` distinct companies whose first words also differ, so no two
/// read as the same firm.
export function companies(rng, count, { exclude = [], second = null } = {}) {
  const usedFirst = new Set(exclude.map((name) => name.split(' ')[0]));
  const out = [];
  let guard = 0;
  while (out.length < count) {
    guard += 1;
    if (guard > 10000) throw new Error('company pool exhausted');
    const name = companyName(rng, { second: second ? rng.pick(second) : null });
    const first = name.split(' ')[0];
    if (usedFirst.has(first)) continue;
    usedFirst.add(first);
    out.push(name);
  }
  return out;
}

export function address(rng) {
  const [town, state, zip] = rng.pick(TOWNS);
  const number = rng.int(12, 9800);
  const street = rng.pick(STREETS);
  const suite = rng.chance(0.4) ? `, Suite ${rng.int(1, 9)}${rng.int(0, 4)}${rng.int(0, 9)}` : '';
  return {
    street: `${number} ${street}${suite}`,
    town,
    state,
    zip: `${zip}${rng.int(10, 99)}`,
    get line() {
      return `${this.street}, ${this.town}, ${this.state} ${this.zip}`;
    },
    get cityLine() {
      return `${this.town}, ${this.state} ${this.zip}`;
    },
  };
}

/// A telephone number in the 555-01xx range reserved for fiction.
export function phone(rng) {
  return `(${rng.int(201, 989)}) 555-01${rng.int(0, 9)}${rng.int(0, 9)}`;
}

export function digits(rng, count) {
  let out = '';
  for (let index = 0; index < count; index += 1) out += String(rng.int(0, 9));
  return out;
}

/// An identifier like "INV-20417" or "PO-88213".
export function identifier(rng, prefix, length = 5) {
  return `${prefix}-${String(rng.int(1, 9))}${digits(rng, length - 1)}`;
}

/// An email address on a fictional domain (.example is reserved).
export function email(name, domain) {
  const [first, ...rest] = name.toLowerCase().replace(/[^a-z ]/g, '').split(' ');
  return `${first}.${rest[rest.length - 1]}@${domain}`;
}

export function domainFor(company) {
  return `${company.toLowerCase().replace(/,? (llc|inc\.|co\.|corporation|ltd\.|llp|group, inc\.|company)$/, '').replace(/[^a-z]+/g, '-').replace(/^-|-$/g, '')}.example`;
}
