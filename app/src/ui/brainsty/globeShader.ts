export const GLOBE = {
  perspective: 0.15,
  backFade: 0.3,
  frontFade: 1,
  backSize: 0.6,
  frontSize: 1.15,
} as const;

const float = (n: number): string => (Number.isInteger(n) ? `${n}.0` : String(n));

export const GLOBE_READ_NODE_DATA = `
uniform highp vec4 u_globeTurn;
uniform highp vec4 u_globeShape;
highp float g_globeFadeSum = 0.0;
highp float g_globeFadeCount = 0.0;

float globeHash(int index) {
  uint h = uint(index);
  h ^= h >> 16;
  h *= 0x7feb352du;
  h ^= h >> 15;
  h *= 0x846ca68bu;
  h ^= h >> 16;
  return float(h & 0xffffu) / 65535.0 * 2.0 - 1.0;
}

float globeSwell(vec2 p) {
  return 0.55 * sin(9.0 * p.x + 2.0 * sin(5.0 * p.y)) + 0.45 * cos(8.0 * p.y - 3.0 * p.x + 1.7);
}

float globeFade() {
  return g_globeFadeCount > 0.0 ? g_globeFadeSum / g_globeFadeCount : 1.0;
}

vec4 readNodeData(sampler2D nodeDataTexture, int nodeDataTextureWidth, int nodeIndex) {
  int t = nodeIndex * 2;
  ivec2 coord = ivec2(t % nodeDataTextureWidth, t / nodeDataTextureWidth);
  vec4 data = texelFetch(nodeDataTexture, coord, 0);
  if (u_globeShape.x < 0.5) return data;
  float radius = u_globeShape.y;
  vec2 d = data.xy - 0.5;
  float room = sqrt(max(radius * radius - dot(d, d), 0.0));
  float z = room * clamp(0.7 * globeSwell(data.xy) + 0.45 * globeHash(nodeIndex), -1.0, 1.0);
  float x1 = d.x * u_globeTurn.x + z * u_globeTurn.y;
  float z1 = -d.x * u_globeTurn.y + z * u_globeTurn.x;
  float y2 = d.y * u_globeTurn.z - z1 * u_globeTurn.w;
  float z2 = d.y * u_globeTurn.w + z1 * u_globeTurn.z;
  float depth = radius > 0.0 ? z2 / radius : 0.0;
  float near = 0.5 + 0.5 * depth;
  data.xy = 0.5 + vec2(x1, y2) * (1.0 + ${float(GLOBE.perspective)} * depth);
  data.z *= mix(${float(GLOBE.backSize)}, ${float(GLOBE.frontSize)}, near);
  g_globeFadeSum += mix(${float(GLOBE.backFade)}, ${float(GLOBE.frontFade)}, near);
  g_globeFadeCount += 1.0;
  return data;
}
`;

const STRIDE = "var NODE_DATA_TEXELS_PER_NODE = 2;";
const READ_NODE_DATA =
  'var GLSL_READ_NODE_DATA = /*glsl*/"\\nvec4 readNodeData(sampler2D nodeDataTexture, int nodeDataTextureWidth, int nodeIndex) {\\n  int t = nodeIndex * ".concat(NODE_DATA_TEXELS_PER_NODE, ";\\n  ivec2 coord = ivec2(t % nodeDataTextureWidth, t / nodeDataTextureWidth);\\n  return texelFetch(nodeDataTexture, coord, 0);\\n}\\n");';
const OPACITY = "v_opacity = a_opacity;";

export const SIGMA_FACTORY = /sigma[\\/]dist[\\/]factory-[0-9a-f]+\.esm\.js$/;

const counted = (source: string, part: string): number => source.split(part).length - 1;

// sigma has no hook into the GLSL every program shares, so its source is rewritten when bundled; a new sigma fails here.
export function globed(source: string): string {
  const found = [
    counted(source, STRIDE),
    counted(source, READ_NODE_DATA),
    counted(source, OPACITY),
  ];
  if (found.join() !== "1,1,2")
    throw new Error(`sigma's shaders changed (found ${found.join()} of 1,1,2): see globeShader.ts`);
  return source
    .replace(
      READ_NODE_DATA,
      () => `var GLSL_READ_NODE_DATA = ${JSON.stringify(GLOBE_READ_NODE_DATA)};`,
    )
    .split(OPACITY)
    .join("v_opacity = a_opacity * globeFade();");
}

export const globeShaders = {
  name: "tisty-globe-shaders",
  transform(code: string, id: string) {
    return SIGMA_FACTORY.test(id.split("?")[0]) ? { code: globed(code), map: null } : null;
  },
};
