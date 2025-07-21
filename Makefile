help: ## Show this help message
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z0-9_-]+:.*##/ { printf "%-30s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

check:  ## cargo check
	cargo check

build:  ## cargo build
	cargo build

release:  ## cargo build --release
	cargo build --release

test_1: release ./target/release/jsonl-tools data/conversation_log-20250630-2312.json ## ./target/release/jsonl-tools --filename=data/conversation_log-20250630-2312.json
	./target/release/jsonl-tools --filename=data/conversation_log-20250630-2312.json
