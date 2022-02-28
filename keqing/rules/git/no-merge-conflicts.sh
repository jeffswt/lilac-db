# external arguments
action_depends=$1
artifacts=$2
bin_echo=$3
bin_git=$4
rules=$5

branch=$6

# clean workspace and checkout
eval $action_depends "$artifacts/git/staged-all"
if [[ $? != 0 ]]; then
    exit 1
fi
"$bin_git" checkout "$branch"
if [[ $? != 0 ]]; then
    "$bin_echo" "Failed to checkout to branch '$branch'."
    exit 1
fi
eval $action_depends "$artifacts/git/staged-all"
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
