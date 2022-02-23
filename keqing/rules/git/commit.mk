# Create commit on current branch.

$(ARTIFACTS)/git/commit:
	git commit --message $(ARG_COMMIT_MSG)
