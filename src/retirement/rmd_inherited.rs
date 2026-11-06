pub use crate::rmd_sources::{InheritedEligibility, InheritedIraKind, InheritedRmd};
// Original designated beneficiaries; 2024 final RMD regulations effective 2025.
use crate::kernel::ratio::fraction;
use crate::line_emission::line_cents;
use crate::{FederalDraft, FederalResult, Message, MessageArg, Owner};

// IRA required beginning date, including the SECURE/SECURE 2.0 birth cohorts.
fn beginning(birth: u32) -> u32 {
    let year = birth / 10000;
    let due_year = if birth <= 19490630 {
        year + 71 + u32::from(birth / 100 % 100 > 6)
    } else if year <= 1950 {
        year + 73
    } else if year <= 1959 {
        year + 74
    } else {
        year + 76
    };
    due_year * 10000 + 401
}
impl InheritedRmd {
    pub(crate) fn calculate(
        &self,
        d: &FederalDraft,
        r: &mut FederalResult,
        owner: Owner,
        index: usize,
        timely: i64,
    ) -> i64 {
        let person = d.people.as_ref().and_then(|p| {
            if owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
        });
        let death_year = self.died / 10000;
        let supported = self.confirmed
            && death_year <= 2025
            && person.is_some_and(|p| p.died.is_none() && p.birth_date <= self.died);
        crate::require_localized!(
            r,
            supported,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Inherited required distributions",
            "ISSUE_CONFIRM_A_LIVING_ORIGINAL_DESIGNATED_BENEFICIARY_AND_OWNER_DEATH_BE_764F8CE9",
            "Confirm a living original designated beneficiary and owner death before 2026. Successor/trust rules, other spouse elections and unresolved source allocations need the administrator's minimum.",
        );
        if !supported {
            return 0;
        }
        let person = person.unwrap();
        let birth_year = person.birth_date / 10000;
        let after =
            self.kind == InheritedIraKind::Traditional && self.died >= beginning(self.owner_birth);
        // This eligibility test is exact dates, unlike the owner joint-table age test.
        let close_age = person.birth_date <= self.owner_birth + 100000;
        let medical = matches!(
            self.eligibility,
            InheritedEligibility::Disabled | InheritedEligibility::ChronicallyIll
        );
        let child = self.eligibility == InheritedEligibility::OwnerChild
            && self.died < person.birth_date + 210000;
        let old_rules = death_year < 2020;
        let spouse = matches!(
            self.eligibility,
            InheritedEligibility::SoleSpouse
                | InheritedEligibility::SoleSpouseUniformLifetime { .. }
        );
        // An election the custodian has not confirmed is not in effect: the
        // spouse uses the inherited Single Life table.
        let spouse_uniform = matches!(
            self.eligibility,
            InheritedEligibility::SoleSpouseUniformLifetime { confirmed: true }
        );
        let beneficiary_age = 2026 - birth_year;
        crate::require_localized!(
            r,
            !spouse_uniform || (after && death_year >= 2023 && beneficiary_age >= 73),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Inherited required distributions",
            "ISSUE_THE_SUPPORTED_INHERITED_ACCOUNT_UNIFORM_LIFETIME_ELECTION_REQUIRES_3A658F85",
            "The supported inherited-account Uniform Lifetime election requires a sole surviving spouse age 73 or older in 2026 and an original owner who died in 2023 or later, on or after the required beginning date. Reconcile other election histories with the administrator.",
        );
        let eligible = spouse || old_rules || close_age || medical || child;
        crate::require_localized!(
            r,
            !self.deadline_election || !after,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Inherited required distributions",
            "ISSUE_AN_ELIGIBLE_BENEFICIARY_CANNOT_ELECT_THE_BEFORE_BEGINNING_DATE_FIXE_4059242D",
            "An eligible beneficiary cannot elect the before-beginning-date fixed payout rule after the owner reached the required beginning date.",
        );
        let deadline = if old_rules && self.deadline_election {
            // The CARES Act omitted 2020 from an unexpired five-year period.
            Some(death_year + 5 + u32::from(death_year >= 2015))
        } else if !eligible || self.deadline_election {
            Some(death_year + 10)
        } else if !old_rules && child && !close_age && !medical {
            Some(birth_year + 31)
        } else {
            None
        };
        crate::require_localized!(
            r,
            deadline.is_none_or(|v| v > 2026),
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Inherited required distributions",
            "ISSUE_AN_EXPIRED_OR_CURRENT_FINAL_PAYOUT_DEADLINE_NEEDS_ADMINISTRATOR_REV_FF8034CB",
            "An expired or current final payout deadline needs administrator review of the remaining balance and prior shortfalls; it cannot be approximated with an annual life-expectancy fraction.",
        );
        let first = if spouse && !after {
            (death_year + 1).max(beginning(self.owner_birth) / 10000 - 1)
        } else {
            death_year + 1
        };
        let annual = (after || (eligible && !self.deadline_election)) && first <= 2026;
        // Living sole spouses recalculate each year rather than subtracting one
        // from the first beneficiary year's factor.
        let first = first.min(2026);
        let beneficiary_factor = if spouse_uniform && beneficiary_age >= 73 {
            crate::rmd_tables::UNIFORM[(beneficiary_age.min(120) - 72) as usize]
        } else if spouse {
            crate::rmd_tables::SINGLE[(2026 - birth_year).min(120) as usize]
        } else {
            (crate::rmd_tables::SINGLE[((first - birth_year).min(120)) as usize]
                - 10 * i64::from(2026 - first))
            .max(10)
        };
        let owner_factor = (crate::rmd_tables::SINGLE
            [((death_year - self.owner_birth / 10000).min(120)) as usize]
            - 10 * i64::from(2026 - death_year))
        .max(10);
        let factor = if after { beneficiary_factor.max(owner_factor) } else { beneficiary_factor };
        let amount = if annual && factor <= 10 {
            crate::require_localized!(
                r,
                self.closing.is_some(),
                &crate::IssueContext::Section(crate::TaxSection::Retirement),
                "Inherited required distributions",
                "ISSUE_LIFE_EXPECTANCY_IS_EXHAUSTED_SUPPLY_THE_ACTUAL_DECEMBER_31_2026_REM_23509D0F",
                "Life expectancy is exhausted: supply the actual December 31, 2026 remaining account value, including earnings. Last year's balance is not the full-distribution shortfall.",
            );
            timely + self.closing.unwrap_or(0)
        } else if annual {
            fraction(self.balance + self.incoming, 10, factor)
        } else {
            0
        };
        crate::require_localized!(
            r,
            !spouse || !annual || timely >= amount,
            &crate::IssueContext::Section(crate::TaxSection::Retirement),
            "Inherited required distributions",
            "ISSUE_A_SOLE_SPOUSE_S_MISSED_INHERITED_IRA_MINIMUM_CAN_TRIGGER_DEEMED_OWN_C6B1E9C1",
            "A sole spouse's missed inherited IRA minimum can trigger deemed ownership. Reconcile the election and complete personal IRA pool with the administrator before calculating a shortfall; do not impose the nonspouse inherited shortfall mechanically.",
        );
        let deadline_message = deadline.map_or_else(
            || Message::new("EXPLAIN_RMD_LIFE_EXPECTANCY_CONTINUES", alloc::vec![]),
            |year| {
                Message::new(
                    "EXPLAIN_RMD_ACCOUNT_DEADLINE",
                    alloc::vec![MessageArg::Number(i64::from(year)),],
                )
            },
        );
        let minimum_key = if annual && factor <= 10 {
            "EXPLAIN_RMD_EXHAUSTED_LIFE_EXPECTANCY"
        } else if annual && spouse_uniform {
            "EXPLAIN_RMD_SPOUSE_UNIFORM_LIFETIME"
        } else if annual && spouse {
            "EXPLAIN_RMD_SPOUSE_SINGLE_LIFE"
        } else if annual {
            "EXPLAIN_RMD_ANNUAL_LIFE_EXPECTANCY"
        } else if spouse && !self.deadline_election {
            "EXPLAIN_RMD_SPOUSE_MINIMUM_NOT_BEGUN"
        } else {
            "EXPLAIN_RMD_NO_ANNUAL_MINIMUM"
        };
        line_cents(
            r,
            &alloc::format!("rmd_{index}_inherited_minimum"),
            amount,
            &["adjustments.required_distributions", "people"],
            "https://www.irs.gov/irb/2024-33_IRB",
            Message::new(
                "EXPLAIN_RMD_INHERITED_MINIMUM",
                alloc::vec![
                    MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                        minimum_key,
                        alloc::vec![]
                    ))),
                    MessageArg::Nested(alloc::boxed::Box::new(deadline_message)),
                ],
            ),
        );
        amount
    }
}
