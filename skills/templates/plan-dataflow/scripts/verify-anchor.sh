#!/bin/bash
# verify-anchor.sh — Verify pipeline anchor functions/files exist in actual code
# Usage: bash .claude/skills/plan-dataflow/scripts/verify-anchor.sh
# Returns: 0 if all anchors found, 1 if any missing

PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SOURCES="$PROJECT_ROOT/Sources"
DRIVERS="$PROJECT_ROOT/Drivers"
MISSING=0

echo "=== Pipeline Anchor Verification ==="
echo ""

# Stage 1: ADC + DMA
check_file() {
    local file="$1"
    local desc="$2"
    if [ -f "$PROJECT_ROOT/$file" ]; then
        echo "  [OK] $file — $desc"
    else
        echo "  [MISSING] $file — $desc"
        MISSING=$((MISSING + 1))
    fi
}

check_func() {
    local func="$1"
    local desc="$2"
    if grep -rq "$func" "$SOURCES" "$DRIVERS" 2>/dev/null; then
        echo "  [OK] $func() — $desc"
    else
        echo "  [MISSING] $func() — $desc"
        MISSING=$((MISSING + 1))
    fi
}

echo "[Stage 1] ADC + DMA"
check_file "Drivers/adc.c" "ADC driver"
check_file "Drivers/dma.c" "DMA driver"

echo ""
echo "[Stage 2] Moving Average"
check_func "accumulate_adc_sample" "ADC sample accumulation"
check_func "get_adc_ma_history_ptr" "MA history access"

echo ""
echo "[Stage 3] Sliding Window Average"
check_func "read_and_average_samples" "Sample reader"
check_func "sum_sliding_window" "Window summation"

echo ""
echo "[Stage 4] DC Calibration"
check_func "calibration_adjust_dc_voltage" "Voltage calibration"
check_func "calibration_adjust_current" "Current calibration"

echo ""
echo "[Stage 5] Resistance Calculation"
check_func "validate_measurement" "R=V/I calculation"

echo ""
echo "[Stage 6] Temperature Compensation"
check_func "compensate_resistance_to_reference_temp" "Temp compensation"

echo ""
echo "[Stage 7] Index Calculation"
check_file "Sources/insulation_index.c" "Index calculator"

echo ""
echo "[Stage 8] Host Communication"
check_func "module_write" "Host register write"

echo ""
echo "=== Result ==="
if [ $MISSING -eq 0 ]; then
    echo "All anchors verified. Pipeline stages match actual code."
    exit 0
else
    echo "$MISSING anchor(s) missing or renamed. Update SKILL.md Pipeline Stages."
    exit 1
fi
