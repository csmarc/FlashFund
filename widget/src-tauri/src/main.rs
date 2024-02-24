// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bip39::{Language, Mnemonic};
use gl_client::bitcoin::Network;
use gl_client::{node, pb::cln, scheduler::Scheduler, signer::Signer, tls::TlsConfig};
use reqwest::Error;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use tauri::async_runtime::Receiver;
use tokio::sync::mpsc;
use zbase32::{decode, encode};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_signer() -> bool {
    return true;
}

#[tauri::command]
async fn get_auth_message() -> String {
    let device_certs = force_get_device_certs().await.unwrap();
    return get_signature(device_certs, get_secret()).await;
}

#[tauri::command]
async fn get_balance() -> String {
    let device_certs = force_get_device_certs().await.unwrap();
    return get_signature(device_certs, get_secret()).await;
}

#[tokio::main]
async fn main() {
    let secret = get_secret();
    let tls = get_tls();
    let signer = get_signer(secret.clone(), tls.clone());
    println!("All set up, starting signer...\ngetting device certs...");
    let device_certs = get_device_certs(signer.clone()).await;
    run_signer(
        secret.clone(),
        device_certs.clone(),
        tokio::sync::mpsc::channel(9999).1,
    );
    println!("Getting address...");
    get_address(device_certs.clone(), secret.clone()).await;
    post_certs(device_certs.clone(), secret.clone())
        .await
        .unwrap();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            start_signer,
            get_auth_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    println!("Done");
}

fn get_secret() -> Vec<u8> {
    let file: Result<File, std::io::Error> = File::open("FlashFundSecret.txt");
    match file {
        Ok(_) => {
            let mut file = File::open("FlashFundSecret.txt").unwrap();
            let mut secret = String::new();
            file.read_to_string(&mut secret).unwrap();
            let decoded_secret = hex::decode(secret).unwrap();
            return decoded_secret;
        }
        Err(_) => {
            print!("No secret found, generating new secret...");
            let secret = generate_secet();
            let return_value = secret.clone();
            write_file(hex::encode(secret)).unwrap();
            return return_value;
        }
    }
}

fn get_tls() -> TlsConfig {
    let device_cert =
        include_bytes!("/Users/mauricepoirrierchuden/repo/own/lightdonation/client.crt");
    let device_key =
        include_bytes!("/Users/mauricepoirrierchuden/repo/own/lightdonation/client-key.pem");
    let tls_config = TlsConfig::new()
        .unwrap()
        .identity(device_cert.to_vec(), device_key.to_vec());
    return tls_config;
}

fn get_signer(secret: Vec<u8>, tls: TlsConfig) -> Signer {
    let signer = Signer::new(secret, gl_client::bitcoin::Network::Testnet, tls).unwrap();
    println!("Signer version: {}", signer.version());
    return signer;
}

fn run_signer(secret: Vec<u8>, certs: DeviceCerts, _shutdown: Receiver<()>) {
    let tls_config = TlsConfig::new().unwrap().identity(
        certs.device_cert.into_bytes(),
        certs.device_key.into_bytes(),
    );
    let signer = Signer::new(secret, Network::Testnet, tls_config).unwrap();
    println!("Starting signer...");
    tokio::spawn(async move {
        let (_tx, rv) = mpsc::channel(1);
        println!("in thread: Running signer...");
        match signer.run_forever(rv).await {
            Ok(_) => println!("Signer ran successfully"),
            Err(e) => eprintln!("Failed to run signer: {}", e),
        }
    });
}

fn generate_secet() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let m = Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap();
    let phrase = m.word_iter().fold("".to_string(), |c, n| c + " " + n);
    println!("Seed phrase: {}", phrase);
    let seed = &m.to_seed("")[0..32]; // Only need the first 32 bytes
    let secret = seed[0..32].to_vec();
    return secret;
}

fn write_file(secret: String) -> std::io::Result<()> {
    let mut file = File::create("FlashFundSecret.txt").unwrap();
    file.write_all(secret.as_bytes()).unwrap();
    Ok(())
}

fn write_device_cert(secret: String) -> std::io::Result<()> {
    let mut file = File::create("deviceCertFlashFund.txt").unwrap();
    file.write_all(secret.as_bytes()).unwrap();
    Ok(())
}
fn write_device_key(secret: String) -> std::io::Result<()> {
    let mut file = File::create("deviceKEYFlashFund.txt").unwrap();
    file.write_all(secret.as_bytes()).unwrap();
    Ok(())
}
#[derive(Clone)]
pub struct DeviceCerts {
    pub device_key: String,
    pub device_cert: String,
}

async fn force_get_device_certs() -> Result<DeviceCerts, std::io::Error> {
    let file: Result<File, std::io::Error> = File::open("deviceKEYFlashFund.txt");
    match file {
        Ok(_) => {
            let mut file = File::open("deviceKEYFlashFund.txt").unwrap();
            let mut key = String::new();
            file.read_to_string(&mut key).unwrap();
            let file: Result<File, std::io::Error> = File::open("deviceCertFlashFund.txt");
            match file {
                Ok(_) => {
                    let mut file = File::open("deviceCertFlashFund.txt").unwrap();
                    let mut cert = String::new();
                    file.read_to_string(&mut cert).unwrap();
                    return Ok(DeviceCerts {
                        device_key: key,
                        device_cert: cert,
                    });
                }
                Err(e) => {
                    println!("No device cert found, registering new device...");
                    return Err(e);
                }
            }
        }
        Err(_) => {
            println!("No device key found, registering new device...");
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No device key found",
            ));
        }
    }
}
async fn get_device_certs(signer: Signer) -> DeviceCerts {
    let file: Result<File, std::io::Error> = File::open("deviceKEYFlashFund.txt");
    match file {
        Ok(_) => {
            let mut file = File::open("deviceKEYFlashFund.txt").unwrap();
            let mut key = String::new();
            file.read_to_string(&mut key).unwrap();
            let file: Result<File, std::io::Error> = File::open("deviceCertFlashFund.txt");
            match file {
                Ok(_) => {
                    let mut file = File::open("deviceCertFlashFund.txt").unwrap();
                    let mut cert = String::new();
                    file.read_to_string(&mut cert).unwrap();
                    return DeviceCerts {
                        device_key: key,
                        device_cert: cert,
                    };
                }
                Err(_) => {
                    println!("No device cert found, registering new device...");
                    return register_device(signer).await;
                }
            }
        }
        Err(_) => {
            println!("No device key found, registering new device...");
            return register_device(signer).await;
        }
    }
}

async fn register_device(signer: Signer) -> DeviceCerts {
    let scheduler = Scheduler::new(signer.node_id(), Network::Testnet)
        .await
        .unwrap();
    println!("Registering device...");
    // Passing in the signer is required because the client needs to prove
    // ownership of the `node_id`
    let mut device_certs = DeviceCerts {
        device_key: "".to_string(),
        device_cert: "".to_string(),
    };
    let registration = scheduler.register(&signer, None).await;
    match registration {
        Ok(result) => {
            println!("Registered successfully");
            device_certs.device_key = result.device_key;
            device_certs.device_cert = result.device_cert;
        }
        Err(e) => {
            eprintln!("Failed to register, trying to recover: {}", e);
            let recovery = scheduler.recover(&signer).await.unwrap();
            device_certs.device_key = recovery.device_key;
            device_certs.device_cert = recovery.device_cert;
            println!("Recovered device key and cert");
        }
    }

    println!("Done registering, writing device key and cert to file...");
    write_device_key(device_certs.device_key.clone()).unwrap();
    write_device_cert(device_certs.device_cert.clone()).unwrap();
    return device_certs;
}

async fn get_address(certs: DeviceCerts, secret: Vec<u8>) {
    let mut node = get_client(certs, secret).await;

    let addr = node.new_addr(cln::NewaddrRequest::default()).await.unwrap();
    println!("{:?}", addr);
}

async fn get_client(certs: DeviceCerts, secret: Vec<u8>) -> node::ClnClient {
    let tls_config = TlsConfig::new().unwrap().identity(
        certs.device_cert.into_bytes(),
        certs.device_key.into_bytes(),
    );
    let signer = Signer::new(secret, Network::Testnet, tls_config.clone()).unwrap();
    let sched = Scheduler::new(signer.node_id(), Network::Testnet)
        .await
        .unwrap();
    let node: node::ClnClient = sched.schedule(tls_config.clone()).await.unwrap();
    return node;
}

async fn get_signature(certs: DeviceCerts, secret: Vec<u8>) -> String {
    let tls_config = TlsConfig::new().unwrap().identity(
        certs.clone().device_cert.into_bytes(),
        certs.clone().device_key.into_bytes(),
    );
    let signer = Signer::new(secret.clone(), Network::Testnet, tls_config.clone()).unwrap();
    let message = "ok".to_string();
    let signature = signer.sign_message(message.clone().into_bytes()).unwrap();
    return format!(
        "{}.{}.{}",
        hex::encode(signer.node_id()),
        message,
        hex::encode(signature.0)
    );
}

// async fn get_balance(certs: DeviceCerts, secret: Vec<u8>) -> String {
//     let mut node = get_client(certs, secret).await;
//     node.get_balance(cln::GetbalanceRequest::default())
//         .await
//         .unwrap();
//     return format!("Balance");
// }

#[derive(Serialize)]
struct FundCreation {
    device_crt: String,
    device_key: String,
    node_id: String,
}

async fn post_certs(certs: DeviceCerts, secret: Vec<u8>) -> Result<(), Error> {
    let tls_config = TlsConfig::new().unwrap().identity(
        certs.device_cert.clone().into_bytes(),
        certs.device_key.clone().into_bytes(),
    );
    let signer = Signer::new(secret, Network::Testnet, tls_config.clone()).unwrap();
    let client = reqwest::Client::new();
    let body = FundCreation {
        device_crt: certs.device_cert.clone(),
        device_key: certs.device_key.clone(),
        node_id: hex::encode(signer.node_id()),
    };
    let res = client
        .post("http://localhost:3000/post")
        .json(&body)
        .send()
        .await?;

    println!("Status: {}", res.status());
    let text = res.text().await?;
    println!("Body: {}", text);

    Ok(())
}
