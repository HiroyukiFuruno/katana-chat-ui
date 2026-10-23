#!/usr/bin/env bash
set -euo pipefail

event_name="${RELEASE_EVENT_NAME:-}"
repository="${RELEASE_REPOSITORY:-}"
source_sha="${RELEASE_SOURCE_SHA:-}"
workflow_ref="${RELEASE_REF:-}"
input_version="${RELEASE_INPUT_VERSION:-}"
script_dir="$(cd "$(dirname "$0")" && pwd)"

die() {
  echo "release source admission failed: $*" >&2
  exit 1
}

[[ -n "${event_name}" ]] || die "RELEASE_EVENT_NAME is required"
[[ -n "${repository}" ]] || die "RELEASE_REPOSITORY is required"
[[ -n "${source_sha}" ]] || die "RELEASE_SOURCE_SHA is required"

head_sha="$(git rev-parse HEAD)"
[[ "${head_sha}" == "${source_sha}" ]] || die "checked-out HEAD ${head_sha} does not match source SHA ${source_sha}"

if [[ "${event_name}" == "pull_request" ]]; then
  [[ "${RELEASE_PR_MERGED:-}" == "true" ]] || die "pull request is not merged"
  [[ "${RELEASE_PR_HEAD_REPOSITORY:-}" == "${repository}" ]] || die "pull request head repository is not the release repository"
  head_ref="${RELEASE_PR_HEAD_REF:-}"
  [[ "${head_ref}" == release/v* ]] || die "pull request head ref must be release/vX.Y.Z"
  merge_sha="${RELEASE_PR_MERGE_SHA:-}"
  [[ -n "${merge_sha}" && "${merge_sha}" == "${source_sha}" ]] || die "source SHA does not match pull request merge commit"
  version="${head_ref#release/}"
elif [[ "${event_name}" == "workflow_dispatch" ]]; then
  [[ "${workflow_ref}" == refs/tags/v* ]] || die "manual recovery must run from refs/tags/vX.Y.Z"
  version="${input_version}"
else
  die "unsupported event ${event_name}"
fi

if [[ -n "${version}" ]]; then
  version_output="$(bash "${script_dir}/verify-version.sh" "${version}")"
else
  version_output="$(bash "${script_dir}/verify-version.sh")"
fi
version="$(awk -F= '$1 == "version" { print $2 }' <<<"${version_output}")"
if [[ "${event_name}" == "workflow_dispatch" ]]; then
  [[ "${workflow_ref}" == "refs/tags/${version}" ]] || die "workflow ref ${workflow_ref} does not match release version ${version}"
fi
if [[ "${event_name}" == "workflow_dispatch" ]]; then
  tag_sha="$(git rev-parse -q --verify "refs/tags/${version}^{commit}" 2>/dev/null || true)"
  [[ -n "${tag_sha}" ]] || die "release tag ${version} is missing"
  [[ "${tag_sha}" == "${head_sha}" ]] || die "release tag ${version} does not match checked-out HEAD"
fi

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "version=${version}"
    echo "source_sha=${source_sha}"
  } >> "${GITHUB_OUTPUT}"
else
  echo "version=${version}"
  echo "source_sha=${source_sha}"
fi
