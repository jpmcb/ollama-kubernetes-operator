mod consts;

use std::pin::pin;

use anyhow::anyhow;
use k8s_openapi::api::core::v1::{Namespace, Node, Pod};
use kube::{
    api::{Api, ObjectMeta, PostParams},
    runtime::{watcher, WatchStreamExt},
    Client, ResourceExt,
};

use futures::TryStreamExt;

use log::{debug, error, info};

/*
 * Inspired by: https://github.com/kube-rs/kube/blob/main/examples/node_watcher.rs
 *
 * TODO
 *
 * - [ ] CRDs for LLMs from Ollama
 */
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client = Client::try_default().await?;
    let nodes: Api<Node> = Api::all(client.clone());

    let namespaces: Api<Namespace> = Api::all(client.clone());

    // Checks if the namespace already exists.
    // If not, creates a new namespace.
    match namespaces.get(consts::OLLAMA_OPERATOR_NAMESPACE).await {
        Ok(_) => {
            info!(
                "Namespace '{}' already exists, no action taken.",
                consts::OLLAMA_OPERATOR_NAMESPACE
            );
        }

        Err(kube::Error::Api(err)) if err.code == 404 => {
            // Namespace not found, create it

            let ns = Namespace {
                metadata: ObjectMeta {
                    name: Some(consts::OLLAMA_OPERATOR_NAMESPACE.to_string()),
                    ..Default::default()
                },
                ..Default::default()
            };

            namespaces.create(&PostParams::default(), &ns).await?;
            info!(
                "Namespace '{}' not found, created it.",
                consts::OLLAMA_OPERATOR_NAMESPACE
            );
        }

        Err(e) => return Err(anyhow!("Unexpected error from namespace: {}", e)),
    }

    let obs = watcher(
        nodes,
        watcher::Config::default().labels(consts::OLLAMA_NODE_SELECTOR),
    )
    .default_backoff()
    .applied_objects();

    let mut obs = pin!(obs);

    while let Some(n) = obs.try_next().await? {
        ensure_ollama_on_node(&client, &n.name_any()).await?;
    }

    Ok(())
}

// Adds an Ollama pod to a given node
//
// TODO
// - [ ] Wait for node to be in a "ready" state
// - [ ] Needs volume for pulling LLMs into
async fn ensure_ollama_on_node(client: &Client, node_name: &str) -> anyhow::Result<()> {
    info!("Inspecting node: {}", node_name);

    let pods: Api<Pod> = Api::namespaced(client.clone(), consts::OLLAMA_OPERATOR_NAMESPACE);

    let pod_name = format!("ollama-{}", node_name);

    // Checks if the namespace already exists.
    // If not, creates a new namespace.
    match pods.get(&pod_name).await {
        Ok(_) => {
            info!(
                "pod {} already exists on {}: no action taken.",
                pod_name, node_name
            );
        }

        Err(kube::Error::Api(err)) if err.code == 404 => {
            info!(
                "pod {} on node {} not found: created it.",
                pod_name, node_name
            );

            let p: Pod = serde_json::from_value(serde_json::json!({
                "apiVersion": "v1",
                "kind": "Pod",
                "metadata": {
                    "name": pod_name,
                    "labels": {
                        "app": "ollama",
                        "operator": "ollama-kubernetes-operator",
                        "node": node_name
                    }
                },
                "spec": {
                    "containers": [{
                        "name": "ollama",
                        "image": consts::OLLAMA_IMAGE_NAME,
                    }],
                    "nodeName": node_name
                }
            }))?;

            pods.create(&PostParams::default(), &p).await?;
            info!("pod {} on node {} created", pod_name, node_name);
        }

        Err(e) => return Err(anyhow!("Unexpected error from creating pod: {}", e)),
    }

    Ok(())
}
