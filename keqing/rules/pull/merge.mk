
$(ARTIFACTS)/pull/merge/%:
	@"$(BIN_PYTHON3)" "$(RULES)/style/commit-msg.py" --lint "$(ARG_COMMIT_MSG)" --requires-type="merge"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/pull/pr-gate/$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/checkout/master"
	@"$(BIN_GIT)" merge --no-ff --commit --log -m "$(ARG_COMMIT_MSG)" "$*"
