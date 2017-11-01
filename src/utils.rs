#[macro_export]
macro_rules! error_formatter {
    ($prefix:expr) => { |error| format!("{} Error: {}", $prefix, error) }
}

#[macro_export]
macro_rules! handle_bad_hyper_response {
    ($prefix:expr) => { |response| match response.status() {
        hyper::StatusCode::Ok => Ok(response),
        _ => Err(format!("{} Bad response. Status: {}", $prefix, response.status()))
    }}
}

#[macro_export]
macro_rules! read_hyper_resp_body {
    ($prefix:expr) => {|response| response.body().concat2().map_err(error_formatter!($prefix)) }
}
