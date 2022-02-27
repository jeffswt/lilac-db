# Validate all Rust subprojects on the branch or commit.
#   * Current workspace is always actively linted.
#   * Commits will be lazilly skipped iff all style checks passed on that node.
#   * Branches are referred to their latest commit.
# Dependencies are implicitly declared within the script.

$(ARTIFACTS)/style/rustfmt/workspace/report.txt:
	@$(BIN_MKDIR) --parents "$(ARTIFACTS)/style/rustfmt"
	@$(BIN_BASH) "$(RULES)/style/rustfmt.sh" "$(ACTION_DEPENDS)" "$(ARTIFACTS)" "$(BIN_BASENAME)" "$(BIN_CARGO)" "$(BIN_CAT)" "$(BIN_FIND)" "$(BIN_GIT)" "$(BIN_MKDIR)" "$(BIN_RUSTUP)" "$(BIN_TEE)" workspace

$(ARTIFACTS)/style/rustfmt/commit/%/report.txt:
	@$(BIN_MKDIR) --parents "$(ARTIFACTS)/style/rustfmt"
	@$(BIN_BASH) "$(RULES)/style/rustfmt.sh" "$(ACTION_DEPENDS)" "$(ARTIFACTS)" "$(BIN_BASENAME)" "$(BIN_CARGO)" "$(BIN_CAT)" "$(BIN_FIND)" "$(BIN_GIT)" "$(BIN_MKDIR)" "$(BIN_RUSTUP)" "$(BIN_TEE)" commit "$*"
	@$(BIN_TOUCH) "$(ARTIFACTS)/style/rustfmt/commit/$*/report.txt"

$(ARTIFACTS)/style/rustfmt/branch/%/report.txt:
	@$(BIN_MKDIR) --parents "$(ARTIFACTS)/style/rustfmt"
	@$(BIN_BASH) "$(RULES)/style/rustfmt.sh" "$(ACTION_DEPENDS)" "$(ARTIFACTS)" "$(BIN_BASENAME)" "$(BIN_CARGO)" "$(BIN_CAT)" "$(BIN_FIND)" "$(BIN_GIT)" "$(BIN_MKDIR)" "$(BIN_RUSTUP)" "$(BIN_TEE)" branch "$*"
