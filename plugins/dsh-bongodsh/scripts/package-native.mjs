import { spawnSync } from 'node:child_process';
import { copyFile, mkdir, chmod, readFile, writeFile, rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

if (!['darwin', 'win32'].includes(process.platform)) {
  throw new Error('Desktop packages can only be built on macOS or Windows.');
}
const root = new URL('../../../', import.meta.url);
const build = spawnSync('cargo', ['build', '--workspace', '--release', '--locked'], {
  cwd: fileURLToPath(root), stdio: 'inherit', shell: false,
});
if (build.error || build.status !== 0) {
  throw new Error('Native build failed; no desktop binary was copied.');
}
const name = process.platform === 'win32' ? 'bongodsh.exe' : 'bongodsh';
const directory = new URL(`../native/${process.platform}-${process.arch}/`, import.meta.url);
await mkdir(directory, { recursive: true });
let target = new URL(name, directory);
if (process.platform === 'darwin') {
  const contents = new URL('BongoDSH Companion.app/Contents/', directory);
  await mkdir(new URL('MacOS/', contents), { recursive: true });
  const { version } = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error('The app bundle needs a numeric version.');
  await writeFile(new URL('Info.plist', contents), `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>bongodsh</string>
<key>CFBundleIdentifier</key><string>com.petal.bongodsh.companion</string>
<key>CFBundleName</key><string>BongoDSH Companion</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>${version}</string>
<key>CFBundleVersion</key><string>${version}</string>
<key>LSUIElement</key><true/>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
`);
  target = new URL('MacOS/bongodsh', contents);
}
await copyFile(new URL(`target/release/${name}`, root), target);
if (process.platform === 'darwin') {
  await chmod(target, 0o755);
  // Replace the previously generated loose binary with the app bundle.
  await rm(new URL(name, directory), { force: true });
}
console.log(`Desktop component prepared for ${process.platform}-${process.arch}. Run npm pack to package it.`);
