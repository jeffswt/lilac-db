# Create branch based off from master (and checkout to that branch).

$(ARTIFACTS)/git/branch/%: $(ARTIFACTS)/style/branch-name/% $(ARTIFACTS)/git/checkout/master
	$(BIN_GIT) checkout -b "$*"
