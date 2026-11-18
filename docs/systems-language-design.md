# RUST++ Systems Profile: design proposal

> **Status: proposal only.** The current RUST++ v0.1 interpreter does not implement any `hardware::`, `memory::`, `cpu::`, `io::`, or `interrupts::` operation below. These examples are language-design sketches, not executable programs. Never run them as if they were real hardware access.

## الهدف | Goal

Make RUST++ approachable for systems programming and eventual operating-system/kernel development, while keeping the beginner language simple. Hardware operations should be compiler-provided intrinsics (built into the compiler/runtime), not external packages. A library-free syntax does **not** eliminate the need for a compiler backend, target definitions, a boot process, or architecture documentation.

The existing interpreter is not a kernel compiler. Real hardware support requires a compiler that emits freestanding machine code for an explicit architecture/ABI, a linker/boot workflow, and tests in an emulator. Until those exist, all snippets in this proposal are illustrative only.

## Proposed namespaces | المساحات المقترحة

These names are reserved design directions, not implemented namespaces:

- `memory::` — fixed-width integer types, address types, checked mapping and allocation interfaces. Raw addresses must require an explicit unsafe boundary.
- `hardware::` — discover/query the selected target and expose supported device abstractions; no generic magic access to arbitrary devices.
- `cpu::` — target-specific processor operations such as halt and wait-for-interrupt, available only when supported by the selected target.
- `io::` — target-specific device I/O; operations must be limited to documented address spaces and privilege contexts.
- `interrupts::` — interrupt table/configuration interfaces with explicit initialization and handler constraints.
- `boot::` — target boot metadata and entry-point declarations.
- `serial::` / `console::` — small built-in output interfaces for diagnostics in a freestanding environment.

All names and signatures below are proposals. The compiler should reject unsupported target operations at compile time rather than silently pretending to access hardware.

## Type and safety rules

1. Define fixed-width integer types (`u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`) and target-sized `usize`/`isize`; document overflow and alignment behavior.
2. Distinguish ordinary integers from addresses and pointers. Do not allow arbitrary integer-to-pointer conversion implicitly.
3. Make potentially unsafe memory or device operations syntactically explicit, e.g. `unsafe { ... }`, and provide safe checked alternatives where possible.
4. Restrict privileged instructions and hardware access to supported targets and documented execution contexts. Emit a clear diagnostic if the target or privilege level is incompatible.
5. Keep register layouts and device structures explicit, with `repr(C)`-like layout guarantees and compile-time size/alignment checks.
6. Provide no hidden allocator, operating-system calls, or standard-library dependency in the freestanding profile. Require explicit configuration of entry point, target, linker, and panic behavior.
7. Treat `hardware::` and similar names as compiler intrinsics, not a promise that all hardware is accessible without drivers or device-specific knowledge.

## Illustrative syntax (not executable today)

### Boot entry point and console

```rust
// PROPOSED SYNTAX ONLY — current RUST++ cannot compile this.
@target("x86_64-unknown-none")
@entry
fn kernel_start(info: boot::Info) -> never {
    console::write("RUST++ kernel started\n");
    cpu::halt_forever();
}
```

The compiler would need to define the entry ABI, validate that the selected target supports `cpu::halt_forever`, and link a freestanding image. A normal interpreter run cannot perform this task.

### Typed memory mapping

```rust
// PROPOSED SYNTAX ONLY — address mapping requires target-specific validation.
let region: memory::Region = memory::region(
    memory::PhysAddr(0x1000_0000),
    memory::Size(4096)
)?;

unsafe {
    memory::map_device(region, memory::ReadWrite, memory::Uncached)?;
}
```

A future implementation must validate alignment, size, address-space rules, and mapping conflicts. It must not silently dereference arbitrary numeric addresses.

### Device access

```rust
// PROPOSED SYNTAX ONLY — illustrative API, not a real driver.
let uart = hardware::device::<hardware::Uart>(hardware::DeviceId::from_firmware("serial0"))?;
uart.configure(serial::Baud::B115200)?;
uart.write_bytes("hello\n".as_bytes())?;
```

Device discovery and UART access are not universal CPU instructions. They require a defined platform, a device description, and a supported built-in driver or user-written driver.

### Interrupt handler

```rust
// PROPOSED SYNTAX ONLY — handler ABI and safety rules are target-specific.
@interrupt(vector = 32)
fn timer_tick(frame: interrupts::Frame) {
    timer::acknowledge();
    scheduler::on_tick(frame);
}
```

The compiler must check handler signature, calling convention, and restrictions on allocation/blocking. These checks do not exist in v0.1.

## No external libraries: what that means

The goal can be “no third-party library required for core primitives,” not “no implementation required.” The compiler/toolchain still needs built-in target support, linker integration, startup/boot support, and a small set of compiler intrinsics. Drivers for different hardware cannot be universal; they should be built into the compiler distribution or written as RUST++ source using documented interfaces.

Suggested separation:

- **Language core:** syntax, fixed-width types, control flow, diagnostics.
- **Compiler intrinsics:** target-specific `cpu::`, `memory::`, `io::` operations with compile-time validation.
- **Built-in platform modules:** optional compiler-distributed definitions/drivers, versioned by target.
- **User kernel code:** boot flow, allocators, drivers, scheduler, and filesystem, built against explicit interfaces.

## Implementation plan

1. Split the single-file interpreter into lexer/parser/diagnostic/runtime modules and add tests before expanding the language.
2. Implement structured, bilingual diagnostics with source spans; report unsupported system features as errors, not as successful no-ops.
3. Specify fixed-width types, layout, overflow, address, and unsafe rules before adding hardware syntax.
4. Add a compiler backend and choose one explicit architecture/ABI and freestanding target; keep interpreter behavior separate.
5. Implement only a tiny, documented intrinsic set for that target, with capability/target checks and emulator tests.
6. Add bootable examples only after reproducible build and emulator validation; clearly label hardware-specific requirements.
7. Expand to additional architectures and device interfaces through separately tested target modules.

## Current, runnable learning example

The existing `examples/kernel-basics.rustqq` is an interpreter-compatible simulation of boot readiness; it does not touch hardware. Run it with:

```bash
cargo run -- examples/kernel-basics.rustqq
```

For current limitations and the bilingual-diagnostic plan, see `docs/system-programming-roadmap.md`.
