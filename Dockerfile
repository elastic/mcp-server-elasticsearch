# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0

# To create a multi-arch image, run:
# docker buildx build --platform linux/amd64,linux/arm64 --tag elasticsearch-core-mcp-server .

FROM rust:1.100@sha256:e84068000361bf3d4a5ba2b7647f62893528b1eb9f847d7a23b493f5ce8ea254 AS builder

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
