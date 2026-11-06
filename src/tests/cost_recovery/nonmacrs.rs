// Non-MACRS source schedules and annual recovery boundaries.
use super::*;
use crate::tests::cross::interview::{asset, with_first_year};
use crate::tests::support::builders::nonmacrs::{
    continuing, finite_life, property, route_mut, set_route,
};
#[test]
fn unit17_r6_published_patent_uses_its_actual_seventeen_year_life() {
    // Pub946 (2025) printed p9: $5100/17 years, April first use:
    // nine months gives $225; next full year gives $300. No stated source
    // calendar year is reinterpreted: the unchanged worksheet is parameterized.
    let facts = FiniteLifeRecovery::PatentCopyright {
        legal_remaining_months: 204,
        useful_months: 204,
        salvage: 0,
    };
    let first = property(510_000, 20260401, NonMacrsRecovery::FiniteLife { facts });
    assert!(first.valid());
    assert!(first.qualifies());
    assert_eq!(first.deduction(false), 22_500);
    let mut next = first.clone();
    next.placed = 20250401;
    next.acquired = 20250401;
    continuing(&mut next, 22_500);
    let NonMacrsRecovery::FiniteLife { facts } = route_mut(&mut next) else { unreachable!() };
    let FiniteLifeRecovery::PatentCopyright { legal_remaining_months, .. } = facts else {
        unreachable!()
    };
    *legal_remaining_months = 195;
    assert_eq!(next.deduction(false), 30_000);
}

#[test]
fn unit17_r6_standalone_software_uses_36_months_instead_of_macrs_class() {
    let facts = SoftwareRecovery {
        classification: SoftwareKind::Standalone,
        exempt_lease: None,
        bonus_eligible: false,
    };
    let source = property(3_600_000, 20260701, NonMacrsRecovery::Software { facts });
    assert!(source.valid());
    assert!(source.qualifies());
    // Six source months /36, with no179 or bonus election.
    assert_eq!(source.bonus_amount(), 0);
    assert_eq!(source.deduction(false), 600_000);
    assert_eq!(crate::depreciation::form_4562_line(&source, "macrs"), Some("16"));
}

#[test]
fn unit17_r6_units_use_remaining_adjusted_basis_less_salvage() {
    let facts = UnitProduction {
        qbi_class: GdsEquipmentClass::Seven,
        elected_originally: true,
        current_units: 200,
        remaining_units: 1000,
        remaining_basis: 800_000,
        salvage: 100_000,
    };
    let mut source = property(1_000_000, 20250101, NonMacrsRecovery::Units { facts });
    continuing(&mut source, 200_000);
    assert!(source.valid());
    assert!(source.qualifies());
    // ($8000 remaining −$1000 salvage) ×200/1000 = $1400.
    assert_eq!(source.deduction(false), 140_000);
    assert_eq!(crate::depreciation::form_4562_line(&source, "macrs"), Some("15"));
}

#[test]
fn unit17_r6_forecast_uses_original_cost_and_tenth_year_remaining_basis() {
    let facts = IncomeForecastRecovery {
        kind: ForecastPropertyKind::FilmVideo,
        current_income: 2_000_000,
        forecast_total_income: 10_000_000,
        remaining_basis: 6_000_000,
        participations: ForecastParticipations::None,
        lookback: ForecastLookback::SmallCostException,
    };
    let mut source = property(10_000_000, 20250101, NonMacrsRecovery::IncomeForecast { facts });
    continuing(&mut source, 4_000_000);
    assert!(source.valid());
    assert!(source.qualifies());
    // $100000 original cost ×20000/100000 = $20000, not $12000
    // from the remaining $60000. Cost is within the statutory lookback exception.
    assert_eq!(source.deduction(false), 2_000_000);
    source.placed = 20160101;
    source.acquired = 20160101;
    let NonMacrsRecovery::IncomeForecast { facts } = route_mut(&mut source) else { unreachable!() };
    facts.current_income = 0;
    assert!(source.qualifies());
    assert_eq!(source.deduction(false), 6_000_000);
}

#[test]
fn unit17_r6_pre1981_straight_line_keeps_the_original_schedule() {
    let facts = HistoricalDepreciation {
        qbi_class: GdsEquipmentClass::Seven,
        property: HistoricalPropertyKind::PersonalOwned,
        kind: HistoricalRecoveryKind::Pre1981StraightLine { remaining_months: 48, salvage: 0 },
        remaining_basis: 800_000,
        original_convention: AssetConvention::ActualMonths,
    };
    let mut source = property(10_000_000, 19800101, NonMacrsRecovery::Historical { facts });
    continuing(&mut source, 9_200_000);
    source.method = EquipmentMethod::StraightLine;
    assert!(source.valid());
    // Forty-six original years ×$2000 leaves $8000/48months ×12.
    assert!(source.qualifies());
    assert_eq!(source.deduction(false), 200_000);
}

#[test]
fn unit17_r6_historical_remaining_life_and_original_rate_are_distinct_sources() {
    let facts = HistoricalDepreciation {
        qbi_class: GdsEquipmentClass::Seven,
        property: HistoricalPropertyKind::PersonalOwned,
        kind: HistoricalRecoveryKind::Pre1981StraightLine { remaining_months: 36, salvage: 0 },
        remaining_basis: 800_000,
        original_convention: AssetConvention::ActualMonths,
    };
    let mut source = property(10_000_000, 19800101, NonMacrsRecovery::Historical { facts });
    continuing(&mut source, 9_200_000);
    // Prospective actual remaining-life estimate: $8000 ×12/36.
    assert_eq!(source.deduction(false), 266_667);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.kind = HistoricalRecoveryKind::Pre1981Declining {
        multiplier: 2.0,
        original_useful_months: 600,
        salvage: 0,
    };
    // Original50-year life still owns the4% rate; no3-year rate reset.
    assert_eq!(source.deduction(false), 32_000);
    source.disposal =
        AssetDisposal::Disposed { date: 20261001, how: crate::DisposalKind::Scrapped };
    assert_eq!(source.deduction(false), 24_000); // Nine full source months.
    assert_eq!(source.deduction_for(false, true), 24_000);
}

#[test]
fn unit17_r6_software_retains_exact_lease_floor_and_final_service_months() {
    let facts = SoftwareRecovery {
        classification: SoftwareKind::Standalone,
        exempt_lease: Some(SoftwareTaxExemptLease { entered: 20260401, months: 121 }),
        bonus_eligible: false,
    };
    let source = property(3_630_000, 20260701, NonMacrsRecovery::Software { facts });
    // 121 ×125% =151.25 months, $36300/151.25 ×6 =$1440.
    assert_eq!(source.deduction(false), 144_000);
    assert_eq!(source.deduction_for(false, true), 144_000);
    assert_eq!(source.nonmacrs_recovery(0).unwrap().quarter_months, 605);
    let mut last = property(
        3_600_000,
        20230701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    continuing(&mut last, 3_000_000);
    assert_eq!(last.deduction(false), 600_000);
    assert_eq!(last.amt_remaining_basis(false), 0);
    assert!(!last.qbi_property());
}

#[test]
fn unit17_r6_software_reduces_the_sole_regular_basis_once_for_179_and_bonus() {
    let mut page = with_first_year(
        asset(3_600_000, 20260701, AssetClass::Seven),
        FirstYearDeduction::Section179 {
            description: "Synthetic public software".into(),
            elected: 900_000,
        },
    );
    set_route(
        &mut page,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::PublicNonexclusive,
                exempt_lease: None,
                bonus_eligible: true,
            },
        },
    );
    let mut source = crate::interview::vehicle_and_asset_records(&[], &[page]).1.remove(0);
    let EquipmentHistory::New { bonus, .. } = &mut source.history else { unreachable!() };
    *bonus = BonusElection::ElectOut;
    assert_eq!(source.elected179(), 900_000);
    assert_eq!(source.deduction(false), 450_000); // $27000 ×6/36.
    let EquipmentHistory::New { bonus, .. } = &mut source.history else { unreachable!() };
    *bonus = BonusElection::Apply;
    assert_eq!(source.bonus_amount(), 2_700_000);
    assert_eq!(source.macrs_amount(false), 0);
    assert_eq!(source.deduction(false), 2_700_000);
}

#[test]
fn unit17_r6_statutory_intangible_periods_do_not_use_the_placeholder_asset_class() {
    for (kind, period, expected) in [
        (FiniteLifeKind::MortgageServicing, 108, 450_000),
        (FiniteLifeKind::CreatedEligible, 180, 270_000),
        (FiniteLifeKind::CreatedRealPropertyBenefits, 300, 162_000),
    ] {
        let source = property(
            5_400_000,
            20260401,
            NonMacrsRecovery::FiniteLife { facts: finite_life(kind, period, period, 0) },
        );
        assert!(source.qualifies());
        assert_eq!(source.qualified179_cost(), 0);
        assert_eq!(source.deduction(false), expected);
        assert_eq!(source.deduction_for(false, true), expected);
        assert!(!source.qbi_property());
        assert_eq!(crate::depreciation::form_4562_line(&source, "macrs"), Some("16"));
    }
}

#[test]
fn unit17_r6_forecast_annual_amount_does_not_turn_missing_lookback_tax_into_zero() {
    let mut source = property(
        20_000_000,
        20230101,
        NonMacrsRecovery::IncomeForecast {
            facts: IncomeForecastRecovery {
                kind: ForecastPropertyKind::FilmVideo,
                current_income: 2_000_000,
                forecast_total_income: 10_000_000,
                remaining_basis: 6_000_000,
                participations: ForecastParticipations::None,
                lookback: ForecastLookback::WorksheetNeeded,
            },
        },
    );
    continuing(&mut source, 14_000_000);
    assert!(source.valid());
    assert_eq!(source.deduction(false), 4_000_000);
    assert!(!source.qualifies());
    assert_eq!(source.nonmacrs_issue(), Some("ASSET_FORECAST_LOOKBACK_UNAVAILABLE"));
    let NonMacrsRecovery::IncomeForecast { facts } = route_mut(&mut source) else { unreachable!() };
    facts.lookback =
        ForecastLookback::WithinTenPercent { actual: 11_000_000, forecast: 10_000_000 };
    assert!(source.qualifies());
    let NonMacrsRecovery::IncomeForecast { facts } = route_mut(&mut source) else { unreachable!() };
    facts.lookback =
        ForecastLookback::WithinTenPercent { actual: 11_000_001, forecast: 10_000_000 };
    assert!(!source.qualifies());
}

#[test]
fn unit17_r6_units_keep_known_zero_separate_from_incoherent_remaining_basis() {
    let mut source = property(
        1_000_000,
        20250101,
        NonMacrsRecovery::Units {
            facts: UnitProduction {
                qbi_class: GdsEquipmentClass::Seven,
                elected_originally: true,
                current_units: 0,
                remaining_units: 1000,
                remaining_basis: 800_000,
                salvage: 100_000,
            },
        },
    );
    continuing(&mut source, 200_000);
    assert!(source.valid());
    assert!(source.qualifies());
    assert_eq!(source.deduction(false), 0);
    let NonMacrsRecovery::Units { facts } = route_mut(&mut source) else { unreachable!() };
    facts.remaining_basis = 700_000;
    assert!(!source.valid());
    assert!(!source.qualifies());
    assert_eq!(source.nonmacrs_fact_issue(), Some("ASSET_NONMACRS_SOURCE_FACTS"));
}

#[test]
fn unit17_r6_expired_software_history_is_not_an_inferred_zero_balance() {
    let mut source = property(
        3_600_000,
        20230101,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    continuing(&mut source, 3_599_999);
    assert!(source.valid());
    assert!(!source.qualifies());
    assert_eq!(source.nonmacrs_issue(), Some("ASSET_NONMACRS_ORIGINAL_HISTORY"));
    continuing(&mut source, 3_600_000);
    assert!(source.valid());
    assert!(source.qualifies());
    assert_eq!(source.deduction(false), 0);
}

fn forecast_history(cost: i64, lookback: ForecastLookback) -> BusinessAsset {
    let mut source = property(
        cost,
        20230101,
        NonMacrsRecovery::IncomeForecast {
            facts: IncomeForecastRecovery {
                kind: ForecastPropertyKind::FilmVideo,
                current_income: 2_000_000,
                forecast_total_income: 10_000_000,
                remaining_basis: 6_000_000,
                participations: ForecastParticipations::None,
                lookback,
            },
        },
    );
    continuing(&mut source, cost - 6_000_000);
    source
}

#[test]
fn unit17_r6_forecast_known_cost_exception_is_factual_at_recomputation_year() {
    // Section167(g)(3) excludes cost basis <=$100000 from mandatory look-back.
    // The actual cost establishes it; a second exception choice adds no fact.
    let source = forecast_history(10_000_000, ForecastLookback::NoRecomputation);
    assert!(source.valid());
    assert_eq!(source.deduction(false), 2_000_000);
    assert!(source.qualifies());
    assert_eq!(source.nonmacrs_issue(), None);
}

#[test]
fn unit17_r6_forecast_small_cost_claim_over_one_cent_is_native_save_refused() {
    use crate::tests::cross::interview::{business, document, rejected_with, save_topic};
    let source = forecast_history(10_000_001, ForecastLookback::SmallCostException);
    let mut page = business(20_000_000);
    page.assets = crate::interview::vehicle_and_asset_items(&[], &[source]).unwrap().1;
    let mut session = restart(complete(FederalStatus::Single));
    let before = session.draft.clone();
    let dirty = session.dirty;
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    rejected_with(send(&mut session, request), "FORECAST_SMALL_COST_EXCEPTION");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}

#[test]
fn unit17_r6_forecast_ten_percent_claim_has_its_literal_native_fact_bound() {
    let source = forecast_history(
        20_000_000,
        ForecastLookback::WithinTenPercent { actual: 11_000_001, forecast: 10_000_000 },
    );
    assert!(!source.valid());
    assert_eq!(source.nonmacrs_fact_issue(), Some("FORECAST_NOT_WITHIN_TEN_PERCENT"));
    // Unentered mandatory tax is a different source state, not invalid money.
    let unknown = forecast_history(20_000_000, ForecastLookback::WorksheetNeeded);
    assert!(unknown.valid());
    assert_eq!(unknown.deduction(false), 4_000_000);
    assert_eq!(unknown.nonmacrs_issue(), Some("ASSET_FORECAST_LOOKBACK_UNAVAILABLE"));
}

#[test]
fn unit17_r6_forecast_cost_exception_does_not_shrink_with_use_or_credit() {
    for reduced_by_credit in [false, true] {
        let mut source = forecast_history(20_000_000, ForecastLookback::SmallCostException);
        if reduced_by_credit {
            source.credit_reduction = 10_000_000;
        } else {
            source.listed = ListedProperty::No { business_percent: 50 };
        }
        continuing(&mut source, 4_000_000);
        assert_eq!(source.business_basis(), 10_000_000);
        assert_eq!(source.opening_basis(), 6_000_000);
        // i8866 Exception: unadjusted basis is total capitalized cost. Neither
        // a use fraction nor a tax-credit basis adjustment changes that cost.
        assert!(!source.valid());
        assert_eq!(source.nonmacrs_fact_issue(), Some("FORECAST_SMALL_COST_EXCEPTION"));
    }
}

#[test]
fn unit17_r6_forecast_tenth_year_does_not_invent_a_late_catchup_deduction() {
    let mut source = forecast_history(10_000_000, ForecastLookback::SmallCostException);
    source.placed = 20150101;
    source.acquired = 20150101;
    // Section167(g)(1)(C) writes off in the tenth following year, not an
    // arbitrary later year with an unexplained retained balance.
    assert!(source.valid());
    assert!(!source.qualifies());
    assert_eq!(source.nonmacrs_issue(), Some("ASSET_NONMACRS_ORIGINAL_HISTORY"));
}

#[test]
fn unit17_r6_forecast_participation_basis_needs_the_actual_per_year_exception() {
    let mut source = forecast_history(
        20_000_000,
        ForecastLookback::WithinTenPercent { actual: 10_000_000, forecast: 10_000_000 },
    );
    assert!(source.qualifies());
    let NonMacrsRecovery::IncomeForecast { facts } = route_mut(&mut source) else { unreachable!() };
    facts.participations = ForecastParticipations::Basis;
    // Section167(g)(7)(C) requires each year's comparison in this case.
    // Equal cumulative totals cannot establish those absent individual rows.
    assert!(source.valid());
    assert_eq!(source.deduction(false), 4_000_000);
    assert!(!source.qualifies());
    assert_eq!(source.nonmacrs_issue(), Some("ASSET_FORECAST_LOOKBACK_UNAVAILABLE"));
}

fn acrs_45_source() -> BusinessAsset {
    // Pub534 Table14: June1986 MM firstyear1.2%, ten23/1000 years,
    // then twenty-nine22/1000 years through2025:17600000c actualprior.
    let mut source = property(
        20_000_000,
        19860615,
        NonMacrsRecovery::Historical {
            facts: HistoricalDepreciation {
                qbi_class: GdsEquipmentClass::ThirtyNine,
                property: HistoricalPropertyKind::RealProperty,
                kind: HistoricalRecoveryKind::Acrs {
                    period: AcrsPeriod::Alternate {
                        years: 45,
                        original_class: AcrsOriginalClass::Real19,
                    },
                    alternate: true,
                },
                remaining_basis: 2_400_000,
                original_convention: AssetConvention::MidMonth,
            },
        },
    );
    source.method = EquipmentMethod::StraightLine;
    continuing(&mut source, 17_600_000);
    source
}

#[test]
fn unit17_r6_acrs_45_continues_actual_table14_instead_of_refusing_its_source() {
    let source = acrs_45_source();
    assert!(source.valid());
    // 2026 is the41st calendar recovery year: actualTable14 uses2.2%.
    assert_eq!(source.opening_basis(), 2_400_000);
    assert_eq!(source.deduction(false), 440_000);
    assert_eq!(source.deduction_for(false, true), 440_000);
    assert!(source.qualifies());
    assert_eq!(
        {
            let r = source.nonmacrs_recovery(0).unwrap();
            (r.quarter_months, r.method, r.convention)
        },
        (2160, "Alternate ACRS", AssetConvention::MidMonth)
    );
}

#[test]
fn unit17_r6_acrs_45_actual_original_full_month_reaches_its_final_table15_year() {
    let mut source = acrs_45_source();
    source.acquired = 19810301;
    source.placed = 19810301;
    continuing(&mut source, 19_920_000);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.remaining_basis = 80_000;
    facts.original_convention = AssetConvention::ActualMonths;
    facts.kind = HistoricalRecoveryKind::Acrs {
        period: AcrsPeriod::Alternate { years: 45, original_class: AcrsOriginalClass::Real15 },
        alternate: true,
    };
    // Table15: March firstyear1.9%, nine2.3% and thirty-five2.2%
    // years give99.6%; final March source percentage is0.4%.
    assert!(source.valid());
    assert_eq!(source.deduction(false), 80_000);
    assert!(source.qualifies());
}

#[test]
fn unit17_r6_acrs_expired_known_zero_is_distinct_from_unexplained_remaining_basis() {
    let mut source = acrs_45_source();
    continuing(&mut source, 20_000_000);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.kind = HistoricalRecoveryKind::Acrs {
        period: AcrsPeriod::Alternate { years: 35, original_class: AcrsOriginalClass::Real19 },
        alternate: true,
    };
    facts.remaining_basis = 0;
    assert!(source.valid());
    assert_eq!(source.deduction(false), 0);
    assert!(source.qualifies());
    continuing(&mut source, 19_999_999);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.remaining_basis = 1;
    assert!(!source.qualifies());
    assert_eq!(source.nonmacrs_issue(), Some("ASSET_NONMACRS_ORIGINAL_HISTORY"));
}

#[test]
fn unit17_r6_acrs_structural_year_bound_does_not_legalize_a44_year_election() {
    let mut source = acrs_45_source();
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.kind = HistoricalRecoveryKind::Acrs {
        period: AcrsPeriod::Alternate { years: 44, original_class: AcrsOriginalClass::Real19 },
        alternate: true,
    };
    assert!(!source.valid());
    assert_eq!(source.nonmacrs_fact_issue(), Some("ASSET_NONMACRS_SOURCE_FACTS"));
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.kind = HistoricalRecoveryKind::Acrs {
        period: AcrsPeriod::Alternate { years: 25, original_class: AcrsOriginalClass::Real19 },
        alternate: true,
    };
    // Pub534 permits 19,35,45 for original19-year real property;25 is
    // an alternate for personal property, not an arbitrary real-property election.
    assert_eq!(source.nonmacrs_fact_issue(), Some("ASSET_NONMACRS_SOURCE_FACTS"));
}

fn forecast_final_with_zero_income(cost: i64, lookback: ForecastLookback) -> BusinessAsset {
    let mut source = forecast_history(cost, lookback);
    source.placed = 20160101;
    source.acquired = 20160101;
    // No actual earned income ever and no original bonus/179: retain zero
    // prior recovery, not the generic positive-income helper's old allowance.
    continuing(&mut source, 0);
    let NonMacrsRecovery::IncomeForecast { facts } = route_mut(&mut source) else { unreachable!() };
    facts.current_income = 0;
    facts.forecast_total_income = 0;
    facts.remaining_basis = cost;
    source
}

#[test]
fn unit17_r6_forecast_final_known_zero_income_does_not_block_statutory_remaining_basis() {
    // Section167(g)(1)(C): the tenth following year deducts opening adjusted
    // basis. This does not divide by a zero forecast or invent earned income.
    let source = forecast_final_with_zero_income(10_000_000, ForecastLookback::NoRecomputation);
    assert!(source.valid());
    assert_eq!(source.deduction(false), 10_000_000);
    assert!(source.qualifies());
    assert_eq!(source.nonmacrs_issue(), None);
}

#[test]
fn unit17_r6_forecast_final_known_annual_amount_keeps_required_lookback_tax_unavailable() {
    let source = forecast_final_with_zero_income(20_000_000, ForecastLookback::WorksheetNeeded);
    assert!(source.valid());
    assert_eq!(source.deduction(false), 20_000_000);
    assert!(!source.qualifies());
    assert_eq!(source.nonmacrs_issue(), Some("ASSET_FORECAST_LOOKBACK_UNAVAILABLE"));
}

#[test]
fn unit17_r6_forecast_uses_other_depreciation_line16_not_units_election_line15() {
    let source = forecast_history(10_000_000, ForecastLookback::NoRecomputation);
    assert!(source.valid());
    assert!(source.qualifies());
    assert_eq!(source.deduction(false), 2_000_000);
    // Official2026Form4562: line15 is only168(f)(1);167(g) is line16.
    assert_eq!(crate::depreciation::form_4562_line(&source, "macrs"), Some("16"));
    let units = property(
        10_000_000,
        20260101,
        NonMacrsRecovery::Units {
            facts: UnitProduction {
                qbi_class: GdsEquipmentClass::Seven,
                elected_originally: true,
                current_units: 100,
                remaining_units: 1000,
                remaining_basis: 10_000_000,
                salvage: 0,
            },
        },
    );
    assert!(units.valid());
    assert_eq!(crate::depreciation::form_4562_line(&units, "macrs"), Some("15"));
}

#[test]
fn unit17_r6_forecast_final_zero_recomputation_sources_satisfy_exact_ten_percent_test() {
    let source = forecast_final_with_zero_income(
        20_000_000,
        ForecastLookback::WithinTenPercent { actual: 0, forecast: 0 },
    );
    // Both source records actually show zero; their difference is within10%
    // of the original zero. No division or invented positive forecast is used.
    assert!(source.valid());
    assert_eq!(source.deduction(false), 20_000_000);
    assert!(source.qualifies());
    assert_eq!(source.nonmacrs_issue(), None);
}

#[test]
fn unit17_r6_normal_patent_software_save_reopen_restart_keeps_sources_and_form16() {
    use crate::tests::cross::interview::{
        accepted, business, document, open_topic, save_topic, stored_business,
    };
    let patent = property(
        510_000,
        20260401,
        NonMacrsRecovery::FiniteLife {
            facts: FiniteLifeRecovery::PatentCopyright {
                legal_remaining_months: 204,
                useful_months: 204,
                salvage: 0,
            },
        },
    );
    let software = property(
        3_600_000,
        20260701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    let mut page = business(10_000_000);
    page.assets = crate::interview::vehicle_and_asset_items(&[], &[patent, software]).unwrap().1;
    let mut session = restart(complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    accepted(&send(&mut session, request));
    let (id, source) = stored_business(&session);
    let sources = source.assets.clone();
    assert!(sources[0].id > 0 && sources[1].id > 0 && sources[0].id != sources[1].id);
    assert_eq!(sources[0].deduction(false), 22_500);
    assert_eq!(sources[1].cost, 3_600_000);
    assert_eq!(sources[1].deduction(false), 600_000);
    assert_eq!(sources.iter().map(BusinessAsset::qualified179_cost).sum::<i64>(), 0);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), 93_775);
    for index in 0..2 {
        assert_eq!(
            result
                .lines
                .iter()
                .find(|row| row.id == alloc::format!("business_{id}_asset_{index}_macrs"))
                .unwrap()
                .form_line,
            Some(FormLine::subrow(TaxForm::Form4562, "16"))
        );
    }
    let reported = result
        .lines
        .iter()
        .find(|row| row.id == alloc::format!("business_{id}_depreciation_class_16"))
        .unwrap();
    assert_eq!(reported.form_line, Some(FormLine::new(TaxForm::Form4562, "16")));
    assert_eq!(reported.amount, 622_500);
    let mut again = restart(session.draft.clone());
    let request = open_topic(&mut again, TaxSection::SelfEmployment);
    accepted(&send(&mut again, request));
    assert_eq!(stored_business(&again).0, id);
    assert_eq!(stored_business(&again).1.assets, sources);
    assert_eq!(
        value(&supported(&again.draft), &alloc::format!("business_{id}_net_profit")),
        93_775
    );
}

#[test]
fn unit17_r6_acrs_actual_business_save_reopen_restart_keeps_original_source() {
    use crate::tests::cross::interview::{
        accepted, business, document, open_topic, save_topic, stored_business,
    };
    let source = acrs_45_source();
    let mut page = business(10_000_000);
    page.assets = crate::interview::vehicle_and_asset_items(&[], &[source]).unwrap().1;
    let mut session = restart(complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    accepted(&send(&mut session, request));
    let (id, record) = stored_business(&session);
    let source = record.assets[0].clone();
    assert!(source.id > 0);
    assert_eq!(source.placed, 19860615);
    assert_eq!(source.opening_basis(), 2_400_000);
    assert_eq!(source.deduction(false), 440_000);
    let mut again = restart(session.draft.clone());
    let request = open_topic(&mut again, TaxSection::SelfEmployment);
    accepted(&send(&mut again, request));
    assert_eq!(stored_business(&again).0, id);
    assert_eq!(stored_business(&again).1.assets[0], source);
    assert_eq!(
        value(&supported(&again.draft), &alloc::format!("business_{id}_net_profit")),
        95_600
    );
}

#[test]
fn unit17_r6_acrs_exact_published_percent_reaches_the_annual_category_once() {
    use crate::tests::cross::interview::{
        accepted, business, document, save_topic, stored_business,
    };
    let mut source = acrs_45_source();
    source.cost = 20_002_250;
    continuing(&mut source, 17_601_000);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.remaining_basis = 2_401_250;
    assert!(source.valid());
    assert_eq!(source.deduction(false), 440_050);
    let mut page = business(10_000_000);
    page.assets = crate::interview::vehicle_and_asset_items(&[], &[source]).unwrap().1;
    let mut session = restart(complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    accepted(&send(&mut session, request));
    let (id, _) = stored_business(&session);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_depreciation")), 4_400);
    assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), 95_600);
}

#[test]
fn unit17_r6_acrs_same_reported_class_keeps_exact_sources_until_one_projection() {
    let mut source = acrs_45_source();
    source.cost = 20_002_250;
    continuing(&mut source, 17_601_000);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.remaining_basis = 2_401_250;
    let amounts =
        crate::nonmacrs::asset_depreciation_amounts(&[source.clone(), source], false, false);
    assert_eq!(crate::kernel::ratio::exact::dollars(&amounts.classes["16"]), Some(8_801));
    assert_eq!(amounts.expense_cents(0), Some(880_100));
}

#[test]
fn unit17_r6_forecast_final_known_zero_normal_save_reopen_restart_keeps_sources() {
    use crate::tests::cross::interview::{
        accepted, business, document, open_topic, save_topic, stored_business,
    };
    let source = forecast_final_with_zero_income(10_000_000, ForecastLookback::NoRecomputation);
    let mut page = business(10_000_000);
    page.assets = crate::interview::vehicle_and_asset_items(&[], &[source]).unwrap().1;
    let mut session = restart(complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    accepted(&send(&mut session, request));
    let (id, record) = stored_business(&session);
    let source = record.assets[0].clone();
    assert!(source.id > 0);
    assert_eq!(source.placed, 20160101);
    assert_eq!(source.cost, 10_000_000);
    assert_eq!(source.opening_basis(), 10_000_000);
    assert_eq!(source.deduction(false), 10_000_000);
    assert_eq!(value(&supported(&session.draft), &alloc::format!("business_{id}_net_profit")), 0);
    let mut again = restart(session.draft.clone());
    let request = open_topic(&mut again, TaxSection::SelfEmployment);
    accepted(&send(&mut again, request));
    assert_eq!(stored_business(&again).0, id);
    assert_eq!(stored_business(&again).1.assets[0], source);
}

#[test]
fn purchased_software_disposition_uses_actual_months_basis_and_existing_recapture_owner() {
    let mut source = property(
        3_600_000,
        20250701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    continuing(&mut source, 600_000);
    source.disposal = AssetDisposal::Disposed {
        date: 20261001,
        how: DisposalKind::Sold {
            proceeds: 3_000_000,
            selling_expenses: 0,
            payment: SalePayment::InFull,
            buyer: Buyer::Unrelated,
            livestock: crate::SoldLivestock::No,
        },
    };
    assert!(source.valid() && source.qualifies());
    assert_eq!(source.deduction(false), 900_000);
    assert_eq!(source.deduction_for(false, true), 900_000);
    let sale = source.sale_for(false, false).unwrap();
    assert_eq!(sale.depreciation, 15_000);
    assert_eq!(sale.gain, 9_000);
    assert_eq!(sale.ordinary, 9_000);
    assert_eq!(sale.section1231, 0);
}

#[test]
fn unit17_r6_nonmacrs_is_outside_the_macrs_midquarter_pool_and_election_class() {
    use crate::tests::cross::interview::{
        accepted, business, document, save_topic, stored_business,
    };
    let mut page = business(20_000_000);
    page.assets = vec![
        with_first_year(asset(700_000, 20260101, AssetClass::Seven), FirstYearDeduction::Regular),
        with_first_year(asset(300_000, 20261001, AssetClass::Seven), FirstYearDeduction::Regular),
    ];
    let patent = property(
        10_000_000,
        20261001,
        NonMacrsRecovery::FiniteLife {
            facts: FiniteLifeRecovery::PatentCopyright {
                legal_remaining_months: 204,
                useful_months: 204,
                salvage: 0,
            },
        },
    );
    page.assets.extend(crate::interview::vehicle_and_asset_items(&[], &[patent]).unwrap().1);
    let mut session = restart(complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    accepted(&send(&mut session, request));
    assert!(!crate::depreciation::mid_quarter(&session.draft));
    let result = supported(&session.draft);
    let (id, _) = stored_business(&session);
    for (index, expected) in [(0, 100_030), (1, 42_870)] {
        assert_eq!(
            result
                .lines
                .iter()
                .find(|line| line.id == alloc::format!("business_{id}_asset_{index}_macrs"))
                .unwrap()
                .amount,
            expected
        );
    }
    let source = property(
        360_000,
        20260101,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::PublicNonexclusive,
                exempt_lease: None,
                bonus_eligible: true,
            },
        },
    );
    let mut direct = session.draft.clone();
    let FederalIncome::Business { data } =
        &mut direct.income.iter_mut().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    let mut ordinary = crate::tests::support::builders::bonus_equipment(5_000, 100);
    ordinary.placed = 20260101;
    data.assets = vec![source.clone(), ordinary];
    assert!(crate::depreciation::consistent_elections(&direct));
    let FederalIncome::Business { data } =
        &mut direct.income.iter_mut().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    let mut another = source;
    let EquipmentHistory::New { bonus, .. } = &mut another.history else { unreachable!() };
    *bonus = BonusElection::Apply;
    data.assets.push(another);
    assert!(!crate::depreciation::consistent_elections(&direct));
}

#[test]
fn unit17_r8_prior_only_exact_and_legacy_sources_share_one_annual_category() {
    use crate::tests::cross::interview::{
        accepted, business, document, save_topic, stored_business,
    };
    let mut source = acrs_45_source();
    source.cost = 20_002_250;
    continuing(&mut source, 17_601_000);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.remaining_basis = 2_401_250;
    let mut legacy = crate::tests::support::builders::macrs_equipment(
        1,
        EquipmentClass::Five,
        EquipmentMethod::Regular,
        20250301,
    );
    legacy.cost = 31_403;
    legacy.acquired = 20250301;
    continuing(&mut legacy, 6_281);
    assert_eq!(source.deduction(false), 440_050);
    assert_eq!(legacy.deduction(false), 10_049);
    let sources = [source, legacy];
    assert!(sources.iter().all(|asset| asset.valid() && asset.qualifies()));
    assert!(!crate::depreciation::assets_need_form_4562(&sources));
    let mut page = business(10_000_000);
    page.assets = crate::interview::vehicle_and_asset_items(&[], &sources).unwrap().1;
    let mut session = restart(complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })],
    );
    accepted(&send(&mut session, request));
    let (id, _) = stored_business(&session);
    let result = supported(&session.draft);
    assert!(
        !result
            .lines
            .iter()
            .any(|row| row.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form4562))
    );
    assert_eq!(
        result
            .lines
            .iter()
            .find(|row| row.id == alloc::format!("business_{id}_depreciation"))
            .unwrap()
            .amount,
        450_100
    );
    assert_eq!(value(&result, &alloc::format!("business_{id}_depreciation")), 4_501);
    assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), 95_499);
}

#[test]
fn original_accelerated_leased_or_real_history_does_not_silently_reuse_regular_amt() {
    let mut source = property(
        10_000_000,
        19800101,
        NonMacrsRecovery::Historical {
            facts: HistoricalDepreciation {
                qbi_class: GdsEquipmentClass::Seven,
                property: HistoricalPropertyKind::PersonalOwned,
                kind: HistoricalRecoveryKind::Pre1981Declining {
                    multiplier: 2.0,
                    original_useful_months: 600,
                    salvage: 0,
                },
                remaining_basis: 800_000,
                original_convention: AssetConvention::ActualMonths,
            },
        },
    );
    continuing(&mut source, 9_200_000);
    assert!(source.qualifies());
    assert_eq!(source.deduction_for(false, true), 32_000);
    for property in [HistoricalPropertyKind::PersonalLeased, HistoricalPropertyKind::RealProperty] {
        let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
        facts.property = property;
        assert!(source.valid());
        assert_eq!(source.nonmacrs_issue(), Some("ASSET_NONMACRS_AMT_HISTORY"));
        assert!(!source.qualifies());
    }
}

#[test]
fn unit17_r6_community_farm_rental_keeps_operator_physical_units_available() {
    crate::tests::support::client::nonmacrs::farm_rental_physical_units_on_community_return(
        Owner::Taxpayer,
    );
}
#[test]
fn unit17_r6_community_farm_rental_keeps_counterpart_physical_units_available() {
    crate::tests::support::client::nonmacrs::farm_rental_physical_units_on_community_return(
        Owner::Spouse,
    );
}

#[test]
fn historical_original_life_uses_wide_arithmetic_without_an_arbitrary_month_cap() {
    let mut source = property(
        1_000_000,
        19800101,
        NonMacrsRecovery::Historical {
            facts: HistoricalDepreciation {
                qbi_class: GdsEquipmentClass::Seven,
                property: HistoricalPropertyKind::PersonalOwned,
                kind: HistoricalRecoveryKind::Pre1981Declining {
                    multiplier: 2.0,
                    original_useful_months: u32::MAX,
                    salvage: 0,
                },
                remaining_basis: 800_000,
                original_convention: AssetConvention::ActualMonths,
            },
        },
    );
    continuing(&mut source, 200_000);
    assert!(source.valid());
    assert!(!source.qbi_property());
    assert_eq!(source.deduction(false), 0);
}

#[test]
fn canonical_real_rental_nonmacrs_save_reopen_retains_original_schedule_and_form16() {
    use crate::tests::cross::interview::{accepted, rental_page, save_rentals, stored_rentals};
    let source = property(
        3_600_000,
        20260701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    let mut page = rental_page(10_000_000);
    page.assets = vec![source.into()];
    let mut session = restart(complete(FederalStatus::Single));
    accepted(&save_rentals(&mut session, vec![page]));
    let (id, _, stored) = stored_rentals(&session).remove(0);
    assert_eq!(stored.assets[0].deduction(false), 600_000);
    let result = supported(&session.draft);
    let row = result
        .lines
        .iter()
        .find(|line| line.id == alloc::format!("rental_{id}_depreciation_class_16"))
        .unwrap();
    assert_eq!(row.amount, 600_000);
    assert_eq!(row.form_line, Some(FormLine::new(TaxForm::Form4562, "16")));
    let mut again = restart(session.draft.clone());
    let request = crate::tests::cross::interview::open_topic(&mut again, TaxSection::Rentals);
    accepted(&send(&mut again, request));
    assert_eq!(stored_rentals(&again)[0].2.assets, stored.assets);
}

#[test]
fn real_rental_original_acrs_fraction_reaches_existing_personal_day_owner_before_rounding() {
    let mut source = acrs_45_source();
    source.cost = 20_002_250;
    continuing(&mut source, 17_601_000);
    let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
    facts.remaining_basis = 2_401_250;
    let mut native = rental(100_000, 0);
    native.buildings[0].in_service = 19870101;
    native.assets = vec![source];
    native.personal_use = crate::RentalPersonalUse::Yes {
        fair_rental_days: 200,
        personal_days: 100,
        overlap_days: 0,
        direct_expenses: 0,
    };
    // Original Table14 amount440049.5cents ×200/300 =293366.333... .
    // Rounding the source to440050cents first would incorrectly yield293367.
    assert_eq!(crate::extended_rules::rental_net(&native), Ok(9_706_634));
    native.assets = vec![forecast_history(20_000_000, ForecastLookback::WorksheetNeeded)];
    assert_eq!(
        crate::extended_rules::rental_net(&native).unwrap_err().0,
        "ASSET_FORECAST_LOOKBACK_UNAVAILABLE"
    );
}

#[test]
fn nonbusiness_rental_software_uses_actual_source_and_normal_financial_consumer() {
    use crate::tests::cross::interview::{accepted, document, open_topic, save_topic};
    let source = property(
        3_600_000,
        20260701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    let page = EquipmentRent {
        related: crate::RelatedForms::default(),
        receipts: 10_000_000,
        expenses: 0,
        basis: 3_600_000,
        depreciable_basis: 3_600_000,
        assets: crate::interview::vehicle_and_asset_items(&[], &[source]).unwrap().1,
        history: PersonalRentalHistory::NONE,
    };
    let mut session = restart(complete(FederalStatus::Single));
    let request = save_topic(
        &mut session,
        TaxSection::OtherIncome,
        vec![document(
            0,
            IncomeSource::Record { value: crate::IncomeRecord::EquipmentRental(page) },
        )],
    );
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "nonbusiness_personal_rent_depreciation"), 6_000);
    assert!(
        result
            .lines
            .iter()
            .any(|row| row.form_line == Some(FormLine::new(TaxForm::Form4562, "16"))
                && row.amount == 600_000)
    );
    let accepted = session.draft.clone();
    let mut again = restart(accepted.clone());
    let request = open_topic(&mut again, TaxSection::OtherIncome);
    crate::tests::cross::interview::accepted(&send(&mut again, request));
    assert_eq!(again.draft.income, accepted.income);
}

#[test]
fn community_real_rental_uses_whole_units_and_lifetime_sale_recovery_before_each_share() {
    for farm in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut source = property(
                10_098,
                20250101,
                NonMacrsRecovery::Units {
                    facts: UnitProduction {
                        qbi_class: GdsEquipmentClass::Seven,
                        elected_originally: true,
                        current_units: 1,
                        remaining_units: 1,
                        remaining_basis: 5_049,
                        salvage: 0,
                    },
                },
            );
            continuing(&mut source, 5_049);
            source.disposal = AssetDisposal::Disposed {
                date: 20261231,
                how: DisposalKind::Sold {
                    proceeds: 10_098,
                    selling_expenses: 0,
                    payment: SalePayment::InFull,
                    buyer: Buyer::Unrelated,
                    livestock: crate::SoldLivestock::No,
                },
            };
            assert!(source.valid() && source.qualifies());
            assert_eq!(source.deduction(false), 5_049);
            let mut draft = complete(FederalStatus::Separate);
            draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
            draft.setup.as_mut().unwrap().community_property = CommunityProperty::Yes {
                state: CommunityState::Arizona,
                rule: CommunityRule::Halves,
            };
            for item in &mut draft.income {
                item.community = CommunityShare::Separate;
            }
            let carrier = if farm {
                let mut native = farm_rental_data(100_000, 0);
                native.assets = vec![source];
                FederalIncome::FarmRental { data: native }
            } else {
                let mut native = rental(100_000, 0);
                native.buildings[0].in_service = 19870101;
                native.assets = vec![source];
                FederalIncome::Rental { data: native }
            };
            add(&mut draft, income(carrier));
            let entry = draft.income.last_mut().unwrap();
            entry.owner = crate::DocumentOwner::from(owner);
            entry.community = CommunityShare::Community;
            let id = entry.id;
            let divided = crate::community::record(&draft, draft.income.last().unwrap());
            let assets = match divided.as_ref() {
                FederalIncome::Rental { data } => &data.assets,
                FederalIncome::FarmRental { data } => &data.assets,
                _ => unreachable!(),
            };
            let amounts = crate::nonmacrs::rental_asset_amounts(
                assets,
                crate::nonmacrs::rental_recovery_source(&draft, id, false).as_ref(),
                false,
                false,
            )
            .unwrap();
            assert_eq!(
                crate::kernel::ratio::exact::rounded(&amounts["15"]),
                Some(if owner == Owner::Taxpayer { 2_525 } else { 2_524 })
            );
            let result = supported(&draft);
            let prefix = if farm { "business" } else { "rental" };
            assert_eq!(
                result
                    .lines
                    .iter()
                    .find(|line| line.id == alloc::format!("{prefix}_{id}_asset_0_macrs"))
                    .unwrap()
                    .amount,
                if owner == Owner::Taxpayer { 2_525 } else { 2_524 }
            );
            // Completed lifetime10098/2=5049c rounds to$50. Independently
            // rounded owner prior2525 +current2525 would incorrectly round to$51.
            assert_eq!(
                value(&result, &alloc::format!("{prefix}_{id}_asset_0_disposal_depreciation")),
                50
            );
            assert_eq!(
                value(&result, &alloc::format!("{prefix}_{id}_asset_0_disposal_adjusted_basis")),
                0
            );
            assert_eq!(
                value(&result, &alloc::format!("{prefix}_{id}_asset_0_disposal_1245_recapture")),
                50
            );
            assert_eq!(
                crate::section1231::gross_property(
                    &draft,
                    &crate::extended_rules::RentalResult::default(),
                    false
                )
                .unwrap(),
                50
            );
            if farm {
                // Whole annual recovery rounds 5049 cents to $50 before the
                // Form 8958 total is shared; the 2524/2525-cent parts do not
                // become deductions in that whole-source column.
                for (suffix, expected) in [("total", 9_995_000), ("this_return", 4_997_500)] {
                    assert_eq!(
                        result
                            .lines
                            .iter()
                            .find(|line| line.id == alloc::format!("form8958_{id}_{suffix}"))
                            .unwrap()
                            .amount,
                        expected
                    );
                }
                let value = crate::asset_sale::farm_rental_property_source_with_claims(
                    &draft,
                    id,
                    false,
                    &crate::related_forms::effective_claimants(&draft),
                    None,
                )
                .unwrap()
                .value
                .unwrap();
                assert_eq!((value.gain, value.recapture, value.ordinary), (50, 50, 50));
                let value = crate::asset_sale::direct_sources(&draft, false)
                    .into_iter()
                    .find(|(source, _)| *source == id)
                    .unwrap()
                    .1;
                assert_eq!((value.gain, value.recapture, value.ordinary), (50, 50, 50));
            } else {
                assert_eq!(crate::asset_sale::rental_property_gross(&draft, false), 50);
                let property =
                    crate::community::property_entry(&draft, draft.income.last().unwrap());
                let FederalIncome::Rental { data } = &property.income else { unreachable!() };
                assert_eq!(
                    crate::asset_sale::rental_sale_amount(
                        crate::related_forms::RelatedFormContext::original(
                            &draft,
                            &crate::related_forms::effective_claimants(&draft)
                        ),
                        id,
                        data,
                        false,
                        false,
                        None
                    ),
                    Ok(5_000)
                );
            }
        }
    }
}

#[test]
fn separate_rental_property_share_preserves_acrs_fraction_without_a_nonowner_residual() {
    use crate::kernel::ratio::exact;
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut source = acrs_45_source();
        source.cost = 20_002_250;
        continuing(&mut source, 17_601_000);
        let NonMacrsRecovery::Historical { facts } = route_mut(&mut source) else { unreachable!() };
        facts.remaining_basis = 2_401_250;
        let mut draft = complete(FederalStatus::Separate);
        draft.setup.as_mut().unwrap().community_property =
            CommunityProperty::Yes { state: CommunityState::Texas, rule: CommunityRule::Halves };
        let mut native = rental(100_000, 0);
        native.assets = vec![source];
        let mut entry = income(FederalIncome::Rental { data: native });
        entry.owner = crate::DocumentOwner::from(owner);
        entry.community = CommunityShare::Separate;
        add(&mut draft, entry);
        let entry = draft.income.last().unwrap();
        // Texas divides income from separate property; property gains keep
        // the owner's whole original fraction and the other spouse's zero.
        for (property, expected) in [
            (
                false,
                if owner == Owner::Taxpayer {
                    exact::whole(220_025)
                } else {
                    exact::ratio(440_049, 2).unwrap()
                },
            ),
            (
                true,
                if owner == Owner::Taxpayer {
                    exact::ratio(880_099, 2).unwrap()
                } else {
                    exact::whole(0)
                },
            ),
        ] {
            let FederalIncome::Rental { data } = &entry.income else { unreachable!() };
            let amounts = crate::nonmacrs::rental_asset_amounts(
                &data.assets,
                crate::nonmacrs::rental_recovery_source(&draft, entry.id, property).as_ref(),
                false,
                false,
            )
            .unwrap();
            assert_eq!(amounts["16"], expected);
        }
    }
}

#[test]
fn mixed_nonmacrs_and_refunded_macrs_keep_each_owned_deduction() {
    let (mut draft, activity, _) =
        cooperative_asset_refunds(bonus_equipment(1000, 100), &[[20_000, 0, 0, 0]]);
    let mut software = property(
        120_000,
        20260101,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::Standalone,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    software.id = draft.next_id;
    draft.next_id += 1;
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    data.assets.push(software);
    let amounts = crate::nonmacrs::asset_depreciation_amounts_for(
        &draft,
        activity,
        match &draft.income[0].income {
            FederalIncome::Business { data } => &data.assets,
            _ => unreachable!(),
        },
        false,
        false,
        None,
    )
    .unwrap();
    // The refunded equipment owns $800 bonus; the software owns $400 over 36 months.
    assert_eq!(amounts.expense_cents(0), Some(120_000));
    assert_eq!(crate::kernel::ratio::exact::dollars(&amounts.classes["14a"]), Some(800));
    assert_eq!(crate::kernel::ratio::exact::dollars(&amounts.classes["16"]), Some(400));
}

#[test]
fn new_nonmacrs_row_initializes_choices_before_its_history_calculation() {
    use crate::tests::cross::interview::open_topic;
    let mut session = restart(complete(FederalStatus::Single));
    let P::SaveIncomes { initial_asset_amt_carry_choices, .. } =
        open_topic(&mut session, TaxSection::SelfEmployment)
    else {
        panic!("asset editor")
    };
    let source = property(
        3_600_000,
        20260701,
        NonMacrsRecovery::Software {
            facts: SoftwareRecovery {
                classification: SoftwareKind::PublicNonexclusive,
                exempt_lease: None,
                bonus_eligible: false,
            },
        },
    );
    let mut row = NonMacrsAssetPage::from(source);
    // CopyFrom constructs this hidden list before any field calculation reply.
    row.allowed_amt_carry = initial_asset_amt_carry_choices;
    assert!(row.allowed_amt_carry.is_empty());
    assert_eq!(row.amt_section179_income_carry, None);
    let NonMacrsPropertyRecovery::Software { history, .. } = &mut row.recovery else {
        unreachable!()
    };
    *history = NonMacrsHistory::Continuing {
        prior_depreciation: 1_200_000,
        initial_section179: 100_000,
        carryforward: 10_000,
        original_bonus: 0,
    };
    let Reply::Response(P::NonMacrsCanonicalCarryChoices { allowed }) = send(
        &mut session,
        P::NonMacrsCanonicalCarryOptions { user: TEST_USER, recovery: row.recovery.clone() },
    ) else {
        panic!("actual original-history calculation")
    };
    assert_eq!(allowed, vec![0]);
    row.allowed_amt_carry = allowed;
    row.acquired = 20250101;
    row.placed = 20250101;
    assert_eq!(
        BusinessAsset::from(row.clone()).nonmacrs_issue(),
        Some("ASSET_NONMACRS_AMT_HISTORY")
    );
    row.amt_section179_income_carry = Some(AssetAmt179CarryAnswer::Records { amount: 10_000 });
    let native = BusinessAsset::from(row);
    assert!(native.valid());
    assert_eq!(native.nonmacrs_issue(), None);
    let reopened = NonMacrsAssetPage::from(native);
    assert_eq!(reopened.allowed_amt_carry, vec![0]);
    assert_eq!(
        reopened.amt_section179_income_carry,
        Some(AssetAmt179CarryAnswer::Records { amount: 10_000 })
    );
    let mut corrected = reopened;
    let NonMacrsPropertyRecovery::Software { history, .. } = &mut corrected.recovery else {
        unreachable!()
    };
    *history =
        NonMacrsHistory::New { section179: Section179::Ineligible, bonus: BonusElection::ElectOut };
    // Actual source applicability wins over a stale echoed catalog and answer.
    let native = BusinessAsset::from(corrected);
    assert_eq!(native.amt_section179_income_carry, None);
    assert!(native.amt_carry_choices().is_empty());
}

#[test]
fn actual_nonmacrs_recovery_reaches_each_business_home_income_limit_once() {
    for farming in [false, true] {
        let mut draft = empty_return(FederalStatus::Single);
        let mut entry = if farming { raised_farm(2_000, 1_500) } else { cash_business(2_000) };
        let FederalIncome::Business { data } = &mut entry else { unreachable!() };
        if !farming {
            data.expenses =
                vec![ordinary_business_expense(BusinessExpenseCategory::Supplies, 150_000)];
        }
        data.assets = vec![property(
            60_000,
            20260701,
            NonMacrsRecovery::Software {
                facts: SoftwareRecovery {
                    classification: SoftwareKind::Standalone,
                    exempt_lease: None,
                    bonus_eligible: false,
                },
            },
        )];
        add(&mut draft, income(entry));
        let id = draft.income.last().unwrap().id;
        add_simplified_office(&mut draft, id, 100);
        let result = supported(&draft);
        // $600 /36 months ×6 = $100. Receipts less $1,500 ordinary costs
        // and that recovery leave $400 for the otherwise $500 office allowance.
        assert_eq!(value(&result, &alloc::format!("business_{id}_home_office")), 400);
        let prefix = if farming { "farm" } else { "business" };
        assert_eq!(value(&result, &alloc::format!("{prefix}_{id}_net_profit")), 0);
        assert_eq!(supported(&restart(draft).draft), result);
    }
}
