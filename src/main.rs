use k8s_openapi::api::core::v1::Node;
use kube::{
    api::{Api, ListParams},
    Client, ResourceExt,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let nodes: Api<Node> = Api::all(client.clone());
    let lp = ListParams::default().labels("workload=ollama");

    for n in nodes.list(&lp).await? {
        println!("Found node: {:?}", n.name_any());
    }

    Ok(())
}
