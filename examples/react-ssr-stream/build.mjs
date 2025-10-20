import { mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const outFile = resolve(__dirname, "dist/app.bundle.js");

await mkdir(dirname(outFile), { recursive: true });

await build({
  entryPoints: [resolve(__dirname, "src/stream.tsx")],
  outfile: outFile,
  bundle: true,
  platform: "node",
  format: "esm",
  target: ["node18"],
  sourcemap: true,
  tsconfig: resolve(__dirname, "tsconfig.json"),
  jsx: "automatic",
  logLevel: "info"
});

console.log("Bundle generated at", outFile);
