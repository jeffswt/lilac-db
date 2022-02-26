# Create commit on current branch.

$(ARTIFACTS)/git/commit:
	@"$(BIN_PYTHON3)" "$(RULES)/style/commit-msg.py" --lint "$(ARG_COMMIT_MSG)"
	@"$(BIN_GIT)" commit --message "$(ARG_COMMIT_MSG)"
