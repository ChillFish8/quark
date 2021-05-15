#![allow(unused)]

use quinn::{
    Certificate, CertificateChain, ClientConfig, ClientConfigBuilder, Endpoint, Incoming,
    PrivateKey, ServerConfig, ServerConfigBuilder, TransportConfig,
};

use std::{error::Error, net::SocketAddr, sync::Arc};
use std::{
    ascii, fs, io,
    path::{self, Path, PathBuf},
    str,
};

use anyhow::{anyhow, bail, Context, Result};
use futures::{StreamExt, TryFutureExt};
use structopt::{self, StructOpt};
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;
use bytes::Bytes;

pub async fn run() -> Result<()> {
    let addr = "127.0.0.1:5000".parse()?;
    let server_cert = fs::read("../tls/server.cert")?;

    let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap(), &[&server_cert])?;
    // connect to server
    let quinn::NewConnection {
        connection,
        mut uni_streams,
        ..
    } = endpoint
        .connect(&addr, "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());

    // Waiting for a stream will complete with an error when the server closes the connection
    let _ = uni_streams.next().await;

    Ok(())
}

pub fn make_client_endpoint(
    bind_addr: SocketAddr,
    server_certs: &[&[u8]],
) -> Result<Endpoint> {
    let client_cfg = configure_client(server_certs)?;
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(client_cfg);
    let (endpoint, _incoming) = endpoint_builder.bind(&bind_addr)?;
    Ok(endpoint)
}

fn configure_client(server_certs: &[&[u8]]) -> Result<ClientConfig> {
    let mut cfg_builder = ClientConfigBuilder::default();
    for cert in server_certs {
        cfg_builder.add_certificate_authority(Certificate::from_der(&cert)?)?;
    }
    Ok(cfg_builder.build())
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}