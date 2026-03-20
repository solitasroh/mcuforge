# Common fff Fake Patterns

Frequently used fake patterns for IRM project modules. Copy and adapt these when generating new test files.

## ADC Fakes

```c
#include "fff.h"
#include "adc.h"

FAKE_VALUE_FUNC(uint16_t, adc_read, uint8_t);
FAKE_VOID_FUNC(adc_init);
FAKE_VOID_FUNC(adc_deinit);
FAKE_VALUE_FUNC(bool, adc_is_calibrated);
```

## DMA Fakes

```c
#include "fff.h"
#include "dma.h"

FAKE_VOID_FUNC(dma_init);
FAKE_VOID_FUNC(dma_enable_channel, uint8_t);
FAKE_VOID_FUNC(dma_disable_channel, uint8_t);
```

## GPIO / DIO Fakes

```c
#include "fff.h"
#include "gpio_hal.h"

FAKE_VOID_FUNC(gpio_set_pin, uint32_t, uint8_t);
FAKE_VOID_FUNC(gpio_clear_pin, uint32_t, uint8_t);
FAKE_VALUE_FUNC(bool, gpio_read_pin, uint32_t, uint8_t);
```

## HV Control Fakes

```c
#include "fff.h"
#include "hv.h"

FAKE_VOID_FUNC(hv_init);
FAKE_VOID_FUNC(hv_deinit);
FAKE_VOID_FUNC(hv_set_output, bool);
FAKE_VALUE_FUNC(bool, hv_is_enabled);
FAKE_VALUE_FUNC(float, hv_get_voltage);
```

## UART / Communication Fakes

```c
#include "fff.h"
#include "uart.h"

FAKE_VOID_FUNC(uart_init, uint32_t);
FAKE_VOID_FUNC(uart_send, const uint8_t*, uint32_t);
FAKE_VALUE_FUNC(uint32_t, uart_receive, uint8_t*, uint32_t);
FAKE_VALUE_FUNC(bool, uart_is_tx_complete);
```

## Task Framework Fakes (wtask/itask)

```c
#include "fff.h"
#include "wtask.h"

FAKE_VOID_FUNC(wtask_schedule_successively_sec, wtask_t*, uint32_t);
FAKE_VOID_FUNC(wtask_cancel, wtask_t*);
FAKE_VALUE_FUNC(bool, wtask_is_scheduled, const wtask_t*);
```

## Log Fakes

```c
#include "fff.h"
#include "log.h"

FAKE_VOID_FUNC(log_error, const char*);
FAKE_VOID_FUNC(log_info, const char*);
```

## Custom Return Value Sequences

```c
// Return different values on successive calls
uint16_t adc_returns[] = {1024, 2048, 4096, 0};
SET_RETURN_SEQ(adc_read, adc_returns, 4);

// Custom fake with side effects
uint16_t custom_adc_read(uint8_t channel) {
    if (channel == 0) return 1024;
    if (channel == 1) return 2048;
    return 0;
}
adc_read_fake.custom_fake = custom_adc_read;
```

## setUp() Template

```c
void setUp(void)
{
    RESET_FAKE(adc_read);
    RESET_FAKE(gpio_set_pin);
    RESET_FAKE(hv_set_output);
    // ... reset all fakes used in this test file
    FFF_RESET_HISTORY();

    // Reset static state if using Include-the-Source
    // e.g., memset(&static_struct, 0, sizeof(static_struct));
}
```
