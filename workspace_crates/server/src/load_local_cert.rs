use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use std::{fs::File, io::BufReader, sync::Arc};

pub fn load_local_tls_config() -> Arc<ServerConfig> {
    let mut cert_reader = BufReader::new(File::open("local_ssl/localhost.crt").unwrap());
    let cert_chain: Vec<CertificateDer<'static>> =
        certs(&mut cert_reader).collect::<Result<_, _>>().unwrap();

    let mut key_reader = BufReader::new(File::open("local_ssl/localhost.key").unwrap());

    let mut keys: Vec<PrivateKeyDer<'static>> = pkcs8_private_keys(&mut key_reader)
        .map(|k| k.map(Into::into))
        .collect::<Result<_, _>>()
        .unwrap();

    if keys.is_empty() {
        let mut key_reader = BufReader::new(File::open("local_ssl/localhost.key").unwrap());
        keys = rsa_private_keys(&mut key_reader)
            .map(|k| k.map(Into::into))
            .collect::<Result<_, _>>()
            .unwrap();
    }
    let key = keys.remove(0);

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .unwrap();

    Arc::new(config)
}
