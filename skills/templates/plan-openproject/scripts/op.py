#!/usr/bin/env python3
"""
OpenProject CLI — Claude Code Skill용.
외부 의존성 없이 Python 표준 라이브러리(urllib, argparse)만 사용.

사용법: python op.py {리소스} {액션} [옵션]

설정 우선순위:
  1. config.json (공유, git 커밋) + config.local.json (개인, gitignore)
  2. 환경변수 OPENPROJECT_URL, OPENPROJECT_API_KEY (fallback)
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import ssl
import sys
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import Request, urlopen

# 스킬 디렉토리 (scripts/ 의 상위)
_SKILL_DIR = Path(__file__).resolve().parent.parent


# ---------------------------------------------------------------------------
# Config Loader
# ---------------------------------------------------------------------------

def _load_config() -> dict:
    """config.json (공유) + config.local.json (개인) → 병합 → 환경변수 fallback."""
    cfg: dict = {}

    # 1. 공유 설정 (git 커밋됨)
    shared = _SKILL_DIR / "config.json"
    if shared.exists():
        with open(shared, encoding="utf-8") as f:
            cfg.update(json.load(f))

    # 2. 개인 설정 (gitignore)
    local = _SKILL_DIR / "config.local.json"
    if local.exists():
        with open(local, encoding="utf-8") as f:
            cfg.update(json.load(f))

    # 3. 환경변수 fallback
    if not cfg.get("url"):
        cfg["url"] = os.environ.get("OPENPROJECT_URL", "")
    if not cfg.get("api_key"):
        cfg["api_key"] = os.environ.get("OPENPROJECT_API_KEY", "")
    if "ssl_verify" not in cfg:
        env_ssl = os.environ.get("OP_SSL_VERIFY", "true").lower()
        cfg["ssl_verify"] = env_ssl not in ("false", "0", "no")

    return cfg


# ---------------------------------------------------------------------------
# API Client
# ---------------------------------------------------------------------------

class OpenProjectAPI:
    """OpenProject API v3 클라이언트 (urllib 기반, 외부 의존성 없음)."""

    def __init__(self) -> None:
        cfg = _load_config()
        url = cfg.get("url", "")
        key = cfg.get("api_key", "")
        if not url or not key:
            _die("OpenProject 설정이 없습니다.\n"
                 "config.local.json에 api_key를 설정하거나,\n"
                 "환경변수 OPENPROJECT_URL + OPENPROJECT_API_KEY를 설정하세요.\n"
                 "상세: references/setup-guide.md")
        self.base_url = url.rstrip("/") + "/api/v3"
        self._auth = "Basic " + base64.b64encode(f"apikey:{key}".encode()).decode()
        self._ssl_ctx = self._make_ssl_context(cfg.get("ssl_verify", True))
        self._my_user_id: int | None = None
        self.default_project_id: int | None = cfg.get("default_project_id")

    @staticmethod
    def _make_ssl_context(ssl_verify: bool) -> ssl.SSLContext | None:
        if not ssl_verify:
            ctx = ssl.create_default_context()
            ctx.check_hostname = False
            ctx.verify_mode = ssl.CERT_NONE
            return ctx
        return None

    def _request(self, method: str, path: str, data: dict | None = None) -> dict:
        url = self.base_url + path if path.startswith("/") else self.base_url + "/" + path
        headers = {
            "Authorization": self._auth,
            "Content-Type": "application/json",
        }
        body = json.dumps(data).encode() if data else None
        req = Request(url, data=body, headers=headers, method=method)
        try:
            kwargs = {"timeout": 30}
            if self._ssl_ctx:
                kwargs["context"] = self._ssl_ctx
            with urlopen(req, **kwargs) as resp:
                raw = resp.read().decode()
                return json.loads(raw) if raw else {}
        except HTTPError as e:
            body_text = ""
            try:
                body_text = e.read().decode()
            except Exception:
                pass
            _die(f"HTTP {e.code}: {body_text}")
        except URLError as e:
            _die(f"연결 실패: {e.reason}")
        return {}

    def get(self, path: str) -> dict:
        return self._request("GET", path)

    def post(self, path: str, data: dict) -> dict:
        return self._request("POST", path, data)

    def patch(self, path: str, data: dict) -> dict:
        return self._request("PATCH", path, data)

    def delete(self, path: str) -> dict:
        return self._request("DELETE", path)

    def whoami(self) -> dict:
        """GET /api/v3/users/me — 현재 사용자 정보 (캐시)."""
        data = self.get("/users/me")
        self._my_user_id = data.get("id")
        return {
            "id": data.get("id"),
            "name": data.get("name", ""),
            "login": data.get("login", ""),
            "email": data.get("email", ""),
        }

    def my_user_id(self) -> int:
        """캐시된 사용자 ID 반환. 없으면 whoami 호출."""
        if self._my_user_id is None:
            self.whoami()
        return self._my_user_id


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _die(msg: str) -> None:
    """에러 메시지를 stderr로 출력하고 종료."""
    if "Status" in msg and ("transition" in msg.lower() or "allowed" in msg.lower()):
        print("ERROR: 워크플로우에서 허용되지 않은 상태 전이입니다.", file=sys.stderr)
        print("현재 사용자의 역할(Role)과 WP 유형(Type)에 따라", file=sys.stderr)
        print("허용된 상태 전이가 다릅니다.", file=sys.stderr)
        print(f"상세: {msg}", file=sys.stderr)
    else:
        print(f"ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def _output(data: dict | list) -> None:
    """JSON 결과를 stdout으로 출력."""
    print(json.dumps(data, ensure_ascii=False, indent=2))


def _summarize_wp(wp: dict) -> dict:
    """WP 응답에서 핵심 필드만 추출."""
    embedded = wp.get("_embedded", {})
    links = wp.get("_links", {})

    # _embedded 우선, 없으면 _links.*.title fallback
    status_name = (embedded.get("status", {}).get("name", "")
                   or links.get("status", {}).get("title", ""))
    type_name = (embedded.get("type", {}).get("name", "")
                 or links.get("type", {}).get("title", ""))
    priority_name = (embedded.get("priority", {}).get("name", "")
                     or links.get("priority", {}).get("title", ""))

    assignee_embedded = embedded.get("assignee")
    assignee_link = links.get("assignee", {})
    assignee_name = (assignee_embedded.get("name", "") if assignee_embedded
                     else assignee_link.get("title") if assignee_link.get("href") else None)

    return {
        "id": wp.get("id"),
        "subject": wp.get("subject", ""),
        "status": status_name,
        "type": type_name,
        "assignee": assignee_name,
        "priority": priority_name,
        "percentageDone": wp.get("percentageDone"),
        "startDate": wp.get("startDate"),
        "dueDate": wp.get("dueDate"),
        "createdAt": wp.get("createdAt"),
        "updatedAt": wp.get("updatedAt"),
    }


# ---------------------------------------------------------------------------
# Command Handlers
# ---------------------------------------------------------------------------

def cmd_setup_test(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    data = api.get("/")
    _output({
        "status": "connected",
        "instance_version": data.get("instanceName", "unknown"),
        "url": api.base_url.replace("/api/v3", ""),
    })


def cmd_setup_whoami(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    _output(api.whoami())


def cmd_setup_ids(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    statuses = api.get("/statuses").get("_embedded", {}).get("elements", [])
    types_ = api.get("/types").get("_embedded", {}).get("elements", [])
    priorities = api.get("/priorities").get("_embedded", {}).get("elements", [])

    _output({
        "statuses": [{"id": s["id"], "name": s["name"],
                       "isClosed": s.get("isClosed", False)} for s in statuses],
        "types": [{"id": t["id"], "name": t["name"]} for t in types_],
        "priorities": [{"id": p["id"], "name": p["name"]} for p in priorities],
    })


def _resolve_project_id(api: OpenProjectAPI, args: argparse.Namespace) -> int:
    """--project-id가 없으면 config의 default_project_id 사용."""
    pid = getattr(args, "project_id", None)
    if pid:
        return pid
    if api.default_project_id:
        return api.default_project_id
    _die("--project-id를 지정하거나 config.json에 default_project_id를 설정하세요.")
    return 0  # unreachable


def cmd_wp_list(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    project_id = _resolve_project_id(api, args)
    filters = []

    if args.mine:
        my_id = api.my_user_id()
        filters.append({"assignee": {"operator": "=", "values": [str(my_id)]}})

    status = getattr(args, "status", "open")
    if status == "open":
        filters.append({"status_id": {"operator": "o", "values": None}})
    elif status == "closed":
        filters.append({"status_id": {"operator": "c", "values": None}})

    if args.type_id:
        filters.append({"type_id": {"operator": "=", "values": [str(args.type_id)]}})

    params = {}
    if filters:
        params["filters"] = json.dumps(filters)
    if args.page:
        params["offset"] = str(args.page)
    if args.per_page:
        params["pageSize"] = str(args.per_page)

    query = f"?{urlencode(params)}" if params else ""
    result = api.get(f"/projects/{project_id}/work_packages{query}")
    elements = result.get("_embedded", {}).get("elements", [])

    _output({
        "total": result.get("total", 0),
        "count": len(elements),
        "work_packages": [_summarize_wp(wp) for wp in elements],
    })


def cmd_wp_get(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    wp = api.get(f"/work_packages/{args.id}")
    detail = _summarize_wp(wp)
    desc = wp.get("description", {})
    detail["description"] = desc.get("raw", "") if isinstance(desc, dict) else ""
    _output(detail)


def cmd_wp_create(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    project_id = _resolve_project_id(api, args)
    # Form API 1단계
    form_payload: dict = {
        "subject": args.subject,
        "_links": {
            "project": {"href": f"/api/v3/projects/{project_id}"},
            "type": {"href": f"/api/v3/types/{args.type_id}"},
        },
    }
    form = api.post("/work_packages/form", form_payload)
    payload = form.get("_embedded", {}).get("payload", form_payload)
    payload["lockVersion"] = payload.get("lockVersion", 0)

    # subject 덮어쓰기 (Form API가 바꿀 수 있으므로)
    payload["subject"] = args.subject

    # _links 구성
    links = payload.setdefault("_links", {})
    links["project"] = {"href": f"/api/v3/projects/{project_id}"}
    links["type"] = {"href": f"/api/v3/types/{args.type_id}"}

    # assignee
    if args.assign_to_me:
        my_id = api.my_user_id()
        links["assignee"] = {"href": f"/api/v3/users/{my_id}"}
    elif args.assignee_id:
        links["assignee"] = {"href": f"/api/v3/users/{args.assignee_id}"}

    # priority
    if args.priority_id:
        links["priority"] = {"href": f"/api/v3/priorities/{args.priority_id}"}

    # parent
    if args.parent_id:
        links["parent"] = {"href": f"/api/v3/work_packages/{args.parent_id}"}

    # description
    if args.description:
        payload["description"] = {"raw": args.description}

    # dates
    if args.start_date:
        payload["startDate"] = args.start_date
    if args.due_date:
        payload["dueDate"] = args.due_date

    result = api.post("/work_packages", payload)
    _output({"created": True, "id": result["id"], "subject": result["subject"]})


def cmd_wp_update(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    # lockVersion 획득
    current = api.get(f"/work_packages/{args.id}")
    payload: dict = {"lockVersion": current["lockVersion"]}
    links: dict = {}

    if args.status_id:
        links["status"] = {"href": f"/api/v3/statuses/{args.status_id}"}
    if args.priority_id:
        links["priority"] = {"href": f"/api/v3/priorities/{args.priority_id}"}
    if args.assignee_id:
        links["assignee"] = {"href": f"/api/v3/users/{args.assignee_id}"}

    if args.subject:
        payload["subject"] = args.subject
    if args.description is not None:
        payload["description"] = {"raw": args.description}
    if args.percentage_done is not None:
        payload["percentageDone"] = args.percentage_done

    # dates: 빈 문자열("")은 null로 변환하여 일정 제거
    if args.start_date is not None:
        payload["startDate"] = args.start_date if args.start_date else None
    if args.due_date is not None:
        payload["dueDate"] = args.due_date if args.due_date else None

    if links:
        payload["_links"] = links

    api.patch(f"/work_packages/{args.id}", payload)
    _output({"updated": True, "id": args.id})


def cmd_wp_delete(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    api.delete(f"/work_packages/{args.id}")
    _output({"deleted": True, "id": args.id})


def cmd_status_list(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    data = api.get("/statuses")
    elements = data.get("_embedded", {}).get("elements", [])
    _output({
        "statuses": [
            {"id": s["id"], "name": s["name"],
             "isClosed": s.get("isClosed", False),
             "isDefault": s.get("isDefault", False),
             "color": s.get("color", "")}
            for s in elements
        ]
    })


def cmd_type_list(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    path = "/types"
    if hasattr(args, "project_id") and args.project_id:
        path = f"/projects/{args.project_id}/types"
    data = api.get(path)
    elements = data.get("_embedded", {}).get("elements", [])
    _output({"types": [{"id": t["id"], "name": t["name"]} for t in elements]})


def cmd_priority_list(api: OpenProjectAPI, args: argparse.Namespace) -> None:
    data = api.get("/priorities")
    elements = data.get("_embedded", {}).get("elements", [])
    _output({
        "priorities": [
            {"id": p["id"], "name": p["name"],
             "isDefault": p.get("isDefault", False)}
            for p in elements
        ]
    })


# ---------------------------------------------------------------------------
# CLI Parser
# ---------------------------------------------------------------------------

def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="op.py",
        description="OpenProject CLI for Claude Code Skill",
    )
    sub = parser.add_subparsers(dest="resource", required=True)

    # --- setup ---
    setup_parser = sub.add_parser("setup", help="Setup and diagnostics")
    setup_sub = setup_parser.add_subparsers(dest="action", required=True)
    setup_sub.add_parser("test", help="Test API connection")
    setup_sub.add_parser("whoami", help="Show current user info")
    setup_sub.add_parser("ids", help="List all status/type/priority IDs")

    # --- wp ---
    wp_parser = sub.add_parser("wp", help="Work Package operations")
    wp_sub = wp_parser.add_subparsers(dest="action", required=True)

    # wp list
    wp_list = wp_sub.add_parser("list", help="List work packages")
    wp_list.add_argument("--project-id", type=int)
    wp_list.add_argument("--mine", action="store_true", help="Only my assigned WPs")
    wp_list.add_argument("--status", choices=["open", "closed", "all"], default="open")
    wp_list.add_argument("--type-id", type=int)
    wp_list.add_argument("--page", type=int)
    wp_list.add_argument("--per-page", type=int)

    # wp get
    wp_get = wp_sub.add_parser("get", help="Get work package detail")
    wp_get.add_argument("id", type=int)

    # wp create
    wp_create = wp_sub.add_parser("create", help="Create work package")
    wp_create.add_argument("--project-id", type=int)
    wp_create.add_argument("--subject", required=True)
    wp_create.add_argument("--type-id", type=int, required=True)
    wp_create.add_argument("--assign-to-me", action="store_true",
                           help="Assign to current user")
    wp_create.add_argument("--assignee-id", type=int)
    wp_create.add_argument("--description")
    wp_create.add_argument("--priority-id", type=int)
    wp_create.add_argument("--start-date", help="YYYY-MM-DD")
    wp_create.add_argument("--due-date", help="YYYY-MM-DD")
    wp_create.add_argument("--parent-id", type=int)

    # wp update
    wp_update = wp_sub.add_parser("update", help="Update work package")
    wp_update.add_argument("id", type=int)
    wp_update.add_argument("--status-id", type=int)
    wp_update.add_argument("--start-date", help="YYYY-MM-DD or empty to clear")
    wp_update.add_argument("--due-date", help="YYYY-MM-DD or empty to clear")
    wp_update.add_argument("--percentage-done", type=int)
    wp_update.add_argument("--subject")
    wp_update.add_argument("--description")
    wp_update.add_argument("--priority-id", type=int)
    wp_update.add_argument("--assignee-id", type=int)

    # wp delete
    wp_delete = wp_sub.add_parser("delete", help="Delete work package")
    wp_delete.add_argument("id", type=int)

    # --- status ---
    status_parser = sub.add_parser("status", help="Status operations")
    status_sub = status_parser.add_subparsers(dest="action", required=True)
    status_sub.add_parser("list", help="List all statuses")

    # --- type ---
    type_parser = sub.add_parser("type", help="Type operations")
    type_sub = type_parser.add_subparsers(dest="action", required=True)
    type_list = type_sub.add_parser("list", help="List all types")
    type_list.add_argument("--project-id", type=int)

    # --- priority ---
    priority_parser = sub.add_parser("priority", help="Priority operations")
    priority_sub = priority_parser.add_subparsers(dest="action", required=True)
    priority_sub.add_parser("list", help="List all priorities")

    return parser


# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------

COMMANDS = {
    ("setup", "test"): cmd_setup_test,
    ("setup", "whoami"): cmd_setup_whoami,
    ("setup", "ids"): cmd_setup_ids,
    ("wp", "list"): cmd_wp_list,
    ("wp", "get"): cmd_wp_get,
    ("wp", "create"): cmd_wp_create,
    ("wp", "update"): cmd_wp_update,
    ("wp", "delete"): cmd_wp_delete,
    ("status", "list"): cmd_status_list,
    ("type", "list"): cmd_type_list,
    ("priority", "list"): cmd_priority_list,
}


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    key = (args.resource, args.action)
    handler = COMMANDS.get(key)
    if not handler:
        _die(f"알 수 없는 명령: {args.resource} {args.action}")

    api = OpenProjectAPI()
    handler(api, args)


if __name__ == "__main__":
    main()
