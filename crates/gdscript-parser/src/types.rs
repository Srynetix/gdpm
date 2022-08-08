use nom::{error::VerboseError, IResult};
use nom_locate::LocatedSpan;
use nom_tracable::TracableInfo;

pub type Span<'a> = LocatedSpan<&'a str, TracableInfo>;
pub type Res<'a, U> = IResult<Span<'a>, U, VerboseError<Span<'a>>>;
