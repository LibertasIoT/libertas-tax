pub use crate::rmd_sources::{RmdIra, RmdMinimum, RmdPeriod, RmdSpouse};
// Owner-account RMD source calculations; Pub. 590-B chapter 1 and Appendix B.
use crate::kernel::ratio::fraction;
use crate::line_emission::line_cents;
use crate::{FederalDraft, FederalResult, Owner};

// Treasury Decision 9930, 85 FR 72472 (Nov. 12, 2020), section 1.401(a)(9)-9(d).
// https://www.govinfo.gov/content/pkg/FR-2020-11-12/html/2020-24723.htm
// Source HTML SHA-256: b3156d4c2d89d3be8cd24608824d3bb13d6eacd25f83e995a01c016d1b0a4282
// Joint-life factors in tenths: owner ages 73–120, sole-beneficiary spouse ages 0–19.
// Publication 590-B prints only spouse ages 20–120; these cells complete that table.
#[rustfmt::skip]
const JOINT_YOUNG_SPOUSE: [[i64; 20]; 48] = [
    [846, 838, 828, 818, 808, 799, 789, 779, 769, 759, 749, 739, 730, 720, 710, 700, 690, 681, 671, 661],
    [846, 838, 828, 818, 808, 799, 789, 779, 769, 759, 749, 739, 730, 720, 710, 700, 690, 680, 671, 661],
    [846, 838, 828, 818, 808, 799, 789, 779, 769, 759, 749, 739, 729, 720, 710, 700, 690, 680, 671, 661],
    [846, 838, 828, 818, 808, 799, 789, 779, 769, 759, 749, 739, 729, 720, 710, 700, 690, 680, 671, 661],
    [846, 838, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 720, 710, 700, 690, 680, 670, 661],
    [846, 838, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 719, 710, 700, 690, 680, 670, 661],
    [846, 838, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 719, 710, 700, 690, 680, 670, 661],
    [846, 838, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 719, 710, 700, 690, 680, 670, 661],
    [846, 838, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 719, 710, 700, 690, 680, 670, 660],
    [846, 838, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 719, 709, 700, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 719, 709, 700, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 789, 779, 769, 759, 749, 739, 729, 719, 709, 700, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 700, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 700, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 700, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
    [846, 837, 828, 818, 808, 798, 788, 779, 769, 759, 749, 739, 729, 719, 709, 699, 690, 680, 670, 660],
];

fn joint_factor(owner_age: u32, spouse_age: u32) -> i64 {
    let owner = (owner_age.min(120) - 73) as usize;
    if spouse_age < 20 {
        JOINT_YOUNG_SPOUSE[owner][spouse_age as usize]
    } else {
        crate::rmd_tables::JOINT[owner][(spouse_age.min(120) - 20) as usize]
    }
}

impl RmdMinimum {
    pub(crate) fn calculate(
        &self,
        d: &FederalDraft,
        r: &mut FederalResult,
        owner: Owner,
        index: usize,
        timely: i64,
    ) -> i64 {
        if let Self::Inherited { data } = self {
            return data.calculate(d, r, owner, index, timely);
        }
        let Self::OwnIras { period, accounts } = self else {
            let Self::Administrator { amount } = self else { unreachable!() };
            return *amount;
        };
        let person = d.people.as_ref().and_then(|p| {
            if owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
        });
        crate::require_localized!(
            r,
            person.is_some(),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Required distributions",
            "ISSUE_CONFIRM_COMPLETE_OWN_IRA_BALANCES_AND_SOURCE_FACTS_USE_THE_INHERITE_5B1D22CE",
            "Confirm complete own-IRA balances and source facts; use the inherited branch for supported beneficiary accounts or an administrator's minimum for annuity and unresolved special-relief cases.",
        );
        let Some(person) = person else {
            return 0;
        };
        let birth = person.birth_date / 10000;
        let year: u32 = if *period == RmdPeriod::FirstPriorYear { 2025 } else { 2026 };
        let age = year.saturating_sub(birth);
        crate::require_localized!(
            r,
            *period == RmdPeriod::Current || age == 73,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Required distributions",
            "ISSUE_FIRST_YEAR_DEFERRAL_IS_AVAILABLE_ONLY_FOR_THE_YEAR_THIS_OWNER_TURNS_FC1B39D7",
            "First-year deferral is available only for the year this owner turns 73; later IRA requirements cannot be deferred for continued employment.",
        );
        let died_before_beginning =
            person.died.as_ref().is_some_and(|v| v.date < (birth + 74) * 10000 + 401)
                && birth >= 1952;
        crate::require_localized!(
            r,
            person.died.is_none()
                || died_before_beginning
                || (*period == RmdPeriod::FirstPriorYear
                    && person.died.as_ref().is_some_and(|v| v.date > 20260401)),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Required distributions",
            "ISSUE_A_REQUIREMENT_WHOSE_DEADLINE_FOLLOWS_THE_OWNER_S_DEATH_NEEDS_ADMINI_80CF0FDD",
            "A requirement whose deadline follows the owner's death needs administrator allocation between the decedent and beneficiaries; do not charge the deceased filer's return for another payee's shortfall.",
        );
        let mut required = 0;
        for (account, data) in accounts.iter().enumerate() {
            let mut denominator = if age >= 73 {
                crate::rmd_tables::UNIFORM[(age.min(120) - 72) as usize]
            } else {
                0
            };
            if let Some(spouse) = &data.spouse {
                crate::require_localized!(
                    r,
                    spouse.birth / 10000 <= year,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                    "Required distributions",
                    "ISSUE_CONFIRM_THE_DISTRIBUTION_YEAR_S_MARRIAGE_AND_SOLE_BENEFICIARY_DESIG_1F368E99",
                    "Confirm the distribution year's marriage and sole-beneficiary designation, including changes after divorce or death.",
                );
                let spouse_age = year.saturating_sub(spouse.birth / 10000);
                if spouse.confirmed && age > spouse_age + 10 && age >= 73 {
                    denominator = joint_factor(age, spouse_age);
                }
            }
            let amount = if denominator == 0 || died_before_beginning {
                0
            } else {
                fraction(data.balance + data.rollover, 10, denominator)
            };
            required += amount;
            line_cents(
                r,
                &alloc::format!("rmd_{index}_account_{account}_minimum"),
                amount,
                &["adjustments.required_distributions", "people"],
                "https://www.irs.gov/publications/p590b",
                msg!("EXPLAIN_RMD_N_ACCOUNT_N_MINIMUM"),
            );
        }
        // The first requirement, for the year the owner turns 73, is due April 1 of the next
        // year (i5329 Part IX; section 401(a)(9)(C)), so a shortfall at December 31, 2026 is 2027
        // tax: the deadline is the statute's, not an election.
        if *period == RmdPeriod::Current && age == 73 {
            line_cents(
                r,
                &alloc::format!("rmd_{index}_deferred_to_following_year"),
                required,
                &["adjustments.required_distributions"],
                "https://www.irs.gov/irb/2024-33_IRB",
                msg!("EXPLAIN_RMD_N_DEFERRED_TO_N"),
            );
            0
        } else {
            required
        }
    }
}

#[cfg(test)]
mod tests {
    use super::joint_factor;

    #[test]
    fn regulatory_joint_table_includes_spouses_younger_than_twenty() {
        // Treasury Decision 9930, Table 3 to section 1.401(a)(9)-9(d).
        assert_eq!(joint_factor(73, 0), 846);
        assert_eq!(joint_factor(73, 18), 671);
        assert_eq!(joint_factor(73, 19), 661);
        assert_eq!(joint_factor(73, 20), 651);
        assert_eq!(joint_factor(90, 19), 660);
        assert_eq!(joint_factor(120, 0), 846);
        assert_eq!(joint_factor(120, 19), 660);
        assert_eq!(joint_factor(120, 20), 650);
    }

    #[test]
    fn young_spouse_rows_meet_the_published_adult_table_without_a_gap() {
        for owner_age in 73..=120 {
            let factors = (0..=20)
                .map(|spouse_age| joint_factor(owner_age, spouse_age))
                .collect::<alloc::vec::Vec<_>>();
            assert!(factors.windows(2).all(|pair| pair[0] >= pair[1]));
            assert_eq!(factors[20], crate::rmd_tables::JOINT[(owner_age - 73) as usize][0]);
        }
    }
}
