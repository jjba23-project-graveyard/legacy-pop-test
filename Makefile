# pop-test - your test orchestration master of puppets

# Copyright (C) 2025 Josep Bigorra (jjba23) <jjbigorra@gmail.com>

# pop-test is free software: you can redistribute it and/or modify
# it under the terms of the Lesser GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# pop-test is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# Lesser GNU General Public License for more details.

# You should have received a copy of the Lesser GNU General Public License
# along with pop-test.  If not, see <https://www.gnu.org/licenses/>.

# pop-test-log-info: Logs informational messages in custom format with highlight.
define pop-test-log-info
	@printf "\n\033[1m[INFO] $(shell date --rfc-3339=seconds) - %s\033[0m\n" $(1)
endef

fmt:
	$(call pop-test-log-info,"formatting Rust source code")
	nix develop -c cargo fix --allow-dirty --bin "pop-test"
	nix develop -c cargo clippy --fix --allow-dirty
	nix develop -c find . -maxdepth 8 -path target -prune -name "*.rs" -type f -exec rustfmt --edition 2021 {} \;

# test: run unit and integration tests for pop-test's implementation itself
test: fmt
	$(call pop-test-log-info,"running Rust unit and integration tests")
	RUST_LOG=debug nix develop -c cargo test

# generate-json-schema
generate-json-schema: fmt
	$(call pop-test-log-info,"generating JSON schema for pop-test")
	RUST_LOG=debug nix develop -c cargo run -- --generate-json-schema=true

# publish-manual: Generates and publishes the manual from Org files using Emacs.
publish-manual:
	$(call pop-test-log-info,"generating manual for pop-test")
	cd docs/Manual/ && emacs -Q --batch \
		-l $(shell realpath ./src/manual/pop-test-manual.el) \
		--funcall pop-test/publish-manual && cd ../../	

#
# container management
#
container-build:
	$(call pop-test-log-info,"building container for pop-test")
	podman build -t jjba23/pop-test:$(CONTAINER_TAG) .
container-run:
	podman run jjba23/pop-test:$(CONTAINER_TAG)
container-push:
	podman push jjba23/pop-test:$(CONTAINER_TAG)


#
# example runs
#

define pop-test-golden-test-example
	@-make fmt
	$(call pop-test-log-info,"running scenario for: $(1)")
	TESTCONTAINERS_COMMAND=keep \
		RUST_LOG=debug \
		nix develop -c time -p cargo run -- -p ./resources/scenarios/$(1)/golden-test.yaml
endef

golden-test-green-energy-mix:
	$(call pop-test-golden-test-example,"green-energy-mix")
golden-test-vehicle-management:
	$(call pop-test-golden-test-example,"vehicle-management")
golden-test-dynamic-tariff-charging:
	$(call pop-test-golden-test-example,"dynamic-tariff-charging")
golden-test-consumer-solar-steering:
	$(call pop-test-golden-test-example,"consumer-solar-steering")
golden-test-lucidplan:
	$(call pop-test-golden-test-example,"lucidplan")
golden-test-wikimusic-api:
	$(call pop-test-golden-test-example,"wikimusic-api")
