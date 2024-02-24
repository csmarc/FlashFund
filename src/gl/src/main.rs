use gl_client::{node, pb::cln, scheduler::Scheduler, signer::Signer, tls::TlsConfig};
use std::env;
use std::fs;
use tokio;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Result;

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
        let mut invoices = Vec::new();

        let _ = &invs.into_inner().invoices.iter().for_each(|i| {
          let invoice = Invoice {
              amount_received: i.amount_received_msat.clone().unwrap_or_default().msat,
              paid_at: i.paid_at.clone().unwrap_or_default(),
              description: i.description.clone().unwrap_or_default(),
              amount_msat: i.amount_msat.clone().unwrap_or_default().msat,
          };
          // Push the invoice into the vector
          invoices.push(invoice);
      });
      let j = serde_json::to_string(&invoices).unwrap();
      println!("{}", j);
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

        let inv = node
            .invoice(cln::InvoiceRequest {
                amount_msat: Some(amount),
                label: generate_random_number_string(8),
                description: String::from("FlashFund Donation"),
                ..Default::default()
            })
            .await
            .unwrap();
            print!("{:?}", &inv.into_inner().bolt11);
    }
}

#[derive(Serialize,Deserialize)]
struct Invoice {
  amount_received: u64,
  paid_at: u64,
  description: String,
  amount_msat: u64,
}

fn generate_random_number_string(length: usize) -> String {
  let mut rng = rand::thread_rng();
  let number: u32 = rng.gen_range(0..=999_999);
  format!("{:0>width$}", number, width = length)
}
