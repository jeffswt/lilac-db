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
