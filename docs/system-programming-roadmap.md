# System programming and bilingual diagnostics roadmap

## Current status / الحالة الحالية

RUST++ v0.1 is a small **interpreted teaching language**. It does not currently compile to machine code, provide a `no_std` runtime, or access hardware. The example `examples/kernel-basics.rustqq` simulates boot-stage decisions using the existing language; it is not a bootloader or a kernel.

RUST++ v0.1 is not Rust and cannot currently be used to build a real operating system or kernel. This roadmap describes a safe, incremental direction without implying that unsupported features already work.

## Goals / الأهداف

- Keep beginner syntax small and approachable.
- Provide useful compiler/interpreter diagnostics in English and Arabic, selected explicitly by the user.
- Add a systems profile only after the project has a compiler and a defined target/runtime model.
- Preserve the current interpreter as a simple way to learn control flow and algorithms.

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

## Kernel-oriented language direction / اتجاه دعم النواة

A real kernel requires much more than syntax. Work should proceed in stages:

1. **Compiler foundation:** parse and type-check source, produce stable diagnostics, and add automated tests.
2. **Explicit low-level types:** fixed-width integers, pointers/references, layout rules, and documented overflow behavior.
3. **Target and runtime model:** select an architecture/ABI, support freestanding output, and define what standard-library facilities are unavailable.
4. **Hardware boundary:** isolate architecture-specific operations behind small, documented interfaces; provide emulator-based tests before relying on physical devices.
5. **Kernel examples:** build and boot a minimal example in an emulator, with reproducible commands and clear safety/testing notes.

Do not use the current interpreter for real hardware, boot, or kernel development. Until the compiler, target support, and runtime above exist, system-programming examples should remain simulations or educational pseudocode.

## Try the current teaching example / جرّب المثال التعليمي الحالي

```bash
cargo run -- examples/kernel-basics.rustqq
```

The example practices functions, conditions, booleans, strings, and loops while simulating boot readiness. It uses only syntax already supported by v0.1.
