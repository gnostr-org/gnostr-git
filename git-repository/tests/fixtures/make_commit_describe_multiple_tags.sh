#!/bin/bash
set -eu -o pipefail

git init -q
git commit --allow-empty -q -m c1
git commit --allow-empty -q -m c2

git tag v0 -m "tag object 0" "HEAD~1"
git tag v2 -m "tag object 1"
git tag v2.5
GIT_COMMITTER_DATE="2022-01-02 00:00:00 +0000" git tag v3 -m "tag object 2"
GIT_COMMITTER_DATE="2022-01-02 00:00:00 +0000" git tag v4 -m "tag object 4"
GIT_COMMITTER_DATE="2022-01-03 00:00:00 +0000" git tag v1
