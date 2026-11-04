// What finishes a document, built for a real browser with nothing around it.
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { build as bundled } from "vite";

const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const root = resolve(app, "..");

export const bundledFinisher = (files) =>
  bundled({
    root: app,
    configFile: false,
    logLevel: "error",
    resolve: {
      alias: {
        "tisty-diagrams": join(root, "packages", "tisty-diagrams", "src"),
        "tisty-editor": join(root, "packages", "tisty-editor", "src"),
        "@tisty/contracts": join(root, "contracts", "dist", "typescript", "index.js"),
      },
    },
    build: {
      outDir: join(files, "finisher"),
      emptyOutDir: true,
      lib: { entry: join(app, "src", "guide", "finishing.ts"), formats: ["es"], fileName: () => "finisher.js" },
    },
  });

export const finisherPage = (what) =>
  `<!doctype html><html><head><meta charset="utf-8"></head><body><script type="module">import("/finisher/finisher.js").then((one)=>one.${what}()).catch((error)=>fetch("/failed",{method:"POST",body:String(error?.stack??error)}))</script></body></html>`;
