---
date: 2026-03-12T18:15:00+09:00
git_commit: 2f906b1 (mcuforge main)
branch: main (mcuforge repo)
status: handoff
project_type: embedded
---

# 핸드오프: mcuforge v0.3.0 Claude Skills 릴리스 완료

## 작업 내용

| 작업 | 상태 |
|------|------|
| PR #3: 34개 스킬 패키징 + Rust CLI 코드 | 완료, 머지됨 |
| `download_skills_package()` — GitHub Releases API + flate2/tar | 완료 |
| GitHub Actions CI (test+clippy) + Release (binary+skills) | 완료 |
| E2E 테스트: 로컬 캐시 → GitHub 다운로드 → 설치 검증 | 완료 |
| v0.3.0 릴리스 + `claude-skills-v0.3.0.tar.gz` 첨부 | 완료 |
| 설치 스크립트 (`install.ps1`, `install.sh`) | 완료 |
| CI 수정 (test-threads=1, clippy allow) | 완료, 진행 중 (빌드 대기) |
| `mcuforge init --claude` 대화형 | 미구현 |

## 핵심 참조 문서

- `D:\work\mcuforge\src\core\claude.rs` — 스킬 다운로드/설치/템플릿 렌더링 핵심 (620줄)
- `D:\work\mcuforge\src\commands\claude.rs` — CLI 서브커맨드 (255줄)
- `D:\work\mcuforge\skills\manifest.json` — 34개 스킬 매니페스트

## 최근 변경사항 (mcuforge repo, main 브랜치)

| 커밋 | 내용 |
|------|------|
| `2f906b1` | install.ps1/sh 추가, CI 수정 |
| `060a779` | download_skills_package() 구현, GitHub Actions, v0.3.0 |
| `d377c01` | PR #3 머지: 34개 스킬 + Rust CLI |

### 주요 파일 변경

- `src/core/claude.rs:351-603` — `download_skills_package()`, `resolve_release()`, `download_asset()`, `extract_tar_gz()`, `cache_candidates()`
- `src/utils/paths.rs:56-59` — `skills_cache_dir()` 추가
- `.github/workflows/ci.yml` — 테스트 직렬화 + clippy lint allow
- `.github/workflows/release.yml` — 태그 시 binary + skills 자동 빌드
- `install.ps1` — Windows PowerShell 원라인 설치
- `install.sh` — Linux/macOS bash 원라인 설치

## 학습한 내용

1. **캐시 키 정규화**: `embtool.toml`의 `version = "0.3.0"`은 `v` 없이, 캐시 디렉토리는 `v0.3.0`으로 저장됨. `cache_candidates()`로 양쪽 모두 시도 (`claude.rs:351-360`)
2. **Windows symlink 제약**: Junction은 관리자 필요. 대신 `.source` 포인터 파일 + `manifest.json` 복사로 "latest" alias 구현 (`claude.rs:576-603`)
3. **CI env var race**: `set_var()`/`remove_var()` 테스트가 병렬 실행 시 경합. `--test-threads=1`로 해결
4. **기존 clippy 부채**: `collapsible_if` 12개 + `derivable_impls` 1개 — 기존 코드, CI에서 allow 처리
5. **flate2/tar 크레이트**: Cargo.toml에 이미 선언되어 있었지만 미사용 상태였음. `archive.rs`는 7z 외부 프로세스만 사용

## 생성된 산출물

### mcuforge repo (GitHub: solitasroh/mcuforge)
- `src/core/claude.rs` — 스킬 관리 핵심 로직 (신규)
- `src/commands/claude.rs` — CLI 서브커맨드 (신규)
- `src/core/project.rs` — ClaudeConfig 구조체 추가
- `src/main.rs` — Commands::Claude + --claude 플래그
- `src/commands/new.rs` — step 8: 스킬 자동 설치
- `src/core/mod.rs`, `src/commands/mod.rs` — mod claude 추가
- `src/utils/paths.rs` — skills_cache_dir()
- `skills/` — 34개 스킬 (9 universal, 12 embedded-c, 13 templates)
- `skills/agents/` — 4개 에이전트
- `skills/hooks/` — 5개 훅
- `skills/manifest.json`, `skills/pack.sh`
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- `install.ps1`, `install.sh`
- GitHub Release v0.3.0 + `claude-skills-v0.3.0.tar.gz`

### a2750irm repo
- `docs/plans/2026-03-12-mcuforge-claude-skills.md` — 설계 문서

## 다음 작업 항목

1. **CI 결과 확인**: run #22994333922 완료 대기 (Ubuntu 테스트 통과 확인, Windows 빌드 중)
2. **`mcuforge init --claude`**: `dialoguer`로 GitLab URL, OP URL, 프로젝트 ID 대화형 입력 → `[claude]` 섹션 생성 (`src/commands/init.rs` 수정)
3. **`mcuforge claude update`**: 설치된 버전 vs 최신 릴리스 비교 → 자동 업데이트 (`commands/claude.rs:96-106` TODO 상태)
4. **다른 MCU 프로젝트 적용**: a2700 등에서 `embtool.toml` 작성 + `mcuforge claude install` 실행 → 템플릿 치환 검증
5. **clippy 부채 해소**: 기존 코드의 `collapsible_if` 12개 정리 (별도 PR)
6. **scoop bucket** 또는 **winget manifest**: 팀 배포 자동화 (장기)

## 기타 참고사항

- **타겟 MCU**: MK10DX128VLH7 (Cortex-M4, 256KB Flash, 64KB SRAM)
- **mcuforge 빌드**: `cargo build --release` (Rust 2024 edition)
- **GitHub Release URL**: https://github.com/solitasroh/mcuforge/releases/tag/v0.3.0
- **설치 테스트 디렉토리**: `/tmp/e2e-test/` (임시, 정리 가능)
- **스킬 캐시 위치**: `~/.embtool/claude-skills/v0.3.0/`
- **E2E 검증 결과**: 캐시 삭제 → GitHub 다운로드 → 34 스킬 설치 → 템플릿 `{{placeholder}}` 0개 잔존 → `claude list`/`status` 정상
