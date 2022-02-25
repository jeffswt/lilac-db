# set parameters
bin_git=$1
bin_perl=$2
bin_python3=$3
git_branch=$4
path_rules=$5

# read commits diff from given branch to master
commits=$("$bin_git" cherry -v master "$git_branch")

# split commit list into lines
IFS=$'\n'
for commit in $commits; do
    # extract commit hash from log
    commit_hash=$(echo "$commit" | $bin_perl -pe 's/^\+ ([0-9a-f]{40}) .*?$/\1/g')
    # extract commit message from hash
    commit_msg=$(git log "$commit_hash" -1 --pretty=format:"%B")
    # validate conventional commit style
    $bin_python3 "$path_rules/style/commit-msg.py" --lint "$commit_msg"
    # exit on error
    if [ $? -ne 0 ]; then
        echo "Commit '$commit_hash' on branch '$git_branch' is badly styled."
        exit 1
    fi
done

echo "All commits on branch '$git_branch' are well-styled."
