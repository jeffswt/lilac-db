#!/bin/bash
# The 'human' interface for Keqing.

# switch cwd to repo root
function goto_repo_root() {
    cwd=$(git rev-parse --show-toplevel)
    cd "$cwd"
}
goto_repo_root

###############################################################################
# Common components

# stop on error
set -e

# execute rule from make
keqing="make --file=\"./keqing/rules/keqing.mk\""

# artifact path prefix
artifacts="./keqing/.artifacts"

# extract commit (returns as a global variable `$commit``)
function __resolve_commit() {
    set +e
    commit=$(git rev-parse "$1" 2>/dev/null)
    retcode=$?
    set -e
    if [[ $retcode != 0 ]]; then
        echo "fatal: '$1' is not a valid commit hash."
        exit 1
    fi
}

# extract branch name (returns as a global variable `$branch``)
function __resolve_branch() {
    set +e
    git rev-parse "refs/heads/$1" 1>/dev/null 2>/dev/null
    retcode=$?
    set -e
    if [[ $retcode != 0 ]]; then
        echo "fatal: '$1' is not a valid branch name."
        exit 1
    fi
    branch="$1"
}

# extracts current branch name
function __show_current_branch() {
    branch=$(git branch --show-current)
}

###############################################################################
# Arguments

# help file
if [[ "$1 $2" == "help " ]]; then
    cat "./keqing/README.md"

# release related
elif [[ "$1 $2;$3" == "build release;" ]]; then
    eval $keqing "$artifacts/build/release/workspace"
elif [[ "$1 $2 $3 $4 ... ;$6" == "build release on commit ... ;" ]]; then
    __resolve_commit "$5"
    eval $keqing "$artifacts/build/release/commit/$commit"
elif [[ "$1 $2 $3 $4 ... ;$6" == "build release on branch ... ;" ]]; then
    __resolve_branch "$5"
    eval $keqing "$artifacts/build/release/branch/$branch"

# autofix
elif [[ "$1 $2;$3" == "fix styles;" ]]; then
    eval $keqing "$artifacts/fix/style/workspace"

# repository actions
elif [[ "$1 $2 ... ;$4" == "create branch ... ;" ]]; then
    eval $keqing "$artifacts/git/branch/$3"
elif [[ "$1 ... ;$3" == "checkout ... ;" ]]; then
    __resolve_branch "$2"
    eval $keqing "$artifacts/git/checkout/$branch"
elif [[ "$1 ... ;$3" == "commit ... ;" ]]; then
    eval $keqing "$artifacts/git/commit" _ARG_COMMIT_MSG="$2"
elif [[ "$1 $2 $3;$4" == "check merge conflicts;" ]]; then
    __show_current_branch
    eval $keqing "$artifacts/git/no-merge-conflicts/$branch"

# misc actions
elif [[ "$1 $2;$3" == "setup environment;" ]]; then
    eval $keqing "$artifacts/install/environ"
elif [[ "$1;$2" == "clean;" ]]; then
    eval $keqing clean

# pull request related
elif [[ "$1 $2 $3 ... ;$5" == "merge with message ... ;" ]]; then
    __show_current_branch
    eval $keqing "$artifacts/pull/merge/$branch" _ARG_COMMIT_MSG="$4"
elif [[ "$1 $2 $3;$4" == "run pr gate;" ]]; then
    __show_current_branch
    eval $keqing "$artifacts/pull/pr-gate/$branch"

# styles
elif [[ "$1 $2 $3;$4" == "check commit messages;" ]]; then
    __show_current_branch
    eval $keqing "$artifacts/style/branch-commit-msgs/$branch"
elif [[ "$1 $2 $3 $4 $5 ... ;$7" == "check commit messages on branch;" ]]; then
    __resolve_branch "$6"
    eval $keqing "$artifacts/style/branch-commit-msgs/$branch"
elif [[ "$1 $2;$3" == "check styles;" ]]; then
    eval $keqing "$artifacts/style/lint/workspace"
elif [[ "$1 $2 $3 $4 ... ;$6" == "check styles on commit ... ;" ]]; then
    __resolve_commit "$5"
    eval $keqing "$artifacts/style/lint/commit/$commit"
elif [[ "$1 $2 $3 $4 ... ;$6" == "check styles on branch ... ;" ]]; then
    __resolve_branch "$5"
    eval $keqing "$artifacts/style/lint/branch/$branch"

# tests
elif [[ "$1 $2 $3;$4" == "run unit tests;" ]]; then
    eval $keqing "$artifacts/test/unit/workspace"
elif [[ "$1 $2 $3 $4 $5 ... ;$7" == "run unit tests on commit ... ;" ]]; then
    __resolve_commit "$6"
    eval $keqing "$artifacts/test/unit/commit/$commit"
elif [[ "$1 $2 $3 $4 $5 ... ;$7" == "run unit tests on branch ... ;" ]]; then
    __resolve_branch "$6"
    eval $keqing "$artifacts/test/unit/branch/$branch"

# invalid arguments
else
    echo "fatal: invalid arguments (use 'help' to see instructions)"
    exit 128
fi
