# Peripheral Drivers

Peripheral components available in the shared components directory (refer to CLAUDE.md "Components Base" for the path).

## UART

Serial communication driver.

- Multiple UART instances (UART0, UART1, UART2, etc.)
- Interrupt-driven TX/RX with ring buffers
- Configurable baud rate, parity, stop bits
- Uses `itask` for deferred RX processing

**Key registers**: UARTx->BDH/BDL (baud), UARTx->C1/C2 (control), UARTx->D (data), UARTx->S1 (status)

## SPI

SPI communication (if present in product).

- Master mode operation
- DMA-capable for large transfers
- Configurable clock phase/polarity

**Key registers**: SPIx->MCR, SPIx->CTAR, SPIx->SR, SPIx->PUSHR/POPR

## I2C

I2C/IIC bus communication.

- Master mode
- Interrupt-driven with state machine
- Clock stretching support

**Key registers**: I2Cx->A1 (address), I2Cx->F (frequency), I2Cx->C1 (control), I2Cx->S (status), I2Cx->D (data)

## ADC

Analog-to-Digital Converter.

- 16-bit resolution capability
- Software and hardware trigger modes
- Calibration routines
- Multiple input channels with mux

**Key registers**: ADCx->SC1[n] (channel select), ADCx->CFG1/CFG2 (config), ADCx->R[n] (result)

## DMA

Direct Memory Access controller.

- Multiple DMA channels
- Memory-to-memory, peripheral-to-memory, memory-to-peripheral
- Linked transfers (scatter-gather)
- Transfer complete interrupts

**Key registers**: DMA_CR (control), DMA->TCD[n] (transfer control descriptors)

**Safety notes**:
- DMA buffers must be aligned to transfer size
- Ensure DMA transfer is complete before accessing destination buffer
- DMA error interrupts should be handled

## GPIO

General Purpose I/O.

- Pin configuration (input/output/alternate function)
- Pull-up/pull-down configuration
- Interrupt on edge/level
- Port Control and Interrupt (PORT) module for pin muxing

**Key registers**: GPIOx->PDDR (direction), GPIOx->PSOR/PCOR/PTOR (set/clear/toggle), PORTx->PCR[n] (pin control)

## Flash

Flash memory programming.

- `flash`: Main program flash operations
- `data_flash`: Data storage in flash (FlexNVM on K22F/K64F)
- Sector erase and program operations
- Flash command sequence handling

**Safety notes**:
- Flash operations disable interrupts during command execution
- Cannot execute code from flash being programmed (execute from RAM or different flash block)
- Verify flash state after programming

## Timer

Hardware timers for periodic events.

- PIT (Periodic Interrupt Timer) — simple periodic interrupts
- FTM (FlexTimer Module) — PWM generation, input capture
- LPTMR (Low-Power Timer) — low-power wake-up

## Watchdog

Watchdog timer to detect firmware hangs.

- Must be periodically refreshed ("kicked")
- Configurable timeout period
- Generates reset on timeout

**Safety notes**:
- Refresh watchdog in main loop, never in ISR
- Ensure all code paths refresh within timeout period
- Disable watchdog during debugging if needed

## MCG

Multipurpose Clock Generator.

- System clock configuration
- PLL/FLL modes
- Bus/Flash clock dividers

**Safety notes**:
- Clock transitions must follow valid mode sequences
- Peripheral clocks must be enabled before accessing peripheral registers (SIM->SCGC registers)

## Utility Components

### indexed_buffer
Ring buffer / indexed buffer implementation for producer-consumer patterns.

### memory_pool
Dynamic memory allocation from pre-allocated pool in Upper SRAM. Replaces malloc/free.

### crc
CRC calculation — hardware CRC module or software implementation.

### time_evaluation
Performance measurement using hardware timers.

### register_access
Safe register read/write abstractions.

### named_data_storage
Key-value storage in flash for configuration parameters.

### update
Firmware update mechanism — receives new firmware image and programs flash.

### file_io
File-like I/O abstraction layer.

### log
Logging system for debug output.

### util
General utility functions (byte manipulation, conversions, etc.).
