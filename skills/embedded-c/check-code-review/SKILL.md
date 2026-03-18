---
name: check-code-review
description: "ALWAYS use when reviewing local firmware code changes for embedded safety: ISR safety verification, volatile usage, volatile 확인, soft-float penalty, stack limits, CMSIS patterns, peripheral initialization checklist, critical section 검토, and hardware access pattern review. ALWAYS use for local file/branch review before PR, staged 변경사항 리뷰, 드라이버 초기화 체크리스트, hv.c/adc.c/uart driver review, or any embedded safety concern (volatile, ISR, critical section, 하드웨어 접근). Do NOT use for GitLab MR diff review with comment posting (use check-mr-review), code complexity / structural quality analysis (use plan-quality-audit), application-level logic review without hardware interaction, or broad static analysis (use do-lint)."
argument-hint: "[--staged] [--branch=<base>] [--safety] [--peripheral=<name>] [<file>...]"
user-invokable: true
agent: embedded-reviewer
---

# MCU Code Review

1. Gather Context
   - Acquire list of changed files using `git status` or `git diff --name-only`.
   - Read changed files to understand context.
   - Read `CLAUDE.md` to refresh on project rules.

2. Automated Verification
   - Run `/check-verify` to check for style violations, missing docs, or known bad patterns.
   - Note any failures.

3. Code Review (Agent Role: Embedded Firmware Reviewer)
   - Analyze changes focusing on:
     - **Safety**: ISR reentrancy, volatile usage, critical sections (see Safety Verification below).
     - **Performance**: Stack usage on MK10D7 (4KB limit), float vs double (soft-float penalty).
     - **Correctness**: Hardware access patterns (CMSIS usage), peripheral clock gating (SIM_SCGC).
     - **Style**: Naming conventions (`snake_case`), float suffix (`0.0F`).

4. Report Findings
   - Group by severity: **Critical** (Safety/Crash), **Warning** (Potential/Style), **Info**.
   - Provide actionable suggestions with code examples.
   - Reference specific lines.

5. User Decision
   - Ask user to approve changes or request modifications based on findings.

## Safety Verification

Use `--safety` to perform detailed ISR safety analysis. The following items are always checked as part of Step 3 Safety, but `--safety` provides a dedicated report.

| # | Check | Pattern | Target |
|---|-------|---------|--------|
| 1 | ISR blocking calls | ISR function containing `while`, `delay`, `malloc`, `printf` | Sources/, Drivers/ |
| 2 | Missing volatile | Variables shared between ISR and main without `volatile` | Sources/, Drivers/ |
| 3 | Non-atomic 64-bit access | `uint64_t`, `double` shared between ISR and main | Sources/ |
| 4 | Missing critical section | Read-modify-write on shared variables without protection | Sources/ |
| 5 | Large stack allocation | Local variables >128 bytes in a function | Sources/, Drivers/ |

**Safety Workflow**:
1. Find ISR handlers: Grep for `*_IRQHandler` functions
2. Analyze ISR body: Check for blocking calls, non-reentrant functions
3. Find shared variables: Variables accessed in both ISR and non-ISR context
4. Check volatile: Verify shared variables have `volatile` qualifier
5. Check critical sections: Verify read-modify-write on shared data is protected
6. Stack check: Calculate local variable sizes per function

**Safety Output**:
```
## MCU Safety Report

### Critical (must fix)
| File:Line | Issue | Details |

### Warning (should fix)
| File:Line | Issue | Details |

### Summary
- ISR handlers checked: N
- Shared variables found: N
- Issues: N critical, N warning
```

## Peripheral Initialization Checklist

Use `--peripheral=<name>` to validate peripheral initialization completeness.

### Universal Checklist (all peripherals)

1. [ ] Clock gating enabled (`SIM->SCGCx`)
2. [ ] Pin mux configured (`PORT->PCR[n]`)
3. [ ] Peripheral configured while disabled
4. [ ] Peripheral enabled after full configuration

### Per-Peripheral Checklists

#### ADC
1. [ ] `SIM->SCGC6 |= SIM_SCGC6_ADC0_MASK`
2. [ ] CFG1: resolution (MODE), clock (ADICLK), divider (ADIV)
3. [ ] CFG2: MUX select (MUXSEL), sample time (ADLSTS)
4. [ ] SC2: trigger (ADTRG), DMA (DMAEN)
5. [ ] SC3: hardware average (AVGE/AVGS), calibration (CAL)
6. [ ] Calibration executed and completed
7. [ ] Analog input pins: PORT_PCR_MUX(0)
8. [ ] [DMA] DMA channel configured, SC2[DMAEN]=1
9. [ ] [IRQ] SC1n[AIEN]=1, NVIC EnableIRQ(ADC0_IRQn)

#### UART
1. [ ] `SIM->SCGC4 |= SIM_SCGC4_UARTx_MASK`
2. [ ] TX/RX pins: PORT_PCR_MUX(3)
3. [ ] Baud rate: BDH/BDL registers
4. [ ] C2: TE, RE (transmit/receive enable)
5. [ ] [IRQ] C2[RIE]=1, NVIC EnableIRQ

#### GPIO
1. [ ] `SIM->SCGC5 |= SIM_SCGC5_PORTx_MASK`
2. [ ] PORT_PCR_MUX(1) for GPIO function
3. [ ] PDDR: direction (input/output)
4. [ ] [IRQ] PORT_PCR_IRQC for edge detect

#### FTM (Timer)
1. [ ] `SIM->SCGC6 |= SIM_SCGC6_FTMx_MASK`
2. [ ] SC: clock source (CLKS), prescaler (PS)
3. [ ] MOD: period
4. [ ] CnSC + CnV: channel mode and value
5. [ ] [IRQ] SC[TOIE]=1 or CnSC[CHIE]=1

#### DMA
1. [ ] `SIM->SCGC6 |= SIM_SCGC6_DMAMUX_MASK` + `SIM->SCGC7 |= SIM_SCGC7_DMA_MASK`
2. [ ] DMAMUX: channel source (CHCFG[SOURCE]), enable (CHCFG[ENBL])
3. [ ] TCD: SADDR, DADDR, SOFF, DOFF, ATTR (size), NBYTES, CITER/BITER
4. [ ] CSR: DREQ (auto-disable) or INTMAJOR (completion interrupt)
5. [ ] [IRQ] NVIC EnableIRQ(DMAx_IRQn), ISR clears CINT
6. [ ] Buffer alignment: source/dest addresses aligned to transfer size

### Deinitialization Checklist (deinit)

When a driver provides `<module>_deinit()`:
1. [ ] Peripheral disabled (TE/RE=0, ADTRG=0, etc.)
2. [ ] Interrupts disabled: NVIC DisableIRQ, clear pending
3. [ ] DMA channel disabled (if used)
4. [ ] Clock gating disabled (`SIM->SCGCx &= ~MASK`) — only if no other module shares the clock
5. [ ] Pin mux reset to default (GPIO or disabled)

### Error Handling in Drivers

1. [ ] Timeout on polling loops (never infinite `while(!flag)`)
2. [ ] Error flag checking after peripheral operations (e.g., ADC conversion complete, UART framing error)
3. [ ] Return value convention: `0` = success, negative = error code, or `bool`

### Peripheral Validation Workflow
1. Read the target driver file in Drivers/
2. Cross-reference with the checklist above (universal + per-peripheral + deinit + error handling)
3. Report missing or incorrect initialization steps
