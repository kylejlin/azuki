const vscode = require('vscode');
const path = require('node:path');
const { AnalysisClient } = require('./analysis-client');
const { FormattingProvider } = require('./formatting');
const { RustHighlighting } = require('./rust-highlighting');

const tokenTypes = ['namespace', 'type', 'struct', 'enum', 'enumMember', 'typeParameter',
    'property', 'variable', 'function', 'keyword', 'operator', 'string', 'number', 'comment'];
const legend = new vscode.SemanticTokensLegend(tokenTypes, ['declaration']);

function activate(context) {
    const selector = { language: 'azuki' };
    const diagnostics = vscode.languages.createDiagnosticCollection('Azuki');
    const output = vscode.window.createOutputChannel('Azuki');
    const changed = new vscode.EventEmitter();
    const client = new AnalysisClient(context, diagnostics, output, changed);
    const formatting = new FormattingProvider(context, output);
    const rustExtension = vscode.extensions.getExtension('vscode.rust');
    const rustPath = path.join(rustExtension ? rustExtension.extensionPath
        : path.join(vscode.env.appRoot, 'extensions/rust'), 'syntaxes/rust.tmLanguage.json');
    const rust = new RustHighlighting(rustPath);
    // Attach the handler immediately, including when no Rust injection is open yet.
    rust.ready.catch(error => output.appendLine(`Rust highlighting unavailable: ${error.message}`));
    context.subscriptions.push(diagnostics, output, changed, client, formatting, rust,
        vscode.languages.registerDocumentSemanticTokensProvider(selector, {
            onDidChangeSemanticTokens: changed.event,
            async provideDocumentSemanticTokens(document, cancellation) {
                const version = document.version;
                const analysis = await client.get(document);
                if (!analysis || cancellation.isCancellationRequested) return undefined;
                let embedded;
                try {
                    embedded = await rust.tokens(document.getText(), analysis.rustRegions);
                } catch (error) {
                    output.appendLine(`Rust highlighting unavailable: ${error.message}`);
                    embedded = analysis.rustRegions.map(span => ({ span, kind: 'variable', declaration: false }));
                }
                if (document.version !== version || cancellation.isCancellationRequested) return undefined;
                const builder = new vscode.SemanticTokensBuilder(legend);
                const tokens = [...analysis.tokens, ...embedded].sort((a, b) => a.span.start - b.span.start);
                for (const token of tokens) {
                    const start = document.positionAt(token.span.start);
                    const end = document.positionAt(token.span.end);
                    for (let line = start.line; line <= end.line; line++) {
                        const from = line === start.line ? start.character : 0;
                        const to = line === end.line ? end.character : document.lineAt(line).text.length;
                        if (to > from) builder.push(line, from, to - from, tokenTypes.indexOf(token.kind), token.declaration ? 1 : 0);
                    }
                }
                return builder.build();
            },
        }, legend),
        vscode.languages.registerDefinitionProvider(selector, {
            async provideDefinition(document, position, cancellation) {
                const found = await lookup(client, document, position, cancellation);
                return found && new vscode.Location(document.uri, range(document, found.definition.span));
            },
        }),
        vscode.languages.registerHoverProvider(selector, {
            async provideHover(document, position, cancellation) {
                const found = await lookup(client, document, position, cancellation);
                if (!found) return undefined;
                const markdown = new vscode.MarkdownString();
                markdown.appendCodeblock(found.definition.preview, 'azuki');
                if (found.occurrence.isSelf) markdown.appendText('\nSelf refers to this enclosing non-terminal.');
                return new vscode.Hover(markdown, range(document, found.occurrence.span));
            },
        }),
        vscode.languages.registerDocumentFormattingEditProvider(selector, formatting),
        vscode.languages.registerRenameProvider(selector, {
            async prepareRename(document, position, cancellation) {
                const found = await lookup(client, document, position, cancellation);
                requireRename(found);
                return { range: range(document, found.occurrence.span), placeholder: found.definition.name };
            },
            async provideRenameEdits(document, position, newName, cancellation) {
                const found = await lookup(client, document, position, cancellation);
                requireRename(found);
                const name = renameName(found.definition, newName);
                if (found.analysis.definitions.some(def => def !== found.definition
                    && def.scope === found.definition.scope && def.name === name)) {
                    throw new Error(`A declaration named ${name} already exists in this scope.`);
                }
                if (found.analysis.unresolvedReferences.some(reference => reference.scope === found.definition.scope
                    && reference.name === name)) {
                    throw new Error(`The name ${name} is already used by an unresolved reference. Renaming would change its meaning.`);
                }
                const edit = new vscode.WorkspaceEdit();
                for (const occurrence of found.analysis.occurrences) {
                    if (occurrence.target !== found.occurrence.target || occurrence.isSelf) continue;
                    let replacement = name;
                    if (occurrence.span.start === found.definition.span.start
                        && name !== found.definition.name && found.definition.implicitBridge) {
                        replacement += ` = ${found.definition.implicitBridge}`;
                    }
                    edit.replace(document.uri, range(document, occurrence.span), replacement);
                }
                return edit;
            },
        }),
        vscode.workspace.onDidOpenTextDocument(document => client.schedule(document)),
        vscode.workspace.onDidChangeTextDocument(event => client.schedule(event.document)),
        vscode.workspace.onDidCloseTextDocument(document => client.invalidate(document)),
        vscode.workspace.onDidChangeConfiguration(event => {
            if (event.affectsConfiguration('azuki.analysisPath')) {
                for (const document of vscode.workspace.textDocuments) client.schedule(document);
                changed.fire();
            }
        }),
    );
    for (const document of vscode.workspace.textDocuments) client.schedule(document);
}

async function lookup(client, document, position, cancellation) {
    const version = document.version;
    const analysis = await client.get(document);
    if (!analysis || document.version !== version || cancellation.isCancellationRequested) return undefined;
    const offset = document.offsetAt(position);
    // Prefer a token containing the cursor, then allow the usual cursor-at-word-end case.
    const occurrence = analysis.occurrences.find(item => item.span.start <= offset && offset < item.span.end)
        || analysis.occurrences.find(item => item.span.start < offset && offset === item.span.end);
    if (!occurrence) return undefined;
    const definition = analysis.definitions[occurrence.target];
    if (definition.ambiguous) return undefined;
    return { analysis, occurrence, definition };
}

function requireRename(found) {
    if (!found) throw new Error('Place the cursor on an unambiguous Azuki declaration or reference. Rust bridge names are outside grammar-local rename.');
    if (!found.analysis.renameAvailable) throw new Error('Correct syntax errors before renaming; incomplete items may contain additional references.');
    if (found.occurrence.isSelf) throw new Error('Rename the enclosing non-terminal declaration. Self remains spelled Self.');
}

function renameName(definition, value) {
    let name = value;
    let pattern;
    if (definition.name.startsWith('$')) {
        name = `$${value.replace(/^\$/, '')}`;
        pattern = /^\$[A-Z][a-zA-Z0-9_]*$/;
    } else if (definition.name.startsWith("'")) {
        name = `'${value.replace(/^'/, '')}`;
        pattern = /^'[a-z][a-zA-Z0-9_]*$/;
    } else if (definition.kind === 'property') {
        pattern = /^[a-z][a-zA-Z0-9_]*$/;
        if (['start', 'struct', 'enum', 'manual', 'terminal', 'bridge', 'crate'].includes(name)) pattern = /$a/;
    } else {
        pattern = /^[A-Z][a-zA-Z0-9_]*$/;
        if (name === 'Self') pattern = /$a/;
    }
    if (!pattern.test(name)) throw new Error('The new name must be a legal Azuki name of the same kind.');
    return name;
}

function range(document, span) {
    return new vscode.Range(document.positionAt(span.start), document.positionAt(span.end));
}

module.exports = { activate };
