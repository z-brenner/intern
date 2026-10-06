/// Draws a page model the way a scanner sees a printed page.
///
/// Glyphs come from the committed atlas (proportional Liberation-derived
/// faces, metric-compatible with the PDF fonts) and are scaled to the target
/// resolution by area averaging in integer arithmetic, so a 300-DPI page is a
/// pure function of its model on every machine. The degradations - turning,
/// skew, blur, speckle, faint toner, uneven lighting, low resolution,
/// bilevel thresholding - are the ordinary ways real scans go wrong, each a
/// deterministic function of the image and a seeded generator. Rotation for
/// skew uses sine and cosine computed by our own power series, never
/// `Math.sin`, whose last bit is allowed to differ between engines.
import { ATLAS, glyphBitmap, advance } from './fonts.mjs';

/// Atlas units: glyph origin at (margin, baseline) inside a cell `height`
/// pixels tall, drawn at `em_pixels` per em.
const ATLAS_EM = ATLAS.em_pixels;
const ATLAS_MARGIN = ATLAS.margin;
const ATLAS_BASELINE = ATLAS.baseline;
// Glyph scales are quantised to a quarter pixel per em: p / Q of the atlas.
const Q = ATLAS_EM * 4;

export function blank(width, height, value = 255) {
  return { width, height, pixels: new Uint8Array(width * height).fill(value) };
}

const SCALED = new Map();

/// A glyph's coverage scaled by p / Q, as 0..255.
function scaledGlyph(face, character, p) {
  const key = `${face}\u0000${character}\u0000${p}`;
  let scaled = SCALED.get(key);
  if (scaled) return scaled;
  const source = glyphBitmap(face, character);
  const sw = source.width;
  const sh = source.height;
  const tw = Math.ceil((sw * p) / Q);
  const th = Math.ceil((sh * p) / Q);
  // Separable box filter with exact integer overlaps: target pixel t spans
  // [t*Q, (t+1)*Q) and source pixel i spans [i*p, (i+1)*p) on a common grid.
  const horizontal = new Uint32Array(sh * tw);
  for (let t = 0; t < tw; t += 1) {
    const a = t * Q;
    const b = a + Q;
    const first = Math.floor(a / p);
    const last = Math.min(sw - 1, Math.floor((b - 1) / p));
    for (let i = first; i <= last; i += 1) {
      const weight = Math.min(b, (i + 1) * p) - Math.max(a, i * p);
      for (let y = 0; y < sh; y += 1) horizontal[y * tw + t] += weight * source.pixels[y * sw + i];
    }
  }
  const pixels = new Uint8Array(tw * th);
  const accumulator = new Float64Array(tw);
  const denominator = 15 * Q * Q;
  for (let u = 0; u < th; u += 1) {
    accumulator.fill(0);
    const a = u * Q;
    const b = a + Q;
    const first = Math.floor(a / p);
    const last = Math.min(sh - 1, Math.floor((b - 1) / p));
    for (let j = first; j <= last; j += 1) {
      const weight = Math.min(b, (j + 1) * p) - Math.max(a, j * p);
      const row = j * tw;
      for (let t = 0; t < tw; t += 1) accumulator[t] += weight * horizontal[row + t];
    }
    for (let t = 0; t < tw; t += 1) pixels[u * tw + t] = Math.round((accumulator[t] * 255) / denominator);
  }
  scaled = { width: tw, height: th, pixels, originX: (ATLAS_MARGIN * p) / Q, originY: (ATLAS_BASELINE * p) / Q };
  SCALED.set(key, scaled);
  return scaled;
}

/// Ink coverage (0 none, 255 full) for a page at `dpi`. `weight` thickens
/// strokes slightly the way toner spreads on a photocopy (1.0 = as drawn).
export function rasterize(page, dpi, { weight = 1 } = {}) {
  const scale = dpi / 72;
  const width = Math.round(page.width * scale);
  const height = Math.round(page.height * scale);
  const ink = new Uint8Array(width * height);
  const plot = (x, y, value) => {
    if (x < 0 || y < 0 || x >= width || y >= height) return;
    const index = y * width + x;
    if (value > ink[index]) ink[index] = value;
  };
  const fillRect = (x0, y0, x1, y1, value) => {
    const left = Math.max(0, Math.round(x0));
    const right = Math.min(width, Math.round(x1));
    const top = Math.max(0, Math.round(y0));
    const bottom = Math.min(height, Math.round(y1));
    for (let y = top; y < bottom; y += 1) for (let x = left; x < right; x += 1) plot(x, y, value);
  };
  for (const item of page.items) {
    if (item.type === 'text') {
      if (item.render === 3) continue;
      const emPixels = item.size * scale;
      const p = Math.max(4, Math.round(emPixels * 4 * weight));
      const darkness = Math.round(255 * (1 - item.grey));
      let pen = item.x * scale;
      const baseline = item.y * scale;
      for (const character of item.text) {
        if (character !== ' ') {
          const glyph = scaledGlyph(item.face, character, p);
          const left = Math.round(pen - glyph.originX);
          const top = Math.round(baseline - glyph.originY);
          for (let gy = 0; gy < glyph.height; gy += 1) {
            const y = top + gy;
            if (y < 0 || y >= height) continue;
            const row = gy * glyph.width;
            for (let gx = 0; gx < glyph.width; gx += 1) {
              const value = glyph.pixels[row + gx];
              if (!value) continue;
              plot(left + gx, y, darkness === 255 ? value : Math.round((value * darkness) / 255));
            }
          }
        }
        pen += advance(character, item.face) * emPixels;
      }
    } else if (item.type === 'line') {
      const thickness = Math.max(1, Math.round(item.width * scale));
      const value = Math.round(255 * (1 - item.grey));
      const x1 = item.x1 * scale;
      const y1 = item.y1 * scale;
      const x2 = item.x2 * scale;
      const y2 = item.y2 * scale;
      if (Math.abs(y1 - y2) < 0.01) fillRect(Math.min(x1, x2), y1 - thickness / 2, Math.max(x1, x2), y1 - thickness / 2 + thickness, value);
      else if (Math.abs(x1 - x2) < 0.01) fillRect(x1 - thickness / 2, Math.min(y1, y2), x1 - thickness / 2 + thickness, Math.max(y1, y2), value);
      else strokePath([[x1, y1], [x2, y2]], thickness, value, plot);
    } else if (item.type === 'rect') {
      const x = item.x * scale;
      const y = item.y * scale;
      const w = item.w * scale;
      const h = item.h * scale;
      if (item.fill !== null && item.fill !== undefined && item.fill < 1) fillRect(x, y, x + w, y + h, Math.round(255 * (1 - item.fill)));
      if (item.stroke !== null && item.stroke !== undefined) {
        const t = Math.max(1, Math.round(item.width * scale));
        const value = Math.round(255 * (1 - item.stroke));
        fillRect(x, y, x + w, y + t, value);
        fillRect(x, y + h - t, x + w, y + h, value);
        fillRect(x, y, x + t, y + h, value);
        fillRect(x + w - t, y, x + w, y + h, value);
      }
    } else if (item.type === 'path') {
      const thickness = Math.max(1, item.width * scale);
      strokePath(item.points.map(([x, y]) => [x * scale, y * scale]), thickness, Math.round(255 * (1 - item.grey)), plot);
    } else if (item.type === 'image') {
      throw new Error('scanned pages are drawn from text and strokes only');
    }
  }
  const pixels = new Uint8Array(width * height);
  for (let index = 0; index < pixels.length; index += 1) pixels[index] = 255 - ink[index];
  return { width, height, pixels };
}

/// A polyline stroked with a round pen of the given thickness.
function strokePath(points, thickness, value, plot) {
  const radius = thickness / 2;
  const reach = Math.ceil(radius);
  const radiusSquared = radius * radius;
  for (let index = 1; index < points.length; index += 1) {
    const [x0, y0] = points[index - 1];
    const [x1, y1] = points[index];
    const steps = Math.max(1, Math.ceil(Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0)) * 2));
    for (let step = 0; step <= steps; step += 1) {
      const cx = x0 + ((x1 - x0) * step) / steps;
      const cy = y0 + ((y1 - y0) * step) / steps;
      const px = Math.round(cx);
      const py = Math.round(cy);
      for (let dy = -reach; dy <= reach; dy += 1) {
        for (let dx = -reach; dx <= reach; dx += 1) {
          if (dx * dx + dy * dy <= radiusSquared + 0.25) plot(px + dx, py + dy, value);
        }
      }
    }
  }
}

/// Exact quarter turns, clockwise.
export function rotate(image, degrees) {
  const { width, height, pixels } = image;
  const turns = ((degrees / 90) % 4 + 4) % 4;
  if (turns === 0) return { width, height, pixels: Uint8Array.from(pixels) };
  if (turns === 2) {
    const out = new Uint8Array(pixels.length);
    for (let index = 0; index < pixels.length; index += 1) out[pixels.length - 1 - index] = pixels[index];
    return { width, height, pixels: out };
  }
  const out = new Uint8Array(pixels.length);
  const newWidth = height;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const value = pixels[y * width + x];
      // Clockwise: (x, y) -> (height-1-y, x); counter-clockwise: (y, width-1-x).
      if (turns === 1) out[x * newWidth + (height - 1 - y)] = value;
      else out[(width - 1 - x) * newWidth + y] = value;
    }
  }
  return { width: newWidth, height: width, pixels: out };
}

/// Sine and cosine of a small angle by power series - exact to double
/// precision for the few degrees a crooked scan is off by, and identical on
/// every engine.
export function sinCos(degrees) {
  const radians = (degrees * 3.141592653589793) / 180;
  const squared = radians * radians;
  let sine = 0;
  let cosine = 0;
  let termSine = radians;
  let termCosine = 1;
  for (let n = 0; n < 12; n += 1) {
    sine += termSine;
    cosine += termCosine;
    termSine *= -squared / ((2 * n + 2) * (2 * n + 3));
    termCosine *= -squared / ((2 * n + 1) * (2 * n + 2));
  }
  return [sine, cosine];
}

/// Turns the page content by a small angle about its centre (positive =
/// clockwise), bilinear sampling, paper-white where nothing maps.
export function skew(image, degrees, paper = 255) {
  const { width, height, pixels } = image;
  const [sine, cosine] = sinCos(degrees);
  const out = new Uint8Array(pixels.length);
  const cx = (width - 1) / 2;
  const cy = (height - 1) / 2;
  for (let y = 0; y < height; y += 1) {
    const dy = y - cy;
    for (let x = 0; x < width; x += 1) {
      const dx = x - cx;
      // Inverse rotation: where in the source this output pixel came from.
      const sx = cosine * dx + sine * dy + cx;
      const sy = -sine * dx + cosine * dy + cy;
      const x0 = Math.floor(sx);
      const y0 = Math.floor(sy);
      if (x0 < 0 || y0 < 0 || x0 + 1 >= width || y0 + 1 >= height) {
        out[y * width + x] = paper;
        continue;
      }
      const fx = sx - x0;
      const fy = sy - y0;
      const index = y0 * width + x0;
      const top = pixels[index] + (pixels[index + 1] - pixels[index]) * fx;
      const bottom = pixels[index + width] + (pixels[index + width + 1] - pixels[index + width]) * fx;
      out[y * width + x] = Math.round(top + (bottom - top) * fy);
    }
  }
  return { width, height, pixels: out };
}

/// Separable box blur of the given radius (an out-of-focus or worn platen).
export function boxBlur(image, radius) {
  const { width, height, pixels } = image;
  const span = radius * 2 + 1;
  const horizontal = new Uint16Array(pixels.length);
  for (let y = 0; y < height; y += 1) {
    const row = y * width;
    for (let x = 0; x < width; x += 1) {
      let sum = 0;
      for (let k = -radius; k <= radius; k += 1) sum += pixels[row + Math.min(width - 1, Math.max(0, x + k))];
      horizontal[row + x] = sum;
    }
  }
  const out = new Uint8Array(pixels.length);
  for (let x = 0; x < width; x += 1) {
    for (let y = 0; y < height; y += 1) {
      let sum = 0;
      for (let k = -radius; k <= radius; k += 1) sum += horizontal[Math.min(height - 1, Math.max(0, y + k)) * width + x];
      out[y * width + x] = Math.round(sum / (span * span));
    }
  }
  return { width, height, pixels: out };
}

/// Dust and toner specks (dark) and dropouts (light). `density` is specks
/// per million pixels.
export function speckle(image, rng, density, { darkShare = 0.7, maxSize = 2 } = {}) {
  const { width, height } = image;
  const pixels = Uint8Array.from(image.pixels);
  const count = Math.round((width * height * density) / 1_000_000);
  for (let index = 0; index < count; index += 1) {
    const x = rng.int(0, width - 1);
    const y = rng.int(0, height - 1);
    const dark = rng.chance(darkShare);
    const size = rng.int(1, maxSize);
    const value = dark ? rng.int(0, 70) : 255;
    for (let dy = 0; dy < size; dy += 1) {
      for (let dx = 0; dx < size; dx += 1) {
        if (x + dx < width && y + dy < height) pixels[(y + dy) * width + x + dx] = value;
      }
    }
  }
  return { width, height, pixels };
}

/// Uniform grain: each pixel moves by up to `amplitude` levels.
export function grain(image, rng, amplitude) {
  const pixels = Uint8Array.from(image.pixels);
  for (let index = 0; index < pixels.length; index += 1) {
    const delta = rng.int(-amplitude, amplitude);
    pixels[index] = Math.max(0, Math.min(255, pixels[index] + delta));
  }
  return { width: image.width, height: image.height, pixels };
}

/// Faint toner on grey paper under uneven light: ink maps to `ink`, paper to
/// `paper`, and the whole page dims by up to `falloff` (0..1) towards the
/// corner opposite the lamp.
export function lowContrast(image, { paper = 226, ink = 128, falloff = 0.18 } = {}) {
  const { width, height } = image;
  const pixels = new Uint8Array(image.pixels.length);
  const span = width + height;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const value = image.pixels[y * width + x];
      const toned = ink + ((paper - ink) * value) / 255;
      const light = 1 - (falloff * (x + y)) / span;
      pixels[y * width + x] = Math.max(0, Math.min(255, Math.round(toned * light)));
    }
  }
  return { width, height, pixels };
}

/// Area-average reduction by an integer factor.
export function downsample(image, factor) {
  const width = Math.floor(image.width / factor);
  const height = Math.floor(image.height / factor);
  const pixels = new Uint8Array(width * height);
  const area = factor * factor;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      let sum = 0;
      for (let dy = 0; dy < factor; dy += 1) {
        const row = (y * factor + dy) * image.width + x * factor;
        for (let dx = 0; dx < factor; dx += 1) sum += image.pixels[row + dx];
      }
      pixels[y * width + x] = Math.round(sum / area);
    }
  }
  return { width, height, pixels };
}

/// Bilevel: below `level` is black.
export function threshold(image, level = 128) {
  const pixels = new Uint8Array(image.pixels.length);
  for (let index = 0; index < pixels.length; index += 1) pixels[index] = image.pixels[index] < level ? 0 : 255;
  return { width: image.width, height: image.height, pixels };
}

/// Packs a bilevel image MSB-first, 1 = white, rows padded to a byte - the
/// layout of both a 1-bit PNG and a 1-bit DeviceGray PDF image.
export function pack1(image) {
  const rowBytes = Math.ceil(image.width / 8);
  const out = Buffer.alloc(rowBytes * image.height);
  for (let y = 0; y < image.height; y += 1) {
    for (let x = 0; x < image.width; x += 1) {
      if (image.pixels[y * image.width + x] >= 128) out[y * rowBytes + (x >> 3)] |= 0x80 >> (x & 7);
    }
  }
  return out;
}

/// The image as a PDF image source: `{ width, height, bits, data }`.
export function pdfImage(image, bits) {
  return {
    width: image.width,
    height: image.height,
    bits,
    data: bits === 1 ? pack1(image) : Buffer.from(image.pixels.buffer, image.pixels.byteOffset, image.pixels.length),
  };
}
