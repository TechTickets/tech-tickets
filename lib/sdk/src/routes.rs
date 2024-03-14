#[cfg(all(feature = "server", not(feature = "client")))]
use axum::http::{Method, StatusCode};
#[cfg(feature = "client")]
use reqwest::{Method, StatusCode};

use errors::TicketsResult;
use serde::{Deserialize, Serialize};

/// Marker struct which cannot be serialized or deserialized.
pub struct Empty;

pub trait SdkRoute {
    type Body = Empty;
    type Response = Empty;
    type QueryParams = Empty;

    fn route() -> &'static str;

    fn method() -> Method;
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
            async fn $backing_fn(executor: &impl crate::routes::SdkExecutor, $($extra_function_tokens)*) -> TicketsResult<$return_type>;
        }

        impl<T$(, $($generics),*)?> $name$(<$($generics),*>)? for T
        where
            T: crate::routes::SdkRoute<$($restrictions)*>,
            $($($where_clauses)*)?
        {
            async fn $backing_fn(executor: &impl crate::routes::SdkExecutor, $($extra_function_tokens)*) -> TicketsResult<$return_type> {
                executor.$inner_call(T::method(), T::route(), $($binder_function_extras)*).await
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
        invoke() -> StatusCode {
            invoke(())
        }

        Restrict {
            Body = Empty,
            Response = Empty,
            QueryParams = Empty
        }
    }

    SdkInvokeWithParams<QueryParams> {
        invoke_with_params(query_params: QueryParams) -> StatusCode {
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
        invoke_with_body(body: Body) -> StatusCode {
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
        invoke_with_body_and_params(body: Body, query_params: QueryParams) -> StatusCode {
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

pub mod consumer {
    use super::SdkRoute;
    #[cfg(all(feature = "axum", not(feature = "client")))]
    use axum::http::Method;
    #[cfg(feature = "client")]
    use reqwest::Method;
    use uuid::Uuid;

    pub struct SubmitTicket;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct SubmitTicketBody {
        pub app_id: Uuid,
        pub message: String,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct SubmitTicketResponse {
        pub ticket_id: Uuid,
    }

    impl SdkRoute for SubmitTicket {
        type Body = SubmitTicketBody;
        type Response = SubmitTicketResponse;

        fn route() -> &'static str {
            "/consumer/submit_ticket"
        }

        fn method() -> Method {
            Method::POST
        }
    }
}

pub mod staff {
    use super::SdkRoute;
    use auth::UserRole;
    #[cfg(all(feature = "server", not(feature = "client")))]
    use axum::http::Method;
    use errors::ParsingError;
    #[cfg(feature = "client")]
    use reqwest::Method;
    use uuid::Uuid;

    pub struct Login;

    impl SdkRoute for Login {
        fn route() -> &'static str {
            "/staff/login"
        }

        fn method() -> Method {
            Method::GET
        }
    }

    pub struct PromoteStaff;

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct PromoteStaffRequest {
        staff_user_id: u64,
        role: UserRole,
    }

    pub struct ToggleGateway;

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct ToggleGatewayBody {
        pub app_id: Uuid,
        pub enabled: bool,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct ToggleGatewayResponse {
        pub gateway: String,
        pub enabled: bool,
    }

    impl SdkRoute for ToggleGateway {
        type Body = ToggleGatewayBody;
        type Response = ToggleGatewayResponse;

        fn route() -> &'static str {
            "/staff/toggle_gateway"
        }

        fn method() -> Method {
            Method::POST
        }
    }

    pub struct CreateApp;

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct CreateAppBody {
        pub app_name: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct CreateAppResponse {
        pub app_id: Uuid,
    }

    impl SdkRoute for CreateApp {
        type Body = CreateAppBody;
        type Response = CreateAppResponse;

        fn route() -> &'static str {
            "/staff/create_app"
        }

        fn method() -> Method {
            Method::POST
        }
    }

    // pub struct LinkDiscordGuildId;
    //
    // #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Hash, Eq, PartialEq)]
    // pub enum GuildPurpose {
    //     Consumer,
    //     Management,
    // }
    //
    // impl std::fmt::Display for GuildPurpose {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         match self {
    //             GuildPurpose::Consumer => write!(f, "consumer"),
    //             GuildPurpose::Management => write!(f, "management"),
    //         }
    //     }
    // }
    //
    // impl TryFrom<String> for GuildPurpose {
    //     type Error = ParsingError;
    //
    //     fn try_from(s: String) -> Result<Self, Self::Error> {
    //         Ok(match s.as_str() {
    //             "consumer" => GuildPurpose::Consumer,
    //             "management" => GuildPurpose::Management,
    //             _ => return Err(ParsingError::InvalidGuildPurpose(s)),
    //         })
    //     }
    // }
    //
    // #[derive(serde::Serialize, serde::Deserialize, Debug)]
    // pub struct LinkDiscordGuildIdBody {
    //     pub app_id: Uuid,
    //     pub guild_id: u64,
    //     pub guild_purpose: GuildPurpose,
    // }
    //
    // impl SdkRoute for LinkDiscordGuildId {
    //     type Body = LinkDiscordGuildIdBody;
    //
    //     fn route() -> &'static str {
    //         "/staff/link_discord_guild_id"
    //     }
    //
    //     fn method() -> Method {
    //         Method::POST
    //     }
    // }
    //
    // #[derive(serde::Serialize, serde::Deserialize, Debug)]
    // pub struct GetBulkGuildDataBody {
    //     pub guild_ids: Vec<u64>,
    // }
    //
    // #[derive(serde::Serialize, serde::Deserialize, Debug)]
    // pub struct GuildData {
    //     pub guild_id: u64,
    //     pub app_id: Uuid,
    //     pub guild_purpose: GuildPurpose,
    // }
    //
    // #[derive(serde::Serialize, serde::Deserialize, Debug)]
    // pub struct GetBulkGuildDataResponse {
    //     pub guild_data: Vec<GuildData>,
    // }
    //
    // pub struct GetBulkGuildData;
    //
    // impl SdkRoute for GetBulkGuildData {
    //     type Body = GetBulkGuildDataBody;
    //     type Response = GetBulkGuildDataResponse;
    //
    //     fn route() -> &'static str {
    //         "/staff/get_bulk_guild_data"
    //     }
    //
    //     fn method() -> Method {
    //         Method::POST
    //     }
    // }
    //
    // #[derive(serde::Serialize, serde::Deserialize, Debug)]
    // pub struct GetGuildDataParams {
    //     pub guild_id: u64,
    // }
    //
    // pub struct GetGuildData;
    //
    // impl SdkRoute for GetGuildData {
    //     type Response = GuildData;
    //     type QueryParams = GetGuildDataParams;
    //
    //     fn route() -> &'static str {
    //         "/staff/get_guild_data"
    //     }
    //
    //     fn method() -> Method {
    //         Method::GET
    //     }
    // }
}
