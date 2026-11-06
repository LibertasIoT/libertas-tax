//! Form 8615 source-return coordination. Amounts entering arithmetic are whole dollars.

// Coverage: Form 8615 — Supported (audited 2026-09-24).
//   Lines: Lines 1–18: net unearned income after $2,700/$1,350+directly connected deductions,
//   taxable-income limit, parent's taxable income and preferential (QD, LTCG, 28%, 1250)
//   allocation, other children's shares, Schedule D/QDCG worksheets. Unit B11 KIDS (2026-10-11):
//   the parent's Form 2555 (ParentTax.foreign_excluded: line 6 is taxable income plus the
//   exclusion, line 10 the tax on that total) and the parent's and other children's Form 4952
//   line 4g election (investment_election, line 3 of the line 9 worksheets) are calculated; the
//   dependent standard deduction and Form 8615 earned income stay separate definitions.
//   Refused by confirmation: Schedule J on the parent's or another child's return; the
//   child's own Form 4952 election.
//   Evidence: M/kiddie.rs, federal_rules.rs kiddie_tax_*; T/health_accounts.rs
//   kiddie_parent_allocation_is_limited_to_child_taxable_income; T/family.rs
//   kiddie_investment_property_tax_uses_direct_expense_worksheet,
//   T2-cy: BusinessIncomeChoice gains NotNeeded and the page's allowed sets; a farm's stored
//   Services under Form 8615 is not known and taken as Capital with a warning (for_8615); the dependent
//   standard deduction applies when the spouse on a joint return can be claimed (i1040 line 12e).
//   T3-small (2026-09-27): a Schedule C business on More: Nothing (stored Services) under
//   Form 8615 (young_8615) gets a review note naming it (services_business_note); T/family.rs
//   services_business_note_appears_only_under_form_8615.
//   T2-src: BusinessIncomeSource::MixedThirtyPercent is 30% of net profit after half SE and is
//   taken as Capital with a warning without net profit (i8615 "Earned income");
//   WorkValue/BusinessIncomeChoice
//   are the page answers (see interview.rs T2-src block);
//   property_child_income_recapture_reconciles_with_and_without_wages; IRS i8615 (+draft).
//   u5-losses (2026-09-28): the parent facts have their page, "Your parent's return (Form
//   8615)" (TaxSection::ParentTax, after Other taxes, SaveParentTax/EstimateParentTax,
//   ParentReturn: status, taxable, gains fold, brothers and sisters, ParentSituation), in the
//   walk exactly while Form 8615 applies (young_8615 and line 1 over $2,700, read from the
//   calculation line kiddie_tax_unearned_income, which an unsupported estimate keeps); owned by
//   the page and pruned outside it. Lines A/B (parent's name and SSN) are printed-only and wait
//   for printing. Unsettled / FamilyWorksheets are refused by name with their lines. Rules on
//   both sides: PARENT_CHILD_TWICE, PARENT_CHILD_GAINS_ABOVE_NET, PARENT_CHILD_GAIN_ABOVE_SOURCE.
//   Parent elections (2026-10-09): the parents' election to report a child's interest and dividends
//   (Form 8814) is supported by child_election.rs and is independent of this form. A child whose
//   income a parent reports there files no return and so no Form 8615; a parent's Form 8814 tax is
//   never part of the parent's line 16 amount here (i8615 line 10) and a brother or sister reported
//   there is not one of the others (i8615 line 7); a filer may hold both a parent page and an
//   election of their own. Still limited: the Schedule J combination above. Test: src/tests/family/family.rs child_election_is_independent_of_form_8615.
//   Tests: src/tests/family/losses.rs parent_page_matches_the_hand_computed_form_8615,
//   a_brother_on_the_parent_page_shares_the_parent_rate, form_8615_starts_above_2700_...,
//   form_8615_person_tests_decide_the_parent_page; i8615 2026 draft fetched 2026-09-28.

use crate::FederalStatus;
use crate::kernel::phase;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub use crate::parent_tax_source::{
    ChildGains, ChildSpecialGains, OtherChildTax, ParentGains, ParentTax,
};

impl ParentGains {
    fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.investment_election)
            && i128::from(self.investment_election)
                <= i128::from(self.qualified) + i128::from(self.long_term.min(self.combined).max(0))
            && (0..=100000000000).contains(&self.qualified)
            && (-100000000000..=100000000000).contains(&self.long_term)
            && (-100000000000..=100000000000).contains(&self.combined)
            && (0..=100000000000).contains(&self.collectibles)
            && (0..=100000000000).contains(&self.unrecaptured)
    }
    fn preferential(&self) -> i64 {
        use crate::kernel::ratio::dollars;
        dollars(self.qualified)
            + phase::floor(phase::cap(dollars(self.long_term), [dollars(self.combined)]), 0)
    }
}

impl ChildSpecialGains {
    fn valid(&self, capital: i64) -> bool {
        use crate::kernel::ratio::dollars;
        (1..=100000000000).contains(&self.net_capital)
            && dollars(self.net_capital) > 0
            && dollars(capital) <= dollars(self.net_capital)
            && (0..=100000000000).contains(&self.collectibles)
            && (0..=100000000000).contains(&self.unrecaptured)
    }
    fn allocated(&self, capital: i64) -> i64 {
        use crate::kernel::ratio::dollars;
        special_allocation(dollars(self.collectibles), capital, dollars(self.net_capital))
            + special_allocation(dollars(self.unrecaptured), capital, dollars(self.net_capital))
    }
}
impl ChildGains {
    fn valid(&self, net: i64) -> bool {
        (0..=100000000000).contains(&self.qualified)
            && (0..=100000000000).contains(&self.capital)
            && (0..=100000000000).contains(&self.investment_election)
            && self.investment_election <= self.qualified + self.capital
            && self.qualified + self.capital <= net
            && self.preferential() <= crate::kernel::ratio::dollars(net)
            && self.special.as_ref().is_none_or(|s| s.valid(self.capital))
    }
    fn preferential(&self) -> i64 {
        use crate::kernel::ratio::dollars;
        dollars(self.qualified) + dollars(self.capital)
    }
}

#[cfg(test)]
pub(crate) use crate::section_labels::PARENT_TAX_SECTION as SECTION;

/// Your parent's return (Form 8615)
/// Your investment income over $2,700 is taxed at your parent's rate. Copy these from your parent's 2026 return, and from the Form 8615 of each brother or sister whose tax uses the same parent.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ParentReturn {
    /// Parent's filing status
    /// Parents filing jointly: use their joint return. Married parents filing separately, or unmarried parents living together: the parent with more taxable income. Divorced or separated: the parent you lived with most of the year.
    #[libertas_default(Joint)]
    pub status: FederalStatus,
    /// Parent's taxable income
    /// Form 1040 line 15 of that return; zero if it is negative.
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000, step = 1)]
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// Parent's qualified dividends and capital gains
    /// Leave it off when that return has neither qualified dividends nor a capital gain.
    /// [NullText]
    /// The parent's return has no qualified dividends or net capital gain
    /// [SetText]
    /// The parent's qualified dividends and capital gains are
    pub gains: Option<ParentGains>,
    /// Parent's Form 2555 excluded income and housing deduction
    /// Lines 45 and 50 of the parent's Form 2555 added together (line 2c of the parent's Foreign Earned Income Tax Worksheet). Leave absent when the parent files no Form 2555. The App adds it to the parent's taxable income on line 6 and figures the line 10 tax on that total, as the worksheet does.
    /// [NullText]
    /// The parent has no Form 2555 excluded income or housing deduction
    /// [SetText]
    /// The parent's Form 2555 excluded income and housing deduction total
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub foreign_excluded: Option<i64>,
    /// Brothers and sisters using the same parent's return
    /// Each other child whose Form 8615 uses this parent's return, even if not a dependent; not a brother or sister whose income your parent reported on Form 8814. Leave it empty when there are none.
    /// ----
    /// Child
    pub others: Vec<OtherChildTax>,
    /// Anything else?
    #[libertas_default(Nothing)]
    pub special: ParentSituation,
}

/// Anything else?
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
pub enum ParentSituation {
    /// None of these
    Nothing,
    /// Not settled yet
    /// Which parent's return to use is not settled, or a brother's or sister's Form 8615 is not finished. Not calculated until it is.
    Unsettled,
    /// Schedule J on the parent's or a brother's or sister's return
    /// Your parent, or a brother or sister whose Form 8615 uses the same parent, averages farm or fishing income on Schedule J. Not calculated yet.
    FamilyWorksheets,
}

impl ParentReturn {
    // The page onto the stored parent facts: each choice of "Anything
    // else?" is the two confirmations.
    pub(crate) fn record(&self) -> ParentTax {
        let (complete, ordinary) = match self.special {
            ParentSituation::Nothing => (true, true),
            ParentSituation::Unsettled => (false, true),
            ParentSituation::FamilyWorksheets => (true, false),
        };
        ParentTax {
            status: self.status,
            taxable: self.taxable,
            gains: self.gains.clone(),
            others: self.others.clone(),
            complete,
            ordinary,
            foreign_excluded: self.foreign_excluded,
        }
    }
    pub(crate) fn from_record(parent: &ParentTax) -> Self {
        let special = match (parent.complete, parent.ordinary) {
            (true, true) => ParentSituation::Nothing,
            (true, false) => ParentSituation::FamilyWorksheets,
            (false, _) => ParentSituation::Unsettled,
        };
        Self {
            status: parent.status,
            taxable: parent.taxable,
            gains: parent.gains.clone(),
            foreign_excluded: parent.foreign_excluded,
            others: parent.others.clone(),
            special,
        }
    }
    // The server twins of the page's client rules.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        for (i, child) in self.others.iter().enumerate() {
            if self.others[..i].iter().any(|o| o.label == child.label) {
                return Some(
                    crate::validation::ValidationIssue::new("PARENT_CHILD_TWICE")
                        .at_field("label")
                        .in_list("others", i),
                );
            }
            let Some(gains) = &child.gains else {
                continue;
            };
            if i128::from(gains.qualified) + i128::from(gains.capital) > i128::from(child.net) {
                return Some(
                    crate::validation::ValidationIssue::new("PARENT_CHILD_GAINS_ABOVE_NET")
                        .at_field("gains")
                        .in_list("others", i),
                );
            }
            if gains.special.as_ref().is_some_and(|s| gains.capital > s.net_capital) {
                return Some(
                    crate::validation::ValidationIssue::new("PARENT_CHILD_GAIN_ABOVE_SOURCE")
                        .at_field("capital")
                        .in_field("gains")
                        .in_list("others", i),
                );
            }
        }
        None
    }
}

impl ParentTax {
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.taxable)
            && self.foreign_excluded.is_none_or(|v| (0..=100000000000).contains(&v))
            && self.gains.as_ref().is_none_or(ParentGains::valid)
            && self.others.len() <= 100
            && self.others.iter().all(|v| {
                !v.label.trim().is_empty()
                    && (0..=100000000000).contains(&v.net)
                    && v.gains.as_ref().is_none_or(|g| g.valid(v.net))
            })
    }
    pub(crate) fn calculate(
        &self,
        net: i64,
        taxable: i64,
        status: FederalStatus,
        child_pref: [i64; 2],
        child_capital: i64,
        child_special: [i64; 3], // original net capital gain, Schedule D lines 18/19
    ) -> (i64, i64) {
        use crate::{
            federal_rules::{preferential_tax_with_special_gains, schedule_d_tax},
            kernel::ratio::{dollars, fraction},
        };
        // Line 6: the parent's taxable income, with the Form 2555 exclusion
        // added back (line 3 of the Foreign Earned Income Tax Worksheet);
        // line 10 is the tax on that total (line 4 of the worksheet).
        let parent = dollars(self.taxable) + self.foreign_excluded.map_or(0, dollars);
        let children = net + self.others.iter().map(|v| dollars(v.net)).sum::<i64>();
        let pref = self.gains.as_ref().map_or(0, ParentGains::preferential);
        let sibling_pref = self
            .others
            .iter()
            .filter_map(|v| v.gains.as_ref())
            .map(ChildGains::preferential)
            .sum::<i64>();
        let parent_net = self.gains.as_ref().map_or(0, |g| {
            phase::floor(phase::cap(dollars(g.long_term), [dollars(g.combined)]), 0)
        });
        let sibling_capital = self
            .others
            .iter()
            .filter_map(|v| v.gains.as_ref())
            .map(|g| dollars(g.capital))
            .sum::<i64>();
        let pools = |net: i64| {
            let unrecaptured =
                phase::cap(self.gains.as_ref().map_or(0, |g| dollars(g.unrecaptured)), [net]);
            let collectibles = phase::cap(
                self.gains.as_ref().map_or(0, |g| dollars(g.collectibles)),
                [net - unrecaptured],
            );
            (collectibles, unrecaptured)
        };
        let original = pools(parent_net);
        let combined_net = parent_net + sibling_capital + child_capital;
        let [child_net, child_28, child_25] = child_special;
        let allocated_28 = special_allocation(child_28, child_capital, child_net);
        let allocated_25 = special_allocation(child_25, child_capital, child_net);
        let parent_special =
            self.gains.as_ref().map_or(0, |g| dollars(g.collectibles) + dollars(g.unrecaptured));
        let sibling_special = self
            .others
            .iter()
            .filter_map(|v| v.gains.as_ref())
            .map(|g| g.special.as_ref().map_or(0, |s| s.allocated(dollars(g.capital))))
            .sum::<i64>();
        let original_unrecaptured = self.gains.as_ref().map_or(0, |g| dollars(g.unrecaptured))
            + self
                .others
                .iter()
                .filter_map(|v| v.gains.as_ref())
                .filter_map(|g| g.special.as_ref())
                .map(|s| dollars(s.unrecaptured))
                .sum::<i64>();
        // Line 3 of the line 9 worksheets: the Form 4952 line 4g election of
        // the parent and every other child comes off the preferential income;
        // the parent's own election is already in the line 10 tax.
        let parent_election = self.gains.as_ref().map_or(0, |g| dollars(g.investment_election));
        let sibling_election = self
            .others
            .iter()
            .filter_map(|v| v.gains.as_ref())
            .map(|g| dollars(g.investment_election))
            .sum::<i64>();
        let increase = phase::floor(
            schedule_d_tax(
                parent + children,
                phase::floor(
                    pref + sibling_pref + child_pref[1] - parent_election - sibling_election,
                    0,
                ),
                phase::cap(
                    parent_special + sibling_special + allocated_28 + allocated_25,
                    [combined_net],
                ),
                phase::cap(original_unrecaptured + child_25, [combined_net]),
                child_28 > 0
                    || self.gains.as_ref().is_some_and(|g| g.collectibles > 0)
                    || self
                        .others
                        .iter()
                        .filter_map(|v| v.gains.as_ref())
                        .filter_map(|g| g.special.as_ref())
                        .any(|s| s.collectibles > 0),
                self.status,
            ) - preferential_tax_with_special_gains(
                parent,
                phase::floor(pref - parent_election, 0),
                original.0,
                original.1,
                self.status,
            ),
            0,
        );
        let allocated =
            if children == 0 { 0 } else { fraction(increase, fraction(net, 1000, children), 1000) };
        let remaining_capital = phase::floor(child_net - child_capital, 0);
        let remaining_tax = schedule_d_tax(
            taxable - net,
            child_pref[0] - child_pref[1],
            phase::cap(child_28 + child_25 - allocated_28 - allocated_25, [remaining_capital]),
            phase::cap(child_25, [remaining_capital]),
            child_28 > 0,
            status,
        );
        let own_tax = schedule_d_tax(
            taxable,
            child_pref[0],
            phase::cap(child_28 + child_25, [child_net]),
            phase::cap(child_25, [child_net]),
            child_28 > 0,
            status,
        );
        (allocated, phase::floor(allocated + remaining_tax, own_tax))
    }
}

// Form 8615 Line 5 Worksheets #1–#3. Direct amounts are allowed deductions,
// split into all unearned-income expenses and the preferential-income portion.
// Results retain separate dividend and gain allocations for on-screen reuse.
pub(crate) fn gain_allocation(
    net: i64,
    unearned: i64,
    agi: i64,
    deduction: i64,
    qualified: i64,
    capital: i64,
    direct: [i64; 2],
) -> [i64; 2] {
    use crate::kernel::ratio::fraction;
    let total = qualified + capital;
    if total == 0 || net == 0 {
        return [0, 0];
    }
    let line2 = phase::floor(
        crate::year_constants::KIDDIE_BASE_DEDUCTION_DOLLARS + direct[0],
        crate::year_constants::KIDDIE_UNEARNED_THRESHOLD_DOLLARS,
    );
    let direct_dividend = fraction(direct[1], fraction(qualified, 1000, total), 1000);
    let direct_gain = direct[1] - direct_dividend;
    let (dividend_deduction, gain_deduction) = if net == unearned - line2
        && line2 == crate::year_constants::KIDDIE_UNEARNED_THRESHOLD_DOLLARS
    {
        (
            fraction(
                crate::year_constants::KIDDIE_UNEARNED_THRESHOLD_DOLLARS,
                fraction(qualified, 1000, unearned).min(1000),
                1000,
            ),
            fraction(
                crate::year_constants::KIDDIE_UNEARNED_THRESHOLD_DOLLARS,
                fraction(capital, 1000, unearned).min(1000),
                1000,
            ),
        )
    } else if net == unearned - line2 {
        let dividend_ratio = fraction(qualified, 1000, unearned).min(1000);
        let gain_ratio = fraction(capital, 1000, unearned).min(1000 - dividend_ratio);
        (
            direct_dividend
                + fraction(
                    crate::year_constants::KIDDIE_BASE_DEDUCTION_DOLLARS,
                    dividend_ratio,
                    1000,
                ),
            direct_gain
                + fraction(crate::year_constants::KIDDIE_BASE_DEDUCTION_DOLLARS, gain_ratio, 1000),
        )
    } else {
        let deduction = fraction(
            phase::floor(deduction - direct[1], 0),
            fraction(total, 1000, agi).min(1000),
            1000,
        );
        let dividend = fraction(deduction, fraction(qualified, 1000, total), 1000);
        (direct_dividend + dividend, direct_gain + deduction - dividend)
    };
    let dividends = phase::cap(phase::floor(qualified - dividend_deduction, 0), [net]);
    [dividends, phase::cap(phase::floor(capital - gain_deduction, 0), [net - dividends])]
}

/// What produces this business's income
/// Check the business's income, equipment, inventory and service records; for mixed income retain the supported market value of the owner's work.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessIncomeSource {
    /// Your personal services; capital is not a material factor
    Services,
    /// Capital; your personal services are not a material factor
    Capital,
    /// Both your personal services and capital are material factors
    Mixed {
        /// Reasonable annual compensation for your personal services
        /// Enter the supported market value of your work, before applying the tax limit. This is not an owner draw, profit distribution or tax amount. The App applies the 30% limit when there is net profit; it does not apply that limit when there is no net profit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        compensation: i64,
    },
    /// Both, and your work is worth at least 30% of your share of the net profit
    // Without profit this answer supplies no actual compensation amount; it cannot be
    // reconstructed from a percentage of no profit. WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL
    // retains the existing conservative source boundary until an actual value is supplied.
    MixedThirtyPercent,
}
impl BusinessIncomeSource {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Mixed { compensation } => (0..=100000000000).contains(compensation),
            _ => true,
        }
    }
    // Form 8615 business earned income, before combining with other sources.
    // The caller supplies the half-SE deduction attributable to this business.
    // MixedThirtyPercent states that the compensation reaches the 30% ceiling,
    // so it is 30% of the profit after half SE; with no net profit there is no
    // ceiling (i8615 "Earned income") and the answer gives no amount: None,
    // which `for_8615` turns into Capital first.
    pub(crate) fn earned(&self, profit: i64, half_se: i64) -> Option<i64> {
        use crate::kernel::ratio::{dollars, fraction};
        match self {
            Self::Services => Some(phase::floor(profit, 0)),
            Self::Capital => Some(0),
            Self::Mixed { compensation } => {
                let reasonable = dollars(*compensation);
                Some(if profit > 0 {
                    phase::cap(reasonable, [fraction(phase::floor(profit - half_se, 0), 30, 100)])
                } else {
                    reasonable
                })
            }
            Self::MixedThirtyPercent => {
                (profit > 0).then(|| fraction(phase::floor(profit - half_se, 0), 30, 100))
            }
        }
    }
    // The source Form 8615 reads. A farm's unasked source stored as Services,
    // or the 30% answer in a year without net profit, gives no actual amount.
    // That unavailable source is taken as Capital: none of
    // the profit is earned income, the conservative path. The flag says the
    // source was assumed, for the caller's warning.
    pub(crate) fn for_8615(&self, farming: bool, profit: i64) -> (&Self, bool) {
        match self {
            Self::Services if farming => (&Self::Capital, true),
            Self::MixedThirtyPercent if profit <= 0 => (&Self::Capital, true),
            other => (other, false),
        }
    }
}

// The review note for a Schedule C business whose income source stays
// Services (the More: Nothing answer) on a return Form 8615 applies to.
#[cfg(test)]
pub(crate) fn services_business_note(label: &str) -> alloc::string::String {
    alloc::format!(
        "Business {label} is treated as income from your own work for Form 8615. If equipment, inventory or other capital earns a material part of its income, open More \u{2192} Something else and change it."
    )
}

/// Value of your work
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum WorkValue {
    /// My work is worth at least 30% of my share of this business's net profit
    /// No amount is needed: your earned income is 30% of the net profit after half of the self-employment tax. In a year with no net profit, enter the amount instead.
    AtLeastThirtyPercent,
    /// Enter the amount
    Amount {
        /// Reasonable annual compensation for your personal services
        /// Enter the supported market value of your work, before applying the tax limit. This is not an owner draw, profit distribution or tax amount. The App applies the 30% limit when there is net profit; it does not apply that limit when there is no net profit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        compensation: i64,
    },
}
impl WorkValue {
    /// The stored income source for "Both" with this answer.
    pub fn record(&self) -> BusinessIncomeSource {
        match self {
            Self::AtLeastThirtyPercent => BusinessIncomeSource::MixedThirtyPercent,
            Self::Amount { compensation } => {
                BusinessIncomeSource::Mixed { compensation: *compensation }
            }
        }
    }
    /// The answer a stored "Both" source reopens as.
    pub fn from_record(source: &BusinessIncomeSource) -> Option<Self> {
        match source {
            BusinessIncomeSource::MixedThirtyPercent => Some(Self::AtLeastThirtyPercent),
            BusinessIncomeSource::Mixed { compensation } => {
                Some(Self::Amount { compensation: *compensation })
            }
            _ => None,
        }
    }
}

/// What produces this business's income
/// Check the business's income, equipment, inventory and service records; for mixed income retain the supported market value of the owner's work.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum BusinessIncomeChoice {
    /// Not needed for this return
    NotNeeded,
    /// Your personal services; capital is not a material factor
    Services,
    /// Capital; your personal services are not a material factor
    Capital,
    /// Both your personal services and capital are material factors
    Mixed {
        /// Value of your work
        value: WorkValue,
    },
}
impl BusinessIncomeChoice {
    /// The choices the page offers: every answer when the return reads the
    /// source, otherwise "Not needed" alone, which the client selects itself.
    pub fn allowed(needed: bool) -> Vec<i32> {
        use libertas_macros::variant_index;
        if needed {
            alloc::vec![
                variant_index!(BusinessIncomeChoice::Services) as i32,
                variant_index!(BusinessIncomeChoice::Capital) as i32,
                variant_index!(BusinessIncomeChoice::Mixed) as i32,
            ]
        } else {
            alloc::vec![variant_index!(BusinessIncomeChoice::NotNeeded) as i32]
        }
    }
    /// This choice's index in the allowed set.
    pub fn index(&self) -> i32 {
        use libertas_macros::variant_index;
        (match self {
            Self::NotNeeded => variant_index!(BusinessIncomeChoice::NotNeeded),
            Self::Services => variant_index!(BusinessIncomeChoice::Services),
            Self::Capital => variant_index!(BusinessIncomeChoice::Capital),
            Self::Mixed { .. } => variant_index!(BusinessIncomeChoice::Mixed),
        }) as i32
    }
    /// The stored income source; "Not needed" stores what the "Nothing
    /// else" branch stores.
    pub fn record(&self) -> BusinessIncomeSource {
        match self {
            Self::NotNeeded | Self::Services => BusinessIncomeSource::Services,
            Self::Capital => BusinessIncomeSource::Capital,
            Self::Mixed { value } => value.record(),
        }
    }
    /// The choice a stored income source reopens as.
    pub fn from_record(source: &BusinessIncomeSource) -> Self {
        match source {
            BusinessIncomeSource::Services => Self::Services,
            BusinessIncomeSource::Capital => Self::Capital,
            BusinessIncomeSource::Mixed { .. } | BusinessIncomeSource::MixedThirtyPercent => {
                Self::Mixed {
                    value: WorkValue::from_record(source)
                        .unwrap_or(WorkValue::AtLeastThirtyPercent),
                }
            }
        }
    }
}

// Form 8615 special-gain worksheets use the ratio to original NET capital gain.
// Keep each original source for the remaining-income worksheet and line 35.
pub(crate) fn special_allocation(original: i64, allocated: i64, net_capital: i64) -> i64 {
    if net_capital <= 0 {
        0
    } else {
        crate::kernel::ratio::fraction(original, allocated, net_capital)
    }
}

// The child's Form 8615 method, retained for tax-attributable-to-income
// comparisons. AGI-dependent deductions stay fixed (i8582cr line 6 worksheet).
pub(crate) struct TaxWorksheet<'a> {
    pub parent: &'a ParentTax,
    pub unearned: i64,
    pub deduction: i64,
    pub agi: i64,
    pub return_deduction: i64,
    pub qualified: i64,
    pub preference: i64,
    pub direct: [i64; 2],
    pub capital: [i64; 3],
    pub status: FederalStatus,
}
impl TaxWorksheet<'_> {
    pub(crate) fn calculate(&self, taxable: i64) -> (i64, [i64; 2], i64, i64) {
        let net = crate::kernel::phase::cap(
            crate::kernel::ratio::positive(self.unearned - self.deduction),
            [taxable],
        );
        let gains = gain_allocation(
            net,
            self.unearned,
            self.agi,
            self.return_deduction,
            self.qualified,
            self.preference - self.qualified,
            self.direct,
        );
        let (allocated, tax) = self.parent.calculate(
            net,
            taxable,
            self.status,
            [self.preference, gains.iter().sum()],
            gains[1],
            self.capital,
        );
        (net, gains, allocated, tax)
    }
}
