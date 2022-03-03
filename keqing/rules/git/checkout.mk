# Move HEAD to target branch. Hide messages if no errors were found.

$(ARTIFACTS)/git/checkout/%:
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/staged-all"
	@"$(BIN_BASH)" "$(RULES)/git/checkout.sh" "$(BIN_GIT)" "$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/staged-all"
