# Batch perform actions on subprojects, regarding their `Makefile`s.
#
#   * Builds, tests, etc. performed on the current workspace should not be
#     preserved as artifact outputs. They are being controlled by the
#     underlying toolchain.
#   * Each commit would be executed (successfully) for at most once. Once
#     completed, a flag would be set under that folder, preventing further
#     redundant actions. Artifacts are preserved.
#   * Actions performed on the branch are proxied to commits. Copy actions are
#     performed (from the commit folder to the branch folder) if the branch
#     has a stale artifact cache. The HEAD commit hash would be stored into
#     the branch flag.
#
# Subprojects contain `Makefile`s in their directory roots which describe the
# following actions:
#
#   * fix/style: automatically fix style issues.
#   * build/release: initiate a full build targeting release.
#   * test/unit: validate codebase quality.
#   * style/lint: check for style errors.
#
# This (sub-)makefile contains a shortcut for other make rules to use the
# script easier, usage as:
#
#     $(ACTION_SUBPROJ) [action] [target] [location]
#
# Whereas:
#
#   * `action` is one of the aforementioned actions, and must be defined in
#     all subprojects' makefiles. This also makes up the artifact output
#     directory name, e.g. the action `test/unit` would create a directory
#     `~/keqing/.artifacts/test/unit/branch/master`.
#   * `target` is one of `workspace`, `commit` or `branch`.
#   * `location` is the actual commit hash or branch name should `target` be
#     either of them.

__PULL__SUBPROJECT_ACTION = "$(BIN_BASH)" "$(RULES)/pull/subproject-action.sh" "$(ACTION_DEPENDS)" "$(ARTIFACTS)" "$(BIN_BASENAME)" "$(BIN_CARGO)" "$(BIN_CAT)" "$(BIN_CP)" "$(BIN_ECHO)" "$(BIN_FIND)" "$(BIN_GIT)" "$(BIN_MAKE)" "$(BIN_MKDIR)" "$(BIN_READLINK)" "$(BIN_RM)" "$(BIN_RUSTUP)" "$(BIN_TEE)" "$(BIN_TOUCH)" "$(RULES)"

$(ARTIFACTS)/pull/build/workspace/flag:
	@$(ACTION_SUBPROJ) build/release workspace
