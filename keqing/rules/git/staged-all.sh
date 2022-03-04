# external arguments
bin_echo=${1}
bin_git=${2}

if [ -n "$("$bin_git" status --porcelain)" ]; then
    cur_branch=$("$bin_git" branch --show-current)
    "$bin_echo" "There are still unstaged changes on '$cur_branch'."
    exit 1
else
    exit 0
fi
