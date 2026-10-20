// Owner-account RMD source calculations; Pub. 590-B chapter 1 and Appendix B.
use crate::federal_rules::valid_birth;
use crate::kernel::lines::line_cents;
use crate::kernel::ratio::fraction;
use crate::{FederalDraft, FederalResult, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

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

/// Source of the required minimum
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RmdMinimum {
    /// Calculate an original beneficiary’s inherited IRA minimum
    Inherited { data: crate::InheritedRmd },
    /// Administrator's required amount
    Administrator {
        /// Minimum required by deadlines in 2026
        /// Retain the administrator's account-balance and beneficiary calculation. Include first payments deferred from 2025 to April 2026; exclude payments lawfully deferred until 2027.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Calculate from own traditional IRA account balances
    OwnIras {
        /// Distribution year and first-year deadline choice
        period: RmdPeriod,
        /// Separately valued accounts with their beneficiary designations
        /// Compute each IRA separately before adding the requirements. A withdrawal from one own traditional IRA may satisfy this group's total; never combine inherited or Roth IRAs here. Own traditional, SEP and SIMPLE IRAs, each account once: no inherited account, first-year spouse ownership election after death, annuity purchase or valuation election or outstanding transfer not entered. Working past age 73 does not defer an IRA's requirement. The December 31 custodian statements (Form 5498 box 5) show the balances.
        #[libertas_size(min = 1)]
        accounts: Vec<RmdIra>,
    },
}

/// IRA distribution year
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RmdPeriod {
    /// 2026 requirement
    /// Due by December 31, 2026. In the year you turn 73 the first requirement is due April 1, 2027 instead: the calculated amount is shown, no shortfall tax is figured on this return, and the next requirement is also due in 2027, by December 31. Waiting can concentrate taxable income in 2027.
    Current,
    /// First 2025 requirement deferred to April 1, 2026
    /// Only for an owner who turned 73 in 2025. Use December 31, 2024 balances and 2025 beneficiary facts. Include all payments satisfying this first requirement by April 1, including any made in 2025, in the timely-payment field. Report the separate 2026 requirement too; never count a payment twice.
    FirstPriorYear,
}

/// IRA balance and beneficiary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RmdIra {
    /// December 31 balance before the distribution year
    /// Use December 31, 2025 for 2026; December 31, 2024 for the deferred first 2025 requirement. Do not add later contributions designated for that earlier year or subtract later withdrawals.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub balance: i64,
    /// Outstanding incoming rollover absent from all year-end accounts
    /// Use its value on receipt and add it only to the receiving IRA. Enter zero when none. Do not enter a rollover already included in the year-end balance or another account here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub rollover: i64,
    /// Spouse qualifying as sole beneficiary for the distribution year
    /// Leave absent if unmarried or the spouse does not qualify as sole beneficiary. Age alone does not qualify another beneficiary for the joint table.
    /// [NullText]
    /// No spouse qualifies as this account's sole beneficiary
    /// [SetText]
    /// The spouse qualifying as sole beneficiary is
    pub spouse: Option<RmdSpouse>,
}

/// Spouse beneficiary facts for this distribution year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RmdSpouse {
    /// Spouse's date of birth
    /// Use that year's beneficiary, which can differ from the spouse on the current return following divorce or death.
    #[libertas_date_only]
    pub birth: u32,
    /// Marriage and sole-beneficiary requirements confirmed
    /// Check the account’s beneficiary designation and marriage records for that distribution year.
    /// Marital status is determined January 1. Confirm sole-beneficiary status for the required year; a spouse's death alone does not disqualify that year, but divorce followed by a beneficiary change during the year does. For a deferred first withdrawal use the distribution year's designation, not the later payment year's. No uses the Uniform Lifetime table (a larger minimum).
    pub confirmed: bool,
}
impl RmdMinimum {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Administrator { amount } => (0..=100000000000).contains(amount),
            Self::Inherited { data } => data.valid(),
            Self::OwnIras { accounts, .. } => {
                !accounts.is_empty()
                    && accounts.len() <= 100
                    && accounts.iter().all(|a| {
                        [a.balance, a.rollover].iter().all(|v| (0..=100000000000).contains(v))
                            && a.spouse.as_ref().is_none_or(|s| valid_birth(s.birth))
                    })
            }
        }
    }
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
