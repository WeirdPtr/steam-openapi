#[macro_export]
macro_rules! cargo_crate_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

#[macro_export]
macro_rules! user_agent_header {
    () => {{
        use $crate::cargo_crate_version;
        format!("Better Steam Web API Client v{}", cargo_crate_version!())
    }};
}