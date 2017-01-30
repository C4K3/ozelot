/** Create an InvalidData io::Error with the description being a
 * formatted string */
macro_rules! io_error {
    ($fmtstr:tt) => { io_error!($fmtstr,) };
    ($fmtstr:tt, $( $args:expr ),* ) => {
        Err(::std::io::Error::new(::std::io::ErrorKind::Other,
                                  format!($fmtstr, $( $args ),* )));
    };
}

