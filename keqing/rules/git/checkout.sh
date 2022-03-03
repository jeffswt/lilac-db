# external arguments
bin_git=${1}

branch=${2}

# silently execute command, unless errors were found
git_error=$("$bin_git" checkout "$branch" 2>&1 1>/dev/null)
git_return=$?
if [[ $git_return != 0 ]]; then
    echo "$git_error" 1>&2
    exit $git_return
fi
