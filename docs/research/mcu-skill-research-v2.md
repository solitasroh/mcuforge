# MCU 개발 스킬 연구 v2: "개발자 생산성" 관점으로 확장

## 리서치에서 발견한 핵심 인사이트

### 1. 스킬 생태계의 현재 흐름

**Jeffallan/claude-skills** (6.8K stars)처럼 성공한 스킬셋의 공통점:
- 12개 카테고리, 66개 스킬이지만 각각은 **한 가지 역할에 집중**
- "Feature Forge → Architecture Designer → Test Master → DevOps Engineer" 같은 **워크플로우 체인** 구조
- references/ 폴더에 도메인 지식을 분리하고, SKILL.md는 **워크플로우와 판단 로직**에 집중

**Embedder.com** (AI 펌웨어 플랫폼):
- 핵심 가치: "엔지니어가 레퍼런스 매뉴얼, 레지스터 맵, 에라타를 찾고 종합하는 시간 > 실제 코드 작성 시간"
- 해결: 하드웨어 카탈로그를 RAG처럼 인덱싱하여 주변장치 질문 시 레지스터 정의 + 타이밍 + 에라타를 동시 제공
- **우리에게 영감**: NXP Kinetis 레퍼런스 매뉴얼의 핵심 정보를 references/로 구조화

**업계 트렌드** (RunSafe Security 조사):
- 80%+ 임베디드 개발자가 이미 AI를 코드 생성, 테스트, 문서화에 사용
- 핵심 도전: AI가 생성한 코드는 보안 취약점이 3배 더 많음 (SonarSource)
- 즉, **"AI가 생성한 코드를 검증하는 스킬"**이 점점 더 중요해짐

### 2. 성공적인 스킬의 패턴

리서치에서 발견한 **고가치 스킬 패턴**:

| 패턴 | 설명 | 예시 |
|------|------|------|
| **End-of-session review** | 작업 마무리 시 놓친 것 점검 | unused imports, console.log, edge cases |
| **Structured debugging report** | MCP/도구 로그를 구조화 리포트로 | 어디서 실패했는지, 무엇을 고쳐야 하는지 |
| **Changelog/PR generator** | git log → 구조화된 릴리즈 노트 | Keep a Changelog 형식 |
| **Planning with files** | 기능 설계를 파일로 관리 | 13K+ stars, 가장 인기 있는 스킬 |
| **Domain expert** | 특정 프레임워크/언어 전문가 | NestJS Expert, React Expert |
| **Workflow chain** | 여러 스킬을 순서대로 연결 | Feature → Architecture → Test → Deploy |

---

## 확장된 스킬 후보 목록

### 카테고리 A: 코드 품질 & 안전성 (가장 즉각적인 가치)

#### A1. MCU 코드 리뷰어 (MCU Code Reviewer)
**원래 아이디어의 확장판**

입력: C 소스 파일
출력: 구조화된 리뷰 리포트

**체크리스트 (references/checklist.md로 분리)**:
- [CRITICAL] volatile 누락 (ISR-main 공유 변수)
- [CRITICAL] ISR 내 과도한 처리 (블로킹 호출, printf, malloc)
- [CRITICAL] 비원자적 16/32비트 접근 (8비트 MCU에서)
- [HIGH] 스택 오버플로우 위험 (큰 로컬 배열, 깊은 재귀)
- [HIGH] NULL 포인터 역참조 가능성
- [HIGH] 배열 경계 미검사
- [MEDIUM] 매직 넘버 사용
- [MEDIUM] 함수 길이 초과 (50줄+)
- [MEDIUM] 전역 변수 과다 사용
- [LOW] 네이밍 컨벤션 위반
- [LOW] 주석 부재

**Embedder에서 배운 점**: 단순 린트가 아닌, **"왜 위험한지"와 "어떻게 고쳐야 하는지"**를 MCU 컨텍스트로 설명

**차별화**: MISRA-C 주요 규칙 서브셋을 references/에 포함

---

#### A2. AI 코드 검증기 (AI-Generated Code Verifier)
**새로운 아이디어 — 업계 트렌드 반영**

입력: AI가 생성한 C 코드 (또는 기존 코드)
출력: 임베디드 적합성 검증 리포트

**고유 체크**:
- WCET(Worst Case Execution Time) 추정 (ISR 데드라인 충족?)
- 스택 사용량 정적 분석
- 불필요한 동적 할당 탐지 (malloc/free 사용)
- 하드웨어 레지스터 접근 패턴 검증
- DMA 버퍼 정렬/캐시 일관성
- 인터럽트 우선순위 역전 가능성
- 전력 소모 영향 (불필요한 폴링, sleep 미사용)

**가치**: AI 코딩이 보편화되면서 "AI가 쓴 펌웨어 코드가 진짜 하드웨어에서 안전한가?"를 검증

---

### 카테고리 B: 정보 종합 & 분석 (시간 절약)

#### B1. 링커 맵 분석기 (Linker Map Analyzer)
**원래 아이디어 유지, 스크립트 강화**

입력: .map 파일 (ARM GCC)
출력: 메모리 사용량 대시보드

**핵심 기능**:
- Flash/RAM 사용률 (% 및 잔여 바이트)
- 모듈별 점유율 Top 10
- 가장 큰 함수/변수 Top 10  
- 미사용 섹션 탐지
- **비교 모드**: 두 .map 파일 diff → 어디서 증가/감소했는지

**구현**: Python 스크립트 (scripts/parse_map.py)
**출력 형식**: 마크다운 테이블 + Mermaid 파이차트

---

#### B2. 데이터시트 네비게이터 (Datasheet Navigator)
**Embedder의 "Hardware Catalog" 아이디어를 스킬로 구현**

입력: MCU 데이터시트 PDF 또는 구조화된 레지스터 정보
출력: 필요한 정보를 빠르게 찾아서 정리

**기능**:
- "ADC 모듈의 샘플링 타임 설정 레지스터 알려줘" → 관련 레지스터 + 비트필드 + 설정 예시
- "SPI 클럭 최대 속도와 제약사항" → 전기적 특성 + 타이밍 다이어그램 설명
- 에라타 자동 참조 (references/errata-k22f.md)

**현실적 접근**: PDF 직접 파싱보다는, 주요 주변장치별 레지스터 맵을 미리 references/에 정리
- references/k22f-adc.md
- references/k22f-spi.md
- references/k22f-uart.md
- references/k10d-adc.md (16-bit ADC 특화)

**가치**: 매번 1000+ 페이지 데이터시트를 뒤지는 시간 절약

---

#### B3. 회로/ADC 정밀도 분석기 (Precision Analyzer)
**원래 아이디어, 입력 형식 구체화**

입력: 파라미터 표 (텍스트 or CSV)
```
adc_bits: 16
vref: 3.3
divider_r1: 100k
divider_r2: 10k
r1_tolerance: 1%
r2_tolerance: 1%
temperature_range: -40 to 85
tempco_r1: 50ppm
```

출력: 정밀도 분석 리포트
- 이론적 분해능 (LSB 크기)
- 분압기 오차 전파
- 온도 드리프트 영향
- DNL/INL 영향 추정
- 총 시스템 오차 (RSS)
- 측정 가능 범위와 정밀도

**가치**: Accura 2750IRM 같은 정밀 측정 장비 개발에서 반복되는 계산 자동화

---

### 카테고리 C: 코드/문서 생성 (생산성)

#### C1. HAL 드라이버 생성기 (HAL Driver Generator)
**원래 아이디어, 입력 형식 재설계**

입력: 구조화된 주변장치 명세 (데이터시트 PDF보다는 정리된 텍스트)
```yaml
peripheral: ADC
mcu: K22F
channels: 16
resolution: [8, 10, 12, 16]
features: [DMA, hardware_trigger, compare, averaging]
```

출력: HAL 드라이버 코드 (.c/.h)
- 레지스터 매크로/구조체
- Init/DeInit/Start/Stop/Read API
- 인터럽트 핸들러 스켈레톤
- 사용 예시 코드
- 단위 테스트용 Mock 인터페이스

**Rootech 코딩 스타일 적용**: references/coding-standard.md

---

#### C2. 펌웨어 테스트 생성기 (Firmware Test Generator)
**원래 아이디어 + 업계 인사이트 반영**

입력: C 헤더 파일 (.h) — 모듈의 공개 API
출력: Unity 테스트 코드 + 테스트 매트릭스

**생성 전략**:
1. 함수 시그니처 분석 → 파라미터별 경계값, 정상값, 에러값 도출
2. 포인터 파라미터 → NULL 테스트 자동 포함
3. 반환 타입이 Status_t → 모든 에러 코드 커버
4. 하드웨어 의존 함수 → Mock 인터페이스 자동 생성

**테스트 매트릭스 출력 예시**:
| 함수 | 테스트 케이스 | 입력 | 기대 결과 | 분류 |
|------|------------|------|----------|------|
| ADC_Init | NULL config | NULL | STATUS_ERROR | 비정상 |
| ADC_Init | Valid config | valid_cfg | STATUS_OK | 정상 |
| ADC_Read | Channel OOB | ch=99 | STATUS_INVALID | 경계 |

**추가 가치**: 테스트 리스트를 xlsx로도 출력 (QA팀 공유용)

---

#### C3. 기술 문서 생성기 (Technical Doc Generator)
**iTask 분석 경험 + manual-writer 스킬 확장**

입력: C 소스 코드 + 선택적 설계 문서
출력: 기술 문서 (docx/md/pdf)

**문서 유형별 모드**:
1. **모듈 API 레퍼런스**: 함수 목록, 파라미터, 반환값, 사용 예시
2. **아키텍처 문서**: 모듈 관계도, 호출 그래프, 데이터 흐름 (Mermaid)
3. **동작 분석서**: iTask 스타일 — 타이밍, 인터럽트 흐름, 상태 머신 분석
4. **코드 변경 이력**: git log 기반 변경사항 요약

**차별화**: Rootech 문서 스타일 (맑은 고딕, Fig/Table 번호 체계) 자동 적용

---

### 카테고리 D: 개발 워크플로우 (팀 생산성)

#### D1. 커밋/MR 어시스턴트 (Commit & MR Assistant)
**changelog-gen 스킬 패턴 + GitLab MR 템플릿 경험 활용**

**기능 1: 커밋 메시지 생성**
입력: git diff
출력: Conventional Commits 형식 메시지
```
feat(adc): add 4x oversampling support for K22F

- Implement ADC_SetOversampling() API
- Add hardware averaging configuration
- Update FIR filter chain for new sample rate

Refs: OP#1234
```

**기능 2: MR 설명 생성**
입력: branch의 커밋 히스토리 + diff 요약
출력: GitLab MR 템플릿 기반 설명문

**기능 3: 릴리즈 노트**
입력: 태그 간 커밋 로그
출력: CHANGELOG.md 엔트리

---

#### D2. 빌드 에러 디버거 (Build Error Debugger)
**Embedder의 RCA(Root Cause Analysis) 아이디어를 스킬로**

입력: 빌드 로그 (컴파일 에러, 링커 에러)
출력: 구조화된 에러 분석 + 해결 방안

**특화 영역**:
- ARM GCC 에러 메시지 해석
- 링커 스크립트 관련 에러 (section overflow, undefined reference)
- NXP SDK 관련 흔한 빌드 이슈
- CMake 설정 문제 진단

**references/에 포함할 것**:
- common-build-errors.md (흔한 에러 + 해결법 DB)
- linker-script-guide.md
- nxp-sdk-pitfalls.md

---

#### D3. 디버깅 전략 가이드 (Debugging Strategist)
**debugging-wizard 스킬을 MCU 특화로**

입력: 증상 설명 (자연어)
출력: 디버깅 전략 + 체크리스트

**예시 시나리오**:
- "ADC 값이 예상보다 0.5mV 높게 나와요" → Ground potential 확인, 레퍼런스 전압 측정, ADC 캘리브레이션 체크, ...
- "간헐적으로 HardFault 발생" → 스택 오버플로우 체크, MPU 설정 확인, 정렬 문제, ...
- "Modbus 통신이 30초마다 끊김" → 타이머 오버플로우, TCP keepalive, 워치독, ...

**references/**:
- hardfault-diagnosis.md
- communication-debug.md
- measurement-debug.md (Accura 특화)

---

### 카테고리 E: 프로젝트 관리 연동 (팀 확장 대비)

#### E1. OpenProject 워크패키지 어시스턴트
**기존 op.py CLI 경험을 스킬로 확장**

입력: 자연어 작업 설명
출력: OpenProject WP 생성/업데이트 명령 또는 직접 API 호출

**기능**:
- "ADC 오버샘플링 기능 추가해야 해" → WP 생성 (제목, 설명, 담당자, 우선순위)
- "이 코드 변경이 어떤 WP에 연결되지?" → git branch명/커밋에서 WP 번호 추출
- 진행률 자동 업데이트 제안

---

## 스킬 간 관계도 (워크플로우 체인)

```
새 기능 개발 워크플로우:
  B2.데이터시트네비게이터 → C1.드라이버생성 → A1.코드리뷰 → C2.테스트생성 → D1.커밋어시스턴트

버그 수정 워크플로우:
  D3.디버깅전략 → A1.코드리뷰 → C2.테스트생성 → D1.커밋어시스턴트

릴리즈 워크플로우:
  B1.링커맵분석 → A2.AI코드검증 → C3.기술문서생성 → D1.릴리즈노트

정밀 측정 개발:
  B3.정밀도분석 → C1.드라이버생성 → A1.코드리뷰 → C3.기술문서생성
```

---

## 우선순위 재평가 (기존 6개 → 12개 확장)

### Tier 1: 즉시 구현, 매일 사용 (1~2주)
| 스킬 | 구현 난이도 | 일일 사용 빈도 | 이유 |
|------|-----------|-------------|------|
| A1. MCU 코드 리뷰어 | ★★☆ | 높음 | 순수 컨텍스트 분석, 도구 불필요 |
| D1. 커밋/MR 어시스턴트 | ★★☆ | 높음 | git diff → 메시지, 반복적 |
| B1. 링커 맵 분석기 | ★★☆ | 중간 | Python 스크립트, 확실한 결과 |

### Tier 2: 높은 가치, 중간 노력 (2~4주)
| 스킬 | 구현 난이도 | 가치 | 이유 |
|------|-----------|------|------|
| C2. 테스트 생성기 | ★★★ | 팀 확장 대비 핵심 | 헤더 파싱 필요 |
| D2. 빌드 에러 디버거 | ★★☆ | 신입 온보딩 필수 | references DB 구축이 관건 |
| D3. 디버깅 전략 가이드 | ★★☆ | 지식 전수 | 도메인 지식 정리가 핵심 |
| A2. AI 코드 검증기 | ★★★ | 미래 대비 | A1 확장 가능 |

### Tier 3: 높은 가치, 높은 노력 (1~2개월)
| 스킬 | 구현 난이도 | 가치 | 이유 |
|------|-----------|------|------|
| C3. 기술 문서 생성기 | ★★★ | 지식 공유 | manual-writer 확장 |
| B2. 데이터시트 네비게이터 | ★★★★ | 시간 절약 큼 | references 구축 노력 |
| C1. HAL 드라이버 생성기 | ★★★★ | 재사용성 높음 | 템플릿 설계 중요 |

### Tier 4: 니치하지만 특화 가치
| 스킬 | 구현 난이도 | 가치 | 이유 |
|------|-----------|------|------|
| B3. 정밀도 분석기 | ★★★ | Accura 특화 | 수학적 모델링 |
| E1. OpenProject 연동 | ★★★ | 팀 인프라 | API 연동 필요 |

---

## 핵심 설계 원칙 (리서치에서 도출)

### 1. "한 가지 일을 잘 하는" 스킬
- 하나의 SKILL.md가 너무 많은 것을 하려 하면 트리거 정확도 떨어짐
- A1(코드 리뷰)와 A2(AI 검증)를 분리하는 것이 맞음

### 2. Progressive Disclosure 활용
```
SKILL.md (< 500줄) — 워크플로우, 판단 로직
  └── references/ — 도메인 지식 (필요 시 로드)
        ├── checklist.md
        ├── misra-c-subset.md
        ├── k22f-errata.md
        └── ...
  └── scripts/ — 자동화 (실행)
        ├── parse_map.py
        ├── generate_test.py
        └── ...
```

### 3. 양쪽 환경 호환
- SKILL.md + references/: claude.ai와 Claude Code 모두에서 동작
- scripts/: 두 환경 모두 bash/Python 실행 가능
- Claude Code 추가: hooks/ (자동 트리거), agents/ (sub-agent 위임)

### 4. 팀 확장 고려 (10명 → 50명)
- 코딩 스타일 일관성: A1 + 코딩 스탠다드 references
- 신입 온보딩: D2(빌드에러) + D3(디버깅) + B2(데이터시트)
- 품질 게이트: A1 + C2(테스트) + B1(메모리)

### 5. 점진적 구축
- Tier 1부터 시작 → 실제 사용하면서 피드백 → Tier 2로 확장
- references/는 계속 축적되는 자산 (한 번 정리하면 여러 스킬이 공유)
