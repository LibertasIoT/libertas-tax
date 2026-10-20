use crate::tests::*;

pub(in crate::tests) fn farm_business(
    receipts: alloc::vec::Vec<FarmReceipt>,
    expenses: i64,
) -> FederalIncome {
    let mut v = cash_business(0);
    let FederalIncome::Business { data } = &mut v else { unreachable!() };
    data.activity = BusinessActivity::Farming {
        data: FarmIncome {
            specials: farm_specials(),
            land_basis: vec![],
            header: Some(farm_header()),
            supplies: None,
            land_sales: vec![],
            conservation: None,
            crp: vec![],
            ccc: None,
            insurance: FarmInsurance::None,
            accounting: FarmAccounting::Cash { receipts },
        },
    };
    data.income_source = BusinessIncomeSource::Mixed { compensation: 3000000 };
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Feed, seeds and hired labor".into() },
        expenses * 100,
    )];
    v
}
pub(in crate::tests) fn raised_farm(proceeds: i64, expenses: i64) -> FederalIncome {
    farm_business(vec![FarmReceipt::Raised { amount: proceeds * 100 }], expenses)
}

pub(in crate::tests) fn accrual_farm(
    sales: i64,
    opening: i64,
    closing: i64,
    purchases: i64,
    expenses: i64,
) -> FederalIncome {
    let mut entry = raised_farm(0, expenses);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.accounting = FarmAccounting::Accrual {
        sales: sales * 100,
        opening: opening * 100,
        closing: closing * 100,
        purchases: purchases * 100,
        custom_hire: 0,
        program: 0,
        other: 0,
    };
    entry
}

pub(in crate::tests) fn farm_loss_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 1000000, 0)];
    add(&mut d, income(raised_farm(10000, 40000)));
    d
}
pub(in crate::tests) fn waive_farm_carryback(d: &mut FederalDraft) {
    d.screening.as_mut().unwrap().farm_nol = Some(FarmNolElection::Waive);
}

// Neutral normal-practice completion for legacy single-crop fixtures;
// the original arithmetic oracles are unchanged. New boundary probes supply
// their own crop-specific actual sales practice.
pub(in crate::tests) fn crop_practice() -> Vec<FarmCropPractice> {
    vec![FarmCropPractice {
        crop: "Grain".into(),
        normal_total: 1_000_000,
        normal_following: 750_000,
    }]
}

pub(in crate::tests) fn crop_records(
    current: alloc::vec::Vec<CropPayment>,
    previous: i64,
    defer: bool,
    withholding: i64,
) -> FarmInsurance {
    FarmInsurance::Payments {
        current,
        previous_deferred: previous * 100,
        election: if defer {
            CropDeferral::Defer { practice: crop_practice() }
        } else {
            CropDeferral::Include
        },
        withholding: withholding * 100,
    }
}
pub(in crate::tests) fn crop_farm(insurance: FarmInsurance) -> FederalIncome {
    let mut entry = raised_farm(30000, 10000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.insurance = insurance;
    entry
}
pub(in crate::tests) fn crop_return(insurance: FarmInsurance) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(crop_farm(insurance)));
    d
}

// Neutral completion facts for existing CCC fixtures: original retained
// section77 election in2025, or no original income election. New source
// history tests below supply their own actual original-year variants.
pub(in crate::tests) fn ccc_source(method: &CccMethod) -> CccSourceFacts {
    CccSourceFacts {
        producer_owned_raised: true,
        unrecovered_production_cost: 0,
        history: match method {
            CccMethod::Loan => CccElectionHistory::NoIncomeElection,
            CccMethod::Income => CccElectionHistory::IncomeElection {
                year: 2025,
                statement: "Retained original timely section77 loan-detail election statement"
                    .into(),
            },
        },
    }
}
pub(in crate::tests) fn ccc_source_input(method: &CccMethod) -> CccSourceFactsInput {
    (&ccc_source(method)).into()
}

pub(in crate::tests) fn ccc_return(
    origin: CccOrigin,
    method: CccMethod,
    disposition: CccDisposition,
    ordinary_profit: i64,
) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(ordinary_profit + 10000, 10000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let source = ccc_source(&method);
    data.ccc = Some(FarmCcc {
        source,
        method,
        lots: vec![CccLot { id: 0, origin, principal: 520000, disposition }],
        withholding: 100000,
    });
    add(&mut d, income(entry));
    d
}
pub(in crate::tests) fn ccc_mut(d: &mut FederalDraft) -> &mut FarmCcc {
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.ccc.as_mut().unwrap()
}
pub(in crate::tests) fn ccc_fifty_thousand(d: &FederalDraft) -> FederalResult {
    let r = supported(d);
    // Independently known 2026 single return: $50,000 profit, $7,065 SE,
    // $3,533 half-SE deduction, $46,467 AGI, $6,073 QBI deduction, $9,732 total tax.
    assert_eq!(r.tax, Some(973000));
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(return_line(&r, "10"), 353300);
    assert_eq!(return_line(&r, "25d"), 100000);
    assert_eq!(r.balance, Some(873000));
    r
}

pub(in crate::tests) fn crp_return(profit: i64, payments: &[(i64, bool)]) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(profit + 10000, 10000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.crp = payments
        .iter()
        .map(|(amount, benefits)| CrpPayment {
            amount: amount * 100,
            benefits: *benefits,
            withholding: 0,
        })
        .collect();
    add(&mut d, income(entry));
    d
}

pub(in crate::tests) fn se_history() -> NonfarmSeHistory {
    NonfarmSeHistory {
        earnings_third_prior_year: 40000,
        earnings_second_prior_year: 40000,
        earnings_prior_year: 0,
        prior_years: vec![],
    }
}
// Retained fixture shorthand; the emitted App owns independent farm/nonfarm fields.
pub(in crate::tests) enum SeMethod {
    Farm,
    Nonfarm { history: NonfarmSeHistory },
    Both { history: NonfarmSeHistory },
}
pub(in crate::tests) fn elect_se(d: &mut FederalDraft, method: SeMethod) {
    d.screening.as_mut().unwrap().optional_se = vec![SeElection {
        owner: Owner::Taxpayer,
        farm: !matches!(method, SeMethod::Nonfarm { .. }),
        nonfarm: match method {
            SeMethod::Farm => None,
            SeMethod::Nonfarm { history } | SeMethod::Both { history } => Some(history),
        },
    }];
}

pub(in crate::tests) fn se_health(d: &mut FederalDraft, index: usize, premium: i64) {
    let FederalIncome::Business { data } = &mut d.income[index].income else { unreachable!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        long_term_care: vec![],
        months: vec![BusinessHealthMonth {
            child_increase: 0,
            month: 1,
            premiums: premium * 100,
            employer_eligible: false,
            reimbursed: None,
        }],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
}

pub(in crate::tests) fn averaging_return(elected: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(120000, 20000)));
    d.screening.as_mut().unwrap().farm_averaging = Some(FarmAveraging {
        elected: elected * 100,
        years: (2023..=2025)
            .map(|year| AveragingBaseYear {
                year,
                status: FederalStatus::Single,
                source: AveragingSource::Return {
                    agi: 2500000,
                    deductions: 1500000,
                    tax: 100300,
                    other_tax: vec![],
                    loss: None,
                },
                dividends: 0,
                capital: AveragingCapital::NONE,
                foreign: None,
                investment: None,
            })
            .collect(),
        selection: FarmAveragingSelection {
            capital_elected: 0,
            unrecaptured_elected: 0,
            sources: vec![],
            history_scope: FarmAveragingHistoryScope::CompleteSupported,
        },
    });
    d
}

pub(in crate::tests) fn averaging_foreign(
    exclusion: i64,
    housing: i64,
    disallowed: i64,
) -> AveragingForeign {
    AveragingForeign {
        forms: vec![AveragingForeignForm {
            owner: Owner::Taxpayer,
            exclusion: exclusion * 100,
            housing: housing * 100,
        }],
        disallowed: disallowed * 100,
    }
}

pub(in crate::tests) fn qualified_dividend(amount: i64) -> FederalIncome {
    FederalIncome::Dividend {
        data: FederalDividend {
            section199a: ReitDividends::None,
            ..native_dividend(amount * 100, amount * 100, 0)
        },
    }
}

pub(in crate::tests) fn foreign_special_distribution(
    gain: i64,
    collectible: i64,
    unrecaptured: i64,
    dividends: i64,
) -> FederalIncome {
    FederalIncome::Dividend {
        data: FederalDividend {
            collectible_distributions: collectible * 100,
            unrecaptured_distributions: unrecaptured * 100,
            ..native_dividend(dividends * 100, dividends * 100, gain * 100)
        },
    }
}

// The section 904(j) payee-statement election: the foreign tax (cents) rides on
// a Form 1099-INT entry, which with the eligibility answer is the election's
// only source.
pub(in crate::tests) fn payee_statement_credit(d: &mut FederalDraft, tax: i64) {
    add(
        d,
        income(FederalIncome::Interest {
            data: FederalInterest { foreign_tax: tax, ..Default::default() },
        }),
    );
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Statements { eligible: Some(true) };
}
// Raises the tax of the entry `payee_statement_credit` added.
pub(in crate::tests) fn raise_payee_statement_tax(d: &mut FederalDraft, extra: i64) {
    let Some(FederalIncome::Interest { data }) = d.income.last_mut().map(|e| &mut e.income) else {
        unreachable!("the payee statement is the last entry")
    };
    data.foreign_tax += extra;
}

pub(in crate::tests) fn averaging_statement_credit(d: &mut FederalDraft) {
    add(d, income(FederalIncome::Interest { data: native_interest(100000, 0) }));
    payee_statement_credit(d, 30000);
}

// Actual NRCS-plan and original-election completion facts for existing dollar examples.
pub(in crate::tests) fn conservation_facts() -> FarmConservationFacts {
    FarmConservationFacts {
        plan: FarmConservationPlan::Nrcs,
        work: FarmConservationWork::QualifyingLandWork,
        land_farmed: true,
        plan_reference: Some("Synthetic retained NRCS plan".into()),
    }
}
pub(in crate::tests) fn conservation_method() -> FarmConservationMethod {
    FarmConservationMethod::OriginalElection { year: 2025 }
}
// Synthetic source completion, outside production: retain one real parcel with
// an actual counter identity before attaching an older arithmetic fixture's costs.
pub(in crate::tests) fn fixture_conservation_parcel(farm: &mut FarmIncome, next_id: &mut i64) {
    let Some(conservation) = &mut farm.conservation else {
        return;
    };
    if farm.land_basis.is_empty() {
        farm.land_basis.push(FarmLandBasis {
            id: *next_id,
            label: "Conservation fixture parcel".into(),
            acquisition: FarmLandAcquisition::Purchased,
            acquired: 20100101,
            regular: 0,
            amt: 0,
            prior_conservation: 0,
            prior_amt_conservation: 0,
            prior_cost_sharing: vec![],
        });
        *next_id += 1;
    }
    let id = farm.land_basis[0].id;
    match conservation {
        FarmConservation::Deduct { costs, assessments, carry, .. } => {
            for cost in costs {
                if cost.property == 0 {
                    cost.property = id;
                }
            }
            for record in assessments {
                if record.property == 0 {
                    record.property = id;
                }
            }
            for record in carry {
                if record.property == 0 {
                    record.property = id;
                }
            }
        }
        FarmConservation::Capitalize { costs, .. } => {
            for cost in costs {
                if cost.property == 0 {
                    cost.property = id;
                }
            }
        }
    }
}
pub(in crate::tests) fn conservation_return() -> FederalDraft {
    let mut d = averaging_return(60000);
    d.screening.as_mut().unwrap().farm_averaging = None;
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = Some(FarmConservation::Deduct {
        assessments: vec![],
        costs: vec![ConservationCost {
            property: 0,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "NRCS terrace work".into(),
            amount: 4000000,
        }],
        carry: vec![FarmConservationCarry {
            property: 0,
            year: 2025,
            regular: 500000,
            amt: 500000,
        }],
        method: conservation_method(),
    });
    fixture_conservation_parcel(data, &mut d.next_id);
    d
}

pub(in crate::tests) fn district_assessment(
    total: i64,
    share: i64,
    year: u32,
    paid: i64,
) -> ConservationAssessment {
    ConservationAssessment {
        property: 3,
        facts: conservation_facts(),
        label: "District irrigation equipment".into(),
        total: total * 100,
        share: share * 100,
        payments: vec![ConservationAssessmentPayment {
            current_date: None,
            year,
            amount: paid * 100,
        }],
    }
}

pub(in crate::tests) fn conservation_final_return() -> FederalDraft {
    let mut d = conservation_return();
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 0 });
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = Some(FarmConservation::Deduct {
        costs: vec![],
        assessments: vec![district_assessment(7000, 900, 2025, 900)],
        carry: vec![],
        method: conservation_method(),
    });
    d
}

pub(in crate::tests) fn farm_land_sale(
    acquired: u32,
    sold: u32,
) -> crate::farm_land::ResolvedFarmLandSale {
    crate::farm_land::ResolvedFarmLandSale {
        cost_sharing_recapture: 0,
        label: "Synthetic farm parcel".into(),
        deferral: false,
        acquisition: FarmLandAcquisition::Purchased,
        amt: FarmLandAmt::Same,
        acquired,
        sold,
        proceeds: 15000000,
        selling: 1000000,
        basis: 10000000,
        conservation: 2000000,
        eligible: true,
        payment: crate::SalePayment::InFull,
    }
}

// Explicit synthetic fixture construction: accepted parcel and sale identities
// share the real return counter; no production normalization or hidden history.
pub(in crate::tests) fn set_land_sales(
    farm: &mut FarmIncome,
    sales: Vec<crate::farm_land::ResolvedFarmLandSale>,
    next_id: &mut i64,
) {
    farm.land_basis.clear();
    farm.land_sales.clear();
    for source in sales {
        let property = *next_id;
        *next_id += 1;
        let id = *next_id;
        *next_id += 1;
        let (amt, prior_amt_conservation) = match source.amt {
            FarmLandAmt::Same => (source.basis, source.conservation),
            FarmLandAmt::Different { basis, conservation } => (basis, conservation),
        };
        farm.land_basis.push(FarmLandBasis {
            id: property,
            label: source.label.clone(),
            acquisition: source.acquisition,
            acquired: source.acquired,
            regular: source.basis,
            amt,
            prior_conservation: source.conservation,
            prior_amt_conservation,
            prior_cost_sharing: vec![],
        });
        assert_eq!(source.cost_sharing_recapture, 0, "Use explicit actual history for1255 fixture");
        farm.land_sales.push(FarmLandSale {
            id,
            basis_property: property,
            label: source.label,
            sold: source.sold,
            proceeds: source.proceeds,
            selling: source.selling,
            deferral: source.deferral,
            situation: FarmLandSituation::Direct,
            payment: source.payment,
        });
    }
}
pub(in crate::tests) fn set_farmland_sales(
    d: &mut FederalDraft,
    sales: Vec<crate::farm_land::ResolvedFarmLandSale>,
) {
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    set_land_sales(data, sales, &mut d.next_id);
}
pub(in crate::tests) fn farmland_resolved_sale(
    d: &FederalDraft,
    index: usize,
) -> crate::farm_land::ResolvedFarmLandSale {
    let FederalIncome::Business { data } = &d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &data.activity else { unreachable!() };
    data.land_sales[index].resolve(d, d.income[1].id, data).unwrap()
}
pub(in crate::tests) fn farmland_basis_mut(d: &mut FederalDraft) -> &mut FarmLandBasis {
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    &mut data.land_basis[0]
}

pub(in crate::tests) fn farmland_sales_mut(
    d: &mut FederalDraft,
) -> &mut alloc::vec::Vec<FarmLandSale> {
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    &mut data.land_sales
}

pub(in crate::tests) fn farmland_installment_return() -> FederalDraft {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    let mut sale = farm_land_sale(20160616, 20260616);
    sale.deferral = true;
    set_land_sales(data, vec![sale], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    d
}

pub(in crate::tests) fn farmland_statement_credit(d: &mut FederalDraft) {
    add(d, income(FederalIncome::Interest { data: native_interest(100000, 0) }));
    payee_statement_credit(d, 30000);
}

pub(in crate::tests) fn farmland_marketplace_repayment(d: &mut FederalDraft) {
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(d, (1..=12).map(|m| marketplace_month(m, 500, 100, 500)).collect());
}

pub(in crate::tests) fn farmland_installment_low_tax_credit(carry: i64) -> FederalDraft {
    let mut d = farmland_installment_return();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let sale = farmland_resolved_sale(&d, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    d.income[1].income = raised_farm(0, 0);
    // Two genuinely distinct parcel/sale sources with the same synthetic
    // dollars. Rebuilding the farm must retain each canonical basis record;
    // cloning an accepted sale ID loses its parcel and duplicates identity.
    set_farmland_sales(&mut d, vec![sale.clone(), sale]);
    let mut history = minimum_tax_history();
    history.amt = 0;
    history.carry = carry * 100;
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(history));
    d
}

pub(in crate::tests) fn farm_supply_advance(dollars: i64) -> FarmSupplyAdvance {
    FarmSupplyAdvance { amount: dollars * 100, purchase: farm_supply_purchase_source() }
}
pub(in crate::tests) fn farm_supply_purchase_source() -> FarmSupplyPurchase {
    FarmSupplyPurchase {
        kind: FarmSupplyPaymentKind::BindingPurchase,
        quantity_price_terms: "Fixed quantity at invoice price; binding delivery, no refund right"
            .into(),
        business_purpose: "Secure next season production supplies".into(),
        material_distortion: false,
        refunded: 0,
        paid: 20261201,
    }
}
pub(in crate::tests) fn farm_supplies(dollars: i64) -> FarmSupplies {
    FarmSupplies {
        production_current: vec![],
        production_carry: vec![],
        poultry: vec![],
        advances: vec![farm_supply_advance(dollars)],
        retained: vec![],
        exception: FarmSupplyException::None,
    }
}
pub(in crate::tests) fn farm_supply_return(
    supplies: FarmSupplies,
    other: i64,
    receipts: i64,
) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(receipts, other);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.supplies = Some(supplies);
    add(&mut d, income(entry));
    d
}
pub(in crate::tests) fn farm_supply_mut(d: &mut FederalDraft) -> &mut FarmSupplies {
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.supplies.as_mut().unwrap()
}
pub(in crate::tests) fn farm_supply_fifty_thousand(d: &FederalDraft) -> FederalResult {
    let r = supported(d);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(return_line(&r, "10"), 353300);
    assert_eq!(r.tax, Some(973000));
    assert_eq!(r.balance, Some(973000));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    r
}

pub(in crate::tests) fn poultry_resale(
    year: u32,
    cost: i64,
    deducted: i64,
    disposed: bool,
) -> FarmPoultryResale {
    FarmPoultryResale {
        method: FarmPoultryMethod::Paid,
        label: "Invoice poultry lot".into(),
        year,
        cost: cost * 100,
        deducted: deducted * 100,
        disposed,
    }
}

pub(in crate::tests) fn production_poultry_carry(
    cost: i64,
    deducted: i64,
) -> FarmProductionPoultryCarry {
    FarmProductionPoultryCarry {
        label: "2025 laying hens".into(),
        cost: cost * 100,
        deducted: deducted * 100,
    }
}

pub(in crate::tests) fn add_supply_farm(
    d: &mut FederalDraft,
    supplies: FarmSupplies,
    other: i64,
    receipts: i64,
) {
    let mut second = farm_supply_return(supplies, other, receipts);
    add(d, income(second.income.remove(0).income));
}

// Neutral completion facts for synthetic direct-farm fixtures, not IRS facts.
pub(in crate::tests) fn farm_header() -> FarmHeader {
    FarmHeader { code: 111900, ein: None, information_returns: FarmInformationReturns::NotRequired }
}

// Neutral original-method facts for synthetic ordinary cash-farm fixtures.
pub(in crate::tests) fn farm_specials() -> FarmSpecials {
    FarmSpecials {
        cost_sharing: vec![],
        weather_sales: vec![],
        prior_weather: vec![],
        other_income: vec![],
        production: vec![],
        fertilizer: vec![],
        external_sources: vec![],
        method_status: FarmMethodStatus {
            original: FarmOriginalMethod::Cash,
            adopted: 2000,
            change: None,
        },
        limitations: vec![],
        other_situation: None,
    }
}

pub(in crate::tests) fn native_cash_fixture() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 3_000_000, 0)];
    d.next_id = 2;
    d.payments.as_mut().unwrap().estimated = Some(10_000);
    d
}

pub(in crate::tests) use crate::tests::cross::interview::{cash_farm, farm_only_return};
pub(crate) fn farm_land(label: alloc::string::String) -> FarmLandBasis {
    FarmLandBasis {
        id: 0,
        label,
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20260101,
        regular: 100_000,
        amt: 100_000,
        prior_conservation: 0,
        prior_amt_conservation: 0,
        prior_cost_sharing: vec![],
    }
}

// A cash farm rental with the ordinary page defaults.
pub(in crate::tests) fn farm_rental_data(production: i64, program: i64) -> FederalFarmRental {
    let mut receipts = Vec::new();
    if production != 0 {
        receipts.push(FarmReceipt::Raised { amount: production * 100 });
    }
    if program != 0 {
        receipts.push(FarmReceipt::Program { amount: program * 100 });
    }
    FederalFarmRental {
        farm: FarmIncome {
            specials: farm_specials(),
            land_basis: vec![],
            header: None,
            accounting: FarmAccounting::Cash { receipts },
            insurance: FarmInsurance::None,
            ccc: None,
            crp: vec![],
            supplies: None,
            conservation: None,
            land_sales: vec![],
        },
        expenses: vec![],
        vehicles: vec![],
        assets: vec![],
        active_participation: true,
        at_risk: Some(crate::RentalAtRisk::All),
        history: RentalHistory::NONE,
        qbi_business: RentalBusiness::No,
        qbi_aggregation_group: None,
        payroll: BusinessPayroll::No,
    }
}
