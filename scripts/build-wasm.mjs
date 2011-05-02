/**
 * Builds the VOS language WebAssembly artifact and stages it into
 * `projects/packages/vos-language/lib`, which is the directory the npm package
 * publishes from and which is gitignored (generated output only).
 *
 * Two steps, both deliberately run from outside the repository:
 *  1. `cargo build -p vos-wasm --target wasm32-unknown-unknown --release`
 *  2. `wasm-bindgen <artifact> --target web --out-dir <package>/lib`
 *
 * Running from outside the repository keeps the developer-only
 * `.cargo/config.toml` `[patch]` (which points at a sibling Oak checkout that
 * usually does not exist) from shadowing the `oak-vos` revision pinned in
 * `Cargo.toml`. `getrandom` needs its WebAssembly backend flag because `uuid`
 * pulls it into the graph.
 */

import { execFileSync, execSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const manifest = join(root, 'Cargo.toml');
const outDir = join(root, 'projects/packages/vos-language/lib');
const artifact = join(root, 'target/wasm32-unknown-unknown/release/vos_wasm.wasm');

const LIB_PACKAGE_JSON = {
    name: 'vos-wasm',
    type: 'module',
    collaborators: ['Yggdrasill Team <192607617@qq.com>'],
    description: 'WebAssembly binding for the VOS language toolchain',
    version: '0.0.0',
    license: 'MPL-2.0',
    repository: { type: 'git', url: 'https://github.com/voml/vos-language' },
    files: ['vos_wasm_bg.wasm', 'vos_wasm.js', 'vos_wasm.d.ts'],
    main: 'vos_wasm.js',
    types: 'vos_wasm.d.ts',
    sideEffects: ['./snippets/*'],
};

function run(command, args, env) {
    execFileSync(command, args, {
        cwd: tmpdir(),
        stdio: 'inherit',
        // No shell: these invocations take absolute paths that contain spaces.
        shell: false,
        env: env ?? process.env,
    });
}

run('cargo', [
    '+nightly-2026-07-20',
    'build',
    '-p',
    'vos-wasm',
    '--target',
    'wasm32-unknown-unknown',
    '--release',
    '--manifest-path',
    manifest,
], { ...process.env, RUSTFLAGS: '--cfg getrandom_backend="wasm_js"' });

mkdirSync(outDir, { recursive: true });

// `wasm-bindgen` refuses output when the CLI and the crate versions differ, and
// `wasm-pack` (which would install the right CLI itself) cannot be used here:
// it invokes cargo from inside the crate, which picks up the repository-local
// `[patch]`. So read the required version from the lock and align the CLI.
const requiredBindgen = /name = "wasm-bindgen"\nversion = "([^"]+)"/.exec(readFileSync(join(root, 'Cargo.lock'), 'utf8'))?.[1];

function installedBindgenVersion() {
    try {
        return /(\d+\.\d+\.\d+)/.exec(execSync('wasm-bindgen --version', { encoding: 'utf8' }))?.[1];
    } catch {
        // No CLI installed at all — that is a mismatch to fix below.
        return undefined;
    }
}

if (requiredBindgen !== installedBindgenVersion()) {
    console.log(`wasm-bindgen CLI does not match crate ${requiredBindgen} — installing the required CLI`);
    run('cargo', ['install', '-f', 'wasm-bindgen-cli', '--version', requiredBindgen]);
}

run('wasm-bindgen', [artifact, '--target', 'web', '--out-dir', outDir]);
writeFileSync(join(outDir, 'package.json'), `${JSON.stringify(LIB_PACKAGE_JSON, null, 2)}\n`);
