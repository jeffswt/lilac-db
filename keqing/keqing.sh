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

###############################################################################
# Arguments

if [[ "$1 $2 $3" == "build release " ]]; then
    eval $keqing "$artifacts/build/release/workspace"
elif [[ "$1 $2 $3 $4" == "build release on commit" ]]; then
    __resolve_commit "$5"
    eval $keqing "$artifacts/build/release/commit/$commit"
elif [[ "$1 $2 $3 $4" == "build release on branch" ]]; then
    __resolve_branch "$5"
    eval $keqing "$artifacts/build/release/branch/$branch"
fi

if [[ "$1 $2 $3" == "fix styles " ]]; then
    eval $keqing "$artifacts/fix/style/workspace"
fi

if [[ "$1" == "checkout" ]]; then
    __resolve_branch "$2"
    eval $keqing "$artifacts/git/checkout/$branch"
fi

if [[ "$1" == "commit" ]]; then
    eval $keqing "$artifacts/git/commit" _ARG_COMMIT_MSG="$2"
fi

if [[ "$1 $2 $3 $4" == "check merge conflicts " ]]; then
    branch=$(git branch --show-current)
    eval $keqing "$artifacts/git/no-merge-conflicts/$branch"
fi

if [[ "$1 $2 $3" == "setup environment " ]]; then
    eval $keqing "$artifacts/install/environ"
fi

if [[ "$1 $2" == "clean " ]]; then
    eval $keqing clean
fi

if [[ "$1 $2 $4 $5" == "merge branch with message" ]]; then
    __resolve_branch "$3"
    eval $keqing "$artifacts/pull/merge/$branch" _ARG_COMMIT_MSG="$6"
fi

if [[ "$1 $2 $3 $4" == "run pr gate " ]]; then
    branch=$(git branch --show-current)
    eval $keqing "$artifacts/pull/pr-gate/$branch"
fi

if [[ "$1 $2 $3 $4" == "check commit messages " ]]; then
    branch=$(git branch --show-current)
    eval $keqing "$artifacts/style/branch-commit-msgs/$branch"
fi

if [[ "$1 $2 $3" == "check styles " ]]; then
    branch=$(git branch --show-current)
    eval $keqing "$artifacts/style/lint/branch/$branch"
fi
