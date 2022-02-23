# Validates if the current workspace is clean for VCS operations.

if [ -n "$(git status --porcelain)" ]; then
    cur_branch=$(git branch --show-current)
    echo "There are still unstaged changes on '$cur_branch'."
    exit 1
else
    exit 0
fi
