# Perform unit tests on all subprojects on the workspace, specific commit or
# branch, as per subproject Makefile definitions.

$(ARTIFACTS)/test/unit/workspace: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) test/unit workspace
	@"$(BIN_ECHO)" "All unit tests passed on current workspace."

$(ARTIFACTS)/test/unit/commit/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) test/unit commit "$*"
	@"$(BIN_ECHO)" "All unit tests passed on commit '$*'."

$(ARTIFACTS)/test/unit/branch/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) test/unit branch "$*"
	@"$(BIN_ECHO)" "All unit tests passed on branch '$*'."
