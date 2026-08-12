#!/usr/bin/env bash
set -euo pipefail

api_url="${API_URL:-http://127.0.0.1:3000}"
admin_email="${JURY_TEST_ADMIN_EMAIL:?Set JURY_TEST_ADMIN_EMAIL to an isolated system administrator account}"
admin_password="${JURY_TEST_ADMIN_PASSWORD:?Set JURY_TEST_ADMIN_PASSWORD to the isolated system administrator password}"
run_id="$(date +%s)"
temporary_password="Jury-Temporary-2026"
updated_password="Jury-Updated-2026"

status_for() {
  curl -sS -o /dev/null -w '%{http_code}' "$@"
}

expect_status() {
  local name="$1"
  local expected="$2"
  shift 2
  local actual
  actual="$(status_for "$@")"
  if [[ "$actual" != "$expected" ]]; then
    printf 'FAIL %s: expected HTTP %s, received HTTP %s\n' "$name" "$expected" "$actual" >&2
    exit 1
  fi
  printf 'PASS %s: HTTP %s\n' "$name" "$actual"
}

login_token() {
  local email="$1"
  local password="$2"
  local response
  response="$(curl -sS -X POST "$api_url/auth/login" -H 'Content-Type: application/json' --data "{\"email\":\"$email\",\"password\":\"$password\"}")"
  jq -r '.token // empty' <<<"$response"
}

change_temporary_password() {
  local token="$1"
  expect_status 'temporary-password-change' '204' -X PUT "$api_url/auth/password" \
    -H "Authorization: Bearer $token" \
    -H 'Content-Type: application/json' \
    --data "{\"current_password\":\"$temporary_password\",\"new_password\":\"$updated_password\"}"
}

admin_token="$(login_token "$admin_email" "$admin_password")"
if [[ -z "$admin_token" ]]; then
  printf 'FAIL administrator login did not return a session token\n' >&2
  exit 1
fi
admin_header="Authorization: Bearer $admin_token"
projects="$(curl -sS "$api_url/projects" -H "$admin_header")"
project_id="$(jq -r '.[0].id // empty' <<<"$projects")"
competition_id="$(jq -r '.[0].competition_id // empty' <<<"$projects")"
category="$(jq -r '.[0].category // empty' <<<"$projects")"
if [[ -z "$project_id" || -z "$competition_id" || -z "$category" ]]; then
  printf 'FAIL an isolated test project with competition and category is required\n' >&2
  exit 1
fi

other_competition_response="$(curl -sS -X POST "$api_url/competitions" -H "$admin_header" -H 'Content-Type: application/json' --data "{\"name\":\"Authorization Regression ${run_id}\",\"description\":\"Isolated scope test competition\",\"organization\":\"Automated Tests\"}")"
other_competition_id="$(jq -r '.id // empty' <<<"$other_competition_response")"
if [[ -z "$other_competition_id" ]]; then
  printf 'FAIL isolated secondary competition creation failed\n' >&2
  exit 1
fi

jury_email="scope-jury-${run_id}@example.test"
observer_email="scope-observer-${run_id}@example.test"
create_user() {
  local email="$1"
  local role="$2"
  local category_value="$3"
  expect_status "create-${role}" '201' -X POST "$api_url/users" \
    -H "$admin_header" \
    -H 'Content-Type: application/json' \
    --data "{\"full_name\":\"Automated ${role}\",\"email\":\"${email}\",\"role\":\"${role}\",\"competition_id\":${competition_id},\"category\":${category_value},\"password\":\"${temporary_password}\"}"
}

create_user "$jury_email" 'jury_member' "\"$category\""
jury_initial_token="$(login_token "$jury_email" "$temporary_password")"
change_temporary_password "$jury_initial_token"
jury_token="$(login_token "$jury_email" "$updated_password")"
if [[ -z "$jury_token" ]]; then
  printf 'FAIL jury login did not return a session token\n' >&2
  exit 1
fi
jury_header="Authorization: Bearer $jury_token"
expect_status 'jury-users-denied' '403' -H "$jury_header" "$api_url/users"
expect_status 'jury-metadata-denied' '403' -H "$jury_header" "$api_url/projects/$project_id/metadata"
expect_status 'jury-files-denied' '403' -H "$jury_header" "$api_url/projects/$project_id/files"
expect_status 'jury-ai-evaluation-denied' '403' -H "$jury_header" "$api_url/projects/$project_id/ai-evaluation"
expect_status 'jury-scoped-project-list' '200' -H "$jury_header" "$api_url/projects?category=$category"
expect_status 'jury-unscoped-project-list-denied' '403' -H "$jury_header" "$api_url/projects"
expect_status 'jury-other-competition-denied' '403' -H "$jury_header" "$api_url/competitions/$other_competition_id/stages"
jury_competitions="$(curl -sS "$api_url/competitions" -H "$jury_header")"
if jq -e --argjson other_id "$other_competition_id" 'any(.[]; .id == $other_id)' <<<"$jury_competitions" >/dev/null; then
  printf 'FAIL jury-competition-list disclosed an out-of-scope competition\n' >&2
  exit 1
fi
printf 'PASS jury-competition-list excludes out-of-scope competition\n'

create_user "$observer_email" 'observer' 'null'
observer_initial_token="$(login_token "$observer_email" "$temporary_password")"
change_temporary_password "$observer_initial_token"
observer_token="$(login_token "$observer_email" "$updated_password")"
if [[ -z "$observer_token" ]]; then
  printf 'FAIL observer login did not return a session token\n' >&2
  exit 1
fi
observer_header="Authorization: Bearer $observer_token"
expect_status 'observer-project-list' '200' -H "$observer_header" "$api_url/projects"
expect_status 'observer-users-denied' '403' -H "$observer_header" "$api_url/users"
expect_status 'observer-audit-denied' '403' -H "$observer_header" "$api_url/audit"
expect_status 'observer-notifications-denied' '403' -H "$observer_header" "$api_url/notifications"
expect_status 'observer-metadata-denied' '403' -H "$observer_header" "$api_url/projects/$project_id/metadata"
expect_status 'observer-other-competition-denied' '403' -H "$observer_header" "$api_url/competitions/$other_competition_id/stages"
printf 'Authorization regression test completed successfully.\n'
