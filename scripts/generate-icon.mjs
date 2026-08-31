import fs from "fs";
import path from "path";
import zlib from "zlib";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SIZE = 1024;
const data = Buffer.alloc(SIZE * SIZE * 4);

function blend(x, y, r, g, b, a) {
  if (x < 0 || y < 0 || x >= SIZE || y >= SIZE || a <= 0) return;
  const i = (y * SIZE + x) * 4;
  const aa = Math.min(1, a);
  const ia = 1 - aa;
  data[i] = Math.round(data[i] * ia + r * aa);
  data[i + 1] = Math.round(data[i + 1] * ia + g * aa);
  data[i + 2] = Math.round(data[i + 2] * ia + b * aa);
  data[i + 3] = Math.min(255, Math.round(data[i + 3] + (255 - data[i + 3]) * aa));
}

function disk(cx, cy, rx, ry, rgb, a = 1) {
  const x0 = Math.max(0, Math.floor(cx - rx - 1));
  const x1 = Math.min(SIZE - 1, Math.ceil(cx + rx + 1));
  const y0 = Math.max(0, Math.floor(cy - ry - 1));
  const y1 = Math.min(SIZE - 1, Math.ceil(cy + ry + 1));
  for (let y = y0; y <= y1; y++) {
    for (let x = x0; x <= x1; x++) {
      const u = (x + 0.5 - cx) / rx;
      const v = (y + 0.5 - cy) / ry;
      const d = u * u + v * v;
      if (d <= 1) {
        const edge = Math.min(1, (1 - Math.sqrt(d)) * Math.min(rx, ry));
        blend(x, y, rgb[0], rgb[1], rgb[2], a * Math.min(1, edge));
      }
    }
  }
}

function ring(cx, cy, r, t, rgb, a = 1) {
  const x0 = Math.max(0, Math.floor(cx - r - t - 1));
  const x1 = Math.min(SIZE - 1, Math.ceil(cx + r + t + 1));
  const y0 = Math.max(0, Math.floor(cy - r - t - 1));
  const y1 = Math.min(SIZE - 1, Math.ceil(cy + r + t + 1));
  for (let y = y0; y <= y1; y++) {
    for (let x = x0; x <= x1; x++) {
      const d = Math.hypot(x + 0.5 - cx, y + 0.5 - cy);
      const dist = Math.abs(d - r);
      if (dist < t + 1) {
        const cov = Math.max(0, 1 - dist / (t + 0.5));
        blend(x, y, rgb[0], rgb[1], rgb[2], a * cov);
      }
    }
  }
}

function tri(ax, ay, bx, by, cx, cy, rgb, a = 1) {
  const minx = Math.max(0, Math.floor(Math.min(ax, bx, cx)));
  const maxx = Math.min(SIZE - 1, Math.ceil(Math.max(ax, bx, cx)));
  const miny = Math.max(0, Math.floor(Math.min(ay, by, cy)));
  const maxy = Math.min(SIZE - 1, Math.ceil(Math.max(ay, by, cy)));
  const area = (bx - ax) * (cy - ay) - (cx - ax) * (by - ay);
  if (Math.abs(area) < 1) return;
  for (let y = miny; y <= maxy; y++) {
    for (let x = minx; x <= maxx; x++) {
      const px = x + 0.5;
      const py = y + 0.5;
      const w0 = ((bx - px) * (cy - py) - (cx - px) * (by - py)) / area;
      const w1 = ((cx - px) * (ay - py) - (ax - px) * (cy - py)) / area;
      const w2 = 1 - w0 - w1;
      if (w0 >= -0.01 && w1 >= -0.01 && w2 >= -0.01) {
        blend(x, y, rgb[0], rgb[1], rgb[2], a);
      }
    }
  }
}

const BG = [12, 10, 7];
const CANARY = [240, 196, 26];
const CANARY_D = [196, 148, 10];
const DARK = [18, 14, 8];
const CAGE = [168, 132, 28];

for (let i = 0; i < SIZE * SIZE; i++) {
  data[i * 4] = BG[0];
  data[i * 4 + 1] = BG[1];
  data[i * 4 + 2] = BG[2];
  data[i * 4 + 3] = 255;
}

disk(512, 520, 380, 380, [48, 36, 6], 0.35);
disk(512, 500, 300, 300, [90, 70, 8], 0.18);
ring(512, 500, 390, 10, CAGE, 0.45);
ring(512, 500, 390, 2.5, CANARY, 0.7);

// tail
disk(250, 560, 90, 48, CANARY_D, 1);
tri(80, 500, 250, 520, 120, 620, CANARY, 1);
tri(70, 560, 240, 580, 150, 680, CANARY_D, 1);

// body
disk(430, 560, 210, 150, CANARY, 1);
disk(400, 600, 160, 110, CANARY_D, 0.35);

// wing
disk(400, 540, 140, 70, CANARY_D, 1);
disk(420, 530, 90, 40, CANARY, 0.7);

// head
disk(620, 400, 92, 92, CANARY, 1);
disk(640, 390, 70, 70, [250, 214, 70], 0.35);

// beak
tri(700, 390, 820, 430, 698, 455, [232, 140, 20], 1);
tri(710, 405, 800, 428, 710, 440, [255, 190, 60], 1);

// eye
disk(640, 385, 16, 16, DARK, 1);
disk(646, 380, 5, 5, [255, 236, 160], 1);

// legs
tri(400, 700, 388, 860, 412, 700, CANARY_D, 1);
tri(470, 700, 490, 860, 484, 700, CANARY_D, 1);
tri(388, 848, 430, 858, 388, 862, CANARY, 1);
tri(490, 848, 540, 842, 490, 862, CANARY, 1);

function chunk(type, payload) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(payload.length, 0);
  const typeBuf = Buffer.from(type);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(zlib.crc32(Buffer.concat([typeBuf, payload])) >>> 0, 0);
  return Buffer.concat([len, typeBuf, payload, crc]);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8;
ihdr[9] = 6;

const raw = Buffer.alloc((SIZE * 4 + 1) * SIZE);
for (let y = 0; y < SIZE; y++) {
  raw[y * (SIZE * 4 + 1)] = 0;
  data.copy(raw, y * (SIZE * 4 + 1) + 1, y * SIZE * 4, (y + 1) * SIZE * 4);
}

const png = Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  chunk("IHDR", ihdr),
  chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

const out = path.join(__dirname, "..", "src-tauri", "icon-source.png");
fs.writeFileSync(out, png);
console.log("wrote", out, png.length);
