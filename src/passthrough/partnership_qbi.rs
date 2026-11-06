use crate::kernel::carry::{Amount, Cents};
// Coverage: partner section 1231 sale QBI — runtime owner integration.
//   Original statement bounds stay tied to the entered property box. Gross
//   SALE sources use the approved linear property-loss allocation, fixing
//   current SSTB qualification before activity limits and retaining the
//   approved current-listing qualification of released property carries.
//   One business eligibility/wage/UBIA decision combines current gains and
//   deductions; historical releases stay separate. Capital items are excluded
//   and undetermined lookback shares retain their named refusal.
//   Evidence: tests/passthrough/k1_partner_sale_qbi.rs and unchanged
//   tests/passthrough/k1_items.rs property fixtures; i8995, 26 CFR 1.199A-3. Core and
//   compound final returns pass tests/passthrough/k1_sale_final.rs and k1_compound.rs;
//   loss-only property EBL uses the resolved shared allocator (k1_excess.rs).
//   One opposite operating/property character also uses its gross qualified
//   loss and retained releases; tests/passthrough/k1_property_offsets.rs. No net-QBI
//   inference or second current-year eligibility pass is applied to histories.
//   Retained ordinary PTP losses reuse the same basis/risk/FIFO owners and
//   the actual same-PTP passive suspension. Released qualification joins only
//   the REIT/PTP component, including without a current statement; original
//   loss-year SSTB qualification stays unchanged. Other carried characters,
//   property and full-interest sales retain source limits. Fully allowed,
//   current ordinary PTP income accepts already-qualified partner deductions
//   before current SSTB eligibility, including an exactly exhausted income.
//   Deductions creating a qualified loss or joining other source characters
//   still need their separate qualified-source worksheet.
//   Positive current trades with partner deductions also retain actual
//   ordinary basis, at-risk and same-PTP passive histories; FIFO releases
//   keep loss-year qualification independently of current partner deductions.
//   Evidence: qbi_current_ptp and partnership_no_statement; i8995 Tracking
//   Losses, i8582 Special Instructions for PTPs, Reg. 1.199A-3(b)(1)(iv)/(c)(3)(ii).
//   Loss-only EBL work temporarily joins source-tagged ordinary/PTP rows
//   through the existing qualified-deferral owner, then restores each category.
//   Current and original-year qualified fractions stay separate; an EBL-held
//   qualified portion is excluded from the current REIT/PTP carry. Future EBL
//   ingestion remains separate from this current-year calculation.
//   Full R9 retained/mixed-source acceptance remains open.
use crate::business_rules::QbiActivity;
use crate::kernel::ratio::dollars;
use crate::kernel::{phase, ratio::exact};
use crate::scorp_basis::{LossRelease, LossVintage, release_losses};
use crate::scorp_qbi::{
    current_qualified_release, finalize_current_vintage, loss_years, valid_history,
};
use crate::{FederalStatus, PartnerK1};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Partnership qualified-loss histories
/// Keep the three loss limits separate. Original qualified portions already reflect the loss year's eligibility, including SSTB restrictions; pre-2018 losses have no qualified portion.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerQbiHistory {
    /// Losses suspended by outside basis
    /// Reconcile to prior basis losses. An empty list confirms none.
    /// ----
    /// Loss year
    pub basis: Vec<crate::ScorpLossYear>,
    /// Losses suspended by at-risk limits
    /// Already passed basis. Do not repeat basis losses.
    /// ----
    /// Loss year
    pub risk: Vec<crate::ScorpLossYear>,
    /// Losses suspended by passive limits
    /// Already passed basis and at-risk limits. Do not repeat either earlier balance.
    /// ----
    /// Loss year
    pub passive: Vec<crate::ScorpLossYear>,
}

pub(crate) struct PartnerQbiResult {
    pub current: QbiActivity,
    pub historical: i64,
    pub historical_cents: Amount<Cents>,
    pub current_loss_cents: Amount<Cents>,
    pub carries: [LossRelease; 3],
    // Exact qualified PTP SOURCE CENTS, retained through SSTB eligibility and
    // equivalent-source joining until Form 8995 line 6 / Form 8995-A line 28.
    // This transient value is zero for an ordinary
    // K-1. Combines with qualified REIT dividends outside `current`, which
    // stays a harmless placeholder so it never enters the wage/UBIA-limited
    // or aggregated activities pool.
    pub ptp: exact::Exact,
}

impl PartnerQbiResult {
    // Called only for actual EBL work. The fully allowed current-only route
    // retains an Exact source; its native qualified-loss cent boundary occurs
    // here, without changing or rounding the normal category join.
    pub(crate) fn ptp_current_for_excess(
        &self,
    ) -> Result<Amount<Cents>, (&'static str, &'static str)> {
        if self.current_loss_cents.get() != 0
            || self.historical_cents.get() != 0
            || self.ptp >= exact::whole(0)
        {
            return Ok(self.current_loss_cents);
        }
        exact::rounded(&(-&self.ptp)).and_then(|value| Amount::new(value).ok()).ok_or((
            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            crate::fixture_english!(
                "Review the document amounts; they must be within the supported input range."
            ),
        ))
    }
}

// The retained adapter covers this interest's ordinary loss character only.
// Other source characters and full-interest sales need their own qualified
// loss association; the ordinary history never establishes it by inference.
const PTP_RETAINED_UNAVAILABLE: (&str, &str) = (
    "ISSUE_PARTNERSHIP_QBI_A_PUBLICLY_TRADED_PARTNERSHIP_INTEREST_WITH_77211554",
    crate::fixture_english!(
        "The App calculates ordinary PTP qualified losses from reconciled basis, at-risk and same-PTP passive records, including positive current trades with qualified partner deductions. Other carried characters, deductions exceeding current qualified income, EBL combinations with positive business adjustments, and retained-loss combinations with property or full-interest sales still need a separate qualified-source worksheet."
    ),
);

pub(crate) fn ptp_retained_source(owner: &PartnerK1) -> Result<(), (&'static str, &'static str)> {
    if !matches!(owner.disposition, Some(crate::K1Disposition::Kept))
        || owner.regular.carried != crate::K1LossCarryforwards::default()
        || owner.regular.capital_carried != crate::PartnerCapitalCarry::default()
        || owner.regular.section1231.is_some()
        || !owner.section179_sales.is_empty()
        || owner.section179 != 0
        || owner.section179_carryover != 0
    {
        return Err(PTP_RETAINED_UNAVAILABLE);
    }
    Ok(())
}

// Admission for already computed PTP EBL sources only. Inspect each
// adjustment's actual signed effect before netting can hide a profit offset.
pub(crate) fn ptp_excess_source(
    owner: &PartnerK1,
    current: Amount<Cents>,
    historical: Amount<Cents>,
    financial_net_cents: i64,
) -> Result<(), (&'static str, &'static str)> {
    ptp_retained_source(owner)?;
    if owner.income.iter().any(|row| {
        row.code.kind() == crate::k1_items::K1ItemKind::BoxOneAdjustment && row.amount > 0
    }) || owner.deductions.iter().any(|row| {
        row.code.kind() == crate::k1_items::K1ItemKind::BoxOneAdjustment && row.amount < 0
    }) {
        return Err(PTP_RETAINED_UNAVAILABLE);
    }
    let range = (
        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
        crate::fixture_english!(
            "Review the document amounts; they must be within the supported input range."
        ),
    );
    let loss = financial_net_cents.checked_neg().ok_or(range)?;
    let qualified = current.get().checked_add(historical.get()).ok_or(range)?;
    if loss <= 0 || qualified > loss {
        return Err((
            "ISSUE_PARTNERSHIP_QBI_RECONCILE_PARTNERSHIP_QBI_WITH_THE_RESOLVED_708814BB",
            crate::fixture_english!(
                "Reconcile partnership QBI with the resolved passive limit and partner deductions."
            ),
        ));
    }
    Ok(())
}

// A newly limited amount enters this provision's current-year ledger. Reuse
// the shared FIFO and original-qualified-percentage engine for every stage.
pub(crate) fn stage(
    history: &[crate::ScorpLossYear],
    current: Amount<Cents>,
    qualified: Amount<Cents>,
    allowed: Amount<Cents>,
) -> Result<LossRelease, (&'static str, &'static str)> {
    let mut years = loss_years(history);
    if current.get() > 0 {
        years.push(LossVintage {
            year: 2026,
            original: current,
            original_qbi: qualified,
            remaining: current,
            remaining_qbi: qualified,
        });
    }
    release_losses(&years, allowed).map(finalize_current_vintage)
        .map_err(|_| ("ISSUE_PARTNERSHIP_QBI_RECONCILE_THE_ROUNDED_PARTNERSHIP_QBI_LOSS_YEAR_202D016C", crate::fixture_english!("Reconcile the rounded partnership QBI loss-year amounts and their qualified portions.")))
}

// The same basis, at-risk and passive FIFO release applies whether or not
// this year's K-1 has a section 199A statement. Historical qualification was
// fixed in the original loss year; it is never redetermined on release.
// https://www.irs.gov/instructions/i8995
pub(crate) struct PartnerLossRelease {
    current: Amount<Cents>,
    pub historical: Amount<Cents>,
    pub carries: [LossRelease; 3],
}

fn released_losses(
    owner: &PartnerK1,
    suspended: Amount<Cents>,
    qualified: i64,
    retained_only: bool,
    resolved_pool: Option<&crate::partnership::PartnerPoolResult>,
) -> Result<PartnerLossRelease, (&'static str, &'static str)> {
    let source = &owner.regular;
    let totals = [source.basis.prior_loss, source.risk.prior_loss, source.passive_loss];
    let empty = PartnerQbiHistory { basis: Vec::new(), risk: Vec::new(), passive: Vec::new() };
    let history = owner.qbi_records.history.as_ref().or_else(|| (totals == [0; 3]).then_some(&empty))
            .ok_or(("ISSUE_PARTNERSHIP_QBI_COMPLETE_HISTORICAL_PARTNERSHIP_QBI_LOSS_RECORDS_4E8D198F", crate::fixture_english!("Complete historical partnership QBI loss records for each nonzero suspended balance.")))?;
    let rows = [history.basis.as_slice(), history.risk.as_slice(), history.passive.as_slice()];
    let histories: [&[crate::ScorpLossYear]; 3] =
        core::array::from_fn(
            |index| {
                if retained_only && totals[index] == 0 { &[] } else { rows[index] }
            },
        );
    for (years, total) in histories.into_iter().zip(totals) {
        if !valid_history(years, Some(total))
            || loss_years(years).iter().map(|v| v.remaining.get()).sum::<i64>() != total
        {
            return Err((
                "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363",
                crate::fixture_english!(
                    "Reconcile historical partnership qualified-loss records with each separate suspended balance, including rounded totals."
                ),
            ));
        }
    }
    let default_pool;
    let pool = if let Some(pool) = resolved_pool {
        pool
    } else {
        default_pool = owner.pool(source)?;
        &default_pool
    };
    if suspended.get() > phase::floor(-pool.passive, 0) {
        return Err((
            "ISSUE_PARTNERSHIP_QBI_RECONCILE_PARTNERSHIP_QBI_WITH_THE_RESOLVED_708814BB",
            crate::fixture_english!(
                "Reconcile partnership QBI with the resolved passive limit and partner deductions."
            ),
        ));
    }
    let basis = stage(
        histories[0],
        Amount::<Cents>::new(phase::floor(-source.ordinary, 0))
            .expect("nonnegative qualified source loss"),
        Amount::<Cents>::new(qualified).expect("nonnegative qualified source loss"),
        pool.activity.basis.allowed[crate::k1_loss::K1LossClass::Ordinary],
    )?;
    if basis.remaining.iter().map(|v| v.remaining.get()).sum::<i64>()
        != pool.activity.basis.suspended[crate::k1_loss::K1LossClass::Ordinary].get()
    {
        return Err((
            "ISSUE_PARTNERSHIP_QBI_ROUNDED_PARTNERSHIP_BASIS_LOSS_VINTAGES_DO_NOT_899E8A3B",
            crate::fixture_english!(
                "Rounded partnership basis-loss vintages do not reconcile to the basis worksheet."
            ),
        ));
    }
    let risk = stage(
        histories[1],
        basis.amount,
        basis.qbi,
        pool.activity.ledger.risk_allowed[crate::k1_loss::K1LossClass::Ordinary],
    )?;
    if risk.remaining.iter().map(|v| v.remaining.get()).sum::<i64>()
        != pool.activity.ledger.risk_suspended[crate::k1_loss::K1LossClass::Ordinary].get()
    {
        return Err((
            "ISSUE_PARTNERSHIP_QBI_ROUNDED_PARTNERSHIP_AT_RISK_VINTAGES_DO_NOT_93E77889",
            crate::fixture_english!(
                "Rounded partnership at-risk vintages do not reconcile to the at-risk worksheet."
            ),
        ));
    }
    let nonpassive = owner.material_for_losses() == crate::participation::Participation::Yes;
    let passive = if nonpassive {
        let mut v = stage(
            histories[2],
            Amount::<Cents>::new(0).expect("nonnegative qualified source loss"),
            Amount::<Cents>::new(0).expect("nonnegative qualified source loss"),
            Amount::<Cents>::new(source.passive_loss - suspended.get())
                .expect("nonnegative qualified source loss"),
        )?;
        v.amount =
            Amount::new(v.amount.get() + risk.amount.get()).expect("bounded qualified release");
        v.qbi = Amount::new(v.qbi.get() + risk.qbi.get()).expect("bounded qualified release");
        v
    } else {
        stage(
            histories[2],
            risk.amount,
            risk.qbi,
            Amount::<Cents>::new(risk.amount.get() + source.passive_loss - suspended.get())
                .expect("nonnegative qualified source loss"),
        )?
    };
    let current_loss =
        current_qualified_release(qualified, &basis, &risk, (!nonpassive).then_some(&passive));
    let historical = passive.qbi.get() - current_loss;
    Ok(PartnerLossRelease {
        current: Amount::new(current_loss).expect("nonnegative qualified current release"),
        historical: Amount::new(historical).expect("nonnegative qualified historical release"),
        carries: [basis, risk, passive],
    })
}

// No current statement means zero current-year qualified income or loss.
// Only histories attached to actual suspended balances enter reconciliation;
// unrelated retained records remain structurally checked by the source gate.
pub(crate) fn historical_without_statement(
    owner: &PartnerK1,
    suspended: Amount<Cents>,
) -> Result<Option<PartnerLossRelease>, (&'static str, &'static str)> {
    historical_without_statement_with_pool(owner, suspended, None)
}
pub(crate) fn historical_without_statement_with_pool(
    owner: &PartnerK1,
    suspended: Amount<Cents>,
    resolved_pool: Option<&crate::partnership::PartnerPoolResult>,
) -> Result<Option<PartnerLossRelease>, (&'static str, &'static str)> {
    if !owner.qbi_records.valid() {
        return Err((
            "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363",
            crate::fixture_english!(
                "Reconcile historical partnership qualified-loss records with each separate suspended balance, including rounded totals."
            ),
        ));
    }
    if [owner.regular.basis.prior_loss, owner.regular.risk.prior_loss, owner.regular.passive_loss]
        == [0; 3]
        || owner.qbi_records.history.is_none()
    {
        return Ok(None);
    }
    if owner.publicly_traded {
        ptp_retained_source(owner)?;
    }
    released_losses(owner, suspended, 0, true, resolved_pool).map(Some)
}

/// Partnership ordinary-business section 199A statement
/// Use your allocated K-1 box 20 code Z statement, not the partnership's totals. This supplies source amounts; the App applies partner deductions and the return's QBI limits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartnerQbiStatement {
    /// Qualified ordinary business income or loss
    /// Signed section 199A ordinary amount before partner-level deductions and suspended-loss releases. Exclude guaranteed payments, capital gains, portfolio income, REIT dividends and PTP items. Missing source facts are not zero.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub income: i64,
    /// Allocated section 199A W-2 wages
    /// Use the partnership statement, not an employee W-2 issued to you.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
    /// Allocated unadjusted basis of qualified property
    /// UBIA from the section 199A statement, not your outside basis or depreciated property value.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub property: i64,
    /// Specified service trade or business
    /// The K-1 box 20 code Z section 199A statement reports it.
    pub service: bool,
    /// Qualified business-property section 1231 gain (loss)
    /// The statement's qualified part of the box 1 business's box 10 amount, with the same sign and no larger. Its return-wide ordinary or capital character is figured before it enters QBI.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub section1231_qbi: i64,
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
}

/// Your partnership QBI records
/// From your retained loss worksheets and allocation records. Document import preserves these records independently of the printed section 199A statement.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PartnerQbiRecords {
    /// Qualified portion of gross self-employment income
    /// From the box 20 code Z statement and the box 14 code B or C gross income, before expenses, not net QBI. Must fit the separate farm/nonfarm SE gross income. Enter the full gross amount when all is qualified and zero when none is qualified or there is no box 14 code B or C amount. Exclude all guaranteed payments and amounts outside SE income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub qualified_se_gross: i64,
    /// Retained original-year qualified-loss records
    /// Prior basis, at-risk or passive balances need their actual original-year qualified portions for QBI. No records entered does not confirm those balances are zero. Missing records retain the separate QBI notice; leaving a qualified loss out can increase the QBI deduction. The income-tax loss keeps its existing limits.
    /// [NullText]
    /// No retained qualified-loss records are entered
    /// [SetText]
    /// Use these retained qualified-loss records
    pub history: Option<PartnerQbiHistory>,
}

impl PartnerQbiRecords {
    pub(crate) fn valid(&self) -> bool {
        self.validation_issue("").is_none()
    }

    pub(crate) fn validation_issue(
        &self,
        key: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        // Arithmetic safety: half_se_by_gross and calculate_combined_with_pool retain
        // native gross/QBI allocations and cents transfers.
        if !(0..=100_000_000_000).contains(&self.qualified_se_gross) {
            return Some(
                crate::validation::ValidationIssue::new(key).at_field("qualified_se_gross"),
            );
        }
        self.history.as_ref().and_then(|history| {
            [("basis", &history.basis), ("risk", &history.risk), ("passive", &history.passive)]
                .into_iter()
                .find_map(|(field, years)| {
                    crate::scorp_qbi::history_issue(years, None, field, key)
                        .map(|issue| issue.in_field("history"))
                })
        })
    }
}

impl PartnerQbiStatement {
    pub(crate) fn valid(&self, owner: &PartnerK1) -> bool {
        self.validation_issue(owner, "").is_none()
    }
    pub(crate) fn validation_issue(
        &self,
        owner: &PartnerK1,
        key: &'static str,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        // Arithmetic safety: calculate_combined_with_pool combines native QBI
        // component cents and allocations before its form-dollar transfer.
        if !((owner.regular.ordinary.min(0)..=owner.regular.ordinary.max(0)).contains(&self.income)
            && {
                let net = owner.regular.section1231.as_ref().map_or(0, |v| v.net);
                (net.min(0)..=net.max(0)).contains(&self.section1231_qbi)
            }
            && [self.wages, self.property, self.reit_dividends]
                .into_iter()
                .all(|v| (0..=100000000000).contains(&v))
            && (-100000000000..=100000000000).contains(&self.ptp_income))
        {
            return Some(ValidationIssue::new(key));
        }
        // Qualified PTP items never carry wage/UBIA amounts. Their actual
        // SSTB classification still applies under Reg. 1.199A-3(c)(3)(ii)
        // and 1.199A-5(a)(2); it is a lawful source answer.
        if owner.publicly_traded {
            if self.wages != 0 {
                return Some(ValidationIssue::new(key).at_field("wages"));
            }
            if self.property != 0 {
                return Some(ValidationIssue::new(key).at_field("property"));
            }
        }
        None
    }

    // Form2441 line24 is an expense of the selected partnership's trade,
    // separate from the earned-income-based health/retirement deductions.
    // Reg1.199A-3(b)(1)(vi): a wholly qualified ordinary trade bears its
    // expense even when a limited partner has no qualified SE gross income.
    // A partial statement still needs actual meaningful gross attribution;
    // net QBI/ordinary-income proportions do not establish that allocation.
    pub(crate) fn qualified_care_deduction(
        &self,
        owner: &PartnerK1,
        amount: i64,
    ) -> Result<i64, (&'static str, &'static str)> {
        if amount == 0 || self.income == owner.regular.ordinary {
            return Ok(amount);
        }
        if owner.qbi_records.qualified_se_gross > 0 {
            return self.qualified_deduction(owner, amount);
        }
        Err((
            "ISSUE_PARTNERSHIP_QBI_PROVIDE_PARTNERSHIP_GROSS_INCOME_FOR_PARTNER_DEDUCTION_67A6F2C5",
            crate::fixture_english!(
                "Provide partnership gross income for partner-deduction QBI attribution."
            ),
        ))
    }

    pub(crate) fn qualified_deduction(
        &self,
        owner: &PartnerK1,
        amount: i64,
    ) -> Result<i64, (&'static str, &'static str)> {
        if amount == 0 {
            return Ok(0);
        }
        let gross = owner
            .se_gross
            .as_ref()
            .ok_or(("ISSUE_PARTNERSHIP_QBI_PROVIDE_PARTNERSHIP_GROSS_INCOME_FOR_PARTNER_DEDUCTION_67A6F2C5", crate::fixture_english!("Provide partnership gross income for partner-deduction QBI attribution.")))?
            .source()
            .1;
        let qualified = owner.qbi_records.qualified_se_gross;
        if gross <= 0 || !owner.qualified_se_gross_fits(gross, qualified) {
            return Err((
                "ISSUE_PARTNERSHIP_QBI_RECONCILE_QUALIFIED_AND_TOTAL_GROSS_INCOME_AD915D46",
                crate::fixture_english!(
                    "Reconcile qualified and total gross income for partner deductions."
                ),
            ));
        }
        Ok(crate::kernel::ratio::allocate_remaining(amount, &[qualified, gross - qualified])[0])
    }

    #[cfg(test)]
    pub(crate) fn calculate(
        &self,
        owner: &PartnerK1,
        suspended: Amount<Cents>,
        deductions: i64,
        taxable: i64,
        status: FederalStatus,
    ) -> Result<PartnerQbiResult, (&'static str, &'static str)> {
        self.calculate_with_sales(
            owner,
            suspended,
            deductions,
            taxable,
            status,
            &Default::default(),
        )
    }

    #[cfg(test)]
    pub(crate) fn calculate_with_sales(
        &self,
        owner: &PartnerK1,
        suspended: Amount<Cents>,
        deductions: i64,
        taxable: i64,
        status: FederalStatus,
        sales: &crate::k1_sale_qbi::CurrentSales,
    ) -> Result<PartnerQbiResult, (&'static str, &'static str)> {
        self.calculate_combined(
            owner,
            suspended,
            deductions,
            (taxable, status),
            sales,
            &crate::section1231::PartnerPropertyQbi::default(),
        )
    }

    pub(crate) fn calculate_combined(
        &self,
        owner: &PartnerK1,
        suspended: Amount<Cents>,
        deductions: i64,
        eligibility: (i64, FederalStatus),
        sales: &crate::k1_sale_qbi::CurrentSales,
        property: &crate::section1231::PartnerPropertyQbi,
    ) -> Result<PartnerQbiResult, (&'static str, &'static str)> {
        self.calculate_combined_with_pool(
            owner,
            suspended,
            deductions,
            eligibility,
            sales,
            property,
            None,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn calculate_combined_with_pool(
        &self,
        owner: &PartnerK1,
        suspended: Amount<Cents>,
        deductions: i64,
        eligibility: (i64, FederalStatus),
        sales: &crate::k1_sale_qbi::CurrentSales,
        property: &crate::section1231::PartnerPropertyQbi,
        resolved_pool: Option<&crate::partnership::PartnerPoolResult>,
    ) -> Result<PartnerQbiResult, (&'static str, &'static str)> {
        let (taxable, status) = eligibility;
        if !self.valid(owner) || !owner.complete() {
            return Err((
                "ISSUE_PARTNERSHIP_QBI_COMPLETE_AND_RECONCILE_THE_PARTNERSHIP_S_ORDINARY_82FE9FDB",
                crate::fixture_english!(
                    "Complete and reconcile the partnership's ordinary section 199A statement and partner adjustments."
                ),
            ));
        }
        // The section 751 gain on the sale of the whole interest is qualified
        // business income of the partnership's trade or business (Reg.
        // 1.199A-3(b)(1)(i)); for a publicly traded partnership it is
        // qualified PTP income ((c)(2)(ii)(B)).
        let statement_income = self.income;
        let income = statement_income + owner.sale_qbi();
        if owner.publicly_traded {
            // Qualified PTP income stays outside the ordinary QBI activity
            // pool and its wage/UBIA limit (Reg. 1.199A-1(d)(3)(iii)).
            // Retained qualification follows the actual ordinary loss owners,
            // never the current year's SSTB factor.
            if deductions < 0
                || owner.regular.carried != crate::K1LossCarryforwards::default()
                || owner.regular.capital_carried != crate::PartnerCapitalCarry::default()
            {
                return Err(PTP_RETAINED_UNAVAILABLE);
            }
            // A current loss can be held entirely before PAL, leaving both
            // history and the final passive suspension empty. Read those
            // earlier suspensions from the same actual loss owners as well.
            // Fully allowed current-only sources, including full-interest
            // sales, retain their exact joining path.
            let current_pool =
                if (owner.regular.ordinary < 0 || deductions != 0) && resolved_pool.is_none() {
                    Some(owner.pool(&owner.regular)?)
                } else {
                    None
                };
            let resolved_pool = resolved_pool.or(current_pool.as_ref());
            let retained = owner.qbi_records.history.is_some()
                || suspended.get() != 0
                || (owner.regular.ordinary < 0
                    && resolved_pool.is_some_and(|pool| {
                        pool.activity.basis.suspended[crate::k1_loss::K1LossClass::Ordinary].get()
                            > 0
                            || pool.activity.ledger.risk_suspended
                                [crate::k1_loss::K1LossClass::Ordinary]
                                .get()
                                > 0
                    }))
                || [
                    owner.regular.basis.prior_loss,
                    owner.regular.risk.prior_loss,
                    owner.regular.passive_loss,
                ]
                .into_iter()
                .any(|amount| amount != 0);
            if retained {
                ptp_retained_source(owner)?;
                if sales.gain_cents != 0
                    || sales.loss_cents.get() != 0
                    || property.gain_cents != 0
                    || property.current_loss_cents.get() != 0
                    || property.historical_loss_cents.get() != 0
                {
                    return Err(PTP_RETAINED_UNAVAILABLE);
                }
            }
            let income = if deductions != 0 {
                // Reg. 1.199A-3(b)(1)(vi), (c)(3)(ii): the caller already
                // attributed these deductions to qualified income. They reduce
                // that current trade before SSTB eligibility, without changing
                // the partnership's basis, at-risk or passive-loss operands.
                ptp_retained_source(owner)?;
                let pool = resolved_pool.ok_or(PTP_RETAINED_UNAVAILABLE)?;
                use crate::k1_loss::K1LossClass::Ordinary;
                if owner.trade_kind != crate::ScorpTradeKind::Ordinary
                    || owner.regular.ordinary < 0
                    || self.ptp_income != 0
                    || owner.regular.rentals != Default::default()
                    || owner.income.iter().any(|row| row.amount != 0)
                    || owner.deductions.iter().any(|row| row.amount != 0)
                    || owner.separate.as_ref().is_some_and(|items| {
                        [
                            items.short,
                            items.long,
                            items.collectibles,
                            items.royalties,
                            items.unrecaptured_partnership,
                            items.unrecaptured_fund,
                            items.cash_charity,
                            items.foreign_taxes,
                        ]
                        .into_iter()
                        .any(|amount| amount != 0)
                    })
                    || owner.prior_fuel_credit.is_some()
                    || sales.gain_cents != 0
                    || sales.loss_cents.get() != 0
                    || property.gain_cents != 0
                    || property.current_loss_cents.get() != 0
                    || property.historical_loss_cents.get() != 0
                    || pool.activity.income.get() != owner.regular.ordinary
                    // Financial ordinary income already reflects allowed
                    // basis/at-risk losses. These are separate from the caller's
                    // current qualified partner deductions and FIFO qualification.
                    || pool.current_ordinary_with_items
                        != owner.regular.ordinary
                            - pool.activity.ledger.risk_allowed[Ordinary].get()
                    // Prior passive losses join only after that financial stage.
                    || pool.nonpassive + pool.passive + owner.regular.passive_loss
                        != pool.current_ordinary_with_items
                {
                    return Err(PTP_RETAINED_UNAVAILABLE);
                }
                // Lift the owner's reported Dollars to cents, preserving that
                // established amount rather than inventing source precision.
                let net = exact::whole(income) - exact::whole(deductions) * exact::whole(100);
                exact::rounded(&net).filter(|value| *value >= 0).ok_or(PTP_RETAINED_UNAVAILABLE)?
            } else {
                income
            };
            // The ordinary trade's SSTB answer does not classify separately
            // reported PTP items from another underlying trade or business.
            // Preserve this approved source boundary until its owner supplies
            // actual item-level business/SSTB classification.
            if self.service && self.ptp_income != 0 {
                return Err((
                    "ISSUE_QBI_PTP_COMPONENT_CLASSIFICATION",
                    crate::fixture_english!(
                        "Separately reported PTP items need their actual business and specified-service source classification before the income limit can be applied."
                    ),
                ));
            }
            let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
                income,
                Some(self.service),
                false,
                taxable,
                status,
            )?;
            let eligible = exact::whole(income)
                * exact::ratio(applicable, range).expect("positive canonical QBI range");
            let released = if retained {
                // The qualified-loss owner accepts source cents. Round its
                // current qualification once at that boundary; later stages
                // use the owner's original ratios and actual allowances.
                let qualified = if income < 0 {
                    exact::rounded(&(-&eligible)).ok_or((
                        "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                        crate::fixture_english!("Review the document amounts; they must be within the supported input range."),
                    ))?
                } else {
                    0
                };
                Some(released_losses(owner, suspended, qualified, false, resolved_pool)?)
            } else {
                None
            };
            let historical_cents = released.as_ref().map_or(Amount::default(), |v| v.historical);
            let current_loss_cents = released.as_ref().map_or(Amount::default(), |v| v.current);
            let ptp = if released.is_some() {
                // Positive current source and allowed qualified losses remain
                // distinct; a PAL-held current loss cannot enter this category.
                (if income > 0 { eligible } else { exact::whole(0) })
                    - exact::whole(current_loss_cents.get())
                    - exact::whole(historical_cents.get())
            } else {
                eligible
            };
            return Ok(PartnerQbiResult {
                current: QbiActivity {
                    source_wages_cents: None,
                    source_property_cents: None,
                    source_income_cents: None,
                    owner: None,
                    income: 0,
                    wages: 0,
                    property: 0,
                    active: false,
                    service: Some(false),
                    group: None,
                },
                historical: dollars(historical_cents.get()),
                historical_cents,
                current_loss_cents,
                carries: released.map_or_else(
                    || {
                        [
                            LossRelease {
                                amount: Amount::default(),
                                qbi: Amount::default(),
                                remaining: Vec::new(),
                            },
                            LossRelease {
                                amount: Amount::default(),
                                qbi: Amount::default(),
                                remaining: Vec::new(),
                            },
                            LossRelease {
                                amount: Amount::default(),
                                qbi: Amount::default(),
                                remaining: Vec::new(),
                            },
                        ]
                    },
                    |value| value.carries,
                ),
                ptp,
            });
        }
        if deductions < 0 {
            return Err((
                "ISSUE_PARTNERSHIP_QBI_RECONCILE_PARTNERSHIP_QBI_WITH_THE_RESOLVED_708814BB",
                crate::fixture_english!(
                    "Reconcile partnership QBI with the resolved passive limit and partner deductions."
                ),
            ));
        }
        // The caller supplies only this business's qualified share from the
        // owner-level gross-income allocation, before current SSTB eligibility.
        let material = owner.participation.as_ref().map_or(
            crate::participation::Participation::Unavailable,
            crate::PartnerParticipation::material,
        );
        let fuel = owner.prior_fuel_credit.as_ref().map_or(0, |p| p.taxable_amount());
        if (income > 0 || sales.gain_cents > 0 || property.gain_cents > 0 || fuel > 0)
            && material == crate::participation::Participation::Unavailable
        {
            return Err((
                "ISSUE_PARTNERSHIP_QBI_RESOLVE_ACTUAL_MATERIAL_PARTICIPATION_FOR_THE_3C3DCC18",
                crate::fixture_english!(
                    "Resolve actual material participation for the minimum QBI deduction; nonpassive trading classification alone is insufficient."
                ),
            ));
        }
        // Establish current loss-year eligibility before applying any limit.
        // The half-SE deduction itself is not a partnership basis/passive loss.
        let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
            income,
            Some(self.service),
            false,
            taxable,
            status,
        )?;
        let qualified = crate::kernel::ratio::fraction(phase::floor(-income, 0), applicable, range);
        let released = released_losses(owner, suspended, qualified, false, resolved_pool)?;
        let current_loss = released.current.get();
        let historical_cents = released.historical.get() + property.historical_loss_cents.get();
        let historical = dollars(historical_cents);
        let gain_cents = phase::floor(income, 0) + sales.gain_cents + property.gain_cents + fuel
            - deductions * 100;
        let (applicable, range) = crate::business_rules::qbi_eligibility_factor(
            gain_cents,
            Some(self.service),
            false,
            taxable,
            status,
        )?;
        let qualified_current_loss =
            current_loss + sales.loss_cents.get() + property.current_loss_cents.get();
        let current_cents =
            crate::kernel::ratio::fraction(gain_cents, applicable, range) - qualified_current_loss;
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
        Ok(PartnerQbiResult {
            current,
            historical,
            historical_cents: Amount::new(historical_cents)
                .expect("nonnegative qualified historical release"),
            current_loss_cents: Amount::new(qualified_current_loss)
                .expect("nonnegative qualified current release"),
            carries: released.carries,
            ptp: exact::whole(0),
        })
    }

    #[cfg(test)]
    pub(crate) fn activity(
        &self,
        owner: &PartnerK1,
        suspended: Amount<Cents>,
        half_se: i64,
        taxable: i64,
        status: FederalStatus,
    ) -> Result<QbiActivity, (&'static str, &'static str)> {
        self.calculate(owner, suspended, half_se, taxable, status).map(|v| v.current)
    }
}

// QBI-specific attribution. Other worksheets retain their own deduction
// allocation. This never recalculates the owner's actual/optional SE tax.
#[derive(Debug)]
pub(crate) enum QbiGrossError {
    Missing(Vec<crate::IssueContext>),
    Inconsistent(crate::IssueContext),
    UnavailableWeight(crate::IssueContext),
}

// Parent queries retain the existing sum of qualified shares. Native current
// statements additionally consume their actual block share before eligibility.
#[derive(Debug, Default)]
pub(crate) struct QbiHalfSeAllocation {
    pub sources: alloc::collections::BTreeMap<i64, i64>,
    pub native_blocks: alloc::collections::BTreeMap<(i64, i64), i64>,
}

struct QbiGrossSource {
    parent: i64,
    component: Option<i64>,
    block: Option<i64>,
    gross: i64,
    qualified: i64,
}

#[cfg(test)]
pub(crate) fn half_se_by_gross(
    draft: &crate::FederalDraft,
    halves: [i64; 2],
) -> Result<QbiHalfSeAllocation, QbiGrossError> {
    let claims = crate::related_forms::effective_claimants(draft);
    half_se_by_gross_with_claims(draft, halves, &claims)
}

pub(crate) fn half_se_by_gross_with_claims(
    draft: &crate::FederalDraft,
    halves: [i64; 2],
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<QbiHalfSeAllocation, QbiGrossError> {
    use crate::{BusinessActivity, BusinessSource, FederalIncome, Owner};
    let mut result = QbiHalfSeAllocation::default();
    let mut missing = Vec::new();
    for (index, half) in halves.into_iter().enumerate() {
        let owner = if index == 0 { Owner::Taxpayer } else { Owner::Spouse };
        let mut sources = Vec::new();
        let mut owner_missing = false;
        for entry in draft.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(owner)) {
            let (gross, qualified) = match &entry.income {
                FederalIncome::Business { data }
                    if !crate::statutory_wages::business(draft, entry.id) =>
                {
                    let routed = if matches!(data.activity, BusinessActivity::Farming { .. }) {
                        crate::federal_rules::farm_routed_income_with_claims(
                            draft,
                            index,
                            entry.id,
                            source_claims,
                        )
                        .map_err(|_| {
                            QbiGrossError::UnavailableWeight(
                                crate::interview::income_issue_context(entry),
                            )
                        })?
                    } else {
                        0
                    };
                    let gross = phase::floor(
                        data.earned_receipts() + routed
                            - if matches!(data.activity, BusinessActivity::Farming { .. }) {
                                0
                            } else {
                                data.inventory.cost()
                            },
                        0,
                    );
                    (
                        gross,
                        if matches!(data.source, BusinessSource::ForeignServices { .. }) {
                            0
                        } else {
                            gross
                        },
                    )
                }
                FederalIncome::Partnership { data } => {
                    if half == 0
                        || (data.limited_for_se == Some(true) && data.guaranteed_services == 0)
                    {
                        (0, 0)
                    } else {
                        let Some(gross) = data.se_gross.as_ref() else {
                            missing.push(crate::interview::income_issue_context(entry));
                            owner_missing = true;
                            continue;
                        };
                        let gross = gross.source().1;
                        // With no section 199A statement the partnership reports
                        // no QBI (Reg. 1.199A-6(b)(3)(iii)), so none of its gross
                        // SE income is qualified.
                        let qualified =
                            data.qbi.as_ref().map_or(0, |_| data.qbi_records.qualified_se_gross);
                        if !data.qualified_se_gross_fits(gross, qualified) {
                            return Err(QbiGrossError::Inconsistent(
                                crate::interview::income_issue_context(entry),
                            ));
                        }
                        (gross, qualified)
                    }
                }
                FederalIncome::PartnershipActivities { data } => {
                    if half == 0 {
                        (0, 0)
                    } else {
                        match data.ordinary_se_source() {
                            Ok(output) => {
                                let Some(source) = output.self_employment else {
                                    sources.push(QbiGrossSource {
                                        parent: entry.id,
                                        component: None,
                                        block: None,
                                        gross: 0,
                                        qualified: 0,
                                    });
                                    continue;
                                };
                                if output.current_qualified.iter().any(|v| v.qualified_se_gross > 0)
                                {
                                    // Each block's actual qualified gross and that component's
                                    // unqualified residual enter the ONE owner-wide allocation.
                                    // No rounded parent share is subsequently split again.
                                    for component in source.components {
                                        let qualified = output
                                            .current_qualified
                                            .iter()
                                            .find(|v| v.component == component.component);
                                        sources.push(QbiGrossSource {
                                            parent: entry.id,
                                            component: Some(component.component),
                                            block: qualified.map(|v| v.block),
                                            gross: component.gross,
                                            qualified: qualified
                                                .map_or(0, |v| v.qualified_se_gross),
                                        });
                                    }
                                    continue;
                                }
                                // Preserve the old single-parent zero-qualified slot, including
                                // its rounding effect on this owner's other business sources.
                                (source.gross, 0)
                            }
                            Err(_) => {
                                missing.push(crate::interview::income_issue_context(entry));
                                owner_missing = true;
                                continue;
                            }
                        }
                    }
                }
                _ => continue,
            };
            sources.push(QbiGrossSource {
                parent: entry.id,
                component: None,
                block: None,
                gross,
                qualified,
            });
        }
        sources.sort_by_key(|v| (v.parent, v.component, v.block));
        let weights: Vec<_> =
            sources.iter().flat_map(|v| [v.qualified, v.gross - v.qualified]).collect();
        if half > 0 && !owner_missing && weights.iter().all(|weight| *weight == 0) {
            return Err(QbiGrossError::UnavailableWeight(
                crate::IssueContext::owner(crate::TaxSection::PassThrough, owner).manual(),
            ));
        }
        let shares = crate::kernel::ratio::allocate_remaining_exact(&exact::whole(half), &weights)
            .ok_or_else(|| {
                QbiGrossError::Inconsistent(
                    crate::IssueContext::owner(crate::TaxSection::PassThrough, owner).manual(),
                )
            })?;
        for (source, shares) in sources.into_iter().zip(shares.chunks_exact(2)) {
            *result.sources.entry(source.parent).or_default() += shares[0];
            if let Some(block) = source.block {
                result.native_blocks.insert((source.parent, block), shares[0]);
            }
        }
    }
    if missing.is_empty() { Ok(result) } else { Err(QbiGrossError::Missing(missing)) }
}

#[derive(Debug)]
pub(crate) enum ExcessError {
    Source(crate::section1231::SourceError),
    Rule((&'static str, &'static str)),
    Prepared(crate::Section179PreparedIssue),
}
impl From<(&'static str, &'static str)> for ExcessError {
    fn from(issue: (&'static str, &'static str)) -> Self {
        Self::Rule(issue)
    }
}

#[cfg(test)]
pub(crate) fn loss_only_excess(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut [QbiActivity],
    excess: i64,
) -> Result<i64, ExcessError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    loss_only_excess_with_claims(draft, rental, activities, excess, &source_claims)
}

#[cfg(test)]
pub(crate) fn loss_only_excess_with_claims(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut [QbiActivity],
    excess: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, ExcessError> {
    loss_only_excess_with_section179_with_claims(
        draft,
        rental,
        activities,
        excess,
        None,
        None,
        source_claims,
    )
}

// Form 8995: only the qualified fraction of section 461(l) suspension leaves
// current QBI. Reuse the shared proportional allocator after earlier limits.

pub(crate) fn loss_only_excess_with_section179_with_claims(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut [QbiActivity],
    excess: i64,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, ExcessError> {
    let mut count = 0;
    let mut direct_loss = 0;
    let mut foreign_profit = 0;
    let mut loss_sources = 0;
    for entry in &draft.income {
        match &entry.income {
            crate::FederalIncome::Partnership { data } => {
                count += 1;
                if data.regular.ordinary > 0
                    || data.guaranteed_se() != 0
                    || data.guaranteed_capital != 0
                    || data.trade_kind != crate::ScorpTradeKind::Ordinary
                    || !data.complete()
                {
                    return Err(ExcessError::Rule((
                        "ISSUE_PARTNERSHIP_QBI_PARTNERSHIP_EXCESS_LOSS_ALLOCATION_WITH_PROFITS__GUARANTEED_CFAD08F9",
                        crate::fixture_english!(
                            "Partnership excess-loss allocation with profits, guaranteed payments or special business character remains unresolved."
                        ),
                    )));
                }
            }
            crate::FederalIncome::Business { data } => {
                let expenses = crate::section179_return::expenses_with_claims(
                    draft,
                    entry.id,
                    data,
                    selected,
                    home,
                    source_claims,
                )
                .map(|expense| {
                    expense
                        - selected
                            .map_or(0, |allocation| allocation.ordinary_limit_adjustment(entry.id))
                })
                .map_err(ExcessError::Prepared)?;
                let net = dollars(data.receipts() - expenses);
                // A business run abroad is resolved when its owner has no
                // Form 2555 record, so nothing of it is excluded.
                let source_resolved = match &data.source {
                    crate::BusinessSource::UnitedStates => true,
                    crate::BusinessSource::ForeignServices { confirmed, .. } => {
                        *confirmed
                            && crate::extended_rules::foreign_claim(
                                draft,
                                usize::from(entry.owner == crate::DocumentOwner::Spouses),
                            )
                            .is_none()
                    }
                };
                if !source_resolved
                    || !matches!(data.activity, crate::BusinessActivity::Ordinary { .. })
                    || (net > 0 && data.source == crate::BusinessSource::UnitedStates)
                    || !matches!(data.health_insurance, crate::BusinessHealthInsurance::None)
                    || !matches!(data.retirement, crate::BusinessRetirement::None)
                {
                    return Err(ExcessError::Rule((
                        "ISSUE_PARTNERSHIP_QBI_MIXED_PARTNERSHIP_SCHEDULE_C_EXCESS_LOSSES_REQUIRE_92B2AD27",
                        crate::fixture_english!(
                            "Mixed partnership/Schedule C excess losses require ordinary loss-only sources with resolved domestic QBI or wholly foreign nonqualified character, without separate foreign exclusions, housing, health or retirement deductions."
                        ),
                    )));
                }
                if net > 0 {
                    foreign_profit += net;
                } else {
                    direct_loss -= net;
                    loss_sources += usize::from(net < 0);
                }
            }
            crate::FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                if data.section1231.is_some() {
                    return Err(ExcessError::Rule((
                        "ISSUE_PARTNERSHIP_QBI_MIXED_PARTNERSHIP_S_CORPORATION_PROPERTY_CHARACTER_NEEDS_SEPAR_A57C0231",
                        crate::fixture_english!(
                            "Mixed partnership/S-corporation property character needs separate allocation."
                        ),
                    )));
                }
                // Reuse the S-corporation source checks without mutating live QBI.
                let mut probe = [QbiActivity {
                    source_wages_cents: Some(0),
                    source_property_cents: Some(0),
                    source_income_cents: Some(0),
                    owner: None,
                    income: 0,
                    wages: 0,
                    property: 0,
                    active: false,
                    service: Some(false),
                    group: None,
                }];
                crate::scorp_qbi::loss_only_business_excess(&[data], &mut probe, 0, 0)?;
                let net = data.activity_result()?.nonpassive;
                direct_loss -= dollars(net);
                loss_sources += usize::from(net < 0);
            }
            crate::FederalIncome::Rental { .. } | crate::FederalIncome::FarmRental { .. } => {
                return Err(ExcessError::Rule((
                    "ISSUE_PARTNERSHIP_QBI_PARTNERSHIP_EXCESS_LOSS_ALLOCATION_MIXED_WITH_OTHER_9CC01703",
                    crate::fixture_english!(
                        "Partnership excess-loss allocation mixed with other business sources remains unresolved."
                    ),
                )));
            }
            _ => {}
        }
    }
    if !rental.inputs_complete
        || count == 0
        || count != rental.partners.len()
        || rental.partners.iter().any(|v| v.net > 0)
    {
        return Err(ExcessError::Rule((
            "ISSUE_PARTNERSHIP_QBI_RESOLVE_ALL_PARTNERSHIP_LOSS_ONLY_SOURCES_BEFORE_563FC7D4",
            crate::fixture_english!(
                "Resolve all partnership loss-only sources before excess-loss allocation."
            ),
        )));
    }
    let loss = direct_loss - rental.partners.iter().map(|v| v.net).sum::<i64>();
    loss_sources += rental.partners.iter().filter(|v| v.net < 0).count();
    // A single loss source retains its established qualified fraction when
    // partially suspended (Form 8995). Multiple mixed-character sources still
    // require an allocation of the profit offset between their losses, unless
    // all losses are qualified or all are nonqualified.
    let qualified_loss = activities.iter().map(|v| -v.income).sum::<i64>();
    if foreign_profit > 0 && loss_sources != 1 && qualified_loss != 0 && qualified_loss != loss {
        return Err(ExcessError::Rule((
            "ISSUE_PARTNERSHIP_QBI_FOREIGN_PROFIT_OFFSETS_WITH_MIXED_QUALIFIED_NONQUALIFIED_663081D3",
            crate::fixture_english!(
                "Foreign profit offsets with mixed qualified/nonqualified losses require further allocation; uniformly qualified or nonqualified loss sources retain their character."
            ),
        )));
    }
    crate::scorp_qbi::defer_loss_only_qbi(activities, loss, excess).map_err(ExcessError::Rule)
}

// Current profit/loss netting uses the sole loss source's qualified fraction,
// or the common fraction when every source has the same qualified fraction. Keep historical releases and partner deductions outside
// this adapter until their separate gross/net allocation is established.
pub(crate) fn current_profit_excess(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut [QbiActivity],
    excess: i64,
) -> Result<i64, (&'static str, &'static str)> {
    let error = (
        "ISSUE_PARTNERSHIP_QBI_PROFIT_OFFSET_PARTNERSHIP_EXCESS_LOSSES_REQUIRE_CURRENT_97A91752",
        crate::fixture_english!(
            "Profit-offset partnership excess losses require current ordinary nonpassive sources with a single loss source or the same qualified fraction, no historical losses, partner deductions, guaranteed payments or other businesses."
        ),
    );
    if !rental.inputs_complete
        || rental.partners.is_empty()
        || rental.partners.len() != activities.len()
        || draft.income.iter().any(|e| {
            matches!(
                e.income,
                crate::FederalIncome::Business { .. }
                    | crate::FederalIncome::Rental { .. }
                    | crate::FederalIncome::FarmRental { .. }
                    | crate::FederalIncome::Scorp { .. }
            )
        })
    {
        return Err(error);
    }
    let mut ratio = None;
    let mut uniform = true;
    let mut losses = Vec::new();
    let mut indices = Vec::new();
    let mut total_loss = 0;
    let mut total_net = 0;
    for (index, (pool, activity)) in rental.partners.iter().zip(activities.iter()).enumerate() {
        let Some(entry) = draft.income.iter().find(|e| e.id == pool.id) else {
            return Err(error);
        };
        let crate::FederalIncome::Partnership { data } = &entry.income else {
            return Err(error);
        };
        let source_net = rental
            .k1_amounts
            .iter()
            .find(|amount| amount.id == pool.id)
            .and_then(|amount| amount.source_net_cents)
            .ok_or(error)?;
        let source_qbi = activity.source_income_cents.ok_or(error)?;
        if !data.complete()
            || data.trade_kind != crate::ScorpTradeKind::Ordinary
            || data.material_for_losses() != crate::participation::Participation::Yes
            || data.regular.basis.prior_loss != 0
            || data.regular.risk.prior_loss != 0
            || data.regular.passive_loss != 0
            || data.guaranteed_services != 0
            || data.guaranteed_capital != 0
            || data.reported_se != Some(0)
            || data.pensions != 0
            || data.retirement != crate::PartnerRetirement::None
            || !matches!(data.health_insurance, crate::BusinessHealthInsurance::None)
            || activity.service != Some(false)
            || (source_net < 0 && !(source_net..=0).contains(&source_qbi))
            || (source_net >= 0 && !(0..=source_net).contains(&source_qbi))
        {
            return Err(error);
        }
        total_net += pool.net;
        if source_net != 0 {
            let ordinary = source_net.abs();
            let qualified = source_qbi.abs();
            if let Some((old_total, old_qualified)) = ratio {
                if ordinary as i128 * old_qualified as i128 != qualified as i128 * old_total as i128
                {
                    uniform = false;
                }
            } else {
                ratio = Some((ordinary, qualified));
            }
        }
        if source_net < 0 {
            total_loss -= pool.net;
            losses.push(activity.clone());
            indices.push(index);
        }
    }
    if (!uniform && losses.len() != 1) || excess < 0 || excess > phase::floor(-total_net, 0) {
        return Err(error);
    }
    let deferred = crate::scorp_qbi::defer_loss_only_qbi(&mut losses, total_loss, excess)?;
    for (index, loss) in indices.into_iter().zip(losses) {
        activities[index] = loss;
    }
    Ok(deferred)
}

// A single partnership with exactly one gross loss character offset by the
// other character's gain. Current and retained qualification come from the
// existing owners before the net business activity is used for QBI.
#[cfg(test)]
pub(crate) fn excess_with_property(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut Vec<QbiActivity>,
    excess: i64,
    taxable: i64,
    status: FederalStatus,
) -> Result<i64, crate::section1231::SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    excess_with_property_with_claims(
        draft,
        rental,
        activities,
        excess,
        taxable,
        status,
        &source_claims,
    )
}

pub(crate) fn excess_with_property_with_claims(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut Vec<QbiActivity>,
    excess: i64,
    taxable: i64,
    status: FederalStatus,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::section1231::SourceError> {
    let error = (
        "ISSUE_PARTNER_EXCESS_LOSS_QBI_AND_PROPERTY_CHARACTER",
        crate::fixture_english!(
            "Resolve partnership QBI and property character before excess-loss allocation."
        ),
    );
    if !rental.inputs_complete
        || rental.partners.len() != 1
        || draft.income.iter().any(|entry| {
            !matches!(
                entry.income,
                crate::FederalIncome::Partnership { .. }
                    | crate::FederalIncome::Wage { .. }
                    | crate::FederalIncome::Interest { .. }
                    | crate::FederalIncome::Dividend { .. }
                    | crate::FederalIncome::Sale { .. }
            ) && !matches!(&entry.income, crate::FederalIncome::Broker { data }
                if crate::broker_calculation::known_components(data, true, false))
        })
    {
        return Err(error.into());
    }
    let mut sources = draft.income.iter().filter_map(|entry| match &entry.income {
        crate::FederalIncome::Partnership { data } => Some((entry.id, entry.owner, data)),
        _ => None,
    });
    let (id, filer, owner) = sources.next().ok_or(error)?;
    if sources.next().is_some() || rental.partners[0].id != id {
        return Err(error.into());
    }
    let character = crate::section1231::partner_loss_with_gain(owner, &owner.regular)?;
    let property = crate::section1231::partner_property_qbi_items_with_claims(
        draft,
        rental,
        id,
        Some((taxable, status)),
        source_claims,
    )?;
    let mut expected = Vec::new();
    let (current, historical, current_index) = if let Some(statement) = &owner.qbi {
        let ordinary = statement.calculate_combined(
            owner,
            rental.partners[0].suspended,
            0,
            (taxable, status),
            &Default::default(),
            &Default::default(),
        )?;
        let combined = statement.calculate_combined(
            owner,
            rental.partners[0].suspended,
            0,
            (taxable, status),
            &Default::default(),
            &property,
        )?;
        if combined.historical_cents.get() > 0 {
            expected.push(QbiActivity {
                source_wages_cents: Some(0),
                source_property_cents: Some(0),
                source_income_cents: Some(-combined.historical_cents.get()),
                owner: None,
                income: -combined.historical,
                wages: 0,
                property: 0,
                active: false,
                service: Some(false),
                group: None,
            });
        }
        let current_index = expected.len();
        expected.push(combined.current);
        let (current, historical) = match character {
            crate::section1231::ResolvedLoss::Operating(_) => {
                (ordinary.current_loss_cents.get(), ordinary.historical_cents.get())
            }
            crate::section1231::ResolvedLoss::Property(_) => {
                (property.current_loss_cents.get(), property.historical_loss_cents.get())
            }
        };
        (current, historical, Some(current_index))
    } else {
        let historical = historical_without_statement(owner, rental.partners[0].suspended)?
            .map_or(0, |v| v.historical.get());
        if historical > 0 {
            expected.push(QbiActivity {
                source_wages_cents: Some(0),
                source_property_cents: Some(0),
                source_income_cents: Some(-historical),
                owner: None,
                income: -dollars(historical),
                wages: 0,
                property: 0,
                active: false,
                service: Some(false),
                group: None,
            });
        }
        (0, historical, None)
    };
    // These are the same source rows the central owner tagged before the
    // financial fingerprint check. Keep their metadata when replacing them.
    for activity in &mut expected {
        activity.owner = Some(filer.owned());
    }
    if *activities != expected {
        return Err(error.into());
    }
    let deferred = crate::scorp_qbi::defer_source_qbi(
        &mut expected,
        current_index,
        dollars(character.amount().get()),
        excess,
        Amount::new(current + historical).expect("qualified source loss"),
        Amount::new(current).expect("current source loss"),
        character.amount(),
    )?;
    *activities = expected;
    Ok(deferred)
}

#[cfg(test)]
pub(crate) fn allocate_excess(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut Vec<QbiActivity>,
    excess: i64,
    taxable: i64,
    status: FederalStatus,
) -> Result<i64, ExcessError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    allocate_excess_with_claims(draft, rental, activities, excess, taxable, status, &source_claims)
}

#[cfg(test)]
pub(crate) fn allocate_excess_with_claims(
    draft: &crate::FederalDraft,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut Vec<QbiActivity>,
    excess: i64,
    taxable: i64,
    status: FederalStatus,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, ExcessError> {
    allocate_excess_with_section179_with_claims(
        crate::related_forms::RelatedFormContext::original(draft, source_claims),
        rental,
        activities,
        excess,
        taxable,
        status,
        None,
    )
}

pub(crate) fn allocate_excess_with_section179_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    rental: &crate::extended_rules::RentalResult,
    activities: &mut Vec<QbiActivity>,
    excess: i64,
    taxable: i64,
    status: FederalStatus,
    selected: Option<&crate::Section179PreparedAllocation>,
) -> Result<i64, ExcessError> {
    let crate::related_forms::RelatedFormContext { draft, claims: source_claims, .. } =
        form_context;
    // Partnership and S-corporation activities already have eligibility fixed.
    // Resolve remaining direct-business SSTB fractions once on a candidate;
    // rejected source allocation must not alter live activities.
    let mut candidate = activities
        .iter()
        .map(|v| crate::business_rules::eligible_qbi_activity(v, taxable, status))
        .collect::<Result<Vec<_>, _>>()?;
    let qualified = match loss_only_excess_with_section179_with_claims(
        draft,
        rental,
        &mut candidate,
        excess,
        selected,
        None,
        source_claims,
    ) {
        Ok(qualified) => qualified,
        Err(ExcessError::Rule(_)) => current_profit_excess(draft, rental, &mut candidate, excess)?,
        // A failed selected operand is not a financial classification and
        // must not enter the alternate financial rule.
        Err(issue @ (ExcessError::Prepared(_) | ExcessError::Source(_))) => return Err(issue),
    };
    *activities = candidate;
    Ok(qualified)
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerQbiStatement {
    amounts: [income, wages, property, section1231_qbi, reit_dividends, ptp_income],
    gains: [],
    facts: [service],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerQbiHistory {
    amounts: [basis, risk, passive],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerQbiRecords {
    amounts: [qualified_se_gross, history],
    gains: [],
    facts: [],
});
