# Validates if the current workspace is clean for VCS operations.

$(ARTIFACTS)/git/staged-all:
	@"$(BIN_BASH)" "$(RULES)/git/staged-all.sh" "$(BIN_ECHO)" "$(BIN_GIT)"
