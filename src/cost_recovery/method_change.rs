use crate::validation::ValidationIssue;
// Coverage: Form 3115 Part IV line 26 (the section 481(a) adjustment) — Supported (unit 481a,
//   2026-10-09). The application itself (heading, Parts I to III, Part IV lines 25 to 29 as
//   preparation, Schedules A to E, signatures) is Deferred (inventor decision of 2026-10-08);
//   line 26 is entered, not computed (inventor decision of 2026-10-09).
//   Sources: Form 3115 (Rev. December 2022) and its instructions (Part IV lines 25 to 28);
//   Rev. Proc. 2015-13 sections 3.04, 7.03(1), (3)(b), (3)(c) and (4)(a); Rev. Proc. 2025-23
//   (the list of automatic changes, DCN 7, 107, 122, 233, 260 to 263); the 2026 draft
//   Instructions for Schedule C line F; Publication 946 (2025) ch. 1 and 2; 26 U.S.C. 481(a) and
//   446(e); 26 CFR 1.199A-3(b)(1)(iii). The 2026 schedules are drafts; step 4 of the parity
//   plan rechecks them when the finals are posted.
//   Lines: one MethodChange record for each Form 3115 with an amount left in 2026, on the
//   business, farm and rental pages (More about this ...). Year of change 2023 to 2026; consent
//   Automatic { dcn } (line 1a) or WithoutDcn (line 1b or a letter ruling); adjustment Decrease
//   (taken in full), FourYears (the standard period for an increase), OneYear (the line 28
//   de minimis election, under $50,000) or TwoYears (a change requested under examination,
//   section 7.03(3)(b)). Each year's part is the difference of two cumulative amounts
//   fraction(total, k, period), so the parts add to line 26 exactly; the 2026 part and the
//   balance after 2026 are derived, never stored. The rest of a multi-year increase is taken in
//   2026 when the activity stops (section 7.03(4)(a), section 3.04): the page answer
//   rest_in_current_year, and, derived from facts the return already holds, a rental sold in 2026
//   (RentalSale::Sold) and the owner's death in 2026 (unknowns::supported sets the flag on the
//   working copy). Where it lands: Schedule C line 6 (increase, inside receipts()) and Part V
//   line 48 to line 27b (decrease, inside operating costs); Schedule F line 8 or Part III line 43
//   and lines 32a to 32f; Schedule E line 3 and line 19. From there it follows the activity
//   (Schedule 1, Schedule SE, Form 8582, QBI and Form 1116 line 3e). A separately supplied
//   AMT adjustment uses the same consent period and acceleration, with its regular/AMT
//   difference on Form 6251.
//   Left out with WARN_METHOD_CHANGE_LEFT_OUT: unsupported consent terms and a rental with personal use (unknowns::supported removes records from
//   the working copy, so every consumer skips the record and its page position survives). Not computed: the section 481(b) limitation
//   (it can only lower tax; NOTE_METHOD_CHANGE_FORM_3115 says so), an eligible acquisition
//   transaction election, cut-off changes (no adjustment). Rows: <activity>_<id>_method_change_<n>
//   (the 2026 part) and ..._remaining (the balance after 2026).
//   Evidence: method_change.rs; federal_model.rs FederalBusiness::receipts and
//   operating_cost_before_supplies; extended_rules.rs rental_use_for and the Schedule E rows;
//   unknowns.rs method_changes; tests: src/tests/cost_recovery/method_change.rs.
use crate::kernel::lines::line_cents;
use crate::kernel::phase;
use crate::kernel::ratio::fraction;
use crate::{FederalResult, IssueContext, Message, MessageArg};
use alloc::{boxed::Box, format, vec, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Accounting-method change (Form 3115)
/// One for each Form 3115 whose section 481(a) adjustment still has an amount in 2026. The App takes this year's part on this activity's other income or other expenses and shows what remains; it does not prepare Form 3115. A change made on a cut-off basis (Form 3115 line 25 Yes) has no adjustment; do not enter it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MethodChange {
    /// Accepted source identity, assigned only by normal Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Tax year of change
    /// The first year under the new method (Form 3115, "Tax year of change begins"). A four-year spread that began before 2023 has ended.
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2023, max = 2026)]
    pub year: u32,
    /// Change number (Form 3115 Part I line 1)
    pub consent: MethodChangeConsent,
    /// Section 481(a) adjustment (Form 3115 Part IV lines 26 and 28)
    pub adjustment: Adjustment481,
    /// Alternative minimum tax amount of this adjustment
    #[libertas_default(Same)]
    pub amt: MethodChangeAmt,
}

/// Change number (Form 3115 Part I line 1)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MethodChangeConsent {
    /// Automatic change with a designated change number (DCN)
    Automatic {
        /// Designated automatic accounting method change number (line 1a)
        /// For example 7 (depreciation you did not claim or claimed too much), 107 (the same for property you disposed of), 122 (cash to accrual), 233 (a small business changing to cash), or 260 or 261 (small business inventory). With several DCNs and one net adjustment on one form, enter the first.
        #[libertas_number(min = 1)]
        dcn: u32,
    },
    /// No DCN
    /// An automatic change described on line 1b, or a non-automatic change the IRS approved by letter ruling (Part III). A different period set by a letter ruling is not calculated; leave that change out.
    WithoutDcn,
}

/// Section 481(a) adjustment (Form 3115 Part IV lines 26 and 28)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Adjustment481 {
    /// Decrease in income (a negative adjustment)
    /// Deducted in full in the year of change, for example depreciation you never claimed (DCN 7 or 107).
    Decrease {
        /// Amount of the decrease (line 26, without the minus sign)
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        amount: i64,
    },
    /// Increase in income spread over four years
    /// The standard period for an increase: a quarter in the year of change and in each of the next three years.
    FourYears {
        /// Amount of the increase (line 26)
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        amount: i64,
        /// The rest of this increase is taken in 2026
        /// Yes when this activity stopped in 2026: you closed it, sold or transferred substantially all of its assets, incorporated it or contributed it to a partnership. Also Yes when a change you made for 2026 requires taking the rest of this earlier adjustment now (Form 3115 line 27). A rental sold in 2026 on this page and an owner who died in 2026 always take the rest. Your closing or sale records and your 2026 Form 3115 show this.
        rest_in_current_year: bool,
    },
    /// Increase under $50,000 taken in full in the year of change
    /// You checked the $50,000 de minimis election on Form 3115 line 28.
    OneYear {
        /// Amount of the increase (line 26)
        #[libertas_money("USD", 2)]
        // Rev. Proc. 2015-13 section 7.03(3)(c): the one-year de minimis election requires an adjustment below $50,000.
        #[libertas_number(min = 1, max = 4999999)]
        amount: i64,
    },
    /// Increase spread over two years because your return was under IRS examination
    /// The period for a change requested during an examination (Rev. Proc. 2015-13 section 7.03(3)(b)): half in the year of change and half in the next year.
    TwoYears {
        /// Amount of the increase (line 26)
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 1)]
        amount: i64,
        /// The rest of this increase is taken in 2026
        /// Yes when this activity stopped in 2026: you closed it, sold or transferred substantially all of its assets, incorporated it or contributed it to a partnership. Also Yes when a change you made for 2026 requires taking the rest of this earlier adjustment now (Form 3115 line 27). A rental sold in 2026 on this page and an owner who died in 2026 always take the rest. Your closing or sale records and your 2026 Form 3115 show this.
        rest_in_current_year: bool,
    },
}

// The one owner of the phrase every "no accounting-method change" scope
// confirmation uses to say that a change entered under Accounting-method
// changes is no longer excluded (improvement row 139). Doc comments are
// literal text for schema extraction, so the confirmations spell it out and
// src/tests/cost_recovery/method_change.rs checks each of them against this constant.
#[cfg(test)]
pub(crate) const ENTERED_UNDER_LIST: &str = "entered under Accounting-method changes";

impl MethodChange {
    // Years in the adjustment period and the signed line 26 amount, in cents.
    fn period(&self) -> (i64, i64) {
        match self.adjustment {
            Adjustment481::Decrease { amount } => (1, -amount),
            Adjustment481::OneYear { amount } => (1, amount),
            Adjustment481::TwoYears { amount, .. } => (2, amount),
            Adjustment481::FourYears { amount, .. } => (4, amount),
        }
    }
    // Cumulative amount taken through the end of `year`: ratable by whole years
    // (Rev. Proc. 2015-13 section 7.03(1)); differences of cumulative amounts
    // keep the yearly parts adding to line 26 exactly.
    fn through(&self, year: i64) -> i64 {
        let (years, total) = self.period();
        fraction(total, (year - i64::from(self.year) + 1).clamp(0, years), years)
    }
    fn rest_in_current_year(&self) -> bool {
        matches!(
            self.adjustment,
            Adjustment481::FourYears { rest_in_current_year: true, .. }
                | Adjustment481::TwoYears { rest_in_current_year: true, .. }
        )
    }
    /// The activity stopped in 2026 (section 7.03(4)(a)): the rest of a
    /// multi-year increase is taken now. A decrease or one-year increase has
    /// no rest.
    pub(crate) fn accelerate(&mut self) {
        if let Adjustment481::FourYears { rest_in_current_year, .. }
        | Adjustment481::TwoYears { rest_in_current_year, .. } = &mut self.adjustment
        {
            *rest_in_current_year = true;
        }
    }
    /// The signed 2026 part in cents: positive is other income, negative other expenses.
    pub(crate) fn amount_current_year(&self) -> i64 {
        let before = self.through(2025);
        if self.rest_in_current_year() {
            self.period().1 - before
        } else {
            self.through(2026) - before
        }
    }
    /// What remains for 2027 and later, in cents.
    pub(crate) fn remaining_after_current_year(&self) -> i64 {
        self.period().1 - self.through(2025) - self.amount_current_year()
    }
    // Whether the consent terms and the amount are inside what the App
    // calculates, repeated on save and on stored data.
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if !(0..=100_000_000_000).contains(&self.id) {
            return Some(ValidationIssue::new("FARM_METHOD_SOURCE_ID").at_field("id"));
        }
        let (years, total) = self.period();
        if matches!(self.amt, MethodChangeAmt::Different { amount } if !(0..=100_000_000_000).contains(&amount))
        {
            return Some(
                ValidationIssue::new("METHOD_CHANGE_OUT_OF_LIMITS")
                    .at_field("amount")
                    .in_field("amt"),
            );
        }
        if self.year > 2026 {
            return Some(ValidationIssue::new("METHOD_CHANGE_OUT_OF_LIMITS").at_field("year"));
        }
        if !(1..=100_000_000_000).contains(&total.abs()) {
            return Some(
                ValidationIssue::new("METHOD_CHANGE_OUT_OF_LIMITS")
                    .at_field("amount")
                    .in_field("adjustment"),
            );
        }
        if matches!(self.consent, MethodChangeConsent::Automatic { dcn } if !(1..=999).contains(&dcn))
        {
            return Some(
                ValidationIssue::new("METHOD_CHANGE_OUT_OF_LIMITS")
                    .at_field("dcn")
                    .in_field("consent"),
            );
        }
        if matches!(self.adjustment, Adjustment481::OneYear { amount } if amount >= 5_000_000) {
            return Some(
                ValidationIssue::new("METHOD_CHANGE_DE_MINIMIS_LIMIT")
                    .at_field("amount")
                    .in_field("adjustment"),
            );
        }
        (i64::from(self.year) + years <= 2026)
            .then(|| ValidationIssue::new("METHOD_CHANGE_NOTHING_IN_CURRENT_YEAR").at_field("year"))
    }
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    // "DCN 7" or "no DCN".
    fn consent_message(&self) -> Message {
        match self.consent {
            MethodChangeConsent::Automatic { dcn } => {
                Message::new("METHOD_CHANGE_DCN", vec![MessageArg::Number(i64::from(dcn))])
            }
            MethodChangeConsent::WithoutDcn => Message::new("METHOD_CHANGE_NO_DCN", vec![]),
        }
    }
}

/// The page rules repeated on save for one activity's list: each record, then the count.
pub(crate) fn list_issue(changes: &[MethodChange]) -> Option<ValidationIssue> {
    changes
        .iter()
        .enumerate()
        .find_map(|(i, change)| change.issue().map(|issue| issue.in_list("method_changes", i)))
        .or_else(|| {
            (changes.len() > 10).then(|| {
                ValidationIssue::new("MESSAGE_ENTER_AT_MOST_10_METHOD_CHANGES")
                    .at_field("method_changes")
            })
        })
}
pub(crate) fn list_rule(changes: &[MethodChange]) -> Option<&'static str> {
    list_issue(changes).map(|issue| issue.key)
}

/// Whether stored changes are inside the supported limits.
pub(crate) fn valid(changes: &[MethodChange]) -> bool {
    list_rule(changes).is_none()
}

// The records the calculation uses with their positions on the page. A record
// left out (unknowns::supported removes records from the working copy, so a
// note's record position stays the page's) takes no part.
fn used(changes: &[MethodChange]) -> impl Iterator<Item = (usize, &MethodChange)> {
    changes.iter().enumerate()
}

/// Sum of the 2026 increases of an activity's changes, in cents.
pub(crate) fn increase(changes: &[MethodChange]) -> i64 {
    used(changes).map(|(_, c)| phase::floor(c.amount_current_year(), 0)).sum()
}

/// Sum of the 2026 decreases of an activity's changes, in cents, as a positive amount.
pub(crate) fn decrease(changes: &[MethodChange]) -> i64 {
    used(changes).map(|(_, c)| phase::floor(-c.amount_current_year(), 0)).sum()
}

/// One activity's rows and notes: each record's 2026 part, the balance left
/// for 2027 and later, and the review note that the adjustment assumes a
/// filed Form 3115. `row` is the activity's return-line prefix (`business_3`,
/// `farm_3`, `rental_3`); `label` names the activity in the note.
pub(crate) fn report(
    changes: &[MethodChange],
    r: &mut FederalResult,
    row: &str,
    input: &str,
    label: &MessageArg,
    context: &IssueContext,
) {
    for (index, change) in used(changes) {
        let part = change.amount_current_year();
        let arguments = vec![
            MessageArg::Number(i64::from(change.year)),
            MessageArg::Nested(Box::new(change.consent_message())),
        ];
        line_cents(
            r,
            &format!("{row}_method_change_{index}"),
            part,
            &[input],
            "https://www.irs.gov/instructions/i3115",
            Message::new(
                if change.period().1 < 0 {
                    "EXPLAIN_METHOD_CHANGE_DECREASE"
                } else {
                    "EXPLAIN_METHOD_CHANGE_INCREASE"
                },
                arguments.clone(),
            ),
        );
        let remaining = change.remaining_after_current_year();
        if remaining != 0 {
            line_cents(
                r,
                &format!("{row}_method_change_{index}_remaining"),
                remaining,
                &[input],
                "https://www.irs.gov/instructions/i3115",
                Message::new("EXPLAIN_METHOD_CHANGE_REMAINING", Vec::new()),
            );
        }
        crate::federal_rules::note_once_scoped(
            r,
            Message::new(
                "NOTE_METHOD_CHANGE_FORM_3115",
                vec![label.clone(), arguments[0].clone(), arguments[1].clone()],
            ),
            context.item("method_changes", index),
        );
    }
}

// Actual accepted source children, sharing the ordinary return ID allocator.
// The accounting adjustment owner remains unchanged.
pub(crate) fn records(income: &crate::FederalIncome) -> Option<&Vec<MethodChange>> {
    match income {
        crate::FederalIncome::Business { data } => Some(&data.method_changes),
        crate::FederalIncome::Rental { data } => Some(&data.method_changes),
        _ => None,
    }
}
pub(crate) fn records_mut(income: &mut crate::FederalIncome) -> Option<&mut Vec<MethodChange>> {
    match income {
        crate::FederalIncome::Business { data } => Some(&mut data.method_changes),
        crate::FederalIncome::Rental { data } => Some(&mut data.method_changes),
        _ => None,
    }
}

/// Validate a farm's link against this activity's actual accepted adjustment
/// source. A sibling farm or a zero-ID draft row cannot establish consent.
pub(crate) fn farm_link_issue(
    income: &crate::FederalIncome,
) -> Option<crate::validation::ValidationIssue> {
    let (farm, changes) = match income {
        crate::FederalIncome::Business { data } => match &data.activity {
            crate::BusinessActivity::Farming { data: farm } => (farm, Some(&data.method_changes)),
            _ => return None,
        },
        crate::FederalIncome::FarmRental { data } => (&data.farm, None),
        _ => return None,
    };
    let link = farm.specials.method_status.change.as_ref()?;
    let Some(changes) = changes else {
        return Some(
            crate::validation::ValidationIssue::new("FARM_METHOD_CHANGE_LINK_UNAVAILABLE")
                .at_field("change")
                .in_field("change")
                .in_field("method_status")
                .in_field("specials")
                .in_field("more"),
        );
    };
    if !changes.iter().any(|change| {
        change.id == link.change
            && change.id > 0
            && change.year >= farm.specials.method_status.adopted
    }) {
        return Some(
            crate::validation::ValidationIssue::new("FARM_METHOD_SOURCE_ID")
                .at_field("change")
                .in_field("change")
                .in_field("method_status")
                .in_field("specials")
                .in_field("more"),
        );
    }
    None
}

/// Alternative minimum tax amount of a section 481(a) adjustment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum MethodChangeAmt {
    /// The same amount as for regular tax
    /// Your Form 3115 states one adjustment, or the change is not a depreciation change.
    Same,
    /// A different amount for the alternative minimum tax
    /// A depreciation change (DCN 7 or 107) for equipment with separate AMT depreciation records; from the AMT statement of Form 3115 Part IV.
    Different {
        /// AMT section 481(a) adjustment (line 26 for the AMT, without the minus sign)
        /// Taken over the same period as the regular-tax amount.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
}

/// Signed replacement AMT part minus regular part, source cents.
pub(crate) fn amt_difference(changes: &[MethodChange]) -> i64 {
    changes
        .iter()
        .map(|c| {
            let MethodChangeAmt::Different { amount } = c.amt else {
                return 0;
            };
            let mut amt = c.clone();
            match &mut amt.adjustment {
                Adjustment481::Decrease { amount: a }
                | Adjustment481::OneYear { amount: a }
                | Adjustment481::TwoYears { amount: a, .. }
                | Adjustment481::FourYears { amount: a, .. } => *a = amount,
            };
            amt.amount_current_year() - c.amount_current_year()
        })
        .sum()
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(MethodChange {
    amounts: [adjustment, amt],
    gains: [],
    facts: [id, year, consent],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(Adjustment481 {
    records: [
        Decrease { amounts: [amount], facts: [] },
        FourYears { amounts: [amount], facts: [rest_in_current_year] },
        OneYear { amounts: [amount], facts: [] },
        TwoYears { amounts: [amount], facts: [rest_in_current_year] },
    ],
    units: [],
    tuples: [],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(MethodChangeAmt {
    records: [Different { amounts: [amount], facts: [] },],
    units: [Same],
    tuples: [],
});
