import atkinsonExt from "@fontsource-variable/atkinson-hyperlegible-next/files/atkinson-hyperlegible-next-latin-ext-wght-normal.woff2?url";
import atkinson from "@fontsource-variable/atkinson-hyperlegible-next/files/atkinson-hyperlegible-next-latin-wght-normal.woff2?url";
import caveatExt from "@fontsource-variable/caveat/files/caveat-latin-ext-wght-normal.woff2?url";
import caveat from "@fontsource-variable/caveat/files/caveat-latin-wght-normal.woff2?url";
import frauncesExt from "@fontsource-variable/fraunces/files/fraunces-latin-ext-wght-normal.woff2?url";
import fraunces from "@fontsource-variable/fraunces/files/fraunces-latin-wght-normal.woff2?url";
import interExt from "@fontsource-variable/inter/files/inter-latin-ext-wght-normal.woff2?url";
import inter from "@fontsource-variable/inter/files/inter-latin-wght-normal.woff2?url";
import loraExt from "@fontsource-variable/lora/files/lora-latin-ext-wght-normal.woff2?url";
import lora from "@fontsource-variable/lora/files/lora-latin-wght-normal.woff2?url";
import nunitoExt from "@fontsource-variable/nunito/files/nunito-latin-ext-wght-normal.woff2?url";
import nunito from "@fontsource-variable/nunito/files/nunito-latin-wght-normal.woff2?url";
import sourceSerifExt from "@fontsource-variable/source-serif-4/files/source-serif-4-latin-ext-wght-normal.woff2?url";
import sourceSerif from "@fontsource-variable/source-serif-4/files/source-serif-4-latin-wght-normal.woff2?url";
import groteskExt from "@fontsource-variable/space-grotesk/files/space-grotesk-latin-ext-wght-normal.woff2?url";
import grotesk from "@fontsource-variable/space-grotesk/files/space-grotesk-latin-wght-normal.woff2?url";
import { type Design, isOwn } from "./designs";
import { FONTS, ownIn } from "./ownDesign";

const FILES: Readonly<Record<string, readonly [string, string]>> = {
  inter: [inter, interExt],
  atkinson: [atkinson, atkinsonExt],
  nunito: [nunito, nunitoExt],
  grotesk: [grotesk, groteskExt],
  source_serif: [sourceSerif, sourceSerifExt],
  lora: [lora, loraExt],
  fraunces: [fraunces, frauncesExt],
  caveat: [caveat, caveatExt],
};

const RANGES = [
  "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
  "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF",
] as const;

const WEIGHTS = "200 900";

const faceOf = (design: Design) => (isOwn(design) ? ownIn(design).font : "");

const addressed = (file: string) => new URL(file, document.baseURI).href;

// A font is asked for once in each document that wears it, and a page drawn before it lands is drawn in the system's.
export const fontLaid = async (page: Document, font: string): Promise<void> => {
  const face = FONTS[font];
  const Face = page.defaultView?.FontFace;
  if (!face || !Face || !page.fonts) return;
  if (!Array.from(page.fonts).some((one) => one.family.replace(/"/g, "") === face.family))
    FILES[font].forEach((file, at) => {
      page.fonts.add(
        new Face(face.family, `url("${addressed(file)}")`, {
          weight: WEIGHTS,
          unicodeRange: RANGES[at],
          display: "swap",
        }),
      );
    });
  await page.fonts.load(`400 16px "${face.family}"`).catch(() => {});
};

export const fontWorn = (page: Document, design: Design): Promise<void> =>
  fontLaid(page, faceOf(design));

const coded = async (file: string) => {
  const bytes = new Uint8Array(await (await fetch(addressed(file))).arrayBuffer());
  let said = "";
  for (let at = 0; at < bytes.length; at += 0x8000)
    said += String.fromCharCode(...bytes.subarray(at, at + 0x8000));
  return btoa(said);
};

// A page that leaves Tisty carries its font inside, since nothing it opens in has it, and without one it leaves all the same.
export const fontCarried = (design: Design): Promise<string> => faced(design).catch(() => "");

const faced = async (design: Design): Promise<string> => {
  const font = faceOf(design);
  const face = FONTS[font];
  if (!face) return "";
  const said = await Promise.all(
    FILES[font].map(
      async (file, at) =>
        `@font-face{font-family:"${face.family}";font-weight:${WEIGHTS};src:url(data:font/woff2;base64,${await coded(file)}) format("woff2");unicode-range:${RANGES[at]}}`,
    ),
  );
  return said.join("");
};
