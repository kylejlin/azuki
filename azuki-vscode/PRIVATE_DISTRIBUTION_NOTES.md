# Azuki VS Code private distribution

From this directory, install dependencies and package the extension using `vsce`:

```sh
npm ci
npm run package
```

If `vsce` is unavailable, install it with `npm install -g @vscode/vsce`.

The package includes the native analysis helper and JavaScript runtime dependencies. Do not use `--no-dependencies`; Rust highlighting needs those dependencies. The package targets the build machine's OS and architecture.

In VS Code, run **Extensions: Install from VSIX...** and select the generated `azuki-vscode` `.vsix` file. See the [README](README.md#packaging) for replacing the earlier extension.
