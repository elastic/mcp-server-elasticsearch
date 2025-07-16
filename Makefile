# set version variable
VERSION = $(shell grep '^version' Cargo.toml | head -n1 | cut -d ' ' -f3 | sed 's/"//g')
ES_IMAGE = "docker.elastic.co/mcp/elasticsearch:$(VERSION)"
ES_IMAGE_LATEST = "docker.elastic.co/mcp/elasticsearch:latest"
AWS_IMAGE = "709825985650.dkr.ecr.us-east-1.amazonaws.com/elastic/mcp/elasticsearch:$(VERSION)"

help: ## Display help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make <target>\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  %-15s %s\n", $$1, $$2 } /^##@/ { printf "\n%s\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

.PHONY: docker-image
docker-image: ## Build a local docker image named es-mcp
	docker build -t "es-mcp:$(VERSION)" .

.PHONY: docker-multiarch-image
docker-multiarch-image: docker-buildx-builder ## Build an amd64+arm64 docker image
	docker buildx build \
		--platform linux/amd64,linux/arm64 \
		--builder es-mcp-multi-arch \
		--load \
		--tag "$(ES_IMAGE)" .
	docker tag "$(ES_IMAGE)" "$(ES_IMAGE_LATEST)"

.PHONY: docker-image-aws
docker-image-aws: docker-buildx-builder ## Build an arm64 docker image using AWS-specific configuration
	docker buildx build \
		--platform linux/arm64 \
		--builder es-mcp-multi-arch \
		--load \
		--file Dockerfile-8000 \
		--tag "$(AWS_IMAGE)" .

.PHONY: docker-buildx-builder
docker-buildx-builder: ## Set up multi-arch Docker buildx builder
	docker buildx ls | grep --silent es-mcp-multi-arch || \
	docker buildx create \
		--name es-mcp-multi-arch \
		--driver docker-container \
		--driver-opt default-load=true \
		--platform linux/amd64,linux/arm64 \
		--bootstrap

.PHONY: docker-push-elastic
docker-push-elastic: docker-multiarch-image ## Push multi-arch image to docker.elastic.co
	docker login \
		-u "devtoolsmachine" \
		-p "$(vault read -field=password secret/ci/elastic-mcp-server-elasticsearch/devtoolsmachine)" \
		docker.elastic.co
	docker push "$(ES_IMAGE)"
	docker push "$(ES_IMAGE_LATEST)"
	docker logout docker.elastic.co
