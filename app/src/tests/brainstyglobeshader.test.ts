import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";
import { GLOBE_READ_NODE_DATA, globed, globeShaders } from "../ui/brainsty/globeShader";

const sigma = dirname(createRequire(import.meta.url).resolve("sigma/package.json"));
const factory = readdirSync(join(sigma, "dist")).filter((one) =>
  /^factory-[0-9a-f]+\.esm\.js$/.test(one),
);

describe("the globe's turn inside sigma's shaders", () => {
  it("is sewn into the sigma this app installs, which stays the one it was written for", () => {
    expect(JSON.parse(readFileSync(join(sigma, "package.json"), "utf8")).version).toBe("4.0.6");
    expect(factory).toHaveLength(1);
    const sewn = globed(readFileSync(join(sigma, "dist", factory[0]), "utf8"));

    expect(sewn).toContain(JSON.stringify(GLOBE_READ_NODE_DATA));
    expect(sewn.split("v_opacity = a_opacity * globeFade();")).toHaveLength(3);
    expect(sewn).not.toContain('var GLSL_READ_NODE_DATA = /*glsl*/"');
  });

  it("is GLSL that defines what sigma's programs call, with every number a float", () => {
    expect(GLOBE_READ_NODE_DATA).toContain(
      "vec4 readNodeData(sampler2D nodeDataTexture, int nodeDataTextureWidth, int nodeIndex) {",
    );
    expect(GLOBE_READ_NODE_DATA).toContain("float globeFade() {");
    expect(GLOBE_READ_NODE_DATA).not.toMatch(/undefined|NaN/);
    expect(GLOBE_READ_NODE_DATA).toContain("data.xy = 0.5 + vec2(x1, y2) * (1.0 + 0.15 * depth);");
    expect(GLOBE_READ_NODE_DATA).toContain("data.z *= mix(0.6, 1.15, near);");
    expect(GLOBE_READ_NODE_DATA).toContain("g_globeFadeSum += mix(0.3, 1.0, near);");
    expect(GLOBE_READ_NODE_DATA.split("{").length).toBe(GLOBE_READ_NODE_DATA.split("}").length);
  });

  it("refuses a sigma whose shaders no longer read as it expects", () => {
    expect(() => globed("var GLSL_READ_NODE_DATA = something else;")).toThrow(
      /sigma's shaders changed/,
    );
  });

  it("touches sigma's factory alone, whatever query the bundler adds", () => {
    const plugin = globeShaders();
    expect(plugin.transform("const a = 1;", "/x/node_modules/vite/client.js")).toBeNull();
    expect(() =>
      plugin.transform("const a = 1;", "/x/node_modules/sigma/dist/factory-63e3ce33.esm.js?v=1"),
    ).toThrow(/sigma's shaders changed/);
  });

  it("fails a build that never met sigma's factory, and leaves development and a failed build alone", () => {
    const built = globeShaders({ mustSew: true });
    built.transform("const a = 1;", "/x/node_modules/vite/client.js");
    expect(() => built.buildEnd()).toThrow(/never reached the bundle/);
    expect(() => built.buildEnd(new Error("broken"))).not.toThrow();
    expect(() => globeShaders().buildEnd()).not.toThrow();

    const sewn = globeShaders({ mustSew: true });
    sewn.transform(
      readFileSync(join(sigma, "dist", factory[0]), "utf8"),
      join(sigma, "dist", factory[0]),
    );
    expect(() => sewn.buildEnd()).not.toThrow();
  });
});
