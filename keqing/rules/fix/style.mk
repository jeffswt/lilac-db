# Fix linter errors as per project Makefile requirements.

$(ARTIFACTS)/fix/style/workspace: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) fix/style workspace
	@"$(BIN_ECHO)" "Style issues fixed on current workspace."
