//! Docker container orchestration for the contract harness.
//!
//! Manages the lifecycle of PostgreSQL and Redis test containers:
//! connect → cleanup stale → start → (run tests) → cleanup.

use std::collections::HashMap;
use std::net::TcpStream;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use bollard::Docker;
use bollard::models::{ContainerCreateBody, HostConfig, PortBinding};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, ListContainersOptionsBuilder,
    RemoveContainerOptionsBuilder, StartContainerOptionsBuilder, StopContainerOptionsBuilder,
};
use futures::TryStreamExt;

const TEST_LABEL_KEY: &str = "madome.role";
const TEST_LABEL_VALUE: &str = "contract-test";

/// Manages Docker containers created for contract testing.
pub struct DockerOrchestrator {
    client: Docker,
    /// IP/hostname to reach containers from the test machine.
    pub host: String,
    test_container_ids: Vec<String>,
}

impl DockerOrchestrator {
    /// Connect to the Docker daemon described by `docker_host`.
    ///
    /// - `unix://...` → Unix socket (local)
    /// - `tcp://HOST:PORT` → unencrypted HTTP to `HOST:PORT`
    ///
    /// Sets `self.host` to the address used to reach containers.
    pub async fn connect(docker_host: &str) -> Result<Self> {
        let (client, host) = if docker_host.starts_with("unix://") {
            let client = Docker::connect_with_local_defaults()
                .context("failed to connect to local Docker socket")?;
            (client, "127.0.0.1".to_owned())
        } else if let Some(rest) = docker_host.strip_prefix("tcp://") {
            let host = docker_host_from_url(docker_host);
            let client = Docker::connect_with_http(rest, 120, bollard::API_DEFAULT_VERSION)
                .context("failed to connect to remote Docker daemon")?;
            (client, host)
        } else {
            let client =
                Docker::connect_with_defaults().context("failed to connect to Docker daemon")?;
            (client, "127.0.0.1".to_owned())
        };

        // Verify connectivity
        client
            .ping()
            .await
            .context("Docker daemon did not respond to ping")?;

        Ok(Self {
            client,
            host,
            test_container_ids: Vec::new(),
        })
    }

    /// Remove all **non-running** containers labeled `madome.role=contract-test`.
    ///
    /// Only removes containers in exited/dead state — never kills running ones
    /// (which may belong to a concurrent harness session).
    pub async fn cleanup_stale(&self) -> Result<()> {
        let mut filters = HashMap::new();
        filters.insert(
            "label".to_owned(),
            vec![format!("{TEST_LABEL_KEY}={TEST_LABEL_VALUE}")],
        );
        filters.insert(
            "status".to_owned(),
            vec!["exited".to_owned(), "dead".to_owned()],
        );

        let options = ListContainersOptionsBuilder::new()
            .all(true)
            .filters(&filters)
            .build();

        let containers = self.client.list_containers(Some(options)).await?;

        for c in containers {
            if let Some(id) = c.id {
                self.client
                    .remove_container(
                        &id,
                        Some(RemoveContainerOptionsBuilder::new().force(true).build()),
                    )
                    .await
                    .ok(); // best-effort; stale cleanup failures are non-fatal
            }
        }

        Ok(())
    }

    /// Start a `postgres:18` container on a random host port.
    ///
    /// Returns a `DATABASE_URL` pointing at the container.
    pub async fn start_postgres(&mut self) -> Result<String> {
        let id = self
            .create_and_start(
                "postgres:18",
                Some(vec![
                    "POSTGRES_USER=postgres".to_owned(),
                    "POSTGRES_PASSWORD=postgres".to_owned(),
                    "POSTGRES_DB=madome_test".to_owned(),
                ]),
                "5432/tcp",
            )
            .await?;

        let port = self.mapped_port(&id, "5432/tcp").await?;
        wait_port_open(&self.host, port, 30).await?;

        Ok(format!(
            "postgres://postgres:postgres@{}:{}/madome_test",
            self.host, port
        ))
    }

    /// Start a `redis:8` container on a random host port.
    ///
    /// Returns a `REDIS_URL` pointing at the container.
    pub async fn start_redis(&mut self) -> Result<String> {
        let id = self.create_and_start("redis:8", None, "6379/tcp").await?;

        let port = self.mapped_port(&id, "6379/tcp").await?;
        wait_port_open(&self.host, port, 30).await?;

        Ok(format!("redis://{}:{}", self.host, port))
    }

    /// Stop and remove all test containers started by this orchestrator.
    ///
    /// Always call this — success or failure. Errors are best-effort; call `.ok()` at the call site.
    pub async fn cleanup(&mut self) -> Result<()> {
        for id in self.test_container_ids.drain(..) {
            let _ = self
                .client
                .stop_container(&id, Some(StopContainerOptionsBuilder::new().t(5).build()))
                .await;
            let _ = self
                .client
                .remove_container(
                    &id,
                    Some(RemoveContainerOptionsBuilder::new().force(true).build()),
                )
                .await;
        }
        Ok(())
    }

    /// Create a container with the test label and a random host port, then start it.
    async fn create_and_start(
        &mut self,
        image: &str,
        env: Option<Vec<String>>,
        container_port: &str,
    ) -> Result<String> {
        // Pull the image if not already present locally.
        self.client
            .create_image(
                Some(CreateImageOptionsBuilder::new().from_image(image).build()),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await
            .with_context(|| format!("failed to pull {image}"))?;

        let mut labels = HashMap::new();
        labels.insert(TEST_LABEL_KEY.to_owned(), TEST_LABEL_VALUE.to_owned());

        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            container_port.to_owned(),
            Some(vec![PortBinding {
                host_ip: Some("127.0.0.1".to_owned()),
                host_port: Some(String::new()), // "" = random port
            }]),
        );

        let config = ContainerCreateBody {
            image: Some(image.to_owned()),
            env,
            labels: Some(labels),
            exposed_ports: Some(vec![container_port.to_owned()]),
            host_config: Some(HostConfig {
                port_bindings: Some(port_bindings),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = self
            .client
            .create_container(Some(CreateContainerOptionsBuilder::new().build()), config)
            .await
            .with_context(|| format!("failed to create {image} container"))?
            .id;

        self.client
            .start_container(&id, Some(StartContainerOptionsBuilder::new().build()))
            .await
            .with_context(|| format!("failed to start {image} container"))?;

        self.test_container_ids.push(id.clone());
        Ok(id)
    }

    /// Inspect the container and return the host-side port mapped to `container_port`.
    async fn mapped_port(&self, container_id: &str, container_port: &str) -> Result<u16> {
        let info = self
            .client
            .inspect_container(container_id, None)
            .await
            .context("failed to inspect container")?;

        let port_str = info
            .network_settings
            .as_ref()
            .and_then(|n| n.ports.as_ref())
            .and_then(|ports| ports.get(container_port))
            .and_then(|bindings| bindings.as_ref())
            .and_then(|bindings| bindings.first())
            .and_then(|b| b.host_port.as_deref())
            .ok_or_else(|| anyhow!("no host port found for {container_port}"))?;

        port_str
            .parse()
            .with_context(|| format!("invalid port number: {port_str}"))
    }
}

/// Poll until `host:port` accepts a TCP connection or `timeout_secs` elapses.
async fn wait_port_open(host: &str, port: u16, timeout_secs: u64) -> Result<()> {
    let addr = format!("{host}:{port}");
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);

    loop {
        if TcpStream::connect(&addr).is_ok() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "timed out waiting for {addr} to accept connections"
            ));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// Extract the addressable hostname from a Docker daemon URL.
///
/// - `unix://...`      → `"127.0.0.1"`
/// - `tcp://HOST:PORT` → `"HOST"`
/// - anything else     → `"127.0.0.1"`
fn docker_host_from_url(url: &str) -> String {
    if url.starts_with("unix://") {
        return "127.0.0.1".to_owned();
    }
    if let Some(rest) = url.strip_prefix("tcp://") {
        return rest
            .split_once(':')
            .map(|(host, _)| host.to_owned())
            .unwrap_or_else(|| rest.to_owned());
    }
    "127.0.0.1".to_owned()
}

#[cfg(test)]
mod tests {
    use super::docker_host_from_url;

    #[test]
    fn should_return_loopback_for_unix_socket() {
        assert_eq!(
            docker_host_from_url("unix:///var/run/docker.sock"),
            "127.0.0.1"
        );
    }

    #[test]
    fn should_extract_host_from_tcp_url() {
        assert_eq!(
            docker_host_from_url("tcp://192.168.1.100:2376"),
            "192.168.1.100"
        );
    }

    #[test]
    fn should_return_loopback_for_unknown_scheme() {
        assert_eq!(docker_host_from_url("http://localhost:2375"), "127.0.0.1");
    }
}
