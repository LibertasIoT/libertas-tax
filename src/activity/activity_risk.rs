// Existing Unit19 Form6198 source projection. Source owners supply native cents
// in original form order; shared loss_limits owners settle and close them.
use crate::kernel::{carry, loss_limits, source_fact::SourceFact};
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Part1Line {
    Operating,
    Property,
    OtherDisposition,
}
impl Part1Line {
    pub(crate) fn printed(self) -> &'static str {
        match self {
            Self::Operating => "1",
            Self::Property => "2b",
            Self::OtherDisposition => "2c",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProjectionIssue {
    InvalidMethod,
    Source(&'static str),
    Arithmetic(loss_limits::LossLimitError),
    DetailedRequired,
    HistoryRequired,
}
impl ProjectionIssue {
    pub(crate) fn key(&self) -> &'static str {
        match self {
            Self::InvalidMethod => "RISK_METHOD_INVALID",
            Self::Source(key) => key,
            Self::Arithmetic(loss_limits::LossLimitError::Overflow) => {
                "ACTIVITY_LOSS_AMOUNT_UNREPRESENTABLE"
            }
            Self::Arithmetic(_) => "RISK_METHOD_INVALID",
            Self::DetailedRequired => "RISK_RECAPTURE_DETAILED_REQUIRED",
            Self::HistoryRequired => "RISK_RECAPTURE_HISTORY_REQUIRED",
        }
    }
    pub(crate) fn report(&self, id: i64, r: &mut crate::FederalResult) {
        let source = crate::IssueContext::income_id(id);
        let context = source.clone().manual();
        // The source report may already own this issue and its correction link;
        // regular, AMT and property projections must not emit it again.
        if !r.issues.iter().any(|issue| {
            issue.context.identity() == source.identity() && issue.display.key == self.key()
        }) {
            crate::issue_localized!(
                r,
                &context,
                "Activity loss source",
                self.key(),
                "Resolve the actual source facts or representable loss amounts before completing this activity calculation.",
            );
        }
        crate::federal_rules::withhold_required_source_result(r);
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceIssue {
    pub source: i64,
    pub issue: ProjectionIssue,
}
impl SourceIssue {
    pub(crate) fn new(source: i64, issue: ProjectionIssue) -> Self {
        Self { source, issue }
    }
    pub(crate) fn report(&self, r: &mut crate::FederalResult) {
        self.issue.report(self.source, r);
    }
}

impl From<loss_limits::LossLimitError> for ProjectionIssue {
    fn from(value: loss_limits::LossLimitError) -> Self {
        Self::Arithmetic(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RiskProjection {
    pub allowed: Vec<i64>,
    pub suspended: Vec<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityProjection {
    pub components: RiskProjection,
    pub lines: Vec<(&'static str, i64)>,
    pub recapture: Result<i64, ProjectionIssue>,
}

pub(crate) fn project_record(
    components: &[(Part1Line, i64)],
    record: &crate::ActivityRiskRecord,
    category: &crate::RiskEffectiveCategory,
    amt: bool,
) -> Result<ActivityProjection, ProjectionIssue> {
    if !record.valid(category) {
        return Err(ProjectionIssue::InvalidMethod);
    }
    let balance = record
        .compute(category, amt)
        .ok_or(ProjectionIssue::Arithmetic(loss_limits::LossLimitError::Overflow))?;
    let settled = loss_limits::risk::settle_signed_components::<_, carry::Cents>(
        SourceFact::Reported(components),
        balance.capacity,
    )?;
    let SourceFact::Reported(settled) = settled else {
        return Err(ProjectionIssue::InvalidMethod);
    };
    let mut lines = Vec::new();
    for printed in ["1", "2a", "2b", "2c", "3", "4"] {
        lines.push((
            printed,
            crate::kernel::ratio::checked_sum(
                components
                    .iter()
                    .filter(|(part, _)| part.printed() == printed)
                    .map(|(_, amount)| *amount),
            )
            .ok_or(ProjectionIssue::Arithmetic(loss_limits::LossLimitError::Overflow))?,
        ));
    }
    lines.push(("5", settled.net));
    lines.extend(balance.lines);
    lines.push(("20", balance.capacity));
    lines.push((
        "21",
        crate::kernel::phase::checked_loss(settled.allowed_net)
            .ok_or(ProjectionIssue::Arithmetic(loss_limits::LossLimitError::Overflow))?,
    ));
    let recapture = if let Some(investment) = balance.detailed {
        let history = match &record.recapture {
            Some(history) => {
                let amounts = if amt { &history.amt } else { &history.regular };
                SourceFact::Reported(loss_limits::risk::RecaptureHistory {
                    prior_deducted: carry::Amount::new(amounts.deducted_post1978)
                        .map_err(|_| ProjectionIssue::InvalidMethod)?,
                    prior_recaptured: carry::Amount::new(amounts.recaptured_before)
                        .map_err(|_| ProjectionIssue::InvalidMethod)?,
                    threshold_1978: history.floor_1978,
                })
            }
            None => SourceFact::NotReported,
        };
        let account = loss_limits::risk::close_economic_account::<carry::Cents>(
            investment,
            settled.allowed_net,
            history,
        )?;
        match account.recapture {
            SourceFact::Reported(recapture) => Ok(recapture.income.get()),
            // A nonnegative ending cannot require recapture even without prior
            // deducted-loss history. A genuine deficit still needs that source.
            SourceFact::NotReported if account.ending >= 0 => Ok(0),
            SourceFact::NotReported | SourceFact::Unavailable => {
                Err(ProjectionIssue::HistoryRequired)
            }
        }
    } else if balance.simplified.is_some_and(|amount| amount < 0) {
        Err(ProjectionIssue::DetailedRequired)
    } else {
        Ok(0)
    };
    Ok(ActivityProjection {
        components: RiskProjection {
            allowed: settled.components.iter().map(|part| part.allowed_net()).collect(),
            suspended: settled.components.iter().map(|part| part.held()).collect(),
        },
        lines,
        recapture,
    })
}

// The native risk owner supplies current operating cents before prior loss
// and the actual held operating component before recapture. Only a uniquely
// determined prior release establishes the history operand for QBI.
pub(crate) fn prior_operating_allowed(
    current: Option<i64>,
    prior: i64,
    held: i64,
) -> Result<Option<i64>, ProjectionIssue> {
    if prior == 0 {
        return Ok(Some(0));
    }
    let Some(current) = current else {
        return Ok(None);
    };
    let overflow = || ProjectionIssue::Arithmetic(loss_limits::LossLimitError::Overflow);
    let current_loss = crate::kernel::phase::checked_loss(current).ok_or_else(overflow)?;
    let losses = current_loss.checked_add(prior).ok_or_else(overflow)?;
    let allowed = losses.checked_sub(held).ok_or_else(overflow)?;
    if !(0..=losses).contains(&allowed) {
        return Err(ProjectionIssue::Arithmetic(loss_limits::LossLimitError::Footing));
    }
    Ok(crate::section1231::determined_share(allowed, losses, prior))
}

// Retain the original rental-specific Schedule E/Form8582 source meaning.
// Generic Ordinary history is not an operating-form source and is parked.
pub(crate) fn source_issue(
    risk: &Option<crate::RentalAtRisk>,
    history: &crate::RentalHistory,
) -> Option<crate::validation::ValidationIssue> {
    source_issue_with_method(risk, history, false)
}

pub(crate) fn farm_rental_source_issue(
    risk: &Option<crate::RentalAtRisk>,
    history: &crate::RentalHistory,
) -> Option<crate::validation::ValidationIssue> {
    source_issue_with_method(risk, history, true)
}

fn source_issue_with_method(
    risk: &Option<crate::RentalAtRisk>,
    history: &crate::RentalHistory,
    optional_method: bool,
) -> Option<crate::validation::ValidationIssue> {
    let Some(risk) = risk else {
        return Some(
            crate::validation::ValidationIssue::new("ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED")
                .at_field("at_risk"),
        );
    };
    if let Some(issue) =
        risk.issue_for_method(&crate::RiskEffectiveCategory::HoldingRealProperty, optional_method)
    {
        return Some(issue.in_field("at_risk"));
    }
    // Existing typed source envelopes; amounts remain distinct by tax system.
    for (field, value) in [("regular", history.regular), ("amt", history.amt)] {
        if !(0..=100_000_000_000).contains(&value) {
            return Some(
                crate::validation::ValidationIssue::new("RISK_METHOD_INVALID")
                    .at_field(field)
                    .in_field("history"),
            );
        }
    }
    None
}
