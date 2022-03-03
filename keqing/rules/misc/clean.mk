
clean:
	@"$(BIN_MKDIR)" --parents "$(ARTIFACTS)"
	@"$(BIN_RM)" --recursive --verbose "$(ARTIFACTS)"
	@"$(BIN_ECHO)" "All artifacts are clean successfully."
