// Licensed to Elasticsearch B.V. under one or more contributor
// license agreements. See the NOTICE file distributed with
// this work for additional information regarding copyright
// ownership. Elasticsearch B.V. licenses this file to you under
// the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

mod base_tools;

use crate::servers::IncludeExclude;
use crate::utils::none_if_empty_string;
use elasticsearch::Elasticsearch;
use elasticsearch::auth::Credentials;
use elasticsearch::cert::CertificateValidation;
use elasticsearch::http::Url;
use elasticsearch::http::response::Response;
use http::header::USER_AGENT;
use http::request::Parts;
use http::{HeaderValue, header};
use indexmap::IndexMap;
use rmcp::RoleServer;
use rmcp::model::ToolAnnotations;
use rmcp::service::RequestContext;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_bool_from_anything;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ElasticsearchMcpConfig {
    /// Cluster URL (optional if dynamic URL will be provided via X-Elasticsearch-URL header)
    #[serde(default = "default_es_url", deserialize_with = "none_if_empty_string")]
    pub url: Option<String>,

    /// API key
    #[serde(default, deserialize_with = "none_if_empty_string")]
    pub api_key: Option<String>,

    /// Username
    #[serde(default, deserialize_with = "none_if_empty_string")]
    pub username: Option<String>,

    /// Password
    #[serde(default, deserialize_with = "none_if_empty_string")]
    pub password: Option<String>,

    /// Should we skip SSL certificate verification?
    #[serde(default, deserialize_with = "deserialize_bool_from_anything")]
    pub ssl_skip_verify: bool,

    /// Search templates to expose as tools or resources
    #[serde(default)]
    pub tools: Tools,

    /// Prompts
    #[serde(default)]
    pub prompts: Vec<String>,
    // TODO: search as resources?
}

fn default_es_url() -> Option<String> {
    Some("http://localhost:9200".to_string())
}

// A wrapper around an ES client that provides a client instance configured
/// for a given request context (i.e. auth credentials and URL)
#[derive(Clone)]
pub struct EsClientProvider {
    client: Elasticsearch,
    ssl_skip_verify: bool,
}

impl EsClientProvider {
    pub fn new(client: Elasticsearch, ssl_skip_verify: bool) -> Self {
        EsClientProvider { client, ssl_skip_verify }
    }

    /// If the incoming request is a http request and has headers for URL or Authorization,
    /// use them to create a new client instance for the remote ES instance.
    pub fn get(&self, context: RequestContext<RoleServer>) -> Cow<'_, Elasticsearch> {
        let client = &self.client;

        let parts = context.extensions.get::<Parts>();
        
        // Check for custom URL header
        let custom_url = parts
            .and_then(|p| p.headers.get("X-Elasticsearch-URL"))
            .and_then(|h| h.to_str().ok());

        // Check for authorization header
        let mut auth = parts
            .and_then(|p| p.headers.get(header::AUTHORIZATION))
            .and_then(|h| h.to_str().ok());

        // If neither URL nor auth is provided, return the default client
        if custom_url.is_none() && auth.is_none() {
            tracing::debug!("Using default Elasticsearch client configuration");
            return Cow::Borrowed(client);
        }

        if custom_url.is_some() || auth.is_some() {
            tracing::debug!(
                "Dynamic configuration detected: custom_url={}, has_auth={}",
                custom_url.is_some(),
                auth.is_some()
            );
        }

        // MCP inspector insists on sending a bearer token and prepends "Bearer" to the value provided
        if let Some(auth_str) = auth {
            if auth_str.starts_with("Bearer ApiKey ") || auth_str.starts_with("Bearer Basic ") {
                auth = Some(auth_str.trim_start_matches("Bearer "));
            }
        }

        // If we have a custom URL, we need to build a new transport from scratch
        if let Some(url_str) = custom_url {
            match self.build_client_with_url(url_str, auth) {
                Ok(new_client) => {
                    tracing::info!("Using dynamic Elasticsearch URL: {}", url_str);
                    return Cow::Owned(new_client);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to create client with custom URL '{}': {}. Falling back to default configuration.",
                        url_str, e
                    );
                    // Fall back to just updating auth if we can't parse the URL
                }
            }
        }

        // If we only have auth (no custom URL), just clone with new auth
        if let Some(auth_str) = auth {
            tracing::debug!("Using dynamic authentication with default URL");
            let transport = client
                .transport()
                .clone_with_auth(Some(Credentials::AuthorizationHeader(auth_str.to_string())));
            return Cow::Owned(Elasticsearch::new(transport));
        }

        // Fallback to default client
        Cow::Borrowed(client)
    }

    /// Build a new Elasticsearch client with a custom URL and optional auth
    fn build_client_with_url(&self, url_str: &str, auth: Option<&str>) -> anyhow::Result<Elasticsearch> {
        let url = Url::parse(url_str)?;
        let pool = elasticsearch::http::transport::SingleNodeConnectionPool::new(url);
        let mut transport = elasticsearch::http::transport::TransportBuilder::new(pool);
        
        if let Some(auth_str) = auth {
            transport = transport.auth(Credentials::AuthorizationHeader(auth_str.to_string()));
        }
        
        if self.ssl_skip_verify {
            transport = transport.cert_validation(CertificateValidation::None);
        }
        
        transport = transport.header(
            USER_AGENT,
            HeaderValue::from_str(&format!("elastic-mcp/{}", env!("CARGO_PKG_VERSION")))?,
        );
        
        let transport = transport.build()?;
        Ok(Elasticsearch::new(transport))
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tools {
    #[serde(flatten)]
    pub incl_excl: Option<IncludeExclude>,
    pub custom: HashMap<String, CustomTool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomTool {
    Esql(EsqlTool),
    SearchTemplate(SearchTemplateTool),
}

impl CustomTool {
    pub fn base(&self) -> &ToolBase {
        match self {
            CustomTool::Esql(esql) => &esql.base,
            CustomTool::SearchTemplate(search_template) => &search_template.base,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolBase {
    pub description: String,
    pub parameters: IndexMap<String, schemars::schema::SchemaObject>,
    pub annotations: Option<ToolAnnotations>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EsqlTool {
    #[serde(flatten)]
    base: ToolBase,
    query: String,
    #[serde(default)]
    format: EsqlResultFormat,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EsqlResultFormat {
    #[default]
    // Output as JSON, either as an array of objects or as a single object.
    Json,
    // If a single object with a single property, output only its value
    Value,
    //Csv,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTemplateTool {
    #[serde(flatten)]
    base: ToolBase,
    #[serde(flatten)]
    template: SearchTemplate,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchTemplate {
    TemplateId(String),
    Template(serde_json::Value), // or constrain to an object?
}

#[derive(Clone)]
pub struct ElasticsearchMcp {}

impl ElasticsearchMcp {
    pub fn new_with_config(config: ElasticsearchMcpConfig, container_mode: bool) -> anyhow::Result<base_tools::EsBaseTools> {
        let creds = if let Some(api_key) = config.api_key.clone() {
            Some(Credentials::EncodedApiKey(api_key))
        } else if let Some(username) = config.username.clone() {
            let pwd = config.password.clone().ok_or(anyhow::Error::msg("missing password"))?;
            Some(Credentials::Basic(username, pwd))
        } else {
            None
        };

        // Use the configured URL or fall back to default
        let url_str = config.url.as_ref()
            .ok_or_else(|| anyhow::Error::msg("Elasticsearch URL is not configured and no default is available"))?;

        if url_str.is_empty() {
            return Err(anyhow::Error::msg("Elasticsearch URL is empty"));
        }

        let mut url = Url::parse(url_str)?;
        if container_mode {
            rewrite_localhost(&mut url)?;
        }

        tracing::info!("Default Elasticsearch URL: {}", url);
        if creds.is_some() {
            tracing::info!("Using configured authentication credentials");
        } else {
            tracing::info!("No default authentication configured (can be provided dynamically via headers)");
        }

        let pool = elasticsearch::http::transport::SingleNodeConnectionPool::new(url.clone());
        let mut transport = elasticsearch::http::transport::TransportBuilder::new(pool);
        if let Some(creds) = creds {
            transport = transport.auth(creds);
        }
        if config.ssl_skip_verify {
            transport = transport.cert_validation(CertificateValidation::None)
        }
        transport = transport.header(
            USER_AGENT,
            HeaderValue::from_str(&format!("elastic-mcp/{}", env!("CARGO_PKG_VERSION")))?,
        );
        let transport = transport.build()?;
        let es_client = Elasticsearch::new(transport);

        Ok(base_tools::EsBaseTools::new(es_client, config.ssl_skip_verify))
    }
}

//------------------------------------------------------------------------------------------------
// Utilities

/// Rewrite urls targeting `localhost` to a hostname that maps to the container host, if possible.
///
/// The host name for the container host depends on the OCI runtime used. This is useful to accept
/// Elasticsearch URLs like `http://localhost:9200`.
fn rewrite_localhost(url: &mut Url) -> anyhow::Result<()> {
    use std::net::ToSocketAddrs;
    let aliases = &[
        "host.docker.internal", // Docker
        "host.containers.internal", // Podman, maybe others
    ];

    if let Some(host) = url.host_str() && host == "localhost" {
        for alias in aliases {
            if let Ok(mut alias_add) = (*alias, 80).to_socket_addrs() && alias_add.next().is_some() {
                url.set_host(Some(alias))?;
                tracing::info!("Container mode: using '{alias}' instead of 'localhost'");
                return Ok(());
            }
        }
    }
    tracing::warn!("Container mode: cound not find a replacement for 'localhost'");
    Ok(())
}

/// Map any error to an internal error of the MCP server
pub fn internal_error(e: impl std::error::Error) -> rmcp::Error {
    rmcp::Error::internal_error(e.to_string(), None)
}

/// Return an error as an error response to the client, which may be able to take
/// action to correct it. This should be refined to handle common error types such
/// as index not found, which could be caused by the client hallucinating an index name.
///
/// TODO (in rmcp): if rmcp::Error had a variant that accepts a CallToolResult, this would
/// allow to use the '?' operator while sending a result to the client.
pub fn handle_error(result: Result<Response, elasticsearch::Error>) -> Result<Response, rmcp::Error> {
    match result {
        Ok(resp) => resp.error_for_status_code(),
        Err(e) => {
            tracing::error!("Error: {:?}", &e);
            Err(e)
        }
    }
    .map_err(internal_error)
}

pub async fn read_json<T: DeserializeOwned>(
    response: Result<Response, elasticsearch::Error>,
) -> Result<T, rmcp::Error> {
    // let text = read_text(response).await?;
    // tracing::debug!("Received json {text}");
    // serde_json::from_str(&text).map_err(internal_error)

    let response = handle_error(response)?;
    response.json().await.map_err(internal_error)
}

#[allow(dead_code)]
pub async fn read_text(result: Result<Response, elasticsearch::Error>) -> Result<String, rmcp::Error> {
    let response = handle_error(result)?;
    response.text().await.map_err(internal_error)
}
