# RUST++ v1.0 Language Guide

RUST++ is a dynamically typed, interpreted language with a small syntax and beginner-friendly built-ins. This guide describes the v1 implementation in this repository. Save programs as `.rustqq` files.

## Run a program

```bash
cargo run -- examples/hello.rustqq
cargo build --release
./target/release/rustpp examples/hello.rustqq
```

Run `cargo run` without a file for the interactive shell. Use `--help` or `--version` for command-line information.

## Values and variables

RUST++ has five value kinds: numbers, strings, booleans, `nil`, and arrays. Numbers are floating-point values; strings use double quotes.

```rust
let score = 10;
score = score + 5;
let title = "RUST++ guide";
let ready = true;
let missing = nil;
```

Variables are introduced with `let`. Assignments update an existing variable. Blocks create lexical scopes; a variable declared inside a block is not visible after that block ends. Names start with a letter or `_` and may contain letters, digits, or `_`.

## Output, comments, and expressions

`print` displays one value and requires a semicolon. Comments begin with `//` and run to the end of the line.

```rust
// Calculate and display a result.
print (2 + 3) * 4;
```

Operators, from lower to higher precedence:

| Operators | Meaning |
| --- | --- |
| `or`, `||` | logical OR (short-circuiting) |
| `and`, `&&` | logical AND (short-circuiting) |
| `==`, `!=` | equality |
| `<`, `<=`, `>`, `>=` | numeric comparisons |
| `+`, `-` | addition/subtraction; `+` also joins strings or arrays of matching types |
| `*`, `/`, `%` | multiplication, division, remainder |
| `!`, `not`, unary `-` | logical negation and numeric negation |

`+` accepts two numbers, two strings, or two arrays. The arithmetic and comparison operators other than `+` require numbers. Only `false` and `nil` are false-like; all other values are true-like.

## Decisions and loops

```rust
let temperature = 24;
if temperature > 30 {
    print "hot";
} else {
    print "comfortable";
}

let count = 0;
while count < 3 {
    print count;
    count = count + 1;
}

for (let i = 0; i < 5; i = i + 1) {
    if i == 2 { continue; }
    if i == 4 { break; }
    print i;
}
```

The `for` header contains an initializer, condition, and increment separated by semicolons. Any clause may be omitted. `break` exits the nearest loop and `continue` starts its next iteration. Both statements require a trailing semicolon.

## Arrays

Arrays are mutable, zero-indexed collections. Array literals may contain values of different types, and a trailing comma is allowed.

```rust
let tasks = ["learn", "build", "share"];
print tasks[0];
tasks[1] = "test";
push(tasks, "ship");
print len(tasks);
let last = pop(tasks);
let combined = tasks + ["done"];
```

Indexing an array or string uses a non-negative integer. Strings are indexed by Unicode character. Reading or assigning an index outside an array's bounds produces a runtime error. `pop` removes and returns the last array item; calling it on an empty array is an error. `push` mutates the array and returns that array.

## Functions

Functions are declared with `fn`, accept comma-separated parameters, and return a value with `return`. A function without a return value produces `nil`. Functions can call themselves recursively.

```rust
fn factorial(n) {
    if n <= 1 { return 1; }
    return n * factorial(n - 1);
}

print factorial(5);
```

Functions are named declarations; function values, closures, and first-class function calls are not part of v1. Up to 255 parameters and call arguments are accepted.

## Built-in functions

| Function | Description |
| --- | --- |
| `len(value)` | Number of array items or Unicode characters in a string. |
| `push(array, value)` | Append an item; returns the array. |
| `pop(array)` | Remove and return the final item. |
| `type(value)` | Return `"number"`, `"string"`, `"boolean"`, `"nil"`, or `"array"`. |
| `str(value)` | Convert a value to its display string. |
| `number(value)` | Convert a number, boolean (`1`/`0`), or numeric string to a number. |
| `input(prompt)` | Display a prompt and read one line from standard input. |
| `assert(condition)` | Return `nil` when true-like; stop with an error when false-like. |

Every built-in requires exactly the number of arguments shown. Invalid conversions, argument counts, and value types produce readable runtime errors.

## Complete example

```rust
fn total(numbers) {
    let result = 0;
    for (let i = 0; i < len(numbers); i = i + 1) {
        result = result + numbers[i];
    }
    return result;
}

let values = [4, 8, 15, 16, 23, 42];
print "Total: " + str(total(values));
assert(total([1, 2, 3]) == 6);
```

## Errors and current limits

Syntax errors include a source line number. Runtime errors explain common issues such as undefined names, incorrect function arity, invalid operators, division by zero, and out-of-bounds indexing. The interpreter is intentionally small: v1 does not include modules, classes, static types, closures, exception handling, async execution, or native compilation. The REPL evaluates one input line at a time; use a source file for multi-line function and block definitions.

Run the test suite with `cargo test`.
