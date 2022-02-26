# Validate all Rust subprojects on the branch or commit.
# Dependencies are implicitly declared within the script.

$(ARTIFACTS)/style/rustfmt/commit/%.txt:
	@$(BIN_MKDIR) --parents "$(ARTIFACTS)/style/rustfmt/commit"
	@$(BIN_BASH) "$(RULES)/style/rustfmt.sh" commit "$*"

$(ARTIFACTS)/style/rustfmt/branch/%.txt:
	@$(BIN_MKDIR) --parents "$(ARTIFACTS)/style/rustfmt/branch"
	@$(BIN_BASH) "$(RULES)/style/rustfmt.sh" branch "$*"
