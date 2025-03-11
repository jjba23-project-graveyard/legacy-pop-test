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

# pop-test is a tool that helps test computer programs,
# in integration, e2e, black-box, and other ways,
# in an easy and future-proof manner that improves reliability and engineering freedom.

# pop-test is your test orchestration master of puppets

# ====== logging ======

# log-info: Logs informational messages in custom format with highlight.
define log-info
	@printf "\n\033[1m[INFO] $(shell date --rfc-3339=seconds) - %s\033[0m\n" $(1)
endef

# ====== formatting ======

fmt:
	$(call log-info,"formatting Rust source code")
	nix develop -c cargo fix --allow-dirty
	nix develop -c cargo clippy --fix --allow-dirty
	nix develop -c find . -maxdepth 8 -path target -prune -name "*.rs" -type f -exec rustfmt --edition 2021 {} \;

# ====== unit testing ======

# test: run unit and integration tests for pop-test's implementation itself
test: fmt
	$(call log-info,"running Rust unit and integration tests")
	RUST_LOG=debug nix develop -c cargo test

# ====== documentation and manuals ======

# generate-json-schema: Generates a JSON/YAML schema with Rust code, which allows LSP checks in JSON/YAML files
generate-json-schema: fmt
	$(call log-info,"generating JSON schema for pop-test")
	RUST_LOG=debug nix develop -c cargo run -- --generate-json-schema=true

# publish-manual: Generates and publishes the manual from Org files using Emacs.
publish-manual:
	$(call log-info,"generating manual for pop-test")
	cd docs/Manual/ && emacs -Q --batch \
		-l $(shell realpath ./src/manual/pop-test-manual.el) \
		--funcall pop-test/publish-manual
	$(call log-info,"generating README for pop-test")
	emacs -Q --batch \
		-l $(shell realpath ./src/manual/pop-test-manual.el) \
		--funcall pop-test/publish-readme
	make generate-json-schema


# ====== container management ======

container-build:
	$(call log-info,"building container for pop-test")
	podman build -t jjba23/pop-test:$(CONTAINER_TAG) .
container-run:
	podman run jjba23/pop-test:$(CONTAINER_TAG)
container-push:
	podman push jjba23/pop-test:$(CONTAINER_TAG)


# ====== Lisp (Guile Scheme) ======

# Development REPL
guile-repl:
	$(call log-info,"starting Lisp (Guile Scheme) development REPL with ares")
	guix shell -m bindings/guile-scheme/manifest.scm -- \
		guile --fresh-auto-compile -c '((@ (ares server) run-nrepl-server))'

# Target to format source code files according to the style guide
# Find all .scm files and format them using 'guix style'
guile-fmt:
	$(call wikimusic-log-info,"formatting all Scheme files")
	find . -maxdepth 8 -name "*.scm" -type f -exec guix style -f {} \;

# eval and load the desired Lisp (Guile Scheme) for a scenario and serialize into JSON
define guile-scheme-serialize
	$(call log-info,"removing any existing previous serialization outputs")
	rm -rfv resources/scenarios/$(1)/$(2).json
	$(call log-info,"eval the Lisp (Guile Scheme) code and serialize the desired scenario")
	guix shell -m bindings/guile-scheme/manifest.scm -- \
		guile --fresh-auto-compile bindings/guile-scheme/serializer.scm \
		../../resources/scenarios/$(1)/$(2).scm \
		resources/scenarios/$(1)/$(2).json
endef

# run pop-test by running and parsing Lisp (Guile Scheme) into JSON
define pop-test-from-guile
	$(call guile-scheme-serialize,$(1),$(2))
	$(call pop-test,$(1))
endef

# ====== Scala ======

# eval and load the desired Scala for a scenario and serialize into JSON
define scala-serialize
	$(call log-info,"removing any existing previous serialization outputs")
	rm -rfv resources/scenarios/$(1)/$(2).json
	$(call log-info,"eval the Scala code and serialize the desired scenario")
	scala-cli \
		./bindings/scala/Bindings.scala \
		./bindings/scala/Serializer.scala \
		./resources/scenarios/$(1)/$(2).scala -- ./resources/scenarios/$(1)/$(2).json
endef

# run pop-test by running and parsing Scala into JSON
define pop-test-from-scala
	$(call scala-serialize,$(1),$(2))
	$(call pop-test,$(1))
endef

# ====== Pop Test runs ======

# pop-test run scenarios from files in directory
define pop-test
	$(call log-info,"running scenarios for: $(1)")
	TESTCONTAINERS_COMMAND=keep \
		RUST_LOG=info \
		nix develop -c time -p cargo run -- -p ./resources/scenarios/$(1)
endef


# ====== Golden tests ======

gt-vehicle-management:
	$(call pop-test,"vehicle-management")
gt-vehicle-events-hub:
	$(call pop-test,"vehicle-events-hub")
gt-dynamic-tariff-charging:
	$(call pop-test,"dynamic-tariff-charging")
gt-consumer-solar-steering:
	$(call pop-test,"consumer-solar-steering")
gt-lucidplan:
	$(call pop-test,"lucidplan")
gt-green-energy-mix:
	$(call pop-test-from-scala,"green-energy-mix","GoldenTest")
gt-wikimusic:
	@-pkill art
	$(call pop-test-from-guile,"wikimusic","golden-test")
	@-pkill art


# ====== E2E tests ======

e2e-enode-flow:
	cd ~/Ontwikkeling/Werk/Vandebron && sbt "vehicleEventsHub/stage"
	cd ~/Ontwikkeling/Werk/Vandebron && sbt "dynamicTariffCharging/stage"
	$(call pop-test,"e2e-enode-flow")
