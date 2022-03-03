# Generate release builds to workspace, specific commit or branch (lazilly).

$(ARTIFACTS)/build/release/workspace: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) build/release workspace
	@"$(BIN_ECHO)" "Build complete on current workspace."

$(ARTIFACTS)/build/release/commit/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) build/release commit "$*"
	@"$(BIN_ECHO)" "Build complete on commit '$*'."

$(ARTIFACTS)/build/release/branch/%: $(ARTIFACTS)/always-execute
	@$(ACTION_SUBPROJ) build/release branch "$*"
	@"$(BIN_ECHO)" "Build complete on branch '$*'."
