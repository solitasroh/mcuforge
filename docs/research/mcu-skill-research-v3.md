# MCU 개발 스킬 연구 v3: 인기 스킬 분석 + Karpathy 원칙 통합

## Part 1: 인기 스킬 생태계 분석

### Top-Tier 스킬/프레임워크 (GitHub Stars 기준)

| 프로젝트 | Stars | 핵심 가치 | MCU 적용 시사점 |
|----------|-------|----------|----------------|
| **Superpowers** (obra) | 42K+ | 7단계 개발 방법론: 브레인스토밍→설계→계획→TDD→서브에이전트→코드리뷰→완료 | **가장 큰 영감원**. "빠른 AI"보다 "규율 있는 AI"가 더 가치 있다는 증명 |
| **Antigravity Awesome Skills** | 22K+ | 1,234개 스킬, 12개 에이전트 호환, 역할별 번들 | 스킬 수보다 **번들(묶음)** 전략이 중요 |
| **Jeffallan/claude-skills** | 6.8K | 66개 풀스택 스킬, 워크플로우 체인, progressive disclosure | references/ 구조와 도메인별 조직 참고 |
| **alirezarezvani/claude-skills** | 5.2K | 192개 스킬, 254개 Python 도구, 11개 에이전트 호환 | Python 스크립트 기반 도구 패턴 참고 |
| **Karpathy Guidelines** | 251 | 4가지 원칙 (Think→Simplicity→Surgical→Goal-Driven) | CLAUDE.md 레벨 원칙으로 모든 스킬에 적용 |

### 인기 스킬의 공통 성공 패턴

**1. Superpowers의 핵심 교훈**
- "코드를 더 빨리 쓰는 것"이 아닌 "소프트웨어 엔지니어처럼 일하는 것"이 가치
- 강제(enforcement), 제안(suggestion)이 아님 — TDD 없이 코드 쓰면 삭제시킴
- 서브에이전트별 단일 태스크 → 2단계 리뷰(스펙 준수 + 코드 품질)
- GraphViz dot 표기로 프로세스 문서화 (Claude가 플로우차트를 더 잘 따름)
- 부트스트랩 2,000 토큰 이하, 나머지 on-demand 로딩

**2. Karpathy 원칙의 핵심 교훈**
문제 정의:
- "모델이 가정을 세우고 확인 없이 달려감"
- "과도한 복잡성, 불필요한 추상화, 죽은 코드 미정리"
- "관련 없는 코드/주석을 부수적으로 변경"

4가지 해법:
```
1. Think Before Coding — 가정 명시, 모호함에 질문, 트레이드오프 제시
2. Simplicity First — 요청된 것만, 투기적 기능 금지, 200줄→50줄
3. Surgical Changes — 요청에 직접 연결되는 변경만, 주변 "개선" 금지
4. Goal-Driven Execution — 명령형→선언형 목표, 검증 루프, 성공 기준 정의
```

**3. NeoLabHQ/code-review 패턴**
전문화된 리뷰어 에이전트 6종:
- bug-hunter
- security-auditor
- code-quality-reviewer
- contracts-reviewer (API 계약)
- historical-context-reviewer (git 히스토리 기반)
- test-coverage-reviewer

→ MCU 적용: ISR-safety-reviewer, memory-bounds-reviewer, volatile-reviewer 등으로 특화 가능

**4. obra/superpowers-lab 실험적 스킬**
- **semantic duplicate finder**: 구문적 복사가 아닌 "같은 의도의 다른 구현" 탐지
- Claude Haiku로 1차 분류 → Opus로 심층 분석하는 2단계 접근
→ MCU 적용: 동일 주변장치의 중복 초기화 코드, 유사 ISR 패턴 탐지에 활용 가능

**5. K-Dense/Claude Scientific Skills**
- 과학/공학 분석에 특화된 스킬 세트
- "PhD 공부 대신 이 문서를 읽으라"는 평가
→ MCU 적용: 회로 분석, 신호 처리, 제어 이론 등 공학적 분석 스킬의 선례

---

## Part 2: Karpathy 원칙을 MCU 개발에 매핑

### CLAUDE.md 레벨 원칙 (모든 스킬에 공통 적용)

```markdown
# MCU Firmware Development Principles

## 1. Think Before Coding (Karpathy #1)
- 레지스터 설정 전 데이터시트 확인 여부를 명시
- 인터럽트 우선순위/중첩 가정을 명시적으로 서술
- 메모리 제약 (Flash/RAM 잔여량) 확인 후 설계
- "이 코드가 ISR에서 호출될 수 있나?" 항상 고려

## 2. Simplicity First (Karpathy #2)
- 동적 할당 대신 정적 할당 우선
- 복잡한 상태머신보다 단순 플래그 패턴 우선
- 불필요한 추상화 레이어 금지 (HAL 위의 HAL 금지)
- "이 MCU에 이 수준의 복잡성이 필요한가?" 항상 질문

## 3. Surgical Changes (Karpathy #3)  
- ADC 설정 수정 시 UART 코드 건드리지 않기
- 기존 인터럽트 핸들러 "개선" 금지
- 레지스터 설정 순서가 의도적일 수 있음 — 함부로 재배치 금지

## 4. Goal-Driven Execution (Karpathy #4)
- "ADC 드라이버 작성" → "ADC에서 3.3V 입력 시 4095 읽히는 것 확인"
- "통신 구현" → "Modbus FC03 요청에 올바른 응답 반환 확인"
- 성공 기준을 하드웨어 동작으로 정의
```

---

## Part 3: 확장된 스킬 설계 (인기 스킬 패턴 반영)

### 설계 철학 변화

기존(v2) 접근: "기능별로 12개 스킬"
새로운 접근: **"개발 워크플로우에 따른 스킬 체인 + Karpathy 원칙 내장"**

Superpowers에서 배운 것:
- 개별 스킬보다 **워크플로우 전체**를 설계하는 것이 더 가치 있음
- 스킬은 "제안"이 아니라 "강제"할 때 효과적
- 서브에이전트 패턴으로 각 단계를 격리

### 스킬 아키텍처 재설계

```
mcu-dev-skills/
├── CLAUDE.md                          # Karpathy 4원칙 + MCU 제약
├── .claude/skills/
│   │
│   ├── mcu-think-first/               # 워크플로우 #1: 설계 전 사고
│   │   ├── SKILL.md                   # "코딩 전에 이것부터 확인"
│   │   └── references/
│   │       ├── hardware-checklist.md   # MCU별 제약 체크리스트
│   │       ├── k22f-constraints.md     # K22F 메모리/주변장치 제약
│   │       ├── k10d-constraints.md     # K10D 특화 제약
│   │       └── imx28-constraints.md    # i.MX28 Linux 제약
│   │
│   ├── mcu-code-reviewer/             # 워크플로우 #2: 코드 품질 강제
│   │   ├── SKILL.md                   # 6종 전문 리뷰어 체크리스트
│   │   └── references/
│   │       ├── isr-safety.md           # ISR 안전성 규칙
│   │       ├── memory-safety.md        # 메모리 안전성 규칙  
│   │       ├── misra-c-subset.md       # MISRA-C 핵심 규칙
│   │       ├── volatile-rules.md       # volatile 사용 규칙
│   │       ├── naming-convention.md    # Rootech 네이밍 표준
│   │       └── common-bugs.md          # MCU 흔한 버그 패턴 DB
│   │
│   ├── mcu-test-generator/            # 워크플로우 #3: TDD 강제
│   │   ├── SKILL.md                   # "테스트 먼저, 구현은 그 다음"
│   │   ├── scripts/
│   │   │   └── generate_unity_test.py  # Unity 테스트 스켈레톤 생성
│   │   └── references/
│   │       ├── unity-patterns.md       # Unity 테스트 프레임워크 패턴
│   │       ├── mock-strategies.md      # 하드웨어 Mock 전략
│   │       └── boundary-values.md      # MCU 특화 경계값 가이드
│   │
│   ├── linker-map-analyzer/           # 도구 스킬 #1: 메모리 분석
│   │   ├── SKILL.md
│   │   └── scripts/
│   │       └── parse_map.py            # ARM GCC .map 파서
│   │
│   ├── mcu-debug-strategist/          # 도구 스킬 #2: 디버깅 전략
│   │   ├── SKILL.md                   # Superpowers systematic-debugging MCU 버전
│   │   └── references/
│   │       ├── hardfault-diagnosis.md
│   │       ├── communication-debug.md
│   │       ├── adc-measurement-debug.md
│   │       └── timing-debug.md
│   │
│   ├── mcu-doc-generator/             # 도구 스킬 #3: 문서 생성
│   │   ├── SKILL.md
│   │   └── references/
│   │       ├── doc-templates.md
│   │       └── rootech-style.md
│   │
│   ├── commit-assistant/              # 도구 스킬 #4: Git 워크플로우
│   │   ├── SKILL.md
│   │   └── references/
│   │       ├── commit-convention.md
│   │       └── mr-template.md
│   │
│   └── precision-analyzer/            # 도구 스킬 #5: 정밀도 분석
│       ├── SKILL.md
│       └── references/
│           ├── adc-error-model.md
│           ├── ieee43-reference.md
│           └── temperature-compensation.md
│
└── references/                        # 공유 레퍼런스 (여러 스킬에서 참조)
    ├── k22f-register-map/
    │   ├── adc.md
    │   ├── spi.md
    │   └── uart.md
    ├── k10d-register-map/
    │   └── adc16.md
    └── errata/
        ├── k22f-errata.md
        └── k10d-errata.md
```

### 핵심 스킬 상세 설계

#### 1. mcu-think-first (Karpathy #1 구현)

**트리거**: 새 기능 구현, 드라이버 작성, 모듈 설계 시작 시
**강제 동작**: 코딩 전 다음 질문에 답하도록 요구

```markdown
## 코딩 전 체크리스트

### 하드웨어 제약 확인
- [ ] 타겟 MCU와 가용 Flash/RAM 확인
- [ ] 사용할 주변장치의 클럭 설정 확인
- [ ] 핀 할당 충돌 없는지 확인
- [ ] 전원 모드 영향 확인 (Sleep에서 동작해야 하는가?)

### 인터럽트 설계
- [ ] 어떤 인터럽트를 사용하는가?
- [ ] 우선순위 레벨은?
- [ ] ISR과 main 사이 공유 데이터는 무엇인가?
- [ ] 원자적 접근이 필요한 변수가 있는가?

### 메모리 설계
- [ ] 정적 할당으로 충분한가?
- [ ] 버퍼 크기는 어떻게 결정했는가?
- [ ] 스택 사용량 추정치는?

### 성공 기준 (Karpathy #4)
- [ ] 이 기능의 검증 가능한 성공 기준은 무엇인가?
- [ ] 하드웨어 없이 테스트 가능한 부분은?
```

#### 2. mcu-code-reviewer (NeoLab 6종 리뷰어 패턴 적용)

**트리거**: C 코드 리뷰 요청, PR 리뷰, 코드 완성 후 점검
**6종 전문 리뷰어**:

| 리뷰어 | 체크 항목 | 심각도 |
|--------|----------|--------|
| **ISR Safety** | ISR 내 블로킹 호출, ISR 실행 시간, 중첩 인터럽트 안전성 | CRITICAL |
| **Memory Safety** | 배열 경계, NULL 역참조, 스택 오버플로우, 정렬 | CRITICAL |
| **Volatile/Atomic** | 공유 변수 volatile, 비원자적 접근, 크리티컬 섹션 | CRITICAL |
| **Code Quality** | 매직넘버, 함수 길이, 네이밍, 복잡도 | MEDIUM |
| **MISRA Subset** | MISRA-C 핵심 규칙 15개 서브셋 | HIGH |
| **Resource Efficiency** | 불필요한 폴링, sleep 미사용, 중복 초기화 | LOW |

**출력 형식** (Superpowers의 severity 분류 채용):
```
## Code Review Report

### CRITICAL (반드시 수정)
- [ISR Safety] `UART_Send()` in `ADC_IRQHandler` (line 42): ISR 내 블로킹 호출
  → 플래그 설정 후 main에서 전송으로 변경

### HIGH (수정 권장)
- [MISRA 11.3] `(uint32_t*)ptr` (line 78): 포인터-정수 캐스트
  → `uintptr_t` 사용 또는 `memcpy` 패턴

### MEDIUM (개선 제안)
- [Code Quality] `process_data()` 87줄: 50줄 이하로 분할 권장
```

#### 3. mcu-debug-strategist (Superpowers systematic-debugging MCU 버전)

**트리거**: 버그 리포트, 예상과 다른 동작, HardFault 등
**4단계 디버깅 방법론** (Superpowers에서 차용):

```
Phase 1: 조사 (Investigate)
  - 증상 정확히 기술
  - 재현 조건 파악
  - 최근 변경사항 확인

Phase 2: 패턴 분석 (Analyze)
  - MCU 특화 버그 패턴 DB에서 유사 사례 검색
  - 하드웨어 vs 소프트웨어 원인 분류
  - 관련 에라타 확인

Phase 3: 가설 검증 (Hypothesis)
  - 가설 우선순위 정렬
  - 각 가설의 검증 방법 제시
  - "3회 실패 시 아키텍처 리뷰" (Superpowers 규칙)

Phase 4: 수정 및 검증 (Fix & Verify)
  - 근본 원인 수정
  - 회귀 테스트 작성
  - 유사 패턴 코드베이스 검색
```

**증상별 빠른 진입점**:
```
"ADC 값이 이상해요"      → adc-measurement-debug.md
"HardFault 발생"         → hardfault-diagnosis.md  
"통신이 끊겨요"           → communication-debug.md
"타이밍이 안 맞아요"      → timing-debug.md
"간헐적 오동작"           → race-condition-debug.md
```

---

## Part 4: 구현 전략

### Phase 1: 기반 구축 (1주)
1. **CLAUDE.md** 작성 — Karpathy 4원칙 MCU 버전
2. **mcu-code-reviewer** 초안 — 6종 리뷰어 체크리스트
3. **공유 references/** — naming-convention.md, common-bugs.md

### Phase 2: 핵심 도구 (2주)
4. **linker-map-analyzer** — Python 스크립트 + 리포트
5. **commit-assistant** — Conventional Commits + MR 템플릿
6. **mcu-think-first** — 코딩 전 체크리스트 강제

### Phase 3: 고급 워크플로우 (3~4주)
7. **mcu-test-generator** — Unity 테스트 생성
8. **mcu-debug-strategist** — 4단계 디버깅 + 증상 DB
9. **mcu-doc-generator** — 코드 기반 기술 문서

### Phase 4: 특화 도구 (필요 시)
10. **precision-analyzer** — ADC/회로 정밀도 분석
11. 데이터시트 레지스터 맵 references 축적

### 환경별 전략

| 구성 요소 | claude.ai | Claude Code |
|----------|-----------|-------------|
| CLAUDE.md 원칙 | 프로젝트 설명에 포함 | 프로젝트 루트 |
| SKILL.md | 업로드 스킬 | .claude/skills/ |
| references/ | 스킬 내 번들 | 디렉토리 참조 |
| scripts/ | bash_tool로 실행 | 직접 실행 + hook 연동 |
| 강제(enforcement) | 프롬프트 기반 | hook + sub-agent |

---

## Part 5: 핵심 인사이트 요약

### "왜 이 스킬들이 수만 개의 Star를 받았는가?"

1. **규율(Discipline) > 속도(Speed)**
   - Superpowers (42K stars): "코드 빨리 쓰는 AI"가 아닌 "규율 있는 AI"
   - Karpathy: "잘못된 가정으로 달려가는 것을 막는 것"이 핵심

2. **강제(Enforce) > 제안(Suggest)**  
   - TDD 스킬: 테스트 없이 코드 쓰면 삭제
   - 코드리뷰: 자동 트리거, 선택이 아닌 필수

3. **워크플로우(Chain) > 단일 기능(Single)**
   - brainstorming → plan → TDD → subagent → review → complete
   - 개별 스킬의 합보다 체인의 가치가 훨씬 큼

4. **Progressive Disclosure 필수**
   - 부트스트랩 < 2,000 토큰
   - 필요한 references만 on-demand 로딩
   - 500줄 이하 SKILL.md + 무제한 references/

5. **도메인 지식은 references/에 분리**
   - SKILL.md = "어떻게 일할 것인가" (워크플로우, 판단 로직)
   - references/ = "무엇을 알아야 하는가" (체크리스트, 규칙, 패턴 DB)
   - 같은 references를 여러 스킬이 공유

### MCU 개발에 대한 핵심 결론

임베디드 개발은 웹 개발보다 **더 규율이 중요한 도메인**:
- 하드웨어 제약이 실수의 비용을 높임 (배포 후 수정 어려움)
- ISR/volatile/메모리 같은 실수는 "런타임에야 발견"되는 경우 많음
- 팀이 10→50명으로 확장될 때 코딩 스타일/안전성 규칙 일관성이 핵심

따라서 Superpowers + Karpathy 접근이 MCU에 **더욱 적합**:
- Think Before Coding → 하드웨어 제약 먼저 확인
- Simplicity First → MCU에서 복잡성은 곧 버그
- Surgical Changes → 레지스터 설정은 순서와 의존성이 있음
- Goal-Driven → 성공 기준을 하드웨어 동작으로 정의
- TDD Enforcement → 하드웨어 Mock으로 단위 테스트 강제
- Code Review Enforcement → ISR/메모리/volatile 안전성 자동 점검
