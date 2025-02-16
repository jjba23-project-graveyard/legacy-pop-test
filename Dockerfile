# pop-test

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

#
# Build image
#
FROM docker.io/library/rust:1.84 as build

WORKDIR /app

COPY ./Cargo.toml .
COPY ./Cargo.lock .
COPY ./src ./src

RUN cargo build --target x86_64-unknown-linux-gnu --release

#
# Release image
#
FROM docker.io/library/rust:1.84

COPY --from=build /app/target/x86_64-unknown-linux-gnu/release/pop-test /app/pop-test

EXPOSE 50088

CMD ["/app/pop-test"]
