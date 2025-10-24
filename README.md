# Elasticsearch MCP Server

> [!CAUTION]
> This MCP server is deprecated and will only receive critical security updates going forward.
> It has been superseded by [Elastic Agent Builder](https://ela.st/agent-builder-docs)'s [MCP endpoint](https://ela.st/agent-builder-mcp), which is available in Elastic 9.2.0+ and Elasticsearch Serverless projects.

Connect to your Elasticsearch data directly from any MCP Client using the Model Context Protocol (MCP).

This server connects agents to your Elasticsearch data using the Model Context Protocol. It allows you to interact with your Elasticsearch indices through natural language conversations.

## Available Tools

* `list_indices`: List all available Elasticsearch indices
* `get_mappings`: Get field mappings for a specific Elasticsearch index
* `search`: Perform an Elasticsearch search with the provided query DSL
* `esql`: Perform an ES|QL query
* `get_shards`: Get shard information for all or specific indices

## Prerequisites

* An Elasticsearch instance
* Elasticsearch authentication credentials (API key or username/password)
* An MCP Client (e.g. [Claude Desktop](https://claude.ai/download), [Goose](https://block.github.io/goose/))

**Supported Elasticsearch versions**

This works with Elasticsearch versions `8.x` and `9.x`.

## Installation & Setup

> [!NOTE]
>
> Versions 0.3.1 and earlier were installed via `npm`. These versions are deprecated and no longer supported. The following instructions only apply to 0.4.0 and later.
>
> To view instructions for versions 0.3.1 and earlier, see the [README for v0.3.1](https://github.com/elastic/mcp-server-elasticsearch/tree/v0.3.1).

This MCP server is provided as a Docker image at `docker.elastic.co/mcp/elasticsearch`
that supports MCP's stdio, SSE and streamable-HTTP protocols.

Running this container without any argument will output a usage message:

```
docker run docker.elastic.co/mcp/elasticsearch
```

```
Usage: elasticsearch-mcp-server <COMMAND>

Commands:
  stdio  Start a stdio server
  http   Start a streamable-HTTP server with optional SSE support
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Using the stdio protocol

The MCP server needs environment variables to be set:

* `ES_URL`: the URL of your Elasticsearch cluster
* For authentication use either an API key or basic authentication:
  * API key: `ES_API_KEY`
  * Basic auth: `ES_USERNAME` and `ES_PASSWORD`
* Optionally, `ES_SSL_SKIP_VERIFY` set to `true` skips SSL/TLS certificate verification when connecting
  to Elasticsearch. The ability to provide a custom certificate will be added in a later version.

The MCP server is started in stdio mode with this command:

```bash
docker run -i --rm -e ES_URL -e ES_API_KEY docker.elastic.co/mcp/elasticsearch stdio
```

The configuration for Claude Desktop is as follows:

```json
{
 "mcpServers": {
   "elasticsearch-mcp-server": {
    "command": "docker",
    "args": [
     "run", "-i", "--rm",
     "-e", "ES_URL", "-e", "ES_API_KEY",
     "docker.elastic.co/mcp/elasticsearch",
     "stdio"
    ],
    "env": {
      "ES_URL": "<elasticsearch-cluster-url>",
      "ES_API_KEY": "<elasticsearch-API-key>"
    }
   }
 }
}
```

### Using the streamable-HTTP and SSE protocols

Note: streamable-HTTP is recommended, as [SSE is deprecated](https://modelcontextprotocol.io/docs/concepts/transports#server-sent-events-sse-deprecated).

The MCP server needs environment variables to be set:

* `ES_URL`, the URL of your Elasticsearch cluster
* For authentication use either an API key or basic authentication:
  * API key: `ES_API_KEY`
  * Basic auth: `ES_USERNAME` and `ES_PASSWORD`
* Optionally, `ES_SSL_SKIP_VERIFY` set to `true` skips SSL/TLS certificate verification when connecting
  to Elasticsearch. The ability to provide a custom certificate will be added in a later version.

The MCP server is started in http mode with this command:

```bash
docker run --rm -e ES_URL -e ES_API_KEY -p 8080:8080 docker.elastic.co/mcp/elasticsearch http
```

If for some reason your execution environment doesn't allow passing parameters to the container, they can be passed
using the `CLI_ARGS` environment variable: `docker run --rm -e ES_URL -e ES_API_KEY -e CLI_ARGS=http -p 8080:8080...`

The streamable-HTTP endpoint is at `http:<host>:8080/mcp`. There's also a health check at `http:<host>:8080/ping`

Configuration for Claude Desktop (free edition that only supports the stdio protocol).

1. Install `mcp-proxy` (or an equivalent), that will bridge stdio to streamable-http. The executable
   will be installed in `~/.local/bin`:

    ```bash
    uv tool install mcp-proxy
    ```

2. Add this configuration to Claude Desktop:

    ```json
    {
      "mcpServers": {
        "elasticsearch-mcp-server": {
          "command": "/<home-directory>/.local/bin/mcp-proxy",
          "args": [
            "--transport=streamablehttp",
            "--header", "Authorization", "ApiKey <elasticsearch-API-key>",
            "http://<mcp-server-host>:<mcp-server-port>/mcp"
          ]
        }
      }
    }
    ```
