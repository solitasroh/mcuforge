# Claude Code 핸드오프 가이드

## 1. 프로젝트 구조 세팅

```bash
# 프로젝트 디렉토리 생성
mkdir -p ~/projects/mcu-dev-skills
cd ~/projects/mcu-dev-skills

# 리서치 산출물 디렉토리
mkdir -p docs/research

# 다운로드한 파일들을 여기에 넣기
# (claude.ai에서 다운로드한 7개 .md 파일)
mv ~/Downloads/mcu-skill-*.md docs/research/
mv ~/Downloads/insight-gap-analysis.md docs/research/
```

## 2. CLAUDE.md 작성 (아래 내용을 프로젝트 루트에 생성)

```bash
cat > CLAUDE.md << 'HEREDOC'
# MCU/WPF 개발 스킬 프로젝트

## 프로젝트 목적
NXP Kinetis (K10D/K22F/K64), STM32 H5, i.MX28/6ULL, WPF(Prism) 개발에 사용할
Claude Code 스킬 세트를 설계하고 구현하는 프로젝트.

## 현재 상태
리서치 및 설계 단계 완료. 구현 시작 전.

### 완료된 리서치 (docs/research/)
- mcu-skill-research.md: 초기 6개 스킬 후보 분석
- mcu-skill-research-v2.md: 12개로 확장, 실현 가능성/가치 평가
- mcu-skill-research-v3.md: 인기 스킬 생태계 분석 (Superpowers, Karpathy, NeoLab 등)
- mcu-skill-bigpicture.md: 24개 스킬 전체 맵 (4 플랫폼 × 4 유형)
- mcu-skill-review-report.md: 비판적 재검토 (24→15개 정리)
- mcu-skill-final-v4.md: 최종 15개 스킬 목록 + 워크플로우 + shared-references 구조
- insight-gap-analysis.md: 리서치 인사이트 10개 누락 분석 + 반영 방향 ★중요★

### 핵심 결정 사항
- Karpathy 4원칙은 CLAUDE.md에 통합 (스킬이 아닌 프로젝트 원칙)
- 15개 독립 스킬: A5 + B1/B2/B3/B4/B6/B7 + C1/C2/C3/C4/C6/C7 + D1/D2/D3
- shared-references/ 구조로 여러 스킬이 지식 자산 공유
- 플랫폼 차이는 references/에서 분기 (스킬 내부가 아님)

### 미반영 인사이트 10개 (insight-gap-analysis.md 참조)
반드시 구현 시 반영해야 할 항목:
1. 토큰 예산: SKILL.md 본문 3,000토큰 이내, CLAUDE.md 1,000토큰 이내
2. stdlib-only 스크립트 원칙: pip 의존성 금지
3. 역할별 번들 전략: 펌웨어/WPF/리눅스/측정장비/신입 온보딩
4. 강제 워크플로우: 설계→테스트→구현→리뷰 순서
5. GraphViz dot 표기: 복잡한 판단 로직에 적용
6. 문서 그룹 참조: D2가 관련 references 3~4개를 동시 참조
7. NeoLab 독립 리뷰어: B1에서 관점별 독립 패스
8. Scientific Skills 계산 스크립트: B6에 precision_calc.py
9. Semantic duplicate: B1 확장 (후순위)
10. Superpowers 강제 워크플로우 메커니즘

## 다음 작업
1. Phase 1: CLAUDE.md 확정 + A5(coding-standard) + B1(mcu-code-reviewer) 초안
2. Phase 2: B4(linker-map-analyzer) + C4(commit-assistant) + D1(debug-strategist)
3. Phase 3: 나머지 스킬들

## 타겟 플랫폼
- Bare Metal: NXP Kinetis K10D/K22F/K64 (Cortex-M4), STM32 H5 (Cortex-M33)
- Embedded Linux: i.MX28 (ARM926), i.MX6ULL (Cortex-A7)
- Desktop: WPF / Prism / .NET Framework

## 컨텍스트
- 회사: Rootech/Solitas, 산업용 측정/보호 장비
- 제품: Accura 2700/2750 시리즈, AccuraLogic (WPF PLC 에디터)
- 팀: 10명 → 50명 확장 중
- 도구: GitLab, OpenProject, SonarQube, ARM GCC, Visual Studio
HEREDOC
```

## 3. Claude Code에서 작업 시작

```bash
cd ~/projects/mcu-dev-skills
claude

# 첫 프롬프트:
# "docs/research/ 폴더의 리서치 문서들을 읽고,
#  특히 insight-gap-analysis.md의 미반영 인사이트 10개를 확인해.
#  그리고 mcu-skill-final-v4.md를 기반으로
#  Phase 1 (CLAUDE.md + A5 + B1) 구현을 시작하자."
```

## 4. 핵심 포인트

### 왜 이 방식인가
- **CLAUDE.md**가 전체 프로젝트 컨텍스트를 제공 → Claude Code가 매 세션 시작 시 읽음
- **docs/research/**에 리서치 산출물 → 필요할 때 참조
- 대화 히스토리를 통째로 옮기는 것보다 **의사결정 결과를 문서로** 옮기는 게 효과적

### 대화 맥락 중 문서에 안 담긴 것
- 스킬 사용 환경: claude.ai + Claude Code **둘 다** 고려
- references 축적 로드맵: 즉시→1개월→3개월→6개월 단계별
- "주 1회 이상 호출할 것인가?" 판단 기준
- Embedder.com의 하드웨어 카탈로그 아이디어가 D2 설계에 영감
- WPF 바인딩 누수 5가지 구체적 패턴 (PropertyDescriptor, 컬렉션, x:Name, 이벤트, 싱글톤)
- iTask 협력형 스케줄러가 순수 Bare Metal도 RTOS도 아닌 중간 영역이라는 점
