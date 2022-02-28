# Assert that the current branch may be safely merged into `master` with no
# merge conflicts.

# external arguments
bin_bash=$1
bin_echo=$2
bin_git=$3
rules=$4
branch=$5

# clean workspace and checkout
"$bin_bash" "$rules/git/staged-all.sh"
if [[ $? != 0 ]]; then
    exit 1
fi
"$bin_git" checkout "$branch"
if [[ $? != 0 ]]; then
    "$bin_echo" "Failed to checkout to branch '$branch'."
    exit 1
fi
"$bin_bash" "$rules/git/staged-all.sh"
if [[ $? != 0 ]]; then
    exit 1
fi

# abort previous merge
abort_merge=$("$bin_git" merge --abort 2>&1)
if [[ $? != 0 && $abort_merge != *"MERGE_HEAD missing"* ]]; then
    "$bin_echo" "There are still merges in progress."
    exit 1
fi

# check if the branches are safe to merge
"$bin_git" merge master --no-ff --no-commit
if [[ $? != 0 ]]; then
    "$bin_echo" "                                                                    "
    "$bin_echo" "    +---------------------------------------------------------------"
    "$bin_echo" "    |  [!] Branch '$branch' has conflicts with 'master'.            "
    "$bin_echo" "    |      Please resolve the merge conflicts before proceeding.    "
    "$bin_echo" "    |                                                               "
    "$bin_echo" "    |      Conflicts are preserved to be fixed manually.            "
    "$bin_echo" "    +---------------------------------------------------------------"
    "$bin_echo" "                                                                    "
    exit 1
fi

# aborting the merge anyway (you'll need to manually merge it)
"$bin_git" merge --abort
if [[ $? != 0 ]]; then
    "$bin_echo" "Merge abort failed (this normally shouldn't happen)!"
    exit 1
fi
