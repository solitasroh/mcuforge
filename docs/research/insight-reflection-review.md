# 리서치 인사이트 반영 상태 검토 보고서

> 검토일: 2026-03-18
> 대상: `insight-gap-analysis.md`의 10개 인사이트 → 실제 skills/ 구현체 대조
> 범위: 29 skills, 12 agents, 18 hooks, manifest.json, settings.json.tmpl

---

## 검토 결과 요약

| # | 인사이트 | 반영 상태 | 핵심 근거 |
|---|---------|:---------:|----------|
| 1 | Superpowers 7단계 강제 워크플로우 | **PARTIAL** | 규칙 강제 O → 단계 순서 강제 X |
| 2 | GraphViz dot 표기 | **NONE** | .dot 파일 0개, 플로우차트 없음 |
| 3 | NeoLab 6종 독립 리뷰어 | **PARTIAL** | 카테고리 존재 → 독립 실행·git 히스토리 X |
| 4 | Semantic Duplicate Finder | **NONE** | 의도 기반 중복 탐지 스킬 없음 |
| 5 | K-Dense Scientific Skills | **NONE** | 정밀도 분석/계산 스킬·스크립트 없음 |
| 6 | Progressive Disclosure / 토큰 예산 | **PARTIAL** | 구조 합리적 → 공식 가이드라인 X |
| 7 | 역할별 번들 전략 | **NONE** | 카테고리만 있고 페르소나 번들 없음 |
| 8 | Embedder 하드웨어 카탈로그 | **PARTIAL** | ref-hardware 존재 → 문서 그룹 참조 X |
| 9 | 2,000 토큰 부트스트랩 제한 | **PARTIAL** | inject-context 존재 → CLAUDE.md 없음 |
| 10 | stdlib-only Python 스크립트 | **NONE** | Python 1개(op.py)만 존재 |

**결과: FULL 0개 / PARTIAL 5개 / NONE 5개**

---

## 인사이트별 상세 분석

### 1. Superpowers 7단계 강제 워크플로우 — PARTIAL

**gap-analysis 제안**: CLAUDE.md에 "설계→테스트→구현→리뷰" 순서 강제 + hooks로 구현

**반영된 부분:**
- 18개 hooks가 `PreToolUse`에서 **개별 코딩 규칙을 blocking으로 강제**
  - `check-float-suffix.sh`, `check-stdint-types.sh`, `check-driver-boundary.sh` 등
  - Write/Edit 시 위반 자동 차단
- `check-verify`가 5개 verify-* 스킬을 **순차 실행하여 통합 검증**
- `do-debug`에 4단계 체계적 프레임워크 (Investigate→Analyze→Hypothesize→Fix)
- `pdca-iterator` 에이전트가 Plan-Do-Check-Act 반복 사이클 제공

**미반영:**
- **개발 단계 순서 강제 메커니즘 없음**
  - "테스트 없이 구현 코드를 제출하지 않는다" 같은 hard rule이 어디에도 없음
  - hooks는 "코드 작성 시 규칙 위반 차단"이지, "테스트 먼저 작성" 같은 워크플로우 순서 강제가 아님
- **CLAUDE.md 자체가 존재하지 않음** → 강제 규칙을 넣을 기반이 없음
- 서브에이전트 위임 전략 미정의 (Superpowers의 Stage 5)

**검증 파일:**
- `skills/settings.json.tmpl` — hooks 정의 (규칙 강제만)
- `skills/universal/check-verify/SKILL.md` — verify-* 순차 실행 (규칙 검증만)

---

### 2. GraphViz dot 표기 — NONE

**gap-analysis 제안**: 분기 복잡한 스킬(B1, D1)에 dot 플로우차트 적용

**현황:**
- 프로젝트 전체에 `.dot` 파일 **0개**
- 모든 SKILL.md가 마크다운 산문(prose)으로만 워크플로우 기술
- 분기가 복잡한 스킬도 번호 리스트로만 표현:
  - `check-code-review`: Safety Verification 5단계 → 산문
  - `do-debug`: 4단계 + MCU 시나리오 4개 분기 → 산문
  - `check-verify`: 6단계 → 산문
  - `do-test-gen`: 3개 모드(--design, --stub, normal) → 산문

**영향:**
- 리서치(v3)에서 "Claude는 산문보다 dot 플로우차트를 더 정확하게 따른다"고 확인했으나 미적용
- 특히 `check-code-review`(142줄)와 `do-debug`(152줄)처럼 조건부 분기가 많은 스킬에서 효과적

---

### 3. NeoLab 6종 독립 리뷰어 — PARTIAL

**gap-analysis 제안**: 독립 실행 + git 히스토리 기반 리뷰 추가

**반영된 부분:**
- `check-code-review` SKILL.md에 4개 리뷰 카테고리:
  - Safety (ISR reentrancy, volatile, critical sections)
  - Performance (stack usage, soft-float penalty)
  - Correctness (CMSIS, peripheral clock gating)
  - Style (naming, float suffix)
- `embedded-reviewer` 에이전트에 4개 렌즈:
  - Safety → Correctness → Architecture → Performance (순차 적용)
- 리뷰 관련 에이전트가 3개로 일부 분리:
  - `embedded-reviewer` (코드 리뷰 전반)
  - `hardware-interface-reviewer` (하드웨어 인터페이스)
  - `code-quality-analyzer` (코드 품질)

**미반영:**
- **단일 패스 리뷰**: 모든 관점이 하나의 에이전트에서 한 번에 실행
  - NeoLab처럼 "ISR-safety만 보는 패스", "concurrency만 보는 패스"를 독립 실행하지 않음
  - `embedded-reviewer.md`가 1~4 렌즈를 순차 적용하지만, 실행은 1회
- **git 히스토리 기반 리뷰 없음**
  - `historical-context-reviewer` 에이전트 없음
  - "이 함수가 최근 3번 수정 → 이번 변경이 이전 의도와 충돌?" 분석 기능 없음
  - `check-code-review` Step 1에서 `git diff`만 사용, `git log --follow` 미사용

**검증 파일:**
- `skills/embedded-c/check-code-review/SKILL.md:20-26` — 4개 카테고리, 단일 워크플로우
- `skills/agents/embedded-reviewer.md:29-89` — 4개 렌즈, 1회 실행

---

### 4. Semantic Duplicate Finder — NONE

**gap-analysis 제안**: B1의 선택적 리뷰 항목으로 추가 또는 독립 스킬

**현황:**
- **어떤 스킬에도 의도 기반 중복 탐지 기능 없음**
- `refactoring-advisor` 에이전트: 리팩토링 조언 전용, 중복 탐지 아님
- `plan-quality-audit`: cyclomatic complexity, function length 등 → 구문적 수준
- `code-quality-analyzer` 에이전트: duplication 언급 가능하나 "같은 의도의 다른 구현" 탐지가 아님
- Haiku→Opus 2단계 접근법 미적용

**비고:**
- MCU 프로젝트에서 여러 제품의 유사 주변장치 초기화 코드 중복은 실제 문제
- 현재 이를 잡아내는 자동화된 메커니즘 없음

---

### 5. K-Dense Scientific Skills — NONE

**gap-analysis 제안**: precision-analyzer 스킬 + 계산 스크립트(precision_calc.py) 생성

**현황:**
- `precision-analyzer` 스킬 없음 (manifest.json에 미등록)
- ADC 분해능/오차 전파/RSS 계산 스크립트 없음
- references/에 공학 공식 레퍼런스 없음
  - `adc-error-model.md` 없음
  - `ieee43-reference.md` 없음
- `ref-hardware`에 레지스터 맵만 있고 **계산 기반 분석** 없음
- `check-binary-analysis`에 memory budget이 있으나 "정밀도 분석"과 무관

**비고:**
- Accura 2750IRM 같은 정밀 측정 장비 개발에서 결정적 가치
- "설명만 하지 말고, 계산도 해라"는 K-Dense 철학 미적용

---

### 6. Progressive Disclosure / 토큰 예산 — PARTIAL

**gap-analysis 제안**: 토큰 예산 가이드라인 수립 + 3계층 로딩 시스템 명시

**반영된 부분:**
- SKILL.md 파일 크기가 대체로 합리적:
  ```
  최소: 32줄  (do-build)
  최대: 290줄 (do-test-gen)
  평균: ~130줄
  전체: 3,767줄 (29 skills)
  ```
- references/ 폴더로 상세 지식 분리 (5개 스킬):
  - `ref-coding-rules/references/safety-rules.md`
  - `do-test-gen/references/` (3개 파일)
  - `ref-hardware/references/` (2개 파일)
  - `ref-architecture/references/common-patterns.md.tmpl`
  - `plan-openproject/references/` (6개 파일)
- description이 "pushy"하게 작성됨 (트리거 키워드 풍부)
- `disable-model-invocation: true`로 불필요한 로딩 방지 (ref-*, verify-*)

**미반영:**
- **공식 토큰 예산 가이드라인 미문서화**
  - "SKILL.md <300줄", "references/ 각 <300줄" 같은 기준이 어디에도 없음
  - 실제로 `verify-barr-c`(271줄), `do-test-gen`(290줄)이 큰 편
- **3계층 로딩 시스템 미명시**
  - 메타데이터→본문→리소스 구분이 암묵적으로만 존재
  - 개발자가 새 스킬 작성 시 참고할 가이드 없음
- CLAUDE.md가 없어서 "상시 로드 ~1,000토큰" 기준 자체가 없음

---

### 7. 역할별 번들 전략 — NONE

**gap-analysis 제안**: manifest.json에 bundles 섹션 + 역할별 스킬 조합 정의

**현황:**
- manifest.json이 `universal/embedded_c/templates` 3개 카테고리로 분류
- 이는 **기술 도메인 분류**이지 **역할/페르소나 분류**가 아님
- 다음이 모두 부재:
  - "펌웨어 개발자 번들" 정의
  - "WPF 개발자 번들" 정의
  - "신입 온보딩 번들" 정의
  - 번들 선택 메커니즘 (manifest.json의 bundles 섹션)
- 사용자가 자신의 역할에 맞는 스킬 조합을 알 방법 없음

**비고:**
- 현재 29개 스킬 전부가 모든 사용자에게 노출되는 구조
- 팀 확장 시 (10명→50명) 역할 분화에 대응 불가

---

### 8. Embedder 하드웨어 카탈로그 (문서 그룹 참조) — PARTIAL

**gap-analysis 제안**: "관련 문서 그룹 참조" 패턴을 SKILL.md에 포함

**반영된 부분:**
- `ref-hardware`: peripherals.md + memory-map.md로 하드웨어 레퍼런스 분리
- `check-code-review`의 Peripheral Initialization Checklist가 ADC/UART/GPIO/FTM/DMA별 상세 체크리스트 제공 (lines 71-137)
- `embedded-reviewer` 에이전트의 skills 참조 목록: `ref-coding-rules`, `ref-architecture`, `ref-hardware`

**미반영:**
- **"문서 그룹 참조" 패턴 없음**
  - "ADC 질문 시 → register map + errata + debug guide + HAL pattern을 동시 참조하라"는 지시가 어떤 SKILL.md에도 없음
  - 현재는 단일 레퍼런스만 참조 가능한 구조
- **errata 문서 없음**: 하드웨어 에라타 레퍼런스 자체가 없음
- **do-debug ↔ ref-hardware 연동 미흡**:
  - `do-debug`가 `ref-hardware`를 참조하라는 지시 없음
  - `do-debug`의 agent는 `systematic-debugger` (ref-hardware 미연결)
  - `hardware-interface-reviewer`만 ref-hardware 참조

**검증 파일:**
- `skills/templates/ref-hardware/SKILL.md.tmpl:59-62` — 단순 SVD/CMSIS 참조만
- `skills/agents/embedded-reviewer.md:3-6` — ref-coding-rules, ref-architecture, ref-hardware 동시 참조 (유일한 그룹 참조 사례)

---

### 9. 2,000 토큰 부트스트랩 제한 — PARTIAL

**gap-analysis 제안**: CLAUDE.md ≤1,000토큰 + SKILL.md body ≤2,500토큰

**반영된 부분:**
- `inject-context` hook (SessionStart)이 세션 시작 시 컨텍스트 주입 → 부트스트랩 역할
- `save-pre-compact` hook (PreCompact)이 컨텍스트 압축 전 상태 보존
- SKILL.md description이 충분히 상세하여 트리거 판단에 별도 로딩 불필요

**미반영:**
- **CLAUDE.md 파일 없음**: 프로젝트 루트에 항상 로드될 핵심 원칙 문서 자체가 없음
  ```bash
  $ find /home/user/mcuforge -name "CLAUDE.md" -maxdepth 2
  # 결과 없음
  ```
- **공식 부트스트랩 토큰 제한 미정의**
  - "상시 로드 ≤1,000토큰" 기준 없음
  - "SKILL.md body ≤2,500토큰" 기준 없음
- inject-context.sh의 실제 주입 내용/크기 미확인

---

### 10. stdlib-only Python 스크립트 — NONE

**gap-analysis 제안**: stdlib-only 원칙 수립 + parse_map.py, generate_unity_test.py 등 생성

**현황:**
- 프로젝트 전체에 Python 스크립트 **1개만** 존재:
  - `skills/templates/plan-openproject/scripts/op.py` (OpenProject API 연동)
- gap-analysis에서 제안된 스크립트들 부재:
  - `parse_map.py` (linker map 파싱) → 없음. `check-binary-analysis`에 유용
  - `generate_unity_test.py` (테스트 생성) → 없음. `do-test-gen`에 유용
  - `precision_calc.py` (정밀도 계산) → 없음. 인사이트 #5와 연동
  - `commit_msg.py` (커밋 메시지) → 없음. `act-commit`에 유용
- **"모든 scripts/는 stdlib-only" 원칙 미수립**
- **스크립트가 적합한 스킬 vs 아닌 스킬 구분 미정의**

**현재 스크립트 활용 현황:**
| 스킬 | scripts/ | 상태 |
|------|----------|------|
| plan-openproject | op.py | 존재 (유일) |
| check-binary-analysis | — | parse_map.py 필요 |
| do-test-gen | — | generate_unity_test.py 필요 |
| act-commit | — | commit_msg.py 가능 |
| (precision-analyzer) | — | precision_calc.py 필요 (스킬 자체 없음) |

---

## 종합 평가

### 잘 된 부분

1. **hooks 기반 규칙 강제**가 체계적 (12개 pre-tool + 4개 post-tool + 3개 lifecycle)
2. **SKILL.md 크기**가 대체로 합리적 (32~290줄, 평균 ~130줄)
3. **description 품질**이 높음 (트리거 키워드 풍부, 양/음 사용 사례 구분)
4. **에이전트 분리**가 잘 됨 (12개 전문화된 에이전트)
5. **references/ 분리 패턴**이 5개 스킬에 이미 적용됨

### 근본적 부재

1. **CLAUDE.md 없음** → 인사이트 #1, #6, #9의 기반 부재
2. **dot 플로우차트 0개** → 복잡한 판단 로직이 산문으로만 기술
3. **Python 스크립트 전략 없음** → 1개만 존재, 확장 계획 없음
4. **번들/페르소나 시스템 없음** → 팀 확장 대비 불가

### 부분 반영의 패턴

"구조는 있으나 명시적 가이드라인이 없는" 상태가 반복됨:
- 토큰 예산: 크기가 합리적이지만 기준이 문서화되지 않음
- 리뷰 카테고리: 존재하지만 독립 실행되지 않음
- 하드웨어 레퍼런스: 존재하지만 그룹 참조 패턴이 없음
- 부트스트랩: inject-context가 있지만 CLAUDE.md가 없음

→ **암묵적으로 올바른 방향이나, 명시적 설계 원칙으로 승격되지 않음**

---

## 반영 액션 우선순위

| 순위 | 인사이트 | 작업 | 영향 | 노력 |
|:----:|---------|------|:----:|:----:|
| 1 | #9 + #6 | CLAUDE.md 생성 + 토큰 예산 가이드라인 문서화 | 높음 | 낮음 |
| 2 | #1 | CLAUDE.md에 워크플로우 순서 강제 규칙 추가 | 높음 | 중간 |
| 3 | #7 | manifest.json에 bundles 섹션 추가 | 중간 | 낮음 |
| 4 | #10 | parse_map.py, generate_unity_test.py 등 생성 | 중간 | 중간 |
| 5 | #8 | ref-hardware에 문서 그룹 참조 패턴 추가 | 중간 | 낮음 |
| 6 | #2 | check-code-review, do-debug에 dot 플로우차트 추가 | 중간 | 중간 |
| 7 | #3 | check-code-review 독립 패스 구조 + git 히스토리 | 중간 | 중간 |
| 8 | #5 | precision-analyzer 스킬 + 계산 스크립트 신규 생성 | 낮음 | 높음 |
| 9 | #4 | semantic duplicate 탐지 모드 추가 | 낮음 | 높음 |
