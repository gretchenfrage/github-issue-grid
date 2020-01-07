#!/usr/bin/env bash

SCRIPT_DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_DIR}"

echo ""
echo "building SASS"
echo ""

sass sass/:static/ || exit 1

echo ""
echo "done building"
echo ""