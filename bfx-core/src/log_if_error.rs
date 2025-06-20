use std::fmt::Display;

pub trait LogIfErrorExt<T> {
    fn log_if_error(self, action: &str);
    fn or_with_log(self, action: &str, default: T) -> T;
    fn or_with_log_default(self, action: &str) -> T
    where
        T: Default;
}

impl<T, E> LogIfErrorExt<T> for Result<T, E>
where
    E: Display,
{
    fn log_if_error(self, action: &str) {
        if let Err(err) = self {
            tracing::error!(%err, "encountered error while {}", action);
        }
    }

    fn or_with_log(self, action: &str, default: T) -> T {
        match self {
            Ok(v) => v,
            Err(err) => {
                tracing::error!(%err, "encountered error while {}", action);
                default
            }
        }
    }

    fn or_with_log_default(self, action: &str) -> T
    where
        T: Default,
    {
        self.or_with_log(action, T::default())
    }
}
