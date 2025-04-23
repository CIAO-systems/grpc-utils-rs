[![Workflow Status](https://github.com/CIAO-systems/grpc-utils-rs/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/CIAO-systems/grpc-utils-rs/actions/workflows/build-and-test.yml)
[![GitHub License](https://img.shields.io/github/license/CIAO-systems/grpc-utils-rs)](https://github.com/CIAO-systems/grpc-utils-rs?tab=Apache-2.0-1-ov-file)

# Usage
To use the library in your project, add it to the dependencies:
```toml
[dependencies]
grpc-utils-rs = { git = "https://github.com/CIAO-systems/grpc-utils-rs" }
```

# Functions
## channel
```rust
pub async fn channel(
    tls: tonic::transport::ClientTlsConfig,
    endpoint: tonic::transport::Endpoint,
) -> Result<tonic::transport::Channel, Box<dyn std::error::Error>>;
```

# Interceptor implementations
* APIKeyClientInterceptor
* BearerTokenInterceptor

# Macros
```rust
interceptors!();
```
Creates a vector of interceptors