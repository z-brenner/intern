/// A minimal deterministic ZIP writer for Office packages.
///
/// Entries are written in the order given, with a fixed DOS timestamp and no
/// extra fields, so the same entries always make the same bytes. Office files
/// are deflated (method 8) the way Word and Excel save them; the existing
/// fixture generator stores them, which is valid but unlike any real file.
import { deflateRawSync } from 'node:zlib';

const FIXED_DOS_TIME = 0;
// 1980-01-01, the DOS epoch: bits are (year-1980)<<9 | month<<5 | day.
const FIXED_DOS_DATE = 0x21;

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let index = 0; index < 256; index += 1) {
    let value = index;
    for (let bit = 0; bit < 8; bit += 1) value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
    table[index] = value >>> 0;
  }
  return table;
})();

export function crc32(bytes, seed = 0) {
  let crc = (seed ^ 0xffffffff) >>> 0;
  for (let index = 0; index < bytes.length; index += 1) crc = CRC_TABLE[(crc ^ bytes[index]) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function u16(value) {
  const bytes = Buffer.alloc(2);
  bytes.writeUInt16LE(value);
  return bytes;
}

function u32(value) {
  const bytes = Buffer.alloc(4);
  bytes.writeUInt32LE(value >>> 0);
  return bytes;
}

/// `entries` is a list of [name, string | Buffer]. `store` lists names to
/// keep uncompressed (already-compressed media, say).
export function zip(entries, { store = [] } = {}) {
  const local = [];
  const central = [];
  let offset = 0;
  for (const [name, value] of entries) {
    const nameBytes = Buffer.from(name, 'utf8');
    const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value, 'utf8');
    const checksum = crc32(bytes);
    const method = store.includes(name) ? 0 : 8;
    const payload = method === 8 ? deflateRawSync(bytes, { level: 9 }) : bytes;
    const header = Buffer.concat([
      Buffer.from('504b0304', 'hex'), u16(20), u16(0), u16(method), u16(FIXED_DOS_TIME), u16(FIXED_DOS_DATE),
      u32(checksum), u32(payload.length), u32(bytes.length), u16(nameBytes.length), u16(0), nameBytes,
    ]);
    local.push(header, payload);
    central.push(Buffer.concat([
      Buffer.from('504b0102', 'hex'), u16(20), u16(20), u16(0), u16(method), u16(FIXED_DOS_TIME), u16(FIXED_DOS_DATE),
      u32(checksum), u32(payload.length), u32(bytes.length), u16(nameBytes.length), u16(0), u16(0), u16(0), u16(0),
      u32(0), u32(offset), nameBytes,
    ]));
    offset += header.length + payload.length;
  }
  const centralBytes = Buffer.concat(central);
  return Buffer.concat([...local, centralBytes, Buffer.concat([
    Buffer.from('504b0506', 'hex'), u16(0), u16(0), u16(entries.length), u16(entries.length),
    u32(centralBytes.length), u32(offset), u16(0),
  ])]);
}

/// Reads back a ZIP written by `zip` (or any simple archive without data
/// descriptors): name -> uncompressed bytes. Used by tests.
export async function unzip(bytes) {
  const { inflateRawSync } = await import('node:zlib');
  const files = new Map();
  let end = bytes.length - 22;
  while (end >= 0 && bytes.readUInt32LE(end) !== 0x06054b50) end -= 1;
  if (end < 0) throw new Error('no end of central directory');
  const count = bytes.readUInt16LE(end + 10);
  let position = bytes.readUInt32LE(end + 16);
  for (let index = 0; index < count; index += 1) {
    if (bytes.readUInt32LE(position) !== 0x02014b50) throw new Error('bad central directory entry');
    const method = bytes.readUInt16LE(position + 10);
    const crc = bytes.readUInt32LE(position + 16);
    const compressed = bytes.readUInt32LE(position + 20);
    const nameLength = bytes.readUInt16LE(position + 28);
    const extraLength = bytes.readUInt16LE(position + 30);
    const commentLength = bytes.readUInt16LE(position + 32);
    const localOffset = bytes.readUInt32LE(position + 42);
    const name = bytes.subarray(position + 46, position + 46 + nameLength).toString('utf8');
    const localName = bytes.readUInt16LE(localOffset + 26);
    const localExtra = bytes.readUInt16LE(localOffset + 28);
    const start = localOffset + 30 + localName + localExtra;
    const data = bytes.subarray(start, start + compressed);
    const content = method === 8 ? inflateRawSync(data) : Buffer.from(data);
    files.set(name, { content, crc, valid: crc32(content) === crc });
    position += 46 + nameLength + extraLength + commentLength;
  }
  return files;
}
