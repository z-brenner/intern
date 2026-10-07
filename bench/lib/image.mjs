/// PNG and TIFF writers for scanned pages.
///
/// PNG: 8-bit grey or 1-bit bilevel, with a pHYs chunk carrying the scan
/// resolution the way scanner software writes it. TIFF: 8-bit grey, one or
/// more frames chained through their image file directories, with Adobe
/// Deflate compression (what most scanning software saves; the worker's
/// decoder reads it) or none. Both are deterministic: fixed chunk order, no
/// timestamps, fixed zlib level.
import { deflateSync } from 'node:zlib';
import { crc32 } from './zip.mjs';
import { pack1 } from './raster.mjs';

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, 'latin1'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([length, body, crc]);
}

/// `image` is `{ width, height, pixels }` grey; `bits` 8 or 1.
export function png(image, { bits = 8, dpi = null, description = null } = {}) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(image.width, 0);
  header.writeUInt32BE(image.height, 4);
  header[8] = bits;
  header[9] = 0; // greyscale
  const rowBytes = bits === 1 ? Math.ceil(image.width / 8) : image.width;
  const packed = bits === 1 ? pack1(image) : Buffer.from(image.pixels.buffer, image.pixels.byteOffset, image.pixels.length);
  const raw = Buffer.alloc((rowBytes + 1) * image.height);
  for (let y = 0; y < image.height; y += 1) {
    raw[y * (rowBytes + 1)] = 0;
    packed.copy(raw, y * (rowBytes + 1) + 1, y * rowBytes, (y + 1) * rowBytes);
  }
  const chunks = [Buffer.from('89504e470d0a1a0a', 'hex'), chunk('IHDR', header)];
  if (dpi) {
    const physical = Buffer.alloc(9);
    const perMetre = Math.round(dpi / 0.0254);
    physical.writeUInt32BE(perMetre, 0);
    physical.writeUInt32BE(perMetre, 4);
    physical[8] = 1;
    chunks.push(chunk('pHYs', physical));
  }
  if (description) chunks.push(chunk('tEXt', Buffer.from(`Software\0${description}`, 'latin1')));
  chunks.push(chunk('IDAT', deflateSync(raw, { level: 9 })), chunk('IEND', Buffer.alloc(0)));
  return Buffer.concat(chunks);
}

/// Decodes the IHDR and checks every chunk's CRC. Used by tests.
export function inspectPng(bytes) {
  if (bytes.subarray(0, 8).toString('hex') !== '89504e470d0a1a0a') return { valid: false };
  let position = 8;
  let valid = true;
  let header = null;
  while (position < bytes.length) {
    const length = bytes.readUInt32BE(position);
    const type = bytes.subarray(position + 4, position + 8).toString('latin1');
    const body = bytes.subarray(position + 4, position + 8 + length);
    if (crc32(body) !== bytes.readUInt32BE(position + 8 + length)) valid = false;
    if (type === 'IHDR') header = { width: body.readUInt32BE(4), height: body.readUInt32BE(8), bits: body[12] };
    position += 12 + length;
  }
  return { valid, ...header };
}

/// A TIFF holding one 8-bit grey frame per image, in order.
export function tiff(frames, { dpi = 300, compression = 'deflate', software = 'InternBench scan' } = {}) {
  const parts = [];
  let offset = 8;
  const header = Buffer.alloc(8);
  header.write('II', 0, 'latin1');
  header.writeUInt16LE(42, 2);
  parts.push(header);
  const softwareBytes = Buffer.from(`${software}\0`, 'latin1');
  const directories = [];
  // Lay out every frame's pixel data and side values first, then the
  // directories, so each directory can point forward to the next.
  for (const frame of frames) {
    const raw = Buffer.from(frame.pixels.buffer, frame.pixels.byteOffset, frame.pixels.length);
    const data = compression === 'deflate' ? deflateSync(raw, { level: 9 }) : raw;
    const dataOffset = offset;
    parts.push(data);
    offset += data.length;
    if (offset % 2) {
      parts.push(Buffer.alloc(1));
      offset += 1;
    }
    const resolutionOffset = offset;
    const resolution = Buffer.alloc(8);
    resolution.writeUInt32LE(dpi, 0);
    resolution.writeUInt32LE(1, 4);
    parts.push(resolution);
    offset += 8;
    const softwareOffset = offset;
    parts.push(softwareBytes);
    offset += softwareBytes.length;
    if (offset % 2) {
      parts.push(Buffer.alloc(1));
      offset += 1;
    }
    directories.push({ frame, dataOffset, dataLength: data.length, resolutionOffset, softwareOffset });
  }
  header.writeUInt32LE(offset, 4);
  const entryCount = 13;
  const directorySize = 2 + entryCount * 12 + 4;
  directories.forEach((directory, index) => {
    const ifd = Buffer.alloc(directorySize);
    ifd.writeUInt16LE(entryCount, 0);
    const { frame } = directory;
    const entries = [
      [254, 4, 1, index === 0 ? 0 : 2], // NewSubfileType: later frames are pages of a multi-page file
      [256, 4, 1, frame.width],
      [257, 4, 1, frame.height],
      [258, 3, 1, 8],
      [259, 3, 1, compression === 'deflate' ? 8 : 1],
      [262, 3, 1, 1], // BlackIsZero
      [273, 4, 1, directory.dataOffset],
      [277, 3, 1, 1],
      [278, 4, 1, frame.height],
      [279, 4, 1, directory.dataLength],
      [282, 5, 1, directory.resolutionOffset],
      [283, 5, 1, directory.resolutionOffset],
      [305, 2, softwareBytes.length, directory.softwareOffset],
    ];
    // ResolutionUnit (296) would make 14 entries; inches are the default unit.
    entries.forEach(([tag, type, count, value], entry) => {
      const at = 2 + entry * 12;
      ifd.writeUInt16LE(tag, at);
      ifd.writeUInt16LE(type, at + 2);
      ifd.writeUInt32LE(count, at + 4);
      if (type === 3 && count === 1) ifd.writeUInt16LE(value, at + 8);
      else ifd.writeUInt32LE(value, at + 8);
    });
    const next = index + 1 < directories.length ? offset + directorySize : 0;
    ifd.writeUInt32LE(next, 2 + entryCount * 12);
    parts.push(ifd);
    offset += directorySize;
  });
  return Buffer.concat(parts);
}
