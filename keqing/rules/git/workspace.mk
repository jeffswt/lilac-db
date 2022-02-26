# Decompress workspace content (archive) at temporary folder.
# Uses a flag to denote compression complete.

$(ARTIFACTS)/git/workspace/flags/$*: $(ARTIFACTS)/git/archive/$*.tar
	"$(BIN_MKDIR)" --parents "$(ARTIFACTS)/git/workspace/$*"
	"$(BIN_TAR)" --extract --verbose --file="$(ARTIFACTS)/git/archive/$*.tar" --directory="$(ARTIFACTS)/git/workspace/$*"
	"$(BIN_MKDIR)" --parents "$(ARTIFACTS)/git/workspace/flags"
	"$(BIN_TOUCH)" "$(ARTIFACTS)/git/workspace/flags/$*"
