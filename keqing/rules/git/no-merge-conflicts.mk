# Assert that the current branch may be safely merged into `master` with no
# merge conflicts.

$(ARTIFACTS)/git/no-merge-conflicts/%:
	@"$(BIN_BASH)" "$(RULES)/git/no-merge-conflicts.sh" "$(ACTION_DEPENDS)" "$(ARTIFACTS)" "$(BIN_ECHO)" "$(BIN_GIT)" "$(RULES)" "$*"
