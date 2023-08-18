ERBOSE := $(if ${CI},--verbose,)

COMMIT := $(shell git rev-parse --short HEAD)

ifneq ("$(wildcard /usr/lib/librocksdb.so)","")
	SYS_LIB_DIR := /usr/lib
else ifneq ("$(wildcard /usr/lib64/librocksdb.so)","")
	SYS_LIB_DIR := /usr/lib64
else
	USE_SYS_ROCKSDB :=
endif

USE_SYS_ROCKSDB :=
SYS_ROCKSDB := $(if ${USE_SYS_ROCKSDB},ROCKSDB_LIB_DIR=${SYS_LIB_DIR},)

CARGO := env ${SYS_ROCKSDB} cargo

test:
	${CARGO} test ${VERBOSE} --all -- --skip trust_metric --nocapture

doc:
	cargo doc --all --no-deps

doc-deps:
	cargo doc --all

# generate GraphQL API documentation
doc-api:
	bash docs/build/gql_api.sh

check:
	${CARGO} check ${VERBOSE} --all

build:
	${CARGO} build ${VERBOSE} --release

check-fmt:
	cargo +nightly fmt ${VERBOSE} --all -- --check

fmt:
	cargo +nightly fmt ${VERBOSE} --all

clippy:
	${CARGO} clippy ${VERBOSE} --all --all-targets --all-features -- \
		-D warnings -D clippy::clone_on_ref_ptr -D clippy::enum_glob_use

sort:
	cargo sort -gw

check-sort:
	cargo sort -gwc

ci: check-fmt clippy test

info:
	date
	pwd
	env

e2e-test-lint:
	cd tests/e2e && yarn && yarn lint

e2e-test:
	cargo build
	rm -rf ./devtools/chain/data
	./target/debug/axon run --config devtools/chain/config.toml --genesis devtools/chain/genesis_single_node.json > /tmp/log 2>&1 &
	cd tests/e2e && yarn
	cd tests/e2e/src && yarn exec http-server &
	cd tests/e2e && yarn exec wait-on -t 5000 tcp:8000 && yarn exec wait-on -t 5000 tcp:8080 && yarn test
	pkill -2 axon
	pkill -2 http-server

e2e-test-ci:
	cargo build
	rm -rf ./devtools/chain/data
	./target/debug/axon run --config devtools/chain/config.toml --genesis devtools/chain/genesis_single_node.json > /tmp/log 2>&1 &
	cd tests/e2e && yarn
	cd tests/e2e/src && yarn exec http-server &
	cd tests/e2e && yarn exec wait-on -t 5000 tcp:8000 && yarn exec wait-on -t 5000 tcp:8080 && HEADLESS=true yarn test
	pkill -2 axon
	pkill -2 http-server

e2e-test-via-docker:
	docker-compose -f tests/e2e/docker-compose-e2e-test.yaml up --exit-code-from e2e-test --force-recreate

# For counting lines of code
stats:
	@cargo count --version || cargo +nightly install --git https://github.com/kbknapp/cargo-count
	@cargo count --separator , --unsafe-statistics

# Use cargo-audit to audit Cargo.lock for crates with security vulnerabilities
# expecting to see "Success No vulnerable packages found"
security-audit:
	@cargo audit --version || cargo install cargo-audit
	@cargo audit

metadata-test:
	cd builtin-contract/metadata \
		&& yarn --from-lock-file && yarn run compile && yarn run test

metadata-genesis-deploy:
	cd builtin-contract/metadata && yarn run deploy

crosschain-test:
	cd builtin-contract/crosschain \
		&& yarn --from-lock-file && yarn run compile && yarn run test

crosschain-genesis-deploy:
	cd builtin-contract/crosschain && yarn run deploy

unit-test: test metadata-test crosschain-test

schema:
	make -C core/cross-client/ schema

.PHONY: build prod prod-test
.PHONY: fmt test clippy doc doc-deps doc-api check stats
.PHONY: ci info security-audit
