import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, "..", "vos.tmLanguage.json");
const destDir = join(here, "..", "..", "vos-on-vscode", "syntaxes");
const dest = join(destDir, "vos.tmLanguage.json");

mkdirSync(destDir, { recursive: true });
copyFileSync(src, dest);
console.log(`synced ${src} -> ${dest}`);
