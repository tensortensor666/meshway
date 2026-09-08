import { deflateSync } from 'node:zlib';
import { mkdirSync, writeFileSync } from 'node:fs';

const root = new URL('../..', import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', '\\');
const icoPath = `${root}\\packaging\\windows\\meshway.ico`;
const pngPath = `${root}\\packaging\\windows\\meshway.png`;
const trayPath = `${root}\\backend\\assets\\meshway-tray.rgba`;
const sourceSize = 256;
const supersample = 4;

const pixels = new Uint8Array(sourceSize * supersample * sourceSize * supersample * 4);
const width = sourceSize * supersample;
const colors = {
  background: [15, 23, 42, 255],
  teal: [45, 212, 191, 255],
  coral: [251, 113, 133, 255],
};

function setPixel(x, y, color) {
  if (x < 0 || y < 0 || x >= width || y >= width) return;
  const offset = (y * width + x) * 4;
  pixels.set(color, offset);
}

function distanceToSegment(px, py, ax, ay, bx, by) {
  const dx = bx - ax;
  const dy = by - ay;
  const length = dx * dx + dy * dy;
  const t = length === 0 ? 0 : Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / length));
  const x = ax + t * dx;
  const y = ay + t * dy;
  return Math.hypot(px - x, py - y);
}

function fillRoundedRect(x, y, w, h, radius, color) {
  for (let py = 0; py < width; py += 1) {
    for (let px = 0; px < width; px += 1) {
      const ux = px / supersample;
      const uy = py / supersample;
      const cx = Math.max(x + radius, Math.min(ux, x + w - radius));
      const cy = Math.max(y + radius, Math.min(uy, y + h - radius));
      if (Math.hypot(ux - cx, uy - cy) <= radius) setPixel(px, py, color);
    }
  }
}

function strokePath(points, strokeWidth, color) {
  for (let py = 0; py < width; py += 1) {
    for (let px = 0; px < width; px += 1) {
      const ux = px / supersample;
      const uy = py / supersample;
      const distance = Math.min(...points.slice(0, -1).map((point, index) =>
        distanceToSegment(ux, uy, point[0], point[1], points[index + 1][0], points[index + 1][1])));
      if (distance <= strokeWidth / 2) setPixel(px, py, color);
    }
  }
}

function fillCircle(cx, cy, radius, color) {
  for (let py = 0; py < width; py += 1) {
    for (let px = 0; px < width; px += 1) {
      if (Math.hypot(px / supersample - cx, py / supersample - cy) <= radius) setPixel(px, py, color);
    }
  }
}

function resize(size) {
  const output = new Uint8Array(size * size * 4);
  const scale = width / size;
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      const totals = [0, 0, 0, 0];
      for (let sy = 0; sy < scale; sy += 1) {
        for (let sx = 0; sx < scale; sx += 1) {
          const offset = ((y * scale + sy) * width + x * scale + sx) * 4;
          for (let channel = 0; channel < 4; channel += 1) totals[channel] += pixels[offset + channel];
        }
      }
      const samples = scale * scale;
      const offset = (y * size + x) * 4;
      for (let channel = 0; channel < 4; channel += 1) output[offset + channel] = Math.round(totals[channel] / samples);
    }
  }
  return output;
}

function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const typeBytes = Buffer.from(type);
  const result = Buffer.alloc(12 + data.length);
  result.writeUInt32BE(data.length, 0);
  typeBytes.copy(result, 4);
  Buffer.from(data).copy(result, 8);
  result.writeUInt32BE(crc32(Buffer.concat([typeBytes, Buffer.from(data)])), 8 + data.length);
  return result;
}

function png(size, rgba) {
  const scanlines = Buffer.alloc(size * (size * 4 + 1));
  for (let y = 0; y < size; y += 1) {
    scanlines[y * (size * 4 + 1)] = 0;
    Buffer.from(rgba.buffer, rgba.byteOffset + y * size * 4, size * 4).copy(scanlines, y * (size * 4 + 1) + 1);
  }
  const header = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  return Buffer.concat([header, chunk('IHDR', ihdr), chunk('IDAT', deflateSync(scanlines)), chunk('IEND', Buffer.alloc(0))]);
}

fillRoundedRect(8, 8, 240, 240, 54, colors.background);
strokePath([[64, 190], [64, 66], [128, 148], [192, 66], [192, 190]], 25, colors.teal);
strokePath([[192, 190], [214, 190]], 12, colors.coral);
fillCircle(216, 190, 14, colors.coral);

const sizes = [16, 32, 64, 256];
const images = sizes.map((size) => ({ size, data: png(size, resize(size)) }));
const header = Buffer.alloc(6);
header.writeUInt16LE(0, 0);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(images.length, 4);
const directory = Buffer.alloc(images.length * 16);
let offset = header.length + directory.length;
for (let index = 0; index < images.length; index += 1) {
  const { size, data } = images[index];
  const entry = index * 16;
  directory[entry] = size === 256 ? 0 : size;
  directory[entry + 1] = size === 256 ? 0 : size;
  directory[entry + 2] = 0;
  directory[entry + 3] = 0;
  directory.writeUInt16LE(1, entry + 4);
  directory.writeUInt16LE(32, entry + 6);
  directory.writeUInt32LE(data.length, entry + 8);
  directory.writeUInt32LE(offset, entry + 12);
  offset += data.length;
}

mkdirSync(`${root}\\backend\\assets`, { recursive: true });
writeFileSync(icoPath, Buffer.concat([header, directory, ...images.map((image) => image.data)]));
writeFileSync(pngPath, images.at(-1).data);
writeFileSync(trayPath, resize(32));
