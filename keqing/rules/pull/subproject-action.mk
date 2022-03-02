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
#   * fix-style: automatically fix style issues.
#   * build-release: initiate a full build targeting release.
#   * test: validate codebase quality.
#   * lint: check for style errors.
#
# This (sub-)makefile contains a shortcut for other make rules to use the
# script easier, usage as:
#
#     $(ACTION_SUBPROJ) [action] [identifier] [target] [location]
#
# Whereas:
#
#   * `action` is one of the aforementioned actions, and must be defined in
#     all subprojects' makefiles.
#   * `identifier` is a component in the artifact output directory, e.g. the
#     subproject 'keqing' would have its built release binaries located under
#     `~/keqing/.artifacts/build/release/branch/master`, then the `identifier`
#     should have been `build/release`.
#   * `target` is one of `workspace`, `commit` or `branch`.
#   * `location` is the actual commit hash or branch name should `target` be
#     either of them.

__PULL__SUBPROJECT_ACTION = "$(BIN_BASH)" "$(RULES)/pull/subproject-action.sh" "$(ACTION_DEPENDS)" "$(ARTIFACTS)" "$(BIN_BASENAME)" "$(BIN_CARGO)" "$(BIN_CAT)" "$(BIN_ECHO)" "$(BIN_FIND)" "$(BIN_GIT)" "$(BIN_MAKE)" "$(BIN_MKDIR)" "$(BIN_READLINK)" "$(BIN_RUSTUP)" "$(BIN_TEE)" "$(RULES)"
