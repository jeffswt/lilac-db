# Move HEAD to target branch.

$(ARTIFACTS)/git/checkout/%:
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/staged-all"
	@"$(BIN_GIT)" checkout "$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/staged-all"
