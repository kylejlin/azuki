const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '../..');
const directory = path.resolve(__dirname, '../bin');
fs.mkdirSync(directory, { recursive: true });
for (const [packageName, target, name] of [
    ['azuki_ide', 'azuki_ide', 'azuki-ide'],
    ['azuki', 'azuki-fmt', 'azuki-fmt'],
]) {
    const output = execFileSync('cargo', [
        'build', '--release', '--manifest-path', path.join(root, packageName, 'Cargo.toml'),
        '--bin', target, '--message-format=json', ...process.argv.slice(2),
    ], { cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
    const artifacts = output.trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
    const artifact = artifacts.find(message => message.reason === 'compiler-artifact'
        && message.target.name === target && message.executable);
    if (!artifact) throw new Error(`Cargo did not produce the ${target} executable.`);
    const destination = path.join(directory, process.platform === 'win32' ? `${name}.exe` : name);
    fs.copyFileSync(artifact.executable, destination);
    fs.chmodSync(destination, 0o755);
    console.log(`Built ${destination}`);
}
