# external arguments
action_depends=${1}
artifacts=${2}
bin_basename=${3}
bin_cargo=${4}
bin_cat=${5}
bin_cp=${6}
bin_echo=${7}
bin_find=${8}
bin_git=${9}
bin_make=${10}
bin_mkdir=${11}
bin_readlink=${12}
bin_rm=${13}
bin_rustup=${14}
bin_tee=${15}
bin_touch=${16}
rules=${17}

arg_action=${18} # the performed action, also '.artifacts/[BUILD/RELEASE]/workspace/project/...'
arg_option=${19} # workspace / commit / branch
arg_param=${20}  # commit hash / branch name

# some parameters set by argument parser
repo_root="."        # root of target directory (containing subprojects)
output_root="."      # directory of artifact output
output_clone_root="" # clone artifact outputs to this folder when non-null
recipe_name=""       # identifier of the build target

lazy_flag_filename=".action_success" # mark action complete

# resolve target branch & commit, target repository root
# extract VCS snapshot, get recipe name
__parse_arguments() {
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
}

# checks if the original artifacts are already up-to-date (0) or not (1)
# targets should not be rebuilt if they are up-to-date
__original_artifacts_need_rebuild() {
    if [[ $arg_option == "workspace" ]]; then
        return 1
    elif [[ $arg_option == "commit" || $arg_option == "branch" ]]; then
        if [[ ! -f "$output_root/$lazy_flag_filename" ]]; then
            return 1
        fi
        return 0
    fi
    exit 128
}

# execute action on subprojects in given order
__build_targets() {
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

    # update lazy flag
    "$bin_touch" "$output_root/$lazy_flag_filename"
}

# validate if the cloned artifacts are up-to-date (0) or not (1 or else)
__cloned_artifacts_need_rebuild() {
    if [[ $arg_option == "workspace" || $arg_option == "commit" ]]; then
        return 0
    elif [[ $arg_option == "branch" ]]; then
        the_file="$output_clone_root/$lazy_flag_filename"
        if [[ ! -f "$the_file" ]]; then
            return 1
        fi
        if [[ $("$bin_cat" "$the_file") != "$commit" ]]; then
            return 1
        fi
        return 0
    fi
    exit 128
}

# clone output artifacts to the cloned artifacts' folder
__clone_targets() {
    if [[ $output_clone_root == "" ]]; then
        exit 128
    fi

    # reset target directory
    "$bin_rm" --recursive "$output_clone_root"
    if [[ $? != 0 ]]; then
        "$bin_echo" "Unable to perform cleanup '$output_clone_root' for outdated artifacts."
        exit 1
    fi
    "$bin_mkdir" --parents "$output_clone_root"
    if [[ $? != 0 ]]; then
        "$bin_echo" "Unable to perform directory reset on '$output_clone_root'."
        exit 1
    fi

    # copy contents
    "$bin_cp" --recursive --no-target-directory "$output_root" "$output_clone_root"
    if [[ $? != 0 ]]; then
        "$bin_echo" "Artifacts clone on '$output_clone_root' failed."
        exit 1
    fi

    # update lazy flag
    "$bin_rm" "$output_clone_root/$lazy_flag_filename"
    "$bin_echo" "$commit" > "$output_clone_root/$lazy_flag_filename"
}

# main procedure
__parse_arguments
__original_artifacts_need_rebuild
if [[ $? != 0 ]]; then
    __build_targets
fi
__cloned_artifacts_need_rebuild
if [[ $? != 0 ]]; then
    __clone_targets
fi
