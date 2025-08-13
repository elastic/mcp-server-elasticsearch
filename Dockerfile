# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0

# To create a multi-arch image, run:
# docker buildx build --platform linux/amd64,linux/arm64 --tag elasticsearch-core-mcp-server .

FROM rust:1.89@sha256:c50cd6e20c46b0b36730b5eb27289744e4bb8f32abc90d8c64ca09decf4f55ba AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir -p ./src/bin && \
    echo "pub fn main() {}" > ./src/bin/elasticsearch-core-mcp-server.rs && \
    cargo build --release

COPY src ./src/

RUN cargo build --release

#--------------------------------------------------------------------------------------------------

FROM cgr.dev/chainguard/wolfi-base:latest

COPY --from=builder /app/target/release/elasticsearch-core-mcp-server /usr/local/bin/elasticsearch-core-mcp-server

ENV CONTAINER_MODE=true

EXPOSE 8080/tcp
ENTRYPOINT ["/usr/local/bin/elasticsearch-core-mcp-server"]
