
$(ARTIFACTS)/always-execute:
	@"$(BIN_MKDIR)" --parents "$(ARTIFACTS)"
	@"$(BIN_TOUCH)" "$(ARTIFACTS)/always-execute"

.PHONY: $(ARTIFACTS)/always-execute
