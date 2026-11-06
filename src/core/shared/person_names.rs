use crate::common::{Message, MessageArg};
use alloc::vec::Vec;

// A person's name, the first and last name the person typed.
pub(crate) fn person_name(first: &str, last: &str) -> Vec<u8> {
    Message::new(
        "PERSON_NAME",
        alloc::vec![MessageArg::Text(first.into()), MessageArg::Text(last.into())],
    )
    .encode()
}
