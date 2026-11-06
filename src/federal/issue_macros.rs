// Call-site macros for issue_helpers::require_localized and issue_localized.
// The section name and the English text exist only in test builds (the
// functions take them as `#[cfg(test)]` parameters); the macros take them as
// ordinary arguments and drop them from the call outside test builds, so a
// call site needs no `#[cfg(test)]` lines. Arguments, in order:
//   require_localized!(result, ok, context, section, KEY, "English text")
//   issue_localized!(result, context, section, KEY, "English text")
// Runtime behavior (the stored key, the issue pushed when `ok` is false) and
// the test-only text are exactly those of the function call they replace.
macro_rules! require_localized {
    ($result:expr, $ok:expr, $context:expr, $section:expr, $key:expr, $english:expr $(,)?) => {
        $crate::issue_helpers::require_localized(
            $result,
            $ok,
            $context,
            #[cfg(test)]
            $section,
            $key,
            #[cfg(test)]
            $english,
        )
    };
}
macro_rules! issue_localized {
    ($result:expr, $context:expr, $section:expr, $key:expr, $english:expr $(,)?) => {
        $crate::issue_helpers::issue_localized(
            $result,
            $context,
            #[cfg(test)]
            $section,
            $key,
            #[cfg(test)]
            $english,
        )
    };
}
pub(crate) use {issue_localized, require_localized};
