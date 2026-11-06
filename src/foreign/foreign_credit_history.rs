use crate::FederalResult;
pub use crate::foreign_credit_source::{
    ForeignCarryback, ForeignCarryoverColumn, ForeignCarryoverOpening, ForeignCarryoverRecords,
    ForeignCarryoverSchedule, ForeignCreditCategory, ForeignCreditHistory, ForeignPriorCredit,
    ForeignPriorLimit,
};
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
impl ForeignCreditHistory {
    pub(crate) fn check(&self, result: &mut FederalResult) {
        if let Self::Retained(_) = self {
            crate::require_localized!(
                result,
                self.valid(),
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign-credit history",
                "ISSUE_PROVIDE_UNIQUE_YEAR_CATEGORY_BALANCES_FROM_COMPLETE_RECONCILED_REGU_96463AA7",
                "Provide unique year/category balances from the regular and AMT records through 2025.",
            );
        }
    }
}

impl ForeignCarryback {
    // The 2025 capacity of one category in the regular tax and the AMT, for
    // the Schedule B line 7 carryback. An unreviewed carryback, a small-credit
    // election year and a category without a retained worksheet have none; the
    // excess then stays a provisional carryforward, with a note when the
    // review is unfinished.
    pub(crate) fn capacity(
        &self,
        kind: ForeignCreditCategory,
        excess: [i64; 2],
        history: &ForeignCreditHistory,
        result: &mut FederalResult,
    ) -> [i64; 2] {
        let name = kind.name();
        if matches!(self, Self::DeductionYear { .. } | Self::SmallCreditElection) {
            let consistent = history.schedule(kind).is_none_or(|schedule| {
                schedule
                    .columns
                    .iter()
                    .all(|c| c.year != 2025 || (c.regular.adjusted() == 0 && c.amt.adjusted() == 0))
            });
            crate::require_localized!(
                result,
                consistent,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign-credit origin year",
                "ISSUE_A_PRIOR_YEAR_FOREIGN_TAX_DEDUCTION_OR_NO_FORM_1116_SMALL_CREDIT_ELECTION_30E20D12",
                "A 2025 foreign-tax deduction or no-Form-1116 small-credit election cannot generate unused 2025 foreign taxes. Reconcile the origin year and prior election; older surviving balances retain their original years.",
            );
            if !consistent {
                return [0; 2];
            }
        }
        let mut capacity = [0; 2];
        match self {
            Self::Unreviewed if excess != [0; 2] => {
                crate::federal_rules::note_once_scoped(
                    result,
                    crate::Message::new(
                        "NOTE_NAME_FOREIGN_CREDIT_EXCESS_REVIEW_THE_PRIOR_YEAR_CARRYBACK_BEF_257AE351",
                        alloc::vec![crate::MessageArg::Text(alloc::string::String::from(name))],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                        .item("foreign-credit-category", kind as usize),
                );
                return [0; 2];
            }
            Self::PriorReturn { categories, .. } | Self::DeductionYear { categories, .. } => {
                let valid = self.valid();
                crate::require_localized!(
                    result,
                    valid,
                    &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                    "Foreign-credit carryback",
                    "ISSUE_CONFIRM_COMPLETE_PRIOR_YEAR_CREDIT_OR_HYPOTHETICAL_DEDUCTION_YEAR_WORKSHE_9E12DBA6",
                    "Enter each category of the 2025 credit or hypothetical deduction-year worksheets once before allocating excess taxes.",
                );
                if !valid {
                    return [0; 2];
                }
                if let Some(prior) = categories.iter().find(|c| c.category == kind) {
                    capacity = [
                        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
                            dollars(prior.regular.limit),
                            dollars(prior.regular.taxes),
                        )
                        .get(),
                        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
                            dollars(prior.amt.limit),
                            dollars(prior.amt.taxes),
                        )
                        .get(),
                    ];
                }
                if let Some(schedule) = history.schedule(kind) {
                    let consistent = schedule.columns.iter().all(|c| {
                        (capacity[0] == 0 || dollars(c.regular.carryover) == 0)
                            && (capacity[1] == 0 || dollars(c.amt.carryover) == 0)
                    });
                    crate::require_localized!(
                        result,
                        consistent,
                        &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                        "Foreign-credit carryback history",
                        "ISSUE_UNUSED_BALANCES_ENTERING_RETURN_YEAR_CONFLICT_WITH_AVAILABLE_PRIOR_YEAR_CAPACITY_2C145BA2",
                        "Unused balances entering 2026 conflict with available 2025 capacity in the same category and tax system. Reconcile older usage before allocating new taxes.",
                    );
                    if !consistent {
                        return [0; 2];
                    }
                }
            }
            _ => (),
        }
        [phase::cap(excess[0], [capacity[0]]), phase::cap(excess[1], [capacity[1]])]
    }
}
