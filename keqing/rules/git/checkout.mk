# Move HEAD to target branch.

$(ARTIFACTS)/git/checkout/%:
	"$(BIN_BASH)" "$(RULES)/git/staged-all.sh"
	"$(BIN_GIT)" checkout "$*"
	"$(BIN_BASH)" "$(RULES)/git/staged-all.sh"
