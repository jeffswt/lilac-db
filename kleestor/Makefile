
fix/style:
	@"$(BIN_ECHO)" "Fixing style issues on KleeStor..."
	@"$(BIN_CARGO)" fmt --all

build/release:
	@"$(BIN_ECHO)" "Building KleeStor to target 'release'..."
	# @"$(BIN_CARGO)" build
	# @"$(BIN_CARGO)" build 2>&1 | cat > "$(ARG_ARTIFACT_ROOT)/cargo_build.txt"

test/unit:
	@"$(BIN_ECHO)" "Executing unit tests on KleeStor..."
	@"$(BIN_CARGO)" test 2>&1 | tee "$(ARG_ARTIFACT_ROOT)/cargo_test.txt"

style/lint:
	@"$(BIN_ECHO)" "Linting subproject KleeStor..."
	@"$(BIN_CARGO)" fmt --all --check | "$(BIN_TEE)" "$(ARG_ARTIFACT_ROOT)/rustfmt.txt"

clean:
	@"$(BIN_ECHO)" "Cleaning temporary files on KleeStor..."
	-@"$(BIN_RM)" --recursive --verbose "./debug"
	-@"$(BIN_RM)" --recursive --verbose "./target"
	-@"$(BIN_RM)" --verbose "./Cargo.lock"
	-@"$(BIN_RM)" --verbose "./*.pdb"
