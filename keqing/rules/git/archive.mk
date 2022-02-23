# Exact archive of workspace at given commit

$(ARTIFACTS)/git/archive/$(ARG_COMMIT_HASH).tar:
	mkdir --parents "$(ARTIFACTS)/git/archive/"
	$(BIN_GIT) archive --format tar --output="$(ARTIFACTS)/git/archive/$(ARG_COMMIT_HASH).tar" "$(ARG_COMMIT_HASH)"
