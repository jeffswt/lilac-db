# Validate that the branch is ready for merge.
# The PR gate dependencies are actually checked on par of commits, which are
# stored in '$(ARTIFACTS)/pull/pr-gate/commit/%'.

$(ARTIFACTS)/pull/pr-gate/%:
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/checkout/$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/git/no-merge-conflicts/$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/style/branch-commit-msgs/$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/build/release/branch/$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/test/unit/branch/$*"
	@$(ACTION_DEPENDS) "$(ARTIFACTS)/style/lint/branch/$*"
