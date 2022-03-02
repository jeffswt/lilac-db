# Declares a phony target in which being relied upon will ensure the caller
# being always built.

$(ARTIFACTS)/always-execute:
	@"$(BIN_MKDIR)" --parents "$(ARTIFACTS)"
	@"$(BIN_TOUCH)" "$(ARTIFACTS)/always-execute"

.PHONY: $(ARTIFACTS)/always-execute
