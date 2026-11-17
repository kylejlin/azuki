# Azuki analysis helper

This crate builds the native grammar analysis helper used by [Azuki VS Code](../azuki-vscode/README.md). The VS Code extension source, build scripts, and packaging instructions are in [`../azuki-vscode/`](../azuki-vscode/README.md).

Run `npm run build` from `azuki-vscode/` to build this crate and copy its executable into the extension's `bin/` directory. The helper reads one Azuki grammar from standard input and writes JSON analysis to standard output.
