PHONY: fmt
fmt:
	cargo fmt
	git add -u
	cargo clippy --fix --allow-staged --all-features

.PHONY: check-fmt
check-fmt:
	cargo fmt --check
	cargo clippy
