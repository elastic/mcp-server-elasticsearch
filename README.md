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

## Dynamic URL and Authentication

When using the HTTP or SSE transport protocols, you can dynamically specify the Elasticsearch cluster URL and authentication credentials on a per-request basis. This is useful for:

* Connecting to multiple Elasticsearch clusters without restarting the MCP server
* Using different credentials for different operations
* Multi-tenant scenarios where each request targets a different cluster

The dynamic configuration is provided via HTTP headers:
* **`X-Elasticsearch-URL`**: Override the configured Elasticsearch URL (e.g., `https://my-cluster.es.io:9200`)
* **`Authorization`**: Override the authentication credentials
  * API Key format: `ApiKey <encoded-key>`
  * Basic auth format: `Basic <base64-encoded-username:password>` (encode with: `echo -n "username:password" | base64`)

When these headers are not provided, the server falls back to the configured defaults from the environment variables or config file.

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

* `ES_URL`: the URL of your Elasticsearch cluster (defaults to `http://localhost:9200` if not provided)
* For authentication use either an API key or basic authentication:
  * API key: `ES_API_KEY`
  * Basic auth: `ES_USERNAME` and `ES_PASSWORD`
* Optionally, `ES_SSL_SKIP_VERIFY` set to `true` skips SSL/TLS certificate verification when connecting
  to Elasticsearch. The ability to provide a custom certificate will be added in a later version.

**Note:** When using the HTTP or SSE protocols, the Elasticsearch URL and authentication can be dynamically provided via HTTP headers:
* `X-Elasticsearch-URL`: Override the configured Elasticsearch cluster URL
* `Authorization`: Provide authentication credentials
  * API Key: `ApiKey <encoded-key>`
  * Basic auth: `Basic <base64-username:password>`

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

* `ES_URL`, the URL of your Elasticsearch cluster (defaults to `http://localhost:9200` if not provided)
* For authentication use either an API key or basic authentication:
  * API key: `ES_API_KEY`
  * Basic auth: `ES_USERNAME` and `ES_PASSWORD`
* Optionally, `ES_SSL_SKIP_VERIFY` set to `true` skips SSL/TLS certificate verification when connecting
  to Elasticsearch. The ability to provide a custom certificate will be added in a later version.

#### Dynamic Configuration via HTTP Headers

When using HTTP or SSE protocols, you can override the Elasticsearch URL and authentication on a per-request basis using HTTP headers:

* **`X-Elasticsearch-URL`**: Specify a different Elasticsearch cluster URL for the request
* **`Authorization`**: Provide authentication credentials (supports `ApiKey <key>` and `Basic <base64>` formats)

This allows you to connect to multiple Elasticsearch clusters or use different credentials without restarting the MCP server.

The MCP server is started in http mode with this command:

```bash
docker run --rm -e ES_URL -e ES_API_KEY -p 8080:8080 docker.elastic.co/mcp/elasticsearch http
```

To enable both streamable-HTTP and SSE transports, use the `--sse` flag:

```bash
docker run --rm -e ES_URL -e ES_API_KEY -p 8080:8080 docker.elastic.co/mcp/elasticsearch http --sse
```

If for some reason your execution environment doesn't allow passing parameters to the container, they can be passed
using the `CLI_ARGS` environment variable: `docker run --rm -e ES_URL -e ES_API_KEY -e CLI_ARGS="http --sse" -p 8080:8080...`

**Available Endpoints:**
- Streamable-HTTP: `http://<host>:8080/mcp`
- SSE (when `--sse` is used): `http://<host>:8080/mcp/sse`
- Health check: `http://<host>:8080/ping`
- Readiness probe: `http://<host>:8080/_health/ready`
- Liveness probe: `http://<host>:8080/_health/live`

#### Direct Connection Support

Many MCP clients can connect directly to the streamable-HTTP and SSE endpoints:
- **Cursor**: Supports direct connections to both streamable-HTTP and SSE endpoints
- **Google Gemini**: Supports direct connections to both streamable-HTTP and SSE endpoints
- **Claude Desktop (free edition)**: Only supports stdio protocol, requires a proxy (see configuration below)

If your MCP client supports direct HTTP/SSE connections, you can configure it to use the endpoints above directly without needing `mcp-proxy`.

#### Configuration for Claude Desktop

Claude Desktop (free edition) only supports the stdio protocol and requires a proxy.

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

3. To dynamically specify the Elasticsearch URL per request, add the `X-Elasticsearch-URL` header:

    **With API Key:**
    ```json
    {
      "mcpServers": {
        "elasticsearch-mcp-server": {
          "command": "/<home-directory>/.local/bin/mcp-proxy",
          "args": [
            "--transport=streamablehttp",
            "--header", "Authorization", "ApiKey <elasticsearch-API-key>",
            "--header", "X-Elasticsearch-URL", "https://my-cluster.es.io:9200",
            "http://<mcp-server-host>:<mcp-server-port>/mcp"
          ]
        }
      }
    }
    ```

    **With Username/Password (Basic Auth):**
    ```json
    {
      "mcpServers": {
        "elasticsearch-mcp-server": {
          "command": "/<home-directory>/.local/bin/mcp-proxy",
          "args": [
            "--transport=streamablehttp",
            "--header", "Authorization", "Basic <base64-encoded-username:password>",
            "--header", "X-Elasticsearch-URL", "https://my-cluster.es.io:9200",
            "http://<mcp-server-host>:<mcp-server-port>/mcp"
          ]
        }
      }
    }
    ```
    
    Note: For Basic auth, encode your credentials as base64. In shell: `echo -n "username:password" | base64`

4. To use the SSE transport (deprecated, but still supported), start the server with `--sse` and use the `/mcp/sse` endpoint:

    ```json
    {
      "mcpServers": {
        "elasticsearch-mcp-server": {
          "command": "/<home-directory>/.local/bin/mcp-proxy",
          "args": [
            "--transport=sse",
            "--header", "Authorization", "ApiKey <elasticsearch-API-key>",
            "http://<mcp-server-host>:<mcp-server-port>/mcp/sse"
          ]
        }
      }
    }
    ```

### Configuration for Other MCP Clients

Clients like **Cursor** and **Google Gemini** support direct connections to streamable-HTTP and SSE endpoints without requiring a proxy.

#### Cursor Configuration

Add to your Cursor MCP settings:

```json
{
  "mcpServers": {
    "elasticsearch": {
      "url": "http://<mcp-server-host>:<mcp-server-port>/mcp",
      "transport": "streamable-http",
      "headers": {
        "Authorization": "ApiKey <elasticsearch-API-key>",
        "X-Elasticsearch-URL": "https://my-cluster.es.io:9200"
      }
    }
  }
}
```

#### Google Gemini Configuration

Add to your Gemini MCP configuration:

**For Streamable-HTTP:**
```json
{
  "mcpServers": {
    "elasticsearch": {
      "httpUrl": "http://<mcp-server-host>:<mcp-server-port>/mcp",
      "headers": {
        "Authorization": "ApiKey <elasticsearch-API-key>",
        "X-Elasticsearch-URL": "https://my-cluster.es.io:9200"
      }
    }
  }
}
```

**For SSE (start server with `--sse`):**
```json
{
  "mcpServers": {
    "elasticsearch": {
      "url": "http://<mcp-server-host>:<mcp-server-port>/mcp/sse",
      "headers": {
        "Authorization": "ApiKey <elasticsearch-API-key>",
        "X-Elasticsearch-URL": "https://my-cluster.es.io:9200"
      }
    }
  }
}
```

**Note**: Replace `<mcp-server-host>` and `<mcp-server-port>` with your server's address (default port is 8080). The dynamic headers (`Authorization` and `X-Elasticsearch-URL`) are optional if you've configured defaults via environment variables.
