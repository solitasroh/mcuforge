# OpenProject 필터 문법

## 기본 구조

필터는 JSON 배열로 전달되며, 각 요소는 AND로 조합됩니다.

```json
[
  {"field_name": {"operator": "op", "values": ["val1", "val2"]}},
  {"another_field": {"operator": "op", "values": ["val"]}}
]
```

URL 인코딩하여 `filters` 쿼리 파라미터로 전달합니다.

## 연산자

| 연산자 | 의미 | values |
|--------|------|--------|
| `=` | 같음 (OR) | `["val1", "val2"]` |
| `!` | 같지 않음 | `["val1"]` |
| `o` | 열림 (open) | `null` |
| `c` | 닫힘 (closed) | `null` |
| `~` | 포함 (contains) | `["text"]` |
| `!~` | 포함하지 않음 | `["text"]` |
| `**` | 존재함 (not null) | `null` |
| `!*` | 존재하지 않음 (null) | `null` |
| `>=` | 이상 | `["2026-03-01"]` |
| `<=` | 이하 | `["2026-03-31"]` |
| `<>d` | 날짜 범위 | `["2026-03-01", "2026-03-31"]` |

## 자주 사용하는 필터 조합

### 열린 이슈

```json
[{"status_id": {"operator": "o", "values": null}}]
```

### 닫힌 이슈

```json
[{"status_id": {"operator": "c", "values": null}}]
```

### 나에게 할당된 이슈

```json
[{"assignee": {"operator": "=", "values": ["USER_ID"]}}]
```

### 특정 유형

```json
[{"type_id": {"operator": "=", "values": ["TYPE_ID"]}}]
```

### 복합: 내 열린 이슈 (특정 유형)

```json
[
  {"assignee": {"operator": "=", "values": ["7"]}},
  {"status_id": {"operator": "o", "values": null}},
  {"type_id": {"operator": "=", "values": ["1"]}}
]
```

### 이번 달 마감 이슈

```json
[
  {"due_date": {"operator": "<=", "values": ["2026-03-31"]}},
  {"due_date": {"operator": ">=", "values": ["2026-03-01"]}},
  {"status_id": {"operator": "o", "values": null}}
]
```

## 필터 가능 필드

| 필드명 | 설명 |
|--------|------|
| `status_id` | 상태 |
| `type_id` | 유형 |
| `priority_id` | 우선순위 |
| `assignee` | 담당자 (user ID) |
| `author` | 생성자 (user ID) |
| `subject` | 제목 (~: 포함) |
| `created_at` | 생성일 |
| `updated_at` | 수정일 |
| `due_date` | 종료일 |
| `start_date` | 시작일 |
| `parent` | 상위 WP |
| `version` | 버전/스프린트 |
