
.PHONY: build.dev
build.dev:
	maturin develop

.PHONY: test
test: build.dev
	pytest test --capture=no
