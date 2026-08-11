import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");

const candidates = [
  "node_modules/3dmol/build/3Dmol-min.js",
  "node_modules/3dmol/build/3Dmol.js",
];

const src = candidates.map((c) => join(root, c)).find((p) => existsSync(p));
if (!src) {
  console.error(
    "copy-3dmol: could not find a 3dmol build under node_modules/3dmol/build/ — checked: " +
      candidates.join(", "),
  );
  process.exit(1);
}

const destDir = join(root, "public/vendor/3dmol");
mkdirSync(destDir, { recursive: true });
const dest = join(destDir, "3Dmol.js");
copyFileSync(src, dest);
console.log(`copy-3dmol: vendored ${src} -> ${dest}`);
