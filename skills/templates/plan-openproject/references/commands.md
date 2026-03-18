# 명령어 레퍼런스

## setup

### `op.py setup test`

API 연결 확인. 환경변수가 올바르게 설정되었는지 검증.

### `op.py setup whoami`

`GET /api/v3/users/me` — 현재 API Key 소유자 정보 반환.

### `op.py setup ids`

status/type/priority ID를 한 번에 조회. `id-mapping.md` 채울 때 사용.

## wp (Work Package)

### `op.py wp list`

| 옵션 | 필수 | 설명 |
|------|------|------|
| `--project-id N` | | 프로젝트 ID (config 기본값: 5) |
| `--mine` | | 나에게 할당된 것만 (내부: whoami → assignee 필터) |
| `--status open\|closed\|all` | | 상태 필터 (기본: open) |
| `--type-id N` | | 유형 필터 |
| `--page N` | | 페이지 번호 (offset) |
| `--per-page N` | | 페이지당 개수 (pageSize) |

### `op.py wp get ID`

WP 상세 정보 조회. description 포함.

### `op.py wp create`

| 옵션 | 필수 | 설명 |
|------|------|------|
| `--project-id N` | | 프로젝트 ID (config 기본값: 5) |
| `--subject "..."` | 필수 | 이슈 제목 |
| `--type-id N` | 필수 | 유형 ID (매핑: 작업=11, 기능=13, 버그=14) |
| `--assign-to-me` | | 나를 assignee로 설정 (whoami 자동) |
| `--assignee-id N` | | 특정 사용자에게 할당 |
| `--description "..."` | | 본문 (Markdown) |
| `--priority-id N` | | 우선순위 ID |
| `--start-date YYYY-MM-DD` | | 시작일 |
| `--due-date YYYY-MM-DD` | | 종료일 |
| `--parent-id N` | | 상위 WP ID |

내부적으로 Form API 2단계 패턴 사용:
1. `POST /api/v3/work_packages/form` → 기본값 획득
2. `POST /api/v3/work_packages` → 실제 생성

### `op.py wp update ID`

| 옵션 | 설명 |
|------|------|
| `--status-id N` | 상태 변경 |
| `--start-date YYYY-MM-DD` | 시작일 (""로 제거) |
| `--due-date YYYY-MM-DD` | 종료일 (""로 제거) |
| `--percentage-done N` | 진행률 (0-100) |
| `--subject "..."` | 제목 변경 |
| `--description "..."` | 본문 변경 |
| `--priority-id N` | 우선순위 변경 |
| `--assignee-id N` | 담당자 변경 |

내부적으로 Optimistic Locking 사용:
1. `GET /api/v3/work_packages/{id}` → lockVersion 획득
2. `PATCH /api/v3/work_packages/{id}` → lockVersion 포함

### `op.py wp delete ID`

WP 삭제. 복구 불가능 — 사용자 확인 필수.

## 참조 데이터

### `op.py status list`

전체 상태 목록 반환 (커스텀 상태 포함). 필드: id, name, isClosed, isDefault, color.

### `op.py type list [--project-id N]`

WP 유형 목록. `--project-id` 지정 시 해당 프로젝트에서 사용 가능한 유형만.

### `op.py priority list`

우선순위 목록. 필드: id, name, isDefault.
