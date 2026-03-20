# Refactoring Patterns — Before/After Examples

## 1. Extract Function

**Trigger**: Function >60 lines with distinct logical sections.

```c
// Before — 85-line function mixing validation + calculation + formatting
void process_measurement(measure_data_t* data)
{
    // 20 lines of input validation
    if (data == NULL) { return; }
    if (data->voltage < 0.0F) { ... }
    // ...

    // 30 lines of resistance calculation
    float resistance = data->voltage / data->current;
    // ...

    // 35 lines of result formatting
    data->result_mohm = resistance / 1e6F;
    // ...
}

// After — 3 focused functions
static bool validate_inputs(const measure_data_t* data);
static float calculate_resistance(float voltage, float current);
static void format_result(measure_data_t* data, float resistance);

void process_measurement(measure_data_t* data)
{
    if (!validate_inputs(data)) { return; }
    float resistance = calculate_resistance(data->voltage, data->current);
    format_result(data, resistance);
}
```

## 2. Replace Conditional with Table

**Trigger**: if-else chain or switch with >5 branches doing similar operations.

```c
// Before — 8-branch switch
switch (command)
{
    case CMD_READ_VOLTAGE:  handle_read_voltage();  break;
    case CMD_READ_CURRENT:  handle_read_current();  break;
    case CMD_READ_RESIST:   handle_read_resistance(); break;
    // ... 5 more cases
}

// After — function pointer table
typedef void (*cmd_handler_t)(void);
static const cmd_handler_t command_table[] = {
    [CMD_READ_VOLTAGE]  = handle_read_voltage,
    [CMD_READ_CURRENT]  = handle_read_current,
    [CMD_READ_RESIST]   = handle_read_resistance,
    // ...
};

if (command < ARRAY_SIZE(command_table) && command_table[command] != NULL)
{
    command_table[command]();
}
```

## 3. Introduce Parameter Object

**Trigger**: Function with >5 parameters.

```c
// Before — 7 parameters
float calibration_adjust(float raw, float gain, float offset,
                         float min_val, float max_val,
                         float temp, float temp_coeff);

// After — parameter struct
typedef struct {
    float gain;
    float offset;
    float min_val;
    float max_val;
    float temp;
    float temp_coeff;
} calibration_params_t;

float calibration_adjust(float raw, const calibration_params_t* params);
```

## 4. Consolidate Duplicate Code

**Trigger**: ≥3 similar code blocks.

```c
// Before — repeated ADC read pattern in 3 places
// In measure.c:
uint16_t v = adc_read(CH_VOLTAGE);
if (v == ADC_ERROR) { log_error("voltage"); return; }

// In hv.c:
uint16_t v = adc_read(CH_HV_FEEDBACK);
if (v == ADC_ERROR) { log_error("hv_fb"); return; }

// After — common helper
static uint16_t safe_adc_read(uint8_t channel, const char* name)
{
    uint16_t val = adc_read(channel);
    if (val == ADC_ERROR)
    {
        log_error(name);
    }
    return val;
}
```

## 5. Reduce Coupling via Callback

**Trigger**: Circular dependencies between modules.

```c
// Before — measure.c directly calls hv.c
#include "hv.h"  // circular if hv.h also includes measure.h
void on_measurement_complete(void)
{
    hv_adjust_output(new_voltage);
}

// After — callback registration
typedef void (*measurement_cb_t)(float voltage);
static measurement_cb_t on_complete_cb = NULL;

void measure_register_callback(measurement_cb_t cb) { on_complete_cb = cb; }

void on_measurement_complete(void)
{
    if (on_complete_cb != NULL) { on_complete_cb(new_voltage); }
}
// hv.c registers: measure_register_callback(hv_adjust_output);
```
