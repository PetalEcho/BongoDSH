import { inflateSync } from 'node:zlib';

// Read alpha bounds for layout; the encoded image is embedded byte for byte.
export function pngBounds(bytes) {
  if (!bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))) {
    throw new Error('Expected a PNG asset.');
  }
  let width = 0;
  let height = 0;
  const chunks = [];
  for (let offset = 8; offset + 12 <= bytes.length;) {
    const length = bytes.readUInt32BE(offset);
    if (offset + length + 12 > bytes.length) throw new Error('Truncated PNG.');
    const kind = bytes.toString('ascii', offset + 4, offset + 8);
    const data = bytes.subarray(offset + 8, offset + 8 + length);
    if (kind === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      if (data.length !== 13 || data[8] !== 8 || data[9] !== 6 || data[10] || data[11] || data[12]
        || width < 1 || height < 1 || width * height > 40_000_000) {
        throw new Error('Assets must be non-interlaced 8-bit RGBA PNGs.');
      }
    } else if (kind === 'IDAT') chunks.push(data);
    else if (kind === 'IEND') break;
    offset += length + 12;
  }
  if (!width || !height) throw new Error('Missing PNG dimensions.');
  const stride = width * 4;
  const raw = inflateSync(Buffer.concat(chunks), { maxOutputLength: (stride + 1) * height });
  if (raw.length !== (stride + 1) * height) throw new Error('Invalid PNG scanlines.');
  let previous = Buffer.alloc(stride);
  let current = Buffer.alloc(stride);
  let left = width;
  let top = height;
  let right = -1;
  let bottom = -1;
  for (let y = 0; y < height; y++) {
    const row = y * (stride + 1);
    const filter = raw[row];
    if (filter > 4) throw new Error('Invalid PNG filter.');
    for (let x = 0; x < stride; x++) {
      const a = x >= 4 ? current[x - 4] : 0;
      const b = previous[x];
      const c = x >= 4 ? previous[x - 4] : 0;
      let predictor = 0;
      if (filter === 1) predictor = a;
      else if (filter === 2) predictor = b;
      else if (filter === 3) predictor = Math.floor((a + b) / 2);
      else if (filter === 4) {
        const p = a + b - c;
        const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
        predictor = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      current[x] = (raw[row + 1 + x] + predictor) & 255;
    }
    for (let x = 0; x < width; x++) {
      if (current[x * 4 + 3] < 16) continue;
      left = Math.min(left, x); right = Math.max(right, x);
      top = Math.min(top, y); bottom = y;
    }
    [previous, current] = [current, previous];
  }
  if (right < 0) throw new Error('Empty character asset.');
  return [left, top, right - left + 1, bottom - top + 1, width, height];
}
