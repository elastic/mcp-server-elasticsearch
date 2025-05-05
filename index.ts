#!/usr/bin/env node

/*
 * Copyright Elasticsearch B.V. and contributors
 * SPDX-License-Identifier: Apache-2.0
 */

import { z } from "zod";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { Client, estypes, ClientOptions } from "@elastic/elasticsearch";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import fs from "fs";

// Configuration schema with auth options
const ConfigSchema = z
  .object({
    url: z
      .string()
      .trim()
      .min(1, "Elasticsearch URL cannot be empty")
      .url("Invalid Elasticsearch URL format")
      .describe("Elasticsearch server URL"),

    apiKey: z
      .string()
      .optional()
      .describe("API key for Elasticsearch authentication"),

    username: z
      .string()
      .optional()
      .describe("Username for Elasticsearch authentication"),

    password: z
      .string()
      .optional()
      .describe("Password for Elasticsearch authentication"),

    caCert: z
      .string()
      .optional()
      .describe("Path to custom CA certificate for Elasticsearch"),
  })
  .refine(
    (data) => {
      // If username is provided, password must be provided
      if (data.username) {
        return !!data.password;
      }

      // If password is provided, username must be provided
      if (data.password) {
        return !!data.username;
      }

      // If apiKey is provided, it's valid
      if (data.apiKey) {
        return true;
      }

      // No auth is also valid (for local development)
      return true;
    },
    {
      message:
        "Either ES_API_KEY or both ES_USERNAME and ES_PASSWORD must be provided, or no auth for local development",
      path: ["username", "password"],
    }
  );

type ElasticsearchConfig = z.infer<typeof ConfigSchema>;

// 添加ML相关类型定义
// Add ML related type definitions
interface MlJobStats {
  processed_record_count?: number;
  processed_field_count?: number;
  input_bytes?: number;
  input_field_count?: number;
  invalid_date_count?: number;
  missing_field_count?: number;
  out_of_order_timestamp_count?: number;
  empty_bucket_count?: number;
  sparse_bucket_count?: number;
}

interface ExtendedMlJob {
  job_id: string;
  description?: string;
  create_time?: string;
  finished_time?: string;
  model_snapshot_id?: string;
  job_state?: string;
  data_counts?: MlJobStats;
}

// ML Job Creation Types
interface DetectorConfig {
  detector_description?: string;
  function: string;
  field_name?: string;
  by_field_name?: string;
  over_field_name?: string;
  partition_field_name?: string;
  use_null?: boolean;
  exclude_frequent?: "all" | "none" | "by" | "over";
}

interface AnalysisConfig {
  bucket_span: string;
  detectors: DetectorConfig[];
  influencers?: string[];
  summary_count_field_name?: string;
  categorization_field_name?: string;
  categorization_filters?: string[];
  latency?: string;
  multivariate_by_fields?: boolean;
}

interface DataDescription {
  time_field: string;
  time_format?: string;
  field_delimiter?: string;
  format?: string;
}

interface AnalysisLimits {
  model_memory_limit?: string;
  categorization_examples_limit?: number;
}

interface ModelPlotConfig {
  enabled: boolean;
  annotations_enabled?: boolean;
  terms?: string[];
}

interface DatafeedConfig {
  indices: string[];
  query?: Record<string, any>;
  runtime_mappings?: Record<string, any>;
  datafeed_id?: string;
  scroll_size?: number;
  frequency?: string;
  delayed_data_check_config?: {
    enabled: boolean;
    check_window?: string;
  };
}

interface CreateMlJobRequest {
  analysis_config: AnalysisConfig;
  data_description: DataDescription;
  description?: string;
  groups?: string[];
  analysis_limits?: AnalysisLimits;
  model_plot_config?: ModelPlotConfig;
  results_index_name?: string;
  allow_lazy_open?: boolean;
  datafeed_config?: DatafeedConfig;
}

export async function createElasticsearchMcpServer(
  config: ElasticsearchConfig
) {
  const validatedConfig = ConfigSchema.parse(config);
  const { url, apiKey, username, password, caCert } = validatedConfig;

  const clientOptions: ClientOptions = {
    node: url,
    maxRetries: 5,
    requestTimeout: 60000, // 60 seconds
    compression: true
  };

  // Set up authentication
  if (apiKey) {
    clientOptions.auth = { apiKey };
  } else if (username && password) {
    clientOptions.auth = { username, password };
  }

  // Set up SSL/TLS certificate if provided
  if (caCert) {
    try {
      const ca = fs.readFileSync(caCert);
      clientOptions.tls = { ca };
    } catch (error) {
      console.error(
        `Failed to read certificate file: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  const esClient = new Client(clientOptions);

  const server = new McpServer({
    name: "elasticsearch-mcp-server",
    version: "0.1.1",
  });

  // Tool 1: List indices
  server.tool(
    "list_indices",
    "List all available Elasticsearch indices",
    {},
    async () => {
      try {
        const response = await esClient.cat.indices({ format: "json" });

        const indicesInfo = response.map((index) => ({
          index: index.index,
          health: index.health,
          status: index.status,
          docsCount: index.docsCount,
        }));

        return {
          content: [
            {
              type: "text" as const,
              text: `Found ${indicesInfo.length} indices`,
            },
            {
              type: "text" as const,
              text: JSON.stringify(indicesInfo, null, 2),
            },
          ],
        };
      } catch (error) {
        console.error(
          `Failed to list indices: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
        return {
          content: [
            {
              type: "text" as const,
              text: `Error: ${
                error instanceof Error ? error.message : String(error)
              }`,
            },
          ],
        };
      }
    }
  );

  // Tool 2: Get mappings for an index
  server.tool(
    "get_mappings",
    "Get field mappings for a specific Elasticsearch index",
    {
      index: z
        .string()
        .trim()
        .min(1, "Index name is required")
        .describe("Name of the Elasticsearch index to get mappings for"),
    },
    async ({ index }) => {
      try {
        const mappingResponse = await esClient.indices.getMapping({
          index,
        });

        return {
          content: [
            {
              type: "text" as const,
              text: `Mappings for index: ${index}`,
            },
            {
              type: "text" as const,
              text: `Mappings for index ${index}: ${JSON.stringify(
                mappingResponse[index]?.mappings || {},
                null,
                2
              )}`,
            },
          ],
        };
      } catch (error) {
        console.error(
          `Failed to get mappings: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
        return {
          content: [
            {
              type: "text" as const,
              text: `Error: ${
                error instanceof Error ? error.message : String(error)
              }`,
            },
          ],
        };
      }
    }
  );

  // Tool 3: Search an index with simplified parameters
  server.tool(
    "search",
    "Perform an Elasticsearch search with the provided query DSL. Highlights are always enabled.",
    {
      index: z
        .string()
        .trim()
        .min(1, "Index name is required")
        .describe("Name of the Elasticsearch index to search"),

      queryBody: z
        .record(z.any())
        .refine(
          (val) => {
            try {
              JSON.parse(JSON.stringify(val));
              return true;
            } catch (e) {
              return false;
            }
          },
          {
            message: "queryBody must be a valid Elasticsearch query DSL object",
          }
        )
        .describe(
          "Complete Elasticsearch query DSL object that can include query, size, from, sort, etc."
        ),
    },
    async ({ index, queryBody }) => {
      try {
        // Get mappings to identify text fields for highlighting
        const mappingResponse = await esClient.indices.getMapping({
          index,
        });

        const indexMappings = mappingResponse[index]?.mappings || {};

        const searchRequest: estypes.SearchRequest = {
          index,
          ...queryBody,
          timeout: '30s' // Set timeout for specific queries
        };

        // Always do highlighting
        if (indexMappings.properties) {
          const textFields: Record<string, estypes.SearchHighlightField> = {};

          for (const [fieldName, fieldData] of Object.entries(
            indexMappings.properties
          )) {
            if (fieldData.type === "text" || "dense_vector" in fieldData) {
              textFields[fieldName] = {};
            }
          }

          searchRequest.highlight = {
            fields: textFields,
            pre_tags: ["<em>"],
            post_tags: ["</em>"],
          };
        }

        const result = await esClient.search(searchRequest);

        // Extract the 'from' parameter from queryBody, defaulting to 0 if not provided
        const from = queryBody.from || 0;

        const contentFragments = result.hits.hits.map((hit) => {
          const highlightedFields = hit.highlight || {};
          const sourceData = hit._source || {};

          let content = "";

          for (const [field, highlights] of Object.entries(highlightedFields)) {
            if (highlights && highlights.length > 0) {
              content += `${field} (highlighted): ${highlights.join(
                " ... "
              )}\n`;
            }
          }

          for (const [field, value] of Object.entries(sourceData)) {
            if (!(field in highlightedFields)) {
              content += `${field}: ${JSON.stringify(value)}\n`;
            }
          }

          return {
            type: "text" as const,
            text: content.trim(),
          };
        });

        const metadataFragment = {
          type: "text" as const,
          text: `Total results: ${
            typeof result.hits.total === "number"
              ? result.hits.total
              : result.hits.total?.value || 0
          }, showing ${result.hits.hits.length} from position ${from}`,
        };

        let aggregationFragments = [];
        if (result.aggregations) {
          aggregationFragments.push({
            type: "text" as const,
            text: `Aggregation results:\n${JSON.stringify(result.aggregations, null, 2)}`,
          });
        }

        return {
          content: [
            metadataFragment,
            ...aggregationFragments,
            ...contentFragments,
          ],
        };
      } catch (error) {
        console.error(
          `Search failed: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
        return {
          content: [
            {
              type: "text" as const,
              text: `Error: ${
                error instanceof Error ? error.message : String(error)
              }`,
            },
          ],
        };
      }
    }
  );


  // Tool 4: Execute any Elasticsearch API
  server.tool(
    "execute_es_api",
    "Execute any Elasticsearch API endpoint directly",
    {
      method: z
        .enum(["GET", "POST", "PUT", "DELETE", "HEAD"])
        .describe("HTTP method to use for the request"),
      path: z
        .string()
        .trim()
        .min(1)
        .describe("The API endpoint path (e.g., '_search', 'my_index/_search', '_cluster/health')"),
      params: z
        .record(z.any())
        .optional()
        .describe("Optional URL parameters for the request"),
      body: z
        .record(z.any())
        .optional()
        .describe("Optional request body as a JavaScript object"),
      headers: z
        .record(z.string())
        .optional()
        .describe("Optional HTTP headers for the request")
    },
    async ({ method, path, params, body, headers }) => {
      try {
        // Sanitize the path (remove leading slash if present)
        const sanitizedPath = path.startsWith('/') ? path.substring(1) : path;
        
        // Ensure Content-Type is set correctly
        let customHeaders = headers || {};
        if (body && !customHeaders['Content-Type']) {
          customHeaders['Content-Type'] = 'application/json';
        }
        
        // Prepare the request options
        const options: any = {
          method,
          path: sanitizedPath,
          querystring: params || {},
          body: body || undefined,
          headers: customHeaders
        };

        // Execute the request
        const response = await esClient.transport.request(options);

        return {
          content: [
            {
              type: "text" as const,
              text: `Successfully executed ${method} request to ${path}`
            },
            {
              type: "text" as const,
              text: JSON.stringify(response, null, 2)
            }
          ]
        };
      } catch (error) {
        console.error(
          `Elasticsearch API request failed: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
        
        // Extract and format error details if available
        let errorDetails = "";
        if (error instanceof Error && 'meta' in error && error.meta) {
          const meta = error.meta as any;
          if (meta.body) {
            errorDetails = `\nError details: ${JSON.stringify(meta.body, null, 2)}`;
          }
        }
        
        return {
          content: [
            {
              type: "text" as const,
              text: `Error: ${
                error instanceof Error ? error.message : String(error)
              }${errorDetails}`
            }
          ]
        };
      }
    }
  );

  // Tool 5: Get shard information
  server.tool(
    "get_shards",
    "Get detailed shard information for indices",
    {
      index: z
        .string()
        .optional()
        .describe("Optional index name to filter results. If not provided, shows shards for all indices"),
    },
    async ({ index }) => {
      try {
        const params: any = { format: "json" };
        if (index) {
          params.index = index;
        }

        const response = await esClient.cat.shards(params);

        return {
          content: [
            {
              type: "text" as const,
              text: `Shard information${index ? ` for index: ${index}` : ''}`,
            },
            {
              type: "text" as const,
              text: JSON.stringify(response, null, 2),
            },
          ],
        };
      } catch (error) {
        console.error(
          `Failed to get shard information: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
        return {
          content: [
            {
              type: "text" as const,
              text: `Error: ${
                error instanceof Error ? error.message : String(error)
              }`,
            },
          ],
        };
      }
    }
  );

  return server;
}

const config: ElasticsearchConfig = {
  url: process.env.ES_URL || "",
  apiKey: process.env.ES_API_KEY || "",
  username: process.env.ES_USERNAME || "",
  password: process.env.ES_PASSWORD || "",
  caCert: process.env.ES_CA_CERT || "",
};

async function main() {
  const transport = new StdioServerTransport();
  const server = await createElasticsearchMcpServer(config);

  await server.connect(transport);

  process.on("SIGINT", async () => {
    await server.close();
    process.exit(0);
  });
}

main().catch((error) => {
  console.error(
    "Server error:",
    error instanceof Error ? error.message : String(error)
  );
  process.exit(1);
});
