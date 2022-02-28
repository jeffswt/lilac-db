# external arguments
bin_echo=$1
branch=$2

if [[ $branch =~ ^[0-9a-z]+(-[0-9a-z]+)*$ ]]; then
    exit 0
else
    "$bin_echo" "Branch name '$branch' must be consisted of lower-case alphanumerics separated with hyphens."
    exit 1
fi
