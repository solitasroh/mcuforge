# Common Patterns and Anti-Patterns

## Configuration Pattern

Use compile-time configuration with defaults:

```c
/* In module header */
#ifndef MODULE_BUFFER_SIZE
#define MODULE_BUFFER_SIZE  256
#endif

#ifndef MODULE_TIMEOUT_MS
#define MODULE_TIMEOUT_MS   1000
#endif
```

Product-specific overrides can be set in CMakeLists.txt via `add_definitions()`.

## Initialization Order

Firmware startup follows a specific initialization sequence:

1. **System clock** (MCG) — configure PLL, bus clocks
2. **Watchdog** — configure timeout or disable for debug
3. **GPIO** — configure pin directions and initial states
4. **Peripheral clocks** — enable SIM->SCGC for each peripheral
5. **Peripheral drivers** — UART, SPI, I2C, ADC, DMA
6. **Component init** — memory pool, logging, data storage
7. **Task framework** — itask, wtask, thread initialization
8. **Application** — register tasks and start main loop

## Main Loop Pattern

```c
int main(void)
{
    system_init();
    application_init();

    for (;;)
    {
        itask_process();      /* Handle deferred interrupts */
        wtask_process();      /* Handle timed tasks */
        thread_process();     /* Handle cooperative threads */
        watchdog_refresh();   /* Keep watchdog alive */
    }

    /* Never reached */
    return 0;
}
```

## Firmware Update Pattern

The update mechanism:

1. Receive new firmware image via communication channel (UART)
2. Store image in program buffer or external storage
3. Verify CRC/checksum of received image
4. Erase target flash sectors
5. Program new image to flash
6. Verify programmed data
7. Reset to bootloader for activation

## Communication Protocol Pattern

For UART-based protocols:

```c
typedef enum
{
    PROTO_STATE_WAIT_HEADER,
    PROTO_STATE_WAIT_LENGTH,
    PROTO_STATE_WAIT_DATA,
    PROTO_STATE_WAIT_CHECKSUM
} proto_state_t;

static void protocol_byte_received(uint8_t byte)
{
    switch (proto_state)
    {
        case PROTO_STATE_WAIT_HEADER:
            if (byte == PROTOCOL_HEADER)
            {
                proto_state = PROTO_STATE_WAIT_LENGTH;
            }
            break;
        /* ... */
    }
}
```

## Anti-Patterns to Avoid

### 1. Busy-wait in ISR
```c
/* BAD */
void UART0_IRQHandler(void)
{
    while (!(UART0->S1 & UART_S1_TDRE_MASK)) { }  /* Blocking! */
    UART0->D = data;
}
```

### 2. malloc/free usage
```c
/* BAD — use memory_pool instead */
char *buf = malloc(256);
```

### 3. Non-atomic shared variable access
```c
/* BAD — 64-bit access is not atomic on Cortex-M4 */
static volatile uint64_t timestamp;

void Timer_IRQHandler(void)
{
    timestamp++;  /* Non-atomic read-modify-write of 64-bit value */
}

uint64_t get_time(void)
{
    return timestamp;  /* May read torn value */
}

/* GOOD — use critical section */
uint64_t get_time(void)
{
    uint64_t value;
    __disable_irq();
    value = timestamp;
    __enable_irq();
    return value;
}
```

### 4. Large stack allocations
```c
/* BAD — risks stack overflow (check CLAUDE.md for stack size) */
void process(void)
{
    uint8_t temp[1024];
}

/* GOOD — use static or memory pool */
static uint8_t temp[1024];
void process(void)
{
    /* use static temp */
}
```

### 5. Ignoring error returns
```c
/* BAD */
flash_write(addr, data, len);  /* Return value ignored */

/* GOOD */
if (flash_write(addr, data, len) != FLASH_OK)
{
    log_error("Flash write failed at 0x%08X", addr);
    return ERROR_FLASH;
}
```

### 6. Forgetting to enable peripheral clock
```c
/* BAD — accessing peripheral before enabling clock causes hard fault */
UART0->C2 = UART_C2_RE_MASK;

/* GOOD — enable clock first */
SIM->SCGC4 |= SIM_SCGC4_UART0_MASK;
UART0->C2 = UART_C2_RE_MASK;
```

## Debug Patterns

### Log levels
```c
log_debug("Entering state %d", new_state);
log_info("Connection established");
log_warning("Buffer 80%% full");
log_error("Flash write failed");
```

### Compile-time debug switches
```c
#ifdef DEBUG
    #define DBG_ASSERT(expr) do { if (!(expr)) { __BKPT(0); } } while(0)
#else
    #define DBG_ASSERT(expr) ((void)0)
#endif
```
