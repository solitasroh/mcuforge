# 모듈 의존성 그래프 & 구현 순서

---

## 의존성 방향

```
commands/  →  core/  →  utils/
                ↓
              mcu/
```

**규칙**: 화살표 역방향 의존 금지

---

## 모듈별 의존성

```
commands/
  setup.rs      → core/project, core/config, core/toolchain_manager, core/template
  toolchain.rs  → core/toolchain_manager
  new.rs        → core/mcu_db, core/template, core/config
  build.rs      → core/builder
  flash.rs      → core/flasher
  migrate.rs    → core/migrate_parser, core/project, core/template

core/
  config.rs              → utils/paths
  project.rs             → utils/paths
  toolchain_registry.rs  → (없음)
  toolchain_manager.rs   → config, toolchain_registry, utils/download, utils/archive
  mcu_db.rs              → mcu/nxp
  template.rs            → mcu_db
  builder.rs             → project, toolchain_manager
  objcopy.rs             → (없음, std::process만)
  flasher.rs             → project, toolchain_manager
  migrate_parser.rs      → mcu_db

mcu/
  nxp.rs   → (없음)
  stm32.rs → (없음, 향후)

utils/
  paths.rs    → (없음)
  download.rs → paths
  archive.rs  → paths
```

---

## 구현 순서

의존성 없는 모듈부터 시작, 아래로 갈수록 의존성이 많아짐

### Week 1: 기반 모듈 (의존성 0)

| # | 모듈 | 의존성 | Phase |
|---|------|--------|-------|
| ① | `utils/paths.rs` | 없음 | 0 |
| ② | `mcu/nxp.rs` | 없음 | 2 (선행) |
| ③ | `core/toolchain_registry.rs` | 없음 | 1 (선행) |

### Week 1: 기반 모듈 (의존성 1)

| # | 모듈 | 의존성 | Phase |
|---|------|--------|-------|
| ④ | `core/config.rs` | ← paths | 0 |
| ⑤ | `core/project.rs` | ← paths | 0 |
| ⑥ | `core/mcu_db.rs` | ← nxp | 2 (선행) |
| ⑦ | `utils/download.rs` | ← paths | 1 |
| ⑧ | `utils/archive.rs` | ← paths | 1 |

### Week 2-3: 핵심 로직

| # | 모듈 | 의존성 | Phase |
|---|------|--------|-------|
| ⑨ | `core/toolchain_manager.rs` | ← config, registry, download, archive | 1 |
| ⑩ | `commands/toolchain.rs` | ← toolchain_manager | 1 |
| ⑪ | `commands/setup.rs` | ← config, project, toolchain_manager | 0 |

**→ v0.1.0 릴리스 가능** (setup + toolchain)

### Week 4-5: 프로젝트 생성

| # | 모듈 | 의존성 | Phase |
|---|------|--------|-------|
| ⑫ | `core/template.rs` | ← mcu_db | 2 |
| ⑬ | `commands/new.rs` | ← mcu_db, template, config | 2 |

**→ v0.2.0 릴리스 가능** (+ new)

### Week 6-7: 빌드

| # | 모듈 | 의존성 | Phase |
|---|------|--------|-------|
| ⑭ | `core/builder.rs` | ← project, toolchain_manager | 3 |
| ⑮ | `core/objcopy.rs` | ← (std::process) | 3 |
| ⑯ | `core/flasher.rs` | ← project, toolchain_manager | 3 |
| ⑰ | `commands/build.rs` | ← builder | 3 |
| ⑱ | `commands/flash.rs` | ← flasher | 3 |

**→ v0.3.0 릴리스 가능** (+ build, flash)

### Week 8-9: 마이그레이션

| # | 모듈 | 의존성 | Phase |
|---|------|--------|-------|
| ⑲ | `core/migrate_parser.rs` | ← mcu_db | 4 |
| ⑳ | `commands/migrate.rs` | ← migrate_parser, project, template | 4 |

**→ v0.4.0 릴리스 가능** (+ migrate)

---

## 요약 다이어그램

```
          ┌─────────────────────────────────────────┐
Week 1    │  paths   nxp   registry                 │  의존성 0
          │    ↓      ↓       ↓                     │
          │ config project mcu_db download archive  │  의존성 1
          └──────────────┬──────────────────────────┘
                         ↓
          ┌──────────────────────────────────┐
Week 2-3  │    toolchain_manager             │  의존성 4
          │         ↓           ↓            │
          │  cmd/toolchain   cmd/setup       │
          └──────────────────────────────────┘
                    v0.1.0 ──────────────────→ 팀 배포
                         ↓
          ┌──────────────────────────────────┐
Week 4-5  │  template → cmd/new              │
          └──────────────────────────────────┘
                    v0.2.0
                         ↓
          ┌──────────────────────────────────┐
Week 6-7  │  builder  objcopy  flasher       │
          │  cmd/build    cmd/flash          │
          └──────────────────────────────────┘
                    v0.3.0
                         ↓
          ┌──────────────────────────────────┐
Week 8-9  │  migrate_parser → cmd/migrate    │
          └──────────────────────────────────┘
                    v0.4.0
```
