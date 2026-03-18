# Boundary Value Analysis — 7-Category Checklist

Apply these categories systematically to every function under test.
Not all categories apply to every function — skip those that are irrelevant.

## Category 1: Guard Conditions

Test inputs that should be rejected or handled specially at the entry point.

| Input | Expected |
|-------|----------|
| 0 | Guard return or safe default |
| NULL pointer | Guard return (if applicable) |
| Negative value | Guard return or clamp |
| Empty array / zero length | Guard return |

```c
void test_function_zero_input(void) {
    float result = target_function(0);
    TEST_ASSERT_EQUAL_FLOAT(EXPECTED_GUARD_VALUE, result);
}
```

## Category 2: Threshold Boundaries

Test values at exact thresholds and +/- epsilon around them.

| Input | Expected |
|-------|----------|
| threshold - epsilon | Behavior A |
| threshold (exact) | Defined behavior |
| threshold + epsilon | Behavior B |

```c
#define THRESHOLD 1e-9F
#define EPSILON 1e-12F

void test_function_below_threshold(void) {
    float result = target_function(THRESHOLD - EPSILON);
    // Assert behavior A
}

void test_function_at_threshold(void) {
    float result = target_function(THRESHOLD);
    // Assert defined boundary behavior
}

void test_function_above_threshold(void) {
    float result = target_function(THRESHOLD + EPSILON);
    // Assert behavior B
}
```

## Category 3: Return Value Guards

Test conditions where the function returns special values (INFINITY, 0, error codes).

| Condition | Expected Return |
|-----------|----------------|
| Divisor = 0 | INFINITY or error |
| Divisor < 0 | INFINITY or error |
| Divisor very small (< threshold) | INFINITY |
| Result overflow | Clamped or INFINITY |

```c
void test_resistance_current_zero(void) {
    float result = resistance_from_current(0.0F);
    TEST_ASSERT_TRUE(isinf(result));
}

void test_resistance_current_below_threshold(void) {
    float result = resistance_from_current(1e-10F);  // below MIN_CURRENT
    TEST_ASSERT_TRUE(isinf(result));
}
```

## Category 4: Normal Operation Range

Test typical values that exercise the main calculation path.

| Input | Expected |
|-------|----------|
| Mid-range value | Calculated result |
| Known calibration point | Expected calibrated value |
| Round-trip conversion | Original value recovered |

```c
void test_adc_to_voltage_midrange(void) {
    // Given: known ADC value
    uint16_t adc = 32768;  // midpoint
    // When
    float voltage = convert_adc_to_voltage(adc);
    // Then: verify against manual calculation
    float expected = (adc / 65535.0F) * V_REF / K_VOLTAGE;
    TEST_ASSERT_FLOAT_WITHIN(0.001F, expected, voltage);
}
```

## Category 5: Special Float Values

Test IEEE 754 special values that may propagate through calculations.

| Input | Expected |
|-------|----------|
| NaN | Defined behavior (guard or propagate) |
| +INFINITY | Defined behavior |
| -INFINITY | Defined behavior |
| -0.0F | Same as +0.0F or distinct |

```c
#include <math.h>

void test_function_nan_input(void) {
    float result = target_function(NAN);
    // NaN should be caught by guard or propagate
    TEST_ASSERT_TRUE(isnan(result) || isinf(result));
}

void test_function_positive_infinity(void) {
    float result = target_function(INFINITY);
    // Assert defined behavior
}

void test_function_negative_infinity(void) {
    float result = target_function(-INFINITY);
    // Assert defined behavior
}
```

## Category 6: Extreme Values

Test at the limits of the data type range.

| Input | Expected |
|-------|----------|
| FLT_MIN (smallest normalized) | Valid or guard |
| FLT_MAX (largest finite) | Valid or overflow guard |
| Subnormal (e.g., 1e-45F) | Valid or underflow guard |
| UINT16_MAX (65535) | ADC max value |

```c
#include <float.h>

void test_function_flt_min(void) {
    float result = target_function(FLT_MIN);
    TEST_ASSERT_FALSE(isnan(result));
}

void test_function_flt_max(void) {
    float result = target_function(FLT_MAX);
    // Assert no overflow to NaN
    TEST_ASSERT_FALSE(isnan(result));
}
```

## Category 7: Domain-Specific (IRM)

Test values specific to the insulation resistance measurement domain.

| Domain | Range | Key Boundaries |
|--------|-------|----------------|
| ADC raw value | 0 ~ 65535 | 0, 32768, 65535 |
| DC voltage | 0 ~ 1000V | 0, 250V, 500V, 1000V |
| DC current | 0 ~ 10mA | 0, MIN_CURRENT_THRESHOLD, 1mA |
| Resistance | 0 ~ 50 GOhm | 0, 1MOhm, 1GOhm, INFINITY |
| HV target | 0 ~ 1000V | Segment boundaries |
| V_REF | 3.3V | ADC reference voltage |
| K_VOLTAGE / K_CURRENT | Divider ratios | From measure_define.h |

```c
void test_adc_zero(void) {
    float result = convert_adc_to_voltage(0);
    TEST_ASSERT_EQUAL_FLOAT(0.0F, result);
}

void test_adc_max(void) {
    float result = convert_adc_to_voltage(65535);
    float expected = V_REF / K_VOLTAGE;  // full-scale voltage
    TEST_ASSERT_FLOAT_WITHIN(0.01F, expected, result);
}
```

## Application Order

When generating tests for a function, work through categories in order:

1. **Guard conditions** — what makes the function return early?
2. **Threshold boundaries** — where does behavior change?
3. **Return value guards** — what special values can it return?
4. **Normal operation** — does it calculate correctly?
5. **Special floats** — does NaN/INF cause crashes?
6. **Extreme values** — does FLT_MIN/FLT_MAX cause issues?
7. **Domain-specific** — are IRM-specific ranges covered?

Aim for at least categories 1-4 for every function. Categories 5-7 are recommended for float-heavy calculation functions.
