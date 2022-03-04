# Assert that the current branch may be safely merged into `master` with no
# merge conflicts. This implicitly switches the branch to the verified branch.

$(ARTIFACTS)/git/no-merge-conflicts/%:
	@"$(BIN_BASH)" "$(RULES)/git/no-merge-conflicts.sh" "$(ACTION_DEPENDS)" "$(ARTIFACTS)" "$(BIN_ECHO)" "$(BIN_GIT)" "$(RULES)" "$*"
	@"$(BIN_ECHO)" "There are no merge conflicts between '$*' and 'master'."
