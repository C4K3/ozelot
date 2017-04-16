//! The error types used

error_chain! {
    foreign_links {
        Curl(::curl::Error);
        Io(::std::io::Error);
        Openssl(::openssl::error::ErrorStack);
        Serde(::serde_json::Error);
        FromUtf8(::std::string::FromUtf8Error);
    }
}
