# Validates if branch name satisfies style rule.

$(ARTIFACTS)/style/branch-name/%:
	@"$(BIN_BASH)" "$(RULES)/style/branch-name.sh" "$*"
