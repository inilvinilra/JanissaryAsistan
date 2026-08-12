#!/usr/bin/env bash
set -euo pipefail

api_url="${API_URL:-http://127.0.0.1:3000}"
test_email="${JURY_TEST_EMAIL:?Set JURY_TEST_EMAIL to an isolated test account email}"
test_password="${JURY_TEST_PASSWORD:?Set JURY_TEST_PASSWORD to the isolated test account password}"

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

expect_status 'health' '200' "$api_url/health"
expect_status 'anonymous-project-access' '401' "$api_url/projects"

login_response="$(curl -sS -X POST "$api_url/auth/login" -H 'Content-Type: application/json' --data "{\"email\":\"$test_email\",\"password\":\"$test_password\"}")"
token="$(jq -r '.token // empty' <<<"$login_response")"
if [[ -z "$token" ]]; then
  printf 'FAIL login: the response did not contain a session token\n' >&2
  exit 1
fi

auth_header="Authorization: Bearer $token"
expect_status 'session' '200' -H "$auth_header" "$api_url/auth/session"
expect_status 'competitions' '200' -H "$auth_header" "$api_url/competitions"
expect_status 'projects' '200' -H "$auth_header" "$api_url/projects"
expect_status 'metrics' '200' "$api_url/metrics"

logout_status="$(status_for -X POST -H "$auth_header" "$api_url/auth/logout")"
if [[ "$logout_status" != '204' ]]; then
  printf 'FAIL logout: expected HTTP 204, received HTTP %s\n' "$logout_status" >&2
  exit 1
fi
printf 'PASS logout: HTTP 204\n'
expect_status 'revoked-session' '401' -H "$auth_header" "$api_url/auth/session"
printf 'API smoke test completed successfully.\n'
