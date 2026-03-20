---
name: do-scaffold
description: "ALWAYS use when creating a new module from scratch in Sources/ or new drivers in Drivers/. Generates standardized firmware module scaffolding (header + source) following project conventions: init/deinit lifecycle, static scoping, snake_case naming. ALWAYS use when user mentions: 새 모듈, new module, 모듈 생성, 모듈 추가, 드라이버 추가, new driver, 파일 생성, 템플릿, scaffold, 스캐폴드, boilerplate, 모듈 만들어줘, 드라이버 만들어줘, .h/.c 파일 생성. Do NOT use for modifying existing modules or CMSIS vendor code."
argument-hint: "[driver|source] <module-name>"
user-invokable: true
agent: module-generator
---

# MCU Module Scaffold

Generate new firmware module files following project conventions.

## Module Types

### Application Module (Sources/)
- Part of specific product firmware
- May use product-specific hardware
- Registered with task framework (itask/wtask)

### Driver Module (Drivers/)
- Reusable hardware driver
- Init/deinit lifecycle
- Clock gating, pin mux, interrupt setup

## Generated Files

### Header Template
```c
#ifndef <MODULE_UPPER>_H
#define <MODULE_UPPER>_H

#include <stdint.h>
#include <stdbool.h>

void <module>_init(void);
void <module>_deinit(void);

#endif /* <MODULE_UPPER>_H */
```

### Source Template
```c
#include "<module>.h"

/* Private variables */

/* Private function prototypes */
static void process(void);

/* Public functions */
void <module>_init(void) { }
void <module>_deinit(void) { }

/* Private functions */
static void process(void) { }
```

## Integration Checklist
- [ ] Add `.c` file to CMakeLists.txt
- [ ] Add `#include` in calling module
- [ ] Call `<module>_init()` in init sequence
- [ ] [Driver] Enable clock gating (SIM->SCGCx) — see `ref-hardware` for register details
- [ ] [Driver] Configure pin mux (PORT->PCR) — see `ref-hardware` for pin mux patterns
- [ ] [ISR] Register handler in vector table — see `ref-architecture` for memory map constraints
