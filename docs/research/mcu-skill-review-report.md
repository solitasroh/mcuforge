# 스킬맵 재검토 보고서

## 검토 관점 5가지

1. **빠진 것 (Gaps)** — 실제 개발에서 필요한데 누락된 영역
2. **과한 것 (Overengineering)** — 스킬로 만들 필요 없는 것, 합치는 게 나은 것
3. **플랫폼 분류 오류** — 적용 범위가 잘못되거나 애매한 것
4. **현실성 문제** — 구현은 가능하지만 실제 가치가 낮은 것
5. **우선순위 문제** — 중요도 재평가가 필요한 것

---

## 1. 빠진 것 (Gaps)

### Gap 1: 협력형 스케줄러(iTask) 전용 지식이 없음
- Accura 제품은 RTOS 없이 iTask 협력형 스케줄러 사용
- 이건 순수 Bare Metal도 아니고 RTOS도 아닌 **중간 영역**
- iTask 특화 패턴이 D3(pattern-library)에 한 줄로만 언급됨
- **수정**: D3 references에 `cooperative-scheduler.md`를 상세히 포함하되, iTask의 타이밍 설계/태스크 분배/ISR-태스크 연동 패턴을 깊게 다룰 것

### Gap 2: 임베디드 리눅스 빌드 시스템 관련 스킬 부재
- i.MX28/6ULL은 Buildroot/Yocto 기반 크로스 컴파일이 핵심
- 디바이스 트리(DTS), 커널 모듈, 루트 파일시스템 커스터마이징
- `build-error-debugger`에서 부분적으로 다루지만 불충분
- **수정**: B7(build-error-debugger)의 references에 `buildroot-troubleshoot.md`, `device-tree-guide.md` 추가

### Gap 3: 통신 프로토콜 디버깅이 D1에만 간접 언급
- Modbus RTU/TCP는 Accura 핵심 프로토콜
- 프로토콜 생성(C6)은 있는데, **프로토콜 분석/디버깅 도구**가 없음
- Wireshark/로직 분석기 출력 해석 가이드 필요
- **수정**: D1(debug-strategist) references에 `modbus-protocol-debug.md` 포함하면 충분. 별도 스킬까진 불필요

### Gap 4: WPF 특화 패턴이 너무 단순
- WPF 메모리 누수는 단순 MVVM 위반이 아닌 **WPF 바인딩 엔진 자체의 특성**에서 발생
  - INotifyPropertyChanged 미구현 시 PropertyDescriptor 강참조 누수
  - INotifyCollectionChanged 미구현 컬렉션 바인딩 누수
  - x:Name 동적 제거 시 필드 참조 잔존
  - 이벤트 핸들러 미해제 (특히 싱글톤 서비스 구독)
  - WeakEventManager 미사용
- **수정**: B2(wpf-code-reviewer) references에 WPF 특유 메모리 누수 패턴을 구체적으로 축적

### Gap 5: Prism 프레임워크 특화 가이드 없음
- AccuraLogic이 Prism 기반 WPF인데, Prism 특화 내용이 전혀 없음
- IEventAggregator 구독/해제, Region Navigation 라이프사이클, DI 컨테이너 관리
- **수정**: D3(pattern-library) references에 `prism-patterns.md` 추가

### Gap 6: 하드웨어-소프트웨어 경계 스킬 없음
- 회로도 ↔ 펌웨어 매핑 (핀 할당, 전기적 특성 → 코드 설정)
- "이 회로에서 R1=100k, R2=10k일 때 ADC 입력 범위는?" 같은 질문
- B6(precision-analyzer)가 부분적으로 다루지만, **핀 할당/전기적 설계 → 코드 매핑**은 별도 영역
- **수정**: B6를 확장하여 "회로→코드 매핑" 기능 포함. 별도 스킬까진 불필요

---

## 2. 과한 것 (Overengineering)

### Over 1: A1~A4 원칙 스킬 4개가 너무 세분화
- Karpathy 4원칙은 **하나의 CLAUDE.md 섹션**이면 충분
- 4개를 별도 스킬로 만들면 트리거 경쟁, 컨텍스트 낭비
- **수정**: A1~A4를 CLAUDE.md에 통합 (스킬이 아닌 프로젝트 원칙)
- A5(coding-standard)만 독립 스킬로 유지 (references/ 참조 필요하므로)

### Over 2: C5(config-generator)와 C2(driver-generator) 중복
- "주변장치 설정 코드 생성"과 "HAL 드라이버 생성"은 실제로 같은 작업의 일부
- 설정 없는 드라이버도 없고, 드라이버 없는 설정도 의미 없음
- **수정**: C2(driver-generator)에 통합. "설정+드라이버 한 번에 생성"

### Over 3: D4(migration-guide)는 스킬보다 문서가 적합
- K22F→STM32 H5 마이그레이션은 일회성 프로젝트 작업
- 매번 스킬로 호출할 빈도가 너무 낮음
- **수정**: 스킬에서 제거, references/에 문서로만 유지. 필요 시 D2(datasheet-navigator)가 참조

### Over 4: D5(errata-checker) 독립 스킬 불필요
- 에라타 확인은 D2(datasheet-navigator)의 하위 기능
- 에라타만 따로 호출하는 빈도 매우 낮음
- **수정**: D2에 통합. references/errata/ 디렉토리만 유지

---

## 3. 플랫폼 분류 수정

### 수정 1: i.MX28은 "Embedded Linux"가 맞지만 디바이스 드라이버는 Bare Metal에 가까움
- i.MX28의 GPMI NFC 드라이버, FEC 이더넷 드라이버 등은 커널 레벨에서 레지스터 직접 접근
- B1(mcu-code-reviewer)를 "△ 부분 적용"으로 표시했는데, 실제로는 **커널 드라이버 작성 시 필수**
- **수정**: 적용 매트릭스에서 i.MX28/6ULL의 B1 적용도를 더 명확히 표기
  - "유저스페이스 앱": B3(linux-code-reviewer) 적용
  - "커널 드라이버": B1(mcu-code-reviewer) + B3 모두 적용

### 수정 2: STM32 H5의 TrustZone이 빠짐
- H5는 Cortex-M33 기반으로 TrustZone(Secure/Non-Secure) 지원
- K22F(M4)와 아키텍처적으로 다른 점이 여러 개 있음
- **수정**: D2(datasheet-navigator) references에 `h5-trustzone-guide.md` 추가 검토

### 수정 3: WPF에서 .NET Framework vs .NET 8+ 분기 필요
- 현재 AccuraLogic이 .NET Framework 기반이지만, 향후 .NET 8 마이그레이션 가능성
- 두 환경의 WPF 차이점 (System.Windows.Forms 호스팅, NuGet 패키지 차이 등)
- **수정**: 지금은 .NET Framework 기준으로 작성하되, references에 마이그레이션 참고 포함

---

## 4. 현실성 검증

### 현실 1: B5(ai-code-verifier)는 시기상조일 수 있음
- WCET 추정, 스택 사용량 정적 분석은 **도구 없이는 정확도가 낮음**
- Claude가 코드를 읽고 "이 함수는 스택을 약 N 바이트 사용"이라고 추정할 수 있지만, 컴파일러 최적화를 모름
- 실질적으로는 B1(mcu-code-reviewer)에서 "동적 할당 사용, 큰 로컬 배열" 정도만 경고하는 것이 현실적
- **수정**: B5를 삭제하고, B1의 리뷰 항목에 "AI 생성 코드 특화 체크리스트" 추가
  - 불필요한 추상화 탐지 (AI가 좋아하는 과도한 패턴)
  - 동적 할당 사용 여부
  - 에러 핸들링 누락
  - 하드웨어 레지스터 접근 패턴 검증

### 현실 2: C2(driver-generator)의 데이터시트 PDF 파싱은 불안정
- 이전 평가에서도 ★★★☆☆으로 낮게 평가
- **현실적 접근**: PDF 자동 파싱 포기. 대신 **구조화된 입력 형식** 정의
  - YAML/텍스트로 레지스터 맵을 미리 정리
  - 또는 NXP/STM32 헤더 파일(.h)의 레지스터 정의를 파싱
- **수정**: C2의 입력을 "데이터시트 PDF"가 아닌 "헤더 파일 또는 구조화된 명세"로 명확히

### 현실 3: D2(datasheet-navigator)의 references 구축 비용
- MCU별 주요 레지스터 맵을 references/에 정리하는 것은 **초기 노력이 큼**
- K22F + K10D + STM32 H5 + i.MX28 + i.MX6ULL = 5개 플랫폼
- 각각 ADC, SPI, UART, I2C, DMA, Timer만 해도 30개+ 문서
- **현실적 접근**: 처음에는 가장 많이 쓰는 주변장치(ADC, UART, SPI)만 정리
  - 나머지는 사용 시점에 점진적으로 축적
  - "이 정보가 없으면 데이터시트를 참조하세요"라는 폴백

---

## 5. 최종 수정된 스킬 목록

### 삭제/통합 결과

| 원래 | 변경 | 이유 |
|------|------|------|
| A1~A4 (4개 원칙 스킬) | → CLAUDE.md 통합 | 스킬이 아닌 프로젝트 원칙 |
| C5 (config-generator) | → C2에 통합 | 드라이버와 설정은 같은 작업 |
| D4 (migration-guide) | → references 문서 | 일회성 작업, 스킬 빈도 낮음 |
| D5 (errata-checker) | → D2에 통합 | 데이터시트 네비게이터의 하위 기능 |
| B5 (ai-code-verifier) | → B1 체크리스트 확장 | 독립 도구 없이는 정확도 낮음 |

### 최종 스킬 목록 (24개 → 19개)

```
CLAUDE.md (프로젝트 원칙)
├── Karpathy 4원칙 (Think / Simplicity / Surgical / Goal-Driven)
├── MCU 제약 원칙 (ISR 최소화, 정적 할당 우선, 레지스터 순서 보존)
└── WPF 원칙 (MVVM 준수, INotifyPropertyChanged 필수, 이벤트 해제)

A. 원칙 스킬 (1개)
├── A5. coding-standard — 플랫폼별 네이밍/구조 규칙 + references

B. 분석/검증 스킬 (5개)
├── B1. mcu-code-reviewer — 6종 리뷰어 + AI 코드 특화 체크리스트
├── B2. wpf-code-reviewer — MVVM/바인딩누수/Prism 패턴 검증
├── B3. linux-code-reviewer — 시스템프로그래밍/크로스컴파일/BusyBox
├── B4. linker-map-analyzer — .map → 메모리 리포트 (Python 스크립트)
└── B6. precision-analyzer — ADC/회로 정밀도 + 회로→코드 매핑

C. 생성/자동화 스킬 (5개)
├── C1. test-generator — Unity(C) / MSTest(C#) 테스트 코드 생성
├── C2. driver-generator — 주변장치 설정 + HAL 드라이버 (통합)
├── C3. doc-generator — 기술 문서 (API 레퍼런스, 아키텍처, 동작분석)
├── C4. commit-assistant — 커밋 메시지, MR 설명, 릴리즈노트
├── C6. protocol-generator — Modbus RTU/TCP, UART 프레임 파서
└── C7. manual-writer — (기존 스킬) 사용자 매뉴얼

D. 지식/레퍼런스 스킬 (3개)
├── D1. debug-strategist — 4단계 디버깅 + 증상 DB + 프로토콜 디버깅
├── D2. datasheet-navigator — 레지스터맵 + 에라타 + TrustZone (통합)
└── D3. pattern-library — 설계 패턴 DB (iTask 상세 포함, Prism 패턴 포함)

B7. build-error-debugger — ARM GCC/MSBuild + Buildroot/DTS 가이드

총: CLAUDE.md + 15개 독립 스킬
```

### 수정된 워크플로우 체인

```
새 기능 (Bare Metal):
  CLAUDE.md(원칙) → D2(데이터시트) → C2(드라이버+설정) → C1(테스트) → B1(리뷰) → C4(커밋)

새 기능 (WPF):
  CLAUDE.md(원칙) → D3(Prism패턴) → C1(테스트) → B2(리뷰) → C4(커밋)

버그 수정 (공통):
  D1(디버깅전략) → C1(재현테스트) → B1/B2/B3(리뷰) → C4(커밋)

릴리즈 (Bare Metal):
  B4(링커맵) → B1(최종리뷰) → C3(기술문서) → C4(릴리즈노트) → C7(매뉴얼)
```

---

## 6. 추가 권장사항

### references/ 축적 전략 (현실적)
1단계 (즉시): 코딩 표준, 커밋 컨벤션, MISRA 서브셋 — 이미 지식 있음
2단계 (1개월): K22F ADC/UART/SPI 레지스터맵, K10D 16bit ADC — 현재 활발 사용 중
3단계 (3개월): STM32 H5 주요 주변장치, i.MX28 GPMI/FEC — 사용 시점에
4단계 (6개월): WPF/Prism 패턴, 디버깅 DB — 경험 축적과 함께

### 스킬 vs CLAUDE.md vs references 판단 기준
- **CLAUDE.md**: 모든 작업에 항상 적용되는 원칙 (Karpathy, 메모리 제약)
- **독립 스킬**: 특정 트리거에 반응하고, 구조화된 출력을 생성하는 것
- **references**: 여러 스킬이 참조하는 도메인 지식, 점진적으로 축적

### 과도한 스킬 분리를 피하는 규칙
"이 스킬을 주 1회 이상 호출할 것인가?" → 아니면 references로 내림
"이 스킬의 트리거가 다른 스킬과 겹치는가?" → 겹치면 통합
"이 스킬 없이도 CLAUDE.md만으로 해결되는가?" → 해결되면 불필요
