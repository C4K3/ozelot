//! The error types used

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Openssl(::openssl::error::ErrorStack);
        Reqwest(::reqwest::Error);
        Serde(::serde_json::Error);
    }
}
