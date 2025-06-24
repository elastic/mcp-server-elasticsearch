#!/bin/sh
# Copyright Elasticsearch B.V. and contributors
# SPDX-License-Identifier: Apache-2.0

# Entrypoint of the Docker image.

# The OTel SDK logs on stdout and pollutes the communication with the client in stdio mode.
# We cannot just disable it in the TS code as the first log statement is output when importing
# the OTel SDK module.
if [ "$1" = "stdio" ]
then
  export OTEL_LOG_LEVEL=none
fi

# By default the OTel agent will try to connect to http://localhost:4318 that won't exist in a container.
# Disable OTel if there's no endpoint configured
# See https://www.elastic.co/docs/reference/opentelemetry/edot-sdks/nodejs/configuration
if [ -z "$OTEL_EXPORTER_OTLP_ENDPOINT" ]
then
  export OTEL_SDK_DISABLED="true"
fi

exec node dist/index.js "$@"
