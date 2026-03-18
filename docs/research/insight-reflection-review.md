# 리서치 인사이트 반영 상태 검토 보고서

> 검토일: 2026-03-18
> 대상: `insight-gap-analysis.md`의 10개 인사이트 → 실제 skills/ 구현체 대조
> 범위: 29 skills, 12 agents, 18 hooks, manifest.json, settings.json.tmpl
> **확장 검토**: Embedded Linux App / WPF(.NET) App 도메인 포함 분석 (2026-03-18 추가)

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

### 멀티 도메인 확장 시 인사이트 영향도

> Embedded Linux App, WPF(.NET) App 도메인 추가 시 각 인사이트의 긴급도 재평가

| # | 인사이트 | Embedded Linux 영향 | WPF/.NET 영향 | 확장 긴급도 |
|---|---------|-------------------|--------------|:---------:|
| 1 | 강제 워크플로우 | **HIGH** — 크로스컴파일→타겟배포→원격디버깅 순서 강제 | **HIGH** — MVVM 설계→ViewModel→View 순서 강제 | ★★★ |
| 2 | dot 플로우차트 | MEDIUM — Yocto 빌드 파이프라인 복잡 | LOW — MVVM 패턴 선형적 | ★★ |
| 3 | 독립 리뷰어 | **HIGH** — 커널/유저 경계, IPC, 보안, fd 누수, 크로스컴파일 호환성 각각 독립 리뷰 | **HIGH** — MVVM 준수, 바인딩 안전성, 메모리 누수, UI 스레드 | ★★★ |
| 4 | 중복 탐지 | **HIGH** — 드라이버/HAL 유사 패턴 산재 | MEDIUM — XAML Style/Template 중복 | ★★ |
| 5 | 계산 스킬 | MEDIUM — 성능 프로파일링 계산 | LOW — UI 도메인, 계산 불필요 | ★ |
| 6 | 토큰 예산 | **CRITICAL** — 3개 도메인 동시 로드 시 예산 폭발 (~5,500줄) | **CRITICAL** | ★★★ |
| 7 | 역할별 번들 | **CRITICAL** — Linux 개발자에게 MCU hooks 불필요 | **CRITICAL** — WPF 개발자에게 float-suffix hook 불필요 | ★★★ |
| 8 | 문서 그룹 참조 | **HIGH** — DTS + 커널 드라이버 + sysfs + udev 동시 참조 | MEDIUM — MSDN + NuGet + XAML 참조 | ★★ |
| 9 | 부트스트랩 제한 | **CRITICAL** — CLAUDE.md가 도메인별로 달라야 함 | **CRITICAL** | ★★★ |
| 10 | stdlib 스크립트 | **HIGH** — bitbake 로그, DTB 분석, rootfs 사이즈 스크립트 | MEDIUM — .csproj 분석 스크립트 | ★★ |

**핵심 발견: #6 토큰 예산, #7 번들, #9 부트스트랩이 멀티 도메인 확장의 전제 조건 (★★★)**

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

**멀티 도메인 확장 영향 (★★★):**
- **Embedded Linux**: 크로스컴파일→scp 타겟배포→원격 GDB→로그 수집 순서 강제 필요. "빌드 없이 배포 금지" hook 필수
- **WPF**: MVVM 설계(ViewModel 정의)→바인딩 테스트→View XAML 작성 순서 강제. "ViewModel 없이 코드비하인드에 로직 금지"
- **공통**: 도메인별 워크플로우 DAG가 다르므로 워크플로우 정의를 도메인별 SKILL.md로 분리 필요

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

**멀티 도메인 확장 영향 (★★):**
- **Embedded Linux**: Yocto 빌드 파이프라인(bitbake→do_fetch→do_compile→do_install→image), systemd 서비스 의존성 그래프 → dot 표현이 매우 유리
- **WPF**: MVVM 바인딩 흐름(View↔ViewModel↔Model), DI 컨테이너 해석 순서 → 상대적으로 선형이라 dot 필요성 낮음

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

**멀티 도메인 확장 영향 (★★★):**
- **Embedded Linux**: 최소 5종 독립 리뷰 필요 — ① 커널/유저 경계 안전성, ② IPC 프로토콜 일관성(D-Bus/socket), ③ 보안(SELinux, 권한 상승), ④ 리소스 누수(fd, mmap), ⑤ 크로스컴파일 호환성
- **WPF**: 최소 4종 — ① MVVM 패턴 준수, ② 바인딩 안전성(INotifyPropertyChanged, ICommand), ③ 메모리 누수(이벤트 핸들러 해제), ④ UI 스레드 안전성(Dispatcher)
- **공통**: git 히스토리 기반 리뷰는 모든 도메인에서 유용 → `universal/` 스킬로 승격 검토

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

**멀티 도메인 확장 영향 (★★):**
- **Embedded Linux**: HAL 래퍼 중복(동일 센서의 sysfs/i2c-dev/ioctl 구현이 제품별 산재) → HIGH
- **WPF**: XAML Style/Template 중복, 유사 ViewModel 중복 → MEDIUM
- **공통**: 의도 기반 중복 탐지는 도메인 무관 → `universal/` 스킬로 제작 적합

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

**멀티 도메인 확장 영향 (★):**
- **Embedded Linux**: 성능 프로파일링 계산(CPU affinity, IRQ affinity, latency histogram), 전력 소비 분석 → MEDIUM
- **WPF**: UI 도메인이라 공학 계산 불필요 → LOW
- 이 인사이트는 **embedded-c 전용 유지** 가능 (universal 승격 불필요)

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

**멀티 도메인 확장 영향 (★★★ CRITICAL):**
- 현재 단일 도메인(embedded-c) 14 skills × ~130줄 = ~1,820줄 → 관리 가능
- 3개 도메인 시: (6 universal + 14 embedded-c + ~12 embedded-linux + ~10 wpf) × ~130줄 = **~5,500줄**
- **모두 동시 로드하면 토큰 예산 초과** → 도메인별 선택적 로딩이 필수
- `ClaudeSkillsConfig.overrides` (HashMap<String, toml::Value>) 확장 가능하나 공식 가이드라인 선행 필요

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

**멀티 도메인 확장 영향 (★★★ CRITICAL):**
3개 도메인 추가 시 manifest.json은 **~50+ skills**로 확장. 번들 없이는:
- **MCU 펌웨어 개발자**가 WPF hooks(`check-mvvm-pattern.sh`)에 blocking 당함
- **WPF 개발자**가 `verify-float-suffix` hook에 불필요하게 차단됨
- Hooks가 **도메인 무관 전부 로드**되는 `settings.json.tmpl` 구조의 근본적 한계

필요한 번들 구조 예시:
```json
"bundles": {
  "firmware-dev":       ["universal", "embedded-c", "templates/ref-hardware"],
  "embedded-linux-dev": ["universal", "embedded-linux", "templates/ref-linux-bsp"],
  "wpf-dev":            ["universal", "wpf-dotnet", "templates/ref-wpf-patterns"],
  "fullstack-embedded": ["universal", "embedded-c", "embedded-linux"],
  "onboarding":         ["universal", "ref-karpathy", "ref-coding-rules"]
}
```

Rust 코드 영향:
- `src/core/project.rs:40-45` — `ClaudeSkillsConfig`에 `bundle: Option<String>` 추가
- `src/core/claude.rs:146-164` — `install_skills()`가 번들 기반 도메인 선택
- `src/core/project.rs:104` — `project_type` 미사용 필드 → 번들 자동 선택에 활용

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

**멀티 도메인 확장 영향 (★★):**
- **Embedded Linux**: Device Tree(.dts) + 커널 드라이버 + sysfs + udev rules를 동시 참조해야 하는 시나리오 빈번 → 문서 그룹 참조 패턴 필수
- **WPF**: MSDN + NuGet 패키지 + XAML 패턴 레퍼런스 동시 참조 → `ref-wpf-patterns/references/` 하위에 다중 문서 구조화
- 현재 `ref-hardware`의 2파일 패턴을 **일반화**하여 `ref-linux-bsp/references/`, `ref-wpf-patterns/references/`에도 적용

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

**멀티 도메인 확장 영향 (★★★ CRITICAL):**
- CLAUDE.md가 **도메인별로 달라져야** 함:
  - Embedded C: "MCU: MK10DN512, Core: Cortex-M4, Stack: 4KB, float F suffix 필수"
  - Embedded Linux: "Target: i.MX8MP, Yocto: kirkstone, toolchain: aarch64-poky-linux"
  - WPF: ".NET 8, MVVM Toolkit, C# 12, nullable reference types 활성"
- `inject-context.sh` (SessionStart hook)이 **프로젝트 타입에 따라 다른 컨텍스트** 주입 필요
- `src/core/claude.rs`에서 CLAUDE.md의 `<!-- BEGIN MCUFORGE SKILLS -->` 섹션 관리 로직이 있으나 **도메인 인식 없음**

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

**멀티 도메인 확장 영향 (★★):**
- **Embedded Linux**: `parse_bitbake_log.py` (Yocto 빌드 로그 파싱), `analyze_dtb.py` (Device Tree 분석), `check_rootfs_size.py` (rootfs 사이즈 검사) 등
- **WPF**: `parse_csproj.py` (.csproj 의존성 분석), `check_binding_errors.py` (바인딩 에러 분석) 등
- 현재 1개 → 3개 도메인 시 **최소 10개+ 스크립트** → stdlib-only 원칙 + 관리 전략 선행 필수

---

## 멀티 도메인 아키텍처 확장성 분석

### 현재 아키텍처: 단일 도메인(Embedded C) 하드코딩

```
skills/
├── universal/        ← 도메인 무관 (6 skills)
├── embedded-c/       ← MCU 펌웨어 전용 (14 skills) ◀ 유일한 도메인
├── templates/        ← 프로젝트별 커스터마이징 (9 skills)
├── agents/           ← AI 에이전트 (12)
└── hooks/            ← 코딩 규칙 강제 (18) — 전부 embedded-c 전용
```

### 확장 목표: 3개 도메인

```
skills/
├── universal/        ← 도메인 무관 (공통)
├── embedded-c/       ← MCU 펌웨어 (Cortex-M, RTOS 없음)
├── embedded-linux/   ← Embedded Linux App (Yocto/Buildroot, C/C++, IPC) ◀ 신규
├── wpf-dotnet/       ← WPF Desktop App (C#, MVVM, .NET 8+)             ◀ 신규
├── templates/        ← 프로젝트별 (도메인 템플릿 변수 추가)
├── agents/           ← 도메인별 + 공통 에이전트
└── hooks/            ← 도메인별 + 공통 hooks
```

### Rust 코드의 도메인 하드코딩 현황

| 파일 | 위치 | 하드코딩 내용 | 확장 방법 |
|------|------|-------------|----------|
| `src/core/project.rs:40-44` | `ClaudeSkillsConfig` | `embedded_c: Option<bool>` 필드만 | `domains: HashMap<String, bool>` |
| `src/core/claude.rs:156-164` | `install_skills()` | `if skills_cfg.embedded_c` 단일 분기 | manifest에서 도메인 동적 로드 루프 |
| `src/core/project.rs:104-109` | `ProjectMeta` | `project_type` 필드 **미사용** | 도메인 자동 선택에 활용 |
| `skills/pack.sh` | tar 명령어 | `embedded-c/` 하드코딩 | manifest 카테고리 키 동적 패키징 |
| `skills/settings.json.tmpl` | hooks 전체 | 모든 hooks 무조건 설치 | 도메인별 hook 필터링 |

### 확장 설계안

```
embtool.toml (프로젝트 설정)
┌────────────────────────────────┐
│ [project]                      │
│ type = "embedded-linux"  ← KEY │
│                                │
│ [claude.skills]                │
│ universal = true               │
│ embedded_linux = true          │
│ embedded_c = false             │
│ wpf_dotnet = false             │
└────────┬───────────────────────┘
         │
         ▼
install_skills() in claude.rs
┌────────────────────────────────┐
│ 1. manifest.json 읽기          │
│ 2. project_type → 기본 번들 결정│
│ 3. skills_cfg → 오버라이드 적용 │
│ 4. 선택된 도메인만 설치         │
│ 5. hooks도 도메인 필터링        │
└────────────────────────────────┘
```

### 도메인별 필요 스킬 맵핑

#### Embedded Linux App (예상 ~12 skills)
```
skills/embedded-linux/
├── do-build-yocto/          ← bitbake 빌드 + SDK 크로스 컴파일
├── do-deploy/               ← scp/rsync 타겟 배포
├── do-scaffold-linux/       ← Linux app 스캐폴딩 (systemd service 포함)
├── do-remote-debug/         ← GDB remote, strace, perf, coredump
├── check-code-review-linux/ ← IPC 안전성, fd 누수, 권한, SELinux
├── check-rootfs-size/       ← rootfs 파티션 사이즈 게이트
├── verify-posix-safety/     ← POSIX API 안전 사용 (signal safety)
├── verify-ipc-protocol/     ← D-Bus/socket/shm 프로토콜 일관성
├── ref-device-tree/         ← DTS 레퍼런스 + 바인딩 문서
├── ref-linux-bsp/           ← BSP 레이어, 커널 모듈, sysfs
├── ref-coding-rules-cpp/    ← Modern C++17/20 코딩 규칙
└── plan-systemd-service/    ← systemd 서비스 설계 + 의존성 분석
```

#### WPF/.NET (예상 ~10 skills)
```
skills/wpf-dotnet/
├── do-build-dotnet/              ← dotnet build/publish
├── do-scaffold-wpf/              ← MVVM 스캐폴딩 (ViewModel+View+Model)
├── do-lint-roslyn/               ← Roslyn analyzer + .editorconfig
├── check-code-review-wpf/       ← MVVM 준수, 바인딩, 메모리 누수
├── check-test-coverage-dotnet/   ← xUnit + Moq 커버리지
├── verify-mvvm-pattern/          ← ViewModel에 View 참조 금지, ICommand
├── verify-nullable/              ← nullable reference type 위반
├── verify-naming-dotnet/         ← C# 네이밍 (PascalCase, _camelCase)
├── ref-wpf-patterns/             ← MVVM, DI, IValueConverter 패턴
└── ref-dotnet-api/               ← .NET 8 API, NuGet 가이드
```

### Hooks 도메인화 방안

현재 `settings.json.tmpl`은 모든 hooks 무조건 설치. 2가지 방안:

**방안 A: domain 필드 추가 (최소 변경)**
```json
{
  "matcher": "Write|Edit",
  "domain": "embedded-c",
  "hooks": [{"command": "check-float-suffix.sh", "blocking": true}]
}
```

**방안 B: hooks 디렉토리 도메인 분리 (깔끔)**
```
skills/hooks/
├── common/          ← 모든 도메인 (function-length, nesting-depth 등)
├── embedded-c/      ← MCU 전용 (float-suffix, stdint, driver-boundary)
├── embedded-linux/  ← Linux 전용 (fd-leak-check, posix-safety)
└── wpf-dotnet/      ← WPF 전용 (mvvm-pattern, nullable-check)
```

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

### 멀티 도메인 확장 시 추가 과제

1. **도메인 하드코딩 제거**: `ClaudeSkillsConfig`와 `install_skills()`에 embedded-c만 하드코딩 → 동적화 필수
2. **Hooks 도메인 격리**: WPF 개발자가 float-suffix hook에 차단되는 문제 해결
3. **토큰 예산 폭발**: 3개 도메인 동시 로드 시 ~5,500줄 → 선택적 로딩 필수
4. **CLAUDE.md 도메인 인식**: 프로젝트 타입에 따라 다른 핵심 원칙 주입

---

## 반영 액션 우선순위 (멀티 도메인 확장 고려)

### Phase 0: 아키텍처 기반 (멀티 도메인 전제 조건)

| 순위 | 인사이트 | 작업 | 수정 파일 | 노력 |
|:----:|---------|------|----------|:----:|
| 0-A | #7 번들 + 도메인 인식 | `ClaudeSkillsConfig` 동적화, `install_skills()` 루프화, `project_type` 활용 | `project.rs`, `claude.rs`, `manifest.json` | **높음** |
| 0-B | #9 + #6 CLAUDE.md 템플릿화 | CLAUDE.md.tmpl 도메인별 섹션 + 토큰 예산 가이드라인 | `templates/`, `claude.rs` | 중간 |
| 0-C | Hooks 도메인 필터링 | `settings.json.tmpl` domain 필드 또는 hooks/ 도메인 분리 | `hooks/`, `settings.json.tmpl` | 중간 |

### Phase 1: 공통 인프라 (도메인 무관)

| 순위 | 인사이트 | 작업 | 노력 |
|:----:|---------|------|:----:|
| 1-A | #1 | 도메인별 워크플로우 DAG → SKILL.md + CLAUDE.md hard rule | 중간 |
| 1-B | #10 | stdlib-only 원칙 문서화 + 도메인별 필수 스크립트 목록 | 낮음 |
| 1-C | #8 | ref-* 스킬에 "관련 문서 동시 참조" 패턴 추가 | 낮음 |

### Phase 2: 스킬 품질 강화 (기존 embedded-c 개선)

| 순위 | 인사이트 | 작업 | 노력 |
|:----:|---------|------|:----:|
| 2-A | #2 | check-code-review, do-debug에 dot 플로우차트 추가 | 중간 |
| 2-B | #3 | check-code-review 독립 패스 구조 + git 히스토리 리뷰 | 중간 |
| 2-C | #4 | universal에 semantic-duplicate-finder 스킬 추가 | 높음 |
| 2-D | #5 | embedded-c에 precision-analyzer 스킬 + 계산 스크립트 | 높음 |

### Phase 3: 신규 도메인 스킬 생성

| 순위 | 작업 | 예상 스킬 수 | 노력 |
|:----:|------|:----------:|:----:|
| 3-A | `skills/embedded-linux/` 도메인 생성 | ~12 | 높음 |
| 3-B | `skills/wpf-dotnet/` 도메인 생성 | ~10 | 높음 |
| 3-C | 도메인별 agents 추가 | ~6 | 중간 |
| 3-D | 도메인별 hooks 추가 | ~8 | 중간 |
