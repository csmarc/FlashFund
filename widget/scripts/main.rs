use gl_client::{node, pb::cln, scheduler::Scheduler, signer::Signer, tls::TlsConfig};
use std::fs;
use tokio;

#[tokio::main]
async fn main() {
    let s = "396d1bf993c3fcf0bbe6b5d2c99fdeea24802c318aefff8ac9bfc491a38ca9c0";

    let device_cert = include_bytes!("/Users/mauricepoirrierchuden/repo/own/lightdonation/client.crt");
    let device_key = include_bytes!("/Users/mauricepoirrierchuden/repo/own/lightdonation/client-key.pem");

    let tls_config = TlsConfig::new().unwrap().identity(device_cert.to_vec(), device_key.to_vec());

    let signer = Signer::new(
        hex::decode(s).unwrap(),
        gl_client::bitcoin::Network::Bitcoin,
        tls_config.clone(),
    )
    .unwrap();

    let sched = Scheduler::new(signer.node_id(), gl_client::bitcoin::Network::Bitcoin)
        .await
        .unwrap();

    let mut node: node::ClnClient = sched.schedule(tls_config.clone()).await.unwrap();

    let addr = node.new_addr(cln::NewaddrRequest::default()).await.unwrap();
    println!("{:?}", addr);
}
