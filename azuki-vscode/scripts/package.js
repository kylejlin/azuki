const { execFileSync } = require('node:child_process');
const metadata = require('../package.json');

const target = `${process.platform}-${process.arch}`;
const repository = metadata.repository.url.replace(/\.git$/, '');
const contentUrl = `${repository}/blob/HEAD/${metadata.repository.directory}`;
execFileSync(process.platform === 'win32' ? 'vsce.cmd' : 'vsce', [
    'package', '--target', target, '--baseContentUrl', contentUrl, ...process.argv.slice(2),
], { cwd: require('node:path').resolve(__dirname, '..'), stdio: 'inherit' });
