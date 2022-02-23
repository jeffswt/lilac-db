# Decompress workspace content (archive) at temporary folder.
# Uses a flag to denote compression complete.

$(ARTIFACTS)/git/workspace/flags/$(ARG_COMMIT_HASH): $(ARTIFACTS)/git/archive/$(ARG_COMMIT_HASH).tar
	mkdir --parents "$(ARTIFACTS)/git/workspace/$(ARG_COMMIT_HASH)"
	$(BIN_TAR) --extract --verbose --file="$(ARTIFACTS)/git/archive/$(ARG_COMMIT_HASH).tar" --directory="$(ARTIFACTS)/git/workspace/$(ARG_COMMIT_HASH)"
	mkdir --parents "$(ARTIFACTS)/git/workspace/flags"
	touch "$(ARTIFACTS)/git/workspace/flags/$(ARG_COMMIT_HASH)"
