use crate::error::Error;

pub type ParserResult<T> = Option<Result<T, Error>>;

#[macro_export]
macro_rules! try_parse {
    ($e : expr) => {
        match e {
            Some(result) => match result {
                Ok(ok) => ok,
                Err(err) => return Some(Err(err)),
            },
            None => return None,
        }
    };
}
