const fs = require('node:fs/promises');
const path = require('node:path');
const textmate = require('vscode-textmate');
const oniguruma = require('vscode-oniguruma');

class RustHighlighting {
    constructor(grammarPath) {
        this.ready = this.load(grammarPath);
    }

    async load(grammarPath) {
        const wasm = await fs.readFile(require.resolve('vscode-oniguruma/release/onig.wasm'));
        await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
        this.registry = new textmate.Registry({
            onigLib: Promise.resolve({
                createOnigScanner: patterns => new oniguruma.OnigScanner(patterns),
                createOnigString: text => new oniguruma.OnigString(text),
            }),
            loadGrammar: async scope => scope === 'source.rust'
                ? textmate.parseRawGrammar(await fs.readFile(grammarPath, 'utf8'), grammarPath) : null,
        });
        return this.registry.loadGrammar('source.rust');
    }

    async tokens(source, regions) {
        const grammar = await this.ready;
        if (!grammar) throw new Error('The built-in Rust grammar could not be loaded.');
        const highlights = [];
        for (const region of regions) {
            let state = textmate.INITIAL;
            let offset = region.start;
            for (const line of source.slice(region.start, region.end).split('\n')) {
                const result = grammar.tokenizeLine(line, state, 100);
                state = result.ruleStack;
                for (const token of result.tokens) {
                    const end = Math.min(token.endIndex, line.length);
                    if (end <= token.startIndex) continue;
                    highlights.push({
                        span: { start: offset + token.startIndex, end: offset + end },
                        kind: scopeKind(token.scopes), declaration: false,
                    });
                }
                offset += line.length + 1;
            }
        }
        return highlights;
    }

    dispose() {
        if (this.registry) this.registry.dispose();
    }
}

function scopeKind(scopes) {
    // Inner string/comment scopes take precedence over their containing Rust construct.
    for (const scope of [...scopes].reverse()) {
        if (scope.startsWith('comment')) return 'comment';
        if (scope.startsWith('string')) return 'string';
        if (scope.startsWith('constant.numeric')) return 'number';
        if (scope.startsWith('constant.character')) return 'string';
        if (scope.startsWith('constant.language')) return 'keyword';
        if (scope.startsWith('constant')) return 'enumMember';
        if (scope.startsWith('entity.name.function') || scope.startsWith('support.function')) return 'function';
        if (scope.includes('lifetime')) return 'typeParameter';
        if (scope.startsWith('entity.name.type') || scope.startsWith('support.type')
            || scope.startsWith('entity.name.trait') || scope.startsWith('storage.type')) return 'type';
        if (scope.startsWith('keyword.operator') || scope.startsWith('punctuation')) return 'operator';
        if (scope.startsWith('keyword') || scope.startsWith('storage')) return 'keyword';
        if (scope.startsWith('entity.name.namespace')) return 'namespace';
        if (scope.startsWith('variable')) return 'variable';
    }
    return 'variable';
}

module.exports = { RustHighlighting };
