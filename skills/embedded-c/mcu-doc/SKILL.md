---
name: mcu-doc
description: "Generates API documentation from component header files by parsing types, constants, and function declarations. Use when documenting component APIs or generating reference docs. Do NOT use for application-level Sources/ documentation."
argument-hint: "<component|all> [--output=<path>] [--format=md|stdout]"
user-invokable: true
---

# MCU Documentation

Generate API documentation by parsing component header files, extracting public interfaces, and presenting them in a structured format.

## Arguments

- `<component|all>`: Component name (e.g., `uart`, `flash`) or `all` to document all components
- `--output=<path>`: Write output to file (default: stdout)
- `--format=md|stdout`: Output format (default: stdout)

## Behavior

1. **Locate component**:
   - If component name: find `Components/<component>/<component>.h`
   - If `all`: read `CMakeLists.txt` for included components list

2. **Parse header file**: Extract from each `.h` file:
   - Include guard and file description (top comment block)
   - Type definitions: `typedef struct`, `typedef enum`, aliases
   - Defines/Constants: `#define` with values and inline comments
   - Function declarations: Return type, name, parameters, preceding comment

3. **Generate documentation**:

   ```markdown
   # uart

   > UART serial communication driver

   **Header**: `Components/uart/uart.h`
   **Source**: `Components/uart/uart.c`

   ## Types
   ### `uart_config_t`
   ...

   ## Constants
   | Name | Value | Description |
   |------|-------|-------------|
   | `UART_RX_BUFFER_SIZE` | `256` | Receive ring buffer size |

   ## Functions
   ### `uart_init`
   ...
   ```

4. **Cross-reference**: Note which functions are called from ISR context and mark them as ISR-safe.
