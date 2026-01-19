/// Simple macro to create a new String, or convert from a &str to a String - basically just gets rid of String::from() / .to_owned() etc
#[macro_export]
macro_rules! S {
    () => {
        String::new()
    };
    ($s:expr) => {
        String::from($s)
    };
}

#[macro_export]
/// Sleep for a given number of milliseconds, is an async fn.
/// If no parameter supplied, defaults to 1000ms
macro_rules! sleep {
    () => {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    };
    ($ms:expr) => {
        tokio::time::sleep(std::time::Duration::from_millis($ms)).await;
    };
}

/// Simple macro to call `.clone()` on whatever is passed in
#[macro_export]
macro_rules! C {
    ($i:expr) => {
        $i.clone()
    };
}
