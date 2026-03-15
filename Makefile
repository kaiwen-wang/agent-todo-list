.PHONY: deploy undeploy build web

build: web ## Build Rust binary (release) and web assets
	cd agt-rs && cargo build --release

web: ## Build Vue frontend
	bun install
	bun run --cwd src/web build-only

deploy: build ## Build and install `agt` binary + web assets to ~/.local
	install -d ~/.local/bin
	install agt-rs/target/release/agt ~/.local/bin/agt
	rm -rf ~/.local/share/agt/web
	install -d ~/.local/share/agt
	cp -r src/web/dist ~/.local/share/agt/web

undeploy: ## Remove `agt` binary and data
	rm -f ~/.local/bin/agt
	rm -rf ~/.local/share/agt
