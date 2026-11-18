# RUST++ v1.0

**RUST++** is a small, friendly, dynamically typed programming language implemented in Rust. It is designed to be approachable for beginners while providing the core tools needed for real programs: variables, functions, arrays, loops, conditionals, input, and useful built-in functions.

RUST++ is its own interpreted language. It is not Rust syntax, a Rust compiler, or a replacement for Rust. Source files use the `.rustqq` extension.

## Get started

Install [Rust and Cargo](https://www.rust-lang.org/tools/install), then run the included example:

```bash
cargo run -- examples/hello.rustqq
```

Build an optimized executable:

```bash
cargo build --release
./target/release/rustpp examples/hello.rustqq
```

Start the interactive shell with `cargo run`. Type `exit` to leave it. Get command-line help with `cargo run -- --help` and the version with `cargo run -- --version`.

## Your first program

```rust
let name = "world";
print "Hello, " + name + "!";

fn square(number) {
    return number * number;
}

print square(6);
```

## Features in v1.0

- Numbers, strings, booleans, `nil`, and mutable arrays.
- `let` variables and assignment; lexical block scopes.
- Named functions, parameters, return values, and recursion.
- `if` / `else`, `while`, C-style `for`, `break`, and `continue`.
- Arithmetic, comparisons, short-circuit logic (`&&`, `||`, `!`), and word aliases (`and`, `or`, `not`).
- Array indexing and assignment, concatenation, and built-ins: `len`, `push`, `pop`, `type`, `str`, `number`, `input`, `assert`.
- `//` line comments, useful English diagnostics, a file runner, and an interactive shell.

See the complete [language guide](docs/LANGUAGE.md), runnable [examples](examples/), and the [systems and AI language specification](docs/V2_LANGUAGE_SPEC.md).

## Important systems and AI status

RUST++ v1 is an interpreter, not a systems compiler. It cannot build or boot an operating system/kernel, access physical memory or devices, or run AI models. `memory::`, `hardware::`, `cpu::`, `boot::`, and `ai::` are future design goals documented in [V2_LANGUAGE_SPEC.md](docs/V2_LANGUAGE_SPEC.md), not available APIs. The specification describes the safety boundaries and implementation milestones needed before claiming those capabilities.

## Tests

```bash
cargo test
```

## v1 scope

This is a compact, dependency-free v1 interpreter. It currently does not provide classes, modules/package management, static types, closures, exceptions, or a bytecode/native compiler. Those are future language/runtime features, not implied by the v1 syntax. Contributions and issue reports are welcome.
