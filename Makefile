.PHONY: default dev build web deploy undeploy dist test lint completions

default: completions ## Quick release build + install CLI (no web assets)
	cd src/rust && cargo build --release
	install src/rust/target/release/agt ~/.local/bin/agt
	-xattr -cr ~/.local/bin/agt 2>/dev/null

dev: ## Start Rust API server (auto-rebuild) + Vite dev server
	@bash -c '\
		trap "kill 0" EXIT; \
		cd src/rust && watchexec -r --stop-signal SIGKILL -w crates --exts rs,toml -- sh -c "cargo build 2>&1 && exec target/debug/agt serve" 2>&1 | grep -v "^Dashboard:" & \
		printf "Waiting for API server on :3000..."; \
		until curl -s http://localhost:3000/api/project > /dev/null 2>&1; do sleep 0.5; done; \
		printf " ready\n\n"; \
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

deploy: build completions ## Full release build + install with web assets
	install -d ~/.local/bin
	install src/rust/target/release/agt ~/.local/bin/agt
	-xattr -d com.apple.quarantine ~/.local/bin/agt 2>/dev/null
	rm -rf ~/.local/share/agt/web
	install -d ~/.local/share/agt
	cp -r src/web/dist ~/.local/share/agt/web

undeploy: ## Remove `agt` binary and data
	rm -f ~/.local/bin/agt
	rm -rf ~/.local/share/agt

completions: ## Install shell completions (fish, bash, zsh)
	install -d ~/.config/fish/completions
	install -m 644 completions/agt.fish ~/.config/fish/completions/agt.fish
	install -d ~/.local/share/bash-completion/completions
	install -m 644 completions/agt.bash ~/.local/share/bash-completion/completions/agt
	install -d ~/.zsh/completions
	install -m 644 completions/_agt ~/.zsh/completions/_agt
	@echo "Shell completions installed (fish, bash, zsh)"

dist: build ## Build distributable tarball for current platform
	$(eval TARGET := $(shell rustc -vV | grep host | cut -d' ' -f2))
	mkdir -p dist/agt-$(TARGET)
	cp src/rust/target/release/agt dist/agt-$(TARGET)/agt
	cp -r src/web/dist dist/agt-$(TARGET)/web
	cd dist/agt-$(TARGET) && tar -czf ../agt-$(TARGET).tar.gz .
	rm -rf dist/agt-$(TARGET)
	@echo "Built dist/agt-$(TARGET).tar.gz"
