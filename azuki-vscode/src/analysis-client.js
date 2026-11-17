const vscode = require('vscode');
const { execFile } = require('node:child_process');
const path = require('node:path');

class AnalysisClient {
    constructor(context, diagnostics, output, changed) {
        this.context = context;
        this.diagnostics = diagnostics;
        this.output = output;
        this.changed = changed;
        this.entries = new Map();
        this.timers = new Map();
        this.reported = new Set();
    }

    get(document) {
        const key = document.uri.toString();
        const existing = this.entries.get(key);
        if (existing && existing.version === document.version) return existing.promise;
        this.invalidate(document);
        const version = document.version;
        const configured = vscode.workspace.getConfiguration('azuki', document.uri).get('analysisPath', '');
        const executable = configured && vscode.workspace.isTrusted ? configured
            : path.join(this.context.extensionPath, 'bin', process.platform === 'win32' ? 'azuki-ide.exe' : 'azuki-ide');
        const entry = { version, child: undefined, promise: undefined };
        entry.promise = new Promise(resolve => {
            if (!path.isAbsolute(executable)) {
                this.failure(document, 'azuki.analysisPath must be an absolute path.');
                resolve(undefined);
                return;
            }
            entry.child = execFile(executable, [], {
                encoding: 'utf8', timeout: 10000, maxBuffer: 16 * 1024 * 1024, windowsHide: true,
            }, (error, stdout, stderr) => {
                if (this.entries.get(key) !== entry || document.version !== version || document.isClosed) {
                    resolve(undefined);
                    return;
                }
                if (error) {
                    this.failure(document, error.code === 'ENOENT'
                        ? 'Azuki analysis executable is missing. Run npm run build in azuki-vscode, or set azuki.analysisPath.'
                        : `Azuki analysis failed or exceeded its 10-second limit: ${stderr || error.message}`);
                    resolve(undefined);
                    return;
                }
                try {
                    const analysis = JSON.parse(stdout);
                    const range = span => new vscode.Range(document.positionAt(span.start), document.positionAt(span.end));
                    this.diagnostics.set(document.uri, analysis.diagnostics.map(item => {
                        const diagnostic = new vscode.Diagnostic(range(item.span), item.message, vscode.DiagnosticSeverity.Error);
                        diagnostic.source = 'Azuki';
                        diagnostic.relatedInformation = item.related.map(related => new vscode.DiagnosticRelatedInformation(
                            new vscode.Location(document.uri, range(related.span)), related.message,
                        ));
                        return diagnostic;
                    }));
                    resolve(analysis);
                } catch (error) {
                    this.failure(document, `Invalid Azuki analysis response: ${error.message}`);
                    resolve(undefined);
                }
            });
            // Failed launches can close stdin before the source has been written.
            entry.child.stdin.on('error', () => {});
            entry.child.stdin.end(document.getText());
        });
        this.entries.set(key, entry);
        return entry.promise;
    }

    schedule(document) {
        if (document.languageId !== 'azuki') return;
        this.invalidate(document);
        const key = document.uri.toString();
        this.timers.set(key, setTimeout(async () => {
            this.timers.delete(key);
            if (document.isClosed) return;
            if (await this.get(document)) this.changed.fire();
        }, 200));
    }

    invalidate(document) {
        const key = document.uri.toString();
        clearTimeout(this.timers.get(key));
        this.timers.delete(key);
        const entry = this.entries.get(key);
        this.entries.delete(key);
        if (entry && entry.child) entry.child.kill();
        this.diagnostics.delete(document.uri);
    }

    failure(document, message) {
        this.output.appendLine(message);
        const diagnostic = new vscode.Diagnostic(new vscode.Range(0, 0, 0, 0), message, vscode.DiagnosticSeverity.Warning);
        diagnostic.source = 'Azuki tooling';
        this.diagnostics.set(document.uri, [diagnostic]);
        if (!this.reported.has(message)) {
            this.reported.add(message);
            vscode.window.showWarningMessage(message);
        }
    }

    dispose() {
        for (const timer of this.timers.values()) clearTimeout(timer);
        for (const entry of this.entries.values()) if (entry.child) entry.child.kill();
        this.timers.clear();
        this.entries.clear();
    }
}

module.exports = { AnalysisClient };
