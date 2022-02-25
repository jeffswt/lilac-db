# Exact archive of workspace at given commit

$(ARTIFACTS)/git/archive/%.tar:
	mkdir --parents "$(ARTIFACTS)/git/archive/"
	"$(BIN_GIT)" archive --format tar --output="$(ARTIFACTS)/git/archive/$*.tar" "$*"
