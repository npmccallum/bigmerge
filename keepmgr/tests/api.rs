// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::all)]

use koine::{Backend, Contract};

use warp::http::header::{HeaderValue, CONTENT_TYPE};
use warp::http::StatusCode;

async fn spawn_server(timeout: &str) -> tokio::io::Result<(String, tokio::process::Child)> {
    const BIN: &str = env!("CARGO_BIN_EXE_keepmgr");

    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;

    use rand::Rng;
    use tokio::net::TcpStream;
    use tokio::process::Command;

    let mut rng = rand::thread_rng();
    loop {
        // Find an unused port
        let port = rng.gen_range(1024u16..=u16::max_value());
        let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let sock = SocketAddr::new(addr, port);

        // If the port is unused, create our server there.
        if TcpStream::connect(&sock).await.is_err() {
            let host = format!("127.0.0.1:{}", port);

            // Execute the server
            let mut child = Command::new("timeout")
                .arg(timeout)
                .arg(BIN)
                .arg(&host)
                .spawn()?;

            // Wait for the server to start.
            while TcpStream::connect(&sock).await.is_err() && child.try_wait()?.is_none() {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            return Ok((host, child));
        }
    }
}

#[tokio::test]
async fn get_contracts() {
    let (host, _) = spawn_server("5").await.unwrap();

    let url = format!("http://{}/contracts", host);
    let response = reqwest::get(&url).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE),
        Some(&HeaderValue::from_static("application/cbor"))
    );

    let bytes = response.bytes().await.unwrap();
    let contracts: Vec<Contract> = ciborium::de::from_reader(&bytes[..]).unwrap();
    let backends: Vec<Backend> = contracts.into_iter().map(|c| c.backend).collect();
    assert!(!backends.is_empty());
}

#[tokio::test]
async fn get_contracts_uuid() {
    let (host, _) = spawn_server("5").await.unwrap();

    // Get all the contracts
    let url = format!("http://{}/contracts", host);
    let response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let contracts: Vec<Contract> = ciborium::de::from_reader(&bytes[..]).unwrap();

    // Fetch each contract one by one
    for contract in contracts {
        let url = format!("http://{}/contracts/{}", host, contract.uuid);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE),
            Some(&HeaderValue::from_static("application/cbor"))
        );

        let bytes = response.bytes().await.unwrap();
        assert_eq!(contract, ciborium::de::from_reader(&bytes[..]).unwrap());
    }
}
