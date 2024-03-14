//! extension traits for certain axum types

use axum::http::header::AsHeaderName;
use axum::http::HeaderMap;
use axum::routing::on;
use axum::Router;
use errors::ParsingError;
use sdk::routes::SdkRoute;

pub trait RequireHeaderFromHeaderMap {
    fn require_header<K: AsHeaderName>(&self, header: K) -> Result<String, ParsingError>;
}

impl RequireHeaderFromHeaderMap for HeaderMap {
    fn require_header<K: AsHeaderName>(&self, header: K) -> Result<String, ParsingError> {
        self.get(header.as_str())
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
            .ok_or_else(|| ParsingError::MissingRequiredHeader {
                header: header.as_str().to_string(),
            })
    }
}

pub trait ApplySdkRoute<H, T, S> {
    fn sdk_route<X: SdkRoute>(self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static;
}

impl<H, T, S> ApplySdkRoute<H, T, S> for Router<S>
where
    H: axum::handler::Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    fn sdk_route<X: SdkRoute>(self, handler: H) -> Self {
        self.route(
            X::route(),
            on(
                X::method()
                    .try_into()
                    .expect("Failed to map method to filter."),
                handler,
            ),
        )
    }
}
