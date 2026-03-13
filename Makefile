.PHONY: deploy undeploy

deploy: ## Build standalone `agt` binary and install to ~/.local/bin
	bun install
	bun build --compile src/cli/index.ts --outfile dist/agt
	install -d ~/.local/bin
	install dist/agt ~/.local/bin/agt

undeploy: ## Remove `agt` from ~/.local/bin
	rm -f ~/.local/bin/agt
