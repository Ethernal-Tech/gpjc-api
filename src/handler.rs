use ed25519_dalek::{PublicKey, Signature, Verifier};
use hex::FromHex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

use actix_web::web::Json;
use actix_web::{post, HttpResponse, Responder};
use csv::Writer;

use crate::types::{ClientStartRequest, Response, ServerStartRequest};

fn get_path(file_name: &str) -> String {
    let binding = std::env::current_dir().unwrap();
    let current_dir_path = binding.to_str().unwrap();
    let sanctions_dir_path =
        current_dir_path.replace("private-join-and-compute", "") + "sanction-lists/";

    let paths = fs::read_dir(&sanctions_dir_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    match paths.iter().find(|&x| x.contains(file_name)) {
        Some(path) => path.to_string(),
        None => {
            let path = format!("{}/{}", sanctions_dir_path, file_name);
            File::create(&path).unwrap();
            path
        }
    }
}

fn create_csv(receiver: String) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(get_path("UN_test.csv"))?;
    wtr.write_record(&[receiver, 0.to_string()])?;
    wtr.flush()?;
    Ok(())
}

#[allow(unused)]
pub fn start_client(destination_address: String) -> Response {
    let client_csv_path = get_path("UN_test.csv");

    let output = Command::new("bazel-bin/private_join_and_compute/client")
        .arg(format!("--client_data_file={client_csv_path}"))
        .arg(format!("--port={destination_address}"))
        .output()
        .unwrap();

    match output.status.code() {
        Some(code) => {
            if code == 0 {
                let output_text = String::from_utf8_lossy(&output.stdout).into_owned();

                Response {
                    exit_code: code,
                    data: output_text,
                }
            } else {
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("{}", String::from_utf8_lossy(&output.stderr));
                return Response {
                    exit_code: code,
                    data: String::from_utf8_lossy(&output.stdout).into_owned(),
                };
            }
        }
        None => Response {
            exit_code: 1,
            data: "Error in client execution".to_string(),
        },
    }
}

pub fn start_server() -> Response {
    let server_csv_path = get_path("UN_List.csv");
    let dns_name = std::env::var("DNS_NAME").expect("DNS_NAME in .env file must be set.");

    let output = Command::new("bazel-bin/private_join_and_compute/server")
        .arg(format!("--server_data_file={server_csv_path}"))
        .arg(format!("--port={dns_name}:10501"))
        .output()
        .unwrap();

    match output.status.code() {
        Some(code) => {
            if code == 0 {
                let output_text = String::from_utf8_lossy(&output.stdout).into_owned();

                Response {
                    exit_code: code,
                    data: output_text,
                }
            } else {
                Response {
                    exit_code: code,
                    data: String::from_utf8_lossy(&output.stdout).into_owned(),
                }
            }
        }
        None => Response {
            exit_code: 1,
            data: format!("Error in gpjc server execution {:#?}", output),
        },
    }
}

#[post("/api/start-client")]
pub async fn start_client_process(request_data: Json<ClientStartRequest>) -> impl Responder {
    tokio::spawn(async move {
        #[allow(unused)]
        let mut resp: Option<Response> = None;
        match create_csv(request_data.receiver.clone()) {
            Ok(()) => {
                resp = Some(start_client(request_data.to.clone()));

                if resp.is_some() {
                    if resp.as_ref().unwrap().exit_code != 0 {
                        println!(
                            "ERROR: GPJC failed with error: {}",
                            resp.as_ref().unwrap().data
                        );
                    }
                } else {
                    println!("ERROR: GPJC failed")
                }
            }
            Err(err) => {
                println!("Creation of csv file failed {}", err);
            }
        }

        #[cfg(feature = "multiple-machines")]
        {
            if let Some(response) = resp {
                match read_public_key_from_file("pub_key.txt") {
                    Ok(bytes) => {
                        println!("Public key in bytes: {:?}", bytes);
                        let public_key = PublicKey::from_bytes(&bytes).unwrap();

                        println!("{:?}", response.data);
                        // res;signature;signed_msg
                        let resp_parts: Vec<&str> = response.data.split(";").collect();

                        let signature_bytes = Vec::from_hex(resp_parts[1])
                            .map_err(|e| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("Invalid hex string: {}", e),
                                )
                            })
                            .unwrap();

                        let signature = Signature::from_bytes(&signature_bytes).unwrap();

                        if public_key
                            .verify(resp_parts[2].to_string().as_bytes(), &signature)
                            .is_ok()
                        {
                            println!("Signature is valid");
                        } else {
                            println!("Signature not valid");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading public key: {}", e);
                    }
                }

                notify_caller(
                    request_data.tx_id.clone(),
                    request_data.policy_id.to_string(),
                    response.data.to_string(),
                )
                .await;
            }
        }
    });

    HttpResponse::Ok()
}

#[post("/api/start-server")]
pub async fn start_server_process(request_data: Json<ServerStartRequest>) -> impl Responder {
    tokio::spawn(async move {
        let resp = start_server();
        if resp.exit_code != 0 {
            println!("ERROR: GPJC failed with error: {}", resp.data);
            return;
        }

        match read_public_key_from_file("pub_key.txt") {
            Ok(bytes) => {
                println!("Public key in bytes: {:?}", bytes);
                let public_key = PublicKey::from_bytes(&bytes).unwrap();

                println!("{:?}", resp.data);
                // res;signature;signed_msg
                let resp_parts: Vec<&str> = resp.data.split(";").collect();

                let signature_bytes = Vec::from_hex(resp_parts[1])
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid hex string: {}", e),
                        )
                    })
                    .unwrap();

                let signature = Signature::from_bytes(&signature_bytes).unwrap();

                if public_key
                    .verify(resp_parts[2].to_string().as_bytes(), &signature)
                    .is_ok()
                {
                    println!("Signature is valid");
                } else {
                    println!("Signature not valid");
                }
            }
            Err(e) => {
                eprintln!("Error reading public key: {}", e);
            }
        }

        notify_caller(
            request_data.tx_id.clone(),
            request_data.policy_id.clone(),
            resp.data.to_string(),
        )
        .await;
    });

    HttpResponse::Ok()
}

async fn notify_caller(tx_id: String, policy_id: String, resulting_value: String) {
    let mut map = HashMap::new();
    map.insert("TransactionId", tx_id);
    map.insert("PolicyId", policy_id);
    map.insert("Value", resulting_value);

    let client = reqwest::Client::new();
    let api_address = std::env::var("BACKEND_API_ADDRESS")
        .expect("BACKEND_API_ADDRESS in .env file must be set.");
    let _res = client
        .post(format!("http://{}/api/submitTransactionProof", api_address))
        .json(&map)
        .send()
        .await;
}

/// Reads a hex-encoded public key from a .txt file and converts it to bytes.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the .txt file.
///
/// # Returns
///
/// * A Result containing a vector of bytes if successful, or an io::Error.
fn read_public_key_from_file(file_path: &str) -> Result<Vec<u8>, io::Error> {
    // Open the file in read-only mode (ignoring errors).
    let path = Path::new(file_path);
    let mut file = File::open(&path)?;

    // Read the file contents into a string.
    let mut hex_string = String::new();
    file.read_to_string(&mut hex_string)?;

    // Remove any whitespace or newline characters.
    let hex_string = hex_string.trim();

    // Convert the hex string to bytes.
    let bytes = Vec::from_hex(hex_string).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid hex string: {}", e),
        )
    })?;

    Ok(bytes)
}
