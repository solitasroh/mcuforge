# Measurement Accuracy Debugging — IRM Pipeline

## Verification Points (upstream → downstream)

### Point 1: Raw ADC Values
- Read `adc_ma_sum[]` directly
- Expected: 16-bit values (0-65535), HW 32x averaging
- Check: Values within expected range for known test voltage

### Point 2: Moving Average Output
- Read output of `get_adc_ma_history_ptr()`
- Expected: Stable values with low noise
- Check: Window size = 10 samples, 32-window ring buffer
- Failure mode: Stale data if DMA ISR not firing

### Point 3: Sliding Window Average
- Read after `read_and_average_samples()` + `sum_sliding_window()`
- Expected: 20-window (200 sample) average, very stable
- Check: `AGGREGATION_PERIOD_MS` = 200ms timing

### Point 4: Calibration Output
- Read after `calibration_adjust_dc_voltage()` / `calibration_adjust_current()`
- Expected: Calibrated V and A values
- Check: Calibration table loaded? 2-point linear coefficients valid?
- Failure mode: INFINITY if gain=0, wrong values if cal table corrupted

### Point 5: Resistance (R=V/I)
- Read after `validate_measurement()`
- Expected: R in ohms (float)
- Check: Current > threshold? Division by near-zero current → INFINITY
- Failure mode: INFINITY result, sign errors

### Point 6: Temperature Compensation
- Read after `compensate_resistance_to_reference_temp()`
- Check: Temperature sensor reading valid? Compensation formula: `k_t = exp2f(-(base-test)/slope)`
- Failure mode: Wrong k_t if temperature reading is stale/invalid

## Common Symptoms → Root Cause

| Symptom | Check First | Likely Root Cause |
|---------|-------------|-------------------|
| INFINITY resistance | Point 5: current value | Current ≤ 0 or near-zero |
| Drifting values | Point 2: MA stability | DMA timing issue or ADC calibration drift |
| All zeros | Point 1: raw ADC | DMA not running, ADC clock gating off |
| Offset error | Point 4: cal coefficients | Calibration not loaded or corrupted |
| Temperature-dependent error | Point 6: temp reading | Temperature sensor disconnected/stale |
