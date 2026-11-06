// disaster items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// The Retirement topic's Form 8915-F checks (T3-8915): one earlier form per
// owner, tax year and disaster year (FORM_8915_TWICE), and across the page's
// 1099-R folds one line 1b amount per disaster (DISASTER_EARLIER_DIFFERS),
// the $22,000 line 1e limit per disaster (DISASTER_ABOVE_LIMIT, the client
// rule checks one 1099-R) and one lines 11/22 election per disaster year
// (DISASTER_ELECTION_DIFFERS). A spouse's forms only on a joint return.
pub(super) fn disaster_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
    // The designated disaster share of each 1099-R: its row's position, owner and answer.
    designations: &[(usize, Owner, &crate::DisasterDistribution)],
) -> Result<Vec<crate::DisasterCarry>, SaveFailure> {
    let mut limits: Vec<(Owner, u32, i64, i64)> = Vec::new();
    let mut elections: Vec<(Owner, u32, crate::DisasterSpread)> = Vec::new();
    for &(index, owner, disaster) in designations {
        let crate::DisasterDistribution::Qualified { fema, began, amount, earlier, spread, .. } =
            disaster
        else {
            continue;
        };
        match limits.iter_mut().find(|(o, id, ..)| *o == owner && id == fema) {
            Some((_, _, _, other)) if other != earlier => {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("DISASTER_EARLIER_DIFFERS")
                        .at_field("earlier")
                        .in_field("disaster"),
                    index,
                )
                .into());
            }
            Some((_, _, total, _)) => *total += amount,
            None => limits.push((owner, *fema, *amount, *earlier)),
        }
        let year = began / 10000;
        match elections.iter().find(|(o, y, _)| *o == owner && *y == year) {
            Some((_, _, other)) if other != spread => {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new("DISASTER_ELECTION_DIFFERS")
                        .at_field("spread")
                        .in_field("disaster"),
                    index,
                )
                .into());
            }
            Some(_) => {}
            None => elections.push((owner, year, *spread)),
        }
    }
    if let Some((owner, fema, _, earlier)) =
        limits.iter().find(|(_, _, total, earlier)| total + earlier > crate::disaster_source::LIMIT)
    {
        // Admission was decided once above, after the original complete group
        // scan. This prefix scan selects metadata only for that rejected group;
        // a later zero/tiny contribution must not hide the first crossing row.
        let mut running = *earlier;
        let (index, field) = designations
            .iter()
            .find_map(|&(index, item_owner, disaster)| {
                let crate::DisasterDistribution::Qualified { fema: source, amount, .. } = disaster
                else {
                    return None;
                };
                if item_owner != *owner || source != fema {
                    return None;
                }
                if running > crate::disaster_source::LIMIT {
                    return Some((index, "earlier"));
                }
                running += amount;
                (running > crate::disaster_source::LIMIT).then_some((index, "amount"))
            })
            .expect("failed complete disaster total has a crossing source row");
        return Err(submitted_data_answer_issue(
            ValidationIssue::new("DISASTER_ABOVE_LIMIT").at_field(field).in_field("disaster"),
            index,
        )
        .into());
    }
    let mut records: Vec<crate::DisasterCarry> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::Record { value: crate::IncomeRecord::Form8915(form) } = &item.document
        else {
            continue;
        };
        if item.owner == crate::DocumentOwner::Spouses && !interview.joint() {
            return Err(crate::interview::status(
                "MESSAGE_FORM_8915_F_YOUR_SPOUSE_S_FORMS_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            ).into());
        }
        if records.iter().any(|c| {
            c.owner == item.owner.owned()
                && core::mem::discriminant(&c.form.year) == core::mem::discriminant(&form.year)
                && c.form.disaster_year == form.disaster_year
        }) {
            return Err(submitted_data_answer_issue(
                ValidationIssue::new("FORM_8915_TWICE").at_record(),
                index,
            )
            .into());
        }
        records.push(crate::DisasterCarry { owner: item.owner.owned(), form: form.clone() });
    }
    Ok(records)
}
