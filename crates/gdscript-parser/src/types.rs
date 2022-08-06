use nom::{error::VerboseError, IResult};
use nom_locate::LocatedSpan;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxResult<T> = Result<T, BoxError>;
pub type Span<'a> = LocatedSpan<&'a str>;
pub type Res<'a, U> = IResult<Span<'a>, U, VerboseError<Span<'a>>>;
