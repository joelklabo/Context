#!/usr/bin/env bash
set -euo pipefail

LOCK_FILE=".git/context-runner.lock"

if [ $# -lt 1 ]; then
  echo "usage: scripts/runner.sh \"commit message\""
  exit 1
fi

(
  flock -n 9 || { echo "Another agent is committing, retry later."; exit 1; }

  git fetch origin || true
  git rebase origin/$(git rev-parse --abbrev-ref HEAD) || true

  make ci

  git add -A
  git commit -m "$1"
  git push origin HEAD

) 9>"${LOCK_FILE}"
