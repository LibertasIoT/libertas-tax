use super::*;
use crate::tests::support::assert::injured_income::home_payroll_actual_clients::checked;
use crate::tests::support::builders::home_employee::{Source, fixture};
// Actual HomeSale/Form8919 source editor plus shared-amount Workflow lifecycle.
// Inputs and independent native figures reuse the verified40-source control family.

#[test]
fn home_payroll_actual_fixtures_keep_independent_source_and_money_controls() {
    for source in [
        Source::ExcludedHome,
        Source::TaxableHome,
        Source::CoveredEmployee,
        Source::OppositeEmployee,
        Source::UncoveredEmployee,
    ] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    for answered in [false, true] {
                        checked(
                            &fixture(source, owner, injured, reverse, answered),
                            source,
                            owner,
                            injured,
                            answered,
                        );
                    }
                }
            }
        }
    }
}
