# c85c: A Compiler for c85 to 8085 Assembly

## Overview

c85c is a simple compiler that translates code written in the c85 language—a minimalistic, C-like syntax tailored for the Intel 8085 microprocessor—into 8085 assembly code. The c85 language supports basic features like variable assignments, register manipulations, conditional statements (if), and comparisons. It's designed for educational purposes or low-level programming on 8085-based systems.

The compiler is implemented in Rust and consists of a lexer, parser, and code generator. It allocates static variables in memory starting from address 0x8000 and assigns them to registers where possible.

### Key Features
- **Static Variable Assignment**: Declare and initialize variables with hex values (e.g., `counter = 0x00;`).
- **Register Assignment**: Directly assign values to registers (e.g., `reg D = 0xAA;`).
- **Conditional Statements**: Supports if conditions with comparisons like `<`, `>`, `==` (e.g., `if(counter < limit){ ... }`).
- **Binary Operations**: Limited to operations with register B as the second operand (e.g., `A + B;`).
- **Pointer Increment/Decrement**: For 16-bit register pairs (e.g., `HL++;`).
- **Memory Allocation**: Basic support for `malloc` to load addresses into register pairs.

Note: The language is highly restricted and requires some understanding of 8085 arch.

## Project Structure

```
c85c
├── Cargo.lock          # Dependency lock file
├── Cargo.toml          # Project manifest
├── input.asm           # Sample output assembly
├── input.c85           # Sample input c85 code
└── src
    ├── codegen.rs      # Assembly code generation
    ├── lexer.rs        # Tokenization
    ├── main.rs         # Entry point
    └── parser.rs       # AST parsing
```

## Installation

1. Ensure you have Rust installed (version 1.75 or later). If not, install it via [rustup](https://rustup.rs/).

2. Clone the repository:
   ```
   git clone https://github.com/yourusername/c85c.git
   cd c85c
   ```

3. Build the project:
   ```
   cargo build --release
   ```

## Usage

Run the compiler on a c85 source file to generate an assembly output file:

```
cargo run -- <input_file.c85>
```

- This will produce `<input_file.asm>` in the same directory.
- Example: `cargo run -- input.c85` generates `input.asm`.

### Sample Input (input.c85)

```
main{
    // Three static variables assigned to A, B, C
    counter = 0x00;
    limit = 0xFF;
    status = 0x05;
    // Compare counter (A) with limit (B)
    if(counter < limit){
        reg D = 0xAA;
    }
    // Compare status (C) with limit (B)
    if(status > limit){
        reg E = 0xBB;
    }
    // Can still use register names directly
    if(A == B){
        reg H = 0xCC;
    }
}
```

### Sample Output (input.asm)

```
MVI A,00H;
STA 8000H;
MVI A,FFH;
STA 8001H;
MOV B,A;
MVI A,05H;
STA 8002H;
MOV C,A;
CMP B;
JZ SKIP_0;
JNC SKIP_0;
MVI D,AAH;
SKIP_0:
MOV A,C;
CMP B;
JZ SKIP_1;
JC SKIP_1;
MVI E,BBH;
SKIP_1:
CMP B;
JNZ SKIP_2;
MVI H,CCH;
SKIP_2:
```

## How It Works

1. **Lexer**: Tokenizes the c85 source code into keywords, identifiers, literals, and symbols.
2. **Parser**: Builds an Abstract Syntax Tree (AST) from tokens, validating syntax and inferring types (8-bit vs. 16-bit).
3. **Code Generator**: Traverses the AST to produce 8085 assembly, handling memory allocation for static variables and register assignments.

### TODO
- Only supports a subset of 8085 instructions.
- No loops, functions, or advanced control flow.
- Error handling is basic; invalid code may panic or produce errors.

