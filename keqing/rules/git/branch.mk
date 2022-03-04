# Create branch based off from master (and checkout to that branch).

$(ARTIFACTS)/git/branch/%:
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/style/branch-name/$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/checkout/master"
	@"$(BIN_GIT)" checkout -b "$*"
