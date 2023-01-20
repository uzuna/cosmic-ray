
.PHONY: setup.py
setup.py:
	pip install maturin pytest black

.PHONY: build.dev
build.dev:
	maturin develop

.PHONY: test
test: build.dev
	pytest test --capture=no

.PHONY: fmt
fmt:
	black test
