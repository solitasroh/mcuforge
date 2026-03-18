# MCU Memory Maps

Detailed memory layouts extracted from actual linker scripts. For the current project's specific configuration, refer to `CLAUDE.md` "Memory Configuration" and the linker script specified in CLAUDE.md "Linker Script".

## a2750p (MK22FX512VLL12)

```
Flash (512 KB total, 448 KB available for application):
  m_interrupts  (RX) : ORIGIN = 0x00010000, LENGTH = 0x1BC    (444 B)
  m_text        (RX) : ORIGIN = 0x00010200, LENGTH = 0x6FE00  (448 KB)

SRAM_L (64 KB): 0x1FFF0000 ~ 0x1FFFFFFF
  m_programbuff (RW) : ORIGIN = 0x1FFF0000, LENGTH = 0xFC     (252 B)
  m_data        (RW) : ORIGIN = 0x1FFF0100, LENGTH = 64K-0x100 (~63.75 KB)

SRAM_U (64 KB): 0x20000000 ~ 0x2000FFFF
  m_memorypool  (RW) : ORIGIN = 0x20000000, LENGTH = 64K      (64 KB)

Stack: 0x600 (1.5 KB)
restart_detection: 0x1FFF00FC
```

## a2750lm Application (MK64FX512VLL12)

Note: The MK64F has 256 KB SRAM total (64 KB lower + 192 KB upper), but the application linker script only uses a subset.

```
Flash (512 KB total, ~458 KB available for application):
  m_interrupts  (RX) : ORIGIN = 0x00010000, LENGTH = 0x200    (512 B)
  m_text        (RX) : ORIGIN = 0x00010200, LENGTH = 0x6FE00  (~458 KB)

SRAM_L (64 KB): 0x1FFF0000 ~ 0x1FFFFFFF
  m_programbuff (RW) : ORIGIN = 0x1FFF0000, LENGTH = 0xFC     (252 B)
  m_data        (RW) : ORIGIN = 0x1FFF0100, LENGTH = 64K-0x100 (~63.75 KB)

SRAM_U (192 KB): 0x20000000 ~ 0x2002FFFF
  m_memorypool  (RW) : ORIGIN = 0x20000000, LENGTH = 192K     (192 KB)

Stack: 0x0A00 (2.5 KB)
```

## a2750irm (MK10DX256VLL7)

```
Flash (256 KB total, ~192 KB available for application):
  m_interrupts  (RX) : ORIGIN = 0x00010000, LENGTH = 0x200    (512 B)
  m_text        (RX) : ORIGIN = 0x00010200, LENGTH = 0x2FE00  (~192 KB)

SRAM_L (32 KB): 0x1FFF8000 ~ 0x1FFFFFFF
  m_programbuff (RW) : ORIGIN = 0x1FFF8000, LENGTH = 0xFC     (252 B)
  m_data        (RW) : ORIGIN = 0x1FFF8100, LENGTH = 32K-0x100 (~31.75 KB)

SRAM_U (32 KB): 0x20000000 ~ 0x20007FFF
  m_memorypool  (RW) : ORIGIN = 0x20000000, LENGTH = 32K      (32 KB)

Stack: 0x600 (1.5 KB)
Linker Script: System/linkerscript.ld
```

## a2750io (MK10DX256VLL7)

```
Flash (256 KB total, ~192 KB available for application):
  m_interrupts  (RX) : ORIGIN = 0x00010000, LENGTH = 0x200    (512 B)
  m_text        (RX) : ORIGIN = 0x00010200, LENGTH = 0x2FE00  (~192 KB)

SRAM_L (32 KB): 0x1FFF8000 ~ 0x1FFFFFFF
  m_programbuff (RW) : ORIGIN = 0x1FFF8000, LENGTH = 0xFC     (252 B)
  m_data        (RW) : ORIGIN = 0x1FFF8100, LENGTH = 32K-0x100 (~31.75 KB)

SRAM_U (32 KB): 0x20000000 ~ 0x20007FFF
  m_memorypool  (RW) : ORIGIN = 0x20000000, LENGTH = 32K      (32 KB)

Stack: 0x600 (1.5 KB)
```

## a2700dsp (MK22FX512VLQ12)

```
Flash (512 KB total, 448 KB available for application):
  m_interrupts  (RX) : ORIGIN = 0x00010000, LENGTH = 0x1BC    (444 B)
  m_text        (RX) : ORIGIN = 0x00010200, LENGTH = 0x6FE00  (~448 KB)

SRAM_L (64 KB): 0x1FFF0000 ~ 0x1FFFFFFF
  m_programbuff (RW) : ORIGIN = 0x1FFF0000, LENGTH = 0xFC     (252 B)
  m_data        (RW) : ORIGIN = 0x1FFF0100, LENGTH = 64K-0x100 (~63.75 KB)

SRAM_U (64 KB): 0x20000000 ~ 0x2000FFFF
  m_memorypool  (RW) : ORIGIN = 0x20000000, LENGTH = 64K      (64 KB)

Stack: 0x600 (1.5 KB)
```

## Flash Layout (Common)

All products share a similar flash layout:

```
0x00000000 ┌──────────────────────┐
           │  Bootloader          │  (64 KB)
0x00010000 ├──────────────────────┤
           │  Interrupt Vectors   │  (444~512 B)
0x00010200 ├──────────────────────┤
           │  Application Code    │  (varies by product)
           │  (.text + .rodata)   │
           ├──────────────────────┤
           │  Initialized Data    │  (copied to RAM at startup)
           │  (LMA of .data)      │
           ├──────────────────────┤
           │  ROM copy table      │
           │  (.romp)             │
           └──────────────────────┘
```

## SRAM Layout (Common)

```
SRAM_L     ┌──────────────────────┐  0x1FFF0000 or 0x1FFF8000
           │  Program Buffer      │  (252 B)
           ├──────────────────────┤
           │  .data (initialized) │
           ├──────────────────────┤
           │  .bss (zeroed)       │
           ├──────────────────────┤
           │  Heap (if any)       │
           ├──────────────────────┤
           │  Stack (grows down)  │  (varies by product)
           └──────────────────────┘  0x20000000
SRAM_U     ┌──────────────────────┐  0x20000000
           │  Memory Pool         │  (32~192 KB)
           └──────────────────────┘
```
