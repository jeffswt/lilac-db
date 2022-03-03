# Validate that the branch is ready for merge.
# The PR gate dependencies are actually checked on par of commits, which are
# stored in '$(ARTIFACTS)/pull/pr-gate/commit/%'.

# usage: @$(__PULL__PR_GATE_MSGBOX) [stage-id] [title] [message]
__PULL__PR_GATE_MSGBOX = "$(BIN_BASH)" "$(RULES)/pull/pr-gate-msgbox.sh" "$(BIN_ECHO)"

$(ARTIFACTS)/pull/pr-gate/%:
	@$(__PULL__PR_GATE_MSGBOX) "1" "Checkout" "Switching to branch '$*'..."
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/checkout/$*"
	@"$(BIN_ECHO)" "Successfully switched to branch '$*'."

	@$(__PULL__PR_GATE_MSGBOX) "2" "Merge Conflicts" "Checking for merge conflicts against master..."
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/no-merge-conflicts/$*"
	@"$(BIN_ECHO)" "There are no merge conflicts between '$*' and 'master'."

	@$(__PULL__PR_GATE_MSGBOX) "3" "Style: Commit Messages" "Commit messages should follow conventional commits standards."
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/style/branch-commit-msgs/$*"

	@$(__PULL__PR_GATE_MSGBOX) "4" "Build - Release" "Executing build scripts on subprojects..."
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/build/release/branch/$*"
	@"$(BIN_ECHO)" "All subprojects builds have completed successfully."

	@$(__PULL__PR_GATE_MSGBOX) "5" "Unit Test" "Performing unit tests on subprojects..."
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/test/unit/branch/$*"
	@"$(BIN_ECHO)" "Unit tests on all subprojects have completed with no fatal errors found."

	@$(__PULL__PR_GATE_MSGBOX) "6" "Style: Lint" "Linting subprojects for style errors..."
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/style/lint/branch/$*"
	@"$(BIN_ECHO)" "Passed all style checks on all subprojects."

	@$(__PULL__PR_GATE_MSGBOX) "7" "Complete" "Branch '$*' is safe to merge."
