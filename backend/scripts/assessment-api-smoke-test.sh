#!/usr/bin/env bash
set -euo pipefail

api_url="${API_URL:-http://127.0.0.1:3000}"
email="${ASSESSMENT_TEST_EMAIL:?ASSESSMENT_TEST_EMAIL is required}"
password="${ASSESSMENT_TEST_PASSWORD:?ASSESSMENT_TEST_PASSWORD is required}"
first_file="${ASSESSMENT_TEST_PROJECT_ONE_FILE:?ASSESSMENT_TEST_PROJECT_ONE_FILE is required}"
second_file="${ASSESSMENT_TEST_PROJECT_TWO_FILE:?ASSESSMENT_TEST_PROJECT_TWO_FILE is required}"

login_response=$(curl -fsS -X POST "$api_url/auth/login" -H 'Content-Type: application/json' --data "{\"email\":\"$email\",\"password\":\"$password\"}")
token=$(node -e 'process.stdout.write(JSON.parse(process.argv[1]).token)' "$login_response")
authorization="Authorization: Bearer $token"
two_factor_setup=$(curl -fsS -X POST "$api_url/auth/2fa/setup" -H "$authorization")
two_factor_secret=$(node -e 'process.stdout.write(JSON.parse(process.argv[1]).secret)' "$two_factor_setup")
two_factor_code=$(node -e '
const crypto = require("crypto");
const secret = process.argv[1].replace(/=+$/, "").toUpperCase();
const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
let bits = "";
for (const character of secret) bits += alphabet.indexOf(character).toString(2).padStart(5, "0");
const bytes = Buffer.from(bits.slice(0, bits.length - (bits.length % 8)).match(/.{8}/g).map((part) => Number.parseInt(part, 2)));
const counter = Buffer.alloc(8);
counter.writeBigUInt64BE(BigInt(Math.floor(Date.now() / 1000 / 30)));
const digest = crypto.createHmac("sha1", bytes).update(counter).digest();
const offset = digest[digest.length - 1] & 15;
const value = ((digest[offset] & 127) << 24) | (digest[offset + 1] << 16) | (digest[offset + 2] << 8) | digest[offset + 3];
process.stdout.write(String(value % 1000000).padStart(6, "0"));
' "$two_factor_secret")
curl -fsS -X POST "$api_url/auth/2fa/confirm" -H "$authorization" -H 'Content-Type: application/json' --data "{\"code\":\"$two_factor_code\"}" >/dev/null
competition_response=$(curl -fsS -X POST "$api_url/competitions" -H "$authorization" -H 'Content-Type: application/json' --data '{"name":"Phase 3 API verification"}')
competition_id=$(node -e 'process.stdout.write(String(JSON.parse(process.argv[1]).id))' "$competition_response")

upload_project() {
  local name="$1"
  local file="$2"
  curl -fsS -X POST "$api_url/projects/upload" -H "$authorization" \
    -F "name=$name" \
    -F 'category=technology' \
    -F "competition_id=$competition_id" \
    -F "file=@$file;type=text/markdown"
}

first_project=$(upload_project 'Autonomous Agricultural Robotics' "$first_file")
second_project=$(upload_project 'Intelligent Farm Robot' "$second_file")
first_project_id=$(node -e 'process.stdout.write(String(JSON.parse(process.argv[1]).id))' "$first_project")
second_project_id=$(node -e 'process.stdout.write(String(JSON.parse(process.argv[1]).id))' "$second_project")

category_fit=$(curl -fsS -X POST "$api_url/projects/$second_project_id/category-fit" -H "$authorization")
similarity=$(curl -fsS -X POST "$api_url/projects/$second_project_id/similarity" -H "$authorization")
readiness=$(curl -fsS "$api_url/projects/$second_project_id/assessment-readiness" -H "$authorization")

node -e '
const [fit, similarity, readiness, firstId, secondId] = process.argv.slice(1).map((value, index) => index < 3 ? JSON.parse(value) : Number(value));
if (fit.project_id !== secondId || !fit.recommended_category) throw new Error("Category-fit response is incomplete");
if (similarity.project_id !== secondId || similarity.highest_similarity <= 0) throw new Error("Similarity response did not identify overlap");
if (!similarity.matches.some((match) => match.project_id === firstId)) throw new Error("Similarity response omitted the comparable project");
const categoryGate = readiness.checks.find((check) => check.key === "category_fit");
const similarityGate = readiness.checks.find((check) => check.key === "similarity");
if (categoryGate?.status !== "passed" || similarityGate?.status !== "passed") throw new Error("Assessment readiness gates are not passed");
' "$category_fit" "$similarity" "$readiness" "$first_project_id" "$second_project_id"

printf 'assessment-api-smoke-test: passed (projects %s and %s)\n' "$first_project_id" "$second_project_id"
