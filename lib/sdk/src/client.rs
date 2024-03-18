use std::sync::Arc;
use std::time::Duration;

use errors::{NetworkError, ParsingError, TicketsError, TicketsResult};
use reqwest::{Client, ClientBuilder, Method, Request, RequestBuilder, Response, StatusCode, Url};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::routes::Empty;
use auth::jwt::{JwtAccessor, JwtConfig, JwtData};
use reqwest::header::{HeaderMap, HeaderValue};

struct MethodWrapper(http::Method);

impl From<MethodWrapper> for Method {
    fn from(val: MethodWrapper) -> Self {
        match val.0 {
            http::Method::GET => Method::GET,
            http::Method::POST => Method::POST,
            http::Method::PUT => Method::PUT,
            http::Method::DELETE => Method::DELETE,
            http::Method::PATCH => Method::PATCH,
            http::Method::HEAD => Method::HEAD,
            http::Method::OPTIONS => Method::OPTIONS,
            http::Method::CONNECT => Method::CONNECT,
            http::Method::TRACE => Method::TRACE,
            _ => panic!("Could not understand http method."),
        }
    }
}

pub trait SdkExecutor {
    async fn call<T: for<'de> Deserialize<'de>, S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        query_params: Q,
    ) -> TicketsResult<T>;

    async fn call_with_body<
        T: for<'de> Deserialize<'de>,
        B: Serialize,
        S: Into<String>,
        Q: Serialize,
    >(
        &self,
        method: Method,
        path: S,
        body: B,
        query_params: Q,
    ) -> TicketsResult<T>;

    async fn invoke<S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        query_params: Q,
    ) -> TicketsResult<StatusCode>;

    async fn invoke_with_body<B: Serialize, S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        body: B,
        query_params: Q,
    ) -> TicketsResult<StatusCode>;
}

macro_rules! sdk_permutation {
    ($($name:ident$(<$($generics:ident),*>)? {
        $backing_fn:ident($($extra_function_tokens:tt)*) -> $return_type:ty {
            $inner_call:ident($($binder_function_extras:tt)*)
        }

        Restrict { $($restrictions:tt)* } $(where $($where_clauses:tt)*)?
    })*) => {
        $(
        pub trait $name$(<$($generics),*>)? {
            async fn $backing_fn(executor: &impl crate::client::SdkExecutor, $($extra_function_tokens)*) -> TicketsResult<$return_type>;
        }

        impl<T$(, $($generics),*)?> $name$(<$($generics),*>)? for T
        where
            T: crate::routes::SdkRoute<$($restrictions)*>,
            $($($where_clauses)*)?
        {
            async fn $backing_fn(executor: &impl crate::client::SdkExecutor, $($extra_function_tokens)*) -> TicketsResult<$return_type> {
                executor.$inner_call(MethodWrapper(T::method()).into(), T::route(), $($binder_function_extras)*).await
            }
        }
        )*
    }
}

sdk_permutation! {
    SdkCall<ResponseType> {
        call() -> ResponseType {
            call(())
        }

        Restrict {
            Body = Empty,
            Response = ResponseType,
            QueryParams = Empty
        } where
            ResponseType: for<'de> Deserialize<'de>
    }

    SdkCallWithParams<ResponseType, QueryParams> {
        call_with_query(query_params: QueryParams) -> ResponseType {
            call(query_params)
        }

        Restrict {
            Body = Empty,
            Response = ResponseType,
            QueryParams = QueryParams,
        } where
            ResponseType: for<'de> Deserialize<'de>,
            QueryParams: Serialize
    }

    SdkCallWithBody<Body, ResponseType> {
        call_with_body(body: Body) -> ResponseType {
            call_with_body(body, ())
        }

        Restrict {
            Body = Body,
            Response = ResponseType,
            QueryParams = Empty
        } where
            ResponseType: for<'de> Deserialize<'de>,
            Body: Serialize

    }

    SdkCallWithBodyAndParams<ResponseType, Body, QueryParams> {
        call_with_body_and_query(body: Body, query_params: QueryParams) -> ResponseType {
            call_with_body(body, query_params)
        }

        Restrict {
            Body = Body,
            Response = ResponseType,
            QueryParams = QueryParams
        } where
            ResponseType: for<'de> Deserialize<'de>,
            Body: Serialize,
            QueryParams: Serialize

    }

    SdkInvoke {
        invoke() -> reqwest::StatusCode {
            invoke(())
        }

        Restrict {
            Body = Empty,
            Response = Empty,
            QueryParams = Empty
        }
    }

    SdkInvokeWithParams<QueryParams> {
        invoke_with_params(query_params: QueryParams) -> reqwest::StatusCode {
            invoke(query_params)
        }

        Restrict {
            Body = Empty,
            Response = Empty,
            QueryParams = QueryParams
        } where
            QueryParams: Serialize
    }

    SdkInvokeWithBody<Body> {
        invoke_with_body(body: Body) -> reqwest::StatusCode {
            invoke_with_body(body, ())
        }

        Restrict {
            Body = Body,
            Response = Empty,
            QueryParams = Empty
        } where
            Body: Serialize
    }

    SdkInvokeWithBodyAndParams<Body, QueryParams> {
        invoke_with_body_and_params(body: Body, query_params: QueryParams) -> reqwest::StatusCode {
            invoke_with_body(body, query_params)
        }

        Restrict {
            Body = Body,
            Response = Empty,
            QueryParams = QueryParams
        } where
            Body: Serialize,
            QueryParams: Serialize
    }
}

#[derive(Clone)]
pub struct InternalSdk {
    headers: HeaderMap,
    base_url: Url,
    jwt: Arc<JwtConfig>,
}

impl TryFrom<(String, JwtConfig, &'static str)> for InternalSdk {
    type Error = ParsingError;

    fn try_from(
        (base_url, jwt, gateway): (String, JwtConfig, &'static str),
    ) -> Result<Self, Self::Error> {
        (base_url, Arc::new(jwt), gateway).try_into()
    }
}

impl TryFrom<(String, Arc<JwtConfig>, &'static str)> for InternalSdk {
    type Error = ParsingError;

    fn try_from(
        (base_url, jwt, gateway): (String, Arc<JwtConfig>, &'static str),
    ) -> Result<Self, Self::Error> {
        Ok((Url::parse(&base_url)?, jwt, gateway).into())
    }
}

impl From<(Url, JwtConfig, &'static str)> for InternalSdk {
    fn from((base_url, jwt, gateway): (Url, JwtConfig, &'static str)) -> Self {
        (base_url, Arc::new(jwt), gateway).into()
    }
}

impl From<(Url, Arc<JwtConfig>, &'static str)> for InternalSdk {
    fn from((base_url, jwt, gateway): (Url, Arc<JwtConfig>, &'static str)) -> Self {
        Self::create(base_url, jwt, gateway)
    }
}

impl InternalSdk {
    // 1 hour = 60 seconds * 60 minutes
    pub const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60);

    pub(crate) fn create(url: Url, config: Arc<JwtConfig>, gateway: &'static str) -> Self {
        let mut headers = HeaderMap::with_capacity(1);
        headers.insert("x-gateway", HeaderValue::from_static(gateway));
        Self {
            headers,
            base_url: url,
            jwt: config,
        }
    }

    pub fn sign_client(
        &self,
        accessor: JwtAccessor,
        ttl: Duration,
    ) -> TicketsResult<SignedTicketClient> {
        SignedTicketClient::new(
            self.base_url.clone(),
            JwtData { accessor },
            ttl,
            self.jwt.clone(),
            self.headers.clone(),
        )
    }
}

pub struct TokenClaim {
    token: String,
    expiration: i64,
}

impl TokenClaim {
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn expiration(&self) -> i64 {
        self.expiration
    }
}

#[derive(Clone)]
pub struct SignedTicketClient {
    base_url: Url,
    client: Client,
    pub token_claim: Arc<RwLock<TokenClaim>>,
    pub data: JwtData,
    ttl: Duration,
    jwt_partial: Arc<JwtConfig>,
    headers: HeaderMap,
}

impl SignedTicketClient {
    pub(crate) fn new(
        base_url: Url,
        data: JwtData,
        ttl: Duration,
        jwt_partial: Arc<JwtConfig>,
        headers: HeaderMap,
    ) -> TicketsResult<Self> {
        let (token, claims) = jwt_partial.generate(data.clone(), ttl)?;

        Ok(Self {
            base_url,
            client: ClientBuilder::new()
                .timeout(Duration::from_secs(30))
                .build()?,
            token_claim: Arc::new(RwLock::new(TokenClaim {
                token,
                expiration: claims.exp,
            })),
            data,
            ttl,
            jwt_partial,
            headers,
        })
    }

    async fn ensure_token(&self) -> TicketsResult<String> {
        let read_token = self.token_claim.read().await;

        if read_token.expiration > chrono::Utc::now().timestamp() {
            drop(read_token);
            let mut write_token = self.token_claim.write().await;
            let (new_token, claims) = self.jwt_partial.generate(self.data.clone(), self.ttl)?;
            write_token.token = new_token.clone();
            write_token.expiration = claims.exp + Duration::from_secs(5 * 60).as_millis() as i64;
            drop(write_token);
            return Ok(new_token);
        }
        let token = read_token.token.clone();
        drop(read_token);
        Ok(token)
    }

    async fn prepare_request<S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        query_params: Q,
    ) -> TicketsResult<RequestBuilder> {
        let request = RequestBuilder::from_parts(
            self.client.clone(),
            Request::new(
                method,
                self.base_url
                    .join(&path.into())
                    .map_err(ParsingError::from)?,
            ),
        );
        let bearer = request.bearer_auth(self.ensure_token().await?);
        Ok(bearer.query(&query_params).headers(self.headers.clone()))
    }

    async fn prepare_request_with_body<S: Into<String>, B: Serialize, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        body: B,
        query_params: Q,
    ) -> TicketsResult<RequestBuilder> {
        Ok(self
            .prepare_request(method, path, query_params)
            .await?
            .json(&body))
    }

    async fn parse_err(response: Response) -> TicketsResult<TicketsError> {
        let value = response.json::<NetworkError>().await?;
        Ok(TicketsError::Network(value))
    }

    async fn parse<B: for<'de> Deserialize<'de>>(response: Response) -> TicketsResult<B> {
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Self::parse_err(response).await?)
        }
    }

    async fn dispose(response: Response) -> TicketsResult<reqwest::StatusCode> {
        if response.status().is_success() {
            Ok(response.status())
        } else {
            Err(Self::parse_err(response).await?)
        }
    }
}

impl SdkExecutor for SignedTicketClient {
    async fn call<T: for<'de> Deserialize<'de>, S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        query_params: Q,
    ) -> TicketsResult<T> {
        let prep: RequestBuilder = self.prepare_request(method, path, query_params).await?;
        let send = prep.send().await?;
        Self::parse(send).await
    }

    async fn call_with_body<
        T: for<'de> Deserialize<'de>,
        B: Serialize,
        S: Into<String>,
        Q: Serialize,
    >(
        &self,
        method: Method,
        path: S,
        body: B,
        query_params: Q,
    ) -> TicketsResult<T> {
        let prep: RequestBuilder = self
            .prepare_request_with_body(method, path, body, query_params)
            .await?;
        let send = prep.send().await?;
        Self::parse(send).await
    }

    async fn invoke<S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        query_params: Q,
    ) -> TicketsResult<reqwest::StatusCode> {
        let prep: RequestBuilder = self.prepare_request(method, path, query_params).await?;
        let send = prep.send().await?;
        Self::dispose(send).await
    }

    async fn invoke_with_body<B: Serialize, S: Into<String>, Q: Serialize>(
        &self,
        method: Method,
        path: S,
        body: B,
        query_params: Q,
    ) -> TicketsResult<reqwest::StatusCode> {
        let prep: RequestBuilder = self
            .prepare_request_with_body(method, path, body, query_params)
            .await?;
        let send = prep.send().await?;
        Self::dispose(send).await
    }
}
