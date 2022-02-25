# Validates all commit messages on the branch.

$(ARTIFACTS)/style/branch-commit-msgs/%: $(ARTIFACTS)/git/checkout/%
	"$(BIN_BASH)" "$(RULES)/style/branch-commit-msgs.sh" "$(BIN_GIT)" "$(BIN_PERL)" "$(BIN_PYTHON3)" "$*" "$(RULES)"
