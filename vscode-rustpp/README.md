# RUST++ Language Support for VS Code

This extension adds `.rustqq` file recognition, syntax highlighting, bracket and comment support, indentation rules, and starter snippets for RUST++ v0.1.

## Build and install

Install the VS Code extension packaging tool and create a VSIX package:

```bash
npm install --global @vscode/vsce
cd vscode-rustpp
vsce package
code --install-extension rustpp-language-support-0.1.0.vsix
```

Restart or reload VS Code, then open a `.rustqq` file. Snippets include `let`, `print`, `ifelse`, `while`, `fn`, and `comment`.

To uninstall:

```bash
code --uninstall-extension black-210.rustpp-language-support
```

## Scope

This is editor support only: it does not bundle the RUST++ interpreter, run files, provide semantic diagnostics, or implement a language server. Run programs from a terminal with the Rust project, for example `cargo run -- path/to/file.rustqq` from the repository root.
