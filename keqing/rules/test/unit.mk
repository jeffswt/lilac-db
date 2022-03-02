# Perform unit tests on all subprojects on the workspace, specific commit or
# branch, as per subproject Makefile definitions.

$(ARTIFACTS)/test/unit/workspace: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) test/unit workspace

$(ARTIFACTS)/test/unit/commit/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) test/unit commit "$*"

$(ARTIFACTS)/test/unit/branch/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) test/unit branch "$*"
