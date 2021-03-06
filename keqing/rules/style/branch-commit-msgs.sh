# set parameters
bin_echo=${1}
bin_git=${2}
bin_perl=${3}
bin_python3=${4}

branch=${5}
path_rules=${6}

# read commits diff from given branch to master
if [[ $branch != "master" ]]; then
    commits=$("$bin_git" cherry -v master "$branch")
    if [[ $? != 0 ]]; then
        exit 1
    fi
# master shows all commits
elif [[ $branch == "master" ]]; then
    commits=$("$bin_git" log --reverse --pretty=format:"+ %H %s" "$branch")
    if [[ $? != 0 ]]; then
        exit 1
    fi
fi

# mark an error here if something fails
lintok="true"

# split commit list into lines
IFS=$'\n'
for commit in $commits; do
    # extract commit hash from log
    commit_hash=$("$bin_echo" "$commit" | "$bin_perl" -pe 's/^\+ ([0-9a-f]{40}) .*?$/\1/g')
    # extract commit message from hash
    commit_msg=$("$bin_git" log "$commit_hash" -1 --pretty=format:"%B")
    # validate conventional commit style
    "$bin_python3" "$path_rules/style/commit-msg.py" --lint "$commit_msg"
    # mark error flag on error
    if [ $? -ne 0 ]; then
        lintok="false"
        "$bin_echo" "Commit '$commit_hash' on branch '$branch' is badly styled."
    else # or just print a notice
        "$bin_echo" "${commit_hash:0:8}: $commit_msg [OK]"
    fi
done

if [[ $lintok == "true" ]]; then
    "$bin_echo" "All commits on branch '$branch' are well-styled."
    exit 0
else
    exit 1
fi
