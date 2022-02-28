# Checks if pull request on a certain branch exists.

# external arguments
artifacts=${1}
branch=${2}
check_exists=${3}

# variables
filename="$artifacts/pull/request/$branch"

# respond according to different validation modes
if [[ $check_exists == "true" ]]; then
    if [[ ! -f $filename ]]; then
        echo "Pull request on branch '$branch' doesn't exist."
        exit 1
    fi
elif [[ $check_exists == "false" ]]; then
    if [[ -f $filename ]]; then
        echo "Pull request on branch '$branch' already exists."
        exit 1
    fi
else
    echo "Invalid argument '$check_exists'."
    exit 128
fi
