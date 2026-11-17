# Private distribution

From this directory, package the extension using `vsce`:

```sh
vsce package --no-dependencies
```

If `vsce` is unavailable, install it with `npm install -g @vscode/vsce`.

In VS Code, run **Extensions: Install from VSIX...** and select `azuki-syntax-0.0.1.vsix`.
