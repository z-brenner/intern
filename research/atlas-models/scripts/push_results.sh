#!/usr/bin/env bash
# Laboratory tooling for .github/workflows/atlas-research.yml; the app does not call it.
#
#   push_results.sh SRC_DIR REPO_PATH MESSAGE
#
# Commits SRC_DIR as REPO_PATH on the branch the workflow runs on, from a separate worktree so
# the build in the checkout is untouched, rebasing and retrying when another job of the same
# run pushed first. Results only: the workflow never pushes anywhere else.
set -euo pipefail
src="$1"
path="$2"
message="$3"
branch="${GITHUB_REF_NAME:?}"
git config --global user.name "github-actions[bot]"
git config --global user.email "41898341+github-actions[bot]@users.noreply.github.com"
wt="$RUNNER_TEMP/results-worktree"
for attempt in 1 2 3 4 5 6 7 8; do
  rm -rf "$wt"
  git worktree prune
  git fetch --quiet origin "$branch"
  git worktree add --quiet --detach "$wt" FETCH_HEAD
  mkdir -p "$wt/$path"
  cp -R "$src"/. "$wt/$path/"
  git -C "$wt" add "$path"
  if git -C "$wt" diff --cached --quiet; then
    echo "nothing new to commit"
    exit 0
  fi
  git -C "$wt" commit --quiet -m "$message"
  if git -C "$wt" push --quiet origin "HEAD:$branch"; then
    echo "pushed $path on attempt $attempt"
    exit 0
  fi
  sleep $(( (RANDOM % 20) + 5 * attempt ))
done
echo "could not push results after 8 attempts" >&2
exit 1
