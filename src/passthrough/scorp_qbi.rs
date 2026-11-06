use crate::kernel::carry::{Amount, Cents};
// Coverage: S-corporation section 1231 sale QBI — runtime owner integration.
//   Gross sale gains/losses join the business-property loss character while
//   entered box qualification keeps its original bounds. Current SSTB loss
//   qualification precedes basis/risk/passive limits; retained qualification
//   is never rephased. Capital character excludes every property component;
//   undetermined partial lookback allocation keeps its named refusal.
//   Evidence: tests/passthrough/k1_sale_qbi.rs k1_sale_qbi_property_*; i8995 (2025 final),
//   26 CFR 1.199A-3. Core and compound final returns pass k1_sale_final.rs
//   and k1_compound.rs. Opposite-character EBL uses one shared gross-source
//   qualified deferral for either entity; tests/passthrough/k1_property_offsets.rs.
//   Broader mixed-source and retained R9 acceptance remains open.
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use crate::scorp_basis::{BasisResult, LossRelease, LossVintage, release_losses};
use crate::validation::ValidationIssue;
use crate::{FederalResult, ScorpK1};
use alloc::vec::Vec;
use libertas_macros::*;

pub(crate) type ActivitiesAndCurrentLoss = (Vec<crate::business_rules::QbiActivity>, Amount<Cents>);

fn note_usd(minor_units: i64) -> crate::MessageArg {
    crate::MessageArg::usd_cents(minor_units)
}

fn note(
    #[cfg(test)] _english: alloc::string::String,
    key: &str,
    args: Vec<crate::MessageArg>,
) -> crate::federal_model::ReviewNote {
    let message = crate::Message::new(key, args);
    crate::federal_model::ReviewNote::localized(
        #[cfg(test)]
        message.english(),
        message,
    )
}

/// Prior business loss suspended by a loss limit
/// Copy one year's retained QBI loss-tracking worksheet. Use positive loss magnitudes. Keep at-risk and passive carryforwards separate. Historical QBI amounts must already reflect the loss year's eligibility, including any specified-service-business restriction.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpLossYear {
    /// Year of disallowance under this limit
    /// Combine pre-2018 balances in 2017; their QBI portion must be zero. Use one row per later year.
    // Tax structure: this retained history precedes the 2026 tax year.
    #[libertas_number(min = 2017, max = 2025)]
    pub year: u32,
    /// Original suspended loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub original: i64,
    /// Original suspended QBI portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub qualified: i64,
    /// Total loss remaining at the end of 2025
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining: i64,
    /// QBI portion remaining at the end of 2025
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining_qualified: i64,
}

/// QBI character and histories for one business-loss character
/// This worksheet tracks loss character only, not the final QBI deduction. Further at-risk, passive and excess-business-loss limits still apply. Zero and empty lists when this character has no current or carried loss.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ScorpLossQbi {
    /// QBI portion of this year's loss in this tax character
    /// From the corporation's section 199A statement, as a positive amount, before this year's shareholder SSTB income limit. The App applies that limit. Zero when this character has no current loss. Do not include prior suspended losses or employee wages.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub current: i64,
    /// Retained basis-loss history
    /// From the QBI loss-tracking records kept with your 2025 return. Remaining amounts must reconcile to the separate prior loss of this character suspended by basis. An empty list confirms no prior basis loss.
    /// ----
    /// Loss year
    pub history: Vec<ScorpLossYear>,
    /// Retained at-risk loss history
    /// From the QBI loss-tracking records kept with your 2025 return for losses of this character suspended by at-risk limits (2025 Form 6198). Remaining amounts must reconcile to those prior at-risk losses; leave the list empty when there were none. Do not repeat basis-loss balances here.
    /// ----
    /// Loss year
    pub at_risk: Vec<ScorpLossYear>,
    /// Retained passive-loss history
    /// From the QBI loss-tracking records kept with your 2025 return for losses of this character suspended by passive limits (2025 Form 8582 worksheets). Reconcile these balances to those prior passive losses, which already passed basis and at-risk limits; leave the list empty when there were none.
    /// ----
    /// Loss year
    pub passive: Vec<ScorpLossYear>,
}

// One source character; amounts remain cents until the shared stage converts
// to return dollars. Basis bucket 0 is ordinary; bucket 4 is section 1231.
#[derive(Clone, Copy)]
struct LossSource {
    current: i64,
    basis: i64,
    risk: i64,
    passive: i64,
    class: crate::k1_loss::K1LossClass,
}
impl LossSource {
    fn ordinary(owner: &ScorpK1) -> Self {
        Self {
            current: phase::floor(owner.ordinary.saturating_neg(), 0),
            basis: owner.prior_loss,
            risk: owner.risk.prior_loss,
            passive: owner.passive.loss,
            class: crate::k1_loss::K1LossClass::Ordinary,
        }
    }
    fn property(source: &crate::K1Section1231) -> Self {
        Self {
            current: phase::floor(source.net.saturating_neg(), 0),
            basis: source.basis_loss,
            risk: source.risk_loss,
            passive: source.passive_loss,
            class: crate::k1_loss::K1LossClass::Section1231,
        }
    }
}

// Ordinary QBI uses the ordinary share of a resolved combined activity. Keep
// property values in the owner so shared basis/at-risk limits remain correct.
fn ordinary_passive_net_with_events(
    owner: &ScorpK1,
    events: Option<&crate::scorp::ScorpEvents>,
) -> Result<i64, (&'static str, &'static str)> {
    let activity = owner.pool_activity_with_events(events)?;
    Ok(activity.characters.map_or(activity.passive, |v| v.passive[0]))
}

impl ScorpLossQbi {
    pub(crate) fn missing_history(&self, balances: [i64; 3]) -> bool {
        [&self.history, &self.at_risk, &self.passive]
            .into_iter()
            .zip(balances)
            .any(|(history, balance)| balance > 0 && history.is_empty())
    }

    // Missing worksheets are an unknown. Supplied worksheets must reconcile
    // even when another stage's history is unavailable.
    pub(crate) fn supplied_histories_valid(&self, balances: [i64; 3]) -> bool {
        self.supplied_histories_issue(balances, "").is_none()
    }

    pub(crate) fn supplied_histories_issue(
        &self,
        balances: [i64; 3],
        key: &'static str,
    ) -> Option<ValidationIssue> {
        [("history", &self.history), ("at_risk", &self.at_risk), ("passive", &self.passive)]
            .into_iter()
            .zip(balances)
            .find_map(|((field, history), balance)| {
                if balance == 0 || history.is_empty() {
                    None
                } else {
                    history_issue(history, Some(balance), field, key)
                }
            })
    }

    pub(crate) fn valid_retained(&self) -> bool {
        self.retained_issue("").is_none()
    }

    pub(crate) fn retained_issue(&self, key: &'static str) -> Option<ValidationIssue> {
        // Arithmetic safety: current_allowed and property calculation retain native
        // signed QBI differences and dollar-to-cent transfers.
        if !(0..=100_000_000_000).contains(&self.current) {
            return Some(ValidationIssue::new(key).at_field("current"));
        }
        [("history", &self.history), ("at_risk", &self.at_risk), ("passive", &self.passive)]
            .into_iter()
            .find_map(|(field, history)| history_issue(history, None, field, key))
    }

    pub(crate) fn valid(&self, owner: &ScorpK1) -> bool {
        self.validation_issue(owner, "").is_none()
    }

    pub(crate) fn validation_issue(
        &self,
        owner: &ScorpK1,
        key: &'static str,
    ) -> Option<ValidationIssue> {
        self.source_issue(LossSource::ordinary(owner), key)
    }

    // No current QBI loss portion and no loss-year history: the section's
    // value when this character has no current or carried loss.
    pub(crate) fn is_empty(&self) -> bool {
        self.current == 0
            && self.history.is_empty()
            && self.at_risk.is_empty()
            && self.passive.is_empty()
    }

    // The page rule SCORP_QBI_LOSS_RECORDS_NEEDED: the current QBI portion
    // fits the current loss, and each carried loss of this character has
    // its loss-year history (validity reconciles the amounts).
    pub(crate) fn records_issue(
        &self,
        current_loss: i64,
        carried: [i64; 3],
    ) -> Option<ValidationIssue> {
        if self.current != 0 && self.current > current_loss {
            return Some(ValidationIssue::new("SCORP_QBI_LOSS_RECORDS_NEEDED").at_field("current"));
        }
        carried
            .into_iter()
            .zip([
                ("history", &self.history),
                ("at_risk", &self.at_risk),
                ("passive", &self.passive),
            ])
            .find_map(|(amount, (field, history))| {
                (amount != 0 && history.is_empty())
                    .then(|| ValidationIssue::new("SCORP_QBI_LOSS_RECORDS_NEEDED").at_field(field))
            })
    }

    fn valid_source(&self, source: LossSource) -> bool {
        self.source_issue(source, "").is_none()
    }

    fn source_issue(&self, source: LossSource, key: &'static str) -> Option<ValidationIssue> {
        // Arithmetic safety: current_allowed uses native signed QBI differences
        // and subsequent source-cent transfers, as in retained_issue.
        if !(0..=100_000_000_000).contains(&self.current) || self.current > source.current {
            return Some(ValidationIssue::new(key).at_field("current"));
        }
        [
            ("history", &self.history, source.basis),
            ("at_risk", &self.at_risk, source.risk),
            ("passive", &self.passive, source.passive),
        ]
        .into_iter()
        .find_map(|(field, history, balance)| history_issue(history, Some(balance), field, key))
    }

    pub(crate) fn calculate(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        self.basis_for(LossSource::ordinary(owner), basis, Amount::default())
    }

    fn basis_for(
        &self,
        source: LossSource,
        basis: &BasisResult,
        other_allowed: Amount<Cents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        if !self.valid_source(source) {
            return Err((
                "ISSUE_SCORP_QBI_RECONCILE_THE_QBI_LOSS_YEAR_RECORDS_WITH_14CFA998",
                crate::fixture_english!(
                    "Reconcile the QBI loss-year records with this character's current and suspended losses."
                ),
            ));
        }
        let mut years = loss_years(&self.history);
        let current = source.current;
        if current > 0 {
            years.push(LossVintage {
                year: 2026,
                original: Amount::<Cents>::new(current).expect("bounded basis result"),
                original_qbi: Amount::<Cents>::new(self.current).expect("bounded basis result"),
                remaining: Amount::<Cents>::new(current).expect("bounded basis result"),
                remaining_qbi: Amount::<Cents>::new(self.current).expect("bounded basis result"),
            });
        }
        basis.release_history_item_part(source.class, &years, other_allowed)
            .map(finalize_current_vintage)
            .map_err(|_| ("ISSUE_SCORP_QBI_THE_ROUNDED_QBI_LOSS_YEAR_AMOUNTS_MUST_3737C544", crate::fixture_english!("The rounded QBI loss-year amounts must reconcile with this character's basis worksheet; review source amounts.")))
    }

    pub(crate) fn after_at_risk(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        self.after_at_risk_with_events(owner, basis, None)
    }

    pub(crate) fn after_at_risk_with_events(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        let risk = owner
            .risk
            .calculate_with_events(owner, basis, events)
            .map_err(|error| error.message())?;
        self.risk_for(
            LossSource::ordinary(owner),
            basis,
            risk.ledger.risk_allowed[crate::k1_loss::K1LossClass::Ordinary],
            risk.ledger.risk_suspended[crate::k1_loss::K1LossClass::Ordinary],
            Amount::default(),
        )
    }

    fn risk_for(
        &self,
        source: LossSource,
        basis: &BasisResult,
        allowed: Amount<Cents>,
        suspended: Amount<Cents>,
        other_allowed: Amount<Cents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        let released = self.basis_for(source, basis, other_allowed)?;
        let mut years = loss_years(&self.at_risk);
        if released.amount.get() > 0 {
            // Losses newly limited at this stage use a 2026 suspension vintage
            // while retaining the qualified fraction passed by the earlier limit.
            years.push(LossVintage {
                year: 2026,
                original: released.amount,
                original_qbi: released.qbi,
                remaining: released.amount,
                remaining_qbi: released.qbi,
            });
        }
        if years.iter().map(|v| v.remaining.get()).sum::<i64>() != allowed.get() + suspended.get() {
            return Err((
                "ISSUE_SCORP_QBI_THE_ROUNDED_AT_RISK_QBI_LOSS_HISTORY_98A3E4A9",
                crate::fixture_english!(
                    "The rounded at-risk QBI loss history must reconcile to this character's at-risk worksheet."
                ),
            ));
        }
        release_losses(&years, allowed).map(finalize_current_vintage).map_err(|_| {
            (
                "ISSUE_SCORP_QBI_THE_AT_RISK_QBI_LOSS_YEAR_RECORDS_ARE_5A80FD65",
                crate::fixture_english!("The at-risk QBI loss-year records are inconsistent."),
            )
        })
    }

    pub(crate) fn after_passive(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        self.after_passive_with_events(owner, suspended, None)
    }

    pub(crate) fn after_passive_with_events(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        let passive = ordinary_passive_net_with_events(owner, events)?;
        if suspended.get() > phase::floor(-passive, 0) {
            return Err((
                "ISSUE_SCORP_QBI_THE_PASSIVE_SUSPENSION_MUST_FIT_THIS_B3AA5C14",
                crate::fixture_english!(
                    "The passive suspension must fit this activity's computed passive loss."
                ),
            ));
        }
        let basis = owner.basis_result()?;
        let current = self.after_at_risk_with_events(owner, &basis, events)?;
        self.passive_for(
            current,
            owner.passive.loss,
            owner.nonpassive() == crate::participation::Participation::Yes,
            suspended,
        )
    }

    fn passive_for(
        &self,
        current: LossRelease,
        prior: i64,
        nonpassive: bool,
        suspended: Amount<Cents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        let mut years = loss_years(&self.passive);
        if years.iter().map(|v| v.remaining.get()).sum::<i64>() != prior {
            return Err((
                "ISSUE_SCORP_QBI_THE_ROUNDED_PASSIVE_QBI_HISTORY_MUST_AE740F7A",
                crate::fixture_english!(
                    "The rounded passive QBI history must reconcile to prior passive losses."
                ),
            ));
        }
        if nonpassive {
            let mut release = release_losses(
                &years,
                Amount::<Cents>::new(prior - suspended.get())
                    .expect("nonnegative allowed source loss"),
            )
            .map_err(|_| {
                (
                    "ISSUE_SCORP_QBI_THE_FORMER_PASSIVE_QBI_LOSS_HISTORY_IS_5E7A8F29",
                    crate::fixture_english!("The former-passive QBI loss history is inconsistent."),
                )
            })?;
            release.amount = Amount::new(release.amount.get() + current.amount.get())
                .expect("bounded released source cents");
            release.qbi = Amount::new(release.qbi.get() + current.qbi.get())
                .expect("bounded released source cents");
            return Ok(release);
        }
        if current.amount.get() > 0 {
            years.push(LossVintage {
                year: 2026,
                original: current.amount,
                original_qbi: current.qbi,
                remaining: current.amount,
                remaining_qbi: current.qbi,
            });
        }
        release_losses(
            &years,
            Amount::<Cents>::new(prior + current.amount.get() - suspended.get())
                .expect("nonnegative allowed source loss"),
        )
        .map(finalize_current_vintage)
        .map_err(|_| {
            (
                "ISSUE_SCORP_QBI_THE_PASSIVE_QBI_LOSS_HISTORY_IS_73C1614D",
                crate::fixture_english!("The passive QBI loss history is inconsistent."),
            )
        })
    }

    fn current_allowed(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
    ) -> Result<i64, (&'static str, &'static str)> {
        let basis = self.calculate(owner, &owner.basis_result()?)?;
        let risk = self.after_at_risk(owner, &owner.basis_result()?)?;
        let passive = if owner.nonpassive() == crate::participation::Participation::Yes {
            None
        } else {
            Some(self.after_passive(owner, suspended)?)
        };
        Ok(current_qualified_release(self.current, &basis, &risk, passive.as_ref()))
    }

    pub(crate) fn report_passive(
        &self,
        owner: &ScorpK1,
        id: i64,
        suspended: Amount<Cents>,
        result: &mut FederalResult,
    ) {
        match self.after_passive(owner, suspended) {
            Ok(release) => {
                result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, QBI loss passing passive limits: ${}. Excess-business-loss limits, current QBI income and final deduction integration remain required.", release.qbi.get()), "NOTE_SCORP_QBI_PASSIVE_RELEASE", alloc::vec![crate::MessageArg::Number(id), note_usd(release.qbi.get())]));
                for year in release.remaining {
                    result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, {} passive-loss carryforward: ${}; QBI portion: ${}. Original suspended loss: ${}; original QBI portion: ${}. This balance already passed basis and at-risk limits.", year.year, year.remaining.get(), year.remaining_qbi.get(), year.original.get(), year.original_qbi.get()), "NOTE_SCORP_QBI_PASSIVE_CARRY", alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::Number(i64::from(year.year)), note_usd(year.remaining.get()), note_usd(year.remaining_qbi.get()), note_usd(year.original.get()), note_usd(year.original_qbi.get())]));
                }
            }
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;
                crate::issue_localized!(
                    result,
                    &crate::IssueContext::income_id(id),
                    "S-corporation QBI passive history",
                    key,
                    message,
                )
            }
        }
    }

    pub(crate) fn report(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
        id: i64,
        result: &mut FederalResult,
    ) {
        match self.calculate(owner, basis) {
            Ok(release) => {
                result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, QBI loss passing basis limits: ${}. This is loss character, not a final QBI deduction; subsequent loss limits and return integration remain required.", release.qbi.get()), "NOTE_SCORP_QBI_BASIS_RELEASE", alloc::vec![crate::MessageArg::Number(id), note_usd(release.qbi.get())]));
                for year in &release.remaining {
                    result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, {} basis-loss carryforward: ${}; QBI portion: ${}. Original suspended loss: ${}; original QBI portion: ${}. Keep this separate from at-risk and passive carryforwards.", year.year, year.remaining.get(), year.remaining_qbi.get(), year.original.get(), year.original_qbi.get()), "NOTE_SCORP_QBI_BASIS_CARRY", alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::Number(i64::from(year.year)), note_usd(year.remaining.get()), note_usd(year.remaining_qbi.get()), note_usd(year.original.get()), note_usd(year.original_qbi.get())]));
                }
                {
                    match self.after_at_risk(owner, basis) {
                        Ok(risk) => {
                            result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, QBI loss passing at-risk limits: ${}. Passive and excess-business-loss limits still apply; this is not the final QBI deduction.", risk.qbi.get()), "NOTE_SCORP_QBI_RISK_RELEASE", alloc::vec![crate::MessageArg::Number(id), note_usd(risk.qbi.get())]));
                            for year in risk.remaining {
                                result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, {} at-risk-loss carryforward: ${}; QBI portion: ${}. Original suspended loss: ${}; original QBI portion: ${}. This balance already passed basis limits.", year.year, year.remaining.get(), year.remaining_qbi.get(), year.original.get(), year.original_qbi.get()), "NOTE_SCORP_QBI_RISK_CARRY", alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::Number(i64::from(year.year)), note_usd(year.remaining.get()), note_usd(year.remaining_qbi.get()), note_usd(year.original.get()), note_usd(year.original_qbi.get())]));
                            }
                        }
                        Err((key, _message)) => {
                            #[cfg(test)]
                            let message = _message;
                            crate::issue_localized!(
                                result,
                                &crate::IssueContext::income_id(id),
                                "S-corporation QBI at-risk history",
                                key,
                                message,
                            )
                        }
                    }
                }
            }
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;
                crate::issue_localized!(
                    result,
                    &crate::IssueContext::income_id(id),
                    "S-corporation QBI basis history",
                    key,
                    message,
                )
            }
        }
    }
}

pub(crate) fn valid_history(history: &[ScorpLossYear], expected: Option<i64>) -> bool {
    history_issue(history, expected, "history", "").is_none()
}

// The same ordered predicates serve retained shape and source reconciliation.
// A total mismatch identifies the collection, never an inferred offending row.
// Boolean facades discard empty-key issues immediately; typed admission
// callers must supply their existing native message key.
pub(crate) fn history_issue(
    history: &[ScorpLossYear],
    expected: Option<i64>,
    field: &'static str,
    key: &'static str,
) -> Option<ValidationIssue> {
    // Arithmetic safety: historical_qbi_activities and current_allowed retain
    // native history totals and differences before their cents transfers.
    let money = |value: i64| (0..=100_000_000_000).contains(&value);
    for (index, value) in history.iter().enumerate() {
        let issue = if !(2017..=2025).contains(&value.year)
            || history[..index].iter().any(|previous| previous.year == value.year)
        {
            Some(ValidationIssue::new(key).at_field("year"))
        } else if let Some((field, _)) = [
            ("original", value.original),
            ("qualified", value.qualified),
            ("remaining", value.remaining),
            ("remaining_qualified", value.remaining_qualified),
        ]
        .into_iter()
        .find(|(_, amount)| !money(*amount))
        {
            Some(ValidationIssue::new(key).at_field(field))
        } else if value.original <= 0 {
            Some(ValidationIssue::new(key).at_field("original"))
        } else if value.qualified > value.original {
            Some(ValidationIssue::new(key).at_field("qualified"))
        } else if value.remaining > value.original {
            Some(ValidationIssue::new(key).at_field("remaining"))
        } else if value.remaining_qualified > value.remaining
            || value.remaining_qualified > value.qualified
        {
            Some(ValidationIssue::new(key).at_field("remaining_qualified"))
        } else if value.remaining - value.remaining_qualified > value.original - value.qualified {
            Some(ValidationIssue::new(key).at_record())
        } else if value.year == 2017 && value.qualified != 0 {
            Some(ValidationIssue::new(key).at_field("qualified"))
        } else {
            None
        };
        if let Some(issue) = issue {
            return Some(issue.in_list(field, index));
        }
    }
    if expected
        .is_some_and(|total| history.iter().map(|value| value.remaining).sum::<i64>() != total)
    {
        return Some(ValidationIssue::new(key).at_field(field));
    }
    None
}

pub(crate) fn loss_years(history: &[ScorpLossYear]) -> Vec<LossVintage> {
    history
        .iter()
        .filter(|v| v.original != 0)
        .map(|v| LossVintage {
            year: v.year,
            original: Amount::<Cents>::new(v.original).expect("bounded basis result"),
            original_qbi: Amount::<Cents>::new(v.qualified).expect("bounded basis result"),
            remaining: Amount::<Cents>::new(v.remaining).expect("bounded basis result"),
            remaining_qbi: Amount::<Cents>::new(v.remaining_qualified)
                .expect("bounded basis result"),
        })
        .collect()
}

// A new carryforward's original amount is what this provision actually
// disallowed, not the larger amount that entered its current-year worksheet.
pub(crate) fn finalize_current_vintage<Unit>(mut release: LossRelease<Unit>) -> LossRelease<Unit> {
    for year in &mut release.remaining {
        if year.year == 2026 && year.remaining.get() > 0 {
            year.original = year.remaining;
            year.original_qbi = year.remaining_qbi;
        }
    }
    release
}

/// Current ordinary-business section 199A statement
/// Copy the shareholder's allocated amounts from the K-1 code V statement, not the corporation's total amounts. Employee W-2 compensation is not shareholder QBI.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpQbiStatement {
    /// Qualified ordinary business income before shareholder loss carryforwards
    /// Positive ordinary income from the section 199A statement. Current ordinary losses use the separate loss-character records.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income: i64,
    /// Allocated W-2 wages for the QBI wage limit
    /// Use the entity's section 199A allocation, not wages paid to you on your employee W-2.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
    /// Allocated unadjusted basis of qualified property
    /// UBIA from the section 199A statement, not stock basis or depreciated asset value.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub property: i64,
    /// Specified service trade or business
    /// Check the Section 199A statement and the actual trade or business activities; ask the issuer if its SSTB classification is missing.
    pub service: bool,
    /// Section 199A dividends
    /// From the statement; already part of the ordinary dividends box.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub reit_dividends: i64,
    /// Qualified PTP items
    /// The statement's qualified publicly traded partnership income or loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub ptp_income: i64,
    /// One ordinary business with no additional QBI adjustments
    /// Confirm no entity-reported aggregation, cooperative or PTP items, section 199A dividends, other businesses, shareholder retirement deductions, or other QBI adjustments outside these source records. Shareholder health-insurance deductions entered under Self-employed benefits are calculated and attributed separately. Additional items need their own calculations. Compare the K-1's section 199A statement (box 17 code V) and its attachments with what is entered. No leaves this corporation's qualified business income out of the QBI deduction, with a notice saying which way your tax may move: leaving out income may overstate tax, leaving out a loss may understate it.
    pub complete: bool,
}

impl ScorpQbiStatement {
    pub(crate) fn valid(&self, owner: &ScorpK1) -> bool {
        // Arithmetic safety: combined_with_sales retains native QBI component
        // additions and loss allocations before its cents transfer.
        [self.income, self.wages, self.property, self.reit_dividends]
            .into_iter()
            .all(|v| (0..=100_000_000_000).contains(&v))
            && (-100_000_000_000..=100_000_000_000).contains(&self.ptp_income)
            && self.income <= owner.ordinary.max(0)
    }

    pub(crate) fn prepare(
        &self,
        owner: &ScorpK1,
        taxable: i64,
        status: crate::FederalStatus,
    ) -> Result<ScorpK1, (&'static str, &'static str)> {
        let mut prepared = owner.clone();
        let property_loss = owner.section1231.as_ref().is_some_and(|v| v.net < 0);
        if owner.ordinary < 0 || property_loss {
            let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
                -owner.loss_qbi.current,
                Some(self.service),
                false,
                taxable,
                status,
            )?;
            // Apply the recorded source classification before loss limitations.
            let phase = |history: &mut ScorpLossQbi| {
                if self.service {
                    history.current =
                        crate::kernel::ratio::fraction(history.current, applicable, range);
                }
            };
            if owner.ordinary < 0 {
                phase(&mut prepared.loss_qbi);
            }
            if property_loss {
                phase(&mut prepared.property_qbi.losses);
            }
        }
        Ok(prepared)
    }

    // Complete current-business eligibility once, with one wage/UBIA amount.
    // Historical releases remain separate activities and are never rephased.
    // None is permissible only when known non-SSTB status makes the current
    // eligibility fraction independent of the unresolved final taxable base.
    // The same current-business calculation, with the individual's actual
    // attributed section162(l) deduction before current SSTB eligibility.
    #[allow(clippy::too_many_arguments)]
    #[cfg(test)]
    pub(crate) fn combined_activities(
        &self,
        owner: &ScorpK1,
        suspended: [Amount<Cents>; 2],
        property_character: crate::section1231::PropertyCharacter,
        taxable: Option<i64>,
        status: crate::FederalStatus,
        section179: i64,
        health: i64,
    ) -> Result<Vec<crate::business_rules::QbiActivity>, (&'static str, &'static str)> {
        self.combined_with_sales(
            owner,
            suspended,
            property_character,
            (taxable, status),
            section179,
            &Default::default(),
            health,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn combined_with_sales(
        &self,
        owner: &ScorpK1,
        suspended: [Amount<Cents>; 2],
        property_character: crate::section1231::PropertyCharacter,
        eligibility: (Option<i64>, crate::FederalStatus),
        section179: i64,
        sales: &crate::k1_sale_qbi::CurrentSales,
        health: i64,
    ) -> Result<Vec<crate::business_rules::QbiActivity>, (&'static str, &'static str)> {
        self.combined_with_sales_with_events(
            owner,
            suspended,
            property_character,
            eligibility,
            section179,
            sales,
            health,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn combined_with_sales_with_events(
        &self,
        owner: &ScorpK1,
        suspended: [Amount<Cents>; 2],
        property_character: crate::section1231::PropertyCharacter,
        eligibility: (Option<i64>, crate::FederalStatus),
        section179: i64,
        sales: &crate::k1_sale_qbi::CurrentSales,
        health: i64,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<Vec<crate::business_rules::QbiActivity>, (&'static str, &'static str)> {
        use crate::business_rules::QbiActivity;
        let (taxable, status) = eligibility;
        let taxable = match taxable {
            Some(value) if value >= 0 => value,
            None if !self.service => 0,
            _ => {
                return Err((
                    "ISSUE_SCORP_QBI_COMBINED_BUSINESS_QBI_NEEDS_THE_COMPLETE_CF7C9CBD",
                    crate::fixture_english!(
                        "Combined business QBI needs the complete taxable-income base before resolving current service-business eligibility."
                    ),
                ));
            }
        };
        let prepared = self.prepare(owner, taxable, status)?;
        let (mut ordinary, ordinary_current_loss) =
            self.activities_with_events(&prepared, suspended[0], section179, events)?;
        let mut gain = self.income + sales.gain_cents - section179 * 100;
        let mut current_loss = ordinary_current_loss.get() + sales.loss_cents.get();
        let mut historical_property = 0;
        if property_character != crate::section1231::PropertyCharacter::Capital {
            let mut facts = owner.property_qbi.calculation(owner, Some((taxable, status)))?;
            facts.events = events.copied();
            let release = facts.after_passive(owner, suspended[1])?;
            let fully_ordinary = property_character
                == crate::section1231::PropertyCharacter::Ordinary
                || matches!(property_character,
                    crate::section1231::PropertyCharacter::Recaptured(amount)
                        if amount == dollars(facts.property.gain - release.amount.get()));
            if fully_ordinary {
                gain += facts.gain_cents;
                let current = facts.current_allowed(owner, suspended[1])?;
                current_loss += current;
                historical_property = release.qbi.get() - current;
            } else if let crate::section1231::PropertyCharacter::Recaptured(amount) =
                property_character
            {
                gain += facts.qualified_recapture(owner, amount)? * 100;
            }
        }
        let material = owner
            .participation
            .as_ref()
            .map(|v| v.material())
            .unwrap_or(crate::participation::Participation::Unavailable);
        if gain > 0 && material == crate::participation::Participation::Unavailable {
            return Err((
                "ISSUE_SCORP_QBI_RESOLVE_PARTICIPATION_FOR_THE_MINIMUM_QBI_D07583A4",
                crate::fixture_english!(
                    "Resolve participation for the minimum QBI deduction on combined business income."
                ),
            ));
        }
        let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
            gain - health * 100,
            Some(self.service),
            false,
            taxable,
            status,
        )?;
        let current_cents =
            crate::kernel::ratio::fraction(gain - health * 100, applicable, range) - current_loss;
        let current = QbiActivity {
            source_wages_cents: Some(phase::remaining_fraction(self.wages, applicable, range)),
            source_property_cents: Some(phase::remaining_fraction(
                self.property,
                applicable,
                range,
            )),
            source_income_cents: Some(current_cents),
            owner: None,
            income: dollars(current_cents),
            wages: dollars(phase::remaining_fraction(self.wages, applicable, range)),
            property: dollars(phase::remaining_fraction(self.property, applicable, range)),
            active: material == crate::participation::Participation::Yes,
            service: Some(false),
            group: None,
        };
        ordinary[0] = current;
        if historical_property > 0 {
            ordinary.push(QbiActivity {
                source_wages_cents: Some(0),
                source_property_cents: Some(0),
                source_income_cents: Some(-historical_property),
                owner: None,
                income: -dollars(historical_property),
                wages: 0,
                property: 0,
                active: false,
                service: Some(false),
                group: None,
            });
        }
        Ok(ordinary)
    }

    // For a current loss, the caller must first prepare its loss-year SSTB
    // eligibility using taxable income before the QBI deduction.
    // `section179` is the box 11 amount and its 2025 carryover allowed on
    // Form 4562 line 12 (whole dollars, section179::pass_through_allowed): the statement's ordinary
    // QBI is stated before it, and a part suspended by section 179(b)(3) is
    // not QBI until allowed (Instructions for Form 8995).
    pub(crate) fn activities(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
        section179: i64,
    ) -> Result<Vec<crate::business_rules::QbiActivity>, (&'static str, &'static str)> {
        self.activities_with_events(owner, suspended, section179, None)
            .map(|(activities, _)| activities)
    }

    pub(crate) fn activities_with_events(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
        section179: i64,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<ActivitiesAndCurrentLoss, (&'static str, &'static str)> {
        use crate::business_rules::QbiActivity;
        // An incomplete statement is left out by the callers (left_out).
        if !self.valid(owner) || !self.complete {
            return Err((
                "ISSUE_SCORP_QBI_COMPLETE_THE_ORDINARY_BUSINESS_QBI_STATEMENT_AND_7B111E6E",
                crate::fixture_english!(
                    "Complete the ordinary-business QBI statement and resolve additional shareholder adjustments."
                ),
            ));
        }
        let material = owner
            .participation
            .as_ref()
            .map(|v| v.material())
            .unwrap_or(crate::participation::Participation::Unavailable);
        if self.income > 0 && material == crate::participation::Participation::Unavailable {
            return Err((
                "ISSUE_SCORP_QBI_COMPLETE_PARTICIPATION_FACTS_FOR_THE_MINIMUM_80D7BD0E",
                crate::fixture_english!(
                    "Complete participation facts for the minimum QBI deduction; nonpassive trading classification alone does not establish active business income."
                ),
            ));
        }
        let passive = ordinary_passive_net_with_events(owner, events)?;
        if suspended.get() > phase::floor(-passive, 0) {
            return Err((
                "ISSUE_SCORP_QBI_THE_QBI_SUSPENSION_MUST_FIT_THIS_81DC6365",
                crate::fixture_english!(
                    "The QBI suspension must fit this activity's computed passive loss."
                ),
            ));
        }
        let basis = owner.basis_result()?;
        let risk = owner
            .risk
            .calculate_with_events(owner, &basis, events)
            .map_err(|error| error.message())?;
        let prior = owner.passive.loss;
        let allowed_loss = risk.ledger.risk_allowed[crate::k1_loss::K1LossClass::Ordinary].get()
            + prior
            - suspended.get();
        if allowed_loss < 0 {
            return Err((
                "ISSUE_SCORP_QBI_THE_QBI_SUSPENSION_IS_INCONSISTENT_WITH_20B70F99",
                crate::fixture_english!(
                    "The QBI suspension is inconsistent with allowed ordinary losses."
                ),
            ));
        }
        let qualified_loss = if allowed_loss == 0 {
            0
        } else {
            owner.loss_qbi.after_passive_with_events(owner, suspended, events)?.qbi.get()
        };
        let current_loss =
            if owner.ordinary < 0 { owner.loss_qbi.current_allowed(owner, suspended)? } else { 0 };
        let qualified_loss = qualified_loss - current_loss;
        if section179 < 0 || section179 > dollars(owner.section179 + owner.section179_carryover) {
            return Err((
                "ISSUE_SCORP_QBI_THE_SECTION_179_DEDUCTION_ALLOWED_FOR_4BE9C19A",
                crate::fixture_english!(
                    "The section 179 deduction allowed for this corporation does not reconcile with its K-1 box 11 amount."
                ),
            ));
        }
        let mut activities = alloc::vec![QbiActivity {
            source_wages_cents: Some(self.wages),
            source_property_cents: Some(self.property),
            source_income_cents: Some(self.income - current_loss - section179 * 100),
            owner: None,
            income: dollars(self.income - current_loss - section179 * 100),
            wages: dollars(self.wages),
            property: dollars(self.property),
            active: owner.participation.as_ref().map(|v| v.material())
                == Some(crate::participation::Participation::Yes),
            service: Some(if owner.ordinary < 0 { false } else { self.service }),
            group: None,
        }];
        if qualified_loss > 0 {
            // Historical SSTB eligibility was determined in the loss year.
            // Do not reapply the current year's SSTB percentage to this loss.
            activities.push(QbiActivity {
                source_wages_cents: Some(0),
                source_property_cents: Some(0),
                source_income_cents: Some(-qualified_loss),
                owner: None,
                income: -dollars(qualified_loss),
                wages: 0,
                property: 0,
                active: false,
                service: Some(false),
                group: None,
            });
        }
        Ok((activities, Amount::new(current_loss).expect("nonnegative current qualified release")))
    }
}

// Trace the current source-year portion through successive limitation stages.
// A new suspension vintage at a later stage can contain older source-year
// losses; keep its qualified ratio without reclassifying all of it as current.
pub(crate) fn current_qualified_release(
    qualified: i64,
    basis: &LossRelease,
    risk: &LossRelease,
    passive: Option<&LossRelease>,
) -> i64 {
    let balance = |release: &LossRelease| {
        release.remaining.iter().find(|v| v.year == 2026).map_or(0, |v| v.remaining_qbi.get())
    };
    let current = qualified - balance(basis);
    let current = qualified_component(basis.qbi.get() - balance(risk), current, basis.qbi.get());
    passive.map_or(current, |release| {
        qualified_component(risk.qbi.get() - balance(release), current, risk.qbi.get())
    })
}

fn qualified_component(allowed: i64, component: i64, total: i64) -> i64 {
    if total == 0 {
        return 0;
    }
    phase::bound(
        crate::kernel::ratio::fraction(allowed, component, total),
        phase::floor(allowed - (total - component), 0),
        phase::cap(component, [allowed]),
    )
}

// Single ordinary nonpassive business, current losses only. No allocation
// between businesses, historical vintages or investment-income classes is
// needed. The caller verifies that no other business/rental participates.
pub(crate) fn sole_business_excess(
    owner: &ScorpK1,
    activities: &mut [crate::business_rules::QbiActivity],
    excess: i64,
) -> Result<i64, (&'static str, &'static str)> {
    if owner.trade_kind != crate::ScorpTradeKind::Ordinary
        || owner.ordinary >= 0
        || owner.prior_loss != 0
        || owner.risk.prior_loss != 0
        || owner.passive.loss != 0
        || owner.participation.as_ref().map(|v| v.material())
            != Some(crate::participation::Participation::Yes)
        || activities.len() != 1
        || activities[0].income > 0
    {
        return Err((
            "ISSUE_SCORP_QBI_EXCESS_LOSS_ALLOCATION_ACROSS_BUSINESSES__PRIOR_LOSSES_E058DC3A",
            crate::fixture_english!(
                "Excess-loss allocation across businesses, prior losses or investment-income classes requires separate coordination."
            ),
        ));
    }
    let source_loss = Amount::new(-owner.activity_result()?.nonpassive).map_err(|_| {
        (
            "ISSUE_SCORP_QBI_THE_EXCESS_BUSINESS_LOSS_AND_QBI_550201BC",
            crate::fixture_english!(
                "The excess business loss and QBI character must fit the allowed ordinary loss."
            ),
        )
    })?;
    let current = activities[0]
        .source_income_cents
        .and_then(|cents| cents.checked_neg())
        .and_then(|cents| Amount::new(cents).ok())
        .ok_or((
            "ISSUE_SCORP_QBI_THE_EXCESS_BUSINESS_LOSS_AND_QBI_550201BC",
            crate::fixture_english!(
                "The excess business loss and QBI character must fit the allowed ordinary loss."
            ),
        ))?;
    if excess == 0 && current.get() <= source_loss.get() {
        return Ok(0);
    }
    defer_source_qbi(
        activities,
        Some(0),
        dollars(source_loss.get()),
        excess,
        current,
        current,
        source_loss,
    )
}

// Preserve qualified gain: defer only the qualified part of the resolved
// loss character, not a percentage of the combined net QBI activity.
pub(crate) fn excess_with_gain(
    owner: &ScorpK1,
    activities: &mut [crate::business_rules::QbiActivity],
    excess: i64,
    taxable: i64,
    status: crate::FederalStatus,
) -> Result<i64, (&'static str, &'static str)> {
    let character = crate::section1231::loss_with_gain(owner)?;
    let source_loss = character.amount();
    let loss = dollars(source_loss.get());
    if activities.is_empty() || excess < 0 || excess > loss {
        return Err((
            "ISSUE_SCORP_QBI_LOSS_DEFERRAL_NEEDS_RESOLVED_CURRENT_AND_9E75276B",
            crate::fixture_english!(
                "Loss deferral needs resolved current and historical QBI activities."
            ),
        ));
    }
    // With no section 199A statement the K-1 adds no QBI, so none of its
    // excess loss is a qualified loss to defer.
    let Some(statement) = owner.qbi_statement.as_ref() else {
        return Ok(0);
    };
    let prepared = statement.prepare(owner, taxable, status)?;
    let parts = prepared.pool_activity_result()?.characters.ok_or((
        "ISSUE_SCORP_QBI_RESOLVE_LOSS_CHARACTERS_AB0F7C8C",
        crate::fixture_english!("Resolve loss characters."),
    ))?;
    let (qualified, current) = match character {
        crate::section1231::ResolvedLoss::Operating(_) => {
            let facts = &prepared.loss_qbi;
            let suspended =
                Amount::new(-parts.passive[0]).expect("resolved ordinary loss is nonnegative");
            (
                facts.after_passive(&prepared, suspended)?.qbi.get(),
                facts.current_allowed(&prepared, suspended)?,
            )
        }
        crate::section1231::ResolvedLoss::Property(_) => {
            let facts = owner.property_qbi.calculation(owner, Some((taxable, status)))?;
            let suspended =
                Amount::new(-parts.passive[1]).expect("resolved property loss is nonnegative");
            (
                facts.after_passive(owner, suspended)?.qbi.get(),
                facts.current_allowed(owner, suspended)?,
            )
        }
    };
    defer_source_qbi(
        activities,
        Some(0),
        loss,
        excess,
        Amount::new(qualified).expect("qualified loss"),
        Amount::new(current).expect("current qualified loss"),
        source_loss,
    )
}

// One proven gross loss character, with current and originally qualified
// historical releases in the caller's existing activity positions.
pub(crate) fn defer_source_qbi(
    activities: &mut [crate::business_rules::QbiActivity],
    current_index: Option<usize>,
    reported_loss: i64,
    reported_excess: i64,
    qualified: Amount<Cents>,
    current: Amount<Cents>,
    source_gross_loss: Amount<Cents>,
) -> Result<i64, (&'static str, &'static str)> {
    let error = (
        "ISSUE_SCORP_QBI_QUALIFIED_LOSS_HISTORY_MUST_RECONCILE_TO_DDB88FE9",
        crate::fixture_english!(
            "Qualified loss history must reconcile to the resolved loss character."
        ),
    );
    let sources = activities
        .iter()
        .map(|activity| activity.source_income_cents)
        .collect::<Option<Vec<_>>>()
        .ok_or(error)?;
    let historical = sources
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != current_index)
        .try_fold(0i64, |total, (_, cents)| total.checked_sub(*cents))
        .ok_or(error)?;
    if reported_loss <= 0
        || reported_excess < 0
        || reported_excess > reported_loss
        || current_index.is_some_and(|index| index >= activities.len())
        || (current_index.is_none() && current.get() != 0)
        || qualified.get() > source_gross_loss.get()
        || current.get() > qualified.get()
        || historical != qualified.get() - current.get()
        || activities.iter().enumerate().any(|(index, activity)| {
            Some(index) != current_index && (sources[index] > 0 || activity.service != Some(false))
        })
    {
        return Err(error);
    }
    // Form461's reported excess/gross-loss fraction is dimensionless. Source
    // qualification stays in cents; separately reported rows are not reverse-lifted.
    let deferred_cents =
        crate::kernel::ratio::fraction(qualified.get(), reported_excess, reported_loss);
    let indices: Vec<_> = current_index
        .into_iter()
        .chain((0..activities.len()).filter(|index| Some(*index) != current_index))
        .collect();
    let weights: Vec<_> = indices
        .iter()
        .map(|index| if Some(*index) == current_index { current.get() } else { -sources[*index] })
        .collect();
    let allocated = crate::kernel::ratio::allocate_remaining(deferred_cents, &weights);
    for (index, amount) in indices.into_iter().zip(allocated) {
        let cents = sources[index].checked_add(amount).ok_or(error)?;
        activities[index].source_income_cents = Some(cents);
        activities[index].income = dollars(cents);
    }
    Ok(dollars(deferred_cents))
}

// Shared partial allowance for resolved loss-only sources. Earlier limits
// establish the qualified proportions, including original historical SSTB
// eligibility. Amounts disallowed again enter the current limitation year;
// do not apply the current SSTB percentage to historical losses a second time.
pub(crate) fn defer_loss_only_qbi(
    activities: &mut [crate::business_rules::QbiActivity],
    loss: i64,
    excess: i64,
) -> Result<i64, (&'static str, &'static str)> {
    if excess == 0 {
        return Ok(0);
    }
    if activities.is_empty()
        || activities.iter().any(|v| v.income > 0 || v.service != Some(false))
        || loss <= 0
        || excess < 0
        || excess > loss
    {
        return Err((
            "ISSUE_SCORP_QBI_EXCESS_LOSS_QBI_REQUIRES_RESOLVED_LOSS_ONLY_ACTIVITIES_1A1A4180",
            crate::fixture_english!(
                "Excess-loss QBI requires resolved loss-only activities and a valid total loss."
            ),
        ));
    }
    if activities.iter().any(|activity| activity.source_income_cents.is_some()) {
        let error = (
            "ISSUE_SCORP_QBI_QUALIFIED_LOSS_TOTAL_OVERFLOW_426F24B2",
            crate::fixture_english!("Qualified loss total overflow."),
        );
        let sources = activities
            .iter()
            .map(|activity| {
                activity.source_income_cents.or_else(|| activity.income.checked_mul(100))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(error)?;
        if sources.iter().any(|source| *source > 0) {
            return Err(error);
        }
        let qualified =
            sources.iter().try_fold(0i64, |sum, source| sum.checked_sub(*source)).ok_or(error)?;
        let deferred = crate::kernel::ratio::fraction(qualified, excess, loss);
        let weights: Vec<_> = sources.iter().map(|source| -*source).collect();
        let amounts = crate::kernel::ratio::allocate_remaining(deferred, &weights);
        for ((activity, source), amount) in activities.iter_mut().zip(sources).zip(amounts) {
            let source = source.checked_add(amount).ok_or(error)?;
            if activity.source_income_cents.is_some() {
                activity.source_income_cents = Some(source);
            }
            activity.income = dollars(source);
        }
        return Ok(dollars(deferred));
    }
    let qualified =
        activities.iter().try_fold(0i64, |sum, v| sum.checked_sub(v.income)).ok_or((
            "ISSUE_SCORP_QBI_QUALIFIED_LOSS_TOTAL_OVERFLOW_426F24B2",
            crate::fixture_english!("Qualified loss total overflow."),
        ))?;
    if qualified > loss {
        return Err((
            "ISSUE_SCORP_QBI_QUALIFIED_LOSSES_CANNOT_EXCEED_THE_ALLOWED_73D3C443",
            crate::fixture_english!("Qualified losses cannot exceed the allowed total loss."),
        ));
    }
    let deferred = crate::kernel::ratio::fraction(excess, qualified, loss);
    let weights: Vec<_> = activities.iter().map(|v| -v.income).collect();
    let amounts = crate::kernel::ratio::allocate_remaining(deferred, &weights);
    for (activity, amount) in activities.iter_mut().zip(amounts) {
        activity.income += amount;
    }
    Ok(deferred)
}

// Current ordinary amounts with identical qualified proportions have an
// unambiguous total deferred QBI, including profits offsetting current losses.
// This does not establish a rule for mixed proportions or historical losses.
pub(crate) fn uniform_business_excess(
    owners: &[&ScorpK1],
    activities: &mut [crate::business_rules::QbiActivity],
    excess: i64,
) -> Result<i64, (&'static str, &'static str)> {
    if owners.is_empty() || owners.len() != activities.len() || excess < 0 {
        return Err((
            "ISSUE_SCORP_QBI_EXCESS_LOSS_ALLOCATION_NEEDS_ONE_RESOLVED_CURRENT_D211A39E",
            crate::fixture_english!(
                "Excess-loss allocation needs one resolved current activity per ordinary business."
            ),
        ));
    }
    let mut ordinary_total = 0i64;
    let mut qualified_total = 0i64;
    let mut ordinary_net = 0i64;
    let mut first = None;
    let mut weights = Vec::new();
    for (owner, activity) in owners.iter().zip(activities.iter()) {
        let net = owner.activity_result()?.nonpassive;
        let source = activity.source_income_cents.ok_or((
            "ISSUE_SCORP_QBI_EXCESS_LOSS_ALLOCATION_NEEDS_ONE_RESOLVED_CURRENT_D211A39E",
            crate::fixture_english!(
                "Excess-loss allocation needs one resolved current activity per ordinary business."
            ),
        ))?;
        ordinary_net = ordinary_net.checked_add(dollars(net)).ok_or((
            "ISSUE_SCORP_QBI_ORDINARY_BUSINESS_NET_OVERFLOW_468BBBC0",
            crate::fixture_english!("Ordinary business net overflow."),
        ))?;
        if owner.ordinary < 0 {
            // Validate without changing the live result.
            let mut candidate = [activity.clone()];
            sole_business_excess(owner, &mut candidate, 0)?;
        } else if owner.trade_kind != crate::ScorpTradeKind::Ordinary
            || owner.prior_loss != 0
            || owner.risk.prior_loss != 0
            || owner.passive.loss != 0
            || owner.participation.as_ref().map(|v| v.material())
                != Some(crate::participation::Participation::Yes)
            || source < 0
            || source > net
            || activity.service != Some(false)
        {
            return Err((
                "ISSUE_SCORP_QBI_POSITIVE_EXCESS_LOSS_ALLOCATION_INPUTS_REQUIRE_CURRENT_1BCA0633",
                crate::fixture_english!(
                    "Positive excess-loss allocation inputs require current ordinary nonpassive income with eligibility-adjusted QBI character and no historical losses."
                ),
            ));
        }
        if net == 0 {
            weights.push(0);
            continue;
        }
        let ordinary = net.abs();
        let qualified = source.abs();
        if let Some((initial_ordinary, initial_qualified)) = first {
            if ordinary as i128 * initial_qualified as i128
                != qualified as i128 * initial_ordinary as i128
            {
                return Err((
                    "ISSUE_SCORP_QBI_DIFFERENT_QUALIFIED_LOSS_PROPORTIONS_NEED_A_SEPARATE_3D45B7F5",
                    crate::fixture_english!(
                        "Different qualified-loss proportions need a separate excess-loss allocation method."
                    ),
                ));
            }
        } else {
            first = Some((ordinary, qualified));
        }
        if net < 0 {
            ordinary_total = ordinary_total.checked_add(dollars(ordinary)).ok_or((
                "ISSUE_SCORP_QBI_ORDINARY_LOSS_TOTAL_OVERFLOW_CBAFF6CC",
                crate::fixture_english!("Ordinary loss total overflow."),
            ))?;
            qualified_total = qualified_total.checked_add(qualified).ok_or((
                "ISSUE_SCORP_QBI_QUALIFIED_LOSS_TOTAL_OVERFLOW_426F24B2",
                crate::fixture_english!("Qualified loss total overflow."),
            ))?;
            weights.push(qualified);
        } else {
            weights.push(0);
        }
    }
    if ordinary_total == 0 || excess > phase::floor(-ordinary_net, 0) {
        return Err((
            "ISSUE_SCORP_QBI_EXCESS_BUSINESS_LOSS_CANNOT_EXCEED_THE_6C6934CC",
            crate::fixture_english!(
                "Excess business loss cannot exceed the net current ordinary business loss."
            ),
        ));
    }
    let deferred = crate::kernel::ratio::fraction(excess, qualified_total, ordinary_total);
    let allocated = crate::kernel::ratio::allocate_remaining(deferred, &weights);
    for (activity, amount) in activities.iter_mut().zip(allocated) {
        let cents =
            activity.source_income_cents.and_then(|source| source.checked_add(amount)).ok_or((
                "ISSUE_SCORP_QBI_QUALIFIED_LOSS_TOTAL_OVERFLOW_426F24B2",
                crate::fixture_english!("Qualified loss total overflow."),
            ))?;
        activity.source_income_cents = Some(cents);
        activity.income = dollars(cents);
    }
    Ok(dollars(deferred))
}

// Form 8995-A loss-tracking worksheet, column D and row 3: losses
// released by earlier limits retain their qualified proportion when another
// provision suspends them. They enter that new provision in its year of
// disallowance. For loss-only sources, allocate the current Form 461
// disallowance proportionally; earlier FIFO releases are already resolved.
pub(crate) fn loss_only_business_excess(
    owners: &[&ScorpK1],
    activities: &mut [crate::business_rules::QbiActivity],
    reported_loss: i64,
    excess: i64,
) -> Result<i64, (&'static str, &'static str)> {
    if owners.is_empty()
        || activities.is_empty()
        || excess < 0
        || activities.iter().any(|v| v.income > 0 || v.service != Some(false))
    {
        return Err((
            "ISSUE_SCORP_QBI_HISTORICAL_EXCESS_LOSS_ALLOCATION_REQUIRES_RESOLVED_LOSS_ONLY_7F81444D",
            crate::fixture_english!(
                "Historical excess-loss allocation requires resolved loss-only QBI activities."
            ),
        ));
    }
    for owner in owners {
        if owner.ordinary > 0
            || owner.trade_kind != crate::ScorpTradeKind::Ordinary
            || owner.participation.as_ref().map(|v| v.material())
                != Some(crate::participation::Participation::Yes)
            || owner.qbi_statement.as_ref().is_none_or(|v| !v.complete)
        {
            return Err((
                "ISSUE_SCORP_QBI_HISTORICAL_EXCESS_LOSS_ALLOCATION_REQUIRES_COMPLETE_ORDINARY_C15F9B6A",
                crate::fixture_english!(
                    "Historical excess-loss allocation requires complete ordinary nonpassive loss sources."
                ),
            ));
        }
        let net = owner.activity_result()?.nonpassive;
        if net > 0 {
            return Err((
                "ISSUE_SCORP_QBI_POSITIVE_BUSINESS_INCOME_REQUIRES_SEPARATE_HISTORICAL_C49B8889",
                crate::fixture_english!(
                    "Positive business income requires separate historical loss allocation."
                ),
            ));
        }
    }
    defer_loss_only_qbi(activities, reported_loss, excess)
}

impl ScorpK1 {
    // Released qualified losses are a separate trade or business and retain
    // their original-year eligibility, even without a current K-1 statement
    // (Reg. 1.199A-3(b)(1)(iv), Form 8995 line 3 instructions).
    pub(crate) fn historical_qbi_activities(
        &self,
        suspended: [Amount<Cents>; 2],
        property_character: Option<crate::section1231::PropertyCharacter>,
    ) -> Result<Vec<crate::business_rules::QbiActivity>, (&'static str, &'static str)> {
        self.historical_qbi_activities_with_events(suspended, property_character, None)
    }

    pub(crate) fn historical_qbi_activities_with_events(
        &self,
        suspended: [Amount<Cents>; 2],
        property_character: Option<crate::section1231::PropertyCharacter>,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<Vec<crate::business_rules::QbiActivity>, (&'static str, &'static str)> {
        let source = self.qbi_projection();
        let ordinary = if self.loss_tracking() {
            source.loss_qbi.after_passive_with_events(&source, suspended[0], events)?.qbi.get()
        } else {
            0
        };
        let property = if self.property_tracking()
            && property_character == Some(crate::section1231::PropertyCharacter::Ordinary)
        {
            let mut facts = source.property_qbi.calculation(&source, None)?;
            facts.events = events.copied();
            facts.after_passive(&source, suspended[1])?.qbi.get()
        } else {
            0
        };
        Ok(if ordinary + property > 0 {
            alloc::vec![crate::business_rules::QbiActivity {
                source_wages_cents: Some(0),
                source_property_cents: Some(0),
                source_income_cents: Some(-(ordinary + property)),
                owner: None,
                income: -dollars(ordinary + property),
                wages: 0,
                property: 0,
                active: false,
                service: Some(false),
                group: None,
            }]
        } else {
            Vec::new()
        })
    }

    // Answers and unknowns: a statement not confirmed complete leaves this
    // corporation's QBI out of the deduction. The note says which way tax
    // may move: leaving out income overstates tax, leaving out a loss
    // (which offsets other businesses' QBI) understates it.
    pub(crate) fn qbi_left_out(&self) -> Option<&'static str> {
        let statement = self.qbi_statement.as_ref()?;
        if statement.complete && (self.section1231.is_none() || self.property_qbi.complete) {
            if self.section1231.as_ref().is_some_and(|property| {
                self.property_qbi.qualified_1250.is_none()
                    && property.unrecaptured > 0
                    && self.property_qbi.gain > 0
                    && self.property_qbi.gain < property.net
            }) {
                return Some("WARN_SCORP_QBI_ALLOCATION_LEFT_OUT");
            }
            return None;
        }
        let loss = self.has_ordinary_loss()
            || self.section1231.as_ref().is_some_and(|v| v.net < 0)
            || crate::k1_sale_qbi::has_loss(self.sale_results());
        Some(if loss { "WARN_SCORP_QBI_LOSS_LEFT_OUT" } else { "WARN_SCORP_QBI_LEFT_OUT" })
    }
    // Without a current statement, omit current reported QBI and explain the
    // possible tax effect. Previously qualified suspended losses remain
    // separate and count when allowed under their original-year records.
    pub(crate) fn note_no_qbi_statement(
        &self,
        entry: &crate::FederalIncomeEntry,
        result: &mut FederalResult,
    ) {
        if self.qbi_statement.is_none() {
            let ordinary_missing = self.loss_qbi.missing_history([
                self.prior_loss,
                self.risk.prior_loss,
                self.passive.loss,
            ]);
            let property_missing = self.section1231.as_ref().is_some_and(|property| {
                self.property_qbi.losses.missing_history([
                    property.basis_loss,
                    property.risk_loss,
                    property.passive_loss,
                ])
            });
            if ordinary_missing || property_missing {
                crate::federal_rules::note_once_scoped(
                    result,
                    crate::Message::new(
                        "WARN_SCORP_QBI_HISTORY_LEFT_OUT",
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                    crate::interview::income_issue_context(entry),
                );
            }
            let loss = self.ordinary < 0
                || self.section1231.as_ref().is_some_and(|v| v.net < 0)
                || crate::k1_sale_qbi::has_loss(self.sale_results());
            crate::federal_rules::note_once_scoped(
                result,
                crate::Message::new(
                    if loss { "WARN_K1_QBI_LOSS_NO_STATEMENT" } else { "WARN_K1_QBI_NO_STATEMENT" },
                    alloc::vec![crate::interview::income_display_arg(entry)],
                ),
                crate::interview::income_issue_context(entry),
            );
        }
    }
}

/// Section 1231 items potentially includible in QBI
/// Use this activity's section 199A statement and retained loss worksheets. Ordinary business QBI remains in its separate records. Capital treatment after return-wide section 1231 netting excludes the affected items; these source amounts are not a final QBI deduction. Zero and empty lists when the K-1 has no section 1231 item.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ScorpPropertyQbi {
    /// Potential QBI portion of current net section 1231 gain
    /// From the section 199A statement, before current-year SSTB limits and taxpayer-level capital/ordinary characterization. Zero for a current net loss or no section 1231 item. Do not add this amount to ordinary business QBI income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub gain: i64,
    /// Qualified portion of the section 1250 gain component
    /// Optional supporting-statement amount included in both the potential QBI gain above and this business's box 8c amount, before SSTB limits or lookback recapture. Zero confirms none; leave unanswered if unavailable. This source breakdown can resolve partial recapture when gain classes have different qualification. It is not a calculated recapture amount. Offsetting property losses require their separate allocations.
    /// [NullText]
    /// The documented qualified section 1250 portion is unavailable
    /// [SetText]
    /// The documented qualified section 1250 portion is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub qualified_1250: Option<i64>,
    /// Section 1231 loss character and separate suspension histories
    /// Reconcile to section 1231 balances, never ordinary or portfolio losses. The current QBI loss portion comes from the section 199A statement and precedes the current-year SSTB restriction; historical portions come from the QBI loss-tracking records kept with your 2025 return and already reflect original-year eligibility.
    pub losses: ScorpLossQbi,
    /// Complete same-business section 199A source allocation
    /// Confirm the statement covers this same business and the potential qualified portion has the same sign as its net section 1231 amount. Differing qualified/nonqualified components that offset within net box 9 need underlying transaction allocations. Historical qualification must be reconciled to original-year worksheets; a later ordinary classification cannot make a pre-2018 loss qualified. Compare the section 199A statement's section 1231 detail with the original-year worksheets. No leaves this corporation's qualified business income out of the QBI deduction, with a notice saying which way your tax may move.
    pub complete: bool,
}

impl ScorpPropertyQbi {
    pub(crate) fn valid_retained(&self) -> bool {
        self.retained_issue("").is_none()
    }

    pub(crate) fn retained_issue(&self, key: &'static str) -> Option<ValidationIssue> {
        // Arithmetic safety: calculation combines native property gain/loss
        // cents and later transfers allowed form dollars back to cents.
        if !(0..=100_000_000_000).contains(&self.gain) {
            return Some(ValidationIssue::new(key).at_field("gain"));
        }
        if self.qualified_1250.is_some_and(|qualified| !(0..=self.gain).contains(&qualified)) {
            return Some(ValidationIssue::new(key).at_field("qualified_1250"));
        }
        self.losses.retained_issue(key).map(|issue| issue.in_field("losses"))
    }

    pub(crate) fn valid(&self, owner: &ScorpK1) -> bool {
        self.validation_issue(owner, "").is_none()
    }

    pub(crate) fn validation_issue(
        &self,
        owner: &ScorpK1,
        key: &'static str,
    ) -> Option<ValidationIssue> {
        let Some(property) = owner.section1231.as_ref() else {
            return (!self.is_empty()).then(|| ValidationIssue::new(key).at_record());
        };
        // Arithmetic safety: calculation combines native property gain/loss
        // cents and later transfers allowed form dollars back to cents.
        if !(0..=100_000_000_000).contains(&self.gain) || self.gain > property.net.max(0) {
            return Some(ValidationIssue::new(key).at_field("gain"));
        }
        if let Some(qualified) = self.qualified_1250 {
            if !(0..=self.gain.min(property.unrecaptured)).contains(&qualified) {
                return Some(ValidationIssue::new(key).at_field("qualified_1250"));
            }
            if property.unrecaptured > property.net.max(0) {
                return Some(ValidationIssue::new(key).at_record());
            }
            if self.gain - qualified > property.net.max(0) - property.unrecaptured {
                return Some(ValidationIssue::new(key).at_field("gain"));
            }
        }
        self.losses
            .source_issue(LossSource::property(property), key)
            .map(|issue| issue.in_field("losses"))
    }

    // The section's value when the K-1 has no section 1231 item; the
    // confirmation is not read then.
    pub(crate) fn is_empty(&self) -> bool {
        self.gain == 0 && self.qualified_1250.unwrap_or(0) == 0 && self.losses.is_empty()
    }

    // The page rule SCORP_QBI_PROPERTY_RECORDS_NEEDED: empty without a
    // section 1231 item; with one, each carried section 1231 loss has its
    // loss-year history (validity reconciles the amounts).
    pub(crate) fn records_issue(&self, owner: &ScorpK1) -> Option<ValidationIssue> {
        let key = "SCORP_QBI_PROPERTY_RECORDS_NEEDED";
        match owner.section1231.as_ref() {
            None => {
                if self.is_empty() {
                    return None;
                }
                if self.gain != 0 {
                    return Some(ValidationIssue::new(key).at_field("gain"));
                }
                if self.qualified_1250.unwrap_or(0) != 0 {
                    return Some(ValidationIssue::new(key).at_field("qualified_1250"));
                }
                let field = if self.losses.current != 0 {
                    "current"
                } else if !self.losses.history.is_empty() {
                    "history"
                } else if !self.losses.at_risk.is_empty() {
                    "at_risk"
                } else {
                    "passive"
                };
                Some(ValidationIssue::new(key).at_field(field).in_field("losses"))
            }
            Some(v) => self
                .losses
                .records_issue(
                    phase::floor(v.net.saturating_neg(), 0),
                    [v.basis_loss, v.risk_loss, v.passive_loss],
                )
                .map(|mut issue| {
                    issue.key = key;
                    issue.in_field("losses")
                }),
        }
    }

    // Stored qualification is validated against the original box. Sale
    // transactions join only this runtime view, never the accepted statement.
    pub(crate) fn calculation(
        &self,
        owner: &ScorpK1,
        eligibility: Option<(i64, crate::FederalStatus)>,
    ) -> Result<PropertyQbiCalculation, (&'static str, &'static str)> {
        if !self.valid(owner) || (owner.section1231.is_some() && !self.complete) {
            return Err((
                "ISSUE_SCORP_QBI_RECONCILE_THE_SECTION_1231_QBI_STATEMENT_431CECE7",
                crate::fixture_english!(
                    "Reconcile the section 1231 QBI statement with its own gain and suspended-loss source records."
                ),
            ));
        }
        let property = owner.business_property()?.unwrap_or_default();
        let mut losses = self.losses.clone();

        let mut gain = self.gain;
        // A missing/declined current statement cannot qualify new sale items.
        // Existing histories keep their original loss-year qualification.
        if owner.qbi_statement.is_some() && owner.qbi_left_out().is_none() {
            for sale in owner.sale_results()? {
                gain += phase::floor(sale.section1231, 0) * 100;
                losses.current += phase::floor(-sale.section1231, 0) * 100;
            }
        }
        let unphased_loss = dollars(losses.current);
        if let Some((taxable, status)) = eligibility
            && owner.qbi_statement.as_ref().is_some_and(|statement| statement.service)
        {
            let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
                -losses.current,
                Some(true),
                false,
                taxable,
                status,
            )?;
            losses.current = crate::kernel::ratio::fraction(losses.current, applicable, range);
        }
        let original = owner.section1231.as_ref();
        Ok(PropertyQbiCalculation {
            source: LossSource {
                current: property.loss,
                // Retained histories and all loss stages use actual source cents.
                basis: original.map_or(0, |p| p.basis_loss),
                risk: original.map_or(0, |p| p.risk_loss),
                passive: original.map_or(0, |p| p.passive_loss),
                class: crate::k1_loss::K1LossClass::Section1231,
            },
            property,
            gain_cents: gain,
            qualified_1250: self.qualified_1250,
            events: None,
            unphased_loss,
            losses,
        })
    }

    #[cfg(test)]
    pub(crate) fn qualified_recapture(
        &self,
        owner: &ScorpK1,
        amount: i64,
    ) -> Result<i64, (&'static str, &'static str)> {
        self.calculation(owner, None)?.qualified_recapture(owner, amount)
    }

    #[cfg(test)]
    pub(crate) fn calculate(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        self.calculation(owner, None)?.calculate(owner, basis)
    }

    #[cfg(test)]
    pub(crate) fn after_at_risk(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        self.calculation(owner, None)?.after_at_risk(owner, basis)
    }

    #[cfg(test)]
    pub(crate) fn after_passive(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        self.calculation(owner, None)?.after_passive(owner, suspended)
    }

    #[cfg(test)]
    pub(crate) fn current_allowed(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
    ) -> Result<i64, (&'static str, &'static str)> {
        self.calculation(owner, None)?.current_allowed(owner, suspended)
    }

    pub(crate) fn report(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
        id: i64,
        suspended: Option<Amount<Cents>>,
        result: &mut FederalResult,
    ) {
        let facts = match self.calculation(owner, None) {
            Ok(facts) => facts,
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    result,
                    &crate::IssueContext::income_id(id),
                    "S-corporation property QBI history",
                    key,
                    message,
                );
                return;
            }
        };
        if facts.gain_cents > 0 && suspended.is_none() {
            result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, potential section 1231 QBI gain ${}. Apply current-year SSTB eligibility and return-wide capital exclusion before including any amount in QBI.", facts.gain_cents), "NOTE_SCORP_QBI_PROPERTY_GAIN", alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::usd_cents(facts.gain_cents)]));
        }
        if facts.losses.current > 0 && owner.qbi_statement.as_ref().is_none_or(|s| s.service) {
            result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, property QBI loss tracking: establish current-year service-business eligibility using final taxable income before fixing new qualified suspension balances. Historical eligibility is not redetermined."), "NOTE_S_CORPORATION_DOCUMENT_ID_PROPERTY_QBI_LOSS_TRACKING_EST_EB69F731", alloc::vec![crate::MessageArg::Number(id)]));
            return;
        }
        let stages = if let Some(suspended) = suspended {
            alloc::vec![(
                "passive",
                "NOTE_SCORP_QBI_PROPERTY_PASSIVE_RELEASE",
                "NOTE_SCORP_QBI_PROPERTY_PASSIVE_CARRY",
                facts.after_passive(owner, suspended)
            )]
        } else {
            alloc::vec![
                (
                    "basis",
                    "NOTE_SCORP_QBI_PROPERTY_BASIS_RELEASE",
                    "NOTE_SCORP_QBI_PROPERTY_BASIS_CARRY",
                    facts.calculate(owner, basis)
                ),
                (
                    "at-risk",
                    "NOTE_SCORP_QBI_PROPERTY_RISK_RELEASE",
                    "NOTE_SCORP_QBI_PROPERTY_RISK_CARRY",
                    facts.after_at_risk(owner, basis)
                )
            ]
        };
        for (_stage, release_key, carry_key, release) in stages {
            #[cfg(test)]
            let stage = _stage;

            match release {
                Ok(release) => {
                    result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, property loss passing {stage} limits: ${}; tracked QBI portion ${}. Intermediate loss character, subject to return-wide section 1231 capital exclusion and remaining return limits; not a QBI deduction.", release.amount.get(), release.qbi.get()), release_key, alloc::vec![crate::MessageArg::Number(id), note_usd(release.amount.get()), note_usd(release.qbi.get())]));
                    for year in release.remaining {
                        result.review_notes.push(note(#[cfg(test)] alloc::format!("S-corporation document {id}, {} property {stage} suspension: ${}; tracked QBI portion ${}; original suspended loss ${}; original QBI portion ${}. Retain separately from ordinary losses; final character and deductibility still require review.", year.year, year.remaining.get(), year.remaining_qbi.get(), year.original.get(), year.original_qbi.get()), carry_key, alloc::vec![crate::MessageArg::Number(id), crate::MessageArg::Number(i64::from(year.year)), note_usd(year.remaining.get()), note_usd(year.remaining_qbi.get()), note_usd(year.original.get()), note_usd(year.original_qbi.get())]));
                    }
                }
                Err((key, _message)) => {
                    #[cfg(test)]
                    let message = _message;
                    crate::issue_localized!(
                        result,
                        &crate::IssueContext::income_id(id),
                        "S-corporation property QBI history",
                        key,
                        message,
                    )
                }
            }
        }
    }
}

// Actual property and qualified gain cents reach the business owner unchanged.
// The lookback worksheet consumes its separately reported Form4797 amount.
pub(crate) struct PropertyQbiCalculation {
    events: Option<crate::scorp::ScorpEvents>,
    source: LossSource,
    property: crate::k1_items::BusinessProperty,
    pub(crate) gain_cents: i64,
    qualified_1250: Option<i64>,
    unphased_loss: i64,
    losses: ScorpLossQbi,
}

impl PropertyQbiCalculation {
    pub(crate) fn qualified_recapture(
        &self,
        _owner: &ScorpK1,
        amount: i64,
    ) -> Result<i64, (&'static str, &'static str)> {
        let total = dollars(self.property.gain);
        let qualified = dollars(self.gain_cents);
        if amount < 0 || amount > total {
            return Err((
                "ISSUE_SCORP_QBI_LOOKBACK_RECAPTURE_MUST_FIT_THE_CURRENT_AAFE1FFE",
                crate::fixture_english!(
                    "Lookback recapture must fit the current business's property gain."
                ),
            ));
        }
        let unresolved = (
            "ISSUE_SCORP_QBI_PARTIAL_LOOKBACK_RECAPTURE_OF_MIXED_QUALIFIED_NONQUALIFIED_9084BCB0",
            crate::fixture_english!(
                "Partial lookback recapture of mixed qualified/nonqualified property gains needs source-gain allocation."
            ),
        );
        if self.property.loss > 0 {
            // With no historical offsets, wholly qualified current sources
            // determine every ordinary recapture dollar without allocation.
            // The caller applies current-business eligibility once afterward.
            if qualified == total
                && self.unphased_loss == dollars(self.property.loss)
                && self.source.basis == 0
                && self.source.risk == 0
                && self.source.passive == 0
            {
                return Ok(amount);
            }
            return Err(unresolved);
        }
        if let Some(value) = crate::section1231::determined_share(amount, total, qualified) {
            return Ok(value);
        }
        let property = &self.property;
        if property.basis_loss != 0 || property.risk_loss != 0 || property.passive_loss != 0 {
            return Err(unresolved);
        }
        let qualified_high = self.qualified_1250.ok_or(unresolved)?;
        let high = dollars(property.unrecaptured);
        let low = total - high;
        // Reconcile independently rounded statement components to rounded
        // totals, preserving both qualified and rate-class amount bounds.
        let qualified_high = phase::bound(
            dollars(qualified_high),
            phase::floor(qualified - low, 0),
            phase::cap(qualified, [high]),
        );
        let recaptured_high = phase::cap(amount, [high]);
        let high_qbi = crate::section1231::determined_share(recaptured_high, high, qualified_high)
            .ok_or(unresolved)?;
        let low_qbi = crate::section1231::determined_share(
            amount - recaptured_high,
            low,
            qualified - qualified_high,
        )
        .ok_or(unresolved)?;
        Ok(high_qbi + low_qbi)
    }

    pub(crate) fn calculate(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        self.losses.basis_for(
            self.source,
            basis,
            Amount::new(owner.rentals.property_losses().iter().sum())
                .expect("nonnegative rental source losses"),
        )
    }

    pub(crate) fn after_at_risk(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        let source = self.source;
        let risk = owner
            .risk
            .calculate_with_events(owner, basis, self.events.as_ref())
            .map_err(|error| error.message())?;
        self.losses.risk_for(
            source,
            basis,
            Amount::new(
                risk.ledger.risk_allowed[crate::k1_loss::K1LossClass::Section1231].get()
                    - owner.rentals.property_allowed(&risk.ledger)?.iter().sum::<i64>(),
            )
            .expect("rental source allowance is part of the property class"),
            risk.ledger.risk_suspended[crate::k1_loss::K1LossClass::Section1231],
            Amount::new(owner.rentals.property_losses().iter().sum())
                .expect("nonnegative rental source losses"),
        )
    }

    pub(crate) fn after_passive(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
    ) -> Result<LossRelease, (&'static str, &'static str)> {
        let source = self.source;
        let activity = owner.pool_activity_with_events(self.events.as_ref())?;
        let net = activity.characters.map_or(0, |v| v.passive[1]);
        if suspended.get() > phase::floor(-net, 0) {
            return Err((
                "ISSUE_SCORP_QBI_THE_PROPERTY_QBI_SUSPENSION_MUST_FIT_CE29E6B2",
                crate::fixture_english!(
                    "The property QBI suspension must fit this character's passive-loss allocation."
                ),
            ));
        }
        let basis = owner.basis_result()?;
        let current = self.after_at_risk(owner, &basis)?;
        self.losses.passive_for(
            current,
            source.passive,
            owner.nonpassive() == crate::participation::Participation::Yes,
            suspended,
        )
    }

    pub(crate) fn current_allowed(
        &self,
        owner: &ScorpK1,
        suspended: Amount<Cents>,
    ) -> Result<i64, (&'static str, &'static str)> {
        let basis = self.calculate(owner, &owner.basis_result()?)?;
        let risk = self.after_at_risk(owner, &owner.basis_result()?)?;
        // Validate the character-specific final suspension even when current
        // nonpassive losses bypass it; only older losses can then be suspended.
        let passive = self.after_passive(owner, suspended)?;
        Ok(current_qualified_release(
            self.losses.current,
            &basis,
            &risk,
            (owner.nonpassive() != crate::participation::Participation::Yes).then_some(&passive),
        ))
    }
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpLossYear {
    amounts: [original, qualified, remaining, remaining_qualified],
    gains: [],
    facts: [year],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpLossQbi {
    amounts: [current, history, at_risk, passive],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpPropertyQbi {
    amounts: [gain, qualified_1250, losses],
    gains: [],
    facts: [complete],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpQbiStatement {
    amounts: [income, wages, property, reit_dividends, ptp_income],
    gains: [],
    facts: [service, complete],
});
