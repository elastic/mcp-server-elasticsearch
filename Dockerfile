# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0
FROM cgr.dev/chainguard/wolfi-base:latest@sha256:1c6a85817d3a8787e094aae474e978d4ecdf634fd65e77ab28ffae513e35cca1

RUN apk update && apk --no-cache add nodejs npm

WORKDIR /app

# Install dependencies (Docker build cache friendly)
COPY package.json package-lock.json tsconfig.json ./
RUN touch index.ts && npm install

COPY *.ts run-docker.sh ./
RUN npm run build

# Future-proof the CLI and require the "stdio" argument
ENV RUNNING_IN_CONTAINER="true"

ENTRYPOINT ["./run-docker.sh"]
