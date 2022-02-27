# external arguments
action_depends=${1}
artifacts=${2}
bin_basename=${3}
bin_cargo=${4}
bin_cat=${5}
bin_find=${6}
bin_git=${7}
bin_mkdir=${8}
bin_rustup=${9}
bin_tee=${10}

# some parameters set by argument parser
reporoot="."    # root of lint target directory (containing subprojects)
reportroot="."  # directory of rustfmt report output
lazylint="true" # set to 'true' to skip subproject if report already exists

# resolve target branch & commit, target repository root
# extract VCS snapshot, create test report dirs
if [[ ${11} == "workspace" ]]; then
    reporoot="."
    reportroot="$artifacts/style/rustfmt/workspace"
    lazylint="false"

    "$bin_mkdir" --parents "$reportroot"

elif [[ ${11} == "commit" ]]; then
    commit=${12}
    reporoot="$artifacts/git/workspace/$commit"
    reportroot="$artifacts/style/rustfmt/commit/$commit"
    lazylint="true"

    "$bin_mkdir" --parents "$reportroot"
    eval $action_depends "$artifacts/git/workspace/flags/$commit"

elif [[ ${11} == "branch" ]]; then
    branch=${12}
    commit=$("$bin_git" log -n 1 $branch --pretty=format:"%H")
    reporoot="$artifacts/git/workspace/$commit"
    reportroot="$artifacts/style/rustfmt/commit/$commit"
    lazylint="true"

    "$bin_mkdir" --parents "$artifacts/style/rustfmt/branch/$branch"
    "$bin_mkdir" --parents "$reportroot"
    eval $action_depends "$artifacts/git/workspace/flags/$commit"

else
    echo "keqing/rustfmt: Invalid format targets."
    exit 1
fi

# flag marking if all subprojects are styled ok
lintok="true"

# filter target project directories in Rust only
for dir in $reporoot/*; do
    project=$("$bin_basename" "$dir")
    if [[ ! -f "$dir/Cargo.toml" ]]; then
        continue
    fi
    # generate a report iff one does not exist
    reportfile="$reportroot/report-$project.txt"
    if [[ $lazylint == "false" || ! -f "$reportfile" ]]; then
        # (cd "$dir" && "$bin_rustup" component add rustfmt)
        # (cd "$dir" && "$bin_cargo" install --path .)
        (cd "$dir" && exec "$bin_cargo" fmt --all --check) | "$bin_tee" "$reportfile"
    else
        "$bin_cat" "$reportfile"
    fi
    # validate if format check succeeded
    thisok=$("$bin_find" "$reportroot" -empty -name "report-$project.txt")
    if [[ $thisok == "" ]]; then
        lintok="false"
    fi
done

# report error if lint failed
if [[ $lintok != "true" ]]; then
    echo "rustfmt: Certain projects had failed style tests."
    exit 1
fi
