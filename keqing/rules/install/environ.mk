
$(ARTIFACTS)/install/environ:
	@# commit LF, checkout LF
	@"$(BIN_GIT)" config core.autocrlf input

	@# install Rust
	@echo "Select 'Proceed with installation (default)' on prompt."
	@"$(BIN_CURL)" --proto '=https' --tlsv1.2 -sSf "https://sh.rustup.rs" | "$(BIN_BASH)"

	@# add required Rust packages
	@"$(BIN_RUSTUP)" component add rustfmt
