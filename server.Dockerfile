# syntax=docker/dockerfile:1.4

# This is a two-stage docker file. We build the server in one container, run
# it in a sceond
FROM rust:buster AS builder

WORKDIR /usr/src/myapp
COPY Cargo.lock .
COPY Cargo.toml .
COPY src ./src

RUN cargo install --locked --path .

FROM debian:buster-slim

EXPOSE 3000
COPY --from=builder /usr/local/cargo/bin/ten-x-weather /usr/local/bin/ten-x-weather
CMD ["ten-x-weather"]