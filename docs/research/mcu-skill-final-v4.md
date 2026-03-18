# 개발 스킬 최종 맵 (Final v4)

> 재검토 완료: 24개 → 15개 스킬로 정리. 과도한 분리 제거, 현실성 검증 반영.

---

## 타겟 플랫폼

```
┌──────────────────────────────────────────────────────────────────┐
│                        개발 플랫폼 스펙트럼                         │
├─── Bare Metal ──────┬─── Embedded Linux ──┬─── Desktop ─────────┤
│                     │                     │                     │
│ NXP Kinetis         │ i.MX28 (ARM926)     │ WPF / C#            │
│  K10D (M4, 128K)    │ i.MX6ULL (A7)       │  Prism / MVVM       │
│  K22F (M4, 512K)    │  Buildroot          │  .NET Framework      │
│  K64F (M4, 1M)      │  BusyBox            │  Visual Studio       │
│                     │  크로스 컴파일        │                     │
│ STM32 H5            │                     │                     │
│  H563 (M33, 2M)     │                     │                     │
│  TrustZone           │                     │                     │
│                     │                     │                     │
│ 공통: C99, ARM GCC   │ C + Linux API       │ C# / XAML            │
│ iTask 협력형 스케줄러  │ 디바이스 트리         │ NuGet / MSBuild      │
│ NXP SDK / STM32 HAL  │ 커널 모듈            │                     │
└─────────────────────┴─────────────────────┴─────────────────────┘
```

---

## 전체 구조

```
┌──────────────────────────────────────┐
│         CLAUDE.md (항상 적용)          │
│                                      │
│  Karpathy 4원칙:                      │
│   • Think First (가정 명시)            │
│   • Simplicity (최소 복잡성)           │
│   • Surgical Changes (최소 변경)       │
│   • Goal-Driven (성공 기준 정의)       │
│                                      │
│  MCU 원칙: ISR 최소화, 정적 할당 우선   │
│  WPF 원칙: MVVM 준수, 이벤트 해제 필수  │
└─────────────────┬────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌────────┐  ┌─────────┐  ┌──────────┐
│ B.분석  │  │ C.생성   │  │ D.지식    │
│ (6개)   │  │ (6개)    │  │ (3개)     │
└────────┘  └─────────┘  └──────────┘
    │             │             │
    └─────────────┼─────────────┘
                  ▼
         ┌────────────────┐
         │ A5.코딩 표준    │ ← 유일한 원칙 스킬
         └────────────────┘
                  │
                  ▼
       ┌──────────────────┐
       │ shared-references │ ← 모든 스킬이 공유하는 지식 자산
       └──────────────────┘
```

---

## 15개 스킬 상세

### A. 원칙 (1개)

| # | 스킬명 | 목적 | 활용 | 플랫폼 |
|---|--------|------|------|--------|
| A5 | **coding-standard** | 코딩 스타일 일관성 강제 | 네이밍, 파일 구조, 주석 규칙을 플랫폼별로 분기. 코드 작성/리뷰 시 자동 참조 | 전체 |

references 분기:
- `c-naming.md`: Module_Function, g_global, s_static, MODULE_CONST
- `csharp-naming.md`: PascalCase, I접두사, async 접미사
- `file-structure-c.md`: 헤더→정의→private→public→ISR 순서
- `file-structure-csharp.md`: MVVM 폴더 구조

---

### B. 분석/검증 (6개)

| # | 스킬명 | 목적 | 활용 | 플랫폼 |
|---|--------|------|------|--------|
| B1 | **mcu-code-reviewer** | C 펌웨어 코드 안전성 검증 | C 코드 업로드 → 심각도별 리포트 (CRITICAL/HIGH/MEDIUM/LOW). AI 생성 코드 특화 체크도 포함 | Bare Metal + 임베디드 리눅스 커널 드라이버 |
| B2 | **wpf-code-reviewer** | WPF/Prism 코드 품질 검증 | C#/XAML 코드 → MVVM 패턴 준수, 바인딩 누수, Prism 라이프사이클, 스레딩 안전성 리포트 | WPF |
| B3 | **linux-code-reviewer** | 리눅스 유저스페이스 코드 검증 | 파일 디스크립터 누수, 시그널 안전성, 크로스컴파일 이슈, BusyBox 호환성, 에러 핸들링 | Embedded Linux |
| B4 | **linker-map-analyzer** | 메모리 사용량 분석 | .map 파일 → Flash/RAM 사용률, 모듈별 점유, Top N, MCU 한계 대비 경고. Python 스크립트 | Bare Metal |
| B6 | **precision-analyzer** | ADC/회로 정밀도 분석 | 회로 파라미터 → 분해능, 오차 전파, 온도 드리프트, 총 시스템 오차. 회로→코드 매핑 포함 | 측정 장비 |
| B7 | **build-error-debugger** | 빌드 에러 진단 | 에러 로그 → 원인 분석 + 해결 방안. ARM GCC, MSBuild, Buildroot, DTS 커버 | 전체 |

**B1 상세 — 6종 리뷰어 + AI 코드 체크**:

| 리뷰어 | 체크 내용 | 심각도 |
|--------|----------|--------|
| ISR Safety | ISR 내 블로킹, 실행 시간, 중첩 안전성 | CRITICAL |
| Memory Safety | 배열 경계, NULL 역참조, 스택 오버플로우 | CRITICAL |
| Volatile/Atomic | 공유 변수 volatile, 크리티컬 섹션 | CRITICAL |
| MISRA Subset | 팀 적용 핵심 15규칙 | HIGH |
| Code Quality | 매직넘버, 함수 길이, 네이밍, 복잡도 | MEDIUM |
| AI Code Check | 과도한 추상화, 동적 할당, 에러 핸들링 누락 | MEDIUM |

**B2 상세 — WPF 특화 체크**:

| 체크 영역 | 구체적 항목 |
|----------|-----------|
| 바인딩 누수 | INotifyPropertyChanged 미구현, 컬렉션 바인딩(ObservableCollection), x:Name 동적 제거, PropertyDescriptor 강참조 |
| 이벤트 누수 | 이벤트 구독 미해제, 싱글톤 서비스 참조, WeakEventManager 미사용 |
| MVVM 위반 | View 코드비하인드에서 ViewModel 직접 캐스팅, View에 비즈니스 로직 |
| Prism 패턴 | IEventAggregator 구독 해제, Region Navigation Dispose, DI 수명 관리 |
| 스레딩 | UI 스레드 외 컬렉션 변경, Dispatcher 남용, async/await 데드락 |

---

### C. 생성/자동화 (6개)

| # | 스킬명 | 목적 | 활용 | 플랫폼 |
|---|--------|------|------|--------|
| C1 | **test-generator** | 테스트 코드 자동 생성 | 헤더(.h)/인터페이스 분석 → 경계값/에러/정상 케이스 + Mock. Unity(C), MSTest(C#) | 전체 |
| C2 | **driver-generator** | 주변장치 설정 + HAL 드라이버 생성 | 구조화된 명세 또는 SDK 헤더 → Init/Read/Write/IRQHandler + 설정 코드 | Bare Metal |
| C3 | **doc-generator** | 코드 기반 기술 문서 생성 | 소스 코드 → API 레퍼런스, 아키텍처 문서, 동작 분석서 (docx/md). Rootech 스타일 적용 | 전체 |
| C4 | **commit-assistant** | Git 워크플로우 자동화 | diff → Conventional Commits 메시지, MR 설명, CHANGELOG. OpenProject WP 연동 | 전체 |
| C6 | **protocol-generator** | 통신 프로토콜 코드 생성 | Modbus RTU/TCP 마스터/슬레이브, UART 프레임 파서, CRC. 임베디드↔리눅스 공통 | MCU + Linux |
| C7 | **manual-writer** | 사용자 매뉴얼 생성 | 설정 사양 → Word 매뉴얼. Accura 스타일. (기존 스킬 확장) | 제품 |

---

### D. 지식/레퍼런스 (3개)

| # | 스킬명 | 목적 | 활용 | 플랫폼 |
|---|--------|------|------|--------|
| D1 | **debug-strategist** | 증상 기반 디버깅 전략 | 4단계 방법론 (조사→패턴분석→가설검증→수정). 증상별 진입점 + 프로토콜 디버깅 포함 | 전체 |
| D2 | **datasheet-navigator** | 데이터시트/에라타 빠른 접근 | MCU별 레지스터맵, 에라타, 전기적 특성. TrustZone 가이드 포함. 점진적 축적 | Bare Metal + Linux |
| D3 | **pattern-library** | 검증된 설계 패턴 DB | MCU: iTask 스케줄러(상세), 링버퍼, 상태머신. WPF: Prism/MVVM, Mediator. 공통: Modbus 구현 | 전체 |

**D1 증상별 진입점**:
```
"ADC 값 이상"        → adc-measurement-debug.md
"HardFault"          → hardfault-diagnosis.md
"통신 끊김"           → communication-debug.md (Modbus 프로토콜 분석 포함)
"타이밍 오류"         → timing-debug.md
"간헐적 오동작"       → race-condition-debug.md
"WPF 메모리 증가"     → wpf-memory-debug.md
"빌드 실패"           → build-error-debugger (B7)로 연결
```

**D3 iTask 특화 내용** (Gap 1 해결):
- 태스크 주기 설계 (1ms/10ms/100ms/1s 티어)
- ISR → 태스크 플래그 전달 패턴
- 태스크 간 데이터 공유 (더블 버퍼, 크리티컬 섹션)
- 워치독 연동
- CPU 부하율 측정

---

## 플랫폼별 적용 매트릭스 (최종)

```
                     Kinetis    STM32 H5   i.MX28/6ULL    WPF
                    (K10D/22F/64)          (유저/커널)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CLAUDE.md 원칙         ●          ●         ●    ●        ●
A5. coding-standard    ● (C)      ● (C)     ● (C) ●       ● (C#)
────────────────────────────────────────────────────────────────
B1. mcu-code-reviewer  ●          ●              ●(커널)
B2. wpf-code-reviewer                                     ●
B3. linux-code-reviewer                     ●(유저)
B4. linker-map-analyzer●          ●
B6. precision-analyzer ●          ●
B7. build-error-debug  ●          ●         ●    ●        ●
────────────────────────────────────────────────────────────────
C1. test-generator     ●          ●         ●    ●        ●
C2. driver-generator   ●          ●
C3. doc-generator      ●          ●         ●    ●        ●
C4. commit-assistant   ●          ●         ●    ●        ●
C6. protocol-generator ●          ●         ●    ●
C7. manual-writer      ●          ●         ●    ●
────────────────────────────────────────────────────────────────
D1. debug-strategist   ●          ●         ●    ●        ●
D2. datasheet-navigator●          ●         ●    ●
D3. pattern-library    ●          ●         ●    ●        ●
```

---

## 워크플로우 체인 (최종)

### 새 기능 개발 (Bare Metal)
```
CLAUDE.md → D2(데이터시트/에라타 확인) → C2(드라이버+설정 생성)
                                              │
                                              ▼
                                        C1(테스트 먼저!)
                                              │
                                              ▼
                                        구현 → B1(6종 리뷰)
                                              │
                                              ▼
                                        C4(커밋) → C3(문서)
```

### 새 기능 개발 (WPF/Prism)
```
CLAUDE.md → D3(Prism 패턴 확인) → C1(테스트 먼저!)
                                       │
                                       ▼
                                 구현 → B2(WPF 리뷰)
                                       │
                                       ▼
                                 C4(커밋)
```

### 버그 수정 (공통)
```
D1(디버깅 전략: 증상→패턴→가설→수정)
  │
  ├→ D2(에라타 확인)     ← Bare Metal인 경우
  │
  ▼
C1(재현 테스트 작성) → 수정 → B1/B2/B3(리뷰) → C4(커밋)
```

### 릴리즈 (Bare Metal)
```
B4(링커맵 분석) → B1(최종 리뷰) → C3(기술 문서)
                                      │
                                      ▼
                                C4(릴리즈노트) → C7(매뉴얼)
```

---

## shared-references/ 구조 (최종)

```
shared-references/
├── platforms/
│   ├── kinetis/
│   │   ├── k22f-constraints.md      # Flash 512K, RAM 128K, 핀, 클럭
│   │   ├── k22f-adc.md              # ADC 레지스터맵 + 사용 예시
│   │   ├── k22f-errata.md           # 실리콘 버그
│   │   ├── k10d-constraints.md
│   │   └── k10d-adc16.md            # 16-bit ADC 특화
│   ├── stm32/
│   │   ├── h5-constraints.md        # Flash 2M, RAM 640K, TrustZone
│   │   ├── h5-trustzone-guide.md    # Secure/Non-Secure 설계
│   │   └── hal-vs-ll-guide.md
│   ├── imx/
│   │   ├── imx28-constraints.md
│   │   ├── imx6ull-constraints.md
│   │   ├── buildroot-guide.md
│   │   └── device-tree-guide.md
│   └── wpf/
│       ├── prism-patterns.md        # EventAggregator, RegionNav, DI
│       ├── binding-leak-patterns.md # WPF 특유 5가지 누수 패턴
│       └── threading-guide.md
│
├── standards/
│   ├── c-naming.md
│   ├── csharp-naming.md
│   ├── misra-c-subset.md            # 팀 적용 핵심 15규칙
│   ├── commit-convention.md
│   └── mr-template.md
│
├── patterns/
│   ├── itask-scheduler.md           # 협력형 스케줄러 상세 (주기 설계, 부하 측정)
│   ├── ring-buffer.md               # ISR-safe 구현
│   ├── state-machine.md
│   ├── modbus-implementation.md
│   └── hal-abstraction.md           # 플랫폼 독립 HAL 설계
│
└── debugging/
    ├── hardfault-diagnosis.md
    ├── adc-measurement-debug.md     # Accura 2750IRM 경험 반영
    ├── communication-debug.md       # UART/SPI/Modbus + Wireshark 가이드
    ├── timing-debug.md
    ├── race-condition-debug.md
    └── wpf-memory-debug.md          # WPF 바인딩/이벤트 누수 진단
```

### references 축적 로드맵
```
즉시    → standards/ 전체 (이미 보유한 지식)
1개월   → kinetis/k22f-*, debugging/hardfault, adc-measurement
2개월   → patterns/itask, ring-buffer, modbus
3개월   → stm32/h5-*, wpf/prism-*, binding-leak
6개월   → imx/*, 나머지 debugging/
```

---

## 재검토에서 확인한 핵심 원칙

1. **"주 1회 이상 호출할 것인가?"** → 아니면 스킬이 아닌 references
2. **"트리거가 다른 스킬과 겹치는가?"** → 겹치면 통합
3. **"도구 없이 정확한 결과를 낼 수 있는가?"** → 못하면 범위 축소
4. **"CLAUDE.md만으로 해결되는가?"** → 해결되면 스킬 불필요
5. **"references는 처음부터 완벽할 필요 없다"** → 사용하면서 점진 축적
