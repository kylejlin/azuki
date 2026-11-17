const { execFileSync } = require('node:child_process');

const target = `${process.platform}-${process.arch}`;
execFileSync(process.platform === 'win32' ? 'vsce.cmd' : 'vsce', [
    'package', '--target', target, ...process.argv.slice(2),
], { cwd: require('node:path').resolve(__dirname, '..'), stdio: 'inherit' });
