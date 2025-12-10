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

pub mod cli;
mod protocol;
mod servers;
mod utils;

use crate::cli::{Cli, Command, Configuration, HttpCommand, StdioCommand};
use crate::protocol::http::{HttpProtocol, HttpServerConfig};
use crate::servers::elasticsearch;
use crate::utils::interpolator;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::{RoleServer, Service, ServiceExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::select;
use tokio_util::sync::CancellationToken;

impl Cli {
    pub async fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Stdio(cmd) => run_stdio(cmd, self.container_mode).await,
            Command::Http(cmd) => run_http(cmd, self.container_mode).await,
        }
    }
}

pub async fn run_stdio(cmd: StdioCommand, container_mode: bool) -> anyhow::Result<()> {
    tracing::info!("Starting stdio server");
    let handler = setup_services(&cmd.config, container_mode).await?;
    let service = handler.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    select! {
        _ = service.waiting() => {},
        _ = tokio::signal::ctrl_c() => {},
    }

    Ok(())
}

pub async fn run_http(cmd: HttpCommand, container_mode: bool) -> anyhow::Result<()> {
    let handler = setup_services(&cmd.config, container_mode).await?;
    let server_provider = move || handler.clone();
    let address: SocketAddr = if let Some(addr) = cmd.address {
        addr
    } else if container_mode {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080)
    } else {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
    };

    let ct = HttpProtocol::serve_with_config(
        server_provider,
        HttpServerConfig {
            bind: address,
            ct: CancellationToken::new(),
            // streaming http:
            keep_alive: None,
            stateful_mode: false,
            session_manager: Arc::new(NeverSessionManager::default()),
        },
    )
    .await?;

    tracing::info!("Starting http server at address {}", address);

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}

pub async fn setup_services(config: &Option<PathBuf>, container_mode: bool) -> anyhow::Result<impl Service<RoleServer> + Clone> {
    // Read config file and expand variables

    let config = if let Some(path) = config {
        std::fs::read_to_string(path)?
    } else {
        // Built-in default configuration, based on env variables.
        r#"{
            "elasticsearch": {
                "url": "${ES_URL:http://localhost:9200}",
                "api_key": "${ES_API_KEY:}",
                "username": "${ES_USERNAME:}",
                "password": "${ES_PASSWORD:}",
                "ssl_skip_verify": "${ES_SSL_SKIP_VERIFY:false}"
            }
        }"#
        .to_string()
    };

    // Expand environment variables in the config file
    let config = interpolator::interpolate_from_env(config)?;

    // JSON5 adds comments and multiline strings (useful for ES|QL) to JSON
    let config: Configuration = match serde_json5::from_str(&config) {
        Ok(c) => c,
        Err(serde_json5::Error::Message { msg, location }) if location.is_some() => {
            let location = location.unwrap();
            let line = location.line;
            let column = location.column;
            anyhow::bail!("Failed to parse config: {msg}, at line {line} column {column}");
        }
        Err(err) => return Err(err)?,
    };

    let handler = elasticsearch::ElasticsearchMcp::new_with_config(config.elasticsearch, container_mode)?;
    Ok(handler)
}
