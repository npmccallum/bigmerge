use std::collections::BTreeMap;

use franca::{Backend, Contract, Keep};

use uuid::Uuid;
use warp::http::header::{HeaderValue, CONTENT_TYPE, LOCATION};
use warp::http::StatusCode;

async fn spawn_server(timeout: &str) -> tokio::io::Result<(String, tokio::process::Child)> {
    const BIN: &str = env!("CARGO_BIN_EXE_contractmgr");

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
    assert_eq!(backends.len(), 4);
    assert!(backends.contains(&Backend::Nil));
    assert!(backends.contains(&Backend::Kvm));
    assert!(backends.contains(&Backend::Sgx));
    assert!(backends.contains(&Backend::Sev));
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
        let body: Contract = ciborium::de::from_reader(&bytes[..]).unwrap();
        assert_eq!(body.uuid, contract.uuid);
        assert_eq!(body.backend, contract.backend);
    }
}

#[tokio::test]
async fn post_contracts_uuid() {
    let (host, _) = spawn_server("5").await.unwrap();

    // Get all the contracts
    let url = format!("http://{}/contracts", host);
    let response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let contracts: Vec<Contract> = ciborium::de::from_reader(&bytes[..]).unwrap();

    // Make a keep for each contract
    for contract in contracts {
        let url = format!("http://{}/contracts/{}", host, contract.uuid);
        let response = reqwest::Client::new().post(&url).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response.headers().get(CONTENT_TYPE),
            Some(&HeaderValue::from_static("application/cbor"))
        );
        let location = response
            .headers()
            .get(LOCATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let bytes = response.bytes().await.unwrap();
        let keep: Keep = ciborium::de::from_reader(&bytes[..]).unwrap();
        assert_eq!(keep.contract.uuid, contract.uuid);
        assert_eq!(keep.contract.backend, contract.backend);
        assert_eq!(location, format!("/keeps/{}", keep.uuid));
    }
}

#[tokio::test]
async fn get_keeps() {
    let (host, _) = spawn_server("5").await.unwrap();

    // Get all the contracts
    let url = format!("http://{}/contracts", host);
    let response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let contracts: Vec<Contract> = ciborium::de::from_reader(&bytes[..]).unwrap();

    // Make a keep for each contract
    let mut keeps: BTreeMap<Uuid, Keep> = BTreeMap::new();
    for contract in contracts {
        let url = format!("http://{}/contracts/{}", host, contract.uuid);
        let response = reqwest::Client::new().post(&url).send().await.unwrap();
        let bytes = response.bytes().await.unwrap();
        let keep: Keep = ciborium::de::from_reader(&bytes[..]).unwrap();
        keeps.insert(keep.uuid, keep);
    }

    // Fetch all the keeps from the server
    let url = format!("http://{}/keeps", host);
    let response = reqwest::get(&url).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE),
        Some(&HeaderValue::from_static("application/cbor"))
    );

    // Make sure the list of created keeps matches the content at /keeps
    let bytes = response.bytes().await.unwrap();
    let unknown: Vec<Keep> = ciborium::de::from_reader(&bytes[..]).unwrap();
    let unknown: BTreeMap<Uuid, Keep> = unknown.into_iter().map(|k| (k.uuid, k)).collect();
    assert_eq!(keeps, unknown);
}

#[tokio::test]
async fn get_keeps_uuid() {
    let (host, _) = spawn_server("5").await.unwrap();

    // Get all the contracts
    let url = format!("http://{}/contracts", host);
    let response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let contracts: Vec<Contract> = ciborium::de::from_reader(&bytes[..]).unwrap();

    for contract in contracts {
        // Create a keep
        let url = format!("http://{}/contracts/{}", host, contract.uuid);
        let response = reqwest::Client::new().post(&url).send().await.unwrap();
        let bytes = response.bytes().await.unwrap();
        let keep: Keep = ciborium::de::from_reader(&bytes[..]).unwrap();

        // Fetch all the keeps from the server
        let url = format!("http://{}/keeps/{}", host, keep.uuid);
        let response = reqwest::get(&url).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE),
            Some(&HeaderValue::from_static("application/cbor"))
        );

        // Make sure the list of created keeps matches the content at /keeps
        let bytes = response.bytes().await.unwrap();
        let unknown: Keep = ciborium::de::from_reader(&bytes[..]).unwrap();
        assert_eq!(keep, unknown);
    }
}

#[tokio::test]
async fn delete_keeps_uuid() {
    let (host, _) = spawn_server("5").await.unwrap();

    // Get all the contracts
    let url = format!("http://{}/contracts", host);
    let response = reqwest::get(&url).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let contracts: Vec<Contract> = ciborium::de::from_reader(&bytes[..]).unwrap();

    for contract in contracts {
        // Create a keep
        let url = format!("http://{}/contracts/{}", host, contract.uuid);
        let response = reqwest::Client::new().post(&url).send().await.unwrap();
        let bytes = response.bytes().await.unwrap();
        let keep: Keep = ciborium::de::from_reader(&bytes[..]).unwrap();

        // Delete the keep
        let url = format!("http://{}/keeps/{}", host, keep.uuid);
        let response = reqwest::Client::new().delete(&url).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Fetch all the keeps from the server
    let url = format!("http://{}/keeps", host);
    let response = reqwest::get(&url).await.unwrap();

    // Ensure that the list of keeps is empty
    let bytes = response.bytes().await.unwrap();
    let unknown: Vec<Keep> = ciborium::de::from_reader(&bytes[..]).unwrap();
    assert_eq!(unknown.len(), 0);
}
