# Exact archive of workspace at given commit

$(ARTIFACTS)/git/archive/$(ARG_COMMIT_HASH).tar:
	git archive --format tar --output="$(ARTIFACTS)/git/archive/$(ARG_COMMIT_HASH).tar" $(ARG_COMMIT_HASH)
