# System programming and bilingual diagnostics roadmap

## Current status / الحالة الحالية

RUST++ v0.1 is a small **interpreted teaching language**. It does not currently compile to machine code, provide a `no_std` runtime, or access hardware. The example `examples/kernel-basics.rustqq` simulates boot-stage decisions using the existing language; it is not a bootloader or a kernel.

RUST++ v0.1 is not Rust and cannot currently be used to build a real operating system or kernel. This roadmap describes a safe, incremental direction without implying that unsupported features already work.

## Goals / الأهداف

- Keep beginner syntax small and approachable.
- Provide useful compiler/interpreter diagnostics in English and Arabic, selected explicitly by the user.
- Add a systems profile only after the project has a compiler and a defined target/runtime model.
- Preserve the current interpreter as a simple way to learn control flow and algorithms.
- Make common low-level capabilities discoverable through consistent built-in namespaces, without requiring third-party packages for core compiler intrinsics.

## Bilingual diagnostics / رسائل الأخطاء باللغتين

The current implementation reports most language errors in Arabic only and generally gives a line number without a source excerpt, column, error code, or fix suggestion. The desired diagnostic format is a future feature, not a current CLI option. For example:

```text
error[E####]: expected `;` after this statement
  --> examples/demo.rustqq:3:12
   |
 3 | let count = 1
   |            ^ add `;` at the end of the statement
   |
   = help: write `let count = 1;`
   = ar: توقّع المترجم فاصلة منقوطة `;` في نهاية التعليمة.
   = الحل: اكتب `let count = 1;`.
```

Implementation steps:

1. Track source spans (byte/character offsets, line, and column) on tokens and parser errors.
2. Replace ad-hoc error strings with structured diagnostics: stable code, severity, message, labels, and optional help.
3. Add a language option such as `--diagnostic-language en|ar|both`; preserve a predictable default and document it.
4. Add tests for each error in English, Arabic, and combined modes, including Unicode source text.
5. Keep error output on stderr and return a non-zero status for invalid programs.

## Proposed built-in systems namespaces / مساحات الأنظمة المقترحة

These are **design proposals**, not features available in the v0.1 interpreter. Syntax such as `memory::` is not currently parsed or executed. Do not use the examples below to access real hardware.

A future systems profile could provide compiler-owned, target-checked namespaces so core primitives do not need third-party packages:

- `memory::` — fixed-width types, address/size types, alignment checks, and explicit mapping interfaces.
- `hardware::` — target/device discovery and supported built-in device interfaces; not arbitrary access to every device.
- `cpu::` — architecture-specific operations such as halt or wait-for-interrupt, enabled only for compatible targets and contexts.
- `io::` — explicitly scoped, target-specific device I/O operations.
- `interrupts::` — interrupt setup and handler declarations with required ABI/signature checks.
- `boot::` — entry-point and boot-information interfaces for a specified boot protocol.
- `serial::` / `console::` — basic output interfaces suitable for early boot diagnostics.

Illustrative syntax only (not executable in current RUST++):

```rust
// PROPOSAL ONLY: the current interpreter cannot compile or run this.
@target("x86_64-unknown-none")
@entry
fn kernel_start(info: boot::Info) -> never {
    console::write("kernel started\\n");
    cpu::halt_forever();
}
```

Illustrative memory API only (not executable):

```rust
// PROPOSAL ONLY: raw memory access needs a compiler, target, and explicit safety rules.
let region: memory::Region = memory::region(
    memory::PhysAddr(0x1000_0000),
    memory::Size(4096)
)?;

unsafe {
    memory::map_device(region, memory::ReadWrite, memory::Uncached)?;
}
```

The compiler must reject unsupported targets and incompatible privilege contexts. Numeric addresses must not silently become pointers. Memory mapping should validate alignment, size, address-space rules, and conflicts. Hardware APIs must describe the platform assumptions they require.

“No external libraries” can mean that core intrinsics and a minimal set of platform interfaces ship with the compiler. It cannot remove the need for a compiler backend, linker, boot protocol, architecture-specific implementation, or device-specific drivers. The design should separate the language core, compiler intrinsics, optional built-in platform modules, and user-written kernel code.

## Kernel-oriented language direction / اتجاه دعم النواة

A real kernel requires much more than syntax. Work should proceed in stages:

1. **Compiler foundation:** parse and type-check source, produce stable diagnostics, and add automated tests.
2. **Explicit low-level types:** fixed-width integers, pointers/references, layout rules, and documented overflow behavior.
3. **Target and runtime model:** select an architecture/ABI, support freestanding output, and define what standard-library facilities are unavailable.
4. **Built-in namespaces:** specify and implement a small target-specific set of `memory::`, `hardware::`, `cpu::`, `io::`, `interrupts::`, and `boot::` operations. Add compile-time target/context checks and explicit unsafe boundaries; never silently simulate a hardware operation in a kernel build.
5. **Hardware boundary:** isolate architecture-specific operations behind documented interfaces; provide emulator-based tests before relying on physical devices.
6. **Kernel examples:** build and boot a minimal example in an emulator, with reproducible commands and clear safety/testing notes.

Do not use the current interpreter for real hardware, boot, or kernel development. Until the compiler, target support, and runtime above exist, system-programming examples should remain simulations or clearly marked design sketches.

## Try the current teaching example / جرّب المثال التعليمي الحالي

```bash
cargo run -- examples/kernel-basics.rustqq
```

The example practices functions, conditions, booleans, strings, and loops while simulating boot readiness. It uses only syntax already supported by v0.1 and does not access hardware.
