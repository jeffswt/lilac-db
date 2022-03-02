
$(ARTIFACTS)/build/release/workspace: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) build/release workspace

$(ARTIFACTS)/build/release/commit/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) build/release commit "$*"

$(ARTIFACTS)/build/release/branch/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) build/release branch "$*"
