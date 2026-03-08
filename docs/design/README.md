# embtool 설계 문서

## 설계 원칙

- **단일 책임**: 각 모듈은 하나의 역할만 수행
- **테스트 가능**: 모든 core 모듈은 유닛 테스트 가능하도록 설계
- **의존성 방향**: `commands → core → utils` (역방향 금지)
- **에러 처리**: `anyhow::Result` 통일, 사용자 친화적 에러 메시지

## 문서 구조

| 파일 | 내용 |
|------|------|
| [phase0-infra.md](phase0-infra.md) | Phase 0: 기반 인프라 (경로, 설정, setup) |
| [phase1-toolchain.md](phase1-toolchain.md) | Phase 1: 툴체인 관리 (install/list/use/remove) |
| [phase2-project.md](phase2-project.md) | Phase 2: 프로젝트 생성 (MCU DB, 템플릿) |
| [phase3-build.md](phase3-build.md) | Phase 3: 빌드 & 플래시 (CMake, PEMicro) |
| [phase4-migrate.md](phase4-migrate.md) | Phase 4: 레거시 마이그레이션 |
| [dependency-graph.md](dependency-graph.md) | 모듈 의존성 그래프 & 구현 순서 |

## 모듈 구조

```
src/
├── main.rs
├── commands/          ← CLI 진입점
│   ├── setup.rs       (Phase 0)
│   ├── toolchain.rs   (Phase 1)
│   ├── new.rs         (Phase 2)
│   ├── build.rs       (Phase 3)
│   ├── flash.rs       (Phase 3)
│   └── migrate.rs     (Phase 4)
├── core/              ← 비즈니스 로직
│   ├── config.rs
│   ├── project.rs
│   ├── toolchain_manager.rs
│   ├── toolchain_registry.rs
│   ├── mcu_db.rs
│   ├── template.rs
│   ├── builder.rs
│   ├── objcopy.rs
│   ├── flasher.rs
│   └── migrate_parser.rs
├── mcu/               ← MCU 정의 데이터
│   ├── nxp.rs
│   └── stm32.rs
└── utils/             ← 공용 유틸리티
    ├── paths.rs
    ├── download.rs
    └── archive.rs
```

## 릴리스 마일스톤

| 버전 | Phase | 시점 | 내용 |
|------|-------|------|------|
| **v0.1.0** | 0+1 | Week 3 | setup + toolchain → 팀 배포 가능 |
| **v0.2.0** | 2 | Week 5 | + 프로젝트 생성 |
| **v0.3.0** | 3 | Week 7 | + 빌드 & 플래시 |
| **v0.4.0** | 4 | Week 9 | + 마이그레이션 |
