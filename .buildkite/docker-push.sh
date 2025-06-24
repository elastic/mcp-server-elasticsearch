#!/usr/bin/env bash

set -euo pipefail

version_tag=$(jq -r ".version" ./package.json)
elastic_image="docker.elastic.co/mcp/elasticsearch:$version_tag"

# build image
docker buildx build -t "$elastic_image" --platform linux/amd64,linux/arm64 .

# push to docker.elastic.co
ELASTIC_PASSWORD=$(vault read -field=password secret/ci/elastic-mcp-server-elasticsearch/devtoolsmachine)
docker login -u "devtoolsmachine" -p "$ELASTIC_PASSWORD" docker.elastic.co
docker push "$elastic_image"
docker logout docker.elastic.co
