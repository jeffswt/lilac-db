# external arguments
bin_cargo="cargo"
bin_git="git"
bin_rustup="rustup"

# resolve target commit hash (esp. with branch name)
if [[ $1 == "commit" ]]; then
    commit=$2
elif [[ $1 == "branch" ]]; then
    commit=$("$bin_git" log -n 1 $2 --pretty=format:"%H")
else
    echo "Invalid format targets."
    exit 1
fi

# filter target project directories in Rust only
for dir in ./*; do
    if [[ ! -f "$dir/Cargo.toml" ]]; then
        continue
    fi
    # set cwd to subproject
    CWD=$(pwd)
    cd $dir
    # # install rustfmt under project
    # "$bin_rustup" component add rustfmt
    # "$bin_cargo" fmt
    # "$bin_cargo" install --path .
    # run rustfmt under project
    "$bin_cargo" fmt --all --check
    # restore cwd pointer
    cd $CWD
done
