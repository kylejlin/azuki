const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '../..');
const output = execFileSync('cargo', [
    'build', '--release', '--package', 'azuki_ide', '--message-format=json',
    ...process.argv.slice(2),
], { cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
const artifacts = output.trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
const artifact = artifacts.find(message => message.reason === 'compiler-artifact'
    && message.target.name === 'azuki_ide' && message.executable);
if (!artifact) throw new Error('Cargo did not produce the azuki_ide executable.');
const directory = path.resolve(__dirname, '../bin');
fs.mkdirSync(directory, { recursive: true });
const destination = path.join(directory, process.platform === 'win32' ? 'azuki-ide.exe' : 'azuki-ide');
fs.copyFileSync(artifact.executable, destination);
fs.chmodSync(destination, 0o755);
console.log(`Built ${destination}`);
