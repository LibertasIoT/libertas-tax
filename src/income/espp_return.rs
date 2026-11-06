use crate::espp_sale::{EsppSale, qualifying_dates};
use crate::espp_source::{EsppAcquisitionChoice, EsppAcquisitionFacts, EsppSaleAnswers, Form3922};
use libertas_macros::*;

/// Acquisition choice shown on the Investment sales topic
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EsppAcquisitionPage {
    /// A Form 3922 already entered in Documents
    Printed {
        /// Acquisition document
        #[libertas_enum_source("^<DocumentQuestion>.espp_acquisitions")]
        document_id: i64,
    },
    /// Actual acquisition records when no Form 3922 is available
    Records { facts: EsppAcquisitionFacts },
}
impl From<EsppAcquisitionChoice> for EsppAcquisitionPage {
    fn from(value: EsppAcquisitionChoice) -> Self {
        match value {
            EsppAcquisitionChoice::Printed { document_id } => Self::Printed { document_id },
            EsppAcquisitionChoice::Records { facts } => Self::Records { facts },
        }
    }
}
impl From<EsppAcquisitionPage> for EsppAcquisitionChoice {
    fn from(value: EsppAcquisitionPage) -> Self {
        match value {
            EsppAcquisitionPage::Printed { document_id } => Self::Printed { document_id },
            EsppAcquisitionPage::Records { facts } => Self::Records { facts },
        }
    }
}

fn source_price(value: &crate::SourceCents) -> Option<i64> {
    match value {
        crate::SourceCents::Reported { amount } => Some(*amount),
        crate::SourceCents::NotReported => Some(0),
        crate::SourceCents::Unavailable => None,
    }
}

pub(crate) fn answers(draft: &crate::FederalDraft, id: i64) -> Option<&EsppSaleAnswers> {
    draft.screening.as_ref()?.espp_sale_answers.iter().find(|row| row.document_id == id)
}

pub(crate) fn is_special(draft: &crate::FederalDraft, id: i64) -> bool {
    answers(draft, id).is_some_and(|answer| answer.more.special)
}

fn lot(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Result<EsppSale, crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue as Issue;
    let crate::FederalIncome::EsppSaleSource { data: paper } = &entry.income else {
        unreachable!()
    };
    let unanswered;
    let answer = match answers(draft, entry.id) {
        Some(answer) => answer,
        None if paper.plan_record.is_some() => {
            unanswered = EsppSaleAnswers::unanswered(entry.id);
            &unanswered
        }
        None => return Err(Issue::new("ESPP_ACQUISITION_NEEDED").at_field("acquisition")),
    };
    // The issue for a qualifying sale whose grant-date price no source shows, located at the
    // answer that supplies it.
    let price_needed: fn() -> Issue;
    let facts = if let Some(record) = &paper.plan_record {
        price_needed =
            || Issue::new("ESPP_RECORD_GRANT_PRICE_NEEDED").at_field("fixed_grant_price");
        EsppAcquisitionFacts {
            granted: record.granted,
            exercised: record.exercised,
            grant_market: record.grant_market,
            market_price: record.market_price,
            exercise_price: record.exercise_price,
            grant_price: record.grant_price.map_or_else(
                || answer.fixed_grant_price.clone(),
                |amount| crate::SourceCents::Reported { amount },
            ),
        }
    } else {
        match answer
            .acquisition
            .as_ref()
            .ok_or_else(|| Issue::new("ESPP_ACQUISITION_NEEDED").at_field("acquisition"))?
        {
            EsppAcquisitionChoice::Records { facts } => {
                price_needed = || {
                    Issue::new("ESPP_RECORDS_GRANT_PRICE_NEEDED")
                        .at_field("grant_price")
                        .in_field("facts")
                        .in_field("Records")
                        .in_field("acquisition")
                };
                facts.clone()
            }
            EsppAcquisitionChoice::Printed { document_id } => {
                let source = crate::source_associations::SourceAssociations::new(draft)
                    .espp_acquisition(*document_id, entry.owner.owned())
                    .map_err(|_| {
                        Issue::new("ESPP_ACQUISITION_LINK_NEEDED")
                            .at_field("document_id")
                            .in_field("Printed")
                            .in_field("acquisition")
                    })?;
                let form = &Form3922::from(&source.form);
                if paper.shares > form.shares {
                    return Err(Issue::new("ESPP_SOLD_ABOVE_TRANSFERRED")
                        .at_field("document_id")
                        .in_field("Printed")
                        .in_field("acquisition"));
                }
                price_needed =
                    || Issue::new("ESPP_FIXED_GRANT_PRICE_NEEDED").at_field("fixed_grant_price");
                EsppAcquisitionFacts {
                    granted: form.granted,
                    exercised: form.exercised,
                    grant_market: form.grant_market,
                    market_price: form.market_price,
                    exercise_price: form.exercise_price,
                    grant_price: form.grant_price.map_or_else(
                        || source.fixed_grant_price.clone(),
                        |amount| crate::SourceCents::Reported { amount },
                    ),
                }
            }
        }
    };
    if facts.granted > facts.exercised {
        return Err(Issue::new("ESPP_EXERCISED_BEFORE_GRANTED").at_field("acquisition"));
    }
    if facts.exercised > paper.sold {
        return Err(Issue::new("ESPP_SOLD_BEFORE_EXERCISED").at_field("acquisition"));
    }
    let more = &answer.more;
    let sale = EsppSale {
        granted: facts.granted,
        exercised: facts.exercised,
        sold: paper.sold,
        shares: paper.shares,
        grant_market: facts.grant_market,
        grant_price: source_price(&facts.grant_price),
        exercise_price: facts.exercise_price,
        market_price: facts.market_price,
        acquisition_costs: more.acquisition_costs,
        proceeds: paper.proceeds,
        selling_fees: more.selling_fees,
        in_wages: false,
        wash_sale: more.wash_sale,
        withholding: paper.withholding,
        reported_basis: paper.reported_basis,
        reporting: paper.reporting,
        special: more.special,
    };
    if sale.qualifying() && sale.grant_price.is_none() {
        return Err(price_needed());
    }
    // The existing monetary owner admits the complete lot before computing compensation or basis.
    if let Some(issue) = sale.wash_sale_issue() {
        return Err(issue);
    }
    if !sale.valid() {
        return Err(Issue::new("ESPP_LOT_REVIEW_NEEDED").at_field("acquisition"));
    }
    Ok(sale)
}

pub(crate) fn resolve(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Result<EsppSale, crate::validation::ValidationIssue> {
    if is_special(draft, entry.id) {
        return Err(crate::validation::ValidationIssue::new("WARN_ESPP_SALE_LEFT_OUT")
            .at_field("special")
            .in_field("more"));
    }
    let mut sale = lot(draft, entry)?;
    if !sale.special && sale.compensation() > 0 {
        sale.in_wages =
            answers(draft, entry.id).and_then(|answer| answer.wage_inclusion).ok_or_else(|| {
                crate::validation::ValidationIssue::new("ESPP_WAGE_INCLUSION_NEEDED")
                    .at_field("wage_inclusion")
            })?;
    }
    Ok(sale)
}

pub(crate) fn wage_needed(draft: &crate::FederalDraft, entry: &crate::FederalIncomeEntry) -> bool {
    !is_special(draft, entry.id) && lot(draft, entry).is_ok_and(|sale| sale.compensation() > 0)
}

fn needs_fixed_price(draft: &crate::FederalDraft, entry: &crate::FederalIncomeEntry) -> bool {
    if is_special(draft, entry.id) {
        return false;
    }
    let crate::FederalIncome::EsppSaleSource { data: paper } = &entry.income else {
        return false;
    };
    if let Some(record) = &paper.plan_record {
        return record.grant_price.is_none()
            && qualifying_dates(record.granted, record.exercised, paper.sold);
    }
    match answers(draft, entry.id).and_then(|answer| answer.acquisition.as_ref()) {
        Some(EsppAcquisitionChoice::Printed { document_id }) => {
            crate::source_associations::SourceAssociations::new(draft)
                .espp_acquisition(*document_id, entry.owner.owned())
                .is_ok_and(|source| {
                    let form = Form3922::from(&source.form);
                    form.grant_price.is_none()
                        && qualifying_dates(form.granted, form.exercised, paper.sold)
                })
        }
        Some(EsppAcquisitionChoice::Records { .. }) | None => false,
    }
}
pub(crate) fn fixed_price_needed(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> bool {
    if !needs_fixed_price(draft, entry) {
        return false;
    }
    let Some(EsppAcquisitionChoice::Printed { document_id }) =
        answers(draft, entry.id).and_then(|answer| answer.acquisition.as_ref())
    else {
        return true;
    };
    !draft.income.iter().any(|other| other.id < entry.id && needs_fixed_price(draft, other)
        && matches!(answers(draft, other.id).and_then(|a| a.acquisition.as_ref()), Some(EsppAcquisitionChoice::Printed { document_id: other_id }) if other_id == document_id))
}
pub(crate) fn fixed_price(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> crate::SourceCents {
    if matches!(&entry.income, crate::FederalIncome::EsppSaleSource { data } if data.plan_record.is_some())
    {
        return answers(draft, entry.id)
            .map_or(crate::SourceCents::Unavailable, |answer| answer.fixed_grant_price.clone());
    }
    match answers(draft, entry.id).and_then(|answer| answer.acquisition.as_ref()) {
        Some(EsppAcquisitionChoice::Printed { document_id }) => {
            crate::source_associations::SourceAssociations::new(draft)
                .espp_acquisition(*document_id, entry.owner.owned())
                .map_or(crate::SourceCents::Unavailable, |source| source.fixed_grant_price.clone())
        }
        _ => crate::SourceCents::Unavailable,
    }
}

pub(crate) fn choices(
    draft: &crate::FederalDraft,
    owner: crate::Owner,
) -> alloc::vec::Vec<crate::RecordChoice> {
    draft
        .screening
        .iter()
        .flat_map(|s| &s.espp_acquisitions)
        .filter(|row| row.owner == owner)
        .map(|row| crate::RecordChoice {
            id: row.id,
            label: crate::Message::new(
                "TEXT",
                alloc::vec![crate::MessageArg::Text(row.form.corporation.name.clone())],
            )
            .encode(),
        })
        .collect()
}

pub(crate) fn reconcile(draft: &mut crate::FederalDraft) {
    let no_compensation: alloc::vec::Vec<i64> = draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, crate::FederalIncome::EsppSaleSource { .. }))
        .filter(|entry| {
            !is_special(draft, entry.id)
                && lot(draft, entry).is_ok_and(|sale| sale.compensation() == 0)
        })
        .map(|entry| entry.id)
        .collect();
    if let Some(screening) = &mut draft.screening {
        screening.espp_sale_answers.retain(|row| {
            draft.income.iter().any(|entry| {
                entry.id == row.document_id
                    && matches!(entry.income, crate::FederalIncome::EsppSaleSource { .. })
            })
        });
        for row in &mut screening.espp_sale_answers {
            if no_compensation.contains(&row.document_id) {
                row.wage_inclusion = None;
            }
        }
    }
}
