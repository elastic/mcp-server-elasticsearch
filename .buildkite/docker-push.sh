#!/usr/bin/env bash

set -euo pipefail

version_tag=$(jq -r ".version" ./package.json)
elastic_image="docker.elastic.co/mcp/elasticsearch"

# build image
docker buildx create --use --name builder
docker buildx inspect --bootstrap
docker buildx build -t "$elastic_image:$version_tag" --platform linux/amd64,linux/arm64 --builder builder --load .
docker tag "$elastic_image:$version_tag" "$elastic_image:latest"

# push to docker.elastic.co
ELASTIC_PASSWORD=$(vault read -field=password secret/ci/elastic-mcp-server-elasticsearch/devtoolsmachine)
docker login -u "devtoolsmachine" -p "$ELASTIC_PASSWORD" docker.elastic.co
docker push "$elastic_image:$version_tag"
docker push "$elastic_image:latest"
docker logout docker.elastic.co
