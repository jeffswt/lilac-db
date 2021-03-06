# Check all subprojects for linter errors on the workspace, specific commit or
# branch, as per subproject Makefile definitions.

$(ARTIFACTS)/style/lint/workspace: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) style/lint workspace
	@"$(BIN_ECHO)" "Passed all style checks on current workspace."

$(ARTIFACTS)/style/lint/commit/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) style/lint commit "$*"
	@"$(BIN_ECHO)" "Passed all style checks on commit '$*'."

$(ARTIFACTS)/style/lint/branch/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) style/lint branch "$*"
	@"$(BIN_ECHO)" "Passed all style checks on branch '$*'."
