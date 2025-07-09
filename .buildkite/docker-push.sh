#!/usr/bin/env bash

set -euo pipefail

version_tag=$(awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' Cargo.toml)
elastic_image="docker.elastic.co/mcp/elasticsearch"

# set up multi-arch image builder
docker buildx create \
  --name multi-arch \
  --driver docker-container \
  --driver-opt default-load=true \
  --platform linux/amd64,linux/arm64 \
  --bootstrap

# build image
docker buildx build -t "$elastic_image:$version_tag" --builder multi-arch .

# tag image
docker tag "$elastic_image:$version_tag" "$elastic_image:latest"

# push to docker.elastic.co
ELASTIC_PASSWORD=$(vault read -field=password secret/ci/elastic-mcp-server-elasticsearch/devtoolsmachine)
docker login -u "devtoolsmachine" -p "$ELASTIC_PASSWORD" docker.elastic.co
docker push "$elastic_image:$version_tag"
docker push "$elastic_image:latest"
docker logout docker.elastic.co
