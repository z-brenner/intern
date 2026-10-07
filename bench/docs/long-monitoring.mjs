/// A one-hundred-page watershed monitoring report. The cover dates the
/// review draft the consultant submitted; the report's own date - the day
/// it was issued in final form - is stated once, in the certification on
/// page 88, after forty stations of field and laboratory results, each with
/// its own sampling dates. Every page is built from the stations' data, so
/// no two pages say the same thing.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold, structure } from '../lib/gold.mjs';
import { addDays, longDate, numericDate } from '../lib/format.mjs';
import { people } from '../lib/names.mjs';
import { digitalPdf, readingSnippets, result } from './common.mjs';
import { blocks, chain, expectPages, fillTo, tableBlocks } from './dense.mjs';

const ISSUER = 'Thornwick Basin Engineering, PLLC';
const CLIENT = 'Cobalt Basin Water Authority';
const LAB = 'Wrenfield Analytical Laboratories, Inc.';
const ECOLOGIST = 'Bramblecote Aquatic Ecology LLC';
const PROJECT = 'TBE-2025-0412';

/// Streams of the basin: name, what drains to it, its character.
const STREAMS = [
  ['Cobalt River', 'the main stem below Halverson Dam', 'large, regulated flow'], ['Fenwick Run', 'dairy pasture and hayfields', 'meandering, silt bed'],
  ['Larkspur Brook', 'forested state land', 'steep, cobble and boulder'], ['Mill Creek', 'the village of Ashby and its wastewater plant', 'channelized through town'],
  ['Otter Slide Brook', 'a ski area and second homes', 'high-gradient, snowmelt-driven'], ['Pennock Creek', 'row crops and a feedlot', 'incised, eroding banks'],
  ['Quarry Hollow Brook', 'an inactive granite quarry', 'clear, low conductivity'], ['Redwing Creek', 'wetlands and a beaver complex', 'slow, tannin-stained'],
  ['Sawyer Brook', 'a gravel pit and the county highway', 'riffle-pool, gravel bed'], ['Tamarack Creek', 'bog and conifer swamp', 'acidic, low pH'],
  ['Upton Mill Brook', 'orchards and a cider mill', 'small, spring-fed'], ['Vesper Creek', 'the regional airport and industrial park', 'piped in its upper reach'],
  ['Willet Brook', 'golf course and suburban lots', 'lawn-dominated riparian zone'], ['Yarrow Creek', 'mixed farms and woodlots', 'cool, shaded'],
];
const REACHES = ['upper', 'middle', 'lower'];
const MONTHS = ['01', '02', '03', '04', '05', '06', '07', '08', '09', '10', '11', '12'];

/// Ways a station's month-by-month record can read, so no two stations'
/// narratives are cut from one sentence.
const OPENINGS = [
  (s) => `${s.name} drains ${s.drains}; at the station the channel is ${s.character}.`,
  (s) => `The station on ${s.stream} sits ${s.km} km above its confluence, where the stream is ${s.character} and drains ${s.drains}.`,
  (s) => `Above station ${s.code}, ${s.stream} collects runoff from ${s.drains}; the reach is ${s.character}.`,
  (s) => `${s.code} samples ${s.stream} at river km ${s.km}, a ${s.character} reach fed by ${s.drains}.`,
];

function stationData(rng, index) {
  const [stream, drains, character] = STREAMS[index % STREAMS.length];
  const reach = REACHES[Math.floor(index / STREAMS.length) % REACHES.length];
  const code = `CB-${String(index + 1).padStart(2, '0')}`;
  const forest = rng.int(10, 90);
  const agriculture = rng.int(0, 100 - forest);
  const urban = 100 - forest - agriculture;
  const pressure = (agriculture + urban * 1.6) / 100;
  const samples = MONTHS.map((month, at) => {
    const day = String(rng.int(3, 26)).padStart(2, '0');
    const summer = Math.sin((at / 11) * Math.PI);
    const temperature = (2 + summer * 19 + rng.int(-20, 20) / 10).toFixed(1);
    const oxygen = (12.5 - summer * 5.5 - pressure * 1.5 + rng.int(-8, 8) / 10).toFixed(1);
    const ph = (6.6 + rng.int(0, 16) / 10 + (stream === 'Tamarack Creek' ? -1.2 : 0)).toFixed(1);
    const conductivity = String(Math.round(40 + pressure * 520 + rng.int(-30, 60)));
    const turbidity = (1 + pressure * 14 * rng.float() + (rng.chance(0.12) ? rng.int(20, 120) : 0)).toFixed(1);
    const ecoli = String(Math.round(10 + pressure * 300 * summer * rng.float() + (rng.chance(0.1) ? rng.int(300, 2400) : 0)));
    const nitrate = (0.1 + pressure * 3.2 * rng.float()).toFixed(2);
    const phosphorus = (0.008 + pressure * 0.12 * rng.float()).toFixed(3);
    const solids = String(Math.round(2 + pressure * 40 * rng.float()));
    const chloride = String(Math.round(4 + urban * 2.4 * rng.float() + (at < 3 ? urban : 0)));
    return { date: `2025-${month}-${day}`, temperature, oxygen, ph, conductivity, turbidity, ecoli, nitrate, phosphorus, solids, chloride };
  });
  return {
    code, stream, reach, drains, character,
    name: `${stream}, ${reach} reach`,
    km: (rng.int(2, 480) / 10).toFixed(1),
    area: (rng.int(30, 2600) / 10).toFixed(1),
    latitude: (44 + rng.int(1000, 9999) / 10000).toFixed(4),
    longitude: (72 + rng.int(1000, 9999) / 10000).toFixed(4),
    forest, agriculture, urban,
    samples,
    index: rng.int(28, 92),
    taxa: rng.int(14, 46),
    ept: rng.int(3, 24),
    prior: { ecoli: rng.int(40, 600), phosphorus: (rng.int(8, 140) / 1000).toFixed(3) },
    // Most visits note nothing out of the ordinary.
    observations: MONTHS.map(() => [WEATHER[rng.int(0, WEATHER.length - 1)], FLOW[rng.int(0, FLOW.length - 1)], rng.chance(0.3) ? OBSERVATIONS[rng.int(0, OBSERVATIONS.length - 1)] : '-']),
    discharge: MONTHS.map(() => [(rng.int(40, 480) / 100).toFixed(2), (rng.int(5, 2400) / 10).toFixed(1)]),
    taxaFound: TAXA.filter(() => rng.chance(0.42)).map(([genus, group, tolerance]) => [genus, group, tolerance, rng.int(1, 140)]),
    habitat: HABITAT.map((parameter) => [parameter, rng.int(4, 20)]),
    opening: OPENINGS[index % OPENINGS.length],
  };
}

/// Benthic genera and their pollution tolerance (0 intolerant, 10 tolerant).
const TAXA = [
  ['Baetis', 'mayfly', 5], ['Epeorus', 'mayfly', 0], ['Ephemerella', 'mayfly', 1], ['Isonychia', 'mayfly', 3], ['Stenonema', 'mayfly', 3],
  ['Paraleptophlebia', 'mayfly', 1], ['Acroneuria', 'stonefly', 0], ['Leuctra', 'stonefly', 0], ['Isoperla', 'stonefly', 2], ['Pteronarcys', 'stonefly', 0],
  ['Hydropsyche', 'caddisfly', 4], ['Cheumatopsyche', 'caddisfly', 5], ['Rhyacophila', 'caddisfly', 0], ['Glossosoma', 'caddisfly', 0], ['Chimarra', 'caddisfly', 4],
  ['Optioservus', 'riffle beetle', 4], ['Stenelmis', 'riffle beetle', 5], ['Psephenus', 'water penny', 4], ['Simulium', 'black fly', 6], ['Antocha', 'crane fly', 3],
  ['Tipula', 'crane fly', 4], ['Chironomidae', 'midge', 6], ['Polypedilum', 'midge', 6], ['Hyalella', 'scud', 8], ['Gammarus', 'scud', 6],
  ['Physa', 'snail', 8], ['Oligochaeta', 'worm', 8], ['Corydalus', 'hellgrammite', 4], ['Boyeria', 'dragonfly', 2], ['Atherix', 'snipe fly', 2],
];
const HABITAT = ['Epifaunal substrate', 'Embeddedness', 'Velocity and depth regime', 'Sediment deposition', 'Channel flow status', 'Channel alteration', 'Riffle frequency', 'Bank stability', 'Bank vegetation', 'Riparian zone width'];
const WEATHER = ['clear', 'overcast', 'light rain', 'snow showers', 'fog clearing', 'partly sunny', 'drizzle', 'after heavy rain'];
const FLOW = ['base flow', 'elevated', 'storm runoff', 'ice cover at margins', 'low, braided', 'snowmelt'];
const OBSERVATIONS = [
  'beaver dam rebuilt 40 m upstream', 'fresh bank slump on the left bank', 'cattle tracks at the ford', 'algae mats on riffle cobbles', 'odor of manure noted',
  'road sand deposited in the pool', 'foam line below the culvert', 'brook trout seen in the plunge pool', 'new culvert installed upstream by the town',
  'tree down across the channel', 'construction silt fence failing on the right bank', 'oil sheen near the storm outfall, reported to the town', 'water clear to the bottom',
  'leaf packs abundant', 'heron feeding at the riffle', 'turbid after field spreading upstream', 'snowplow berm at the bridge', 'gravel bar reworked by high water',
  'trash rack at the culvert cleared', 'no unusual conditions',
];

/// The water quality criteria the results are compared with.
const CRITERIA = [
  ['E. coli', 'single sample 235 MPN/100 mL; geometric mean 126', 'Class B(2) waters'], ['Dissolved oxygen', 'not less than 6.0 mg/L (cold water) or 5.0 mg/L (warm water)', 'aquatic life use'],
  ['pH', '6.5 to 8.5', 'aquatic life use'], ['Turbidity', 'not more than 10 NTU (cold water) or 25 NTU (warm water) above background', 'aquatic life use'],
  ['Total phosphorus', 'nutrient threshold 0.027 mg/L (small high-gradient streams)', 'nutrient criteria'], ['Chloride', 'chronic 230 mg/L', 'aquatic life use'],
  ['Nitrate-nitrogen', '10 mg/L (drinking water source protection)', 'water supply use'],
];

const METHODS = [
  'Stations were sampled once a month from January through December 2025, on the dates shown in each station table, between 07:00 and 15:00. Where ice prevented access, the sample was taken at the nearest open water within 50 m and the location recorded.',
  'Temperature, dissolved oxygen, pH, specific conductance and turbidity were measured in the field with a multiparameter sonde calibrated each morning against certified standards; calibration records are in Appendix A. Grab samples for the laboratory parameters were collected at mid-channel by the hand-dip method, preserved on ice, and delivered under chain of custody to the laboratory within the holding times of the methods.',
  `Laboratory analyses were performed by ${LAB}, accredited under the National Environmental Laboratory Accreditation Program: E. coli by enzyme substrate (Colilert, SM 9223 B), nitrate-nitrogen by ion chromatography (EPA 300.0), total phosphorus by persulfate digestion (SM 4500-P E), total suspended solids by SM 2540 D, and chloride by EPA 300.0.`,
  `Benthic macroinvertebrates were collected in late September with a kick net from four riffle locations per station, composited, and identified to genus by ${ECOLOGIST}. The biological index combines taxa richness, the number of mayfly, stonefly and caddisfly (EPT) taxa, and a pollution tolerance index into a score from 0 to 100; scores of 60 and above meet the aquatic life use.`,
  'Results below the reporting limit are shown as half the reporting limit for the calculation of means and as "<RL" in the narrative. Geometric means of E. coli are calculated over the samples of May through October, the recreation season.',
];

export function watershedMonitoringReport100() {
  const id = 'watershed-monitoring-report-100p';
  const rng = Rng.from(id);
  const issued = '2026-06-19';
  const draft = '2026-03-03';
  const board = '2026-07-08';
  const stations = Array.from({ length: 40 }, (_, index) => stationData(rng.fork(`station-${index}`), index));
  const flow = new Flow({
    face: 'serif', fontSize: 10, leading: 1.35, margins: { top: 64, bottom: 66 }, keep: [ISSUER, CLIENT, LAB, ECOLOGIST],
    header: (page, { number }) => {
      if (number > 1) page.text(72, 44, `Annual Watershed Monitoring Report, Monitoring Year 2025 - Project ${PROJECT}`, { face: 'sans', size: 7.5, grey: 0.4 });
    },
    footer: (page, { number }) => page.textRight(540, 760, String(number), { face: 'sans', size: 8, grey: 0.4 }),
  });
  const pick = rng.fork('report');
  const exceeds = (station, test) => station.samples.filter(test);
  const bacteria = stations.filter((station) => exceeds(station, (sample) => Number(sample.ecoli) > 235).length > 0);
  const nutrient = stations.filter((station) => exceeds(station, (sample) => Number(sample.phosphorus) > 0.027).length > 6);
  const biology = stations.filter((station) => station.index < 60);

  // Cover and contents.
  flow.heading('ANNUAL WATERSHED MONITORING REPORT', { level: 1, align: 'center', size: 17, before: 60 });
  flow.paragraph('Cobalt River Basin - Monitoring Year 2025', { face: 'sans', size: 12, align: 'center', after: 30 });
  flow.paragraph(`Prepared for ${CLIENT}, 900 Reservoir Road, Ashby, Vermont 05032`, { align: 'center', size: 11 });
  flow.paragraph(`Prepared by ${ISSUER}, 61 Foundry Street, Montpelier, Vermont 05602`, { align: 'center', size: 11, after: 24 });
  flow.paragraph(`Project ${PROJECT}. Review draft submitted to the Authority on ${longDate(draft)}.`, { face: 'sans', size: 10, align: 'center', after: 30 });
  flow.paragraph('CONTENTS', { face: 'sans-bold', size: 10, after: 4 });
  for (const [chapter, title] of [['1', 'Summary'], ['2', 'Monitoring program and methods'], ['3', 'Results by station'], ['4', 'Conclusions and recommendations'], ['5', 'Quality assurance and quality control'], ['6', 'Certification'], ['A', 'Appendix A - Instrument calibration log'], ['B', 'Appendix B - Field personnel'], ['C', 'Appendix C - Chain of custody log']]) {
    flow.paragraph(`${chapter}. ${title}`, { size: 10, indent: 18, after: 2 });
  }
  flow.pageBreak();

  // 1. Summary.
  flow.heading('1. SUMMARY', { level: 2 });
  flow.paragraph(`${ISSUER} monitored ${stations.length} stations on ${STREAMS.length} streams of the Cobalt River basin for ${CLIENT} through the 2025 monitoring year. Each station was sampled monthly for field and laboratory parameters and once for benthic macroinvertebrates. This report presents the results station by station, compares them with the state water quality criteria, and recommends follow-up.`);
  flow.paragraph(`E. coli exceeded the single-sample criterion at least once at ${bacteria.length} stations, mostly in July and August after storms; phosphorus exceeded the nutrient threshold in more than half the samples at ${nutrient.length} stations, all draining agricultural or developed land; and ${biology.length} stations scored below 60 on the biological index. Dissolved oxygen and pH met the criteria in almost every sample, except the naturally acidic Tamarack Creek.`);
  flow.paragraph('Compared with 2024, geometric mean E. coli fell at most stations draining pasture where the Authority\'s fencing cost-share was taken up, and rose below the Ashby wastewater plant during its outfall replacement. Chloride continued to rise at the stations below the county highway and the airport, peaking in the January to March samples.');
  // 2. Methods and criteria.
  flow.heading('2. MONITORING PROGRAM AND METHODS', { level: 2 });
  METHODS.forEach((text, index) => flow.paragraph(`2.${index + 1} ${text}`));
  flow.table([{ header: 'Parameter', width: 0.22 }, { header: 'Criterion', width: 0.5 }, { header: 'Use protected', width: 0.28 }], CRITERIA, { size: 8.5 });
  flow.table([{ header: 'Station', width: 0.1 }, { header: 'Stream and reach', width: 0.32 }, { header: 'River km', width: 0.12, align: 'right' }, { header: 'Drainage (km2)', width: 0.16, align: 'right' }, { header: 'Latitude', width: 0.15, align: 'right' }, { header: 'Longitude', width: 0.15, align: 'right' }],
    stations.map((station) => [station.code, station.name, station.km, station.area, station.latitude, `-${station.longitude}`]), { size: 8.5 });

  // 3. Results by station.
  flow.heading('3. RESULTS BY STATION', { level: 2 });
  const stationBlocks = stations.flatMap((station, number) => [
    () => {
      flow.paragraph(`3.${station.code.slice(3).replace(/^0/, '')} Station ${station.code} - ${station.name}`, { face: 'sans-bold', size: 10.5, before: 6, after: 3, keepWithNext: 60 });
      flow.paragraph(`${station.opening(station)} Land cover in the ${station.area} km2 drainage is ${station.forest}% forest, ${station.agriculture}% agriculture and ${station.urban}% developed.`, { size: 9.5 });
    },
    () => flow.table([{ header: 'Sampled', width: 0.16 }, { header: 'Temp (C)', width: 0.14, align: 'right' }, { header: 'DO (mg/L)', width: 0.14, align: 'right' }, { header: 'pH', width: 0.1, align: 'right' }, { header: 'Cond. (uS/cm)', width: 0.18, align: 'right' }, { header: 'Turbidity (NTU)', width: 0.28, align: 'right' }],
      station.samples.map((sample) => [numericDate(sample.date), sample.temperature, sample.oxygen, sample.ph, sample.conductivity, sample.turbidity]), { size: 8.5 }),
    () => flow.table([{ header: 'Sampled', width: 0.16 }, { header: 'E. coli (MPN)', width: 0.17, align: 'right' }, { header: 'Nitrate-N', width: 0.15, align: 'right' }, { header: 'Total P', width: 0.15, align: 'right' }, { header: 'TSS', width: 0.12, align: 'right' }, { header: 'Chloride', width: 0.25, align: 'right' }],
      station.samples.map((sample) => [numericDate(sample.date), sample.ecoli, sample.nitrate, sample.phosphorus, sample.solids, sample.chloride]), { size: 8.5 }),
    () => flow.table([{ header: 'Sampled', width: 0.15 }, { header: 'Weather', width: 0.17 }, { header: 'Flow', width: 0.19 }, { header: 'Stage (ft)', width: 0.11, align: 'right' }, { header: 'Discharge (cfs)', width: 0.14, align: 'right' }, { header: 'Observation', width: 0.24 }],
      station.samples.map((sample, month) => [numericDate(sample.date), ...station.observations[month].slice(0, 2), ...station.discharge[month], station.observations[month][2]]), { size: 8 }),
    () => flow.table([{ header: 'Genus', width: 0.26 }, { header: 'Group', width: 0.26 }, { header: 'Tolerance', width: 0.2, align: 'right' }, { header: 'Count', width: 0.28, align: 'right' }],
      station.taxaFound.map(([genus, group, tolerance, count]) => [genus, group, String(tolerance), String(count)]), { size: 8 }),
    () => flow.table([{ header: 'Habitat parameter', width: 0.6 }, { header: 'Score (0-20)', width: 0.4, align: 'right' }],
      [...station.habitat.map(([parameter, score]) => [parameter, String(score)]), ['Total', String(station.habitat.reduce((sum, [, score]) => sum + score, 0))]], { size: 8 }),
    () => {
      const high = exceeds(station, (sample) => Number(sample.ecoli) > 235);
      const phosphorus = exceeds(station, (sample) => Number(sample.phosphorus) > 0.027).length;
      const peak = station.samples.reduce((best, sample) => (Number(sample.turbidity) > Number(best.turbidity) ? sample : best));
      const low = station.samples.reduce((best, sample) => (Number(sample.oxygen) < Number(best.oxygen) ? sample : best));
      flow.paragraph(SUMMARIES[(number + Math.floor(number / OPENINGS.length)) % SUMMARIES.length]({ station, high, phosphorus, peak, low }), { size: 9.5 });
    },
  ]);
  let stationIndex = 0;
  const results = () => {
    if (stationIndex >= stationBlocks.length) return false;
    stationBlocks[stationIndex]();
    stationIndex += 1;
    return true;
  };
  // 4. Conclusions, then the quality assurance tables to page 88.
  const conclusions = blocks(RECOMMENDATIONS, (text, index) => {
    if (index === 0) {
      flow.heading('4. CONCLUSIONS AND RECOMMENDATIONS', { level: 2 });
      flow.paragraph(`The ${stations.length} stations fall into three groups: forested headwaters that meet every criterion; agricultural streams with recurring bacteria and nutrient exceedances; and developed streams where chloride and stormwater turbidity dominate. We recommend:`);
    }
    flow.paragraph(`4.${index + 1} ${text}`, { indent: 12 });
  });
  const duplicates = stations.flatMap((station) => [pick.int(0, 11), pick.int(0, 11)].map((month) => {
    const sample = station.samples[month];
    const replicate = (Number(sample.phosphorus) * pick.int(88, 112) / 100).toFixed(3);
    const difference = Math.abs(Number(sample.phosphorus) - Number(replicate)) / ((Number(sample.phosphorus) + Number(replicate)) / 2) * 100;
    return [station.code, numericDate(sample.date), 'Total P', sample.phosphorus, replicate, `${difference.toFixed(1)}%`, difference <= 20 ? 'pass' : 'qualified J'];
  }));
  const qa = tableBlocks(flow, [{ header: 'Station', width: 0.11 }, { header: 'Sampled', width: 0.15 }, { header: 'Parameter', width: 0.14 }, { header: 'Sample', width: 0.13, align: 'right' }, { header: 'Duplicate', width: 0.14, align: 'right' }, { header: 'RPD', width: 0.12, align: 'right' }, { header: 'Result', width: 0.21 }], duplicates,
    { title: '5. QUALITY ASSURANCE AND QUALITY CONTROL', intro: 'Field duplicates were collected at about one in ten samples. The relative percent difference (RPD) objective is 20% for nutrients; results outside it are qualified as estimates (J). Field blanks, laboratory method blanks and matrix spikes met their objectives except as noted in the laboratory reports.', chunk: 8 });
  const blanks = tableBlocks(flow, [{ header: 'Blank', width: 0.14 }, { header: 'Collected', width: 0.16 }, { header: 'Station', width: 0.12 }, { header: 'E. coli', width: 0.12, align: 'right' }, { header: 'Total P', width: 0.14, align: 'right' }, { header: 'Nitrate-N', width: 0.14, align: 'right' }, { header: 'Result', width: 0.18 }],
    stations.flatMap((station, index) => (index % 2 ? [] : [station.samples[pick.int(0, 11)]]).map((sample) => [`FB-${station.code.slice(3)}`, numericDate(sample.date), station.code, '<1', pick.chance(0.9) ? '<0.005' : '0.006', pick.chance(0.95) ? '<0.02' : '0.03', pick.chance(0.9) ? 'clean' : 'trace; batch reviewed'])),
    { intro: 'Table 5-2. Field blanks: laboratory-grade water carried to the station and poured into sample bottles, to detect contamination in handling.', chunk: 8 });
  const spikes = tableBlocks(flow, [{ header: 'Batch', width: 0.14 }, { header: 'Parameter', width: 0.2 }, { header: 'Spike added', width: 0.16, align: 'right' }, { header: 'Recovered', width: 0.16, align: 'right' }, { header: 'Recovery', width: 0.14, align: 'right' }, { header: 'Limits', width: 0.2 }],
    MONTHS.flatMap((month) => [['Total P', 0.1], ['Nitrate-N', 1.0], ['Chloride', 10]].map(([parameter, added]) => {
      const recovery = pick.int(84, 114);
      return [`2025-${month}`, parameter, String(added), (added * recovery / 100).toFixed(parameter === 'Total P' ? 3 : 2), `${recovery}%`, '80-120%'];
    })),
    { intro: 'Table 5-3. Laboratory matrix spikes by monthly batch.', chunk: 9 });
  const methodBlanks = tableBlocks(flow, [{ header: 'Batch', width: 0.14 }, { header: 'Parameter', width: 0.2 }, { header: 'Reporting limit', width: 0.18, align: 'right' }, { header: 'Method blank', width: 0.18, align: 'right' }, { header: 'Samples in batch', width: 0.14, align: 'right' }, { header: 'Result', width: 0.16 }],
    MONTHS.flatMap((month) => [['E. coli', '1 MPN'], ['Nitrate-N', '0.02 mg/L'], ['Total P', '0.005 mg/L'], ['TSS', '1 mg/L'], ['Chloride', '0.5 mg/L']].map(([parameter, limit]) => {
      const detected = pick.chance(0.06);
      return [`2025-${month}`, parameter, limit, detected ? `${limit.replace(/^[\d.]+/, (value) => (Number(value) * 1.4).toFixed(value.includes('.') ? 3 : 0))} (detected)` : '<RL', String(pick.int(36, 44)), detected ? 'batch results flagged B' : 'clean'];
    })),
    { intro: 'Table 5-4. Laboratory method blanks by monthly batch, with the reporting limit of each method.', chunk: 10 });
  // The year at each station in figures, after the station sections.
  const parameters = [['Temperature (C)', 'temperature', null], ['Dissolved oxygen (mg/L)', 'oxygen', (value) => value < 6], ['E. coli (MPN/100 mL)', 'ecoli', (value) => value > 235], ['Total P (mg/L)', 'phosphorus', (value) => value > 0.027], ['Chloride (mg/L)', 'chloride', (value) => value > 230]];
  const statistics = tableBlocks(flow, [{ header: 'Station', width: 0.1 }, { header: 'Parameter', width: 0.27 }, { header: 'Minimum', width: 0.12, align: 'right' }, { header: 'Median', width: 0.12, align: 'right' }, { header: 'Mean', width: 0.12, align: 'right' }, { header: 'Maximum', width: 0.12, align: 'right' }, { header: 'Outside criterion', width: 0.15, align: 'right' }],
    stations.flatMap((station) => parameters.map(([label, key, outside]) => {
      const values = station.samples.map((sample) => Number(sample[key])).sort((a, b) => a - b);
      const places = key === 'phosphorus' ? 3 : key === 'ecoli' || key === 'chloride' ? 0 : 1;
      const median = ((values[5] + values[6]) / 2).toFixed(places);
      const mean = (values.reduce((sum, value) => sum + value, 0) / values.length).toFixed(places);
      return [station.code, label, values[0].toFixed(places), median, mean, values[11].toFixed(places), outside ? String(values.filter(outside).length) : '-'];
    })),
    { title: 'ANNUAL STATISTICS BY STATION', intro: 'Table 3-1. The twelve monthly results at each station in figures: minimum, median, mean and maximum, and the number of samples outside the criterion in Section 2.', chunk: 10, size: 8 });
  fillTo(flow, 88, chain(results, statistics, conclusions, qa, blanks, spikes, methodBlanks), { id });
  if (stationIndex < stationBlocks.length) throw new Error(`${id}: only ${Math.floor(stationIndex / 7)} of ${stations.length} stations fit before the certification`);

  // 6. Certification, on page 88: the report's own date.
  const [engineer, scientist, reviewer] = people(rng.fork('certification'), 3);
  flow.heading('6. CERTIFICATION', { level: 2, before: 6 });
  flow.paragraph(`This report was prepared by ${ISSUER} for the exclusive use of ${CLIENT} under the Authority's monitoring agreement. It was issued in final form on ${longDate(issued)}, after the Authority's comments on the review draft were addressed, and supersedes that draft. It will be presented to the Authority's Board of Directors at its meeting of ${longDate(board)}.`);
  flow.paragraph(`I certify that the monitoring described in this report was performed under my direction, that the field and laboratory data were reviewed against the quality assurance project plan, and that the report fairly presents the results. - ${engineer}, P.E., Vermont License No. ${pick.int(7000, 9999)}, Principal Engineer. Field program: ${scientist}, Senior Scientist. Technical review: ${reviewer}.`);

  // Appendices to page 100.
  const crew = people(rng.fork('crew'), 8, { exclude: [engineer, scientist, reviewer] });
  const custody = stations.flatMap((station) => station.samples.map((sample, month) => [`${station.code}-${MONTHS[month]}`, numericDate(sample.date), `${String(pick.int(7, 14)).padStart(2, '0')}:${String(pick.int(0, 59)).padStart(2, '0')}`, crew[pick.int(0, crew.length - 1)].split(' ').map((part) => part[0]).join(''), `${(pick.int(10, 58) / 10).toFixed(1)} C`, numericDate(addDays(sample.date, pick.int(0, 1)))]));
  const custodyLog = tableBlocks(flow, [{ header: 'Sample ID', width: 0.17 }, { header: 'Collected', width: 0.17 }, { header: 'Time', width: 0.11 }, { header: 'By', width: 0.1 }, { header: 'Cooler', width: 0.15, align: 'right' }, { header: 'Received by lab', width: 0.3 }], custody,
    { title: 'APPENDIX C - CHAIN OF CUSTODY LOG', intro: `Grab samples delivered to ${LAB}, in station order, with the collector's initials and the cooler temperature on receipt (objective: above 0 and not more than 6 C). The pages that follow are an extract; the complete log is kept in the project file.`, chunk: 10 });
  const calibrations = MONTHS.flatMap((month) => ['Sonde A (serial 21K104)', 'Sonde B (serial 21K118)', 'Turbidimeter (serial T-5520)'].map((instrument) => [`2025-${month}`, instrument, `${(pick.int(980, 1020) / 10).toFixed(1)}%`, `${(7 + pick.int(-4, 4) / 100).toFixed(2)}`, `${pick.int(1405, 1420)}`, pick.chance(0.08) ? 'recalibrated' : 'within limits']));
  const calibrationLog = tableBlocks(flow, [{ header: 'Month', width: 0.12 }, { header: 'Instrument', width: 0.32 }, { header: 'DO check', width: 0.13, align: 'right' }, { header: 'pH 7 check', width: 0.13, align: 'right' }, { header: 'Cond. 1413', width: 0.13, align: 'right' }, { header: 'Result', width: 0.17 }], calibrations,
    { title: 'APPENDIX A - INSTRUMENT CALIBRATION LOG', intro: 'Post-sampling checks against standards, summarized by month; daily records are kept in the project file.', chunk: 9 });
  const personnel = blocks(crew, (person, index) => {
    if (index === 0) flow.heading('APPENDIX B - FIELD PERSONNEL', { level: 2 });
    flow.paragraph(`${person}: ${pick.pick(['field technician', 'staff scientist', 'hydrologist', 'GIS analyst', 'data manager'])}; ${pick.int(1, 18)} years of experience; trained in swiftwater safety and the quality assurance project plan; ${pick.int(20, 140)} sampling events in 2025.`, { size: 9.5 });
  });
  // The custody log, the longest, comes last.
  fillTo(flow, 100, chain(calibrationLog, personnel, custodyLog), { id, room: 80 });
  flow.paragraph(`End of report. Data files for every station are available from ${ISSUER} on request, in the format of the state monitoring database.`, { size: 9, before: 6 });

  const pages = flow.finish();
  if (pages.length !== 100) throw new Error(`${id}: expected 100 pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  expectPages(id, text, longDate(issued), [88]);
  expectPages(id, text, longDate(draft), [1]);
  const lines = text.flatMap((page) => page.split('\n'));
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'One-hundred-page watershed monitoring report dated in its certification on page 88',
    kind: 'report',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_100', 'middle_fact', 'competing_dates', 'irrelevant_names', 'information_dense', 'table'],
    notes: `The cover dates the review draft (${longDate(draft)}); the report's own date, the day it was issued in final form (${longDate(issued)}), is stated once, in the certification on page 88, beside the board meeting it will go to (${longDate(board)}). Between them are ${stations.length} stations of monthly results, each with twelve 2025 sampling dates. The laboratory and the benthic subconsultant are named but are not parties; the report is the consultant's, for the Authority.`,
    structure: structure({
      readingOrder: readingSnippets(lines, 10),
    }),
    recording: 'pending',
    gold: gold({
      type: 'Watershed Monitoring Report',
      acceptableTypes: ['Annual Watershed Monitoring Report', 'Water Quality Monitoring Report', 'Monitoring Report', 'Annual Monitoring Report'],
      date: issued,
      role: 'issuance',
      forbiddenDates: [[draft, 'review draft submitted'], [board, 'board meeting the report will go to']],
      parties: [CLIENT],
      relation: 'for',
      acceptablePartySets: [{ parties: [ISSUER], relation: 'from' }],
      roles: [[CLIENT, 'subject'], [CLIENT, 'client'], [CLIENT, 'customer'], [ISSUER, 'issuer'], [ISSUER, 'contractor'], [ISSUER, 'provider']],
      forbiddenParties: [[LAB, 'analytical laboratory'], [ECOLOGIST, 'benthic subconsultant']],
      facts: [['Cobalt River'], ['E. coli', 'bacteria'], ['phosphorus', 'nutrient']],
      subjectTerms: ['watershed', 'water quality', 'monitoring'],
      readiness: 'ready',
      dateText: [longDate(issued)],
      dateAnchor: 'issued in final form on',
      typeText: ['ANNUAL WATERSHED MONITORING REPORT'],
      identifierText: [PROJECT],
    }),
  });
}

/// The paragraph closing each station's results, in several wordings so
/// that forty stations do not repeat one sentence.
const SUMMARIES = [
  ({ station, high, phosphorus, peak }) => `${high.length ? `E. coli exceeded 235 MPN/100 mL on ${high.map((sample) => numericDate(sample.date)).join(', ')}` : 'E. coli met the single-sample criterion in every sample'}. Total phosphorus was above the 0.027 mg/L threshold in ${phosphorus} of 12 samples, against a 2024 mean of ${station.prior.phosphorus} mg/L. Turbidity peaked at ${peak.turbidity} NTU on ${numericDate(peak.date)}. The benthic community (${station.taxa} taxa, ${station.ept} of them EPT) ${station.index >= 60 ? 'meets' : 'falls short of'} the aquatic life use with an index of ${station.index}.`,
  ({ station, high, phosphorus, low }) => `The September kick samples held ${station.taxa} genera, ${station.ept} mayflies, stoneflies or caddisflies among them, for an index score of ${station.index}${station.index >= 60 ? ', a pass' : ', below the passing score of 60'}. Oxygen was lowest on ${numericDate(low.date)} at ${low.oxygen} mg/L. ${phosphorus ? `Phosphorus ran over the nutrient threshold ${phosphorus} ${phosphorus > 1 ? 'times' : 'time'}` : 'Phosphorus never reached the nutrient threshold'}; the 2024 mean was ${station.prior.phosphorus} mg/L. ${high.length ? `Bacteria counts broke the single-sample limit ${high.length} ${high.length > 1 ? 'times' : 'time'}, the worst ${Math.max(...high.map((sample) => Number(sample.ecoli)))} MPN.` : 'No sample exceeded the bacteria limit.'}`,
  ({ station, high, phosphorus, peak }) => `${high.length > 2 ? `With ${high.length} bacteria exceedances, ${station.code} is among the stations to watch` : high.length ? `${station.code} had ${high.length === 1 ? 'a single bacteria exceedance' : 'two bacteria exceedances'}` : `${station.code} had no bacteria exceedance`}; its 2024 geometric mean was ${station.prior.ecoli} MPN/100 mL. Nutrients: ${phosphorus} of twelve phosphorus results over 0.027 mg/L. The highest turbidity, ${peak.turbidity} NTU, came on ${numericDate(peak.date)}. Biology ${station.index >= 60 ? 'supports' : 'does not support'} the aquatic life use (index ${station.index}; ${station.taxa} taxa; EPT ${station.ept}).`,
  ({ station, high, phosphorus, low, peak }) => `Dissolved oxygen bottomed out at ${low.oxygen} mg/L (${numericDate(low.date)}) and turbidity topped out at ${peak.turbidity} NTU (${numericDate(peak.date)}). ${phosphorus > 6 ? `Phosphorus exceeded the threshold in most samples (${phosphorus} of 12)` : `Phosphorus exceeded the threshold in ${phosphorus} of 12 samples`}, ${Number(station.prior.phosphorus) > 0.027 ? 'as it did on average in 2024' : 'though the 2024 average was under it'}. ${high.length ? `E. coli was high on ${high.map((sample) => numericDate(sample.date)).join(' and ')}.` : 'E. coli stayed under 235 MPN all year.'} An index of ${station.index} from ${station.taxa} taxa (${station.ept} EPT) ${station.index >= 60 ? 'meets' : 'misses'} the biological criterion.`,
];

const RECOMMENDATIONS = [
  'Extend the Authority\'s fencing and stream-crossing cost-share to the Pennock Creek and Fenwick Run drainages, where cattle access coincides with the highest E. coli counts.',
  'Ask the Ashby wastewater plant to sample its new outfall weekly through the 2026 recreation season and share the results with the Authority.',
  'Add two stations on Vesper Creek above and below the airport deicing pad to locate the chloride source, and sample them in January, February and March.',
  'Work with the county highway department to calibrate salt spreaders and to pilot brine pre-wetting on the segment above Sawyer Brook.',
  'Continue monthly sampling at every station; reduce the forested headwater stations to quarterly sampling if 2026 confirms they meet every criterion.',
  'Repeat the benthic survey at the stations scoring below 60 in 2026 to confirm the scores before listing any reach as impaired.',
  'Install continuous temperature loggers at the shaded Yarrow Creek and Larkspur Brook stations, which support wild brook trout.',
  'Publish the station results on the Authority\'s website each quarter, in the format the state monitoring database accepts.',
];
