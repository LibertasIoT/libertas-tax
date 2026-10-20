// Coverage: Form5329 PRINT PartsIII/IV (2026-10-13): existing per-filer
//   computations project lines9–25, including reduction subtotals13/21; only
//   a carried positive-tax excess or current excess creates the corresponding
//   part. Corpus footings and two-filer/absorbed-carry tests preserve figures.
//   Source: IRS2026 draft f5329; prior-tax qualification is retained in IraExcess.
// Coverage: Form 1099-Q box 4b (529 to Roth IRA) — Supported (audited 2026-10-09, unit B3 Form
//   1099-Q). The rollover is one branch of the 1099-Q document (education_account.rs): boxes 2 and 3
//   of that form are the transfer's earnings and contribution portions, and QtpRothRollover holds
//   only the facts the form does not show. Lines: box 2/3 split into Roth basis, $35,000 lifetime
//   and annual-room limits. The lifetime use in 2024 and 2025 is asked on the IRA contributions page
//   (IraMore.qtp_roth_prior, stored as FederalIraContribution.qtp_roth_lifetime_prior). The taxable
//   earnings of an ordinary 529 or Coverdell distribution and the 10% additional tax are in
//   education_account.rs.
//   Unit B11 (2026-10-12): the part of a transfer above the year's room (the IRA dollar limit less the
//   year's contributions; compensation does not limit it) is not a qualified rollover: a Roth excess on
//   Form 5329 line 23 and a taxable 529 withdrawal (education_account.rs); a transfer whose records are
//   not confirmed is an ordinary 529 withdrawal with NOTE_QTP_ROTH_NOT_QUALIFIED. Form 5329 lines 10, 12
//   and 19 and the contributions returned in 2027 are in this module too (see retirement.rs).
//   Evidence: M/ira.rs, federal_rules.rs; T/mod.rs qtp_roth_rollover, two_qtp_roth_rollovers;
//   T/retirement.rs qtp_to_roth_rollover_uses_annual_room_but_not_income_or_saver_base; IRS Notice
//   2025-67, Pub 590-A.
//   State source capture (2026-09-30): QtpRothRollover.plan_state retains the
//   sponsoring state from plan records, without a guessed home/payer state;
//   tests/federal/state_return.rs verifies persistence and unchanged federal treatment.

// Form8379 line18 retains each owner's native PartsIII/IV tax; no joint
// excess re-rounding or wage-based allocation. T/injured_spouse.rs unit13_ira_excess_tax.
// 2026 limits: https://www.irs.gov/pub/irs-drop/n-25-67.pdf
// Worksheet ownership: https://www.irs.gov/publications/p590a (1-2, 2-2 and Appendix B).
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use crate::kernel::ratio::exact::{self, Exact};
use crate::*;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// The IRA-contributions editor's preflight, before cloning accepted state.
// Paths are relative to IraContributions, not to a stored FederalIras record.
pub(crate) fn contributions_issue(
    value: &IraContributions,
    d: &FederalDraft,
    owners: &[i32],
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    if let Some(index) = value.people.iter().position(|p| !owners.contains(&(p.owner as i32))) {
        return Some(
            ValidationIssue::new(
                "MESSAGE_YOUR_SPOUSE_S_CONTRIBUTIONS_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            )
            .at_field("owner")
            .in_list("people", index),
        );
    }
    if value.people.len() == 2 && value.people[0].owner == value.people[1].owner {
        return Some(
            ValidationIssue::new("IRA_CONTRIBUTOR_TWICE").at_field("owner").in_list("people", 1),
        );
    }
    if let Some(index) = value.people.iter().position(|p| p.more.nondeductible > p.traditional) {
        return Some(
            ValidationIssue::new("IRA_NONDEDUCTIBLE_ABOVE_TRADITIONAL")
                .at_field("nondeductible")
                .in_field("more")
                .in_list("people", index),
        );
    }
    for (index, p) in value.people.iter().enumerate() {
        let Some(excess) = &p.more.excess else {
            continue;
        };
        // The actual accepted Form 8606 is authoritative. Ignore the editor's
        // read-only form_8606_value, retaining the source's unknown value.
        let key = match (excess.traditional_value, crate::interview::form_8606_year_end(d, p.owner))
        {
            (Some(_), Some(_)) => Some("IRA_EXCESS_TRADITIONAL_ON_8606"),
            (None, None) => Some("IRA_EXCESS_TRADITIONAL_VALUE_NEEDED"),
            _ => None,
        };
        if let Some(key) = key {
            return Some(
                ValidationIssue::new(key)
                    .at_field("traditional_value")
                    .in_field("excess")
                    .in_field("more")
                    .in_list("people", index),
            );
        }
        if excess.returned_prior_excess > excess.prior_traditional {
            return Some(
                ValidationIssue::new("IRA_RETURNED_EXCESS_ABOVE_CARRIED")
                    .at_field("returned_prior_excess")
                    .in_field("excess")
                    .in_field("more")
                    .in_list("people", index),
            );
        }
        let withdrawn: i64 = d
            .income
            .iter()
            .filter(|e| e.owner == p.owner)
            .map(|e| match &e.income {
                FederalIncome::Retirement { data } if data.kind == RetirementKind::Ira => {
                    data.gross
                }
                _ => 0,
            })
            .sum();
        if excess.returned_prior_excess > withdrawn {
            return Some(
                ValidationIssue::new("IRA_RETURNED_EXCESS_ABOVE_DISTRIBUTIONS")
                    .at_field("returned_prior_excess")
                    .in_field("excess")
                    .in_field("more")
                    .in_list("people", index),
            );
        }
    }
    for (index, p) in value.people.iter().enumerate() {
        for (returned_index, returned) in p.more.returned_in_following_year.iter().enumerate() {
            if let Some(issue) = returned.issue() {
                return Some(
                    issue
                        .in_list("returned_in_following_year", returned_index)
                        .in_field("more")
                        .in_list("people", index),
                );
            }
            if !returned.valid() {
                // This existing intrinsic fallback combines date and amount
                // checks. Locate its actual record without guessing a leaf.
                return Some(
                    ValidationIssue::new("IRA_RETURN_LOSS_ABOVE_PRINCIPAL")
                        .at_list_item("returned_in_following_year", returned_index)
                        .in_field("more")
                        .in_list("people", index),
                );
            }
        }
    }
    None
}

// IRC 529(c)(3)(E), 408A(c)(3)(E), (c)(5)(B), (e)(1)(C):
// https://uscode.house.gov/view.xhtml?req=(title:26%20section:529%20edition:prelim)
// https://uscode.house.gov/view.xhtml?req=(title:26%20section:408a%20edition:prelim)
/// Direct 2026 transfer from a long-held section 529 plan to its beneficiary's Roth IRA
/// Form 1099-Q box 4b is checked: the 529 trustee moved this money directly to the beneficiary's Roth IRA. The form's earnings (box 2) and basis (box 3) are the transfer's earnings and contribution portions. A qualified rollover, reported by the Roth custodian in 2026 Form 5498 box 10. It shares the annual IRA dollar and compensation room with regular contributions but is not a regular Roth contribution or a Saver's Credit contribution.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QtpRothRollover {
    /// Source 529 account reference
    /// Use a short local reference or the Form 1099-Q account identifier; do not enter a full account number. Use the same reference for multiple Forms 1099-Q from the same 529 account, and different references for different accounts.
    #[libertas_size(min = 1)]
    pub account_reference: String,
    /// State sponsoring the source 529 plan
    /// Use the plan's records, not your home state, school location or the payer's address. Leave undefined when the plan does not identify a sponsoring state.
    /// [NullText]
    /// The plan records do not identify a sponsoring state
    /// [SetText]
    /// The source 529 plan's sponsoring state is
    #[libertas_default($null)]
    pub plan_state: Option<crate::TaxState>,
    /// Date the source 529 account was opened for this beneficiary
    /// The account must have been maintained for this same beneficiary throughout the 15-year period ending on the direct transfer date. A beneficiary change or uncertain account history needs separate review.
    #[libertas_date_only]
    pub account_opened_on: u32,
    /// Date the 529 trustee transferred the money directly to the Roth IRA trustee
    /// This entry covers a transfer completed in 2026 and designated for 2026. A transfer in 2027 designated for 2026 needs a separate timing worksheet.
    #[libertas_date_only]
    pub transferred_on: u32,
    /// Amount of the source account available from deposits made at least five years earlier
    /// Use this source account's administrator-reconciled balance of contributions made before the five-year period ending on this transfer date plus earnings attributable to them, immediately before this transfer and after all earlier distributions and transfers, including earlier listed transfers on the same date. It must cover this entire transfer. Do not infer it from the account opening date or another account's balance.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub seasoned_available: i64,
    /// Transfer and source records confirmed
    /// Check Form 1099-Q, the 529 plan statements and Form 5498: this account's history for the same beneficiary throughout the 15 years, the direct trustee-to-trustee transfer into that person's own Roth IRA, this account's five-year seasoned balance after earlier distributions, and that the earlier transfers on the IRA contributions page include every earlier qualifying transfer. No means the transfer is not a qualified rollover: the form is taxed as an ordinary 529 withdrawal and the amount deposited is a regular Roth contribution you enter on the IRA contributions page.
    pub confirmed: bool,
}

impl QtpRothRollover {
    // The shape of the stored facts, whatever their tax meaning.
    pub(crate) fn well_formed(&self) -> bool {
        let date = crate::federal_rules::valid_date;
        !self.account_reference.trim().is_empty()
            && date(self.account_opened_on)
            && date(self.transferred_on)
            && (0..=100_000_000_000).contains(&self.seasoned_available)
    }

    // The facts of box 4b against the transfer's amount (the 1099-Q's boxes 2
    // and 3, in cents).
    pub(crate) fn valid(&self, amount: i64) -> bool {
        self.well_formed()
            && (20260101..=20261231).contains(&self.transferred_on)
            && self.account_opened_on / 10000 + crate::year_constants::QTP_ROTH_ACCOUNT_YEARS
                <= self.transferred_on / 10000
            && self.account_opened_on + crate::year_constants::QTP_ROTH_ACCOUNT_YEARS * 10000
                <= self.transferred_on
            && (1..=crate::year_constants::QTP_ROTH_LIFETIME_LIMIT_CENTS).contains(&amount)
            && amount <= self.seasoned_available
    }
}

// One 529-to-Roth transfer as the calculation reads it: the facts of box 4b
// with the same Form 1099-Q's contribution portion (box 3) and earnings
// portion (box 2), in cents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QtpTransfer {
    pub(crate) id: i64,
    pub(crate) rollover: QtpRothRollover,
    pub(crate) contributions: i64,
    pub(crate) earnings: i64,
}

impl QtpTransfer {
    pub(crate) fn amount(&self) -> i64 {
        self.contributions + self.earnings
    }

    pub(crate) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.contributions)
            && (0..=100_000_000_000).contains(&self.earnings)
            && self.rollover.valid(self.amount())
    }

    pub(crate) fn eligible(&self) -> bool {
        self.valid() && self.rollover.confirmed
    }
}

// Each owner's 529-to-Roth transfers in their order: transfer date, then the
// order the forms were entered (the same-account seasoned-balance check reads
// earlier transfers first). Index 0 is the taxpayer, 1 the spouse.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct QtpTransfers(pub(crate) [alloc::vec::Vec<QtpTransfer>; 2]);

impl QtpTransfers {
    pub(crate) fn of(income: &[FederalIncomeEntry]) -> Self {
        let mut all = Self::default();
        for entry in income {
            if let FederalIncome::EducationDistribution { data } = &entry.income
                && let crate::Q1099Transfer::RothIra { rollover } = &data.transfer
                && let crate::EducationAccount::Qtp { earnings, basis } = &data.account
            {
                // A transfer whose records are not confirmed is not a qualified rollover: its
                // form is an ordinary 529 withdrawal (education_account.rs).
                if !rollover.confirmed {
                    continue;
                }
                all.0[usize::from(entry.owner == Owner::Spouse)].push(QtpTransfer {
                    id: entry.id,
                    rollover: rollover.clone(),
                    contributions: *basis,
                    earnings: *earnings,
                });
            }
        }
        for owner in &mut all.0 {
            owner.sort_by_key(|t| t.rollover.transferred_on);
        }
        all
    }

    pub(crate) fn owner(&self, index: usize) -> &[QtpTransfer] {
        &self.0[index]
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.iter().all(|owner| owner.is_empty())
    }
}

// The IRA facts the rules work with: the entered ones, or, for a return whose
// only IRA fact is a 529-to-Roth transfer on a Form 1099-Q (no contributions
// page was saved), the empty facts of the filers on this return so the same
// annual room and Roth basis rules run.
pub(crate) fn facts<'a>(
    d: &'a FederalDraft,
    transfers: &QtpTransfers,
) -> Option<alloc::borrow::Cow<'a, FederalIras>> {
    let adjustments = d.adjustments.as_ref()?;
    if let Some(iras) = adjustments.entered_iras() {
        return Some(alloc::borrow::Cow::Borrowed(iras));
    }
    if transfers.is_empty() {
        return None;
    }
    let setup = d.setup.as_ref()?;
    let covered = |owner| FederalIraContribution {
        covered_at_work: crate::interview::ira_covered(d, owner),
        ..FederalIraContribution::NONE
    };
    let spouse = match (setup.filing_status(), setup.lived_apart_all_year()) {
        (FederalStatus::Joint, _) => IraSpouse::Joint { facts: covered(Owner::Spouse) },
        (FederalStatus::Separate, false) => IraSpouse::SeparateTogether {
            covered_at_work: matches!(
                adjustments.iras.spouse,
                IraSpouse::SeparateTogether { covered_at_work: true }
            ),
        },
        _ => IraSpouse::NotApplicable,
    };
    Some(alloc::borrow::Cow::Owned(FederalIras { taxpayer: covered(Owner::Taxpayer), spouse }))
}

pub(crate) fn valid_qtp_rollovers(rollovers: &[QtpTransfer], lifetime_prior: i64) -> bool {
    (0..=crate::year_constants::QTP_ROTH_LIFETIME_LIMIT_CENTS).contains(&lifetime_prior)
        && rollovers.iter().all(QtpTransfer::valid)
        && rollovers.iter().map(QtpTransfer::amount).sum::<i64>()
            <= crate::year_constants::QTP_ROTH_LIFETIME_LIMIT_CENTS - lifetime_prior
        && rollovers.iter().enumerate().all(|(index, entry)| {
            rollovers[..index].iter().all(|earlier| {
                earlier.rollover.account_reference != entry.rollover.account_reference
                    || (earlier.rollover.account_opened_on == entry.rollover.account_opened_on
                        && (earlier.rollover.transferred_on != entry.rollover.transferred_on
                            || entry.rollover.seasoned_available
                                <= earlier.rollover.seasoned_available - earlier.amount()))
            })
        })
}

pub(crate) fn qtp_rollover_amount(rollovers: &[QtpTransfer]) -> i64 {
    rollovers.iter().map(QtpTransfer::amount).sum()
}

// Form 5329 line 12 (section 408(d)(5)): the part of the owner's 2026 traditional IRA withdrawals
// that paid back an excess contribution of a year before 2026 is on Form 1040 line 4a only: it is
// neither taxable nor a distribution of the IRA's basis (Form 8606 line 7). It leaves the
// owner's distributions the way a qualified HSA funding distribution does, so the exclusion
// is added to that result (allocated to the owner's IRA rows in entry order).
pub(crate) fn exclude_returned_prior_excess(
    d: &FederalDraft,
    funding: &mut crate::hsa_funding::FundingResult,
) {
    let Some(facts) = d.adjustments.as_ref().and_then(|a| a.entered_iras()) else {
        return;
    };
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let v =
            if owner == Owner::Taxpayer { Some(&facts.taxpayer) } else { facts.spouse.as_ref() };
        let mut left = v.and_then(|v| v.excess.as_ref()).map_or(0, |e| e.returned_prior_excess);
        let mut entries: alloc::vec::Vec<_> = d
            .income
            .iter()
            .filter(|e| e.owner == owner)
            .filter_map(|e| match &e.income {
                FederalIncome::Retirement { data }
                    if data.kind == crate::RetirementKind::Ira && !data.outside_form_8606() =>
                {
                    Some((e.id, data))
                }
                _ => None,
            })
            .collect();
        entries.sort_by_key(|(id, _)| *id);
        for (id, data) in entries {
            if left == 0 {
                break;
            }
            let room = phase::floor(
                data.gross - data.rolled_amount() - data.converted_amount() - funding.for_entry(id),
                0,
            );
            let taken = phase::cap(left, [room]);
            if taken > 0 {
                *funding.excluded.entry(id).or_insert(0) += taken;
                left -= taken;
            }
        }
    }
}

// Each owner's carried Roth excess (Form 5329 line 18) that this year's unused Roth room takes in
// (line 19, less the owner's Roth withdrawals of line 20), in cents. It becomes a 2026 regular
// Roth contribution for the Roth ordering (section 4973(f)). Provisional: it runs before the
// income is known, so the room is the IRA dollar limit without the income phaseout and without
// the compensation limit; the Form 5329 lines themselves use the exact limits (excess_tax).
pub(crate) fn provisional_roth_absorbed(d: &FederalDraft) -> [i64; 2] {
    let mut out = [0; 2];
    let Some(facts) = d.adjustments.as_ref().and_then(|a| a.entered_iras()) else {
        return out;
    };
    for (owner, slot) in out.iter_mut().enumerate() {
        let (Some(person), Some(v)) = (
            d.people
                .as_ref()
                .and_then(|p| if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() }),
            if owner == 0 { Some(&facts.taxpayer) } else { facts.spouse.as_ref() },
        ) else {
            continue;
        };
        let Some(excess) = v.excess.as_ref() else {
            continue;
        };
        if excess.prior_roth == 0 {
            continue;
        }
        let withdrawn: i64 = d
            .income
            .iter()
            .filter(|e| usize::from(e.owner == Owner::Spouse) == owner)
            .map(|e| match &e.income {
                FederalIncome::Roth { data } if data.inherited == crate::InheritedLink::No => {
                    data.gross
                }
                _ => 0,
            })
            .sum();
        let room = phase::floor(contribution_limit(person) * 100 - v.traditional - v.roth, 0);
        *slot = phase::cap(
            room,
            [crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
                excess.prior_roth,
                withdrawn,
            )
            .get()],
        );
    }
    out
}

// The part of each 529-to-Roth transfer above the year's room, by Form 1099-Q entry, in cents
// (section 529(c)(3)(E)(ii)(II): the IRA dollar limit less the beneficiary's contributions to all
// individual retirement plans for the year; compensation does not limit a rollover). Regular
// contributions use the room first; the transfers fill what is left in their date order, and the
// rest of a transfer is not a qualified rollover.
pub(crate) fn qtp_excess_by_entry(d: &FederalDraft) -> alloc::collections::BTreeMap<i64, i64> {
    let mut out = alloc::collections::BTreeMap::new();
    let transfers = QtpTransfers::of(&d.income);
    let Some(adjustments) = d.adjustments.as_ref() else {
        return out;
    };
    let facts = adjustments.entered_iras();
    for owner in 0..2 {
        let person = d
            .people
            .as_ref()
            .and_then(|p| if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
        let contributions = facts.map_or(0, |f| {
            let v = if owner == 0 { Some(&f.taxpayer) } else { f.spouse.as_ref() };
            v.map_or(0, |v| v.traditional + v.roth)
        });
        let Some(person) = person else {
            continue;
        };
        let mut room = phase::floor(contribution_limit(person) * 100 - contributions, 0);
        for transfer in transfers.owner(owner) {
            let qualified = crate::kernel::carry::native_consume_step(transfer.amount(), &mut room);
            out.insert(
                transfer.id,
                crate::kernel::carry::native_balance(transfer.amount(), qualified),
            );
        }
    }
    out
}

/// Returned contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ReturnedContribution {
    /// Kind of IRA
    #[libertas_default(Traditional)]
    pub kind: IraKind,
    /// Contribution principal returned
    /// The original 2026 contribution, even when a loss reduced the cash paid back.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub principal: i64,
    /// Earnings paid with it, or the loss
    /// The custodian's net income allocable to the contribution; a loss is negative. Positive earnings are 2026 income.
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub earnings: i64,
    /// Date the custodian paid it back
    #[libertas_date_only]
    // Treas. Reg. 301.9100-2: the six-month corrective-return extension ends October 15, 2027.
    #[libertas_number(min = 20270101, max = 20271015)]
    pub returned_on: u32,
    /// How a return after April 15, 2027 was still timely
    /// Leave empty for a return by April 15, 2027.
    /// [NullText]
    /// The contribution was returned by April 15, 2027
    /// [SetText]
    /// The later return was timely because
    #[libertas_default($null)]
    pub late: Option<LateCorrection>,
}
/// Kind of IRA
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum IraKind {
    /// Traditional IRA
    Traditional,
    /// Roth IRA
    Roth,
}
/// How a return after April 15, 2027 was still timely
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum LateCorrection {
    /// A filing extension to October 15, 2027
    /// Form 4868 was filed by April 15, 2027.
    FilingExtension,
    /// The return was filed on time and the contribution returned within six months after April 15, 2027
    /// Treas. Reg. 301.9100-2: an amended 2026 return reports the earnings, which this App does not file.
    TimelyFiledReturn,
}
impl ReturnedContribution {
    pub(crate) fn valid(&self) -> bool {
        crate::federal_rules::valid_date(self.returned_on)
            && (20270101..=20271015).contains(&self.returned_on)
            && (1..=100_000_000_000).contains(&self.principal)
            && (-self.principal..=100_000_000_000 - self.principal).contains(&self.earnings)
    }
    // The client rules of the record, checked again on save.
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let issue = |key, field| crate::validation::ValidationIssue::new(key).at_field(field);
        if self.returned_on > 20270415 && self.late.is_none() {
            Some(issue("IRA_RETURN_AFTER_DUE_DATE_NEEDS_ROUTE", "late"))
        } else if self.returned_on <= 20270415 && self.late.is_some() {
            Some(issue("IRA_RETURN_ROUTE_WITHOUT_LATE_DATE", "late"))
        } else if i128::from(self.principal) + i128::from(self.earnings) < 0 {
            Some(issue("IRA_RETURN_LOSS_ABOVE_PRINCIPAL", "earnings"))
        } else {
            None
        }
    }
    // Form 1040 line 4a: the cash the custodian paid back.
    pub(crate) fn gross(&self) -> i64 {
        self.principal + self.earnings
    }
    // Form 1040 line 4b: positive earnings; a loss is nothing.
    pub(crate) fn taxable(&self) -> i64 {
        phase::floor(self.earnings, 0)
    }
}

#[derive(Clone, Default)]
pub(crate) struct IraResult {
    pub deduction: i64,
    pub deduction_by_owner: [i64; 2],
    // Pub974(2025), printed52: monetary inputs inside Iterative work retain
    // cents. Existing annual compensation and prescribed $10 phaseout limits
    // remain this owner's inputs; filing deduction/basis outputs stay separate.
    pub work_deduction_cents: i64,
    pub nondeductible: [i64; 2],
    pub contributions: [i64; 2], // cents, for the shared Saver's Credit base
    // Form 5329 line 10 absorbed into this year's traditional contribution, in cents.
    pub absorbed: [i64; 2],
}

// Form 5329 line 10: the carried traditional excess (line 9 less the part returned, line 12, and
// the 2026 distributions included in income, line 11) that this year's unused room takes in
// (line 10: the limit less traditional and Roth contributions). Cents.
pub(crate) fn absorbed_traditional(v: &FederalIraContribution, room: i64, line_11: i64) -> i64 {
    let Some(excess) = v.excess.as_ref() else {
        return 0;
    };
    if excess.prior_traditional == 0 {
        return 0;
    }
    let line_10 = phase::floor(room - v.traditional - v.roth, 0);
    phase::cap(
        line_10,
        [crate::kernel::carry::native_remaining::<crate::kernel::carry::Cents>(
            crate::kernel::carry::native_balance(excess.prior_traditional, line_11),
            excess.returned_prior_excess,
        )
        .get()],
    )
}

impl IraNextYearDeposits {
    pub(crate) fn valid(&self) -> bool {
        (20270101..=20271231).contains(&self.last_deposited_on)
            && crate::federal_rules::valid_date(self.last_deposited_on)
            && (0..=100_000_000_000).contains(&self.traditional)
            && (0..=100_000_000_000).contains(&self.roth)
            && (self.traditional > 0 || self.roth > 0)
    }

    pub(crate) fn eligible(&self, facts: &FederalIraContribution) -> bool {
        self.valid()
            && self.last_deposited_on <= 20270415
            && self.designation_confirmed
            && self.traditional <= facts.traditional
            && self.roth <= facts.roth
    }
}

fn contribution_limit(person: &FederalPerson) -> i64 {
    // IRC 219(b)(5)(B): age 50 before the close of the contribution year,
    // day-exact (Pub. 501): a person born January 1, 1977 attained 50 on
    // December 31, 2026.
    if crate::federal_model::age_at_year_end(person.birth_date) >= 50 { 8600 } else { 7500 }
}

// Partial IRA limits round UP to $10 with a $200 minimum, but vanish at the endpoint.
// This differs from ordinary tax-line rounding and from deductions reduced in $1,000 steps.
pub(crate) fn phase_limit(cap: i64, magi: i64, start: i64, width: i64) -> i64 {
    phase::linear_up(cap, magi, start, width, 10, 200)
}

// Refigure only the chapter 1 deduction limit with the AMT compensation. Actual contributions,
// Form 8606 basis and Form 5329 excess stay regular.
pub(crate) fn amt_deduction(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: [i64; 2],
    traditional_magi: i64,
    transfers: &QtpTransfers,
    distributions: [i64; 2],
) -> Result<i64, (&'static str, &'static str)> {
    calculate(f, p, s, compensation, traditional_magi, transfers, distributions)
        .map(|result| result.deduction)
}

pub(crate) fn calculate(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: [i64; 2],
    traditional_magi: i64,
    transfers: &QtpTransfers,
    distributions: [i64; 2],
) -> Result<IraResult, (&'static str, &'static str)> {
    calculate_with_magi_cents(
        f,
        p,
        s,
        compensation,
        traditional_magi * 100,
        transfers,
        distributions,
    )
}

pub(crate) fn calculate_with_magi_cents(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: [i64; 2],
    traditional_magi_cents: i64,
    transfers: &QtpTransfers,
    distributions: [i64; 2],
) -> Result<IraResult, (&'static str, &'static str)> {
    calculate_with_difficulty_of_care(
        f,
        p,
        s,
        Compensation { taxable: compensation, difficulty_of_care: [0; 2] },
        traditional_magi_cents,
        transfers,
        distributions,
    )
}

pub(crate) struct Compensation {
    pub taxable: [i64; 2],
    pub difficulty_of_care: [i64; 2],
}

pub(crate) fn calculate_with_difficulty_of_care(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: Compensation,
    traditional_magi_cents: i64,
    transfers: &QtpTransfers,
    distributions: [i64; 2],
) -> Result<IraResult, (&'static str, &'static str)> {
    let Compensation { taxable: compensation, difficulty_of_care } = compensation;
    let joint = s.filing_status() == FederalStatus::Joint;
    if joint != f.spouse.is_some() || joint != p.spouse.is_some() {
        return Err((
            "ISSUE_IRA_SPOUSE_COVERAGE",
            crate::fixture_english!(
                "Complete both IRA coverage records for a joint return; remove spouse IRA facts for any other filing status."
            ),
        ));
    }
    let facts = [Some(&f.taxpayer), f.spouse.as_ref()];
    let people = [Some(&p.taxpayer), p.spouse.as_ref()];
    if facts
        .iter()
        .flatten()
        .any(|f| f.next_year_deposits.as_ref().is_some_and(|deposits| !deposits.eligible(f)))
    {
        return Err((
            "ISSUE_IRA_NEXT_YEAR_DEPOSITS",
            crate::fixture_english!(
                "Confirm every actual 2027 IRA deposit was made by April 15 and designated for 2026 with its custodian. Its traditional and Roth amounts must be included once in the matching retained 2026 contribution totals."
            ),
        ));
    }
    if facts.iter().enumerate().any(|(i, f)| {
        f.is_some_and(|f| {
            !valid_qtp_rollovers(transfers.owner(i), f.qtp_roth_lifetime_prior)
                || transfers.owner(i).iter().any(|t| !t.eligible())
        })
    }) {
        return Err((
            "ISSUE_IRA_QTP_ROLLOVERS",
            crate::fixture_english!(
                "Confirm each 2026 direct 529-to-Roth transfer, its distinct source-account history and five-year seasoned amount after earlier transfers, its source contribution/earnings split, and the $35,000 beneficiary lifetime room across all accounts."
            ),
        ));
    }
    let separate_together =
        s.filing_status() == FederalStatus::Separate && !s.lived_apart_all_year();
    if separate_together != matches!(f.spouse, IraSpouse::SeparateTogether { .. }) {
        return Err((
            "ISSUE_IRA_SPOUSE_SITUATION",
            crate::fixture_english!(
                "Choose the IRA spouse situation matching filing status and whether you lived together during the year."
            ),
        ));
    }
    let mut result = IraResult::default();
    let mut annual_totals = [0; 2];
    for (i, fact) in facts.iter().enumerate() {
        if let Some(v) = fact {
            result.contributions[i] = v.traditional + v.roth;
            annual_totals[i] = result.contributions[i] + qtp_rollover_amount(transfers.owner(i));
        }
    }
    for i in 0..2 {
        let (Some(v), Some(person)) = (facts[i], people[i]) else {
            continue;
        };
        if !(0..=v.traditional).contains(&v.nondeductible_election) {
            return Err((
                "ISSUE_IRA_NONDEDUCTIBLE_ELECTION",
                crate::fixture_english!(
                    "The voluntary nondeductible amount must be between zero and this filer's regular traditional contributions."
                ),
            ));
        }
        let cap = contribution_limit(person);
        let available = if joint && compensation[i] < compensation[1 - i] {
            phase::floor(compensation.iter().sum::<i64>() * 100 - annual_totals[1 - i], 0)
        } else {
            phase::floor(compensation[i], 0) * 100
        };
        // Contributions above the limit are not refused here: the deduction
        // stops at the limit and Form 5329 Parts III/IV (excess_tax) tax the
        // rest, and a 529-to-Roth transfer above the shared room is a Roth excess.
        let room = phase::cap(cap * 100, [available + difficulty_of_care[i]]);
        // Form 5329 line 10: a carried traditional excess absorbed by this year's unused room is
        // a 2026 contribution for the deduction and Form 8606 line 1.
        let absorbed = absorbed_traditional(v, room, distributions[i]);
        result.absorbed[i] = absorbed;
        let traditional = v.traditional + absorbed;
        if traditional == 0 {
            continue;
        }
        // Form 8606 line 1 holds only the permitted contributions: an
        // excess left in the IRA is Form 5329 line 15, not basis.
        let permitted = dollars(phase::cap(traditional, [room]));
        if v.nondeductible_election == v.traditional && absorbed == 0 {
            result.nondeductible[i] = permitted;
            continue;
        }
        let owner = if i == 0 { Owner::Taxpayer } else { Owner::Spouse };
        let limit = deduction_band(f, s, owner).map_or(cap, |(start, width)| {
            phase_limit(cap, traditional_magi_cents, start * 100, width * 100)
        });
        let deduction = phase::cap(
            phase::cap(dollars(traditional) - dollars(v.nondeductible_election), [limit]),
            [dollars(available)],
        );
        result.deduction += deduction;
        result.deduction_by_owner[i] = deduction;
        // Pub590-A Worksheet1-2 lines5–7: compare actual contribution after
        // the user's nondeductible election with compensation and the limit.
        // Subtract source cents before rounding a work amount; don't subtract
        // two independently rounded filing inputs inside the health method.
        result.work_deduction_cents += phase::cap(
            phase::cap(traditional - v.nondeductible_election, [limit * 100]),
            [available],
        );
        result.nondeductible[i] = permitted - deduction;
    }
    Ok(result)
}

// Each owner's 2026 Roth IRA contribution limit in cents (Form 5329 line 19
// room and line 23 excess use it); zero for an owner without Roth facts.
pub(crate) fn validate_roth(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: [i64; 2],
    magi: i64,
    transfers: &QtpTransfers,
) -> Result<[i64; 2], (&'static str, &'static str)> {
    let joint = s.filing_status() == FederalStatus::Joint;
    // The engine still checks Roth coverage after reporting an invalid traditional worksheet.
    // Missing spouse facts must remain an ordinary coverage error, never an unwrap panic.
    if joint != f.spouse.is_some() || joint != p.spouse.is_some() {
        return Err((
            "ISSUE_IRA_SPOUSE_COVERAGE",
            crate::fixture_english!(
                "Complete both IRA coverage records for a joint return; remove spouse IRA facts for any other filing status."
            ),
        ));
    }
    let (start, width) = roth_band(s);
    let facts = [Some(&f.taxpayer), f.spouse.as_ref()];
    let people = [Some(&p.taxpayer), p.spouse.as_ref()];
    let mut limits = [0; 2];
    for i in 0..2 {
        let (Some(v), Some(person)) = (facts[i], people[i]) else {
            continue;
        };
        let available = if joint && compensation[i] < compensation[1 - i] {
            let other = facts[1 - i].unwrap();
            (compensation.iter().sum::<i64>() * 100
                - other.traditional
                - other.roth
                - qtp_rollover_amount(transfers.owner(1 - i)))
            .max(0)
        } else {
            compensation[i].max(0) * 100
        };
        let cap = (contribution_limit(person)).min(dollars(available));
        // Worksheet 2-2 compares the phased total limit with the unphased limit less
        // traditional contributions. Do not subtract traditional contributions twice.
        limits[i] =
            (phase_limit(cap, magi, start, width) * 100).min((cap * 100 - v.traditional).max(0));
    }
    Ok(limits)
}

/// Excess IRA contributions and year-end values
/// The App computes the 2026 traditional and Roth IRA limits and any excess itself. These facts complete Form 5329 Parts III and IV: the 6% tax on an uncorrected excess is capped by the account values at year end, and a 2025 excess carried in is reduced by 2026 withdrawals.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraExcess {
    /// Value of all your traditional IRAs on December 31, 2026
    /// Include SEP and SIMPLE IRAs and any 2026 contributions made in 2027. Use the custodians' year-end statements.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    /// [NullText]
    /// Use the Form 8606 values
    pub traditional_value: Option<i64>,
    /// Value of all your Roth IRAs on December 31, 2026
    /// Include any 2026 contributions made in 2027.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub roth_value: i64,
    /// Traditional IRA excess carried from line 16 of your 2025 Form 5329
    /// Enter it only when line 17 of that form showed tax; otherwise leave zero.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_traditional: i64,
    /// Roth IRA excess carried from line 24 of your 2025 Form 5329
    /// Enter it only when line 25 of that form showed tax; otherwise leave zero.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_roth: i64,
    /// Part of your 2026 withdrawals that returned an excess from a year before 2026
    /// Form 5329 line 12: a traditional IRA excess contribution for 2025 or earlier paid back to you in 2026 after that year's return due date, when you took no deduction for it, none was allowable, and that year's total contributions were within its limit. It is not taxable and is not an IRA withdrawal on Form 8606. Leave 0 otherwise; a withdrawal that fails a condition is ordinary income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub returned_prior_excess: i64,
}

// The IRA contributions page's excess record (question C18).
// The traditional value has one home: an owner with a Form 8606 on the
// Retirement page leaves it empty and the stored record takes the Form 8606
// year-end value (kept in step whenever the return changes).
/// Excess contributions or a carried 2025 excess
/// The App computes the 2026 traditional and Roth IRA limits and any excess itself. These facts complete Form 5329 Parts III and IV: the 6% tax on an uncorrected excess is capped by the account values at year end, and a 2025 excess carried in is reduced by 2026 withdrawals.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IraExcessEntry {
    /// Value of all your traditional IRAs on December 31, 2026
    /// Include SEP and SIMPLE IRAs and any 2026 contributions made in 2027. Leave it empty when you entered a Form 8606 on the Retirement page; its year-end value, shown above, plus your 2026 traditional contributions paid in 2027 is used.
    /// [NullText]
    /// Use the Retirement page's Form 8606 year-end value
    /// [SetText]
    /// The value of all my traditional IRAs was
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub traditional_value: Option<i64>,
    /// Value of all your Roth IRAs on December 31, 2026
    /// Include any 2026 contributions made in 2027.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub roth_value: i64,
    /// Traditional IRA excess carried from line 16 of your 2025 Form 5329
    /// Enter it only when line 17 of that form showed tax; otherwise leave zero.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_traditional: i64,
    /// Roth IRA excess carried from line 24 of your 2025 Form 5329
    /// Enter it only when line 25 of that form showed tax; otherwise leave zero.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_roth: i64,
    /// Part of your 2026 withdrawals that returned an excess from a year before 2026
    /// Form 5329 line 12: a traditional IRA excess contribution for 2025 or earlier paid back to you in 2026 after that year's return due date, when you took no deduction for it, none was allowable, and that year's total contributions were within its limit. It is not taxable and is not an IRA withdrawal on Form 8606. Leave 0 otherwise; a withdrawal that fails a condition is ordinary income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub returned_prior_excess: i64,
}

impl IraExcessEntry {
    // The stored record, given the owner's Form 8606 year-end value when one
    // is on the return; None when the traditional value is missing or given
    // twice (the rules IRA_EXCESS_TRADITIONAL_VALUE_NEEDED and
    // IRA_EXCESS_TRADITIONAL_ON_8606).
    pub(crate) fn record(&self, form_8606: Option<Exact>) -> Option<IraExcess> {
        let traditional_value = match (self.traditional_value, form_8606) {
            (Some(value), None) => Some(value),
            (None, Some(value)) => exact::rounded(&value),
            _ => return None,
        };
        Some(IraExcess {
            traditional_value,
            roth_value: self.roth_value,
            prior_traditional: self.prior_traditional,
            prior_roth: self.prior_roth,
            returned_prior_excess: self.returned_prior_excess,
        })
    }
    // The page's entry for a stored record.
    pub(crate) fn from_record(record: &IraExcess, form_8606: Option<i64>) -> Option<Self> {
        Some(Self {
            traditional_value: match form_8606 {
                Some(_) => None,
                None => Some(record.traditional_value?),
            },
            roth_value: record.roth_value,
            prior_traditional: record.prior_traditional,
            prior_roth: record.prior_roth,
            returned_prior_excess: record.returned_prior_excess,
        })
    }
}

impl IraExcess {
    pub(crate) fn valid(&self, form_8606: Option<Exact>) -> bool {
        // A wide known duplicate projection is None only when the original
        // Form8606 operands establish it. Without that source, a value is required.
        // Form8606 remains authoritative over the readonly duplicate, including
        // an older representable duplicate that has not yet followed an edit.
        let consistent = form_8606.is_some() || self.traditional_value.is_some();
        consistent
            && self.traditional_value.is_none_or(|value| value >= 0)
            && [
                self.roth_value,
                self.prior_traditional,
                self.prior_roth,
                self.returned_prior_excess,
            ]
            .iter()
            .all(|v| (0..=100_000_000_000).contains(v))
            && self.returned_prior_excess <= self.prior_traditional
    }
}

// Form 5329 Parts III and IV for each owner: lines 9-17 (traditional) and
// 18-25 (Roth). `roth_limits` come from validate_roth; `ira_taxable` is each
// owner's 2026 traditional IRA distributions included in income (line 11)
// and `roth_withdrawn` each owner's gross Roth IRA withdrawals (line 20),
// both in whole dollars. Returns each owner's final6% tax in whole dollars;
// the ordinary return sums these same native PartsIII/IV amounts.
#[allow(clippy::too_many_arguments)]
pub(crate) fn excess_tax_with_difficulty_of_care(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: [i64; 2],
    difficulty_of_care: [i64; 2],
    roth_limits: [i64; 2],
    ira_taxable: [i64; 2],
    roth_withdrawn: [i64; 2],
    traditional_values: [Option<Exact>; 2],
    transfers: &QtpTransfers,
    rollover_excess: [i64; 2],
    r: &mut FederalResult,
) -> [i64; 2] {
    use crate::kernel::ratio::{fraction, positive};
    let joint = s.filing_status() == FederalStatus::Joint;
    if joint != f.spouse.is_some() || joint != p.spouse.is_some() {
        return [0; 2];
    }
    let facts = [Some(&f.taxpayer), f.spouse.as_ref()];
    let people = [Some(&p.taxpayer), p.spouse.as_ref()];
    let mut tax = [0; 2];
    for i in 0..2 {
        let (Some(v), Some(person)) = (facts[i], people[i]) else {
            continue;
        };
        let other = facts[1 - i];
        let available = if joint && compensation[i] < compensation[1 - i] {
            let o = other.unwrap();
            phase::floor(
                compensation.iter().sum::<i64>() * 100
                    - o.traditional
                    - o.roth
                    - qtp_rollover_amount(transfers.owner(1 - i)),
                0,
            )
        } else {
            phase::floor(compensation[i], 0) * 100
        };
        let traditional_limit =
            phase::cap(contribution_limit(person) * 100, [available + difficulty_of_care[i]]);
        // The excess is decided in cents; the form's lines are whole dollars.
        let traditional_over = positive(v.traditional - traditional_limit);
        let roth_over = positive(v.roth - roth_limits[i]);
        let line15 = dollars(traditional_over);
        // A 529-to-Roth transfer above the year's room is not a qualified rollover: it is a
        // Roth contribution above the limit (section 529(c)(3)(E)).
        let line23 = dollars(roth_over + rollover_excess[i]);
        let who = if i == 0 { "taxpayer" } else { "spouse" };
        let Some(excess) = v.excess.as_ref() else {
            let (key, _explanation) = if traditional_over > 0 && rollover_excess[i] == 0 {
                (
                    "ISSUE_IRA_TRADITIONAL_EXCESS_NEEDS_YEAR_END_VALUES",
                    crate::fixture_english!(
                        "Form 5329 Part III line 15: 2026 traditional IRA contributions exceed this filer's age-based or compensation limit. Enter the December 31, 2026 IRA values under More about these contributions on the IRA contributions page so the 6% tax is calculated, or correct the excess."
                    ),
                )
            } else {
                (
                    "ISSUE_IRA_ROTH_EXCESS_NEEDS_YEAR_END_VALUES",
                    crate::fixture_english!(
                        "Form 5329 Part IV line 23: 2026 Roth IRA contributions exceed this filer's income-based, age-based or compensation limit. Enter the December 31, 2026 IRA values under More about these contributions on the IRA contributions page so the 6% tax is calculated, or correct or recharacterize the excess."
                    ),
                )
            };
            #[cfg(test)]
            let explanation = _explanation;

            crate::require_localized!(
                r,
                traditional_over == 0 && roth_over == 0 && rollover_excess[i] == 0,
                &crate::IssueContext::Section(crate::TaxSection::IraContributions),
                "IRA contributions",
                key,
                explanation,
            );
            continue;
        };
        // Part III.
        let line9 = dollars(excess.prior_traditional);
        let line10 = if line9 > 0 {
            dollars(positive(traditional_limit - v.traditional - v.roth))
        } else {
            0
        };
        let line11 = if line9 > 0 { ira_taxable[i] } else { 0 };
        // Line 12: the part of 2026 withdrawals that returned an excess from before 2026
        // (section 408(d)(5)); the IRA page checks it against line 9.
        let line12 =
            if line9 > 0 { phase::cap(dollars(excess.returned_prior_excess), [line9]) } else { 0 };
        let line13 = line10 + line11 + line12;
        let line14 =
            crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line9, line13)
                .get();
        let line16 = line14 + line15;
        let traditional_value =
            traditional_values[i].clone().or_else(|| excess.traditional_value.map(exact::whole));
        let Some(traditional_value) = traditional_value else {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::IraContributions),
                "IRA contributions",
                "IRA_EXCESS_TRADITIONAL_VALUE_NEEDED",
                "Enter the actual traditional IRA year-end value or its Form 8606 source.",
            );
            continue;
        };
        // For nonnegative source Cents and an integer-Dollar line16 limit,
        // capping and the existing category rounding commute. Cap the exact
        // source first so its unused duplicate cannot narrow the legal amount.
        let taxable_excess =
            exact::dollars(&traditional_value.min(exact::whole(line16) * exact::whole(100)))
                .expect("excess tax base bounded by native line16");
        let line17 = fraction(taxable_excess, 6, 100);
        // Part IV.
        let line18 = dollars(excess.prior_roth);
        let line19 = if line18 > 0 { dollars(positive(roth_limits[i] - v.roth)) } else { 0 };
        let line20 = if line18 > 0 {
            // Withdrawing every Roth IRA balance counts at least the carried excess.
            if roth_withdrawn[i] > 0 && excess.roth_value == 0 {
                phase::floor(roth_withdrawn[i], line18)
            } else {
                roth_withdrawn[i]
            }
        } else {
            0
        };
        let line21 = line19 + line20;
        let line22 =
            crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line18, line21)
                .get();
        let line24 = line22 + line23;
        let line25 = fraction(phase::cap(line24, [dollars(excess.roth_value)]), 6, 100);
        tax[i] += line17 + line25;
        // Prior carried amounts are entered only when the prior Form5329
        // showed tax. Either that source or a current excess requires this
        // person's part, even when absorption leaves no current additional tax.
        let traditional_required = excess.prior_traditional > 0 || traditional_over > 0;
        let roth_required = excess.prior_roth > 0 || roth_over + rollover_excess[i] > 0;
        emit_lines(
            r,
            &mut (),
            [
                (9, "line9_prior_traditional_excess", line9),
                (10, "line10_unused_traditional_room", line10),
                (11, "line11_traditional_distributions", line11),
                (12, "line12_returned_prior_excess", line12),
                (13, "line13_traditional_total_reduction", line13),
                (14, "line14_prior_traditional_remaining", line14),
                (15, "line15_traditional_excess", line15),
                (16, "line16_traditional_total_excess", line16),
                (17, "line17_traditional_excess_tax", line17),
                (18, "line18_prior_roth_excess", line18),
                (19, "line19_unused_roth_room", line19),
                (20, "line20_roth_distributions", line20),
                (21, "line21_roth_total_reduction", line21),
                (22, "line22_prior_roth_remaining", line22),
                (23, "line23_roth_excess", line23),
                (24, "line24_roth_total_excess", line24),
                (25, "line25_roth_excess_tax", line25),
            ],
            |(number, suffix, amount), _, _| {
                let required = if number < 18 { traditional_required } else { roth_required };
                Some(calculation_row(
                    required.then(|| FormLine::new(TaxForm::Form5329, &alloc::format!("{number}"))),
                    &alloc::format!("ira_excess_{who}_{suffix}"),
                    LineValue::MoneyDollars(amount),
                    &["adjustments.iras", "income.retirement"],
                    "https://www.irs.gov/pub/irs-dft/i5329--dft.pdf",
                    msg!("EXPLAIN_IRA_EXCESS_N_N"),
                ))
            },
        );
    }
    tax
}

// One owner for deduction phaseout bands, shared by calculation and candidate search.
fn deduction_band(f: &FederalIras, s: &FederalSetup, owner: Owner) -> Option<(i64, i64)> {
    let (v, other) = if owner == Owner::Taxpayer {
        (Some(&f.taxpayer), f.spouse.as_ref())
    } else {
        (f.spouse.as_ref(), Some(&f.taxpayer))
    };
    let v = v?;
    if v.nondeductible_election == v.traditional {
        return None;
    }
    let covered = v.covered_at_work;
    let spouse_covered = (s.filing_status() == FederalStatus::Joint
        && other.is_some_and(|v| v.covered_at_work))
        || matches!(f.spouse, IraSpouse::SeparateTogether { covered_at_work: true });
    if matches!(f.spouse, IraSpouse::SeparateTogether { .. }) && (covered || spouse_covered) {
        Some((0, 10000))
    } else if covered {
        Some(if matches!(s.filing_status(), FederalStatus::Joint | FederalStatus::Surviving) {
            (
                crate::year_constants::IRA_PHASEOUT_START_COVERED_JOINT_DOLLARS,
                crate::year_constants::IRA_PHASEOUT_RANGE_JOINT_DOLLARS,
            )
        } else {
            (
                crate::year_constants::IRA_PHASEOUT_START_COVERED_SINGLE_DOLLARS,
                crate::year_constants::IRA_PHASEOUT_RANGE_SINGLE_DOLLARS,
            )
        })
    } else if spouse_covered {
        Some((242000, 10000))
    } else {
        None
    }
}

pub(crate) fn income_limited(f: &FederalIras, s: &FederalSetup, owner: Owner) -> bool {
    deduction_band(f, s, owner).is_some()
}

// Both owners share one MAGI. Visit every integer-income boundary where either
// rounded deduction can change, rather than taking a Cartesian product of limits.
// Below zero and above the last boundary the deduction pair is constant.
pub(crate) fn deduction_candidates(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: [i64; 2],
    transfers: &QtpTransfers,
    distributions: [i64; 2],
) -> alloc::vec::Vec<IraResult> {
    let mut points = alloc::collections::BTreeSet::from([0]);
    for (owner, fact, person) in [
        (Owner::Taxpayer, Some(&f.taxpayer), Some(&p.taxpayer)),
        (Owner::Spouse, f.spouse.as_ref(), p.spouse.as_ref()),
    ] {
        if let (Some(fact), Some(person), Some((start, width))) =
            (fact, person, deduction_band(f, s, owner))
            && (fact.traditional > 0
                || fact.excess.as_ref().is_some_and(|e| e.prior_traditional > 0))
        {
            let cap = contribution_limit(person);
            points.insert(start + width);
            for limit in (200..=cap).step_by(10) {
                points.insert(start + width - crate::kernel::ratio::trunc_i64(limit * width, cap));
            }
        }
    }
    let mut candidates = alloc::vec::Vec::new();
    let mut previous = None;
    for magi in points {
        if let Ok(candidate) = calculate(f, p, s, compensation, magi, transfers, distributions)
            && previous != Some(candidate.nondeductible)
        {
            previous = Some(candidate.nondeductible);
            candidates.push(candidate);
        }
    }
    candidates
}

pub(crate) fn roth_band(s: &FederalSetup) -> (i64, i64) {
    if matches!(s.filing_status(), FederalStatus::Joint | FederalStatus::Surviving) {
        (242000, 10000)
    } else if s.filing_status() == FederalStatus::Separate && !s.lived_apart_all_year() {
        (0, 10000)
    } else {
        (153000, 15000)
    }
}

// Pub. 590-A General Limit and Full Deduction reduce the traditional IRA
// capacity by employee-funded trust contributions. Do not infer the ordering
// of this reduction inside phased deductions or Roth limits from the generic
// worksheet, which does not explicitly describe that combination.
pub(crate) fn validate_pension_contributions(
    f: &FederalIras,
    p: &FederalPeople,
    s: &FederalSetup,
    compensation: [i64; 2],
    magi_cents: i64,
    pension: [i64; 2],
    transfers: &QtpTransfers,
) -> Result<(), (&'static str, &'static str)> {
    let joint = s.filing_status() == FederalStatus::Joint;
    if joint != f.spouse.is_some() || joint != p.spouse.is_some() {
        return Err((
            "ISSUE_PENSION_IRA_SPOUSE_COVERAGE",
            crate::fixture_english!("Complete both IRA coverage records for a joint return."),
        ));
    }
    let facts = [Some(&f.taxpayer), f.spouse.as_ref()];
    let people = [Some(&p.taxpayer), p.spouse.as_ref()];
    for i in 0..2 {
        let (Some(v), Some(person)) = (facts[i], people[i]) else {
            continue;
        };
        if pension[i] == 0 {
            continue;
        }
        if v.roth > 0 || !transfers.owner(i).is_empty() {
            return Err((
                "ISSUE_PENSION_ROTH_COORDINATION",
                crate::fixture_english!(
                    "Section 501(c)(18)(D) contributions with this employee's Roth IRA contributions or 529-to-Roth rollover require further verified contribution-limit coordination."
                ),
            ));
        }
        let available =
            if s.filing_status() == FederalStatus::Joint && compensation[i] < compensation[1 - i] {
                let other = facts[1 - i].unwrap();
                (compensation.iter().sum::<i64>() * 100 - other.traditional - other.roth).max(0)
            } else {
                compensation[i].max(0) * 100
            };
        let limit = ((contribution_limit(person) * 100).min(available) - pension[i]).max(0);
        if v.traditional > limit {
            return Err((
                "ISSUE_PENSION_TRADITIONAL_IRA_LIMIT",
                crate::fixture_english!(
                    "Traditional IRA contributions exceed the remaining annual or compensation limit after section 501(c)(18)(D) contributions; resolve the excess before calculating this return."
                ),
            ));
        }
        let owner = if i == 0 { Owner::Taxpayer } else { Owner::Spouse };
        if v.traditional > v.nondeductible_election
            && deduction_band(f, s, owner).is_some_and(|(start, width)| {
                magi_cents > start * 100 && magi_cents < (start + width) * 100
            })
        {
            return Err((
                "ISSUE_PENSION_IRA_PHASEOUT_COORDINATION",
                crate::fixture_english!(
                    "Section 501(c)(18)(D) contributions combined with a partially phased-out traditional IRA deduction need verified limit-ordering coordination."
                ),
            ));
        }
    }
    Ok(())
}
