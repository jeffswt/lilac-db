
$(ARTIFACTS)/install/environ:
	@"$(BIN_CURL)" --proto '=https' --tlsv1.2 -sSf "https://sh.rustup.rs" | "$(BIN_BASH)"
