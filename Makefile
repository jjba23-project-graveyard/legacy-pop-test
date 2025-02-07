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

test-green-energy-mix: fmt
	nix develop -c cargo run -- play-scenario ./resources/scenarios/test-green-energy-mix.yaml
test-byggsteg: fmt
	nix develop -c cargo run -- play-scenario ./resources/scenarios/test-byggsteg.yaml
test-dynamic-tariff-charging: fmt
	nix develop -c cargo run -- play-scenario ./resources/scenarios/test-dynamic-tariff-charging.yaml

test-dynamic-tariff-charging-call:
	curl -v -X GET --header "Authorization: Bearer $(ADMIN_TOKEN)" http://localhost:8222/api/v1/registeredChargepoints/3514fbab-9a15-4869-9a7d-895e9dfb3560

fmt:
	nix develop -c cargo fix --allow-dirty --bin "pop-test"
	nix develop -c cargo clippy --fix --allow-dirty
	nix develop -c find . -maxdepth 8 -name "*.rs" -type f -exec rustfmt --edition 2021 {} \;
test: fmt
	nix develop -c cargo test
# publish-manual: Generates and publishes the manual from Org files using Emacs.
publish-manual:
	cd docs/Manual/ && emacs -Q --batch \
		-l $(shell realpath ./src/manual/pop-test-manual.el) \
		--funcall pop-test/publish-manual && cd ../../	

# username = "alice"
# password = "wonderland"

# user_settings = {
#     "username": username,
#     "enabled": True,
#     "credentials": [{
#         "type": "password",
#         "value": password,
#         "temporary": False,
#     }]
# }


# resp = requests.post(
#     f"{keycloak_root}/admin/realms/{realm}/users",
#     json=user_settings,
#     headers=auth_headers,
# )
# resp.raise_for_status()
# location = resp.headers["Location"]
# print(location)

# http://localhost:8642/realms/keldor/protocol/openid-connect/token
# Do the token request (here using the password grant for simplicity):

# resp = requests.post(
#     token_endpoint,
#     data={
#         "client_id": client_id,
#         "username": username,
#         "password": password,
#         "grant_type": "password",
#     }
# )
# resp.raise_for_status()
# resp.json()
