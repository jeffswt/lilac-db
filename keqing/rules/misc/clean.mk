
clean:
	@"$(BIN_MKDIR)" --parents "$(ARTIFACTS)"
	@$(ACTION_SUBPROJ) clean workspace
	@"$(BIN_RM)" --recursive --verbose "$(ARTIFACTS)"
	@"$(BIN_ECHO)" "All artifacts cleaned."
