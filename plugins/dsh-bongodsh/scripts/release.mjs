import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, mkdir, mkdtemp, readFile, rename, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const directory = fileURLToPath(new URL('../', import.meta.url));
const root = new URL('../../../', import.meta.url);
const manifest = JSON.parse(await readFile(join(directory, 'package.json'), 'utf8'));
const cargo = await readFile(new URL('Cargo.toml', root), 'utf8');
const cargoVersion = cargo.match(/\[workspace\.package\]\s+version\s*=\s*"([^"]+)"/)?.[1];
if (cargoVersion !== manifest.version) throw new Error('Rust and plugin versions must match.');
if (!process.env.npm_execpath) throw new Error('Run this script with npm run release.');

function run(command, args, cwd = directory, capture = false) {
  const result = spawnSync(command, args, {
    cwd, shell: false, encoding: 'utf8', stdio: capture ? 'pipe' : 'inherit',
  });
  if (result.error || result.status !== 0) {
    throw new Error(result.error?.message || result.stderr || `${command} failed.`);
  }
  return result.stdout;
}

run(process.execPath, ['scripts/build.mjs']);
run(process.execPath, [process.env.npm_execpath, 'run', 'check']);
run(process.execPath, ['scripts/package-native.mjs']);

const dist = join(directory, 'dist');
await mkdir(dist, { recursive: true });
const staging = await mkdtemp(join(tmpdir(), 'bongodsh-release-'));
const artifacts = [];
try {
  for (const name of ['package.json', ...manifest.files.filter(name => name !== 'native')]) {
    await cp(join(directory, name), join(staging, name), { recursive: true });
  }
  async function pack(suffix) {
    const result = JSON.parse(run(process.execPath, [
      process.env.npm_execpath, 'pack', '--json', '--ignore-scripts', '--pack-destination', dist,
    ], staging, true));
    const name = `${manifest.name}-${manifest.version}-${suffix}.tgz`;
    await rename(join(dist, result[0].filename), join(dist, name));
    artifacts.push(name);
  }
  await pack('web');
  const platform = `${process.platform}-${process.arch}`;
  await cp(join(directory, 'native', platform), join(staging, 'native', platform), { recursive: true });
  await pack(platform);
  const checksums = await Promise.all(artifacts.map(async name => {
    const hash = createHash('sha256').update(await readFile(join(dist, name))).digest('hex');
    return `${hash}  ${name}`;
  }));
  await writeFile(join(dist, 'SHA256SUMS.txt'), `${checksums.join('\n')}\n`);
  console.log(`Release ${manifest.version}:\n${[...artifacts, 'SHA256SUMS.txt'].map(name => join(dist, name)).join('\n')}`);
} finally {
  await rm(staging, { recursive: true, force: true });
}
