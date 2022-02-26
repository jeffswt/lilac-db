
$(ARTIFACTS)/install/environ:
	@echo "Select 'Proceed with installation (default)' on prompt."
	@"$(BIN_CURL)" --proto '=https' --tlsv1.2 -sSf "https://sh.rustup.rs" | "$(BIN_BASH)"
	@"$(BIN_RUSTUP)" component add rustfmt
