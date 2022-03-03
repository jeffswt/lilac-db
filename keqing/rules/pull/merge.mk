
$(ARTIFACTS)/pull/merge/%:
	@"$(BIN_PYTHON3)" "$(RULES)/style/commit-msg.py" --lint "$(ARG_COMMIT_MSG)" --requires-type="merge"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/pull/pr-gate/$*"

	@"$(BIN_ECHO)" "Switching to 'master' branch for merge..."
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/checkout/master"
	@"$(BIN_GIT)" merge --no-ff --commit --log -m "$(ARG_COMMIT_MSG)" "$*"

	@"$(BIN_ECHO)" "                                                        "
	@"$(BIN_ECHO)" "+-------------------------------------------------------"
	@"$(BIN_ECHO)" "|  Successfully merged branch '$*' into 'master'."
	@"$(BIN_ECHO)" "|  Please delete the source branch manually.            "
	@"$(BIN_ECHO)" "+-------------------------------------------------------"
	@"$(BIN_ECHO)" "                                                        "
