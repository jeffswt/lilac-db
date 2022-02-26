# Exact archive of workspace at given commit

$(ARTIFACTS)/git/archive/%.tar:
	@"$(BIN_MKDIR)" --parents "$(ARTIFACTS)/git/archive/"
	@"$(BIN_GIT)" archive --format tar --output="$(ARTIFACTS)/git/archive/$*.tar" "$*"
