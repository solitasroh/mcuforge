---
name: ref-peripheral-driver
description: "Guides peripheral driver development: clock gating, pin mux, interrupt setup, DMA configuration, and init/deinit lifecycle. Use when creating new drivers in Drivers/ or modifying existing peripheral interfaces. Do NOT use for application-level Sources/ code."
disable-model-invocation: true
---

# Peripheral Driver Development

## Directory Structure

Drivers are located in `Drivers/` and organized by peripheral type:

```
Drivers/
├── adc/
│   ├── adc.c
│   └── adc.h
├── uart/
│   ├── uart.c
│   └── uart.h
├── gpio/
│   ├── gpio.c
│   └── gpio.h
...
```

## Implementation Guidelines

### 1. Initialization

Drivers must provide an initialization function that takes a configuration structure (if applicable) or sets default values.

- **Naming**: `<module>_init(void)` or `<module>_init(const <module>_config_t* config)`
- **Behavior**: Enable clock (SIM), configure pins (PORT), set default register values.

### 2. API Design

Expose high-level operations, hiding register details.

- **Prefix**: All public functions must start with the module name (e.g., `uart_write_byte`).
- **Blocking vs Non-blocking**:
  - Functions are generally blocking unless specified otherwise.
  - Non-blocking functions should use interrupts or DMA.

### 3. Interrupts

- ISRs should be short.
- Use `volatile` for shared data.
- Clear interrupt flags explicitly.

### 4. Dependencies

- Drivers can depend on `CMSIS/` headers.
- Drivers should NOT depend on other Drivers (unless hierarchical) or `Sources/` (application logic).
- Common definitions can be in `Components/`.

## Common Drivers

| Driver | File | Description |
|---|---|---|
| ADC | `Drivers/adc/adc.c` | Analog-to-Digital Conversion |
| DMA | `Drivers/dma/dma.c` | Direct Memory Access |
| GPIO | `Drivers/gpio/gpio.c` | General Purpose I/O |
| UART | `Drivers/uart/uart.c` | Serial Communication |
| FTM | `Drivers/ftm/ftm.c` | Timer/PWM |

## Usage Example

```c
#include "Drivers/adc/adc.h"

void measure_init(void) {
    adc_init(); // internal default config
}

uint16_t measure_read(void) {
    return adc_read_channel(ADC_CHANNEL_TEMP);
}
```
