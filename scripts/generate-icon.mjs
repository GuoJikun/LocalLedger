// 生成应用图标源图（1024x1024 PNG），再用 `pnpm tauri icon` 派生各平台图标。
// 仅使用 Node 内置模块，避免为一次性任务引入图形库依赖。
import { deflateSync } from "node:zlib";
import { writeFileSync, mkdirSync } from "node:fs";

const SIZE = 1024;
const pixels = Buffer.alloc(SIZE * SIZE * 4, 0);

function blend(x, y, [r, g, b, a = 255]) {
  if (x < 0 || y < 0 || x >= SIZE || y >= SIZE || a === 0) return;
  const i = (y * SIZE + x) * 4;
  if (a === 255) {
    pixels[i] = r;
    pixels[i + 1] = g;
    pixels[i + 2] = b;
    pixels[i + 3] = 255;
    return;
  }
  const na = a / 255;
  pixels[i] = Math.round(pixels[i] * (1 - na) + r * na);
  pixels[i + 1] = Math.round(pixels[i + 1] * (1 - na) + g * na);
  pixels[i + 2] = Math.round(pixels[i + 2] * (1 - na) + b * na);
  pixels[i + 3] = Math.max(pixels[i + 3], a);
}

function roundedRect(x0, y0, w, h, radius, color) {
  const r = radius;
  for (let y = y0; y < y0 + h; y++) {
    for (let x = x0; x < x0 + w; x++) {
      const dx = Math.max(x0 + r - x, 0, x - (x0 + w - r - 1));
      const dy = Math.max(y0 + r - y, 0, y - (y0 + h - r - 1));
      if (dx * dx + dy * dy <= r * r) blend(x, y, color);
    }
  }
}

function circle(cx, cy, radius, color) {
  for (let y = cy - radius; y <= cy + radius; y++) {
    for (let x = cx - radius; x <= cx + radius; x++) {
      if ((x - cx) ** 2 + (y - cy) ** 2 <= radius * radius) blend(x, y, color);
    }
  }
}

// 背景：蓝色圆角方块
roundedRect(0, 0, SIZE, SIZE, 220, [37, 99, 235]);
// 内置纸张：白色圆角矩形
roundedRect(256, 200, 512, 624, 48, [255, 255, 255]);
// 纸张上的三条账目横线
for (let i = 0; i < 3; i++) {
  roundedRect(320, 336 + i * 132, 320 - i * 42, 28, 14, [37, 99, 235]);
}
// 右下角绿色收入标记
circle(760, 720, 96, [15, 157, 110]);
circle(760, 720, 44, [255, 255, 255]);

// ---- 写 PNG ----
const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (const byte of buf) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const typeBuf = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])));
  return Buffer.concat([len, typeBuf, data, crc]);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // RGBA
const raw = Buffer.alloc((SIZE * 4 + 1) * SIZE);
for (let y = 0; y < SIZE; y++) {
  const rowStart = y * (SIZE * 4 + 1);
  raw[rowStart] = 0; // filter: none
  pixels.copy(raw, rowStart + 1, y * SIZE * 4, (y + 1) * SIZE * 4);
}

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

mkdirSync("scripts", { recursive: true });
writeFileSync("scripts/icon-source.png", png);
console.log("已生成 scripts/icon-source.png");
