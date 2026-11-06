// Argument-free source messages use this shorthand. Typed arguments are explicit.
// Source-owned English error fixtures have no runtime rendering responsibility.
macro_rules! fixture_english {
    ($english:expr) => {{
        #[cfg(test)]
        {
            $english
        }
        #[cfg(not(test))]
        {
            ""
        }
    }};
}
pub(crate) use fixture_english;

macro_rules! msg {
    ($key:literal $(,)?) => {
        $crate::common::Message::new($key, alloc::vec![])
    };
}

// Record/page twins retain their native declarations and share only conversion bodies.
macro_rules! record_page_conversions {
    ($record:path, $page:path, [$($borrow:tt)*], |$value:ident, $target:ident $(, $source:ident)?| $body:expr) => {
        impl From<$($borrow)* $page> for $record {
            fn from($value: $($borrow)* $page) -> Self {
                use $record as $target;
                $(use $page as $source;)?
                $body
            }
        }
        impl From<$($borrow)* $record> for $page {
            fn from($value: $($borrow)* $record) -> Self {
                use $page as $target;
                $(use $record as $source;)?
                $body
            }
        }
    };
}
