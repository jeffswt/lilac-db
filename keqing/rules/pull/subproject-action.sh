# external arguments
action_depends=${1}
artifacts=${2}
bin_basename=${3}
bin_cargo=${4}
bin_cat=${5}
bin_echo=${6}
bin_find=${7}
bin_git=${8}
bin_make=${9}
bin_mkdir=${10}
bin_readlink=${11}
bin_rustup=${12}
bin_tee=${13}
rules=${14}

arg_action=${15} # the performed action, also '.artifacts/[BUILD/RELEASE]/workspace/project/...'
arg_option=${16} # workspace / commit / branch
arg_param=${17}  # commit hash / branch name

# some parameters set by argument parser
repo_root="."        # root of target directory (containing subprojects)
output_root="."      # directory of artifact output
output_clone_root="" # clone artifact outputs to this folder when non-null
recipe_name=""       # identifier of the build target

# resolve target branch & commit, target repository root
# extract VCS snapshot, get recipe name
if [[ $arg_option == "workspace" ]]; then
    repo_root="."
    output_root="$artifacts/$arg_action/workspace"
    recipe_name="workspace"

    "$bin_mkdir" --parents "$output_root"

elif [[ $arg_option == "commit" ]]; then
    commit=$arg_param

    repo_root="$artifacts/git/workspace/$commit"
    output_root="$artifacts/$arg_action/commit/$commit"
    recipe_name="commit/$commit"

    "$bin_mkdir" --parents "$output_root"
    eval $action_depends "$artifacts/git/workspace/flags/$commit"

elif [[ $arg_option == "branch" ]]; then
    branch=$arg_param
    commit=$("$bin_git" log -n 1 $branch --pretty=format:"%H")

    repo_root="$artifacts/git/workspace/$commit"
    output_root="$artifacts/$arg_action/commit/$commit"
    output_clone_root="$artifacts/$arg_action/branch/$branch"
    recipe_name="branch/$branch@$commit"

    "$bin_mkdir" --parents "$output_root"
    "$bin_mkdir" --parents "$output_clone_root"
    eval $action_depends "$artifacts/git/workspace/flags/$commit"

else
    "$bin_echo" "keqing/subproject-action: Invalid arguments."
    exit 128
fi

# execute action on subprojects in given order
IFS=$'\n'
for project in $("$bin_cat" "$rules/pull/subproject-action-order.txt"); do
    # skip comments
    if [[ $project =~ ^\# ]]; then
        continue
    fi

    # resolve make arguments
    arg_subproj_root=$("$bin_readlink" --canonicalize "$repo_root/$project")
    arg_artifact_root=$("$bin_readlink" --canonicalize "$output_root/$project")
    arg_subproj_makefile=$("$bin_readlink" --canonicalize "$repo_root/$project/Makefile")
    arg_arguments_makefile=$("$bin_readlink" --canonicalize "$rules/arguments.mk")

    # create directories
    "$bin_mkdir" --parents "$output_root/$project" # not using the readlink here

    # check if recipe exists
    if [[ ! -f "$arg_subproj_makefile" ]]; then
        "$bin_echo" "Subproject '$project' does not contain a Makefile on '$recipe_name'."
        exit 1
    fi

    # perform action on this subproject
    "$bin_make" --silent --file="$rules/pull/subproject-action-header.mk" "$arg_action" _ARG_SUBPROJ_ROOT="$arg_subproj_root" _ARG_ARTIFACT_ROOT="$arg_artifact_root" _ARG_SUBPROJ_MAKEFILE="$arg_subproj_makefile" _ARG_ARGUMENTS_MAKEFILE="$arg_arguments_makefile"
    if [[ $? != 0 ]]; then
        "$bin_echo" "Subproject '$project' failed action '$arg_action' on '$recipe_name'."
        exit 1
    fi
done
