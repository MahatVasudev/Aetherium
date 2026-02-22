#[macro_export]
macro_rules! storage_assert {
    ($($arg:tt)*) => {
        return Err(StorageError::assertion_fail(format_args!($($arg)*)))
    };
}
