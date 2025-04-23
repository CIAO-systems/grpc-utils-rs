/// Interceptors for the gRPC channel
pub mod interceptor;

/// Creates a [tonic::transport::Channel] for the endpoint using the given 
/// TLS configuration
pub async fn channel(
    tls: tonic::transport::ClientTlsConfig,
    endpoint: tonic::transport::Endpoint,
) -> Result<tonic::transport::Channel, Box<dyn std::error::Error>> {
    Ok(endpoint
        .keep_alive_while_idle(true)
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
        .tls_config(tls)?
        .connect()
        .await?)
}
