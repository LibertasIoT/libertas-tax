mod annual;

use crate::annual_debt_source::signed_share;
use crate::debt_reporting_source::DebtReporting;
use crate::validation::ValidationIssue;

pub(crate) fn int_printed_issue(
    v: &crate::interest_source::Form1099IntPaper,
) -> Option<ValidationIssue> {
    if v.specified_private_activity_bond_interest > v.tax_exempt_interest {
        return Some(
            ValidationIssue::new("INT_PRIVATE_ACTIVITY_ABOVE_EXEMPT").at_field("private_activity"),
        );
    }
    None
}

// Ordered source-owner validation, shared by page saves/previews and restored records.
pub(crate) fn int_rule(v: &crate::interest_source::FederalInterest) -> Option<&'static str> {
    int_issue(v).map(|issue| issue.key)
}
pub(crate) fn int_issue(v: &crate::interest_source::FederalInterest) -> Option<ValidationIssue> {
    int_input_issue(v, true)
}

pub(crate) fn int_stored_issue(
    v: &crate::interest_source::FederalInterest,
) -> Option<ValidationIssue> {
    int_input_issue(v, false)
}

fn int_input_issue(
    v: &crate::interest_source::FederalInterest,
    source_alignment: bool,
) -> Option<ValidationIssue> {
    if let Some(party) = &v.seller_mortgage
        && let Some(issue) = party.issue()
    {
        return Some(issue.in_field("seller_mortgage"));
    }

    if v.seller_mortgage.is_some()
        && (v.nominee.is_some()
            || v.adjustments.is_some()
            || v.after_death.is_some()
            || v.special != crate::interest_source::InterestSpecial::NoneOfThese
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
        return annual::int_issue(v, records, source_alignment);
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
    if source_alignment && n.taxable + a.taxable > v.taxable {
        return Some(ValidationIssue::new("INT_OUTSIDE_TAXABLE_ABOVE_BOX").at_field("taxable"));
    }
    if source_alignment && n.us_obligations + a.us_obligations > v.us_obligations {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_US_OBLIGATIONS_ABOVE_BOX").at_field("us_obligations"),
        );
    }
    if source_alignment && n.exempt + a.exempt > v.exempt {
        return Some(ValidationIssue::new("INT_OUTSIDE_EXEMPT_ABOVE_BOX").at_field("exempt"));
    }
    if source_alignment && n.private_activity + a.private_activity > v.private_activity {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX")
                .at_field("private_activity"),
        );
    }
    if source_alignment && n.foreign_tax + a.foreign_tax > v.foreign_tax {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_FOREIGN_TAX_ABOVE_BOX").at_field("foreign_tax"),
        );
    }
    if source_alignment && n.bond_premium + a.bond_premium > v.bond_premium {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_BOND_PREMIUM_ABOVE_BOX").at_field("bond_premium"),
        );
    }
    if source_alignment
        && n.treasury_bond_premium + a.treasury_bond_premium > v.treasury_bond_premium
    {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_TREASURY_BOND_PREMIUM_ABOVE_BOX")
                .at_field("treasury_bond_premium"),
        );
    }
    if source_alignment
        && n.tax_exempt_bond_premium + a.tax_exempt_bond_premium > v.tax_exempt_bond_premium
    {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX")
                .at_field("tax_exempt_bond_premium"),
        );
    }
    if source_alignment && n.accrued_interest + a.accrued_interest > v.accrued_interest {
        return Some(
            ValidationIssue::new("INT_OUTSIDE_ACCRUED_INTEREST_ABOVE_BOX")
                .at_field("accrued_interest"),
        );
    }
    if source_alignment && n.market_discount + a.market_discount > v.market_discount {
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
    if source_alignment
        && v.private_activity - n.private_activity - a.private_activity
            > v.exempt - n.exempt - a.exempt
    {
        return Some(
            ValidationIssue::new("INT_RETAINED_PRIVATE_ACTIVITY_ABOVE_EXEMPT")
                .at_field("private_activity"),
        );
    }
    if v.adjustments.as_ref().is_some_and(|x| {
        x.previously_reported < 0
            || source_alignment
                && x.previously_reported > v.us_obligations - n.us_obligations - a.us_obligations
    }) {
        return Some(
            ValidationIssue::new("INTEREST_PRIOR_ABOVE_RETAINED")
                .at_field("previously_reported")
                .in_field("adjustments"),
        );
    }
    None
}
pub(crate) fn div_rule(v: &crate::dividend_source::FederalDividend) -> Option<&'static str> {
    div_issue(v).map(|issue| issue.key)
}
pub(crate) fn div_issue(v: &crate::dividend_source::FederalDividend) -> Option<ValidationIssue> {
    div_input_issue(v, true)
}

// Keep printed-box checks in the same order as the topic's required answers.
pub(crate) fn div_paper_issue(
    v: &crate::dividend_source::Form1099DivPaper,
    reit_answer_missing: bool,
    exclusion_answer_missing: bool,
) -> Option<ValidationIssue> {
    let issue = |key, field| ValidationIssue::new(key).at_field(field);
    if v.qualified > v.ordinary {
        return Some(issue("DIV_QUALIFIED_ABOVE_ORDINARY", "qualified"));
    }
    if reit_answer_missing {
        return Some(issue("DIV_REIT_ELIGIBILITY_NEEDED", "reit_eligible").in_field("answers"));
    }
    if v.unrecaptured_distributions > v.capital_distributions {
        return Some(issue("DIV_2B_ABOVE_2A", "unrecaptured_distributions"));
    }
    if v.collectible_distributions > v.capital_distributions {
        return Some(issue("DIV_2D_ABOVE_2A", "collectible_distributions"));
    }
    if v.section_1202_gain > v.capital_distributions {
        return Some(issue("DIV_SECTION_1202_ABOVE_2A", "section_1202_gain"));
    }
    if exclusion_answer_missing {
        return Some(
            issue("DIV_SECTION_1202_NEEDS_EXCLUSION", "section_1202_exclusion").in_field("more"),
        );
    }
    if v.private_activity_dividends > v.exempt_dividends {
        return Some(issue("DIV_13_ABOVE_12", "private_activity_dividends"));
    }
    if v.section_897_ordinary > v.ordinary {
        return Some(issue("DIV_897_ORDINARY_ABOVE_BOX", "section_897_ordinary"));
    }
    if v.section_897_capital > v.capital_distributions {
        return Some(issue("DIV_897_CAPITAL_ABOVE_BOX", "section_897_capital"));
    }
    None
}

// Printed amounts are admitted before the topic supplies share-lot records.
pub(crate) fn div_printed_issue(
    v: &crate::dividend_source::Form1099DivPaper,
) -> Option<ValidationIssue> {
    div_paper_issue(v, false, false)
        .or_else(|| {
            div_capital_amount_issue(
                v.nondividend_distributions.unwrap_or(0),
                v.cash_liquidation.unwrap_or(0),
            )
        })
        .or_else(|| div_retained_reit_issue(v.section199a.unwrap_or(0), v.ordinary.unwrap_or(0)))
        .or_else(|| {
            div_qualified_reit_issue(
                v.qualified.unwrap_or(0) + v.section199a.unwrap_or(0),
                v.ordinary.unwrap_or(0),
            )
        })
        .or_else(|| {
            div_capital_parts_issue(
                v.unrecaptured_distributions.unwrap_or(0)
                    + v.collectible_distributions.unwrap_or(0),
                v.capital_distributions.unwrap_or(0),
            )
        })
}

fn div_capital_amount_issue(nondividend: i64, cash: i64) -> Option<ValidationIssue> {
    for (field, amount) in [("nondividend_distributions", nondividend), ("cash_liquidation", cash)]
    {
        if !(0..=100_000_000_000).contains(&amount) {
            return Some(ValidationIssue::new("DIV_CAPITAL_RETURNS_RECONCILE").at_field(field));
        }
    }
    None
}

fn div_retained_reit_issue(section199a: i64, ordinary: i64) -> Option<ValidationIssue> {
    (section199a > ordinary).then(|| {
        ValidationIssue::new("DIV_RETAINED_SECTION_199A_ABOVE_ORDINARY").at_field("section199a")
    })
}

fn div_qualified_reit_issue(qualified_and_reit: i64, ordinary: i64) -> Option<ValidationIssue> {
    (qualified_and_reit > ordinary)
        .then(|| ValidationIssue::new("DIV_OUTSIDE_QUALIFIED_REIT_OVERLAP").at_field("ordinary"))
}

fn div_capital_parts_issue(
    unrecaptured_and_collectible: i64,
    capital: i64,
) -> Option<ValidationIssue> {
    (unrecaptured_and_collectible > capital).then(|| {
        ValidationIssue::new("DIV_OUTSIDE_CAPITAL_PARTS_OVERLAP").at_field("capital_distributions")
    })
}

pub(crate) fn div_stored_issue(
    v: &crate::dividend_source::FederalDividend,
) -> Option<ValidationIssue> {
    div_input_issue(v, false)
}

fn div_input_issue(
    v: &crate::dividend_source::FederalDividend,
    source_alignment: bool,
) -> Option<ValidationIssue> {
    let key = "DIV_CAPITAL_RETURNS_RECONCILE";
    if v.capital_returns.len() > 100 {
        return Some(ValidationIssue::new(key).at_field("capital_returns"));
    }
    if let Some(issue) = div_capital_amount_issue(v.nondividend_distributions, v.cash_liquidation) {
        return Some(issue);
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
    if source_alignment {
        for (field, total, stated) in [
            (
                "nondividend_distributions",
                v.capital_returns
                    .iter()
                    .filter(|lot| {
                        lot.kind == crate::dividend_source::CapitalDistributionKind::Nondividend
                    })
                    .map(|lot| lot.amount)
                    .sum::<i64>(),
                v.nondividend_distributions,
            ),
            (
                "cash_liquidation",
                v.capital_returns
                    .iter()
                    .filter(|lot| {
                        lot.kind != crate::dividend_source::CapitalDistributionKind::Nondividend
                    })
                    .map(|lot| lot.amount)
                    .sum::<i64>(),
                v.cash_liquidation,
            ),
            (
                "noncash_liquidation",
                v.capital_returns
                    .iter()
                    .flat_map(|lot| &lot.property)
                    .map(|p| p.value)
                    .sum::<i64>(),
                v.noncash_liquidation,
            ),
        ] {
            if total != stated {
                return Some(ValidationIssue::new(key).at_field(field));
            }
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
    if source_alignment && n.ordinary + a.ordinary > v.ordinary {
        return Some(ValidationIssue::new("DIV_OUTSIDE_ORDINARY_ABOVE_BOX").at_field("ordinary"));
    }
    if source_alignment && n.qualified + a.qualified > v.qualified {
        return Some(ValidationIssue::new("DIV_OUTSIDE_QUALIFIED_ABOVE_BOX").at_field("qualified"));
    }
    if source_alignment && n.capital + a.capital_distributions > v.capital_distributions {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_CAPITAL_ABOVE_BOX").at_field("capital_distributions"),
        );
    }
    if source_alignment
        && n.unrecaptured + a.unrecaptured_distributions > v.unrecaptured_distributions
    {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_UNRECAPTURED_ABOVE_BOX")
                .at_field("unrecaptured_distributions"),
        );
    }
    if source_alignment && n.collectible + a.collectible_distributions > v.collectible_distributions
    {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_COLLECTIBLE_ABOVE_BOX")
                .at_field("collectible_distributions"),
        );
    }
    if source_alignment && n.section_1202 + a.section_1202_gain > v.section_1202_gain {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_SECTION_1202_ABOVE_BOX")
                .at_field("section_1202_gain"),
        );
    }
    if source_alignment && n.section_199a + a.section199a > v.section199a.amount() {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_SECTION_199A_ABOVE_BOX").at_field("section199a"),
        );
    }
    if source_alignment && n.exempt + a.exempt_dividends > v.exempt_dividends {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_EXEMPT_ABOVE_BOX").at_field("exempt_dividends"),
        );
    }
    if source_alignment
        && n.private_activity + a.private_activity_dividends > v.private_activity_dividends
    {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_PRIVATE_ACTIVITY_ABOVE_BOX")
                .at_field("private_activity_dividends"),
        );
    }
    if source_alignment && n.foreign_tax + a.foreign_tax > v.foreign_tax {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_FOREIGN_TAX_ABOVE_BOX").at_field("foreign_tax"),
        );
    }
    if source_alignment && n.section_897_ordinary + a.section_897_ordinary > v.section_897_ordinary
    {
        return Some(
            ValidationIssue::new("DIV_OUTSIDE_SECTION_897_ORDINARY_ABOVE_BOX")
                .at_field("section_897_ordinary"),
        );
    }
    if source_alignment && n.section_897_capital + a.section_897_capital > v.section_897_capital {
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
    if source_alignment
        && v.qualified - n.qualified - a.qualified > v.ordinary - n.ordinary - a.ordinary
    {
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
    if source_alignment
        && v.private_activity_dividends - n.private_activity - a.private_activity_dividends
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
    if source_alignment
        && let Some(issue) = div_retained_reit_issue(
            v.section199a.amount() - n.section_199a - a.section199a,
            v.ordinary - n.ordinary - a.ordinary,
        )
    {
        return Some(issue);
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
    if source_alignment
        && v.section_897_ordinary - n.section_897_ordinary - a.section_897_ordinary
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
    if source_alignment
        && v.section_897_capital - n.section_897_capital - a.section_897_capital
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
    if source_alignment
        && v.unrecaptured_distributions - n.unrecaptured - a.unrecaptured_distributions
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
    if source_alignment
        && v.collectible_distributions - n.collectible - a.collectible_distributions
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
    if source_alignment
        && v.section_1202_gain - n.section_1202 - a.section_1202_gain
            > v.capital_distributions - n.capital - a.capital_distributions
    {
        return Some(
            ValidationIssue::new("DIV_RETAINED_SECTION_1202_ABOVE_CAPITAL")
                .at_field("section_1202_gain"),
        );
    }
    if let Some(issue) = div_qualified_reit_issue(n.qualified + n.section_199a, n.ordinary)
        .or_else(|| div_qualified_reit_issue(a.qualified + a.section199a, a.ordinary))
        .or_else(|| {
            if source_alignment {
                div_qualified_reit_issue(
                    v.qualified - n.qualified - a.qualified + v.section199a.amount()
                        - n.section_199a
                        - a.section199a,
                    v.ordinary - n.ordinary - a.ordinary,
                )
            } else {
                None
            }
        })
    {
        return Some(issue);
    }
    if let Some(issue) = div_capital_parts_issue(n.unrecaptured + n.collectible, n.capital)
        .or_else(|| {
            div_capital_parts_issue(
                a.unrecaptured_distributions + a.collectible_distributions,
                a.capital_distributions,
            )
        })
        .or_else(|| {
            if source_alignment {
                div_capital_parts_issue(
                    v.unrecaptured_distributions - n.unrecaptured - a.unrecaptured_distributions
                        + v.collectible_distributions
                        - n.collectible
                        - a.collectible_distributions,
                    v.capital_distributions - n.capital - a.capital_distributions,
                )
            } else {
                None
            }
        })
    {
        return Some(issue);
    }
    None
}
pub(crate) fn oid_rule(v: &crate::oid_source::FederalOid) -> Option<&'static str> {
    oid_issue(v).map(|issue| issue.key)
}
pub(crate) fn oid_issue(v: &crate::oid_source::FederalOid) -> Option<ValidationIssue> {
    oid_input_issue(v, true)
}

pub(crate) fn oid_stored_issue(v: &crate::oid_source::FederalOid) -> Option<ValidationIssue> {
    oid_input_issue(v, false)
}

fn oid_input_issue(
    v: &crate::oid_source::FederalOid,
    source_alignment: bool,
) -> Option<ValidationIssue> {
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
        return annual::oid_issue(v, records, source_alignment);
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
    if source_alignment && n.oid > v.oid {
        return Some(
            ValidationIssue::new("OID_NOMINEE_OID_ABOVE_BOX").at_field("oid").in_field("nominee"),
        );
    }
    if source_alignment && n.other_periodic_interest > v.other_periodic_interest {
        return Some(
            ValidationIssue::new("OID_NOMINEE_OTHER_PERIODIC_INTEREST_ABOVE_BOX")
                .at_field("other_periodic_interest")
                .in_field("nominee"),
        );
    }
    if source_alignment && n.market_discount > v.market_discount {
        return Some(
            ValidationIssue::new("OID_NOMINEE_MARKET_DISCOUNT_ABOVE_BOX")
                .at_field("market_discount")
                .in_field("nominee"),
        );
    }
    if source_alignment && n.acquisition_premium > v.acquisition_premium {
        return Some(
            ValidationIssue::new("OID_NOMINEE_ACQUISITION_PREMIUM_ABOVE_BOX")
                .at_field("acquisition_premium")
                .in_field("nominee"),
        );
    }
    if source_alignment && n.bond_premium > v.bond_premium {
        return Some(
            ValidationIssue::new("OID_NOMINEE_BOND_PREMIUM_ABOVE_BOX")
                .at_field("bond_premium")
                .in_field("nominee"),
        );
    }
    if source_alignment && n.exempt > v.exempt {
        return Some(
            ValidationIssue::new("OID_NOMINEE_EXEMPT_ABOVE_BOX")
                .at_field("exempt")
                .in_field("nominee"),
        );
    }
    if source_alignment && n.tax_exempt_bond_premium > v.tax_exempt_bond_premium {
        return Some(
            ValidationIssue::new("OID_NOMINEE_TAX_EXEMPT_BOND_PREMIUM_ABOVE_BOX")
                .at_field("tax_exempt_bond_premium")
                .in_field("nominee"),
        );
    }
    if source_alignment && n.private_activity > v.private_activity {
        return Some(
            ValidationIssue::new("OID_NOMINEE_PRIVATE_ACTIVITY_ABOVE_BOX")
                .at_field("private_activity")
                .in_field("nominee"),
        );
    }
    if source_alignment && !signed_share(v.treasury_oid, n.treasury_oid) {
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
    if source_alignment && v.private_activity - n.private_activity > v.exempt - n.exempt {
        return Some(
            ValidationIssue::new("OID_RETAINED_PRIVATE_ABOVE_EXEMPT").at_field("private_activity"),
        );
    }
    if source_alignment && v.tax_exempt_bond_premium > v.exempt
        || n.tax_exempt_bond_premium > n.exempt
        || source_alignment
            && v.tax_exempt_bond_premium - n.tax_exempt_bond_premium > v.exempt - n.exempt
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

impl crate::after_death_income_source::InterestAfterDeath {
    pub(crate) fn valid(&self, statement: &crate::interest_source::FederalInterest) -> bool {
        let mut checked = statement.clone();
        checked.after_death = Some(self.clone());
        int_rule(&checked).is_none()
    }
}

impl crate::after_death_income_source::DividendsAfterDeath {
    pub(crate) fn valid(&self, statement: &crate::dividend_source::FederalDividend) -> bool {
        let mut checked = statement.clone();
        checked.after_death = Some(self.clone());
        div_rule(&checked).is_none()
    }
}
