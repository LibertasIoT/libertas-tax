pub use crate::nol_model::{NolHistory, NolYear, PtpEblNol};
// Coverage: Form 172 carryforward / Schedule 1 line 8a — Supported for business histories
//   (u5-losses, 2026-09-28). Asked: the kind "Net operating loss from an earlier year"
//   (IncomeRecord::NetOperatingLoss, LossCarryforward) on the Self-employment and Farming
//   topics, onto FederalScreening.nol (regular years) and .atnol (Form 6251 line 2f years);
//   owned by those topics' ticks (walk.rs). LossSource::Business writes both confirmations Yes;
//   SomethingElse writes No and is refused by name ("Regular loss history", Form 172; Schedule
//   1 line 8a / "AMT loss history", Form 6251 line 2f). Calculated: pre-2018 losses in full,
//   post-2017 losses to 80% of taxable income without the NOL and QBI deductions less the
//   pre-2018 losses (i172 2026 draft, "NOL deduction limitation"); absorption oldest first;
//   2006 losses expire after 2026, earlier ones already expired. Rules on both sides:
//   NOL_NOTHING_ENTERED, NOL_YEAR_TWICE, NOL_TWICE, ATNOL_EXCEPTION_AFTER_2017.
//   RareSituation::NetOperatingLoss removed (STORAGE_FORMAT 73). Tests: src/tests/federal/losses.rs
//   operating_loss_kind_matches_hand_computed_form_172_and_6251 and the boundary tests
//   (80% limit, pre-2018, 2005/2006 expiry, 90% AMT limit, rules, refusal, Farming share).
//   PTP §461(l) source character: native original/qualified and surviving
//   pairs persist on NolYear; NoPtpEbl is an explicit actual absence, while
//   Unavailable never inherits qualification from Business/confirmed. Final
//   allocation-only character projection retains original qualification,
//   known mixed-cohort footing, actual deduction versus absorption, pre-debt
//   closing/lost qualification and independent injured-qualified ownership.
//   Source attribution emits native financial cents once to the existing
//   qualified-vintage owner; that owner emits native qualified cents. No
//   rational API leaves either owner. There are no new source-amount/count caps. Accepted opening
//   facts persist normally; no filed-return or automatic new-year route exists.
//   Tests: tests/federal/losses.rs ptp_ebl_nol_*; Reg1.199A-3(b)(1)(iv),(v),
//   (c)(3)(ii), i8995 Tracking Losses; i172 financial formulas unchanged.
//   Sources: i172 (Dec 2024) and i172 draft, f172 draft, fetched 2026-09-28.
use crate::FederalResult;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::positive;
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{line, line_cents};
use alloc::{collections::BTreeMap, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Net operating loss from an earlier year
/// Business losses from earlier years still unused at the start of 2026, from your 2025 Form 172 or carryover worksheet. Enter it once for the return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct LossCarryforward {
    /// Regular tax loss by year
    /// One row per loss year with a balance left, after every carryback and every return since. Enter what is left, not the original loss. The oldest year is used first.
    /// ----
    /// Loss year
    pub regular: Vec<crate::NolYearPage>,
    /// Alternative minimum tax loss by year
    /// Only if a return since the loss year figured the alternative minimum tax loss (Form 6251 line 2f). Leave it empty otherwise.
    /// ----
    /// Loss year
    pub amt: Vec<crate::AtnolYear>,
    /// Where the losses came from
    #[libertas_default(Business)]
    pub source: LossSource,
}

/// Where the losses came from
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum LossSource {
    /// Only from running U.S. businesses
    /// Schedule C or F businesses, partnerships and S corporations you actively ran, or separately identified PTP excess-business-loss amounts, with the same filer (or the same married couple filing jointly) every year since. Every carryback and earlier return is settled.
    Business,
    /// Something else applies
    /// Part of a loss came from investments, trading securities or an ordinary-business excess business loss, a farm loss is being carried back, the loss was foreign, inherited or from a trust, you married, divorced or changed filing status since, or an earlier return is being amended. Not calculated yet.
    SomethingElse,
}

impl LossCarryforward {
    // The kind onto the two stored histories (Form 172 and Form 6251 line
    // 2f): an empty list is no history; the source is each history's
    // confirmation.
    pub(crate) fn histories(&self) -> (NolHistory, crate::AtnolHistory) {
        let confirmed = self.source == LossSource::Business;
        let regular = if self.regular.is_empty() {
            NolHistory::None
        } else {
            NolHistory::Records {
                years: self.regular.iter().cloned().map(Into::into).collect(),
                confirmed,
            }
        };
        let amt = if self.amt.is_empty() {
            crate::AtnolHistory::None
        } else {
            crate::AtnolHistory::Records { years: self.amt.clone(), confirmed }
        };
        (regular, amt)
    }
    // The stored histories as the kind shows them; None when nothing is
    // stored, Some(None) when the two confirmations are not one page answer.
    pub(crate) fn from_histories(
        regular: &NolHistory,
        amt: &crate::AtnolHistory,
    ) -> Option<Option<Self>> {
        let (regular, first) = match regular {
            NolHistory::None => (Vec::new(), None),
            NolHistory::Records { years, confirmed } => {
                (years.iter().cloned().map(Into::into).collect(), Some(*confirmed))
            }
        };
        let (amt, second) = match amt {
            crate::AtnolHistory::None => (Vec::new(), None),
            crate::AtnolHistory::Records { years, confirmed } => (years.clone(), Some(*confirmed)),
        };
        let confirmed = match (first, second) {
            (None, None) => return None,
            (Some(a), Some(b)) if a != b => return Some(None),
            (Some(a), _) | (_, Some(a)) => a,
        };
        let source = if confirmed { LossSource::Business } else { LossSource::SomethingElse };
        Some(Some(Self { regular, amt, source }))
    }
    // The server twins of the kind's client rules, then the stored ranges.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some((i, _)) = self
            .regular
            .iter()
            .enumerate()
            .find(|(_, y)| y.injured_owners.as_ref().is_some_and(|v| !v.valid(y.balance)))
        {
            return Some(
                ValidationIssue::new("INJURED_NOL_OWNER_TOTAL")
                    .at_field("injured_owners")
                    .in_list("regular", i),
            );
        }
        if self.regular.is_empty() && self.amt.is_empty() {
            return Some(ValidationIssue::new("NOL_NOTHING_ENTERED").at_field("regular"));
        }
        if let Some((i, _)) = self
            .regular
            .iter()
            .enumerate()
            .find(|(i, y)| self.regular[..*i].iter().any(|old| old.year == y.year))
        {
            return Some(
                ValidationIssue::new("NOL_YEAR_TWICE").at_field("year").in_list("regular", i),
            );
        }
        if let Some((i, _)) = self
            .amt
            .iter()
            .enumerate()
            .find(|(i, y)| self.amt[..*i].iter().any(|old| old.year == y.year))
        {
            return Some(ValidationIssue::new("NOL_YEAR_TWICE").at_field("year").in_list("amt", i));
        }
        if let Some(issue) = self
            .amt
            .iter()
            .enumerate()
            .find_map(|(i, y)| y.issue().map(|issue| issue.in_list("amt", i)))
        {
            return Some(issue);
        }
        for (i, year) in self.regular.iter().enumerate() {
            let stored: NolYear = year.clone().into();
            if let Some(issue) =
                stored.ptp_ebl.issue(stored.year, stored.balance, stored.injured_owners.as_ref())
            {
                return Some(issue.in_field("ptp_ebl").in_list("regular", i));
            }
            if let Some(issue) = crate::retained_research::loss_issue(&stored, None) {
                return Some(issue.in_list("regular", i));
            }
        }
        None
    }
}

#[derive(Default)]
pub(crate) struct NolRun {
    // Shared ordered-return projection for IRC 613A(d)(1), before oil/gas
    // depletion and section 199A but after this year's NOL deduction (only
    // carrybacks are disregarded). Recursive IRA/adoption
    // worksheet passes retain this context; the actual return reuses its limits.
    pub royalty_projection: bool,
    pub royalty_limits: Option<[i64; 2]>,
    pub deduction: i64,
    pub modify_income: bool,
    pub exclude_qbi: bool,
    pub charity_limit: Option<i64>,
    pub charity_deduction: i64,
    pub charity_carry: i64,
    pub charity_nol_reduction: i64,
    pub charity_class_deductions: [i64; 8],
    pub charity_class_carry: [i64; 8],
    pub charity_class_reduction: [i64; 8],
    // Calculation-only attribution of the existing section170(d)(1)(B)
    // endpoint: progressively permit each class's hypothetical excess.
    pub charity_class_ceiling: Option<([i64; 8], u8)>,
    // Unfloored taxable income before NOL and QBI; the carryover pass also
    // excludes section 151 senior deductions. Never read presentation lines
    // back into the calculation. Each vintage reuses the ordered income
    // worksheets with only earlier NOL deductions when measuring absorption.
    pub taxable_base: Option<i64>,
    // Form 172 Part II adds section 1202 exclusions for absorption, not for
    // the current deduction limit or the actual return's taxable income.
    pub qsbs_excluded: i64,
    // Part II line 7 adds the lost medical deduction when line 4 increases
    // modified AGI. This affects prior-loss absorption, not current tax.
    pub section1202_medical_adjustment: i64,
    // A higher Form 172 modified AGI can phase down the 2026 SALT cap. Add
    // the resulting lost itemized deduction to prior-loss absorption only.
    pub section1202_salt_adjustment: i64,
    // Form 172 Part II line 32 restores the ordinary personal-casualty
    // deduction lost when section 1202 gain raises the 10%-of-AGI floor.
    pub section1202_casualty_adjustment: i64,
    // Part II line 27 restores the supported cash-charity deduction lost
    // when the 2026 0.5%-of-AGI floor rises in modified income.
    pub section1202_charity_adjustment: i64,
    // Section 68 is recomputed after the modified itemized deductions and
    // section 1202 gain are included in Form 172's hypothetical income.
    pub section1202_section68_adjustment: i64,
    // Part II line 6 adds the student-loan deduction lost when line 4 raises
    // modified AGI; each prior-loss vintage gets its own calculation.
    pub section1202_loan_adjustment: i64,
    // Part II line 6 restores the traditional IRA deduction lost when
    // section 1202 gain raises Appendix B's modified income.
    pub section1202_ira_adjustment: i64,
    // Part II line 6 also adds newly taxable Social Security benefits after
    // the section 1202 gain enters provisional income.
    pub section1202_social_adjustment: i64,
    pub allocation: Option<alloc::boxed::Box<NolAllocation>>,
}
impl NolRun {
    pub(crate) fn limit_charity(&self, worksheet: &crate::charity::Worksheet) -> i64 {
        self.charity_limit.map_or(worksheet.deduction(), |actual| {
            let excess = self.charity_class_ceiling.map_or(0, |(classes, mask)| {
                worksheet
                    .classes
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(i, c)| positive(c.current_allowed - c.current_floor - classes[i]))
                    .sum::<i64>()
            });
            phase::cap(actual + excess, [worksheet.deduction()])
        })
    }
}
// Runtime original-period use, retained until section108 attribute reduction.
// Display summaries may aggregate by calendar year only after this allocation.
pub(crate) struct LossUse {
    pub year: u32,
    pub opening: i64,
    pub used: i64,
    pub expired: i64,
    pub closing: i64,
    pub expires_now: bool,
}

#[derive(Default)]
pub(crate) struct NolAllocation {
    pub deduction: i64,
    pub carryforward: Exact,
    years: Vec<LossUse>,
    injured_vintages: Vec<(crate::estate_trust::LossVintage, i64)>,
    // Exact deduction, distinct from the owner's carry absorption.
    deductions: Vec<(Option<i64>, u32, i64)>,
    estate_source: Option<i64>,
    has_estate: bool,
}
pub(crate) struct InjuredNolPart {
    pub key: alloc::string::String,
    pub deduction: i64,
    pub balances: Option<[Exact; 2]>,
}

// Native failures belong to the actual loss output, never to record count.
#[derive(Clone, Debug)]
pub(crate) struct LossAmountIssue {
    pub source: Option<i64>,
    pub year: Option<u32>,
    pub alternative: bool,
}
impl LossAmountIssue {
    pub(crate) fn new(source: Option<i64>, year: Option<u32>, alternative: bool) -> Self {
        Self { source, year, alternative }
    }
    pub(crate) fn report(&self, result: &mut FederalResult) {
        let context = self.source.map_or(
            crate::IssueContext::Section(crate::TaxSection::SelfEmployment),
            crate::IssueContext::income_id,
        );
        crate::federal_rules::issue_message(
            result,
            &context,
            #[cfg(test)]
            "Retained operating loss",
            #[cfg(test)]
            "The retained loss output is outside its native amount representation.",
            crate::Message::new(
                if self.alternative {
                    "ATNOL_AMOUNT_UNAVAILABLE"
                } else {
                    "NOL_AMOUNT_UNAVAILABLE"
                },
                alloc::vec![crate::MessageArg::Text(self.year.map_or_else(
                    || "the combined retained losses".into(),
                    |year| alloc::format!("retained loss year {year}"),
                ))],
            ),
        );
        crate::federal_rules::withhold_required_source_result(result);
    }
}

impl NolHistory {
    pub(crate) fn allocate(
        taxable: i64,
        vintages: &[crate::estate_trust::LossVintage],
        mut modified_income: impl FnMut(i64) -> i64,
    ) -> Result<alloc::boxed::Box<NolAllocation>, LossAmountIssue> {
        if vintages.is_empty() {
            return Ok(alloc::boxed::Box::default());
        }
        // The source projection orders actual original tax periods and pools
        // only equal character/expiry cohorts. The existing owner formula is
        // unchanged; a short fiduciary year is not encoded as a fake year.
        // The 52/53-week effective-date class is retained separately from
        // the actual original year. Personal calendar-year classes are unchanged.
        let issue = || LossAmountIssue::new(None, None, false);
        let ordered = vintages
            .iter()
            .map(|y| {
                exact::dollars(&y.balance)
                    .map(|opening| (y, opening))
                    .ok_or_else(|| LossAmountIssue::new(y.origin, Some(y.year), false))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let old: Exact = ordered
            .iter()
            .filter(|(y, _)| !y.post_2017 && y.eligible())
            .map(|(_, opening)| exact::whole(*opening))
            .sum();
        let newer: Exact = ordered
            .iter()
            .filter(|(y, _)| y.post_2017)
            .map(|(_, opening)| exact::whole(*opening))
            .sum();
        let income_after_old = phase::floor(exact::whole(taxable) - &old, exact::whole(0));
        let modern_limit = exact::rounded_exact(&(&income_after_old * exact::ratio(4, 5).unwrap()));
        let modern_deduction =
            exact::rounded(&phase::cap(newer, [modern_limit.clone()])).ok_or_else(issue)?;
        // Preserve the existing whole-dollar cohort boundary and 80% limit.
        // This actual required deduction is ultimately a native MoneyCents row.
        let deduction = exact::rounded(&(old + exact::whole(modern_deduction)))
            .filter(|amount| amount.checked_mul(100).is_some())
            .ok_or_else(issue)?;
        let reserved_income =
            exact::rounded(&(income_after_old - modern_limit)).ok_or_else(issue)?;
        let mut earlier_deductions = 0;
        let mut modern_remaining = modern_deduction;
        let mut result = alloc::boxed::Box::new(NolAllocation { deduction, ..Default::default() });
        for (y, opening) in ordered {
            let deducted = if !y.eligible() {
                0
            } else if !y.post_2017 {
                opening
            } else {
                phase::cap(opening, [modern_remaining])
            };
            // Section 172(b)(2): refigure modified taxable income ignoring
            // this vintage and every later vintage, then retain the 20%
            // amount specified by (a)(2)(B)(ii). IRA and student-loan
            // deductions can change as earlier NOLs reduce their MAGI.
            let used = if !y.eligible() {
                0
            } else {
                exact::rounded(&phase::cap(
                    exact::whole(opening),
                    [phase::floor(
                        exact::whole(modified_income(earlier_deductions))
                            - exact::whole(reserved_income),
                        exact::whole(0),
                    )],
                ))
                .expect("absorption is bounded by the native cohort opening")
            };
            result.deductions.push((y.origin, y.year, deducted));
            earlier_deductions += deducted;
            if y.post_2017 {
                modern_remaining = crate::kernel::carry::native_balance(modern_remaining, deducted);
            }
            let settlement = crate::kernel::carry::settle_values::<crate::kernel::carry::Dollars>(
                opening,
                used,
                y.expires(),
            )
            .expect("validated loss opening and absorption");
            let used = settlement.used.get();
            let expired = settlement.expired.get();
            let closing = settlement.closing.get();
            result.carryforward += exact::whole(closing);
            result.injured_vintages.push((y.clone(), deducted));
            result.years.push(LossUse {
                year: y.year,
                opening,
                used,
                expired,
                closing,
                expires_now: y.expires_now(),
            });
        }
        Ok(result)
    }
}
// Native source cents leave the NOL owner once. A missing character or
// deduction association is distinct from a documented zero. Financial NOL
// deduction and absorption are already settled and are never recomputed here.
pub(crate) struct PtpNolDeduction {
    pub year: u32,
    pub qualified: Option<i64>,
    pub owners: Option<[i64; 2]>,
    pub closing: Option<i64>,
    pub lost: Option<i64>,
}

impl NolAllocation {
    pub(crate) fn ptp_deductions(&self, d: &crate::FederalDraft) -> Vec<PtpNolDeduction> {
        use crate::kernel::source_fact::SourceFact;
        let Some(NolHistory::Records { years, confirmed: true }) =
            d.screening.as_ref().map(|s| &s.nol)
        else {
            return Vec::new();
        };
        let sources = crate::estate_trust::loss_vintages(d, false);
        let mut result = Vec::new();
        for ((cohort, deducted), financial) in self.injured_vintages.iter().zip(&self.years) {
            let members: Vec<_> = sources
                .iter()
                .filter(|source| {
                    source.year == cohort.year
                        && source.period == cohort.period
                        && source.expiration == cohort.expiration
                        && source.special == cohort.special
                        && source.post_2017 == cohort.post_2017
                })
                .collect();
            // The pooled vintage retains the first origin, which is not
            // attribution. Resolve the actual personal member independently.
            let Some(personal) = members.iter().find(|s| s.origin.is_none()) else { continue };
            let Some(record) = years.iter().find(|r| r.year == personal.year) else { continue };
            // Reg.1.199A-3(b)(1)(iv)(A): pre-2018 losses cannot enter QBI.
            // Applicability does not rewrite the accepted source availability.
            if record.year < 2018 {
                continue;
            }
            let fact = match &record.ptp_ebl {
                PtpEblNol::Unavailable => SourceFact::Unavailable,
                PtpEblNol::NoPtpEbl => SourceFact::NotReported,
                PtpEblNol::Reported { .. } => SourceFact::Reported(&record.ptp_ebl),
            }
            .require(|| ())
            .ok();
            if matches!(fact, Some(None)) {
                continue;
            }
            let mut part = PtpNolDeduction {
                year: record.year,
                qualified: (*deducted == 0).then_some(0),
                owners: (*deducted == 0).then_some([0, 0]),
                closing: None,
                lost: None,
            };
            if let Some(Some(PtpEblNol::Reported {
                original,
                qualified,
                remaining,
                remaining_qualified,
                injured_qualified,
            })) = fact
            {
                let footing: Exact = members.iter().map(|s| s.balance.clone()).sum();
                let estate_character_known = members.iter().filter_map(|s| s.origin).all(|id| {
                    d.income.iter().any(|e| e.id == id && matches!(&e.income,
                        crate::FederalIncome::EstateTrustPortfolio { data }
                            if data.statements.final_nol.as_ref().is_some_and(|n| n.fiduciary_history_complete)))
                });
                if footing != cohort.balance
                    || personal.balance != exact::whole(record.balance)
                    || !estate_character_known
                {
                    result.push(part);
                    continue;
                }
                // NOL source attribution emits actual PTP financial cents
                // once. Its settled dollar amount does not establish that a
                // larger fractional source has been fully deducted.
                let principal = |amount: i64| {
                    exact::rounded(&phase::cap(
                        exact::whole(amount) * exact::whole(100) * exact::whole(*remaining)
                            / &footing,
                        [exact::whole(*remaining)],
                    ))
                    .expect("PTP source allowance is bounded by native remaining cents")
                };
                // The existing qualified-vintage owner consumes those cents;
                // no local copy of its original-percentage/capacity formula.
                use crate::kernel::carry::{Amount, Cents};
                use crate::kernel::loss_limits::vintages;
                let vintage = [vintages::LossVintage {
                    key: record.year,
                    original: Amount::<Cents>::new(*original).expect("validated PTP original"),
                    original_qualified: Amount::new(*qualified)
                        .expect("validated PTP qualification"),
                    remaining: Amount::new(*remaining).expect("validated PTP remaining"),
                    remaining_qualified: Amount::new(*remaining_qualified)
                        .expect("validated remaining qualification"),
                }];
                let release = |allowance| {
                    let SourceFact::Reported(released) =
                        vintages::release_vintages(SourceFact::Reported(vintages::VintageInput {
                            vintages: &vintage,
                            amount: Amount::new(allowance).ok()?,
                            year: crate::year_constants::TAX_YEAR_END_DATE / 10000,
                            order: &[0],
                        }))
                        .ok()?
                    else {
                        return None;
                    };
                    Some(released.qualified.get())
                };
                part.qualified = release(principal(*deducted));
                let consumed = if financial.closing == 0 {
                    Some(*remaining_qualified)
                } else {
                    release(principal(financial.used + financial.expired))
                };
                // A deducted qualified cent cannot also survive. Projected
                // native consumption has an exact remaining/lost residual.
                part.closing = consumed.zip(part.qualified).map(|(consumed, current)| {
                    remaining_qualified - phase::floor(consumed, current)
                });
                part.lost = part
                    .qualified
                    .zip(part.closing)
                    .map(|(used, closing)| remaining_qualified - used - closing);
                part.owners = part.qualified.and_then(|amount| {
                    if amount == 0 {
                        return Some([0, 0]);
                    }
                    if d.setup
                        .as_ref()
                        .is_some_and(|s| s.filing_status() != crate::FederalStatus::Joint)
                    {
                        return Some([amount, 0]);
                    }
                    // Surviving qualified ownership does not establish each
                    // owner's original qualified percentage or the financial
                    // PTP subset released. Exclusive qualified ownership, or
                    // actual release of the whole qualified subset, resolves
                    // it without inventing an owner ratio.
                    match injured_qualified.as_ref()?.amounts() {
                        [_, 0] => Some([amount, 0]),
                        [0, _] => Some([0, amount]),
                        owners if amount == *remaining_qualified => Some(owners),
                        _ => None,
                    }
                });
            }
            result.push(part);
        }
        result
    }
    // Retain the exact joint deduction by original-period cohort. Absorption
    // and expiration are distinct and must never become Form 8379 income.
    // Equal cohorts may pool both beneficiary and personal source records;
    // rebuild their owner balances from the same unpooled source projection.
    pub(crate) fn injured_parts(&self, d: &crate::FederalDraft) -> Vec<InjuredNolPart> {
        let sources = crate::estate_trust::loss_vintages(d, false);
        self.injured_vintages
            .iter()
            .filter(|(_, deducted)| *deducted != 0)
            .map(|(cohort, deducted)| {
                let mut owners = Some([exact::whole(0), exact::whole(0)]);
                for source in sources.iter().filter(|s| {
                    s.year == cohort.year
                        && s.period == cohort.period
                        && s.expiration == cohort.expiration
                        && s.special == cohort.special
                        && s.post_2017 == cohort.post_2017
                }) {
                    let amounts = if let Some(id) = source.origin {
                        d.income.iter().find(|e| e.id == id).map(|e| {
                            let mut a = [exact::whole(0), exact::whole(0)];
                            a[usize::from(e.owner == crate::DocumentOwner::Spouses)] =
                                source.balance.clone();
                            a
                        })
                    } else {
                        d.screening.as_ref().and_then(|s| match &s.nol {
                            NolHistory::Records { years, .. } => years
                                .iter()
                                .find(|v| v.year == source.year)
                                .and_then(|v| v.injured_owners.as_ref())
                                .map(|owners| owners.amounts().map(exact::whole)),
                            NolHistory::None => None,
                        })
                    };
                    owners = owners.zip(amounts).map(|(a, b)| [&a[0] + &b[0], &a[1] + &b[1]]);
                }
                InjuredNolPart {
                    key: alloc::format!(
                        "regular_nol_{}_{}_{}",
                        cohort.year,
                        cohort.period.map_or(0, |p| p.0),
                        cohort.period.map_or(0, |p| p.1)
                    ),
                    deduction: *deducted,
                    balances: owners,
                }
            })
            .collect()
    }
    pub(crate) fn research_deductions(&self, d: &crate::FederalDraft) -> BTreeMap<i64, i64> {
        let mut result = BTreeMap::new();
        let Some(crate::NolHistory::Records { years, confirmed: true }) =
            d.screening.as_ref().map(|s| &s.nol)
        else {
            return result;
        };
        for &(origin, year, allowed) in &self.deductions {
            if origin.is_some() {
                continue;
            }
            let Some(record) = years.iter().find(|r| r.year == year) else {
                continue;
            };
            let Some(crate::ResearchLossParts::Parts { sources }) = &record.research_parts else {
                continue;
            };
            // The kernel conserves owner dollars in input order. Use stable
            // actual source keys so moving a row cannot move a rounding dollar
            // between businesses. None retains the unrelated-business remainder.
            let mut claims: BTreeMap<Option<i64>, i64> =
                sources.iter().map(|part| (Some(part.source), part.amount)).collect();
            claims.insert(None, record.balance - claims.values().sum::<i64>());
            let weights: Vec<_> = claims.values().copied().collect();
            for (source, amount) in
                claims.into_keys().zip(crate::kernel::ratio::allocate_remaining(allowed, &weights))
            {
                if let Some(source) = source {
                    *result.entry(source).or_default() += amount;
                }
            }
        }
        result
    }

    pub(crate) fn set_sources(&mut self, single_estate: Option<i64>, has_estate: bool) {
        self.estate_source = single_estate;
        self.has_estate = has_estate;
    }
    // Section 108(b)(2)(A) includes carryovers TO the discharge year.
    // A 2006 loss is still eligible in 2026; reduce it before year-end expiry.
    pub(crate) fn discharge_balance(&self) -> Exact {
        &self.carryforward
            + self
                .years
                .iter()
                .filter(|y| y.expires_now)
                .map(|y| exact::whole(y.expired))
                .sum::<Exact>()
    }
    pub(crate) fn record(
        &self,
        current: i64,
        discharge: i64,
        r: &mut FederalResult,
    ) -> Result<(), LossAmountIssue> {
        let inputs: &[&str] = if self.estate_source.is_some() {
            &["income.estate_trust_portfolio.final_nol"]
        } else if self.has_estate {
            &["screening.nol", "income.estate_trust_portfolio.final_nol"]
        } else {
            &["screening.nol"]
        };
        let RecordedLossYears { years: yearly, carryforward_cents: carryforward } =
            recorded_loss_years(&self.years, current, discharge, self.estate_source, false)?;
        for (year, [opening, used, expired, reduced, closing]) in yearly {
            emit_lines(
                r,
                &mut (),
                [
                    ("opening", opening, msg!("EXPLAIN_NOL_N_N_OPENING")),
                    ("used", used, msg!("EXPLAIN_NOL_N_N_USED")),
                    ("expired", expired, msg!("EXPLAIN_NOL_N_N_EXPIRED")),
                    ("debt_reduction", reduced, msg!("EXPLAIN_NOL_N_N_DEBT_REDUCTION")),
                    ("carryforward", closing, msg!("EXPLAIN_NOL_N_N_CARRYFORWARD")),
                ],
                |(suffix, amount, explanation), _, _| {
                    Some(calculation_row(
                        None,
                        &self.estate_source.map_or_else(
                            || alloc::format!("nol_{year}_{suffix}"),
                            |id| alloc::format!("estate_trust_{id}_nol_{year}_{suffix}"),
                        ),
                        LineValue::MoneyCents(amount),
                        inputs,
                        if suffix == "debt_reduction" {
                            "https://www.irs.gov/instructions/i982"
                        } else {
                            "https://www.irs.gov/instructions/i172"
                        },
                        explanation,
                    ))
                },
            );
        }
        line(
            r,
            "prior_nol_deduction",
            self.deduction,
            inputs,
            "https://www.irs.gov/instructions/i172",
            msg!("EXPLAIN_PRIOR_NOL_DEDUCTION"),
        );
        line_cents(
            r,
            "regular_total_nol_carryforward",
            carryforward,
            inputs,
            "https://www.irs.gov/instructions/i172",
            msg!("EXPLAIN_REGULAR_TOTAL_NOL_CARRYFORWARD"),
        );
        Ok(())
    }
}

// Both tax systems reduce the same already-rounded cohort ledger in cents.
// Build all required native rows before emitting any, so a failed projection
// cannot publish a partially represented carryforward.
pub(crate) struct RecordedLossYears {
    pub years: Vec<(u32, [i64; 5])>,
    pub carryforward_cents: i64,
}

pub(crate) fn recorded_loss_years(
    years: &[LossUse],
    current: i64,
    discharge: i64,
    source: Option<i64>,
    alternative: bool,
) -> Result<RecordedLossYears, LossAmountIssue> {
    let mut remaining = discharge;
    let mut total = exact::whole(current);
    let mut yearly = BTreeMap::<u32, [Exact; 5]>::new();
    for y in years {
        let cents = |amount| exact::whole(amount) * exact::whole(100);
        let expiring = if y.expires_now { cents(y.expired) } else { exact::whole(0) };
        let reduced =
            exact::rounded(&phase::cap(cents(y.closing) + &expiring, [exact::whole(remaining)]))
                .ok_or_else(|| LossAmountIssue::new(source, Some(y.year), alternative))?;
        remaining = crate::kernel::carry::native_balance(remaining, reduced);
        let expired_reduction = phase::cap(exact::whole(reduced), [expiring]);
        let closing = cents(y.closing) - (exact::whole(reduced) - &expired_reduction);
        total += &closing;
        let amounts = [
            cents(y.opening),
            cents(y.used),
            cents(y.expired) - expired_reduction,
            exact::whole(reduced),
            closing,
        ];
        for (sum, amount) in yearly
            .entry(y.year)
            .or_insert_with(|| core::array::from_fn(|_| exact::whole(0)))
            .iter_mut()
            .zip(amounts)
        {
            *sum += amount;
        }
    }
    let mut rows = Vec::new();
    for (year, amounts) in yearly {
        let mut projected = [0; 5];
        for (to, amount) in projected.iter_mut().zip(amounts) {
            *to = exact::rounded(&amount)
                .ok_or_else(|| LossAmountIssue::new(source, Some(year), alternative))?;
        }
        rows.push((year, projected));
    }
    Ok(RecordedLossYears {
        years: rows,
        carryforward_cents: exact::rounded(&total)
            .ok_or_else(|| LossAmountIssue::new(source, None, alternative))?,
    })
}
