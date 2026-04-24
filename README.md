# Elasticsearch MCP Server

> [!CAUTION]
> This MCP server is deprecated and will only receive critical security updates going forward.
> It has been superseded by the [Elastic Agent Builder](https://ela.st/agent-builder-docs) [MCP endpoint](https://ela.st/agent-builder-mcp), which is available in Elastic 9.2.0+ and Elasticsearch Serverless projects.

## Use the Elasticsearch MCP Server for AI Agents

The Elasticsearch MCP Server connects your AI agents to Elasticsearch data using the [Model Context Protocol](https://modelcontextprotocol.io/docs/getting-started/intro) (MCP).
It enables natural language interactions with your Elasticsearch indices, allowing agents to query, analyze, and retrieve data without custom APIs.

Follow these steps to deploy and configure the Elasticsearch MCP Server container image from AWS Marketplace.

### Before you begin

Before you start, ensure you have:

- An Elasticsearch cluster (version 8.x or 9.x) accessible from your AWS environment
- Elasticsearch authentication credentials:
  - An [API key](https://www.elastic.co/docs/deploy-manage/api-keys), or
  - A [username](https://www.elastic.co/docs/deploy-manage/users-roles) and password pair
- Docker installed and running in your AWS environment (for example, on an EC2 instance or in a container service)
- An MCP client configured (such as Claude Desktop, Cursor, VS Code, or another MCP-compatible tool)
- Network connectivity between your deployment environment and your Elasticsearch cluster

> [!NOTE]
>
> These instructions apply to Elasticsearch MCP Server 0.4.0 and later.
> For versions 0.3.1 and earlier, refer to the [README for v0.3.1](https://github.com/igabi/mcp-server-elasticsearch/tree/v0.3.1).

### Deploy the Elasticsearch MCP Server

The Elasticsearch MCP Server is provided as a Docker container image available from AWS Marketplace. You can run it using either the stdio protocol (for direct client connections) or the streamable-HTTP protocol (for web-based integrations).

#### Choose a protocol

The server supports two protocols:

- [stdio](https://modelcontextprotocol.io/specification/latest/basic/transports#stdio): Direct communication between the MCP client and server. Use this when your client supports stdio and runs in the same environment.
- [streamable-HTTP](https://modelcontextprotocol.io/specification/latest/basic/transports#streamable-http): HTTP-based protocol recommended for web integrations, stateful sessions, and concurrent clients.

> **Note:** Server-Sent Events (SSE) is deprecated. Use streamable-HTTP instead.

### Configure the stdio protocol

Use the stdio protocol when your MCP client connects directly to the server process.

#### Set environment variables for stdio mode

Set the following environment variables:

- `ES_URL`: The URL of your Elasticsearch cluster (for example, `https://your-cluster.es.amazonaws.com:9200`)
- For authentication, use one of these options:
  - API key: Set `ES_API_KEY` to your Elasticsearch API key
  - Basic authentication: Set `ES_USERNAME` and `ES_PASSWORD` to your Elasticsearch credentials
- (Optional) `ES_SSL_SKIP_VERIFY`: Set to `true` to skip SSL/TLS certificate verification when connecting to Elasticsearch. Only use this for development or testing environments.

#### Run the container in stdio mode

Start the MCP server in stdio mode:

```bash
docker run -i --rm \
  -e ES_URL \
  -e ES_API_KEY \
  docker.elastic.co/mcp/elasticsearch \
  stdio
```

#### Configure Claude Desktop

Add this configuration to your Claude Desktop configuration file:

```json
{
  "mcpServers": {
    "elasticsearch-mcp-server": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-e", "ES_URL",
        "-e", "ES_API_KEY",
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

Replace `<elasticsearch-cluster-url>` with your Elasticsearch cluster URL and `<elasticsearch-API-key>` with your API key.

### Configure the streamable-HTTP protocol

Use the streamable-HTTP protocol for web-based integrations or when you need to support multiple concurrent clients.

#### Set environment variables for HTTP mode

Set the same environment variables as the stdio protocol:

- `ES_URL`: The URL of your Elasticsearch cluster
- For authentication, use one of these options:
  - API key: Set `ES_API_KEY` to your Elasticsearch API key
  - Basic authentication: Set `ES_USERNAME` and `ES_PASSWORD` to your Elasticsearch credentials
- (Optional) `ES_SSL_SKIP_VERIFY`: Set to `true` to skip SSL/TLS certificate verification

#### Run the container in HTTP mode

Start the MCP server in HTTP mode:

```bash
docker run --rm \
  -e ES_URL \
  -e ES_API_KEY \
  -p 8080:8080 \
  docker.elastic.co/mcp/elasticsearch \
  http
```

The streamable-HTTP endpoint is available at `http://<host>:8080/mcp`. A health check endpoint is available at `http://<host>:8080/ping`.

#### Configure Claude Desktop with HTTP proxy

If you're using Claude Desktop (free edition) which only supports the stdio protocol, use `mcp-proxy` to bridge stdio to streamable-HTTP:

1. Install `mcp-proxy`:

   ```bash
   uv tool install mcp-proxy
   ```
   For alternative installation options, refer to [mcp-proxy/README.md](https://github.com/sparfenyuk/mcp-proxy/blob/main/README.md).

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

   Replace `<home-directory>`, `<elasticsearch-API-key>`, `<mcp-server-host>`, and `<mcp-server-port>` with your values.

### Verify the connection

After configuring your MCP client, verify the connection works:

1. Start your MCP client (for example, Claude Desktop or Cursor).
2. Check that the Elasticsearch MCP Server appears in your available MCP servers.
3. Test a simple query through your agent interface to confirm it can access your Elasticsearch indices.

If the connection fails, verify:

- Your Elasticsearch cluster URL is correct and accessible from your AWS environment
- Your authentication credentials are valid and have the necessary permissions
- Network connectivity exists between the container and your Elasticsearch cluster (check security groups and network ACLs)
- Docker is running and the container started successfully (check container logs with `docker logs <container-id>`)

### Monitor health and status

Monitor the health and proper function of the Elasticsearch MCP Server using these methods:

#### Check container status

Verify the container is running:

```bash
docker ps | grep elasticsearch-mcp-server
```

The container should appear in the list with a status of `Up`.

#### Test the health endpoint (HTTP mode)

If you're using the streamable-HTTP protocol, test the health check endpoint:

```bash
curl http://<host>:8080/ping
```

A successful response returns `pong`, indicating the server is running and healthy.

#### Check container logs

View container logs to identify any issues:

```bash
docker logs <container-id>
```

Look for error messages related to:

- Elasticsearch connection failures
- Authentication errors
- Network connectivity issues

#### Verify Elasticsearch connectivity

Test connectivity to your Elasticsearch cluster from the container:

```bash
docker exec <container-id> curl -k -u <username>:<password> <ES_URL>
```

Or with an API key:

```bash
docker exec <container-id> curl -k -H "Authorization: ApiKey <api-key>" <ES_URL>
```

A successful response indicates the container can reach your Elasticsearch cluster.

### Security and sensitive information

The Elasticsearch MCP Server handles authentication credentials securely:

#### Credential storage

- **API keys and passwords**: Stored only in environment variables passed to the container. They are not persisted to disk or logged.
- **Environment variables**: Set when you run the container. Use AWS Secrets Manager or AWS Systems Manager Parameter Store to manage credentials securely in production environments.

#### Data encryption

- **In transit**: The MCP server communicates with Elasticsearch over HTTPS when your `ES_URL` uses the `https://` protocol. Ensure your Elasticsearch cluster has SSL/TLS enabled.
- **At rest**: The container does not store data locally. All data remains in your Elasticsearch cluster, which uses your cluster's encryption settings.

#### Best practices

- Rotate API keys regularly (every 30-90 days for production environments)
- Use API keys with minimal required permissions (read-only access to specific indices when possible)
- Never commit credentials to version control or share them in logs
- Use AWS Secrets Manager or Parameter Store to inject credentials at runtime instead of hardcoding them

### AWS service quotas

The Elasticsearch MCP Server runs as a container in your AWS environment. Consider these AWS service quotas:

- **EC2 instance limits**: If running on EC2, ensure your instance type supports your expected workload
- **Elastic Container Service (ECS)**: If using ECS, review [ECS service quotas](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/service-quotas.html)
- **Elastic Kubernetes Service (EKS)**: If using EKS, review [EKS service quotas](https://docs.aws.amazon.com/eks/latest/userguide/service-quotas.html)
- **Network bandwidth**: Ensure sufficient network bandwidth between your container and Elasticsearch cluster

To request quota increases, use the [AWS Service Quotas console](https://console.aws.amazon.com/servicequotas/) or refer to the [AWS General Reference Guide](https://docs.aws.amazon.com/general/latest/gr/aws_service_limits.html).

### Available tools

Once connected, the MCP server provides these tools to your agent:

- `list_indices`: List all available Elasticsearch indices
- `get_mappings`: Get field mappings for a specific Elasticsearch index
- `search`: Perform an Elasticsearch search using query DSL
- `esql`: Execute an ES|QL query
- `get_shards`: Get shard information for all or specific indices

Your agent can use these tools to interact with your Elasticsearch data through natural language conversations.

### Next steps

- Learn about AI-powered features available in the Elastic platform
- Explore Agent Builder for building custom AI agents with Elasticsearch
