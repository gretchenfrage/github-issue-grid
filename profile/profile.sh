#!/usr/bin/env bash

SCRIPT_DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_DIR}"
cd ..

export REPO="gretchenfrage/reflex"
export GITHUB_TOKEN=$(cat ../issue-cli-secret.secret)

cargo build --release --package github-issues-export --bin github-issues-export \
    || exit 1

(
    time (
        for (( c=1; c<=5; c++ ))
        do
            ./target/release/github-issues-export "${REPO}" || exit 1
        done
    ) || exit 1
) 3>&2 2>&1 1>&3 || exit 1

rm -rf ./md || exit 1
