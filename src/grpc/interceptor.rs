use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use tonic::metadata::Ascii;
use tonic::metadata::MetadataKey;
use tonic::{metadata::AsciiMetadataValue, service::Interceptor, Status};

#[derive(Clone)]
pub struct APIKeyClientInterceptor {
    header_name: Option<String>,
    api_key: String,
}

const X_API_KEY: &str = "x-api-key";

impl APIKeyClientInterceptor {
    /// Cretes a new interceptor for API-Key authentication
    /// # Arguments
    /// * `api_key`: The API key that should be used for authentication
    ///
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            header_name: None,
        }
    }

    fn header_name(&self) -> String {
        self.header_name
            .clone()
            .unwrap_or_else(|| String::from(X_API_KEY))
    }

    pub fn header_key(&self) -> MetadataKey<Ascii> {
        let header_name = self.header_name();
        match MetadataKey::<Ascii>::from_bytes(header_name.as_bytes()) {
            Ok(key) => key,
            Err(e) => {
                log::error!("Invalid meta data key: {e}");
                MetadataKey::from_static(X_API_KEY)
            }
        }
    }
}

impl Interceptor for APIKeyClientInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        if let Ok(value) = AsciiMetadataValue::from_str(&self.api_key) {
            request.metadata_mut().insert(self.header_key(), value);
            return Ok(request);
        }

        Err(Status::invalid_argument(
            "Error while setting additional metadata",
        ))
    }
}

pub struct BearerTokenInterceptor {
    token: String,
}

impl BearerTokenInterceptor {
    pub fn new(token: String) -> Self {
        BearerTokenInterceptor { token }
    }
}

impl Interceptor for BearerTokenInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        req.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", self.token)
                .parse()
                .map_err(|_| tonic::Status::invalid_argument("Invalid Token"))?,
        );
        Ok(req)
    }
}

/// A type alias for an [Interceptor] implementation
pub type BoxedInterceptor = Box<dyn Interceptor + Send + Sync>;
/// A type alias for a list of [Interceptor] implementations
pub type Interceptors = Arc<Mutex<Vec<BoxedInterceptor>>>;

/// A composite interceptor
///
/// It contain a list of interceptors, that will be called in sequence on
/// every call
///
pub struct CompositeInterceptor {
    interceptors: Interceptors,
}

impl CompositeInterceptor {
    /// Creates a new composite interceptor
    /// # Arguments
    /// * `interceptors`: A vector of [Interceptor] instances
    pub fn new(interceptors: Interceptors) -> Self {
        Self { interceptors }
    }
}

impl Interceptor for CompositeInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let mut interceptors = self.interceptors.lock().map_err(|e| {
            // Map the error to a Status, indicating that the lock operation failed
            Status::internal(format!("Failed to lock interceptors: {}", e))
        })?;

        for interceptor in interceptors.iter_mut() {
            req = interceptor.call(req)?;
        }
        Ok(req)
    }
}

#[macro_export]
macro_rules! interceptors {
    // Match the case where we have at least one item
( $($interceptor:expr),* ) => {{
        // Create the interceptors vector
        let interceptors: Vec<Box<dyn tonic::service::Interceptor + Send + Sync>> = vec![
            $( Box::new($interceptor) ),*
        ];

        // Wrap the vector inside an Arc<Mutex>
        std::sync::Arc::new(std::sync::Mutex::new(interceptors))
    }};
}

#[cfg(test)]
mod tests {
    use crate::grpc::interceptor::{APIKeyClientInterceptor, BearerTokenInterceptor, X_API_KEY};

    #[test]
    fn test_api_key_header_none() {
        let test_object = APIKeyClientInterceptor {
            api_key: "key".to_string(),
            header_name: None,
        };

        assert_eq!("key", test_object.api_key);
        assert_eq!(X_API_KEY, test_object.header_key());
    }

    #[test]
    fn test_api_key_header_some() {
        let test_object = APIKeyClientInterceptor {
            api_key: "key".to_string(),
            header_name: Some(String::from("alternative-key")),
        };

        assert_eq!("key", test_object.api_key);
        assert_eq!("alternative-key", test_object.header_key());
    }

    #[test]
    fn test_macro() {
        let interceptors = interceptors!(
            APIKeyClientInterceptor {
                api_key: "key".to_string(),
                header_name: None,
            },
            BearerTokenInterceptor {
                token: "token".to_string()
            }
        );

        assert_eq!(2, interceptors.lock().unwrap().len());
    }

    #[test]
    fn test_bearer_token() {
        let test_object = BearerTokenInterceptor::new("test-token".to_string());

        assert_eq!("test-token", test_object.token);
    }
}
