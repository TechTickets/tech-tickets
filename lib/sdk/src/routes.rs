use http::Method;

/// Marker struct which cannot be serialized or deserialized.
pub struct Empty;

pub trait SdkRoute {
    type Body = Empty;
    type Response = Empty;
    type QueryParams = Empty;

    fn route() -> &'static str;

    fn method() -> Method;
}

pub mod consumer {
    use super::SdkRoute;
    use http::Method;
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
    use http::Method;
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
}
