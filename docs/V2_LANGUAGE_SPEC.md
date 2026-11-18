# RUST++ Next: simple systems and AI language specification

> **Status: design specification, not implemented compiler features.** The repository currently contains a Rust-hosted tree-walking interpreter for beginner programs. It does not emit machine code, boot an operating system, run an AI model, or access physical memory/devices. Examples marked **future syntax** below are plans, not runnable programs. This document defines a practical path toward those goals without pretending the current interpreter can do them.

## Product goal

RUST++ should make ordinary programs feel direct and forgiving, while allowing an expert to opt into explicit systems capabilities. The same readable language should eventually support application code, AI workloads, embedded programs, and freestanding kernels. It should be easier to start with than Python, but simplicity must not hide dangerous hardware behavior.

The language uses English keywords and diagnostics. Namespaces such as `memory::` and `hardware::` are compiler/platform interfaces, not magic access to every device.

## What exists today

The current interpreter supports dynamically typed numbers, strings, booleans, `nil`, arrays, variables, named functions, conditionals, loops, and a small set of built-ins. Run its supported examples with `cargo run -- examples/hello.rustqq`; run tests with `cargo test`.

The following do **not** exist in the current executable: `memory::`, `hardware::`, `ai::`, fixed-width integer types, static type inference, module imports, machine-code output, a freestanding runtime, boot support, drivers, or compiler error codes. Use the existing language guide for the implemented v1 syntax.

## Beginner-first syntax direction

Local values should infer their types, so a beginner does not need declarations such as `let count: i32`:

```rust
// Future syntax: type inference is a compiler goal, not implemented today.
count = 0;
count = count + 1;
print count;
```

A declaration keyword may be optional for unambiguous new local names in a future language version. At module, public API, hardware layout, and ABI boundaries, explicit types remain available and encouraged. Inference must never guess a hardware layout or pointer type.

The compiler should offer a one-command workflow (`rustpp run app.rustqq`, eventually `rustpp build --target ...`) and explain each error with a short fix. Compatibility with current v1 files should be preserved or covered by a documented migration.

## Planned built-in namespaces

These are API design goals only. Each operation must be implemented and tested for a declared target before it is advertised as available.

| Namespace | Intended role | Required restriction |
| --- | --- | --- |
| `memory::` | Sizes, aligned regions, allocation, and mapping interfaces | No implicit integer-to-pointer conversion; validate bounds, alignment, ownership, and permissions. |
| `hardware::` | Enumerate supported platform devices and request a typed device handle | Only supported devices and drivers; no arbitrary device probing. |
| `cpu::` | A small set of target-specific processor operations | Compile-time target and execution-context checks. |
| `io::` | Approved device I/O interfaces | Explicit capability/privilege and target requirements. |
| `interrupts::` | Register and configure handlers | Validate handler signature, ABI, and restrictions on blocking/allocation. |
| `boot::` | Entry point and boot-protocol information | Explicit boot protocol and ABI; no claim that an ordinary executable is bootable. |
| `console::` / `serial::` | Early diagnostic output | Only available where the target provides an implementation. |
| `ai::` | Tensor, dataset, model, and inference APIs | A library/runtime interface; not a built-in claim that a model is trained or hardware-accelerated. |

### Future syntax: kernel entry

```rust
// FUTURE SYNTAX — not accepted by the current interpreter.
@target("x86_64-unknown-none")
@entry
fn kernel_start(info: boot::Info) -> never {
    console::write("kernel started\n");
    cpu::halt_forever();
}
```

A real implementation needs a code-generating compiler, documented calling convention, linker script or equivalent, bootloader protocol, panic behavior, and emulator tests. `cpu::halt_forever()` must not quietly compile for an unsupported target.

### Future syntax: memory and devices

```rust
// FUTURE SYNTAX — illustration only; never run against real hardware today.
let uart = hardware::find<Uart>("serial0")?;
uart.write("hello\n");

let bytes = memory::allocate(4096, align = 4096)?;
```

Hardware APIs should prefer typed, checked handles. Any raw physical address or volatile operation belongs behind a narrowly-scoped `unsafe` block with a compiler diagnostic explaining the risk. The compiler must not turn a plain device name into an unchecked write.

## AI support direction

AI conveniences should be ordinary documented modules that run on a defined backend, not mysterious syntax that hides memory use or model behavior. Keep the first APIs small and inspectable: tensors with explicit shape, dataset loading, deterministic preprocessing, model construction, training/inference calls, and CPU-first examples. Optional accelerators must be selected explicitly and report when unavailable.

```rust
// FUTURE SYNTAX — illustrative API; no ai module exists in the current interpreter.
let data = ai::dataset::from_csv("samples.csv")?;
let model = ai::models::linear(input = 4, output = 1);
model.train(data, epochs = 10);
print model.predict([1.0, 2.0, 3.0, 4.0]);
```

A credible AI module also needs tensor shape/type checks, a numerical backend, reproducible tests, model/data format documentation, and clear CPU/RAM requirements. This language repository currently contains none of those components.

## Diagnostics: every error should teach

The compiler should use stable error codes and show the source line, a caret, a plain-language explanation, and one concrete fix. Example of the **planned format**:

```text
error[E0101]: expected `;` after this statement
  --> app.rustqq:2:17
   |
 2 | let count = 1
   |                 ^ add `;` here
   = help: write `let count = 1;`
```

Initial code families: `E01xx` syntax, `E02xx` names and scopes, `E03xx` types, `E04xx` memory safety, `E05xx` target/platform capability, and `E06xx` build/link. Diagnostics should avoid unexplained compiler jargon, state what the program was trying to do, suggest the smallest safe change, and never claim a dangerous operation is safe just to be friendly. The current interpreter has ordinary English errors, not this structured diagnostic system.

## Build and verification plan

1. **Language foundation:** split the interpreter into modules; add source spans, structured diagnostics, regression tests, and a documented grammar.
2. **Core compiler:** specify fixed-width integers, inference, arrays/structs, modules, errors/results, calling conventions, and deterministic overflow rules.
3. **Backend and target:** select one target and ABI; emit object files; integrate a linker; keep hosted programs separate from freestanding builds.
4. **Safe systems profile:** implement checked address/size types and a tiny target-validated intrinsic set. Test unsupported-target failures and safety diagnostics.
5. **Bootable milestone:** create the smallest serial-printing kernel, build it reproducibly, and boot it in an emulator before documenting hardware support.
6. **AI libraries:** provide a tested CPU tensor/runtime module, then add optional backends without changing basic language syntax.
7. **Hardware expansion:** add documented device drivers and emulator tests per platform; never imply universal device support.

A milestone is complete only when code, automated tests, reproducible commands, and documentation agree. Proposal examples are not implementation evidence.

## Current safe way to learn

Use `examples/kernel-basics.rustqq` only as an interpreter simulation of readiness checks. It does not initialize memory, access a CPU device, configure interrupts, or boot a kernel. Use `examples/hello.rustqq` for currently implemented arrays, loops, input, and functions.
