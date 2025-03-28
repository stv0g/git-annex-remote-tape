#!/bin/env bash

exec 3>&1 4>&2 >test.log 2>&1
set -e

PATH=$PATH:$(pwd)/target/debug

function header() {
    echo
    echo "========= $1"
}

if [ -d test-repo ]; then
    chmod u+w -R test-repo
    rm -rf test-repo
fi

mkdir test-repo
cd test-repo

header "Build code"
cargo build --bin git-annex-remote-tape

header "Initializing git repo"
git init

header "Initializing git-annex"
git annex -d init

header "Init git-annex remote"
git annex -d initremote tape1 type=external externaltype=tape drive=/dev/st0 encryption=none

header "Enable git-annex remote with settings"
git annex -d enableremote tape1 type=external externaltype=tape drive=/dev/st0 encryption=none

header "Enable git-annex remote without settings"
git annex -d enableremote tape1

header "Get infos from git-annex remote"
git annex -d info tape1

header "Create a test file"
echo "Hello World" > test.txt

header "Add test file to git-annex"
git annex -d add test.txt

header "Commit test file"
git commit -m "first file"

header "Move test file to tape1"
git annex -d move --to tape1 test.txt

header "Move test file to tape1 (again)"
git annex -d move --to tape1 test.txt