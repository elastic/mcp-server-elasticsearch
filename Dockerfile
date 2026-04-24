# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0
#
# Multi-stage build aligned with Docker Hub: mcp/elasticsearch
#   (https://hub.docker.com/r/mcp/elasticsearch) — same layer pattern:
#   - Build stage:  rust:1.89
#   - Runtime:      cgr.dev/chainguard/wolfi-base (small distroless-style base)
#
# Multi-arch:
#   DOCKER_BUILDKIT=1 docker buildx build --platform linux/amd64,linux/arm64 -t elasticsearch-core-mcp-server .

# syntax=docker/dockerfile:1

FROM rust:1.89@sha256:c50cd6e20c46b0b36730b5eb27289744e4bb8f32abc90d8c64ca09decf4f55ba AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# Cache dependencies: same idea as
# https://github.com/igabi/mcp-server-elasticsearch/blob/b2912b2ba544060962c7fabf38eb36cc940e9098/Dockerfile
# plus a git cache for the elasticsearch crate (git dependency in Cargo.lock).
RUN --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mkdir -p ./src/bin && \
    echo "fn main() {}" > ./src/bin/elasticsearch-core-mcp-server.rs && \
    echo "fn main() {}" > ./src/bin/start_http.rs && \
    cargo build --release

COPY src ./src/

# BuildKit cache on /app/target is not in the image layer, so the binary is not available for
# COPY unless we copy it to a normal path in this same RUN.
RUN --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    install -D /app/target/release/elasticsearch-core-mcp-server /out/elasticsearch-core-mcp-server

#--------------------------------------------------------------------------------------------------

FROM cgr.dev/chainguard/wolfi-base:latest

ARG VERSION=0.0.0-dev
ENV ELASTIC_MCP_VERSION=${VERSION}

COPY --from=builder /out/elasticsearch-core-mcp-server /usr/local/bin/elasticsearch-core-mcp-server

ENV CONTAINER_MODE=true

EXPOSE 8080/tcp
ENTRYPOINT ["/usr/local/bin/elasticsearch-core-mcp-server"]
