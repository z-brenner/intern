/// Calendar and number formatting without the platform's locale.
///
/// `toLocaleString` and `Intl` depend on the ICU data Node was built with, so
/// the same document could print "4,812.50" on one machine and "4 812,50" on
/// another. Everything here is plain integer arithmetic over ISO dates
/// (`YYYY-MM-DD`) and integer cents.

export const MONTHS = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
];
export const WEEKDAYS = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

function pad(value, width = 2) {
  return String(value).padStart(width, '0');
}

function parse(iso) {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  if (!match) throw new Error(`not an ISO date: ${iso}`);
  return [Number(match[1]), Number(match[2]), Number(match[3])];
}

/// Days since 1970-01-01 (Howard Hinnant's days_from_civil).
export function toDays(iso) {
  let [year, month, day] = parse(iso);
  year -= month <= 2 ? 1 : 0;
  const era = Math.floor(year / 400);
  const yearOfEra = year - era * 400;
  const dayOfYear = Math.floor((153 * (month + (month > 2 ? -3 : 9)) + 2) / 5) + day - 1;
  const dayOfEra = yearOfEra * 365 + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100) + dayOfYear;
  return era * 146097 + dayOfEra - 719468;
}

export function fromDays(days) {
  const shifted = days + 719468;
  const era = Math.floor(shifted / 146097);
  const dayOfEra = shifted - era * 146097;
  const yearOfEra = Math.floor((dayOfEra - Math.floor(dayOfEra / 1460) + Math.floor(dayOfEra / 36524) - Math.floor(dayOfEra / 146096)) / 365);
  const dayOfYear = dayOfEra - (365 * yearOfEra + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100));
  const monthPrime = Math.floor((5 * dayOfYear + 2) / 153);
  const day = dayOfYear - Math.floor((153 * monthPrime + 2) / 5) + 1;
  const month = monthPrime + (monthPrime < 10 ? 3 : -9);
  const year = yearOfEra + era * 400 + (month <= 2 ? 1 : 0);
  return `${pad(year, 4)}-${pad(month)}-${pad(day)}`;
}

export function isRealDate(iso) {
  try {
    return fromDays(toDays(iso)) === iso;
  } catch {
    return false;
  }
}

export function addDays(iso, count) {
  return fromDays(toDays(iso) + count);
}

/// Adds calendar months, clamping the day to the target month's length the
/// way leases and loan schedules do (January 31 + 1 month = February 28).
export function addMonths(iso, count) {
  const [year, month, day] = parse(iso);
  const index = year * 12 + (month - 1) + count;
  const targetYear = Math.floor(index / 12);
  const targetMonth = (index % 12) + 1;
  const length = daysInMonth(targetYear, targetMonth);
  return `${pad(targetYear, 4)}-${pad(targetMonth)}-${pad(Math.min(day, length))}`;
}

export function daysInMonth(year, month) {
  const next = month === 12 ? `${pad(year + 1, 4)}-01-01` : `${pad(year, 4)}-${pad(month + 1)}-01`;
  return toDays(next) - toDays(`${pad(year, 4)}-${pad(month)}-01`);
}

export function endOfMonth(iso) {
  const [year, month] = parse(iso);
  return `${pad(year, 4)}-${pad(month)}-${pad(daysInMonth(year, month))}`;
}

/// 0 = Sunday.
export function weekday(iso) {
  return ((toDays(iso) % 7) + 7 + 4) % 7;
}

/// The next weekday on or after the date (Saturday and Sunday roll forward).
export function businessDayOnOrAfter(iso) {
  let date = iso;
  while (weekday(date) === 0 || weekday(date) === 6) date = addDays(date, 1);
  return date;
}

export function year(iso) {
  return parse(iso)[0];
}

/// "March 4, 2026"
export function longDate(iso) {
  const [y, m, d] = parse(iso);
  return `${MONTHS[m - 1]} ${d}, ${y}`;
}

/// "4 March 2026"
export function dayMonthYear(iso) {
  const [y, m, d] = parse(iso);
  return `${d} ${MONTHS[m - 1]} ${y}`;
}

/// "Mar 4, 2026"
export function shortMonthDate(iso) {
  const [y, m, d] = parse(iso);
  return `${MONTHS[m - 1].slice(0, 3)} ${d}, ${y}`;
}

/// "04-Mar-2026", the way ledgers and maintenance logs print dates.
export function ledgerDate(iso) {
  const [y, m, d] = parse(iso);
  return `${pad(d)}-${MONTHS[m - 1].slice(0, 3)}-${y}`;
}

/// "03/04/2026" (month first, US).
export function numericDate(iso) {
  const [y, m, d] = parse(iso);
  return `${pad(m)}/${pad(d)}/${y}`;
}

/// "3/4/26"
export function shortNumericDate(iso) {
  const [y, m, d] = parse(iso);
  return `${m}/${d}/${String(y).slice(2)}`;
}

export function ordinal(value) {
  const tens = value % 100;
  if (tens >= 11 && tens <= 13) return `${value}th`;
  return `${value}${{ 1: 'st', 2: 'nd', 3: 'rd' }[value % 10] ?? 'th'}`;
}

/// "4th day of March, 2026"
export function legalDate(iso) {
  const [y, m, d] = parse(iso);
  return `${ordinal(d)} day of ${MONTHS[m - 1]}, ${y}`;
}

/// "March 2026"
export function monthYear(iso) {
  const [y, m] = parse(iso);
  return `${MONTHS[m - 1]} ${y}`;
}

/// An RFC 5322 date: "Tue, 17 Mar 2026 09:42:11 -0500".
export function emailDate(iso, time, offset) {
  const [y, m, d] = parse(iso);
  return `${WEEKDAYS[weekday(iso)].slice(0, 3)}, ${d} ${MONTHS[m - 1].slice(0, 3)} ${y} ${time} ${offset}`;
}

/// Groups an integer's digits in threes with commas.
export function grouped(value) {
  const negative = value < 0;
  const digits = String(Math.abs(Math.trunc(value)));
  let out = '';
  for (let index = 0; index < digits.length; index += 1) {
    if (index && (digits.length - index) % 3 === 0) out += ',';
    out += digits[index];
  }
  return negative ? `-${out}` : out;
}

/// "$4,812.50" from 481250 cents. Negative amounts print in parentheses, as
/// statements do, unless `minus` is set.
export function money(cents, { symbol = '$', minus = false, cents: showCents = true } = {}) {
  const negative = cents < 0;
  const absolute = Math.abs(cents);
  const whole = Math.floor(absolute / 100);
  const fraction = absolute % 100;
  const body = `${symbol}${grouped(whole)}${showCents ? `.${pad(fraction)}` : ''}`;
  if (!negative) return body;
  return minus ? `-${body}` : `(${body})`;
}

/// "4,812.50" - an amount without a currency symbol, as table cells print it.
export function amount(cents) {
  return money(cents, { symbol: '' });
}

/// A decimal with a fixed number of places, from an integer count of
/// 10^-places units: decimal(12345, 2) = "123.45".
export function decimal(units, places) {
  const negative = units < 0;
  const absolute = Math.abs(units);
  const scale = 10 ** places;
  const whole = Math.floor(absolute / scale);
  const fraction = absolute % scale;
  return `${negative ? '-' : ''}${grouped(whole)}${places ? `.${pad(fraction, places)}` : ''}`;
}

/// "12.5%" from basis points of a percent: percent(1250) = "12.50%" with
/// places = 2, "12.5%" with places = 1.
export function percent(hundredths, places = 2) {
  if (places === 2) return `${decimal(hundredths, 2)}%`;
  if (places === 1) return `${decimal(Math.round(hundredths / 10), 1)}%`;
  return `${grouped(Math.round(hundredths / 100))}%`;
}

const SMALL = ['zero', 'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight', 'nine', 'ten',
  'eleven', 'twelve', 'thirteen', 'fourteen', 'fifteen', 'sixteen', 'seventeen', 'eighteen', 'nineteen'];
const TENS = ['', '', 'twenty', 'thirty', 'forty', 'fifty', 'sixty', 'seventy', 'eighty', 'ninety'];

/// English words for 0..999,999,999 ("thirty (30)" style clauses).
export function words(value) {
  if (value < 20) return SMALL[value];
  if (value < 100) return `${TENS[Math.floor(value / 10)]}${value % 10 ? `-${SMALL[value % 10]}` : ''}`;
  if (value < 1000) return `${SMALL[Math.floor(value / 100)]} hundred${value % 100 ? ` ${words(value % 100)}` : ''}`;
  if (value < 1_000_000) return `${words(Math.floor(value / 1000))} thousand${value % 1000 ? ` ${words(value % 1000)}` : ''}`;
  return `${words(Math.floor(value / 1_000_000))} million${value % 1_000_000 ? ` ${words(value % 1_000_000)}` : ''}`;
}

/// "thirty (30)"
export function wordsAndDigits(value) {
  return `${words(value)} (${value})`;
}

export function capitalize(text) {
  return text ? text[0].toUpperCase() + text.slice(1) : text;
}
