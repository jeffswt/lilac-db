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

# extract commit (returns as a global variable `$commit`)
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

# extract commit message (returns as a global variable `$commit_msg`)
function __resolve_commit_msg() {
    set +e
    commit_msg=$(git log "$commit" -1 --pretty=format:"%s")
    retcode=$?
    set -e
    if [[ $retcode != 0 ]]; then
        echo "fatal: unable to resolve commit message on '$commit'"
        exit 1
    fi
}

# extract branch name (returns as global variables `$branch` and `$commit`)
function __resolve_branch() {
    set +e
    commit=$(git rev-parse "refs/heads/$1" 2>/dev/null)
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
    commit=$(git rev-parse "refs/heads/$branch" 2>/dev/null)
}

# display current HEAD to user, possible arguments:
#   - [description] 'workspace';
#   - [description] 'commit'; [Global variables: $commit]
#   - [description] 'branch'; [Global variables: $commit, $branch]
function __echo_current_head() {
    if [[ $2 == "workspace" ]]; then
        echo "$1 started on"
        echo "  - $cwd"
        echo "----------------------------------------------------------------"
    elif [[ $2 == "commit" ]]; then
        __resolve_commit_msg
        echo "$1 started on"
        echo "  - ${commit:0:8} $commit_msg"
        echo "----------------------------------------------------------------"
    elif [[ $2 == "branch" ]]; then
        __resolve_commit_msg
        echo "$1 started on"
        echo "  - $branch@${commit:0:8} $commit_msg"
        echo "----------------------------------------------------------------"
    else
        exit 128
    fi
}

###############################################################################
# Arguments

# help file
if [[ "$1 $2" == "help " ]]; then
    cat "./keqing/README.md"

# release related
elif [[ "$1 $2;$3" == "build release;" ]]; then
    __echo_current_head "Release build" workspace
    eval $keqing "$artifacts/build/release/workspace"
elif [[ "$1 $2 $3 $4 ... ;$6" == "build release on commit ... ;" ]]; then
    __resolve_commit "$5"
    __echo_current_head "Release build" commit
    eval $keqing "$artifacts/build/release/commit/$commit"
elif [[ "$1 $2 $3 $4 ... ;$6" == "build release on branch ... ;" ]]; then
    __resolve_branch "$5"
    __echo_current_head "Release build" branch
    eval $keqing "$artifacts/build/release/branch/$branch"

# autofix
elif [[ "$1 $2;$3" == "fix styles;" ]]; then
    __echo_current_head "Code style fix" workspace
    eval $keqing "$artifacts/fix/style/workspace"

# repository actions
elif [[ "$1 $2 ... ;$4" == "create branch ... ;" ]]; then
    eval $keqing "$artifacts/git/branch/$3"
elif [[ "$1 ... ;$3" == "checkout ... ;" ]]; then
    __resolve_branch "$2"
    eval $keqing "$artifacts/git/checkout/$branch"
elif [[ "$1 ... ;$3" == "commit ... ;" ]]; then
    eval $keqing "$artifacts/git/commit" "_ARG_COMMIT_MSG=\"$2\""
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
    __echo_current_head "Code check-in" branch
    eval $keqing "$artifacts/pull/merge/$branch" "_ARG_COMMIT_MSG=\"$4\""
elif [[ "$1 $2 $3;$4" == "run pr gate;" ]]; then
    __show_current_branch
    __echo_current_head "Code check-in validation" branch
    eval $keqing "$artifacts/pull/pr-gate/$branch"

# styles
elif [[ "$1 $2 $3;$4" == "check commit messages;" ]]; then
    __show_current_branch
    __echo_current_head "Commit message style checks" branch
    eval $keqing "$artifacts/style/branch-commit-msgs/$branch"
elif [[ "$1 $2 $3 $4 $5 ... ;$7" == "check commit messages on branch ... ;" ]]; then
    __resolve_branch "$6"
    __echo_current_head "Commit message style checks" branch
    eval $keqing "$artifacts/style/branch-commit-msgs/$branch"
elif [[ "$1 $2;$3" == "check styles;" ]]; then
    __echo_current_head "Code style checks" workspace
    eval $keqing "$artifacts/style/lint/workspace"
elif [[ "$1 $2 $3 $4 ... ;$6" == "check styles on commit ... ;" ]]; then
    __resolve_commit "$5"
    __echo_current_head "Code style checks" commit
    eval $keqing "$artifacts/style/lint/commit/$commit"
elif [[ "$1 $2 $3 $4 ... ;$6" == "check styles on branch ... ;" ]]; then
    __resolve_branch "$5"
    __echo_current_head "Code style checks" branch
    eval $keqing "$artifacts/style/lint/branch/$branch"

# tests
elif [[ "$1 $2;$3" == "run tests;" ]]; then
    __echo_current_head "Tests" workspace
    eval $keqing "$artifacts/test/unit/workspace"
elif [[ "$1 $2 $3 $4 ... ;$6" == "run tests on commit ... ;" ]]; then
    __resolve_commit "$5"
    __echo_current_head "Tests" commit
    eval $keqing "$artifacts/test/unit/commit/$commit"
elif [[ "$1 $2 $3 $4 ... ;$6" == "run tests on branch ... ;" ]]; then
    __resolve_branch "$5"
    __echo_current_head "Tests" branch
    eval $keqing "$artifacts/test/unit/branch/$branch"

# invalid arguments
else
    echo "fatal: invalid arguments (use 'help' to see instructions)"
    exit 128
fi
