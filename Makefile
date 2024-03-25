# install the required dependencies for fuzzing
install-fuzz-deps:
	# install the nightly toolchain for rust 1.77.0
	# https://github.com/tkaitchuck/aHash/issues/227
	# This is fixed but cosmwasm-std hasn't been updated to use the latest version
	@rustup install nightly-2024-01-21
	@cargo install cargo-fuzz

# lists all the available fuzz targets
fuzz-list:
	@cargo fuzz list

# Run the specified fuzz target
fuzz-run:
	@cargo +nightly-2024-01-21 fuzz run $(FUZZ_TARGET)

# Run the specified fuzz target
fuzz-run-timeout:
	@timeout $(TIME) cargo +nightly-2024-01-21 fuzz run $(FUZZ_TARGET)