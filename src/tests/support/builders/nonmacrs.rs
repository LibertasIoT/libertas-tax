use crate::*;
use alloc::string::String;

pub(in crate::tests) fn property(
    cost: i64,
    placed: u32,
    system: NonMacrsRecovery,
) -> BusinessAsset {
    NonMacrsAssetPage {
        farm_rental_use: None,
        macrs_source: None,
        id: 0,
        cost,
        credit_reduction: 0,
        acquired: placed,
        placed,
        business_percent: 100,
        recovery: NonMacrsPropertyRecovery::from_source(
            String::new(),
            system,
            EquipmentHistory::New {
                section179: Section179::Ineligible,
                bonus: BonusElection::ElectOut,
                tables: false,
            },
        ),
        history_details: None,
        amt_section179_income_carry: None,
        allowed_amt_carry: alloc::vec![],
        disposal: AssetDisposal::No,
    }
    .into()
}
pub(in crate::tests) fn continuing(source: &mut BusinessAsset, prior: i64) {
    source.history = EquipmentHistory::Continuing {
        initial_section179: 0,
        carryforward: 0,
        prior_depreciation: prior,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
}
pub(in crate::tests) fn route_mut(source: &mut BusinessAsset) -> &mut NonMacrsRecovery {
    let EquipmentClass::NonMacrs { recovery } = &mut source.class else {
        panic!("actual non-MACRS source")
    };
    recovery
}
pub(in crate::tests) fn set_route(page: &mut Asset, system: NonMacrsRecovery) {
    let history = match &page.recovery {
        AssetRecoveryPage::Gds { facts } => match &facts.history {
            AssetHistory::New { first_year } => NonMacrsHistory::New {
                section179: match first_year {
                    FirstYearDeduction::Section179 { elected, .. } => {
                        Section179::Eligible { elected: *elected }
                    }
                    _ => Section179::Ineligible,
                },
                bonus: if *first_year == FirstYearDeduction::Regular {
                    BonusElection::ElectOut
                } else {
                    BonusElection::Apply
                },
            },
            AssetHistory::Continuing {
                prior_depreciation,
                first_year_179,
                first_year_bonus,
                carryforward,
                ..
            } => NonMacrsHistory::Continuing {
                prior_depreciation: *prior_depreciation,
                initial_section179: *first_year_179,
                carryforward: *carryforward,
                original_bonus: *first_year_bonus,
            },
        },
        AssetRecoveryPage::NonMacrs { facts } => {
            NonMacrsHistory::from_native(facts.record(page).history)
        }
    };
    page.recovery = AssetRecoveryPage::NonMacrs {
        facts: NonMacrsPropertyRecovery::from_source(String::new(), system, history.native()),
    };
}

pub(in crate::tests) fn finite_life(
    kind: FiniteLifeKind,
    legal: u32,
    useful: u32,
    salvage: i64,
) -> FiniteLifeRecovery {
    match kind {
        FiniteLifeKind::PatentCopyright => FiniteLifeRecovery::PatentCopyright {
            legal_remaining_months: legal,
            useful_months: useful,
            salvage,
        },
        FiniteLifeKind::OtherFiniteRight => {
            assert_eq!(legal, useful);
            assert_eq!(salvage, 0);
            FiniteLifeRecovery::OtherFiniteRight { remaining_months: legal }
        }
        FiniteLifeKind::MortgageServicing => {
            assert_eq!((legal, useful, salvage), (108, 108, 0));
            FiniteLifeRecovery::MortgageServicing
        }
        FiniteLifeKind::CreatedEligible => {
            assert_eq!((legal, useful, salvage), (180, 180, 0));
            FiniteLifeRecovery::CreatedEligible
        }
        FiniteLifeKind::CreatedRealPropertyBenefits => {
            assert_eq!((legal, useful, salvage), (300, 300, 0));
            FiniteLifeRecovery::CreatedRealPropertyBenefits
        }
    }
}
