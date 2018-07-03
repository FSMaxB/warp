//! dox?
use std::str::FromStr;

use ::error::Kind;
use ::filter::{Cons, Filter, filter_fn, HCons, HList};
use ::route;


/// Create an exact match path `Filter`.
///
/// This will try to match exactly to the current request path segment.
///
/// # Note
///
/// Exact path filters cannot be empty, or contain slashes.
pub fn path(p: &'static str) -> impl Filter<Extract=(), Error=::Error> + Copy {
    assert!(!p.is_empty(), "exact path segments should not be empty");
    assert!(!p.contains('/'), "exact path segments should not contain a slash: {:?}", p);

    segment(move |seg| {
        trace!("{:?}?: {:?}", p, seg);
        if seg == p {
            Ok(())
        } else {
            Err(Kind::NotFound.into())
        }
    })
}

/// Matches the end of a route.
pub fn index() -> impl Filter<Extract=(), Error=::Error> + Copy {
    filter_fn(move || {
        route::with(|route| {
            if route.path().is_empty() {
                Ok(())
            } else {
                Err(Kind::NotFound.into())
            }
        })
    })
}

/// Create an `Extract` path filter.
///
/// An `Extract` will try to parse a value from the current request path
/// segment, and if successful, the value is returned as the `Filter`'s
/// "extracted" value.
pub fn param<T: FromStr + Send>() -> impl Filter<Extract=Cons<T>, Error=::Error> + Copy {
    segment(|seg| {
        trace!("param?: {:?}", seg);
        T::from_str(seg)
            .map(|t| HCons(t, ()))
            .map_err(|_| Kind::NotFound.into())
    })
}

fn segment<F, U>(func: F) -> impl Filter<Extract=U, Error=::Error> + Copy
where
    F: Fn(&str) -> Result<U, ::Error> + Copy,
    U: HList + Send,
{
    filter_fn(move || {
        route::with(|route| {
            let (u, idx) = {
                let seg = route.path()
                    .splitn(2, '/')
                    .next()
                    .expect("split always has at least 1");
                (func(seg)?, seg.len())
            };
            route.set_unmatched_path(idx);
            Ok(u)
        })
    })
}

#[macro_export]
macro_rules! path {
    (@p $first:tt / $($tail:tt)*) => ({
        let __p = path!(@p $first);
        $(
        let __p = $crate::Filter::and(__p, path!(@p $tail));
        )*
        __p
    });
    (@p $param:ty) => (
        $crate::path::param::<$param>()
    );
    (@p $s:expr) => (
        $crate::path($s)
    );
    ($($pieces:tt)*) => (
        path!(@p $($pieces)*)
    );
}

