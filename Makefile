.PHONY: deploy undeploy

deploy: ## Build standalone `agt` binary and web assets to ~/.local
	bun install
	bun run --cwd src/web build-only
	bun build --compile src/cli/index.ts --outfile dist/agt
	install -d ~/.local/bin
	install dist/agt ~/.local/bin/agt
	rm -rf ~/.local/share/agt/web
	install -d ~/.local/share/agt
	cp -r src/web/dist ~/.local/share/agt/web

undeploy: ## Remove `agt` binary and data
	rm -f ~/.local/bin/agt
	rm -rf ~/.local/share/agt
