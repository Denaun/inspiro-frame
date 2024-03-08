import { type Raw } from "npm:sharp@0.33.2";

interface Rgb {
  r: number;
  g: number;
  b: number;
}

export function ditherInplace(
  data: Uint8Array,
  { width, height, channels }: Raw,
  palette: Rgb[],
) {
  const image = new ImageBuffer(data, width, height, channels);
  const errors = new ImageBuffer(Array.from(data), width, height, channels);

  for (let x = 0; x < image.width; x++) {
    for (let y = 0; y < image.height; y++) {
      const oldColor = errors.getPixel(x, y);
      const newColor = findClosest(oldColor, palette);
      image.setPixel(x, y, newColor);

      const error = sub(oldColor, newColor);
      // Sierra Lite
      errors.tryMadPixel(x + 1, y, error, 2 / 4);
      errors.tryMadPixel(x - 1, y + 1, error, 1 / 4);
      errors.tryMadPixel(x, y + 1, error, 1 / 4);
    }
  }
}

class ImageBuffer {
  constructor(
    readonly data: Uint8Array | number[],
    readonly width: number,
    readonly height: number,
    readonly channels: 1 | 2 | 3 | 4,
  ) {}

  getPixel(x: number, y: number): Rgb {
    const index = this.index(x, y);
    return {
      r: this.data[index],
      g: this.data[index + 1],
      b: this.data[index + 2],
    };
  }

  setPixel(x: number, y: number, { r, g, b }: Rgb) {
    const index = this.index(x, y);
    this.data[index] = r;
    this.data[index + 1] = g;
    this.data[index + 2] = b;
  }

  tryMadPixel(x: number, y: number, { r, g, b }: Rgb, factor: number) {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) {
      return;
    }
    const index = this.index(x, y);
    this.data[index] += r * factor;
    this.data[index + 1] += g * factor;
    this.data[index + 2] += b * factor;
  }

  private index(x: number, y: number): number {
    return (x + y * this.width) * this.channels;
  }
}

function findClosest(color: Rgb, palette: Rgb[]): Rgb {
  const distances = palette.map((c) => distance(color, c));
  const minIndex = distances.reduce(
    (minIndex, currentDistance, currentIndex) =>
      distances[minIndex] <= currentDistance ? minIndex : currentIndex,
    0,
  );
  return palette[minIndex];
}

function sub(lhs: Rgb, rhs: Rgb): Rgb {
  return {
    r: lhs.r - rhs.r,
    g: lhs.g - rhs.g,
    b: lhs.b - rhs.b,
  };
}

function distance(lhs: Rgb, rhs: Rgb): number {
  const r = lhs.r - rhs.r;
  const g = lhs.g - rhs.g;
  const b = lhs.b - rhs.b;
  return r * r + g * g + b * b;
}

export function packBuffer(
  data: Uint8Array,
  raw: Raw,
  color: Rgb,
): Uint8Array {
  const helper = new LsbHelper(data, raw, color);
  for (let x = 0; x < helper.width; x++) {
    for (let y = 0; y < helper.height; y++) {
      if (helper.isColor(x, y)) {
        helper.setPixel(x, y);
      }
    }
  }
  return helper.buffer;
}

class LsbHelper {
  readonly width: number;
  readonly height: number;
  readonly channels: 1 | 2 | 3 | 4;
  readonly buffer: Uint8Array;

  constructor(
    private readonly data: Uint8Array,
    { width, height, channels }: Raw,
    private readonly color: Rgb,
  ) {
    this.width = width;
    this.height = height;
    this.channels = channels;
    this.buffer = new Uint8Array(width * height / 8);
  }

  isColor(x: number, y: number): boolean {
    const index = (x + y * this.width) * this.channels;
    return (
      this.data[index] == this.color.r &&
      this.data[index + 1] == this.color.g &&
      this.data[index + 2] == this.color.b
    );
  }

  setPixel(x: number, y: number) {
    const index = Math.floor((x + y * this.width) / 8);
    this.buffer[index] |= 0x80 >> (x % 8);
  }
}
