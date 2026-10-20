use crate::validation::ValidationIssue;
// Canonical projections of the supported calculation working copy.
// Ownership and adjustments are applied by their owners before this projection.
// IRC 6049; IRS 2026 draft Schedule B pp. B-1–B-3 and Pub.550 (2025) pp.25–27.
use crate::{FederalIncome, FederalIncomeEntry};
mod annual;
pub use annual::{
    AnnualDebtRecord, AnnualDebtRecords, AnnualDebtTreatment, BondPremiumInstrument,
    BondPremiumSource, DebtComponents, DebtPremiumOffset, OriginalDebtIssue, PremiumHistory,
};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct InvestmentComponents {
    pub interest: i64,
    pub exempt_interest: i64,
    pub private_activity: i64,
    pub ordinary_dividends: i64,
    pub qualified_dividends: i64,
    pub capital_distributions: i64,
    pub unrecaptured_distributions: i64,
    pub collectible_distributions: i64,
    pub section_1202_gain: i64,
    pub section199a: i64,
    pub foreign_tax: i64,
    pub withholding: i64,
}

// Call only on the central unknowns::supported working copy: accepted input
// identities/owners are unchanged, but death/nominee allocations have been applied.
// Return-level Form 8815 exclusion remains its owner's separate output.
pub(crate) fn components(entry: &FederalIncomeEntry) -> Option<InvestmentComponents> {
    let mut result = InvestmentComponents::default();
    match &entry.income {
        FederalIncome::Interest { data } => {
            result.interest = data.net_taxable();
            result.exempt_interest = data.net_exempt();
            result.private_activity = data
                .adjustments
                .as_ref()
                .and_then(|a| a.instrument_records.as_ref())
                .map_or(data.private_activity, |r| r.private_activity());
            result.foreign_tax = data
                .adjustments
                .as_ref()
                .and_then(|a| a.instrument_records.as_ref())
                .map_or(data.foreign_tax, |r| r.foreign_tax());
            result.withholding = data.withholding;
        }
        FederalIncome::Oid { data } => {
            result.interest = data.net_taxable();
            result.exempt_interest =
                data.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref()).map_or_else(
                    || crate::kernel::ratio::positive(data.exempt - data.tax_exempt_bond_premium),
                    |r| r.exempt(),
                );
            result.private_activity = data
                .adjustments
                .as_ref()
                .and_then(|a| a.instrument_records.as_ref())
                .map_or(data.private_activity, |r| r.private_activity());
            result.withholding = data.withholding;
        }
        FederalIncome::Dividend { data } => {
            result.ordinary_dividends = data.ordinary;
            result.qualified_dividends = data.qualified;
            result.capital_distributions = data.capital_distributions;
            result.unrecaptured_distributions = data.unrecaptured_distributions;
            result.collectible_distributions = data.collectible_distributions;
            result.section_1202_gain = data.section_1202_gain;
            result.section199a = data.section199a.qualified();
            result.exempt_interest = data.exempt_dividends;
            result.private_activity = data.private_activity_dividends;
            result.foreign_tax = data.foreign_tax;
            result.withholding = data.withholding;
        }
        _ => return None,
    }
    Some(result)
}

/// Interest belonging to someone outside this return
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InterestNominee {
    /// Taxable premium belonging to the other owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Treasury premium belonging to the other owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,
    /// Exempt premium belonging to the other owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Purchased accrued interest belonging to the other owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub accrued_interest: i64,
    /// Current-election market discount belonging to the other owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Taxable interest or OID belonging to others, excluding U.S. obligations
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// U.S. savings-bond and Treasury interest belonging to others
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Exempt interest belonging to others
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private-activity amount within the exempt nominee part
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Foreign tax associated with those other owners' income
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
}

/// Dividend components belonging to someone outside this return
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DividendNominee {
    /// Section897 subset of ordinary dividends
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_897_ordinary: i64,
    /// Section897 subset of capital distributions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_897_capital: i64,
    /// Ordinary dividends
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Qualified part of ordinary dividends
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified: i64,
    /// Capital gain distributions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub capital: i64,
    /// Unrecaptured section 1250 part
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
    /// Collectibles part
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub collectible: i64,
    /// Section 1202 part
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_1202: i64,
    /// Section 199A dividends
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_199a: i64,
    /// Exempt-interest dividends
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private-activity part of exempt dividends
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Foreign tax belonging to others
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
}

/// OID statement components belonging to someone outside this return
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OidNominee {
    /// Box 1 OID
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub oid: i64,
    /// Box 2 other periodic interest
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_periodic_interest: i64,
    /// Box 8 Treasury OID, including signed deflation
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub treasury_oid: i64,
    /// Current-election market discount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Acquisition premium belonging to this owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub acquisition_premium: i64,
    /// Taxable bond premium belonging to this owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Exempt OID
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Exempt premium belonging to this owner
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Private-activity part of exempt OID
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub private_activity: i64,
}

/// How the annual interest or OID amount is reported
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
pub enum DebtReporting {
    /// Gross income with separately stated premium
    #[default]
    Gross,
    /// Already reduced by premium; never subtract it again
    Net,
}

/// Personal reporting facts, not imported from the form
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InterestAdjustments {
    /// Instrument worksheets replacing this payer’s calculation, with separate ownership and reporting facts
    /// [NullText]
    /// Use the payer-level interest amounts and adjustments
    /// [SetText]
    /// Replace the payer-level calculation with
    #[libertas_default($null)]
    pub instrument_records: Option<AnnualDebtRecords>,
    /// Actual taxable-premium offset already applied to retained payer net income, after outside-owner allocation
    /// [NullText]
    /// No taxable-premium offset has been documented for the retained net amount
    /// [SetText]
    /// The documented taxable-premium offset already applied is
    pub net_premium_offset: Option<DebtPremiumOffset>,
    /// Savings-bond interest already included on an earlier return, within retained box 3
    /// Include a documented prior owner's final-return inclusion only once. Not an OID adjustment.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub previously_reported: i64,
    /// Whether reported income has already been reduced by the reported premium
    #[libertas_default(Gross)]
    pub reporting: DebtReporting,
}

// Ordered source-owner validation, shared by page saves/previews and restored records.
pub(crate) fn int_rule(v: &crate::FederalInterest) -> Option<&'static str> {
    int_issue(v).map(|issue| issue.key)
}
pub(crate) fn int_issue(v: &crate::FederalInterest) -> Option<ValidationIssue> {
    if let Some(party) = &v.seller_mortgage
        && let Some(issue) = party.issue()
    {
        return Some(issue.in_field("seller_mortgage"));
    }

    if v.seller_mortgage.is_some()
        && (v.nominee.is_some()
            || v.adjustments.is_some()
            || v.after_death.is_some()
            || v.special != crate::InterestSpecial::NoneOfThese
            || !v.foreign_country.is_empty()
            || [
                v.us_obligations,
                v.exempt,
                v.private_activity,
                v.withholding,
                v.early_penalty,
                v.bond_premium,
                v.treasury_bond_premium,
                v.tax_exempt_bond_premium,
                v.accrued_interest,
                v.foreign_tax,
                v.investment_expenses,
                v.market_discount,
            ]
            .iter()
            .any(|x| *x != 0))
    {
        return Some(
            ValidationIssue::new("SELLER_MORTGAGE_BUYER_INVALID").at_field("seller_mortgage"),
        );
    }
    if let Some(a) = &v.adjustments {
        if a.net_premium_offset.as_ref().is_some_and(|o| !o.valid()) {
            return Some(
                ValidationIssue::new("DEBT_COMPONENT_RANGE")
                    .at_field("net_premium_offset")
                    .in_field("adjustments"),
            );
        }
        if a.reporting != DebtReporting::Net && a.net_premium_offset.is_some() {
            return Some(
                ValidationIssue::new("DEBT_OFFSET_ONLY_FOR_NET")
                    .at_field("net_premium_offset")
                    .in_field("adjustments"),
            );
        }
    }
    if let Some(records) = v.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref()) {
        return annual::int_issue(v, records);
    }
    let n = v.nominee.clone().unwrap_or_default();
    let a = v.after_death.clone().unwrap_or_default();
    for (owner, field, amount) in [
        ("nominee", "taxable", n.taxable),
        ("nominee", "us_obligations", n.us_obligations),
        ("nominee", "exempt", n.exempt),
        ("nominee", "private_activity", n.private_activity),
        ("nominee", "foreign_tax", n.foreign_tax),
        ("nominee", "bond_premium", n.bond_premium),
        ("nominee", "treasury_bond_premium", n.treasury_bond_premium),
        ("nominee", "tax_exempt_bond_premium", n.tax_exempt_bond_premium),
        ("nominee", "accrued_interest", n.accrued_interest),
        ("nominee", "market_discount", n.market_discount),
        ("after_death", "taxable", a.taxable),
        ("after_death", "us_obligations", a.us_obligations),
        ("after_death", "exempt", a.exempt),
        ("after_death", "private_activity", a.private_activity),
        ("after_death", "foreign_tax", a.foreign_tax),
        ("after_death", "bond_premium", a.bond_premium),
        ("after_death", "treasury_bond_premium", a.treasury_bond_premium),
        ("after_death", "tax_exempt_bond_premium", a.tax_exempt_bond_premium),
        ("after_death", "accrued_interest", a.accrued_interest),
        ("after_death", "market_discount", a.market_discount),
    ] {
        if !(0..=100_000_000_000).contains(&amount) {
            return Some(
                ValidationIssue::new("INVESTMENT_SPLIT_RANGE").at_field(field).in_field(owner),
            );
        }
    }
    if n.taxable + a.taxable > v.taxable {
        return Some(ValidationIssue::new("INT_OUTSIDE_TAXABLE_ABOVE_BOX").at_field("taxable"));
    }
    if n.us_obligations + a.us_obligations > v.us_obligations {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_US_OBLIGATIONS_ABOVE_BOX").at_field("us_obligations"),
        );
    }
    if n.exempt + a.exempt > v.exempt {
        return Some(ValidationIssue::new("INT_OUTSIDE_EXEMPT_ABOVE_BOX").at_field("exempt"));
    }
    if n.private_activity + a.private_activity > v.private_activity {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX")
                .at_field("private_activity"),
        );
    }
    if n.foreign_tax + a.foreign_tax > v.foreign_tax {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_FOREIGN_TAX_ABOVE_BOX").at_field("foreign_tax"),
        );
    }
    if n.bond_premium + a.bond_premium > v.bond_premium {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_BOND_PREMIUM_ABOVE_BOX").at_field("bond_premium"),
        );
    }
    if n.treasury_bond_premium + a.treasury_bond_premium > v.treasury_bond_premium {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_TREASURY_BOND_PREMIUM_ABOVE_BOX")
                .at_field("treasury_bond_premium"),
        );
    }
    if n.tax_exempt_bond_premium + a.tax_exempt_bond_premium > v.tax_exempt_bond_premium {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX")
                .at_field("tax_exempt_bond_premium"),
        );
    }
    if n.accrued_interest + a.accrued_interest > v.accrued_interest {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_ACCRUED_INTEREST_ABOVE_BOX")
                .at_field("accrued_interest"),
        );
    }
    if n.market_discount + a.market_discount > v.market_discount {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_MARKET_DISCOUNT_ABOVE_BOX")
                .at_field("market_discount"),
        );
    }
    if n.private_activity > n.exempt {
        return Some(
            ValidationIssue::new("INT_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                .at_field("private_activity")
                .in_field("nominee"),
        );
    }
    if a.private_activity > a.exempt {
        return Some(
            ValidationIssue::new("INT_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                .at_field("private_activity")
                .in_field("after_death"),
        );
    }
    if v.private_activity - n.private_activity - a.private_activity > v.exempt - n.exempt - a.exempt
    {
        return Some(
            ValidationIssue::new("INT_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                .at_field("private_activity"),
        );
    }
    if v.adjustments.as_ref().is_some_and(|x| {
        x.previously_reported < 0
            || x.previously_reported > v.us_obligations - n.us_obligations - a.us_obligations
    }) {
        return Some(
            ValidationIssue::new("INTEREST_PRIOR_ABOVE_RETAINED")
                .at_field("previously_reported")
                .in_field("adjustments"),
        );
    }
    None
}
pub(crate) fn div_rule(v: &crate::FederalDividend) -> Option<&'static str> {
    div_issue(v).map(|issue| issue.key)
}
pub(crate) fn div_issue(v: &crate::FederalDividend) -> Option<ValidationIssue> {
    let key = "DIV_CAPITAL_RETURNS_RECONCILE";
    if v.capital_returns.len() > 100 {
        return Some(ValidationIssue::new(key).at_field("capital_returns"));
    }
    for (field, amount) in [
        ("nondividend_distributions", v.nondividend_distributions),
        ("cash_liquidation", v.cash_liquidation),
        ("noncash_liquidation", v.noncash_liquidation),
    ] {
        if !(0..=100_000_000_000).contains(&amount) {
            return Some(ValidationIssue::new(key).at_field(field));
        }
    }
    for (index, lot) in v.capital_returns.iter().enumerate() {
        if let Some(issue) = lot.validation_issue() {
            return Some(if issue.field.is_some() {
                issue.in_list("capital_returns", index)
            } else {
                // An intrinsic composite names its actual row, without guessing a leaf.
                issue.at_list_item("capital_returns", index)
            });
        }
    }
    for (field, total, stated) in [
        (
            "nondividend_distributions",
            v.capital_returns
                .iter()
                .filter(|lot| lot.kind == crate::CapitalDistributionKind::Nondividend)
                .map(|lot| lot.amount)
                .sum::<i64>(),
            v.nondividend_distributions,
        ),
        (
            "cash_liquidation",
            v.capital_returns
                .iter()
                .filter(|lot| lot.kind != crate::CapitalDistributionKind::Nondividend)
                .map(|lot| lot.amount)
                .sum::<i64>(),
            v.cash_liquidation,
        ),
        (
            "noncash_liquidation",
            v.capital_returns.iter().flat_map(|lot| &lot.property).map(|p| p.value).sum::<i64>(),
            v.noncash_liquidation,
        ),
    ] {
        if total != stated {
            return Some(ValidationIssue::new(key).at_field(field));
        }
    }
    let n = v.nominee.clone().unwrap_or_default();
    let a = v.after_death.clone().unwrap_or_default();
    for (owner, field, amount) in [
        ("nominee", "ordinary", n.ordinary),
        ("nominee", "qualified", n.qualified),
        ("nominee", "capital", n.capital),
        ("nominee", "unrecaptured", n.unrecaptured),
        ("nominee", "collectible", n.collectible),
        ("nominee", "section_1202", n.section_1202),
        ("nominee", "section_199a", n.section_199a),
        ("nominee", "exempt", n.exempt),
        ("nominee", "private_activity", n.private_activity),
        ("nominee", "foreign_tax", n.foreign_tax),
        ("nominee", "section_897_ordinary", n.section_897_ordinary),
        ("nominee", "section_897_capital", n.section_897_capital),
        ("after_death", "ordinary", a.ordinary),
        ("after_death", "qualified", a.qualified),
        ("after_death", "capital_distributions", a.capital_distributions),
        ("after_death", "unrecaptured_distributions", a.unrecaptured_distributions),
        ("after_death", "collectible_distributions", a.collectible_distributions),
        ("after_death", "section_1202_gain", a.section_1202_gain),
        ("after_death", "section199a", a.section199a),
        ("after_death", "exempt_dividends", a.exempt_dividends),
        ("after_death", "private_activity_dividends", a.private_activity_dividends),
        ("after_death", "foreign_tax", a.foreign_tax),
        ("after_death", "section_897_ordinary", a.section_897_ordinary),
        ("after_death", "section_897_capital", a.section_897_capital),
    ] {
        if !(0..=100_000_000_000).contains(&amount) {
            return Some(
                ValidationIssue::new("INVESTMENT_SPLIT_RANGE").at_field(field).in_field(owner),
            );
        }
    }
    if n.ordinary + a.ordinary > v.ordinary {
        return Some(ValidationIssue::new("DIV_OUTSIDE_ORDINARY_ABOVE_BOX").at_field("ordinary"));
    }
    if n.qualified + a.qualified > v.qualified {
        return Some(ValidationIssue::new("DIV_OUTSIDE_QUALIFIED_ABOVE_BOX").at_field("qualified"));
    }
    if n.capital + a.capital_distributions > v.capital_distributions {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_CAPITAL_ABOVE_BOX").at_field("capital_distributions"),
        );
    }
    if n.unrecaptured + a.unrecaptured_distributions > v.unrecaptured_distributions {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_UNRECAPTURED_ABOVE_BOX")
                .at_field("unrecaptured_distributions"),
        );
    }
    if n.collectible + a.collectible_distributions > v.collectible_distributions {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_COLLECTIBLE_ABOVE_BOX")
                .at_field("collectible_distributions"),
        );
    }
    if n.section_1202 + a.section_1202_gain > v.section_1202_gain {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_SECTION_1202_ABOVE_BOX")
                .at_field("section_1202_gain"),
        );
    }
    if n.section_199a + a.section199a > v.section199a.amount() {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_SECTION_199A_ABOVE_BOX").at_field("section199a"),
        );
    }
    if n.exempt + a.exempt_dividends > v.exempt_dividends {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_EXEMPT_ABOVE_BOX").at_field("exempt_dividends"),
        );
    }
    if n.private_activity + a.private_activity_dividends > v.private_activity_dividends {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX")
                .at_field("private_activity_dividends"),
        );
    }
    if n.foreign_tax + a.foreign_tax > v.foreign_tax {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_FOREIGN_TAX_ABOVE_BOX").at_field("foreign_tax"),
        );
    }
    if n.section_897_ordinary + a.section_897_ordinary > v.section_897_ordinary {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_SECTION_897_ORDINARY_ABOVE_BOX")
                .at_field("section_897_ordinary"),
        );
    }
    if n.section_897_capital + a.section_897_capital > v.section_897_capital {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_SECTION_897_CAPITAL_ABOVE_BOX")
                .at_field("section_897_capital"),
        );
    }
    if n.qualified > n.ordinary {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_QUALIFIED_ABOVE_ORDINARY")
                .at_field("qualified")
                .in_field("nominee"),
        );
    }
    if a.qualified > a.ordinary {
        return Some(
            ValidationIssue::new("DIV_AFTER_QUALIFIED_ABOVE_ORDINARY")
                .at_field("qualified")
                .in_field("after_death"),
        );
    }
    if v.qualified - n.qualified - a.qualified > v.ordinary - n.ordinary - a.ordinary {
        return Some(
            ValidationIssue::new("DIV_RETAINED_QUALIFIED_ABOVE_ORDINARY").at_field("qualified"),
        );
    }
    if n.private_activity > n.exempt {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                .at_field("private_activity")
                .in_field("nominee"),
        );
    }
    if a.private_activity_dividends > a.exempt_dividends {
        return Some(
            ValidationIssue::new("DIV_AFTER_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                .at_field("private_activity_dividends")
                .in_field("after_death"),
        );
    }
    if v.private_activity_dividends - n.private_activity - a.private_activity_dividends
        > v.exempt_dividends - n.exempt - a.exempt_dividends
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                .at_field("private_activity_dividends"),
        );
    }
    if n.section_199a > n.ordinary {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_SECTION_199A_ABOVE_ORDINARY")
                .at_field("section_199a")
                .in_field("nominee"),
        );
    }
    if a.section199a > a.ordinary {
        return Some(
            ValidationIssue::new("DIV_AFTER_SECTION_199A_ABOVE_ORDINARY")
                .at_field("section199a")
                .in_field("after_death"),
        );
    }
    if v.section199a.amount() - n.section_199a - a.section199a
        > v.ordinary - n.ordinary - a.ordinary
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_SECTION_199A_ABOVE_ORDINARY")
                .at_field("section199a"),
        );
    }
    if n.section_897_ordinary > n.ordinary {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_SECTION_897_ORDINARY_ABOVE_ORDINARY")
                .at_field("section_897_ordinary")
                .in_field("nominee"),
        );
    }
    if a.section_897_ordinary > a.ordinary {
        return Some(
            ValidationIssue::new("DIV_AFTER_SECTION_897_ORDINARY_ABOVE_ORDINARY")
                .at_field("section_897_ordinary")
                .in_field("after_death"),
        );
    }
    if v.section_897_ordinary - n.section_897_ordinary - a.section_897_ordinary
        > v.ordinary - n.ordinary - a.ordinary
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_SECTION_897_ORDINARY_ABOVE_ORDINARY")
                .at_field("section_897_ordinary"),
        );
    }
    if n.section_897_capital > n.capital {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_SECTION_897_CAPITAL_ABOVE_CAPITAL")
                .at_field("section_897_capital")
                .in_field("nominee"),
        );
    }
    if a.section_897_capital > a.capital_distributions {
        return Some(
            ValidationIssue::new("DIV_AFTER_SECTION_897_CAPITAL_ABOVE_CAPITAL")
                .at_field("section_897_capital")
                .in_field("after_death"),
        );
    }
    if v.section_897_capital - n.section_897_capital - a.section_897_capital
        > v.capital_distributions - n.capital - a.capital_distributions
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_SECTION_897_CAPITAL_ABOVE_CAPITAL")
                .at_field("section_897_capital"),
        );
    }
    if n.unrecaptured > n.capital {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_UNRECAPTURED_ABOVE_CAPITAL")
                .at_field("unrecaptured")
                .in_field("nominee"),
        );
    }
    if a.unrecaptured_distributions > a.capital_distributions {
        return Some(
            ValidationIssue::new("DIV_AFTER_UNRECAPTURED_ABOVE_CAPITAL")
                .at_field("unrecaptured_distributions")
                .in_field("after_death"),
        );
    }
    if v.unrecaptured_distributions - n.unrecaptured - a.unrecaptured_distributions
        > v.capital_distributions - n.capital - a.capital_distributions
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_UNRECAPTURED_ABOVE_CAPITAL")
                .at_field("unrecaptured_distributions"),
        );
    }
    if n.collectible > n.capital {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_COLLECTIBLE_ABOVE_CAPITAL")
                .at_field("collectible")
                .in_field("nominee"),
        );
    }
    if a.collectible_distributions > a.capital_distributions {
        return Some(
            ValidationIssue::new("DIV_AFTER_COLLECTIBLE_ABOVE_CAPITAL")
                .at_field("collectible_distributions")
                .in_field("after_death"),
        );
    }
    if v.collectible_distributions - n.collectible - a.collectible_distributions
        > v.capital_distributions - n.capital - a.capital_distributions
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_COLLECTIBLE_ABOVE_CAPITAL")
                .at_field("collectible_distributions"),
        );
    }
    if n.section_1202 > n.capital {
        return Some(
            ValidationIssue::new("DIV_NOMINEE_SECTION_1202_ABOVE_CAPITAL")
                .at_field("section_1202")
                .in_field("nominee"),
        );
    }
    if a.section_1202_gain > a.capital_distributions {
        return Some(
            ValidationIssue::new("DIV_AFTER_SECTION_1202_ABOVE_CAPITAL")
                .at_field("section_1202_gain")
                .in_field("after_death"),
        );
    }
    if v.section_1202_gain - n.section_1202 - a.section_1202_gain
        > v.capital_distributions - n.capital - a.capital_distributions
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_SECTION_1202_ABOVE_CAPITAL")
                .at_field("section_1202_gain"),
        );
    }
    if n.qualified + n.section_199a > n.ordinary
        || a.qualified + a.section199a > a.ordinary
        || v.qualified - n.qualified - a.qualified + v.section199a.amount()
            - n.section_199a
            - a.section199a
            > v.ordinary - n.ordinary - a.ordinary
    {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_QUALIFIED_REIT_OVERLAP").at_field("ordinary"),
        );
    }
    if n.unrecaptured + n.collectible > n.capital
        || a.unrecaptured_distributions + a.collectible_distributions > a.capital_distributions
        || v.unrecaptured_distributions - n.unrecaptured - a.unrecaptured_distributions
            + v.collectible_distributions
            - n.collectible
            - a.collectible_distributions
            > v.capital_distributions - n.capital - a.capital_distributions
    {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_CAPITAL_PARTS_OVERLAP")
                .at_field("capital_distributions"),
        );
    }
    None
}
pub(crate) fn oid_rule(v: &crate::FederalOid) -> Option<&'static str> {
    oid_issue(v).map(|issue| issue.key)
}
pub(crate) fn oid_issue(v: &crate::FederalOid) -> Option<ValidationIssue> {
    if let Some(a) = &v.adjustments {
        if a.net_premium_offset.as_ref().is_some_and(|o| !o.valid()) {
            return Some(
                ValidationIssue::new("DEBT_COMPONENT_RANGE")
                    .at_field("net_premium_offset")
                    .in_field("adjustments"),
            );
        }
        if a.reporting != DebtReporting::Net && a.net_premium_offset.is_some() {
            return Some(
                ValidationIssue::new("DEBT_OFFSET_ONLY_FOR_NET")
                    .at_field("net_premium_offset")
                    .in_field("adjustments"),
            );
        }
    }
    if let Some(records) = v.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref()) {
        return annual::oid_issue(v, records);
    }
    let n = v.nominee.clone().unwrap_or_default();
    for (owner, field, amount) in [
        ("nominee", "oid", n.oid),
        ("nominee", "other_periodic_interest", n.other_periodic_interest),
        ("nominee", "market_discount", n.market_discount),
        ("nominee", "acquisition_premium", n.acquisition_premium),
        ("nominee", "bond_premium", n.bond_premium),
        ("nominee", "exempt", n.exempt),
        ("nominee", "tax_exempt_bond_premium", n.tax_exempt_bond_premium),
        ("nominee", "private_activity", n.private_activity),
    ] {
        if !(0..=100_000_000_000).contains(&amount) {
            return Some(
                ValidationIssue::new("INVESTMENT_SPLIT_RANGE").at_field(field).in_field(owner),
            );
        }
    }
    if n.oid > v.oid {
        return Some(
            ValidationIssue::new("OID_NOMINEE_OID_ABOVE_BOX").at_field("oid").in_field("nominee"),
        );
    }
    if n.other_periodic_interest > v.other_periodic_interest {
        return Some(
            ValidationIssue::new("OID_NOMINEE_OTHER_PERIODIC_INTEREST_ABOVE_BOX")
                .at_field("other_periodic_interest")
                .in_field("nominee"),
        );
    }
    if n.market_discount > v.market_discount {
        return Some(
            ValidationIssue::new("OID_NOMINEE_MARKET_DISCOUNT_ABOVE_BOX")
                .at_field("market_discount")
                .in_field("nominee"),
        );
    }
    if n.acquisition_premium > v.acquisition_premium {
        return Some(
            ValidationIssue::new("OID_NOMINEE_ACQUISITION_PREMIUM_ABOVE_BOX")
                .at_field("acquisition_premium")
                .in_field("nominee"),
        );
    }
    if n.bond_premium > v.bond_premium {
        return Some(
            ValidationIssue::new("OID_NOMINEE_BOND_PREMIUM_ABOVE_BOX")
                .at_field("bond_premium")
                .in_field("nominee"),
        );
    }
    if n.exempt > v.exempt {
        return Some(
            ValidationIssue::new("OID_NOMINEE_EXEMPT_ABOVE_BOX")
                .at_field("exempt")
                .in_field("nominee"),
        );
    }
    if n.tax_exempt_bond_premium > v.tax_exempt_bond_premium {
        return Some(
            ValidationIssue::new("OID_NOMINEE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX")
                .at_field("tax_exempt_bond_premium")
                .in_field("nominee"),
        );
    }
    if n.private_activity > v.private_activity {
        return Some(
            ValidationIssue::new("OID_NOMINEE_PRIVATE_ACTIVITY_ABOVE_BOX")
                .at_field("private_activity")
                .in_field("nominee"),
        );
    }
    if !signed_share(v.treasury_oid, n.treasury_oid) {
        return Some(
            ValidationIssue::new("OID_NOMINEE_TREASURY_SIGN_OR_BOUND")
                .at_field("treasury_oid")
                .in_field("nominee"),
        );
    }
    if n.private_activity > n.exempt {
        return Some(
            ValidationIssue::new("OID_NOMINEE_PRIVATE_ABOVE_EXEMPT")
                .at_field("private_activity")
                .in_field("nominee"),
        );
    }
    if v.private_activity - n.private_activity > v.exempt - n.exempt {
        return Some(
            ValidationIssue::new("OID_RETAINED_PRIVATE_ABOVE_EXEMPT").at_field("private_activity"),
        );
    }
    if v.tax_exempt_bond_premium > v.exempt
        || n.tax_exempt_bond_premium > n.exempt
        || v.tax_exempt_bond_premium - n.tax_exempt_bond_premium > v.exempt - n.exempt
    {
        return Some(
            ValidationIssue::new("EXEMPT_REDUCTION_ABOVE_AMOUNT")
                .at_field("tax_exempt_bond_premium"),
        );
    }
    if v.adjustments.as_ref().is_some_and(|a| a.previously_reported != 0) {
        return Some(
            ValidationIssue::new("OID_PRIOR_IS_NOT_BOX3")
                .at_field("previously_reported")
                .in_field("adjustments"),
        );
    }
    None
}
fn signed_share(whole: i64, share: i64) -> bool {
    if whole >= 0 { (0..=whole).contains(&share) } else { (whole..=0).contains(&share) }
}

// Called after final-return allocation and before community-property allocation.
// Invalid personal facts are retained for normal stored-data validation, never repaired.
pub(crate) fn apply(
    d: &mut crate::FederalDraft,
    notes: &mut alloc::vec::Vec<crate::federal_model::ReviewNote>,
) -> bool {
    let mut financial_omission = false;
    let bond_sources = crate::savings_bonds::sources(&d.income);
    let election = d.screening.as_ref().and_then(|s| s.taxable_premium_election);
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let mut nominee = false;
        let mut premium_left_out = false;
        if let Some((keys, omitted)) = apply_annual(
            &mut entry.income,
            d.screening.as_ref().and_then(|s| s.taxable_premium_election),
        ) {
            financial_omission |= omitted;
            for key in keys {
                let message = crate::Message::new(key, alloc::vec::Vec::new());
                notes.push(crate::federal_model::ReviewNote::scoped(
                    #[cfg(test)]
                    message.english(),
                    message,
                    context.clone(),
                ));
            }
            continue;
        }
        match &mut entry.income {
            FederalIncome::Interest { data } if int_rule(data).is_none() => {
                if let Some(part) = data.nominee.take() {
                    nominee = [
                        part.taxable,
                        part.us_obligations,
                        part.exempt,
                        part.private_activity,
                        part.foreign_tax,
                        part.bond_premium,
                        part.treasury_bond_premium,
                        part.tax_exempt_bond_premium,
                        part.accrued_interest,
                        part.market_discount,
                    ]
                    .iter()
                    .any(|x| *x != 0);
                    data.taxable -= part.taxable;
                    data.us_obligations -= part.us_obligations;
                    data.exempt -= part.exempt;
                    data.private_activity -= part.private_activity;
                    data.foreign_tax -= part.foreign_tax;
                    data.bond_premium -= part.bond_premium;
                    data.treasury_bond_premium -= part.treasury_bond_premium;
                    data.tax_exempt_bond_premium -= part.tax_exempt_bond_premium;
                    data.accrued_interest -= part.accrued_interest;
                    data.market_discount -= part.market_discount;
                }
                let a = data.adjustments.take().unwrap_or_default();
                data.us_obligations -= a.previously_reported;
                if a.reporting == DebtReporting::Net {
                    data.bond_premium = 0;
                    data.treasury_bond_premium = 0;
                    data.tax_exempt_bond_premium = 0;
                    if election != Some(true)
                        && let Some(offset) = a.net_premium_offset
                    {
                        data.taxable += offset.ordinary;
                        data.us_obligations += offset.treasury;
                        premium_left_out = offset.ordinary + offset.treasury > 0;
                    }
                } else if election != Some(true) {
                    premium_left_out = data.bond_premium + data.treasury_bond_premium > 0;
                    data.bond_premium = 0;
                    data.treasury_bond_premium = 0;
                }
            }
            FederalIncome::Dividend { data } if div_rule(data).is_none() => {
                if let Some(part) = data.nominee.take() {
                    nominee = [
                        part.ordinary,
                        part.qualified,
                        part.capital,
                        part.unrecaptured,
                        part.collectible,
                        part.section_1202,
                        part.section_199a,
                        part.exempt,
                        part.private_activity,
                        part.foreign_tax,
                        part.section_897_ordinary,
                        part.section_897_capital,
                    ]
                    .iter()
                    .any(|x| *x != 0);
                    data.ordinary -= part.ordinary;
                    data.qualified -= part.qualified;
                    data.capital_distributions -= part.capital;
                    data.unrecaptured_distributions -= part.unrecaptured;
                    data.collectible_distributions -= part.collectible;
                    data.section_1202_gain -= part.section_1202;
                    if let crate::ReitDividends::Reported { amount, .. } = &mut data.section199a {
                        *amount -= part.section_199a;
                    }
                    data.exempt_dividends -= part.exempt;
                    data.private_activity_dividends -= part.private_activity;
                    data.foreign_tax -= part.foreign_tax;
                    data.section_897_ordinary -= part.section_897_ordinary;
                    data.section_897_capital -= part.section_897_capital;
                }
            }
            FederalIncome::Oid { data } if oid_rule(data).is_none() => {
                if let Some(part) = data.nominee.take() {
                    nominee = [
                        part.oid,
                        part.other_periodic_interest,
                        part.market_discount,
                        part.acquisition_premium,
                        part.bond_premium,
                        part.exempt,
                        part.tax_exempt_bond_premium,
                        part.private_activity,
                        part.treasury_oid,
                    ]
                    .iter()
                    .any(|x| *x != 0);
                    data.oid -= part.oid;
                    data.other_periodic_interest -= part.other_periodic_interest;
                    data.market_discount -= part.market_discount;
                    data.acquisition_premium -= part.acquisition_premium;
                    data.bond_premium -= part.bond_premium;
                    data.exempt -= part.exempt;
                    data.tax_exempt_bond_premium -= part.tax_exempt_bond_premium;
                    data.private_activity -= part.private_activity;
                    data.treasury_oid -= part.treasury_oid;
                }
                let a = data.adjustments.take().unwrap_or_default();
                if a.reporting == DebtReporting::Net {
                    data.acquisition_premium = 0;
                    data.bond_premium = 0;
                    data.tax_exempt_bond_premium = 0;
                    if election != Some(true)
                        && let Some(offset) = a.net_premium_offset
                    {
                        data.oid += offset.ordinary;
                        data.treasury_oid += offset.treasury;
                        premium_left_out = offset.ordinary + offset.treasury > 0;
                    }
                } else if election != Some(true) {
                    premium_left_out = data.bond_premium > 0;
                    data.bond_premium = 0;
                }
            }
            _ => {}
        }
        if premium_left_out {
            financial_omission = true;
            let message = crate::Message::new("WARN_DEBT_PREMIUM_ELECTION_LEFT_OUT", alloc::vec![]);
            notes.push(crate::federal_model::ReviewNote::scoped(
                #[cfg(test)]
                message.english(),
                message,
                context.clone(),
            ));
        }
        if nominee {
            let message = crate::Message::new("NOTE_INVESTMENT_NOMINEE", alloc::vec![]);
            let note = crate::federal_model::ReviewNote::scoped(
                #[cfg(test)]
                message.english(),
                message,
                context,
            );
            if !notes.iter().any(|old| old.same_notice(&note)) {
                notes.push(note);
            }
        }
    }
    financial_omission | crate::savings_bonds::normalize(&mut d.income, &bond_sources, notes)
}

pub(crate) fn annual_records(income: &FederalIncome) -> Option<&AnnualDebtRecords> {
    match income {
        FederalIncome::Interest { data } => data.adjustments.as_ref(),
        FederalIncome::Oid { data } => data.adjustments.as_ref(),
        _ => None,
    }
    .and_then(|a| a.instrument_records.as_ref())
}
pub(crate) fn annual_ids(income: &FederalIncome) -> alloc::vec::Vec<i64> {
    annual_records(income)
        .map_or_else(alloc::vec::Vec::new, |a| a.records.iter().map(|r| r.id).collect())
}
pub(crate) fn annual_ids_mut(income: &mut FederalIncome) -> alloc::vec::Vec<&mut i64> {
    let adjustments = match income {
        FederalIncome::Interest { data } => data.adjustments.as_mut(),
        FederalIncome::Oid { data } => data.adjustments.as_mut(),
        _ => None,
    };
    adjustments
        .and_then(|a| a.instrument_records.as_mut())
        .map_or_else(alloc::vec::Vec::new, |a| a.records.iter_mut().map(|r| &mut r.id).collect())
}
// Normalize a calculation copy only. Keep instrument facts so every downstream
// owner uses this one per-instrument calculation, not a second premium engine.
fn apply_annual(
    income: &mut FederalIncome,
    election: Option<bool>,
) -> Option<(alloc::vec::Vec<&'static str>, bool)> {
    let oid_source = matches!(income, FederalIncome::Oid { .. });
    let valid = match income {
        FederalIncome::Interest { data } => int_rule(data).is_none(),
        FederalIncome::Oid { data } => oid_rule(data).is_none(),
        _ => false,
    };
    if !valid || annual_records(income).is_none() {
        return None;
    }
    let adjustments = match income {
        FederalIncome::Interest { data } => data.adjustments.as_mut(),
        FederalIncome::Oid { data } => data.adjustments.as_mut(),
        _ => None,
    }
    .expect("annual statement");
    let records = adjustments.instrument_records.as_mut().expect("annual records");
    let mut notices = alloc::vec::Vec::new();
    let mut financial_omission = false;
    for r in &mut records.records {
        if r.original_issue
            .as_ref()
            .is_some_and(|issue| issue.de_minimis() && !issue.elect_de_minimis_oid)
        {
            notices.push("NOTE_DEBT_DE_MINIMIS_OID");
        }
        let n = r.nominee.is_some();
        let a = r.after_death.is_some();
        r.amounts = r.retained();
        r.nominee = None;
        r.after_death = None;
        r.previously_reported = 0;
        if n {
            notices.push("NOTE_INVESTMENT_NOMINEE");
        }
        if a {
            notices.push("NOTE_DEBT_AFTER_DEATH");
        }
        if r.treatment == AnnualDebtTreatment::Contingent {
            financial_omission = true;
            notices.push("ISSUE_DEBT_CONTINGENT_INCOME");
        }
        if r.reporting == DebtReporting::Net
            && election != Some(true)
            && let Some(offset) = &r.net_premium_offset
        {
            if oid_source {
                r.amounts.oid += offset.ordinary;
            } else {
                r.amounts.taxable += offset.ordinary;
            }
            if oid_source {
                r.amounts.treasury_oid += offset.treasury;
            } else {
                r.amounts.us_obligations += offset.treasury;
            }
            if offset.ordinary + offset.treasury > 0 {
                financial_omission = true;
                notices.push("WARN_DEBT_PREMIUM_ELECTION_LEFT_OUT");
            }
            r.net_premium_offset = Some(DebtPremiumOffset::default());
        }
        let taxable_premium = r.amounts.bond_premium + r.amounts.treasury_bond_premium;
        let carry = r.premium_history.as_ref().map_or(0, |h| h.premium_carry);
        if r.reporting == DebtReporting::Gross
            && taxable_premium + carry > 0
            && election != Some(true)
        {
            r.amounts.bond_premium = 0;
            r.amounts.treasury_bond_premium = 0;
            r.premium_history = None;
            financial_omission = true;
            notices.push("WARN_DEBT_PREMIUM_ELECTION_LEFT_OUT");
        }
        let available = crate::kernel::ratio::positive(
            r.amounts.taxable
                + r.amounts.oid
                + r.amounts.other_periodic_interest
                + r.amounts.us_obligations
                + r.amounts.treasury_oid
                - r.amounts.acquisition_premium
                - r.amounts.treasury_acquisition_premium
                - r.amounts.accrued_interest
                - r.amounts.treasury_accrued_interest,
        );
        if r.reporting == DebtReporting::Gross
            && taxable_premium > available
            && r.premium_history.is_none()
        {
            financial_omission = true;
            notices.push("WARN_DEBT_PREMIUM_HISTORY_LEFT_OUT");
        }
        if r.reporting == DebtReporting::Net && carry > 0 {
            financial_omission = true;
            notices.push("WARN_DEBT_NET_CARRY_LEFT_OUT");
        }
        if r.disposed_on.is_some()
            && (taxable_premium > 0
                || r.premium_history.as_ref().is_some_and(|h| h.premium_carry > 0))
        {
            financial_omission = true;
            notices.push("WARN_DEBT_TERMINAL_PREMIUM_LEFT_OUT");
        }
        if let Some(value) = r.private_activity_after_premium {
            r.amounts.private_activity = value;
        }
    }
    let records = records.clone();
    match income {
        FederalIncome::Interest { data } => {
            data.taxable = records.records.iter().map(|r| r.amounts.taxable).sum();
            data.us_obligations = records.records.iter().map(|r| r.amounts.us_obligations).sum();
            data.exempt = records.records.iter().map(|r| r.amounts.exempt).sum();
            data.private_activity =
                records.records.iter().map(|r| r.amounts.private_activity).sum();
            data.foreign_tax = records.records.iter().map(|r| r.amounts.foreign_tax).sum();
            data.market_discount = records.records.iter().map(|r| r.amounts.market_discount).sum();
            data.bond_premium = records.records.iter().map(|r| r.amounts.bond_premium).sum();
            data.treasury_bond_premium =
                records.records.iter().map(|r| r.amounts.treasury_bond_premium).sum();
            data.tax_exempt_bond_premium =
                records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum();
            data.accrued_interest = records
                .records
                .iter()
                .map(|r| r.amounts.accrued_interest + r.amounts.treasury_accrued_interest)
                .sum();
        }
        FederalIncome::Oid { data } => {
            data.oid = records.records.iter().map(|r| r.amounts.oid).sum();
            data.other_periodic_interest =
                records.records.iter().map(|r| r.amounts.other_periodic_interest).sum();
            data.treasury_oid = records.records.iter().map(|r| r.amounts.treasury_oid).sum();
            data.exempt = records.records.iter().map(|r| r.amounts.exempt).sum();
            data.private_activity =
                records.records.iter().map(|r| r.amounts.private_activity).sum();
            data.market_discount = records.records.iter().map(|r| r.amounts.market_discount).sum();
            data.bond_premium = records
                .records
                .iter()
                .map(|r| r.amounts.bond_premium + r.amounts.treasury_bond_premium)
                .sum();
            data.acquisition_premium = records
                .records
                .iter()
                .map(|r| r.amounts.acquisition_premium + r.amounts.treasury_acquisition_premium)
                .sum();
            data.tax_exempt_bond_premium =
                records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum();
        }
        _ => unreachable!(),
    }
    notices.sort_unstable();
    notices.dedup();
    Some((notices, financial_omission))
}

// Both functions consume the central supported working copy. They never
// recompute ownership and expose real accepted child identities only.
pub(crate) fn computed_bond_premium(d: &crate::FederalDraft) -> i64 {
    d.income
        .iter()
        .filter_map(|e| annual_records(&e.income))
        .flat_map(|r| &r.records)
        .map(AnnualDebtRecord::excess_deduction)
        .sum()
}
pub(crate) fn bond_instruments(d: &crate::FederalDraft) -> alloc::vec::Vec<BondPremiumInstrument> {
    let mut notes = alloc::vec::Vec::new();
    let working = crate::unknowns::supported(d, &mut notes);
    let working = working.as_ref().unwrap_or(d);
    working
        .income
        .iter()
        .flat_map(|entry| {
            annual_records(&entry.income).into_iter().flat_map(move |rows| {
                rows.records.iter().filter(|r| r.amounts.exempt == 0).map(move |r| {
                    BondPremiumInstrument {
                        id: r.id,
                        label: crate::Message::new(
                            "BOND_PREMIUM_INSTRUMENT_LABEL",
                            alloc::vec![
                                crate::MessageArg::Text(entry.label.clone()),
                                crate::MessageArg::Text(r.instrument.clone())
                            ],
                        )
                        .encode(),
                        computed_deduction: r.excess_deduction(),
                    }
                })
            })
        })
        .collect()
}
pub(crate) fn manual_bond_premium_rule(
    d: &crate::FederalDraft,
    lines: &[crate::OtherItemizedLine],
) -> Option<&'static str> {
    manual_bond_premium_issue(d, lines).map(|issue| issue.key)
}

pub(crate) fn manual_bond_premium_issue(
    d: &crate::FederalDraft,
    lines: &[crate::OtherItemizedLine],
) -> Option<ValidationIssue> {
    let sources = bond_instruments(d);
    let mut seen = alloc::collections::BTreeSet::new();
    for (index, row) in lines.iter().enumerate() {
        let crate::OtherItemizedLine::BondPremium { source, .. } = row else {
            continue;
        };
        match source {
            BondPremiumSource::OtherDocumented { instrument } => {
                if instrument.trim().is_empty() {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_DESCRIPTION_NEEDED")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                }
            }
            BondPremiumSource::AcceptedAnnualInstrument { instrument } => {
                if !seen.insert(*instrument) {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_SOURCE_TWICE")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                }
                let Some(source) = sources.iter().find(|s| s.id == *instrument) else {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_SOURCE_MISSING")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                };
                if source.computed_deduction > 0 {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_COMPUTED_SOURCE_TWICE")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                }
            }
        }
    }
    None
}

pub(crate) fn unresolved_contingent(income: &FederalIncome) -> bool {
    annual_records(income).is_some_and(|rows| {
        rows.records.iter().any(|r| r.treatment == AnnualDebtTreatment::Contingent)
    })
}

// A net report does not establish the taxable gross amount when the election
// is absent or declined. Only an actual retained applied-offset fact can do so.
pub(crate) fn net_income_unresolved(income: &FederalIncome, election: Option<bool>) -> bool {
    if election == Some(true) {
        return false;
    }
    if let Some(rows) = annual_records(income) {
        return rows.records.iter().any(|r| {
            r.reporting == DebtReporting::Net
                && r.net_premium_offset.is_none()
                && (r.amounts.exempt == 0
                    || r.amounts.taxable
                        + r.amounts.us_obligations
                        + r.amounts.oid
                        + r.amounts.other_periodic_interest
                        + r.amounts.bond_premium
                        + r.amounts.treasury_bond_premium
                        != 0
                    || r.amounts.treasury_oid != 0)
        });
    }
    match income {
        FederalIncome::Interest { data } => data.adjustments.as_ref().is_some_and(|a| {
            a.reporting == DebtReporting::Net
                && a.net_premium_offset.is_none()
                && (data.exempt == 0
                    || data.taxable
                        + data.us_obligations
                        + data.bond_premium
                        + data.treasury_bond_premium
                        > 0)
        }),
        FederalIncome::Oid { data } => data.adjustments.as_ref().is_some_and(|a| {
            a.reporting == DebtReporting::Net
                && a.net_premium_offset.is_none()
                && (data.exempt == 0
                    || data.oid + data.other_periodic_interest + data.bond_premium > 0
                    || data.treasury_oid != 0)
        }),
        _ => false,
    }
}

/// Buyer whose home purchase you financed
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SellerMortgageParty {
    /// Buyer name
    #[libertas_size(min = 1)]
    pub buyer: alloc::string::String,
    /// Buyer's Social Security number or taxpayer identification number
    /// Enter all nine digits, including leading zeroes, without punctuation.
    // Legal identifier: the buyer taxpayer identification number has nine decimal digits.
    #[libertas_size(min = 9, max = 9)]
    #[libertas_pattern(r"[0-9]{9}")]
    pub buyer_tin: alloc::string::String,
    /// Buyer address
    #[libertas_size(min = 1)]
    pub buyer_address: alloc::string::String,
}
impl SellerMortgageParty {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let field = if self.buyer.trim().is_empty() {
            Some("buyer")
        } else if self.buyer_tin.len() != 9 || !self.buyer_tin.bytes().all(|b| b.is_ascii_digit()) {
            // Legal TIN format: exactly nine ASCII digits.
            Some("buyer_tin")
        } else if self.buyer_address.trim().is_empty() {
            Some("buyer_address")
        } else {
            None
        };
        field.map(|field| ValidationIssue::new("SELLER_MORTGAGE_BUYER_INVALID").at_field(field))
    }
}
/// Interest received from financing the buyer of your home
/// Schedule B line 1: enter interest separately from repayment of principal.
/// Buyer facts are personal records, not a Form 1098 mortgage-expense import.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SellerMortgageInterest {
    /// Buyer name, taxpayer identification number and address
    pub buyer: SellerMortgageParty,
    /// Interest received in 2026, excluding principal
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub interest: i64,
}
