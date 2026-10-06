/// Seeded randomness for the InternBench generator.
///
/// Every choice the generator makes - which name, which amount, which clause
/// comes next - is drawn from here, never from `Math.random`, so a document
/// is a pure function of its seed and two runs on any machine produce the
/// same bytes. Mulberry32 is small, fast, and has no platform-dependent
/// arithmetic: it is 32-bit integer multiplication and shifts only.

/// FNV-1a over the UTF-16 code units of a string. Used to turn a document id
/// and a purpose ("names", "amounts") into a seed, so adding a draw to one
/// part of a builder does not shift every value drawn after it elsewhere.
export function hashString(text) {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash >>> 0;
}

export function mulberry32(seed) {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let value = state;
    value = Math.imul(value ^ (value >>> 15), value | 1);
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
    return (value ^ (value >>> 14)) >>> 0;
  };
}

export class Rng {
  constructor(seed) {
    this.seed = seed >>> 0;
    this.next = mulberry32(this.seed);
  }

  /// A generator seeded from a label, e.g. `Rng.from('credit-agreement/lenders')`.
  static from(label) {
    return new Rng(hashString(label));
  }

  /// An independent stream derived from this generator's seed and a label.
  /// Forking does not consume from this stream, so builders can carve out
  /// sub-streams without disturbing the draws that follow.
  fork(label) {
    return new Rng(hashString(`${this.seed}:${label}`));
  }

  uint32() {
    return this.next();
  }

  /// A float in [0, 1).
  float() {
    return this.next() / 4294967296;
  }

  /// An integer in [min, max], inclusive.
  int(min, max) {
    if (max < min) throw new Error(`empty range ${min}..${max}`);
    const span = max - min + 1;
    return min + Math.floor(this.float() * span);
  }

  chance(probability) {
    return this.float() < probability;
  }

  pick(items) {
    if (!items.length) throw new Error('pick from an empty list');
    return items[Math.floor(this.float() * items.length)];
  }

  /// A shuffled copy (Fisher-Yates).
  shuffle(items) {
    const copy = [...items];
    for (let index = copy.length - 1; index > 0; index -= 1) {
      const other = Math.floor(this.float() * (index + 1));
      [copy[index], copy[other]] = [copy[other], copy[index]];
    }
    return copy;
  }

  /// `count` distinct items, in random order.
  sample(items, count) {
    if (count > items.length) throw new Error(`cannot sample ${count} of ${items.length}`);
    return this.shuffle(items).slice(0, count);
  }

  /// An integer amount in [min, max] rounded to a multiple of `step`.
  amount(min, max, step = 1) {
    return Math.round(this.int(min, max) / step) * step;
  }
}
