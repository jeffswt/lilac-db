if [[ $1 =~ ^[0-9a-z]+(-[0-9a-z]+)*$ ]]; then
    exit 0
else
    echo "Branch name '$1' must be consisted of lower-case alphanumerics separated with hyphens."
    exit 1
fi
