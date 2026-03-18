# 활용 시나리오 상세

## 시나리오 1: 내 이슈 조회

```
사용자: "내 이슈 보여줘"

내부 동작:
  1. GET /api/v3/users/me → 현재 사용자 ID 획득
  2. GET /api/v3/projects/5/work_packages
     ?filters=[{"assignee":{"operator":"=","values":["{내ID}"]}}]
     → 나에게 할당된 WP 목록 반환
```

- `--mine` 옵션이 자동으로 위 과정을 수행
- API Key 기반 인증이므로, 각 사용자의 API key로 요청하면 "me"는 해당 사용자

## 시나리오 2: 이슈 생성

```
사용자: "Accura 2750에 ADC 관련 버그 이슈 만들어줘"

내부 동작:
  1. GET /api/v3/users/me → 내 ID 획득 (또는 캐시)
  2. POST /api/v3/work_packages/form → 기본값 획득 (Form API 1단계)
  3. POST /api/v3/work_packages
     payload:
       subject: "[A2750] [ADC] ..."
       _links.project.href: "/api/v3/projects/5"
       _links.type.href: "/api/v3/types/{type_id}"
       _links.assignee.href: "/api/v3/users/{내ID}"
       startDate: "2026-03-05" (선택)
       dueDate: "2026-03-15" (선택)
```

- Form API 2단계 패턴은 OpenProject에서 WP 생성 시 필수
- `--assign-to-me`가 없어도 별도 지시 없으면 기본 적용

## 시나리오 3: 상태 변경

```
사용자: "이슈 #42를 '개발중'으로 바꿔줘"

내부 동작:
  1. id-mapping.md에서 "개발중" → status_id 확인
     (없으면 GET /api/v3/statuses로 조회)
  2. GET /api/v3/work_packages/42 → lockVersion 획득
  3. PATCH /api/v3/work_packages/42
     payload:
       lockVersion: {현재값}
       _links.status.href: "/api/v3/statuses/{status_id}"
```

- 워크플로우 제약: Role × Type별로 허용된 상태 전이가 정의됨
- 허용되지 않은 전이 시도 → API 에러 → 한글 안내 메시지

## 시나리오 4: 일정 등록/변경

```
사용자: "이슈 #42의 시작일을 3월 5일, 종료일을 3월 15일로 설정해줘"

내부 동작:
  1. GET /api/v3/work_packages/42 → lockVersion 획득
  2. PATCH /api/v3/work_packages/42
     payload:
       lockVersion: {현재값}
       startDate: "2026-03-05"
       dueDate: "2026-03-15"
```

- ISO 8601 날짜 형식: YYYY-MM-DD
- date 필드는 Milestone 타입 WP에만 사용 (일반 WP는 startDate + dueDate)
- null 값으로 일정 제거 가능 (빈 문자열 "" 전달)
