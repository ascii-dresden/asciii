#!/usr/bin/env bash
l=$(git --no-pager --work-tree . --git-dir ./.git log --oneline "--format=%h")
COMMIT_NUM="$(echo $l | wc -w)"
COMMIT_SHORT_HASH="$(echo $l | awk '{print $1}')"
printf 'description string=build %i (%s)' "$COMMIT_NUM" "$COMMIT_SHORT_HASH" | tee .most_recent_commit