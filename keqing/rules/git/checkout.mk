# Move HEAD to target branch.

$(ARTIFACTS)/git/checkout/$(ARG_BRANCH):
	$(BIN_BASH) "$(RULES)/git/staged-all.sh"
	$(BIN_GIT) checkout "$(ARG_BRANCH)"
