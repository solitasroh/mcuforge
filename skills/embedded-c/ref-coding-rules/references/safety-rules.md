# Embedded Safety Rules

## Volatile Usage

### When volatile is required
- Variables shared between ISR and main context
- Variables shared between tasks (if not using OS-level synchronization)
- Hardware register pointers (already handled by CMSIS headers)
- DMA buffer descriptors

### Correct pattern
```c
static volatile uint32_t adc_value;     /* Written in ISR, read in main */
static volatile bool     data_ready;    /* Flag set in ISR, polled in main */
```

### Common mistakes
- Forgetting `volatile` on ISR-shared variables (compiler may optimize away reads)
- Using `volatile` unnecessarily on purely local variables (hurts performance)
- Accessing multi-byte `volatile` variables non-atomically (see critical sections below)

## ISR Safety Rules

### Allowed in ISR
- Read/write `volatile` variables
- Set flags, update counters
- Short buffer operations (ring buffer push)
- Clear interrupt flags (hardware register writes)
- Call `itask_signal()` for deferred processing

### Forbidden in ISR
- `malloc()`, `free()`, or any heap allocation
- `printf()`, `sprintf()`, or any stdio functions
- Blocking operations (busy waits, semaphore waits)
- Calling non-reentrant functions
- Long-running computations
- Nested function calls deeper than 2-3 levels

### ISR Pattern (deferred processing)
```c
void UART0_IRQHandler(void)
{
    uint8_t data = UART0->D;                /* Read data register */
    ring_buffer_push(&rx_buffer, data);      /* Quick buffer operation */
    itask_signal(UART_RX_EVENT);             /* Defer processing to task */
}
```

## Critical Sections

For non-atomic access to shared multi-byte variables:

```c
uint32_t value;
__disable_irq();
value = shared_counter;    /* Atomic read of 32-bit on Cortex-M4 */
__enable_irq();
```

Note: On Cortex-M4, single 32-bit aligned reads/writes are atomic. Critical sections are needed for:
- 64-bit variables
- Read-modify-write sequences
- Multi-variable consistency

## Stack Limits

- **Stack size**: Refer to the project's `CLAUDE.md` "Memory Configuration" > "Stack Size" (e.g., 0x0600 = 1,536 bytes or 0x0A00 = 2,560 bytes depending on product)
- **Guidelines**:
  - Maximum local variable size per function: ~128 bytes
  - Avoid large local arrays — use static buffers or memory pool instead
  - Be aware of call depth: main → component → driver → ISR
  - ISR uses the same stack (MSP on Cortex-M)
  - Reserve ~256 bytes for ISR stack usage

### Stack-safe alternatives
```c
/* BAD: Large local array risks stack overflow */
void process_data(void)
{
    uint8_t buffer[512];    /* Too large for constrained stack! */
}

/* GOOD: Use static or memory pool */
static uint8_t buffer[512];    /* File-scope static */
void process_data(void)
{
    /* Use static buffer */
}
```

## Memory Pool Usage

The project uses a custom `memory_pool` component instead of `malloc`/`free`:

- Memory pool is allocated in Upper SRAM (`m_memorypool` region)
- Pool size varies by product — refer to CLAUDE.md "Memory Configuration"

### Usage pattern
```c
#include "memory_pool.h"

void *ptr = memory_pool_alloc(size);
if (ptr == NULL)
{
    /* Handle allocation failure */
}
/* ... use buffer ... */
memory_pool_free(ptr);
```

## Register Access Safety

- Always use CMSIS-defined register structures (e.g., `UART0->C2`)
- For read-modify-write on registers, use bit manipulation:
  ```c
  UART0->C2 |= UART_C2_RE_MASK;    /* Set bit */
  UART0->C2 &= ~UART_C2_TE_MASK;   /* Clear bit */
  ```
- Never cache register values unless you know the register is not volatile
- Clear interrupt flags by writing to the correct status register (check datasheet for w1c bits)
