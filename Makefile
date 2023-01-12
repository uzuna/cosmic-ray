PHONY: fmt
fmt:
	cargo fmt --all
	git add -u
	cargo clippy --fix --allow-staged --all-features

.PHONY: check-fmt
check-fmt:
	cargo fmt --check --all
	cargo clippy --all-features

.PHONY: test
test:
	cargo test --all-features

.PHONY: build
build:
	cargo build --release 

.PHONY: deb
deb: build
	cargo deb -- --features build-binary
