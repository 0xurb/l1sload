# Cargo profile for builds. Default is for local builds, CI uses an override.
PROFILE ?= release

.PHONY: build
build:
	cargo build --profile "$(PROFILE)"

.PHONY: fix
fix:
	make fix-lint && \
	make fmt

fmt:
	cargo +nightly fmt --all

lint:
	make fmt && \
	cargo +nightly clippy \
		--all-features \
    	-- -D warnings

fix-lint:
	cargo +nightly clippy \
    	--workspace \
    	--lib \
    	--tests \
    	--features "$(FEATURES)" \
    	--fix \
    	--allow-staged \
    	--allow-dirty \
    	-- -D warnings

test:
	cargo test -- --nocapture
