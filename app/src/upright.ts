const shortOf = (bytes: Uint8Array, at: number, little: boolean): number =>
  little ? bytes[at] | (bytes[at + 1] << 8) : (bytes[at] << 8) | bytes[at + 1];

const longOf = (bytes: Uint8Array, at: number, little: boolean): number =>
  little
    ? (bytes[at] | (bytes[at + 1] << 8) | (bytes[at + 2] << 16) | (bytes[at + 3] << 24)) >>> 0
    : ((bytes[at] << 24) | (bytes[at + 1] << 16) | (bytes[at + 2] << 8) | bytes[at + 3]) >>> 0;

const inTiff = (bytes: Uint8Array, head: number): number => {
  if (head + 8 > bytes.length) return 1;
  const mark = shortOf(bytes, head, false);
  if (mark !== 0x4949 && mark !== 0x4d4d) return 1;
  const little = mark === 0x4949;
  const first = head + longOf(bytes, head + 4, little);
  if (first + 2 > bytes.length) return 1;
  const many = shortOf(bytes, first, little);
  for (let one = 0; one < many; one += 1) {
    const at = first + 2 + one * 12;
    if (at + 12 > bytes.length) return 1;
    if (shortOf(bytes, at, little) === 0x0112) {
      const turn = shortOf(bytes, at + 8, little);
      return turn >= 1 && turn <= 8 ? turn : 1;
    }
  }
  return 1;
};

export const orientedOf = (from: ArrayLike<number>): number => {
  const bytes = from instanceof Uint8Array ? from : Uint8Array.from(from);
  if (bytes[0] !== 0xff || bytes[1] !== 0xd8) return 1;
  let at = 2;
  while (at + 4 <= bytes.length) {
    if (bytes[at] !== 0xff) return 1;
    const marker = bytes[at + 1];
    if (marker === 0xda || marker === 0xd9) return 1;
    const size = shortOf(bytes, at + 2, false);
    if (size < 2) return 1;
    if (marker === 0xe1) {
      const head = at + 4;
      const said = String.fromCharCode(...bytes.slice(head, head + 4));
      if (said === "Exif") return inTiff(bytes, head + 6);
    }
    at += 2 + size;
  }
  return 1;
};

const SOF = [0xc0, 0xc1, 0xc2, 0xc3, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcd, 0xce, 0xcf];

export const sizedOf = (from: ArrayLike<number>): [number, number] => {
  const bytes = from instanceof Uint8Array ? from : Uint8Array.from(from);
  if (bytes[0] !== 0xff || bytes[1] !== 0xd8) return [0, 0];
  let at = 2;
  while (at + 4 <= bytes.length) {
    if (bytes[at] !== 0xff) return [0, 0];
    const marker = bytes[at + 1];
    if (marker === 0xda || marker === 0xd9) return [0, 0];
    const size = shortOf(bytes, at + 2, false);
    if (size < 2) return [0, 0];
    if (SOF.includes(marker)) {
      if (at + 9 > bytes.length) return [0, 0];
      return [shortOf(bytes, at + 7, false), shortOf(bytes, at + 5, false)];
    }
    at += 2 + size;
  }
  return [0, 0];
};

export const CAP = 2400;

export const scaled = (wide: number, tall: number, cap = CAP): [number, number] => {
  const most = Math.max(wide, tall);
  if (!most || most <= cap) return [wide, tall];
  const by = cap / most;
  return [Math.max(1, Math.round(wide * by)), Math.max(1, Math.round(tall * by))];
};

export const upright = (src: string): Promise<string> =>
  new Promise((resolve) => {
    if (typeof document === "undefined") return resolve(src);
    const seen = new Image();
    seen.onload = () => {
      try {
        const [wide, tall] = scaled(seen.naturalWidth, seen.naturalHeight);
        if (!wide || !tall) return resolve(src);
        const sheet = document.createElement("canvas");
        sheet.width = wide;
        sheet.height = tall;
        const pen = sheet.getContext("2d");
        if (!pen) return resolve(src);
        pen.drawImage(seen, 0, 0, wide, tall);
        resolve(sheet.toDataURL("image/jpeg", 0.92));
      } catch {
        resolve(src);
      }
    };
    seen.onerror = () => resolve(src);
    seen.src = src;
  });

export const redrawn = (from: ArrayLike<number>): boolean => {
  if (orientedOf(from) > 1) return true;
  const [wide, tall] = sizedOf(from);
  return Math.max(wide, tall) > CAP;
};
