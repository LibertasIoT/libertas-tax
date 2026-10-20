// Per-instrument annual amounts: IRC 171, 1272, 1273 and 1286;
// Pub. 550 (2025) pp. 48–50; Pub. 1212 (Rev. December 2025) pp. 8–18.
// These are documented annual tax amounts, not a duplicate yield engine.
use super::DebtReporting;
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use crate::year_constants::{TAX_YEAR_END_DATE, TAX_YEAR_START_DATE};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Annual debt tax components in their original reporting characters
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DebtComponents {
    /// Non-Treasury taxable interest, Form 1099-INT box 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// U.S.-obligation interest, Form 1099-INT box 3
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Non-Treasury OID, Form 1099-OID box 1
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub oid: i64,
    /// Periodic coupon interest, Form 1099-OID box 2
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_periodic_interest: i64,
    /// Treasury OID, including a signed TIPS deflation adjustment
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub treasury_oid: i64,
    /// Exempt interest or OID, excluding any taxable component
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private-activity component within exempt income
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Foreign income tax paid belonging to this instrument
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
    /// Market discount included currently as interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Current taxable premium amortization on non-Treasury interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Current taxable premium amortization on Treasury interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,
    /// Mandatory premium amortization on exempt interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Acquisition premium reducing non-Treasury OID
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub acquisition_premium: i64,
    /// Acquisition premium reducing Treasury OID
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub treasury_acquisition_premium: i64,
    /// Purchased accrued non-Treasury interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub accrued_interest: i64,
    /// Purchased accrued Treasury interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub treasury_accrued_interest: i64,
}

/// Computation underlying the documented annual amounts
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
pub enum AnnualDebtTreatment {
    /// Regular debt instrument with a complete annual tax worksheet
    #[default]
    Regular,
    /// Stripped bond or coupon with its own complete annual tax worksheet
    Stripped,
    /// Contingent-payment computation needing comparable yield and payment adjustments
    /// Known required income cannot be guessed from a generic OID amount.
    Contingent,
}

/// Same instrument's documented taxable-premium history
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PremiumHistory {
    /// Prior accrual periods’ net interest inclusions on this instrument
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_net_interest_inclusions: i64,
    /// Prior bond-premium deductions on this instrument
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_premium_deductions: i64,
    /// Undeducted taxable premium carried into this period
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub premium_carry: i64,
}

/// Actual taxable-premium offset already applied to retained net income
/// AFTER nominee and after-death allocations. This is the amount actually offset, not total annual amortization or unused premium. Enter a documented zero only when no taxable premium was applied.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DebtPremiumOffset {
    /// Applied offset within retained ordinary interest or non-Treasury OID
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub ordinary: i64,
    /// Applied offset within retained U.S.-obligation interest or Treasury OID
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub treasury: i64,
}

/// One instrument's documented annual tax amounts
/// Keep ordinary, Treasury and exempt instruments in separate rows. Do not enter purchase premium as annual amortization.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct AnnualDebtRecord {
    /// Optional original fixed-principal issue terms for the de-minimis OID test
    /// Leave absent when this is not that ordinary original-issue classification. Annual amounts still come from the actual broker or retained worksheet.
    /// [NullText]
    /// Use the reported annual amounts without an original-issue terms worksheet
    /// [SetText]
    /// The original fixed-principal issue terms are
    #[libertas_default($null)]
    pub original_issue: Option<OriginalDebtIssue>,

    /// Stable accepted instrument identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Instrument name or identifier from your records
    #[libertas_size(min = 1)]
    pub instrument: String,
    /// Computation underlying this annual amount
    #[libertas_default(Regular)]
    pub treatment: AnnualDebtTreatment,
    /// Whether reported income is already reduced by premium
    #[libertas_default(Gross)]
    pub reporting: DebtReporting,
    /// Documented taxable-premium offset already applied to this instrument’s retained net amount
    /// Needed to restore gross income when the whole-return taxable-premium election is not Yes.
    /// [NullText]
    /// No taxable-premium offset has been documented for this instrument's net amount
    /// [SetText]
    /// The instrument's documented taxable-premium offset is
    pub net_premium_offset: Option<DebtPremiumOffset>,
    /// Reported annual amounts, before any outside-owner allocation
    pub amounts: DebtComponents,
    /// Documented components legally belonging to a nominee owner
    /// [NullText]
    /// No reported components belong to a nominee owner
    /// [SetText]
    /// The components belonging to a nominee owner are
    pub nominee: Option<DebtComponents>,
    /// Documented components belonging to the recipient after the owner's 2026 death
    /// [NullText]
    /// No reported components belong to a recipient after the owner's death
    /// [SetText]
    /// The components belonging to the recipient after the owner's death are
    pub after_death: Option<DebtComponents>,
    /// Retained savings-bond interest included on an earlier return
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub previously_reported: i64,
    /// Same-bond history for an excess-premium deduction or carried premium
    /// Leave absent if unknown; known interest remains and only uncomputed relief is omitted.
    /// [NullText]
    /// No supported excess-premium deduction or carried premium is claimed
    /// [SetText]
    /// The same-bond premium history is
    #[libertas_default($null)]
    pub premium_history: Option<PremiumHistory>,
    /// Sale, retirement or full redemption date, if applicable
    /// [NullText]
    /// I am not adding a date for terminal-premium review
    /// [SetText]
    /// The instrument was sold, retired or fully redeemed on
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    #[libertas_default($null)]
    pub disposed_on: Option<u32>,
    /// Documented retained private-activity amount after mandatory premium
    /// Required when exempt premium reduces a private-activity component; never infer a proportional allocation.
    /// [NullText]
    /// No private-activity component needs an allocation after exempt-bond premium
    /// [SetText]
    /// The documented private-activity amount after mandatory premium is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub private_activity_after_premium: Option<i64>,
}

/// Original issue terms for an ordinary taxable fixed-principal obligation
/// Not for inflation-indexed, tax-exempt, stripped or installment-principal debt.
/// The source's original term determines the threshold, not your holding period.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OriginalDebtIssue {
    /// Original issue price, from issuer records
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub issue_price: i64,
    /// Stated redemption price at maturity
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub redemption_price: i64,
    /// Complete original years to maturity
    #[libertas_number(min = 0)]
    pub complete_years: u32,
    /// An actual election includes otherwise de-minimis OID currently
    /// Relevant only below the threshold. Check the retained election and issuer's original-issue statement. No uses the ordinary zero-OID rule; current annual OID must agree with the actual election and source records.
    pub elect_de_minimis_oid: bool,
}
impl OriginalDebtIssue {
    pub(crate) fn de_minimis(&self) -> bool {
        // Strict comparison: equality is not de minimis. Both sides use cents.
        (self.redemption_price - self.issue_price) * 400
            < self.redemption_price * i64::from(self.complete_years)
    }
    fn issue(&self) -> Option<ValidationIssue> {
        let field = if !(1..=100_000_000_000).contains(&self.redemption_price) {
            Some("redemption_price")
        } else if !(0..=self.redemption_price).contains(&self.issue_price) {
            Some("issue_price")
        } else if self.complete_years > 100 {
            Some("complete_years")
        } else {
            None
        };
        field.map(|field| ValidationIssue::new("DEBT_ORIGINAL_ISSUE_INVALID").at_field(field))
    }
}

/// Instrument records backing this payer statement
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct AnnualDebtRecords {
    /// Annual instrument worksheets
    /// ----
    /// Instrument
    #[libertas_size(min = 1)]
    pub records: Vec<AnnualDebtRecord>,
}

impl DebtComponents {
    fn values(&self) -> [i64; 16] {
        [
            self.taxable,
            self.us_obligations,
            self.oid,
            self.other_periodic_interest,
            self.treasury_oid,
            self.exempt,
            self.private_activity,
            self.foreign_tax,
            self.market_discount,
            self.bond_premium,
            self.treasury_bond_premium,
            self.tax_exempt_bond_premium,
            self.acquisition_premium,
            self.treasury_acquisition_premium,
            self.accrued_interest,
            self.treasury_accrued_interest,
        ]
    }
    fn subtract(&mut self, other: &Self) {
        self.taxable -= other.taxable;
        self.us_obligations -= other.us_obligations;
        self.oid -= other.oid;
        self.other_periodic_interest -= other.other_periodic_interest;
        self.treasury_oid -= other.treasury_oid;
        self.exempt -= other.exempt;
        self.private_activity -= other.private_activity;
        self.foreign_tax -= other.foreign_tax;
        self.market_discount -= other.market_discount;
        self.bond_premium -= other.bond_premium;
        self.treasury_bond_premium -= other.treasury_bond_premium;
        self.tax_exempt_bond_premium -= other.tax_exempt_bond_premium;
        self.acquisition_premium -= other.acquisition_premium;
        self.treasury_acquisition_premium -= other.treasury_acquisition_premium;
        self.accrued_interest -= other.accrued_interest;
        self.treasury_accrued_interest -= other.treasury_accrued_interest;
    }
}

impl AnnualDebtRecord {
    pub(crate) fn retained(&self) -> DebtComponents {
        let mut v = self.amounts.clone();
        if let Some(part) = &self.nominee {
            v.subtract(part);
        }
        if let Some(part) = &self.after_death {
            v.subtract(part);
        }
        v.us_obligations -= self.previously_reported;
        v
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if self.net_premium_offset.as_ref().is_some_and(|o| !o.valid()) {
            return Some(
                ValidationIssue::new("DEBT_COMPONENT_RANGE").at_field("net_premium_offset"),
            );
        }
        if self.reporting != DebtReporting::Net && self.net_premium_offset.is_some() {
            return Some(
                ValidationIssue::new("DEBT_OFFSET_ONLY_FOR_NET").at_field("net_premium_offset"),
            );
        }
        let n = self.nominee.clone().unwrap_or_default();
        let a = self.after_death.clone().unwrap_or_default();
        if self.instrument.trim().is_empty() || self.id < 0 {
            return Some(ValidationIssue::new("DEBT_INSTRUMENT_INVALID").at_field("instrument"));
        }
        for (i, ((whole, n), a)) in
            self.amounts.values().into_iter().zip(n.values()).zip(a.values()).enumerate()
        {
            let field = [
                "taxable",
                "us_obligations",
                "oid",
                "other_periodic_interest",
                "treasury_oid",
                "exempt",
                "private_activity",
                "foreign_tax",
                "market_discount",
                "bond_premium",
                "treasury_bond_premium",
                "tax_exempt_bond_premium",
                "acquisition_premium",
                "treasury_acquisition_premium",
                "accrued_interest",
                "treasury_accrued_interest",
            ][i];
            for (owner, amount) in [("amounts", whole), ("nominee", n), ("after_death", a)] {
                if !(-100_000_000_000..=100_000_000_000).contains(&amount) || (i != 4 && amount < 0)
                {
                    return Some(
                        ValidationIssue::new("DEBT_COMPONENT_RANGE")
                            .at_field(field)
                            .in_field(owner),
                    );
                }
            }
            if !super::signed_share(whole, n)
                || !super::signed_share(whole, a)
                || !super::signed_share(whole, n + a)
            {
                let field = [
                    "taxable",
                    "us_obligations",
                    "oid",
                    "other_periodic_interest",
                    "treasury_oid",
                    "exempt",
                    "private_activity",
                    "foreign_tax",
                    "market_discount",
                    "bond_premium",
                    "treasury_bond_premium",
                    "tax_exempt_bond_premium",
                    "acquisition_premium",
                    "treasury_acquisition_premium",
                    "accrued_interest",
                    "treasury_accrued_interest",
                ][i];
                return Some(
                    ValidationIssue::new("DEBT_OUTSIDE_COMPONENT_OVERLAP")
                        .at_field(field)
                        .in_field("amounts"),
                );
            }
        }
        if !(0..=self.amounts.us_obligations - n.us_obligations - a.us_obligations)
            .contains(&self.previously_reported)
        {
            return Some(
                ValidationIssue::new("DEBT_PRIOR_ABOVE_RETAINED").at_field("previously_reported"),
            );
        }
        let v = self.retained();
        for (owner, part) in
            [("amounts", &self.amounts), ("nominee", &n), ("after_death", &a), ("amounts", &v)]
        {
            if part.private_activity > part.exempt {
                return Some(
                    ValidationIssue::new("DEBT_PRIVATE_ABOVE_EXEMPT")
                        .at_field("private_activity")
                        .in_field(owner),
                );
            }
        }
        let source = &self.amounts;
        let ordinary = source.taxable
            + source.oid
            + source.other_periodic_interest
            + source.bond_premium
            + source.acquisition_premium
            + source.accrued_interest
            > 0;
        let treasury = source.us_obligations != 0
            || source.treasury_oid != 0
            || source.treasury_bond_premium != 0
            || source.treasury_acquisition_premium != 0
            || source.treasury_accrued_interest != 0;
        let exempt = source.exempt + source.tax_exempt_bond_premium > 0;
        if (ordinary as u8 + treasury as u8 + exempt as u8) > 1 {
            return Some(
                ValidationIssue::new("DEBT_MIXED_INSTRUMENT_CHARACTERS").at_field("amounts"),
            );
        }
        if exempt && self.premium_history.is_some() {
            return Some(
                ValidationIssue::new("DEBT_EXEMPT_PREMIUM_HISTORY").at_field("premium_history"),
            );
        }
        if self.net_premium_offset.as_ref().is_some_and(|offset| {
            (exempt && offset.ordinary + offset.treasury > 0)
                || (ordinary && offset.treasury > 0)
                || (treasury && offset.ordinary > 0)
                || (offset.ordinary > 0 && offset.treasury > 0)
        }) {
            return Some(
                ValidationIssue::new("DEBT_NET_OFFSET_CHARACTER").at_field("net_premium_offset"),
            );
        }
        if self.reporting == DebtReporting::Gross {
            for (owner, part) in [("nominee", &n), ("after_death", &a), ("amounts", &v)] {
                if part.tax_exempt_bond_premium > part.exempt {
                    return Some(
                        ValidationIssue::new("DEBT_EXEMPT_REDUCTION_ABOVE_AMOUNT")
                            .at_field("tax_exempt_bond_premium")
                            .in_field(owner),
                    );
                }
            }
        }
        if let Some(history) = &self.premium_history {
            if [
                history.prior_net_interest_inclusions,
                history.prior_premium_deductions,
                history.premium_carry,
            ]
            .iter()
            .any(|x| !(0..=100_000_000_000).contains(x))
            {
                return Some(
                    ValidationIssue::new("DEBT_COMPONENT_RANGE").at_field("premium_history"),
                );
            }
            if history.prior_premium_deductions > history.prior_net_interest_inclusions {
                return Some(
                    ValidationIssue::new("DEBT_HISTORY_DEDUCTIONS_ABOVE_INCLUSIONS")
                        .at_field("prior_premium_deductions")
                        .in_field("premium_history"),
                );
            }
        }
        if self.disposed_on.is_some_and(|date| {
            !(TAX_YEAR_START_DATE..=TAX_YEAR_END_DATE).contains(&date)
                || !crate::federal_rules::valid_date(date)
        }) {
            return Some(
                ValidationIssue::new("DEBT_DISPOSAL_DATE_INVALID").at_field("disposed_on"),
            );
        }
        if let Some(issue) = &self.original_issue {
            if let Some(issue) = issue.issue() {
                return Some(issue.in_field("original_issue"));
            }
            if self.treatment != AnnualDebtTreatment::Regular {
                return Some(
                    ValidationIssue::new("DEBT_ORIGINAL_ISSUE_INVALID").at_field("treatment"),
                );
            }
            if self.amounts.exempt != 0 {
                return Some(
                    ValidationIssue::new("DEBT_ORIGINAL_ISSUE_INVALID")
                        .at_field("exempt")
                        .in_field("amounts"),
                );
            }
            if issue.de_minimis()
                && !issue.elect_de_minimis_oid
                && (v.oid != 0 || v.treasury_oid != 0)
            {
                return Some(
                    ValidationIssue::new("DEBT_DE_MINIMIS_OID_CONFLICT")
                        .at_field("elect_de_minimis_oid")
                        .in_field("original_issue"),
                );
            }
        }
        let after_exempt = if self.reporting == DebtReporting::Gross {
            v.exempt - v.tax_exempt_bond_premium
        } else {
            v.exempt
        };
        if self
            .private_activity_after_premium
            .is_some_and(|x| x < 0 || x > v.private_activity || x > after_exempt)
            || (v.tax_exempt_bond_premium > 0
                && v.private_activity > 0
                && self.private_activity_after_premium.is_none())
        {
            return Some(
                ValidationIssue::new("DEBT_PRIVATE_AFTER_PREMIUM_INVALID")
                    .at_field("private_activity_after_premium"),
            );
        }
        None
    }
    // Signed Treasury adjustments remain signed until the payer-level netting.
    pub(crate) fn taxable(&self) -> i64 {
        let v = self.retained();
        let ordinary = v.taxable + v.oid + v.other_periodic_interest;
        let treasury = v.us_obligations + v.treasury_oid;
        if self.reporting == DebtReporting::Net {
            return ordinary + treasury + v.market_discount;
        }
        let carry = self.premium_history.as_ref().map_or(0, |x| x.premium_carry);
        let ordinary_carry = if v.bond_premium > 0 || ordinary > 0 { carry } else { 0 };
        let treasury_carry = carry - ordinary_carry;
        crate::kernel::ratio::positive(
            ordinary - v.acquisition_premium - v.accrued_interest - v.bond_premium - ordinary_carry,
        ) + if treasury < 0 {
            treasury
        } else {
            crate::kernel::ratio::positive(
                treasury
                    - v.treasury_acquisition_premium
                    - v.treasury_accrued_interest
                    - v.treasury_bond_premium
                    - treasury_carry,
            )
        } + v.market_discount
    }
    pub(crate) fn exempt(&self) -> i64 {
        let v = self.retained();
        v.exempt
            - if self.reporting == DebtReporting::Gross { v.tax_exempt_bond_premium } else { 0 }
    }
    pub(crate) fn private_activity(&self) -> i64 {
        self.private_activity_after_premium.unwrap_or_else(|| self.retained().private_activity)
    }
    pub(crate) fn excess_deduction(&self) -> i64 {
        if self.reporting == DebtReporting::Net {
            return 0;
        }
        let Some(h) = &self.premium_history else {
            return 0;
        };
        let v = self.retained();
        if v.exempt + v.tax_exempt_bond_premium > 0 {
            return 0;
        }
        let before_premium = crate::kernel::ratio::positive(
            v.taxable + v.oid + v.other_periodic_interest + v.us_obligations + v.treasury_oid
                - v.acquisition_premium
                - v.treasury_acquisition_premium
                - v.accrued_interest
                - v.treasury_accrued_interest,
        );
        phase::cap(
            crate::kernel::ratio::positive(
                v.bond_premium + v.treasury_bond_premium + h.premium_carry - before_premium,
            ),
            [h.prior_net_interest_inclusions - h.prior_premium_deductions],
        )
    }
}

impl AnnualDebtRecords {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if self.records.is_empty() || self.records.len() > 100 {
            return Some(ValidationIssue::new("DEBT_INSTRUMENT_INVALID").at_field("records"));
        }
        self.records
            .iter()
            .enumerate()
            .find_map(|(index, record)| record.issue().map(|issue| issue.in_list("records", index)))
    }
    pub(crate) fn taxable(&self) -> i64 {
        crate::kernel::ratio::positive(self.records.iter().map(AnnualDebtRecord::taxable).sum())
    }
    pub(crate) fn exempt(&self) -> i64 {
        self.records.iter().map(AnnualDebtRecord::exempt).sum()
    }
    pub(crate) fn private_activity(&self) -> i64 {
        self.records.iter().map(AnnualDebtRecord::private_activity).sum()
    }
    pub(crate) fn foreign_tax(&self) -> i64 {
        self.records.iter().map(|x| x.retained().foreign_tax).sum()
    }
}

pub(super) fn int_issue(
    v: &crate::FederalInterest,
    records: &AnnualDebtRecords,
) -> Option<ValidationIssue> {
    if let Some(issue) = records.issue() {
        return Some(issue.in_field("instrument_records").in_field("adjustments"));
    }
    if v.nominee.is_some()
        || v.after_death.is_some()
        || v.adjustments.as_ref().is_some_and(|a| {
            a.previously_reported != 0
                || a.reporting != DebtReporting::Gross
                || a.net_premium_offset.is_some()
        })
    {
        return Some(
            ValidationIssue::new("DEBT_PAYER_ALLOCATION_OVERLAP")
                .at_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if records.records.iter().map(|r| r.amounts.taxable).sum::<i64>() != v.taxable {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("taxable"));
    }
    if records.records.iter().map(|r| r.amounts.us_obligations).sum::<i64>() != v.us_obligations {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("us_obligations"),
        );
    }
    if records.records.iter().map(|r| r.amounts.exempt).sum::<i64>() != v.exempt {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("exempt"));
    }
    if records.records.iter().map(|r| r.amounts.private_activity).sum::<i64>() != v.private_activity
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("private_activity"),
        );
    }
    if records.records.iter().map(|r| r.amounts.foreign_tax).sum::<i64>() != v.foreign_tax {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("foreign_tax"),
        );
    }
    if records.records.iter().map(|r| r.amounts.market_discount).sum::<i64>() != v.market_discount {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("market_discount"),
        );
    }
    if records.records.iter().map(|r| r.amounts.bond_premium).sum::<i64>() != v.bond_premium {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("bond_premium"),
        );
    }
    if records.records.iter().map(|r| r.amounts.treasury_bond_premium).sum::<i64>()
        != v.treasury_bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("treasury_bond_premium"),
        );
    }
    if records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum::<i64>()
        != v.tax_exempt_bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("tax_exempt_bond_premium"),
        );
    }
    if records
        .records
        .iter()
        .map(|r| r.amounts.accrued_interest + r.amounts.treasury_accrued_interest)
        .sum::<i64>()
        != v.accrued_interest
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("accrued_interest"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.acquisition_premium != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("acquisition_premium")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.oid != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("oid")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.other_periodic_interest != 0)
    {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("other_periodic_interest")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) =
        records.records.iter().position(|r| r.amounts.treasury_acquisition_premium != 0)
    {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("treasury_acquisition_premium")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.treasury_oid != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("treasury_oid")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    None
}

pub(super) fn oid_issue(
    v: &crate::FederalOid,
    records: &AnnualDebtRecords,
) -> Option<ValidationIssue> {
    if let Some(issue) = records.issue() {
        return Some(issue.in_field("instrument_records").in_field("adjustments"));
    }
    if v.nominee.is_some()
        || v.adjustments.as_ref().is_some_and(|a| {
            a.previously_reported != 0
                || a.reporting != DebtReporting::Gross
                || a.net_premium_offset.is_some()
        })
    {
        return Some(
            ValidationIssue::new("DEBT_PAYER_ALLOCATION_OVERLAP")
                .at_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if records.records.iter().map(|r| r.amounts.oid).sum::<i64>() != v.oid {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("oid"));
    }
    if records.records.iter().map(|r| r.amounts.other_periodic_interest).sum::<i64>()
        != v.other_periodic_interest
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("other_periodic_interest"),
        );
    }
    if records.records.iter().map(|r| r.amounts.treasury_oid).sum::<i64>() != v.treasury_oid {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("treasury_oid"),
        );
    }
    if records.records.iter().map(|r| r.amounts.exempt).sum::<i64>() != v.exempt {
        return Some(ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("exempt"));
    }
    if records.records.iter().map(|r| r.amounts.private_activity).sum::<i64>() != v.private_activity
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("private_activity"),
        );
    }
    if records.records.iter().map(|r| r.amounts.market_discount).sum::<i64>() != v.market_discount {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("market_discount"),
        );
    }
    if records
        .records
        .iter()
        .map(|r| r.amounts.bond_premium + r.amounts.treasury_bond_premium)
        .sum::<i64>()
        != v.bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("bond_premium"),
        );
    }
    if records
        .records
        .iter()
        .map(|r| r.amounts.acquisition_premium + r.amounts.treasury_acquisition_premium)
        .sum::<i64>()
        != v.acquisition_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH").at_field("acquisition_premium"),
        );
    }
    if records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum::<i64>()
        != v.tax_exempt_bond_premium
    {
        return Some(
            ValidationIssue::new("DEBT_INSTRUMENT_TOTAL_MISMATCH")
                .at_field("tax_exempt_bond_premium"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.accrued_interest != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("accrued_interest")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.foreign_tax != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("foreign_tax")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.taxable != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("taxable")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) =
        records.records.iter().position(|r| r.amounts.treasury_accrued_interest != 0)
    {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("treasury_accrued_interest")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    if let Some(index) = records.records.iter().position(|r| r.amounts.us_obligations != 0) {
        return Some(
            ValidationIssue::new("DEBT_WRONG_STATEMENT_COMPONENT")
                .at_field("us_obligations")
                .in_field("amounts")
                .in_list("records", index)
                .in_field("instrument_records")
                .in_field("adjustments"),
        );
    }
    None
}

/// Source of a manually documented Schedule A bond-premium deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BondPremiumSource {
    /// A different documented bond, with its own deduction records
    OtherDocumented {
        /// Instrument name or description, not an accepted record identity
        #[libertas_size(min = 1)]
        instrument: String,
    },
    /// An annual instrument already accepted on Interest and dividends
    AcceptedAnnualInstrument {
        /// Accepted instrument
        instrument: i64,
    },
}

/// Accepted annual instrument that a manually documented deduction can name
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BondPremiumInstrument {
    /// Accepted instrument identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Payer and instrument
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Current annual excess deduction computed from this instrument
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub computed_deduction: i64,
}

impl DebtPremiumOffset {
    pub(crate) fn valid(&self) -> bool {
        (0..=100_000_000_000).contains(&self.ordinary)
            && (0..=100_000_000_000).contains(&self.treasury)
    }
}
