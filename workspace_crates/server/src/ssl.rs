use rcgen::{CertificateParams, DistinguishedName, KeyPair, PKCS_ECDSA_P256_SHA256, SanType};

pub fn generate_self_signed_cert() -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let alg = &PKCS_ECDSA_P256_SHA256;
    let key_pair = KeyPair::generate_for(alg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(rcgen::DnType::CommonName, "localhost");
    params.subject_alt_names = vec![
        SanType::DnsName("localhost".try_into().unwrap()),
        SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
    ];

    let cert = params.self_signed(&key_pair)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Return the full PKCS#8 key pair DER
    Ok((cert.der().to_vec(), key_pair.serialize_der()))
}
 