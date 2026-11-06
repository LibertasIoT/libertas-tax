use crate::common::Owner;
use crate::kernel::phase;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Earlier Form 8915-F
/// A 2024 or 2025 Form 8915-F whose spread income or repayments continue in 2026.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EarlierForm8915 {
    /// Tax year of that form (item A)
    pub year: Form8915Year,
    /// Year the disasters began (item B)
    // Form 8915-F: the retained disaster-year worksheet precedes the 2026 tax year.
    #[libertas_number(min = 2021, max = 2025)]
    pub disaster_year: u32,
    /// Line 11
    /// Employer-plan income spread over 3 years. Enter 0 if you checked the box on line 11.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line_11: i64,
    /// Line 22
    /// IRA income spread over 3 years. Enter 0 if you checked the box on line 22.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub line_22: i64,
    /// Repayments after you filed that form
    pub repayments: Form8915Repayments,
}

/// Tax year of that form (item A)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum Form8915Year {
    // Actual historical form year and line layout, not a prior-year alias.
    /// 2025
    Filed2025 {
        /// Line 13
        /// Leave lines 13, 14, 24 and 25 at zero when lines 14 and 25 are blank.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        line_13: i64,
        /// Line 14
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        line_14: i64,
        /// Line 24
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        line_24: i64,
        /// Line 25
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        line_25: i64,
    },
    // Actual historical form year, not a second-prior-year alias.
    /// 2024
    Filed2024,
    /// 2023 or earlier
    Earlier,
}

/// Repayments after you filed that form
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct Form8915Repayments {
    /// Repaid to an employer plan by the time you file for 2026
    /// Of the distributions that form reported, within 3 years of each. Only the taxable part.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub plan_repaid: i64,
    /// Repaid to an IRA by the time you file for 2026
    /// Of the distributions that form reported, within 3 years of each. Only the taxable part.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub ira_repaid: i64,
    /// Line 14 excess you already carried back on an amended return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub plan_carried_back: i64,
    /// Line 25 excess you already carried back on an amended return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub ira_carried_back: i64,
}

/// Earlier Form 8915-F of one filer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DisasterCarry {
    /// Whose form
    pub owner: Owner,
    /// Form amounts
    pub form: EarlierForm8915,
}

impl EarlierForm8915 {
    // The client rules of the kind, checked again on save.
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if matches!(self.year, Form8915Year::Filed2024) && self.disaster_year > 2024 {
            return Some(
                crate::validation::ValidationIssue::new("FORM_8915_DISASTER_AFTER_FORM")
                    .at_field("disaster_year"),
            );
        }
        let (plan_back, ira_back) =
            (self.repayments.plan_carried_back, self.repayments.ira_carried_back);
        let (plan_excess, ira_excess) = self.excess_2025();
        if plan_back != 0 && plan_back > plan_excess {
            Some(
                crate::validation::ValidationIssue::new("FORM_8915_CARRIED_BACK_ABOVE_EXCESS")
                    .at_field("plan_carried_back")
                    .in_field("repayments"),
            )
        } else if ira_back != 0 && ira_back > ira_excess {
            Some(
                crate::validation::ValidationIssue::new("FORM_8915_CARRIED_BACK_ABOVE_EXCESS")
                    .at_field("ira_carried_back")
                    .in_field("repayments"),
            )
        } else {
            None
        }
    }
    // Worksheets 3 and 5 line 3a: last year's line 14 (25) over line 13 (24).
    // A carried-back amount is allowed only against a 2025 form's excess.
    pub(crate) fn excess_2025(&self) -> (i64, i64) {
        match &self.year {
            Form8915Year::Filed2025 { line_13, line_14, line_24, line_25 } => {
                (phase::floor(line_14 - line_13, 0), phase::floor(line_25 - line_24, 0))
            }
            _ => (0, 0),
        }
    }
    pub(crate) fn year(&self) -> Option<u32> {
        match self.year {
            Form8915Year::Filed2025 { .. } => Some(2025),
            Form8915Year::Filed2024 => Some(2024),
            Form8915Year::Earlier => None,
        }
    }
    pub(crate) fn repaid(&self) -> (i64, i64, i64, i64) {
        let Form8915Repayments { plan_repaid, ira_repaid, plan_carried_back, ira_carried_back } =
            self.repayments;
        (plan_repaid, ira_repaid, plan_carried_back, ira_carried_back)
    }
    pub(crate) fn valid(&self) -> bool {
        let (a, b, c, d) = self.repaid();
        let lines = match &self.year {
            Form8915Year::Filed2025 { line_13, line_14, line_24, line_25 } => {
                [*line_13, *line_14, *line_24, *line_25]
            }
            _ => [0; 4],
        };
        (2021..=2025).contains(&self.disaster_year)
            && [self.line_11, self.line_22, a, b, c, d]
                .iter()
                .chain(lines.iter())
                .all(|v| (0..=100000000000).contains(v))
            && self.rule().is_none()
    }
}
