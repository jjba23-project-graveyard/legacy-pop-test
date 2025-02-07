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
	curl -v -X GET --header "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJwNVREVnVFNGQ1NjBsdXVzNmJhUF9XSjdQOWRfUGhtMm96R05CLWVaVkxvIn0.eyJleHAiOjE3Mzg5MjQ3MzIsImlhdCI6MTczODkyMzgzMiwiYXV0aF90aW1lIjoxNzM4OTIzODI2LCJqdGkiOiI4ZGI5NGY2ZS1kNDViLTQwMmYtYWM5My0zYTExMjUyMjA2MTQiLCJpc3MiOiJodHRwczovL3Rlc3QudmRiaW5mcmEubmwvYXV0aC9yZWFsbXMvdmFuZGVicm9uIiwiYXVkIjpbImN1c3RvbWVycy1hcGkiLCJhY2NvdW50Il0sInN1YiI6IjMzMTE0MzEyLWE4NDUtNGM0ZS04NDBiLTQzMDU5NmIzODNiMSIsInR5cCI6IkJlYXJlciIsImF6cCI6IndlYnNpdGUiLCJub25jZSI6IjRlOTJjMjYyLWQ4MDAtNDA2OS04MWQ0LWNkYWQ0OWNhZDNiNCIsInNlc3Npb25fc3RhdGUiOiI0NTlkMjM4MS03ZWE0LTQ0YzctOGNkZC1kZjkyMTkzNzVjNjUiLCJhbGxvd2VkLW9yaWdpbnMiOlsiKiJdLCJyZWFsbV9hY2Nlc3MiOnsicm9sZXMiOlsiRVYiLCJvZmZsaW5lX2FjY2VzcyIsIkNPTlNVTUVSIiwiQkFDS09GRklDRSIsInVtYV9hdXRob3JpemF0aW9uIiwiZGVmYXVsdC1yb2xlcy12YW5kZWJyb24iXX0sInJlc291cmNlX2FjY2VzcyI6eyJjdXN0b21lcnMtYXBpIjp7InJvbGVzIjpbInVtYV9wcm90ZWN0aW9uIiwidXJuOnJvbGU6dmFuZGVicm9uOm9mZmVyaW5nczp3cml0ZSIsInVybjpyb2xlOnZhbmRlYnJvbjpvZmZlcmluZ3M6cmVhZCJdfSwiYWNjb3VudCI6eyJyb2xlcyI6WyJtYW5hZ2UtYWNjb3VudCIsIm1hbmFnZS1hY2NvdW50LWxpbmtzIiwidmlldy1wcm9maWxlIl19fSwic2NvcGUiOiJvcGVuaWQgcHJvZmlsZSBlbWFpbCIsInNpZCI6IjQ1OWQyMzgxLTdlYTQtNDRjNy04Y2RkLWRmOTIxOTM3NWM2NSIsImVtYWlsX3ZlcmlmaWVkIjp0cnVlLCJuYW1lIjoiTWF0dGhpanMgR3VpY2hlbGFhciIsInByZWZlcnJlZF91c2VybmFtZSI6IjIwMDE2ODMiLCJnaXZlbl9uYW1lIjoiTWF0dGhpanMiLCJmYW1pbHlfbmFtZSI6Ikd1aWNoZWxhYXIiLCJlbWFpbCI6ImZhY3R1cmVuQHZhbmRlYnJvbi5ubCJ9.Hf1p31zEI86bVWwOAEx4WQ1kX_j32dJ3WrXj9LM3up2W2IBcIIa3ldXBKet8SUmmDvdfq-jOmQO86ZMe2iI8wvSNB3LZkV2IBmKksqOAYuwsCY14tAj3rbsvslJ4KmOejVJRDVZdncy6xA_pmN4vuyJ7H49cijwa3BHASXF3pWABtcmnOFPaKwTHIS2VuQYUQy7u9p9lV3RJSRu0DswK4nCFqexMm5vKDFS5-L-CO-Qp2u1-cuSJ5L8fPP6Y0lQYXxFCIVBhtP7IF62y37r-NzQswK2QlKoNVXiTQ5SGso5ZRGFHzYK1zCH9dSWTo412yw8EeTuGgPJvRDksY2paEA" http://localhost:8222/api/v1/registeredChargepoints/3514fbab-9a15-4869-9a7d-895e9dfb3560

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
