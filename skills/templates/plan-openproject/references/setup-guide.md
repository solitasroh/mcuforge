# OpenProject Skill 설치 가이드

## 1단계: API Key 발급

1. 웹 브라우저에서 OpenProject에 로그인
2. 우측 상단 아바타 클릭 → **My account**
3. 좌측 메뉴에서 **Access tokens** 클릭
4. API 행의 **+ API token** (또는 Generate/Reset) 클릭
5. 생성된 토큰을 **즉시 복사** (이후 다시 볼 수 없음)
6. 사용자당 1개만 존재 — 재생성 시 이전 키 즉시 무효화

## 2단계: 환경변수 설정

셸 설정 파일(~/.bashrc, ~/.zshrc 등)에 추가:

```bash
export OPENPROJECT_URL="https://openproject.your-company.com"
export OPENPROJECT_API_KEY="발급받은-토큰-붙여넣기"
```

자체서명 인증서 사용 시:

```bash
export OP_SSL_VERIFY="false"
```

설정 후 셸 재시작 또는 `source ~/.bashrc`

## 3단계: 연결 확인

```bash
python .claude/skills/openproject/scripts/op.py setup test
```

성공 시 출력 예:

```json
{"status": "connected", "instance_version": "15.x.x", "url": "https://..."}
```

실패 시 확인사항:

- 401: API Key가 틀렸거나 만료됨
- 연결 거부: OPENPROJECT_URL 확인
- SSL 에러: OP_SSL_VERIFY="false" 설정 확인

## 4단계: 본인 확인

```bash
python .claude/skills/openproject/scripts/op.py setup whoami
```

출력 예:

```json
{"id": 7, "name": "수장", "login": "sujang", "email": "sujang@company.com"}
```

이 ID가 이슈 조회/생성 시 "나"로 사용됩니다.

## 5단계: ID 매핑 확인

```bash
python .claude/skills/openproject/scripts/op.py setup ids
```

출력된 상태/유형/우선순위 ID를 `references/id-mapping.md`에 기록합니다.
특히 커스텀 상태(In Progress, Done, Close 등)의 ID를 반드시 확인하세요.
