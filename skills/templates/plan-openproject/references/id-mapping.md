# ID 매핑표

## 프로젝트 (고정)

| ID | 이름 | 식별자 |
|----|------|--------|
| 5 | Accura 2750IRM | — |

## 상태 (Status)

> 커스텀 상태 + OpenProject 기본 상태가 공존합니다.
> 상태 변경은 워크플로우 규칙의 제약을 받습니다.

### 커스텀 상태 (주로 사용)

| ID | 이름 | 닫힘 여부 |
|----|------|----------|
| 1 | 대기중(Backlog) | open |
| 16 | 열림(Open) | open |
| 17 | 진행중(In progress) | open |
| 18 | 완료(Done) | open |
| 21 | 종료(Closed) | closed |
| 20 | 취소(Canceled) | closed |
| 19 | 보류(On hold) | open |

### 기본 상태 (레거시)

| ID | 이름 | 닫힘 여부 |
|----|------|----------|
| 2 | In specification | open |
| 3 | Specified | open |
| 4 | Confirmed | open |
| 5 | To be scheduled | open |
| 6 | Scheduled | open |
| 7 | In progress | open |
| 8 | Developed | open |
| 9 | In testing | open |
| 10 | Tested | open |
| 11 | Test failed | open |
| 12 | Closed | closed |
| 13 | On hold | open |
| 14 | Rejected | closed |

## 유형 (Type)

### 커스텀 유형 (주로 사용)

| ID | 이름 |
|----|------|
| 3 | 이니셔티브(Initiative) |
| 8 | 에픽(Epic) |
| 13 | 기능(Feature) |
| 14 | 버그(Bug) |
| 11 | 작업(Task) |
| 12 | 마일스톤(Milestone) |
| 15 | TestCase Template |

### 기본 유형 (레거시)

| ID | 이름 |
|----|------|
| 1 | Task |
| 2 | Milestone |
| 4 | Feature |
| 5 | Epic |
| 6 | User story |
| 7 | Bug |

## 우선순위 (Priority)

| ID | 이름 |
|----|------|
| 7 | Low |
| 8 | Normal (기본) |
| 9 | High |
| 10 | Immediate |
