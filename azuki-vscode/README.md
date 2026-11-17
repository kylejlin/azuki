# Azuki

The Azuki extension provides these features for [Azuki](https://github.com/kylejlin/azuki) grammars (`.azuki` files):

- Parser-aware highlighting for declarations and references.
- Rust coloring inside injection strings, including multiline strings and matching hash delimiters. Azuki's lexer determines each injection's boundary; the built-in VS Code Rust grammar colors its contents independently.
- Go to Definition (including Command-click on macOS) and hover showing the grammar definition. `Self` resolves to its enclosing non-terminal.
- Rename Symbol for terminals, non-terminals, terminal enum names, lifetimes, named fields, and enum variants. References update within the current grammar. Renaming shorthand declarations inserts an explicit `= @OldName` where necessary to preserve the existing bridge type. `Self`, comments, and injected Rust remain unchanged.
- Format Document and formatting on save, using Azuki's existing formatter. Files with lexer or parser errors stay unchanged. Preflight or parser-table errors do not prevent formatting. Rust injection strings are preserved verbatim.
- Live error squiggles from Azuki's lexer, parser, preflight checks, and parser-table conflict checks. Duplicate declarations include a related location for the first occurrence.

Navigation and rename are grammar-local. Rust bridge references (`@Type`, `@function`, and `bridge` paths), generated Rust files, and Rust injection contents do not participate in navigation or rename. Rename requires a syntactically complete grammar and rejects name collisions, including capture of unresolved references. Rust coloring does not provide Rust type checking.

Azuki files format when saved (Command-S on macOS). The extension defaults `editor.defaultFormatter` to `kylejlin.azuki-vscode` and `editor.formatOnSave` to `true` for Azuki. Explicit user or workspace settings can override these defaults. You can also run **Format Document** manually. The formatter uses four spaces per indent and a preferred line width of 80; VS Code's indentation options do not change its style.

The extension uses VS Code providers, a bundled native helper from the [`azuki_ide` crate](../azuki_ide/README.md), and the `azuki-fmt` command from the `azuki` crate. The analysis helper checks grammars in a separate process, with a short edit debounce, cancellation of outdated checks, and a ten-second process limit. Formatting also runs in a separate process with cancellation and a ten-second limit; results for an outdated document version are discarded. Rust coloring runs in the extension host. Diagnostics report the first Azuki lexer, parser, preflight, or parser-table error. Independently parseable items can still provide navigation while another item is incomplete; recovery is limited when delimiters are unbalanced.

## Source layout

- `src/`: VS Code providers, the helper client, formatting, and Rust coloring.
- `images/`: the extension icon and its artwork licensing notice.
- `scripts/build.js`: builds the Rust analysis helper and formatter and copies them into the extension.
- `scripts/package.js`: packages the extension for the current OS and architecture.
- [`../azuki_ide/`](../azuki_ide/README.md): Rust grammar analysis used by the extension.

## Development

From this directory:

```sh
npm ci
npm run build
```

The build requires the workspace's Rust toolchain. It builds `azuki_ide` and `azuki-fmt` and copies both executables into `bin/`. Cargo flags can be passed through, for example `npm run build -- --offline`.

Open this directory in VS Code and press F5. Open an `.azuki` file in the Extension Development Host window. Semantic highlighting is enabled by default for Azuki; an explicit user setting can override it. The original TextMate grammar remains available as a fallback.

To use a separately built helper, set `azuki.analysisPath` to its absolute path. This override is honored only in trusted workspaces.

## Packaging

With `vsce` available, run `npm run package`. This builds the helper and formatter and packages the extension with its runtime dependencies for the current OS and architecture. Build a separate package on each target platform.

To install a packaged extension, run **Extensions: Install from VSIX...** in VS Code and select the `.vsix` file.

If you have the earlier `kylejlin.azuki-syntax` extension installed, disable or uninstall it when switching to `kylejlin.azuki-vscode`, to avoid duplicate Azuki language support. VS Code uses [`publisher.name` as the extension ID](https://code.visualstudio.com/api/references/extension-manifest), so the renamed package installs separately.

## Icon artwork

The extension icon adapts Echo's walking pose from _Enbito_ (2023), with shorter
antlers, brighter colors, and a cream background for legibility at small sizes
and in light and dark themes. Its artwork is reserved separately from the
extension's MIT license; see the [artwork notice](images/NOTICE.md).

The source recipe and generator are in `../mascots_playground/`. From the
repository root, run `python3 mascots_playground/scripts/generate_extension_icon.py`
to regenerate `images/icon.png` and the larger playground export.
