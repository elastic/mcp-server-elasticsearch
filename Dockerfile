# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0

# To create a multi-arch image, run:
# docker buildx build --platform linux/amd64,linux/arm64 --tag elasticsearch-core-mcp-server .

FROM rust:1.99@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07 AS builder

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
