# MCU/MPU/WPF 개발 스킬 전체 맵 (Big Picture)

## 타겟 플랫폼 정리

```
┌─────────────────────────────────────────────────────────────┐
│                    개발 플랫폼 스펙트럼                         │
├────────────┬────────────┬──────────────┬───────────────────┤
│  Bare Metal │ Bare Metal │ Embedded     │ Desktop App      │
│  (Cortex-M) │ (Cortex-M) │ Linux (ARM9) │ (Windows/.NET)   │
├────────────┼────────────┼──────────────┼───────────────────┤
│ NXP Kinetis │ STM32 H5   │ i.MX28       │ WPF / C#         │
│ K10D (M4)   │ H563 (M33) │ (ARM926EJ-S) │ Prism / MVVM     │
│ K22F (M4)   │            │ i.MX6ULL     │ .NET Framework    │
│ K64F (M4)   │            │ (Cortex-A7)  │                   │
├────────────┼────────────┼──────────────┼───────────────────┤
│ C99         │ C99/C11    │ C + Linux    │ C# / XAML         │
│ NXP SDK     │ STM32 HAL  │ BusyBox      │ NuGet             │
│ KDS/MCUXpr  │ STM32Cube  │ Buildroot    │ Visual Studio     │
│ ARM GCC     │ ARM GCC    │ Cross-compile│ MSBuild           │
│ 128K~1M     │ 2M Flash   │ NAND/eMMC    │ 제약 없음          │
│ Flash       │ 640K RAM   │ DDR 128M+    │                   │
└────────────┴────────────┴──────────────┴───────────────────┘
```

---

## 스킬 분류 체계

### 분류 1: 적용 범위에 따른 3계층

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: 공통 스킬 (모든 플랫폼)                               │
│   개발 원칙, Git 워크플로우, 문서 생성, 프로젝트 관리              │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: 도메인 스킬 (임베디드 or 데스크톱)                      │
│   MCU 코드리뷰, 메모리분석, 디버깅 / WPF 아키텍처, MVVM 패턴     │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: 플랫폼 특화 스킬 (특정 MCU/OS)                        │
│   K22F 에라타, STM32 HAL 패턴, i.MX 부트로더, WPF 컨트롤 패턴   │
└─────────────────────────────────────────────────────────────┘
```

### 분류 2: 목적에 따른 4가지 유형

```
A. 원칙/가드레일 (Principles)  — "어떻게 일할 것인가"
B. 분석/검증 (Analysis)        — "코드/시스템이 올바른가"
C. 생성/자동화 (Generation)    — "반복 작업을 자동으로"
D. 지식/레퍼런스 (Knowledge)   — "빠르게 정보를 찾기"
```

---

## 전체 스킬맵

### A. 원칙/가드레일 스킬 (Principles)

| # | 스킬명 | 적용 범위 | 목적 | 활용 방식 |
|---|--------|----------|------|----------|
| A1 | **think-first** | 공통 | 코딩 전 설계 사고 강제 | Karpathy #1. 가정 명시, 트레이드오프 제시, 모호함에 질문. 플랫폼별 체크리스트 분기 (MCU→하드웨어 제약, WPF→UI/UX 제약) |
| A2 | **simplicity-guard** | 공통 | 과도한 복잡성 방지 | Karpathy #2. "이 추상화가 정말 필요한가?" MCU에서는 HAL 위의 HAL 금지, WPF에서는 과도한 DI 컨테이너 금지 |
| A3 | **surgical-changes** | 공통 | 최소 변경 원칙 강제 | Karpathy #3. 요청에 직접 연결되는 변경만. 레지스터 설정 순서 보존, XAML 스타일 건드리지 않기 |
| A4 | **goal-driven-dev** | 공통 | 성공 기준 정의 강제 | Karpathy #4. "ADC 3.3V→4095" "Modbus FC03 응답 확인" "버튼 클릭→ViewModel 업데이트" |
| A5 | **coding-standard** | 공통 | 코딩 스타일 일관성 | 플랫폼별 references 분기: C 네이밍(Module_Function), C# 네이밍(PascalCase), 파일 구조 |

> **구현 형태**: A1~A4는 CLAUDE.md에 통합. A5는 독립 스킬 + references/

---

### B. 분석/검증 스킬 (Analysis)

| # | 스킬명 | 적용 범위 | 목적 | 활용 방식 |
|---|--------|----------|------|----------|
| B1 | **mcu-code-reviewer** | Bare Metal | ISR/메모리/volatile 안전성 검증 | 6종 전문 리뷰어 (ISR Safety, Memory Safety, Volatile/Atomic, Code Quality, MISRA Subset, Resource Efficiency). C 코드 업로드 → 심각도별 리포트 |
| B2 | **wpf-code-reviewer** | WPF | MVVM 패턴/메모리 누수/바인딩 검증 | WPF 특화 리뷰어 (MVVM 준수, IDisposable/메모리누수, 바인딩 에러 패턴, 스레딩 안전성, XAML 성능) |
| B3 | **linux-code-reviewer** | Embedded Linux | 시스템 프로그래밍 안전성 검증 | 파일 디스크립터 누수, 시그널 안전성, 크로스컴파일 이슈, BusyBox 호환성 |
| B4 | **linker-map-analyzer** | Bare Metal | 메모리 사용량 분석 | .map 파일 → Flash/RAM 리포트. K10D(128K), K22F(512K), K64(1M), H5(2M) 한계 대비 경고 |
| B5 | **ai-code-verifier** | 공통 | AI 생성 코드 안전성 검증 | MCU: WCET 추정, 스택분석, 동적할당 탐지. WPF: 스레드 안전성, 리소스 누수. 공통: 에러 핸들링 누락 |
| B6 | **precision-analyzer** | 측정 장비 | ADC/회로 정밀도 분석 | 파라미터 입력 → 분해능, 오차 전파, 온도 드리프트, 총 시스템 오차 계산. Accura 제품 특화 |
| B7 | **build-error-debugger** | 공통 | 빌드 에러 진단 | ARM GCC/MSBuild 에러 해석, 링커 에러, SDK 이슈. 플랫폼별 references |

---

### C. 생성/자동화 스킬 (Generation)

| # | 스킬명 | 적용 범위 | 목적 | 활용 방식 |
|---|--------|----------|------|----------|
| C1 | **test-generator** | 공통 | 테스트 코드 자동 생성 | MCU: Unity 테스트 + Mock. WPF: MSTest/NUnit + Moq. 헤더/인터페이스 분석 → 경계값/에러/정상 케이스 |
| C2 | **driver-generator** | Bare Metal | HAL 드라이버 스켈레톤 생성 | 주변장치 명세 입력 → Init/Read/Write/IRQHandler 코드. NXP SDK 스타일과 STM32 HAL 스타일 선택 |
| C3 | **doc-generator** | 공통 | 코드 기반 기술 문서 생성 | 모듈 API 레퍼런스, 아키텍처 문서, 동작 분석서. docx/md/pdf 출력. Rootech 스타일 자동 적용 |
| C4 | **commit-assistant** | 공통 | Git 커밋/MR/릴리즈노트 자동화 | Conventional Commits 메시지, GitLab MR 설명, CHANGELOG 생성. OpenProject WP 번호 연동 |
| C5 | **config-generator** | Bare Metal | 주변장치 설정 코드 생성 | "ADC 12bit, DMA, 4채널" → 클럭+핀+ADC+DMA 초기화 코드. STM32CubeMX 대안 (텍스트 기반) |
| C6 | **protocol-generator** | 공통 | 통신 프로토콜 코드 생성 | Modbus RTU/TCP 마스터/슬레이브, UART 프레임 파서, CRC 계산. 임베디드 Linux ↔ Bare Metal 공통 |
| C7 | **manual-writer** | 제품 | 사용자 매뉴얼 생성 | (기존 스킬 확장) 설정 사양 → Word 매뉴얼. Accura 스타일 |

---

### D. 지식/레퍼런스 스킬 (Knowledge)

| # | 스킬명 | 적용 범위 | 목적 | 활용 방식 |
|---|--------|----------|------|----------|
| D1 | **debug-strategist** | 공통 | 증상 기반 디버깅 전략 | Superpowers의 4단계 방법론 (조사→패턴분석→가설검증→수정). 증상별 빠른 진입 (HardFault, ADC이상, 통신끊김, UI프리징) |
| D2 | **datasheet-navigator** | Bare Metal | 데이터시트 정보 빠른 접근 | MCU별 주요 레지스터맵, 에라타, 전기적 특성을 references/에 축적. "K22F SPI 최대 클럭?" → 즉시 답변 |
| D3 | **pattern-library** | 공통 | 검증된 설계 패턴 DB | MCU: 상태머신, 링버퍼, 이벤트 플래그, 협력형 스케줄러. WPF: MVVM, Mediator, 커맨드 패턴, 다이얼로그 서비스 |
| D4 | **migration-guide** | 플랫폼 전환 | MCU 간 마이그레이션 가이드 | K22F→STM32H5 전환 시 주변장치 매핑, SDK API 대응표, 인터럽트 모델 차이 |
| D5 | **errata-checker** | Bare Metal | 에라타/실리콘 버그 확인 | "K10D ADC 16bit 사용 시 주의사항", "STM32H5 DMA 버스트 이슈" 등을 references/에 축적 |

---

## 플랫폼별 스킬 적용 매트릭스

```
                          Kinetis    STM32 H5   i.MX28    i.MX6ULL    WPF
                         (K10D/K22F)            (ARM9)   (Cortex-A7)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
A1. think-first            ●          ●          ●          ●          ●
A2. simplicity-guard       ●          ●          ●          ●          ●
A3. surgical-changes       ●          ●          ●          ●          ●
A4. goal-driven-dev        ●          ●          ●          ●          ●
A5. coding-standard        ● (C)      ● (C)      ● (C)      ● (C)      ● (C#)
───────────────────────────────────────────────────────────────────────────
B1. mcu-code-reviewer      ●          ●          △          △          
B2. wpf-code-reviewer                                                  ●
B3. linux-code-reviewer                           ●          ●         
B4. linker-map-analyzer    ●          ●                                
B5. ai-code-verifier       ●          ●          ●          ●          ●
B6. precision-analyzer     ●          ●                                
B7. build-error-debugger   ●          ●          ●          ●          ●
───────────────────────────────────────────────────────────────────────────
C1. test-generator         ●          ●          ●          ●          ●
C2. driver-generator       ●          ●                                
C3. doc-generator          ●          ●          ●          ●          ●
C4. commit-assistant       ●          ●          ●          ●          ●
C5. config-generator       ●          ●                                
C6. protocol-generator     ●          ●          ●          ●          
C7. manual-writer          ●          ●          ●          ●          
───────────────────────────────────────────────────────────────────────────
D1. debug-strategist       ●          ●          ●          ●          ●
D2. datasheet-navigator    ●          ●          ●          ●          
D3. pattern-library        ●          ●          ●          ●          ●
D4. migration-guide        ●  ←→  ●                                
D5. errata-checker         ●          ●          ●          ●          

● = 직접 적용    △ = 부분 적용 (레지스터 접근 있는 디바이스 드라이버)
```

---

## 워크플로우별 스킬 체인

### 1. 새 기능 개발 (Bare Metal)
```
A1.think-first ──→ D2.datasheet ──→ C5.config-gen ──→ C2.driver-gen
       │                                                     │
       │                                                     ▼
       │                                              C1.test-gen (TDD)
       │                                                     │
       │                                                     ▼
       │                                              B1.mcu-code-review
       │                                                     │
       ▼                                                     ▼
  A4.goal-driven ──────────────────────────────────→ C4.commit-assistant
```

### 2. 새 기능 개발 (WPF)
```
A1.think-first ──→ D3.pattern-lib ──→ C1.test-gen (TDD)
       │                                     │
       │                                     ▼
       │                              B2.wpf-code-review
       │                                     │
       ▼                                     ▼
  A4.goal-driven ──────────────────→ C4.commit-assistant
```

### 3. 버그 수정 (공통)
```
D1.debug-strategist ──→ D5.errata-checker ──→ 원인 파악
       │                                          │
       ▼                                          ▼
  증상 DB 검색                               C1.test-gen (재현 테스트)
                                                  │
                                                  ▼
                                          B1/B2.code-review (수정 검증)
                                                  │
                                                  ▼
                                          C4.commit-assistant
```

### 4. 릴리즈 준비
```
B4.linker-map-analyzer ──→ B5.ai-code-verifier ──→ C3.doc-generator
                                                          │
                                                          ▼
                                                   C4.commit (릴리즈노트)
                                                          │
                                                          ▼
                                                   C7.manual-writer
```

### 5. 플랫폼 마이그레이션 (K22F → STM32 H5)
```
D4.migration-guide ──→ C5.config-gen (STM32용) ──→ C2.driver-gen (STM32 HAL)
       │                                                    │
       ▼                                                    ▼
  레지스터 매핑표                                      B1.mcu-code-review
  API 대응표                                                │
  인터럽트 모델 차이                                          ▼
                                                     C1.test-gen
```

---

## 스킬 간 공유 구조

### 공유 references/ (여러 스킬이 참조)

```
shared-references/
├── platforms/
│   ├── kinetis/
│   │   ├── k22f-constraints.md        # Flash 512K, RAM 128K, 주변장치 목록
│   │   ├── k22f-register-map/         # 주요 주변장치 레지스터 맵
│   │   ├── k22f-errata.md             # 실리콘 버그 목록
│   │   ├── k10d-constraints.md        # K10D 특화 (16bit ADC)
│   │   └── k10d-errata.md
│   ├── stm32/
│   │   ├── h5-constraints.md          # Flash 2M, RAM 640K, TrustZone
│   │   ├── h5-register-map/
│   │   ├── h5-errata.md
│   │   └── hal-vs-ll-guide.md         # HAL vs Low-Layer 선택 가이드
│   ├── imx/
│   │   ├── imx28-constraints.md       # ARM926, NAND, GPMI, FEC
│   │   ├── imx6ull-constraints.md     # Cortex-A7, DDR, GPU 없음
│   │   └── linux-driver-patterns.md
│   └── wpf/
│       ├── mvvm-patterns.md
│       ├── prism-guide.md
│       └── performance-checklist.md
│
├── standards/
│   ├── c-naming-convention.md         # Module_Function, g_global, s_static
│   ├── csharp-naming-convention.md    # PascalCase, 인터페이스 I접두사
│   ├── misra-c-subset.md              # 팀에서 적용할 핵심 15규칙
│   ├── commit-convention.md           # Conventional Commits + OP 연동
│   └── mr-template.md                 # GitLab MR 템플릿
│
├── patterns/
│   ├── state-machine.md               # 유한 상태 머신 구현 패턴
│   ├── ring-buffer.md                 # ISR-safe 링 버퍼
│   ├── event-flag.md                  # 이벤트 플래그 패턴
│   ├── cooperative-scheduler.md       # iTask 스타일 스케줄러
│   ├── modbus-implementation.md       # Modbus RTU/TCP 패턴
│   └── hal-abstraction.md             # 플랫폼 독립 HAL 패턴
│
└── debugging/
    ├── hardfault-diagnosis.md
    ├── adc-measurement-debug.md       # Accura 2750IRM 경험 반영
    ├── communication-debug.md         # UART/SPI/Modbus 디버깅
    ├── timing-debug.md
    ├── memory-leak-debug.md           # 임베디드 리눅스 + WPF
    └── wpf-binding-debug.md
```

---

## 스킬 유형별 요약

| 유형 | 개수 | 핵심 가치 | 구현 우선순위 |
|------|------|----------|-------------|
| **A. 원칙** (5) | CLAUDE.md + 1스킬 | 모든 작업의 기반, Karpathy 원칙 | ★★★★★ 최우선 |
| **B. 분석** (7) | 코드리뷰 3종 + 도구 4종 | 품질 보장, 실수 방지 | ★★★★☆ |
| **C. 생성** (7) | 코드/테스트/문서/커밋 | 반복 작업 자동화, 시간 절약 | ★★★★☆ |
| **D. 지식** (5) | 디버깅/데이터시트/패턴 | 경험 축적, 팀 지식 공유 | ★★★☆☆ (축적형) |

**총 24개 스킬 후보** (원칙 5 + 분석 7 + 생성 7 + 지식 5)

---

## 큰 그림 요약

```
                    ┌─────────────────────┐
                    │   A. 원칙/가드레일    │
                    │ (Karpathy 4원칙)     │
                    │ CLAUDE.md에 통합      │
                    └─────────┬───────────┘
                              │ 모든 작업에 적용
                              ▼
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  B. 분석/검증    │ │  C. 생성/자동화   │ │  D. 지식/참조    │
│                 │ │                 │ │                 │
│ mcu-code-review │ │ test-generator  │ │ debug-strategy  │
│ wpf-code-review │ │ driver-gen      │ │ datasheet-nav   │
│ linux-code-rev  │ │ doc-generator   │ │ pattern-library │
│ linker-map      │ │ commit-assist   │ │ migration-guide │
│ ai-verifier     │ │ config-gen      │ │ errata-checker  │
│ precision       │ │ protocol-gen    │ │                 │
│ build-error     │ │ manual-writer   │ │                 │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
                              ▼
                    ┌─────────────────────┐
                    │  shared-references/  │
                    │ platforms/ standards/ │
                    │ patterns/ debugging/  │
                    │ (여러 스킬이 공유하는  │
                    │  지식 자산)           │
                    └─────────────────────┘
```

이 전체 구조에서 가장 중요한 것:
1. **A 원칙은 CLAUDE.md에 한 번 쓰면 모든 스킬에 자동 적용**
2. **shared-references/는 한 번 축적하면 여러 스킬이 공유하는 자산**
3. **각 스킬은 한 가지 일에 집중, 워크플로우 체인으로 연결**
4. **플랫폼 차이는 스킬 내부가 아닌 references/에서 분기**
