---
name: ref-coding-rules
description: "Provides embedded C coding standards reference: snake_case naming, stdint.h types, float F suffix, static scoping, K&R braces. Use when writing new code or reviewing style compliance. Do NOT use for CMSIS vendor code or third-party libraries."
disable-model-invocation: true
---

# MCU Coding Rules

## Overview

Based on the project's `CLAUDE.md` and standard embedded C practices.

## Guidelines

### 1. Types & Literals

- **Float Suffix**: All float literals must have `F` suffix (e.g., `0.0F`, `1.5F`).
  - **Why**: Default literal is `double`, which triggers software emulation on Cortex-M4 (no hardware FPU).
- **Fixed Width Types**: Use `<stdint.h>` types (`uint8_t`, `int32_t`, etc.) instead of bare `int`, `long`, `short`.
- **Booleans**: Use `<stdbool.h>` (`bool`, `true`, `false`).

### 2. Naming Conventions

- **Functions**: `snake_case` (e.g., `adc_start_conversion()`)
- **Variables**: `snake_case` (e.g., `adc_result`)
- **Types**: `snake_case_t` (e.g., `adc_config_t`)
- **Macros/Constants**: `UPPER_SNAKE_CASE` (e.g., `ADC_MAX_VALUE`)
- **Pointers**: `type* name` (e.g., `uint32_t* buffer`) - Left alignment

### 3. Structure & Scope

- **Static Functions**: Prefer `static` for internal module functions to avoid pollution.
- **Header Guards**: Use `#ifndef HEADER_H` ... `#endif` or `#pragma once`.
- **Indentation**: Use Tab (width 4).
- **Braces**: K&R style (opening brace on same line).

### 4. Safety

- **Volatile**: Use `volatile` for variables shared between ISR and main loop.
- **Critical Sections**: Disable interrupts when accessing shared non-atomic resources.
- **Memory**: No dynamic allocation (`malloc`/`free`) in runtime.

## Detection Patterns (for Verify Skills)

| Rule | Tool | Pattern | Description |
|---|---|---|---|
| Float Suffix | grep | `[0-9]\.[0-9]+[^Ff]` | Detects float literals missing 'F' suffix |
| Double usage | grep | `double` | Detects explicit double usage |
| Type safety | grep | `unsigned int\|unsigned long\|unsigned short` | Prefer `uint32_t`, `uint16_t` |
| Pointer align | grep | `[a-zA-Z0-9_] \*[a-zA-Z0-9_]` | Detects `type *name` (preferred `type* name`) |

## File Scope

Applies to all files in:
- `Sources/`
- `Drivers/`
- `Components/`

Excludes:
- `CMSIS/`
- `System/` (vendor code)
