# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0
FROM cgr.dev/chainguard/wolfi-base:latest@sha256:b72df108f3388c82b0638bcfbad1511d85c60593e67fb8f8a968255f7e0588df

RUN apk --no-cache add nodejs npm

WORKDIR /app

# Install dependencies (Docker build cache friendly)
COPY package.json package-lock.json tsconfig.json ./
RUN touch index.ts && npm install

COPY *.ts run-docker.sh ./
RUN npm run build

# Future-proof the CLI and require the "stdio" argument
ENV RUNNING_IN_CONTAINER="true"

ENTRYPOINT ["./run-docker.sh"]
