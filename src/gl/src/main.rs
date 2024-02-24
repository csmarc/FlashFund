use gl_client::{node, pb::cln, scheduler::Scheduler, signer::Signer, tls::TlsConfig};
use std::env;
use std::fs;
use tokio;
// use crate::pb::cln::{amount_or_any, Amount, AmountOrAny};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("A command is needed");
        return;
    }
    let cmd = &args[1];

    if cmd == "getinvoices" {
        if args.len() < 4 {
            println!("Not enough arguments");
            return;
        }

        dbg!(&args);

        let device_cert = fs::read(&args[3]).unwrap();
        let device_key = fs::read(&args[4]).unwrap();

        let tls_config = TlsConfig::new()
            .unwrap()
            .identity(device_cert.to_vec(), device_key.to_vec());

        let sched = Scheduler::new(
            hex::decode(&args[2]).unwrap(),
            gl_client::bitcoin::Network::Testnet,
        )
        .await
        .unwrap();

        let mut node: node::ClnClient = sched.schedule(tls_config.clone()).await.unwrap();

        let invs = node
            .list_invoices(cln::ListinvoicesRequest::default())
            .await
            .unwrap();

        println!("{:?}", &invs.into_inner().invoices);
    }

    if cmd == "createinvoice" {
        if args.len() < 4 {
            println!("Not enough arguments");
            return;
        }

        dbg!(&args);

        let device_cert = fs::read(&args[3]).unwrap();
        let device_key = fs::read(&args[4]).unwrap();

        let tls_config = TlsConfig::new()
            .unwrap()
            .identity(device_cert.to_vec(), device_key.to_vec());

        let sched = Scheduler::new(
            hex::decode(&args[2]).unwrap(),
            gl_client::bitcoin::Network::Testnet,
        )
        .await
        .unwrap();

        let mut node: node::ClnClient = sched.schedule(tls_config.clone()).await.unwrap();

        /*
                message InvoiceRequest {
            AmountOrAny amount_msat = 10;
            string description = 2;
            string label = 3;
            optional uint64 expiry = 7;
            repeated string fallbacks = 4;
            optional bytes preimage = 5;
            optional uint32 cltv = 6;
            optional bool deschashonly = 9;
        }
         */
        let amount = cln::AmountOrAny {
            value: Some(cln::amount_or_any::Value::Amount(cln::Amount { msat: 1000 })),
        };

        println!("Making call...")
        let inv = node
            .invoice(cln::InvoiceRequest {
                amount_msat: Some(amount),
                description: String::from("FlashFund Donation"),
                ..Default::default()
            })
            .await
            .unwrap();

        println!("Finished\n\n");
        println!("{:?}", &inv);
        println!("{:?}", &inv.into_inner());
    }
}
