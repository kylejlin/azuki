const vscode = require('vscode');
const { execFile } = require('node:child_process');
const path = require('node:path');

class FormattingProvider {
    constructor(context, output) {
        this.context = context;
        this.output = output;
        this.children = new Set();
        this.disposed = false;
    }

    provideDocumentFormattingEdits(document, options, cancellation) {
        if (this.disposed || document.isClosed || cancellation.isCancellationRequested) return [];
        const version = document.version;
        const source = document.getText();
        const executable = path.join(this.context.extensionPath, 'bin',
            process.platform === 'win32' ? 'azuki-fmt.exe' : 'azuki-fmt');
        return new Promise((resolve, reject) => {
            const child = execFile(executable, [], {
                encoding: 'utf8', timeout: 10000, maxBuffer: 16 * 1024 * 1024, windowsHide: true,
            }, (error, stdout, stderr) => {
                this.children.delete(child);
                subscription.dispose();
                if (this.disposed || document.isClosed || document.version !== version
                    || cancellation.isCancellationRequested) {
                    resolve([]);
                    return;
                }
                if (error) {
                    const message = error.code === 'ENOENT'
                        ? 'Azuki formatter executable is missing. Run npm run build in azuki-vscode.'
                        : `Azuki formatting failed or exceeded its 10-second limit: ${stderr || error.message}`;
                    this.output.appendLine(message);
                    reject(new Error(message));
                    return;
                }
                resolve(formattingEdits(document, source, stdout));
            });
            const subscription = cancellation.onCancellationRequested(() => child.kill());
            this.children.add(child);
            // Failed launches can close stdin before the source has been written.
            child.stdin.on('error', () => {});
            child.stdin.end(source);
        });
    }

    dispose() {
        this.disposed = true;
        for (const child of this.children) child.kill();
        this.children.clear();
    }
}

function formattingEdits(document, source, formatted) {
    if (source === formatted) return [];
    let start = 0;
    while (start < source.length && start < formatted.length && source[start] === formatted[start]) start++;
    if (splitsPair(source, start) || splitsPair(formatted, start)) start--;
    let end = source.length;
    let formattedEnd = formatted.length;
    while (end > start && formattedEnd > start && source[end - 1] === formatted[formattedEnd - 1]) {
        end--;
        formattedEnd--;
    }
    if (splitsPair(source, end) || splitsPair(formatted, formattedEnd)) {
        end++;
        formattedEnd++;
    }
    return [vscode.TextEdit.replace(
        new vscode.Range(document.positionAt(start), document.positionAt(end)), formatted.slice(start, formattedEnd),
    )];
}

function splitsPair(text, offset) {
    return (text[offset - 1] === '\r' && text[offset] === '\n')
        || (/[\uD800-\uDBFF]/.test(text[offset - 1]) && /[\uDC00-\uDFFF]/.test(text[offset]));
}

module.exports = { FormattingProvider };
