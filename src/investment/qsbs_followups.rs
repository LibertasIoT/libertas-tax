// QSBS personal facts belong to the accepted source, never the printed statement.
use crate::validation::ValidationIssue;
use crate::{DocumentAnswer as A, QsbIssuerSale};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Your answer about this stock sale
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbsBooleanAnswer {
    /// Your answer
    /// [NullText]
    /// Choose your answer
    #[libertas_read_only]
    #[libertas_link("AnswerQsbsBoolean")]
    pub value: Option<bool>,
    /// Known answer for corrections
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub known: Vec<bool>,
}

/// Known prior eligible gain
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbsGainValue {
    /// Eligible gain for this issuer
    #[libertas_money("USD", 2)]
    pub value: i64,
}

/// Prior eligible gain for this issuer
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbsGainAnswer {
    /// Prior eligible gain
    /// [NullText]
    /// Enter the reconciled prior gain
    #[libertas_read_only]
    #[libertas_link("AnswerQsbsPriorGain")]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: Option<i64>,
    /// Known answer for corrections
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub known: Vec<QsbsGainValue>,
}

/// Issuer empowerment-zone records
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum QsbsZoneCertification {
    /// The certification or historical records are unavailable; leave the increase unclaimed
    NoRecords,
    /// No, the issuer did not qualify for the additional exclusion
    No,
    /// Yes, issuer records establish qualification for the additional exclusion
    Yes,
}
impl QsbsZoneCertification {
    pub(crate) fn value(self) -> Option<bool> {
        match self {
            Self::NoRecords => None,
            Self::No => Some(false),
            Self::Yes => Some(true),
        }
    }
}

/// Historical valuation for this sold lot
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum QsbsZoneValuation {
    /// No pre-2019 empowerment-zone gain is established
    NoRecords,
    /// Gain attributable to time through December 31, 2018 is established
    Established {
        /// Gain, limited to the lot's realized gain
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0, max = 100000000000)]
        gain: i64,
    },
}
impl QsbsZoneValuation {
    pub(crate) fn gain(self) -> Option<i64> {
        match self {
            Self::NoRecords => None,
            Self::Established { gain } => Some(gain),
        }
    }
}

/// Your answer about issuer zone records
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbsZoneAnswer {
    /// Issuer records
    /// [NullText]
    /// Choose the issuer records answer
    #[libertas_read_only]
    #[libertas_link("AnswerQsbsZone")]
    pub value: Option<QsbsZoneCertification>,
    /// Known answer for corrections
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub known: Vec<QsbsZoneCertification>,
}

/// Your answer about this lot's historical valuation
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbsValuationAnswer {
    /// Sold lot from the accepted statement
    #[libertas_read_only]
    pub lot: crate::QsbsPaperLot,
    /// Historical valuation
    /// [NullText]
    /// Choose the historical valuation answer
    #[libertas_read_only]
    #[libertas_link("AnswerQsbsValuation")]
    pub value: Option<QsbsZoneValuation>,
    /// Known answer for corrections
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub known: Vec<QsbsZoneValuation>,
}

fn boolean(value: Option<bool>) -> A {
    A::QsbsBoolean { value: QsbsBooleanAnswer { known: value.into_iter().collect(), value } }
}

fn zone_lot(lot: &crate::QsbStockLot) -> bool {
    (crate::year_constants::SECTION_1202_ZONE_FIRST_DATE
        ..=crate::year_constants::SECTION_1202_FIFTY_PERCENT_LAST_DATE)
        .contains(&lot.issued)
}

// Keep answered predecessors available for correction. A negative answer ends
// the existing exclusion conjunction, so its irrelevant successors are not asked.
pub(crate) fn questions(data: &QsbIssuerSale) -> Vec<(String, &'static str, A)> {
    let mut rows = Vec::new();
    for (field, prompt, value, proceed) in [
        ("special", "QSBS_QUESTION_SPECIAL", data.special, false),
        ("issuer_qualified", "QSBS_QUESTION_ISSUER", data.issuer_qualified, true),
        ("active_business", "QSBS_QUESTION_ACTIVE", data.active_business, true),
        ("no_disqualification", "QSBS_QUESTION_REDEMPTIONS", data.no_disqualification, true),
        ("complete_history", "QSBS_QUESTION_HISTORY", data.complete_history, true),
    ] {
        rows.push((alloc::format!("qsbs_{field}"), prompt, boolean(value)));
        if value != Some(proceed) {
            return rows;
        }
    }
    rows.push((
        "qsbs_prior_gain".into(),
        "QSBS_QUESTION_PRIOR_GAIN",
        A::QsbsPriorGain {
            value: QsbsGainAnswer {
                value: data.prior_eligible_gain,
                known: data
                    .prior_eligible_gain
                    .into_iter()
                    .map(|value| QsbsGainValue { value })
                    .collect(),
            },
        },
    ));
    if data.lots.iter().any(zone_lot) {
        rows.push((
            "qsbs_zone".into(),
            "QSBS_QUESTION_ZONE",
            A::QsbsZone {
                value: QsbsZoneAnswer {
                    value: data.zone_60_exclusion,
                    known: data.zone_60_exclusion.into_iter().collect(),
                },
            },
        ));
        if data.zone_60_exclusion == Some(QsbsZoneCertification::Yes) {
            for (index, lot) in data.lots.iter().enumerate().filter(|(_, lot)| zone_lot(lot)) {
                rows.push((
                    alloc::format!("qsbs_zone_gain/{index}"),
                    "QSBS_QUESTION_ZONE_GAIN",
                    A::QsbsZoneGain {
                        value: QsbsValuationAnswer {
                            lot: crate::QsbsPaperLot::from(lot),
                            value: lot.zone_gain_through_2018,
                            known: lot.zone_gain_through_2018.into_iter().collect(),
                        },
                    },
                ));
            }
        }
    }
    rows
}

pub(crate) fn needed(draft: &crate::FederalDraft) -> bool {
    draft.income.iter().any(|entry| match &entry.income {
        crate::FederalIncome::QualifiedSmallBusinessSale { data } => {
            questions(data).iter().any(|(_, _, answer)| missing(answer))
        }
        _ => false,
    })
}

pub(crate) fn missing(answer: &A) -> bool {
    matches!(answer, A::QsbsBoolean { value } if value.value.is_none())
        || matches!(answer, A::QsbsPriorGain { value } if value.value.is_none())
        || matches!(answer, A::QsbsZone { value } if value.value.is_none())
        || matches!(answer, A::QsbsZoneGain { value } if value.value.is_none())
}

pub(crate) fn apply(
    data: &mut QsbIssuerSale,
    field: &str,
    answer: A,
    required: bool,
) -> Result<(), ValidationIssue> {
    let invalid = || ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value");
    if required && missing(&answer) {
        return Err(ValidationIssue::new("QSBS_ANSWER_NEEDED").at_field("value"));
    }
    match (field, answer) {
        ("qsbs_special", A::QsbsBoolean { value }) => data.special = value.value,
        ("qsbs_issuer_qualified", A::QsbsBoolean { value }) => data.issuer_qualified = value.value,
        ("qsbs_active_business", A::QsbsBoolean { value }) => data.active_business = value.value,
        ("qsbs_no_disqualification", A::QsbsBoolean { value }) => {
            data.no_disqualification = value.value
        }
        ("qsbs_complete_history", A::QsbsBoolean { value }) => data.complete_history = value.value,
        ("qsbs_prior_gain", A::QsbsPriorGain { value }) => {
            data.prior_eligible_gain = value.value;
        }
        ("qsbs_zone", A::QsbsZone { value }) => data.zone_60_exclusion = value.value,
        (field, A::QsbsZoneGain { value }) => {
            let index = field
                .strip_prefix("qsbs_zone_gain/")
                .and_then(|index| index.parse::<usize>().ok())
                .ok_or_else(invalid)?;
            let lot = data.lots.get_mut(index).ok_or_else(invalid)?;
            lot.zone_gain_through_2018 = value.value;
            if let Some(issue) = lot.issue() {
                return Err(issue.map_field_prefix(&["zone_gain_through_2018"], &["value"]));
            }
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

impl QsbIssuerSale {
    pub(crate) fn keep_answers(&mut self, before: &Self) {
        if self.issuer_key() != before.issuer_key() {
            return;
        }
        self.special = before.special;
        self.issuer_qualified = before.issuer_qualified;
        self.active_business = before.active_business;
        self.no_disqualification = before.no_disqualification;
        self.complete_history = before.complete_history;
        self.prior_eligible_gain = before.prior_eligible_gain;
        self.zone_60_exclusion = before.zone_60_exclusion;
        let mut issue_dates: Vec<_> = self.lots.iter().map(|lot| lot.issued).collect();
        let mut old_issue_dates: Vec<_> = before.lots.iter().map(|lot| lot.issued).collect();
        issue_dates.sort_unstable();
        old_issue_dates.sort_unstable();
        let issue_dates_same = issue_dates == old_issue_dates;
        let mut holding_dates: Vec<_> =
            self.lots.iter().map(|lot| (lot.issued, lot.sold)).collect();
        let mut old_holding_dates: Vec<_> =
            before.lots.iter().map(|lot| (lot.issued, lot.sold)).collect();
        holding_dates.sort_unstable();
        old_holding_dates.sort_unstable();
        let holding_dates_same = holding_dates == old_holding_dates;
        if !issue_dates_same {
            self.issuer_qualified = None;
            self.no_disqualification = None;
        }
        if !holding_dates_same {
            self.active_business = None;
            self.zone_60_exclusion = None;
        }
        if self.lots.len() != before.lots.len() {
            self.complete_history = None;
        }
        // Match the actual lot, rather than retaining a valuation by list index
        // after a row insertion, removal, reorder or economic correction.
        for lot in &mut self.lots {
            let mut matches = before.lots.iter().filter(|old| {
                old.issued == lot.issued
                    && old.sold == lot.sold
                    && old.basis == lot.basis
                    && old.proceeds == lot.proceeds
            });
            if let Some(old) = matches.next()
                && matches.next().is_none()
            {
                lot.zone_gain_through_2018 = old.zone_gain_through_2018;
            } else {
                let same_dates: Vec<_> = before
                    .lots
                    .iter()
                    .filter(|old| old.issued == lot.issued && old.sold == lot.sold)
                    .collect();
                if !same_dates.is_empty()
                    && same_dates
                        .iter()
                        .all(|old| old.zone_gain_through_2018 == Some(QsbsZoneValuation::NoRecords))
                {
                    lot.zone_gain_through_2018 = Some(QsbsZoneValuation::NoRecords);
                }
            }
        }
    }
}
