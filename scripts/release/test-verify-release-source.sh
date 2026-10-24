#!/usr/bin/env bash
set -euo pipefail

# GitHub Actionsや呼び出し元のGIT_*がfixtureの別repositoryを向くと、
# fixture初期化のconfig/add/commitが外部indexやconfigを変更するため、検査自身のroutingを破棄する。
while IFS= read -r git_env; do
  unset "${git_env}"
done < <(compgen -A variable GIT_)

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
guard="${repo_root}/scripts/release/verify-release-source.sh"
mkdir -p "${repo_root}/tmp/verification"
fixture="$(mktemp -d "${repo_root}/tmp/verification/release-source-fixture.XXXXXX")"

git -C "${fixture}" init -q
fixture_root="$(git -C "${fixture}" rev-parse --show-toplevel)"
[[ "${fixture_root}" == "${fixture}" ]] || {
  echo "fixture Git root mismatch: ${fixture_root}" >&2
  exit 1
}
git -C "${fixture}" config user.email test@example.invalid
git -C "${fixture}" config user.name release-test
cat > "${fixture}/Cargo.toml" <<'EOF'
[package]
name = "release-source-fixture"
version = "0.1.0"
edition = "2021"
EOF
git -C "${fixture}" add Cargo.toml
git -C "${fixture}" commit -q -m initial
git -C "${fixture}" tag -a v0.1.0 -m release
old_sha="$(git -C "${fixture}" rev-parse HEAD)"
output="${fixture}/github-output"

run_guard() {
  : > "${output}"
  (cd "${fixture}" && env \
    GITHUB_OUTPUT="${output}" \
    RELEASE_EVENT_NAME="${RELEASE_EVENT_NAME:-}" \
    RELEASE_REPOSITORY="${RELEASE_REPOSITORY:-example/repo}" \
    RELEASE_SOURCE_SHA="${RELEASE_SOURCE_SHA:-${old_sha}}" \
    RELEASE_REF="${RELEASE_REF:-refs/tags/v0.1.0}" \
    RELEASE_INPUT_VERSION="${RELEASE_INPUT_VERSION:-}" \
    RELEASE_PR_MERGED="${RELEASE_PR_MERGED:-}" \
    RELEASE_PR_HEAD_REPOSITORY="${RELEASE_PR_HEAD_REPOSITORY:-}" \
    RELEASE_PR_HEAD_REF="${RELEASE_PR_HEAD_REF:-}" \
    RELEASE_PR_MERGE_SHA="${RELEASE_PR_MERGE_SHA:-}" \
    bash "${guard}")
}

expect_success() { run_guard >/dev/null; }
expect_failure() { if run_guard >/dev/null 2>&1; then echo "expected failure" >&2; exit 1; fi; }

RELEASE_EVENT_NAME=workflow_dispatch RELEASE_INPUT_VERSION=v0.1.0 expect_success
grep -Fx "version=v0.1.0" "${output}"
grep -Fx "source_sha=${old_sha}" "${output}"
RELEASE_EVENT_NAME=workflow_dispatch RELEASE_INPUT_VERSION='' expect_success
RELEASE_EVENT_NAME=workflow_dispatch RELEASE_REF=refs/heads/release/v0.1.0 expect_failure
RELEASE_EVENT_NAME=workflow_dispatch RELEASE_SOURCE_SHA=deadbeef expect_failure
RELEASE_EVENT_NAME=workflow_dispatch RELEASE_REF=refs/tags/v0.1.1 RELEASE_INPUT_VERSION=v0.1.0 expect_failure
RELEASE_EVENT_NAME=workflow_dispatch RELEASE_REF=refs/tags/v0.1.1 RELEASE_INPUT_VERSION=v0.1.1 expect_failure

cat > "${fixture}/Cargo.toml" <<'EOF'
[package]
name = "release-source-fixture"
version = "0.1.1"
edition = "2021"
EOF
git -C "${fixture}" add Cargo.toml
git -C "${fixture}" commit -q -m missing-tag
missing_sha="$(git -C "${fixture}" rev-parse HEAD)"
RELEASE_EVENT_NAME=workflow_dispatch RELEASE_SOURCE_SHA="${missing_sha}" RELEASE_REF=refs/tags/v0.1.1 RELEASE_INPUT_VERSION='' expect_failure
git -C "${fixture}" switch --detach -q "${old_sha}"

cat > "${fixture}/marker" <<'EOF'
wrong-tag-commit
EOF
git -C "${fixture}" add marker
git -C "${fixture}" commit -q -m wrong-tag
wrong_tag_sha="$(git -C "${fixture}" rev-parse HEAD)"
RELEASE_EVENT_NAME=workflow_dispatch RELEASE_SOURCE_SHA="${wrong_tag_sha}" RELEASE_REF=refs/tags/v0.1.0 RELEASE_INPUT_VERSION=v0.1.0 expect_failure
git -C "${fixture}" switch --detach -q "${old_sha}"

RELEASE_EVENT_NAME=pull_request RELEASE_PR_MERGED=true RELEASE_PR_HEAD_REPOSITORY=example/repo RELEASE_PR_HEAD_REF=release/v0.1.0 RELEASE_PR_MERGE_SHA="${old_sha}" expect_success
RELEASE_EVENT_NAME=pull_request RELEASE_PR_MERGED=false RELEASE_PR_HEAD_REPOSITORY=example/repo RELEASE_PR_HEAD_REF=release/v0.1.0 RELEASE_PR_MERGE_SHA="${old_sha}" expect_failure
RELEASE_EVENT_NAME=pull_request RELEASE_PR_MERGED=true RELEASE_PR_HEAD_REPOSITORY=example/repo RELEASE_PR_HEAD_REF=feature/v0.1.0 RELEASE_PR_MERGE_SHA="${old_sha}" expect_failure
RELEASE_EVENT_NAME=pull_request RELEASE_PR_MERGED=true RELEASE_PR_HEAD_REPOSITORY=fork/repo RELEASE_PR_HEAD_REF=release/v0.1.0 RELEASE_PR_MERGE_SHA="${old_sha}" expect_failure
RELEASE_EVENT_NAME=pull_request RELEASE_PR_MERGED=true RELEASE_PR_HEAD_REPOSITORY=example/repo RELEASE_PR_HEAD_REF=release/v0.1.0 RELEASE_PR_MERGE_SHA=deadbeef expect_failure
RELEASE_EVENT_NAME=push expect_failure

echo "release source admission regression tests passed"
echo "fixture=${fixture}"
