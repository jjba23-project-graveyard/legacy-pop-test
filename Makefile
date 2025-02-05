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

run-green-energy-mix: fmt
	cargo run -- play-scenario ./resources/scenarios/test-green-energy-mix.yaml
run-byggsteg: fmt
	cargo run -- play-scenario ./resources/scenarios/test-byggsteg.yaml
run-dynamic-tariff-charging: fmt
	cargo run -- play-scenario ./resources/scenarios/test-dynamic-tariff-charging.yaml	
fmt:
	cargo fix --allow-dirty --bin "pop-test"
	cargo clippy --fix --allow-dirty
	find . -maxdepth 8 -name "*.rs" -type f -exec rustfmt --edition 2021 {} \;
test: fmt
	cargo test
# publish-manual: Generates and publishes the manual from Org files using Emacs.
publish-manual:
	cd docs/Manual/ && emacs -Q --batch \
		-l $(shell realpath ./src/manual/pop-test-manual.el) \
		--funcall pop-test/publish-manual && cd ../../	
