.PHONY: default dev build web deploy undeploy dist test lint

default: ## Quick release build + install CLI (no web assets)
	cd src/rust && cargo build --release
	install src/rust/target/release/agt ~/.local/bin/agt
	-xattr -cr ~/.local/bin/agt 2>/dev/null

dev: ## Start Rust API server + Vite dev server
	@bash -c '\
		trap "kill 0" EXIT; \
		src/rust/target/debug/agt serve & \
		until curl -s http://localhost:3000/api/project > /dev/null 2>&1; do sleep 0.2; done; \
		cd src/web && bunx vite \
	'

build: web ## Build Rust binary (release) and web assets
	cd src/rust && cargo build --release

web: ## Build Vue frontend
	bun install
	bun run --cwd src/web build-only

test: ## Run Rust tests
	cd src/rust && cargo test

lint: ## Run oxlint + oxfmt
	bunx oxlint src/web/src --fix
	bunx oxfmt src/web/src

deploy: build ## Full release build + install with web assets
	install -d ~/.local/bin
	install src/rust/target/release/agt ~/.local/bin/agt
	-xattr -d com.apple.quarantine ~/.local/bin/agt 2>/dev/null
	rm -rf ~/.local/share/agt/web
	install -d ~/.local/share/agt
	cp -r src/web/dist ~/.local/share/agt/web

undeploy: ## Remove `agt` binary and data
	rm -f ~/.local/bin/agt
	rm -rf ~/.local/share/agt

dist: build ## Build distributable tarball for current platform
	$(eval TARGET := $(shell rustc -vV | grep host | cut -d' ' -f2))
	mkdir -p dist/agt-$(TARGET)
	cp src/rust/target/release/agt dist/agt-$(TARGET)/agt
	cp -r src/web/dist dist/agt-$(TARGET)/web
	cd dist/agt-$(TARGET) && tar -czf ../agt-$(TARGET).tar.gz .
	rm -rf dist/agt-$(TARGET)
	@echo "Built dist/agt-$(TARGET).tar.gz"
