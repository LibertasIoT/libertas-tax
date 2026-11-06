use crate::diagnostics::{FederalIssue, ReviewNote};
use crate::return_state::FederalResult;

pub(crate) fn issue_localized(
    result: &mut FederalResult,
    context: &crate::IssueContext,
    #[cfg(test)] section: &str,
    key: &str,
    #[cfg(test)] english: &str,
) {
    result.issues.push(FederalIssue {
        context: context.clone(),
        #[cfg(test)]
        section: section.into(),
        #[cfg(test)]
        message: english.into(),
        display: crate::Message::new(key, alloc::vec![]),
    });
}

pub(crate) fn issue_message(
    result: &mut FederalResult,
    context: &crate::IssueContext,
    #[cfg(test)] section: &str,
    #[cfg(test)] english: impl Into<alloc::string::String>,
    display: crate::Message,
) {
    result.issues.push(FederalIssue {
        context: context.clone(),
        #[cfg(test)]
        section: section.into(),
        #[cfg(test)]
        message: english.into(),
        display,
    });
}

pub(crate) fn require_localized(
    result: &mut FederalResult,
    ok: bool,
    context: &crate::IssueContext,
    #[cfg(test)] section: &str,
    key: &str,
    #[cfg(test)] english: &str,
) {
    if !ok {
        issue_localized(
            result,
            context,
            #[cfg(test)]
            section,
            key,
            #[cfg(test)]
            english,
        );
    }
}

pub(crate) fn require_message(
    result: &mut FederalResult,
    ok: bool,
    context: &crate::IssueContext,
    #[cfg(test)] section: &str,
    #[cfg(test)] english: impl Into<alloc::string::String>,
    display: crate::Message,
) {
    if !ok {
        issue_message(
            result,
            context,
            #[cfg(test)]
            section,
            #[cfg(test)]
            english,
            display,
        );
    }
}

pub(crate) fn note_once(result: &mut FederalResult, message: crate::Message) {
    if !result.review_notes.iter().any(|n| n.context.is_none() && n.display == message) {
        result.review_notes.push(ReviewNote::localized(
            #[cfg(test)]
            message.english(),
            message,
        ));
    }
}

pub(crate) fn note_once_scoped(
    result: &mut FederalResult,
    message: crate::Message,
    context: crate::IssueContext,
) {
    let note = ReviewNote::scoped(
        #[cfg(test)]
        message.english(),
        message,
        context,
    );
    if !result.review_notes.iter().any(|existing| existing.same_notice(&note)) {
        result.review_notes.push(note);
    }
}
