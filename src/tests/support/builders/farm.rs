use crate::tests::*;

pub(in crate::tests) fn farm_business(
    receipts: alloc::vec::Vec<FarmReceipt>,
    expenses: i64,
) -> FederalIncome {
    let mut v = cash_business(0);
    let FederalIncome::Business { data } = &mut v else { unreachable!() };
    data.activity = BusinessActivity::Farming {
        data: FarmIncome {
            related: vec![],
            specials: farm_specials(),
            land_basis: vec![],
            header: Some(farm_header()),
            supplies: None,
            land_sales: vec![],
            conservation: None,
            crp: vec![],
            ccc: None,
            insurance_papers: Default::default(),
            insurance: FarmInsuranceRecords::None,
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
pub(in crate::tests) fn install_crop_records(data: &mut FarmIncome, insurance: FarmInsurance) {
    use crate::farm_insurance_paper::CropInsuranceFacts;
    match insurance {
        FarmInsurance::None => {
            data.insurance = FarmInsuranceRecords::None;
            data.insurance_papers = CropInsuranceFacts::default();
        }
        FarmInsurance::Payments { current, previous_deferred, election, withholding } => {
            data.insurance =
                FarmInsuranceRecords::Payments { current, previous_deferred, withholding };
            data.insurance_papers = match election {
                CropDeferral::Include => CropInsuranceFacts {
                    timing: Some(CropPaperTiming::Include),
                    ..Default::default()
                },
                CropDeferral::Defer { practice } => CropInsuranceFacts {
                    timing: Some(CropPaperTiming::Defer),
                    practice,
                    ..Default::default()
                },
            };
        }
    }
}
pub(in crate::tests) fn answer_crop_timing(
    session: &mut Interview,
    timing: CropPaperTiming,
) -> Reply {
    let activity = crate::farm::sources(&session.draft).next().unwrap().entry.id;
    let request = P::SaveCropInsuranceTiming {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        activity,
        timing,
    };
    send(session, request)
}
pub(in crate::tests) fn answer_crop_practice(
    session: &mut Interview,
    crop: &str,
    following: i64,
) -> Reply {
    let activity = crate::farm::sources(&session.draft).next().unwrap().entry.id;
    let opened = {
        let request = P::OpenCropPractice {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            activity,
            crop: crop.into(),
        };
        send(session, request)
    };
    assert!(matches!(opened, Reply::Response(P::BeginCropPractice { .. })), "{opened:?}");
    {
        let request = P::SaveCropPractice {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            activity,
            crop: crop.into(),
            normal_total: 1_000_000,
            normal_following: following,
        };
        send(session, request)
    }
}
pub(in crate::tests) fn crop_farm(insurance: FarmInsurance) -> FederalIncome {
    let mut entry = raised_farm(30000, 10000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    install_crop_records(data, insurance);
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
        data: dividend_source(FederalDividend {
            section199a: ReitDividends::None,
            ..native_dividend(amount * 100, amount * 100, 0)
        }),
    }
}

pub(in crate::tests) fn foreign_special_distribution(
    gain: i64,
    collectible: i64,
    unrecaptured: i64,
    dividends: i64,
) -> FederalIncome {
    FederalIncome::Dividend {
        data: dividend_source(FederalDividend {
            collectible_distributions: collectible * 100,
            unrecaptured_distributions: unrecaptured * 100,
            ..native_dividend(dividends * 100, dividends * 100, gain * 100)
        }),
    }
}

// The section 904(j) payee-statement election: the foreign tax (cents) rides on
// a Form 1099-INT entry, which with the eligibility answer is the election's
// only source.
pub(in crate::tests) fn payee_statement_credit(d: &mut FederalDraft, tax: i64) {
    add(
        d,
        income(FederalIncome::Interest {
            data: interest_source(FederalInterest { foreign_tax: tax, ..Default::default() }),
        }),
    );
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Statements { eligible: Some(true) };
}
// Raises the tax of the entry `payee_statement_credit` added.
pub(in crate::tests) fn raise_payee_statement_tax(d: &mut FederalDraft, extra: i64) {
    let Some(FederalIncome::Interest { data }) = d.income.last_mut().map(|e| &mut e.income) else {
        unreachable!("the payee statement is the last entry")
    };
    edit_interest(data).foreign_tax += extra;
}

pub(in crate::tests) fn averaging_statement_credit(d: &mut FederalDraft) {
    add(d, income(FederalIncome::Interest { data: interest_source(native_interest(100000, 0)) }));
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
            farm_rental_use: None,
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
    add(d, income(FederalIncome::Interest { data: interest_source(native_interest(100000, 0)) }));
    payee_statement_credit(d, 30000);
}

pub(in crate::tests) fn farmland_marketplace_repayment(d: &mut FederalDraft) {
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(d, (1..=12).map(|m| marketplace_month(m, 500, 100, 500)).collect());
}

pub(in crate::tests) fn farmland_installment_low_tax_credit(carry: i64) -> FederalDraft {
    let mut d = farmland_installment_return();
    d.deductions.as_mut().unwrap().itemize_override = None;
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
    d.payments.as_mut().unwrap().estimated =
        crate::tests::support::builders::unknown_estimated(10_000, PaymentPayer::Taxpayer);
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
        improvement_buildings: Vec::new(),
        interest_limit_election: None,
        farm: FarmIncome {
            related: vec![],
            specials: farm_specials(),
            land_basis: vec![],
            header: None,
            accounting: FarmAccounting::Cash { receipts },
            insurance_papers: Default::default(),
            insurance: FarmInsuranceRecords::None,
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

// Derived partial-sale records under Publication 225, Sale of a Farm.
pub(in crate::tests) fn partial_land_allocation(amounts: [i64; 4]) -> FarmLandSituation {
    let [regular, amt, conservation, amt_conservation] =
        amounts.map(|amount| SourceCents::Reported { amount });
    FarmLandSituation::PartialParcel {
        allocation: FarmLandAllocation { regular, amt, conservation, amt_conservation },
    }
}

pub(in crate::tests) fn partial_land_parcel() -> FarmLandBasis {
    FarmLandBasis {
        id: 0,
        label: "East and west farm tracts".into(),
        acquisition: FarmLandAcquisition::Purchased,
        acquired: 20220101,
        regular: 10_000_000,
        amt: 9_000_000,
        prior_conservation: 2_000_000,
        prior_amt_conservation: 1_500_000,
        prior_cost_sharing: vec![],
    }
}

// Fully recovered property has established zero remaining basis.
pub(in crate::tests) fn farmer_gross_asset(
    gain: i64,
    livestock: crate::Livestock,
) -> crate::BusinessAsset {
    let mut asset = if gain < 0 {
        depreciable_asset(-gain, EquipmentClass::Five, 20260101, 0)
    } else {
        continuing_asset(100_000, 20180101, EquipmentClass::Five, 100_000, crate::AssetDisposal::No)
    };
    asset.disposal = crate::AssetDisposal::Disposed {
        date: 20260601,
        how: crate::DisposalKind::Sold {
            proceeds: gain.max(0) * 100,
            selling_expenses: 0,
            livestock: livestock.into(),
            buyer: crate::Buyer::Unrelated,
            payment: crate::SalePayment::InFull,
        },
    };
    asset
}

pub(in crate::tests) fn farmer_gross_return(farming: i64, wages: i64) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    draft.next_id = 2;
    add(&mut draft, income(raised_farm(farming, farming)));
    draft.screening.as_mut().unwrap().section1231 =
        Some(crate::Section1231History { regular: vec![], amt: vec![] });
    draft
}

pub(in crate::tests) fn farmer_gross_earlier_property() -> EarlierInstallment {
    EarlierInstallment {
        owner: Owner::Taxpayer,
        sale: InstallmentPayments {
            property: "Retained business-property source".into(),
            sold: 20230601,
            gross_profit: 600_000,
            contract_price: 1_000_000,
            earlier_payments: 100_000,
            payments: 500_000,
            kind: crate::PropertyCharacter::BusinessProperty {
                used: crate::PropertyUse::Business,
                unrecaptured: 0,
            },
            special: false,
            qbi: InstallmentQbi::NotQualified,
            livestock: None,
        },
    }
}

pub(in crate::tests) fn farmer_gross_rental_royalty_return() -> FederalDraft {
    let mut draft = farmer_gross_return(60_000, 20_000);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.expenses.clear();
    add(&mut draft, income(FederalIncome::Rental { data: rental(30_000, 40_000) }));
    add(&mut draft, income(royalty(30_000, 40_000)));
    add(&mut draft, income(personal_rent(10_000, 20_000)));
    draft.payments.as_mut().unwrap().prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 10_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    draft
}

// Retained real-property exchange; section 1031 has excluded livestock since 2018.
pub(in crate::tests) fn farmer_gross_earlier_exchange() -> crate::EarlierRelatedExchange {
    crate::EarlierRelatedExchange {
        owner: Owner::Taxpayer,
        exchange: crate::RelatedExchange {
            property: "Retained business parcel".into(),
            last_transfer: 20250501,
            disposed: 20260430,
            exception: crate::RelatedException::None,
            deferred: 300_000,
            kind: crate::PropertyCharacter::BusinessProperty {
                used: crate::PropertyUse::Business,
                unrecaptured: 0,
            },
            special: false,
        },
    }
}

pub(in crate::tests) fn farmer_gross_qof_source() -> FederalIncome {
    FederalIncome::OpportunityFund {
        data: crate::QofInvestment {
            fund_ein: Default::default(),
            source_recipient: Default::default(),
            fund: "Synthetic retained original-property gain".into(),
            invested_on: 20230601,
            short_term: Some(0),
            long_term: Some(300_000),
            gain_code: crate::QofGainCode::Section1231,
            event: Some(crate::QofEvent::Held { fair_value: Some(300_000) }),
            special: false,
        },
    }
}

// Explicit prior-return facts request the penalty computation; the ordinary
// gross-only fixture deliberately leaves that separate worksheet unrequested.
pub(in crate::tests) fn farmer_gross_penalty_return(farming: i64, wages: i64) -> FederalDraft {
    let mut draft = farmer_gross_return(farming, wages);
    draft.payments.as_mut().unwrap().prior_year = Some(crate::PriorYearTax {
        full_year: true,
        tax: 1_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    draft
}

pub(in crate::tests) fn relate_farm_component(
    draft: &mut FederalDraft,
    activity: i64,
    component: i64,
) {
    let entry = draft.income.iter_mut().find(|entry| entry.id == activity).unwrap();
    let farm = match &mut entry.income {
        FederalIncome::Business { data } => {
            let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
            data
        }
        FederalIncome::FarmRental { data } => &mut data.farm,
        _ => unreachable!(),
    };
    farm.related.push(component);
}

pub(in crate::tests) fn farm_government_component(draft: &FederalDraft) -> i64 {
    draft
        .income
        .iter()
        .rev()
        .find_map(|entry| match &entry.income {
            FederalIncome::GovernmentPayment { data }
                if data.form.agriculture_payments.unwrap_or(0) != 0 =>
            {
                Some(data.component_ids.agriculture)
            }
            _ => None,
        })
        .unwrap()
}

// Independent annual penalty operands; source availability is supplied by each case.
pub(in crate::tests) fn farmland_underpayment_inputs(
    farmland_applicable: crate::kernel::source_fact::SourceFact<i64>,
) -> crate::underpayment::Inputs {
    crate::underpayment::Inputs {
        line_1: 10_000,
        line_2: 0,
        household_tax: 0,
        line_3: 125,
        withholding: 0,
        excess_social: 0,
        estate_estimated: crate::kernel::ratio::exact::whole(0),
        separate: false,
        farming_return_year: Ok(false),
        farmland_applicable,
        fund_credit: false,
        status: FederalStatus::Single,
        agi: 100_000,
        taxable: 83_900,
        standard: 16_100,
        itemizing: false,
        credits: 0,
        child_credit: 0,
        child_credit_by_column: [0; 4],
        se_tax: [0; 2],
        ordinary_tax: true,
        other_taxes: false,
    }
}

// An actual selected sale remains in the source graph when its activity's
// optional scope is omitted. The second case retains another elected sale.
pub(in crate::tests) fn farmland_omitted_penalty_return(retain_other: bool) -> FederalDraft {
    let mut d = farmland_installment_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.special_treatment = true;
    if retain_other {
        let mut retained = income(raised_farm(30_000, 30_000));
        let FederalIncome::Business { data } = &mut retained.income else { unreachable!() };
        let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
        let mut sale = farm_land_sale(20160616, 20260616);
        sale.deferral = true;
        set_land_sales(data, vec![sale], &mut d.next_id);
        add(&mut d, retained);
    }
    d.payments.as_mut().unwrap().prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 2_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    d
}

// Synthetic completion records for the independently stated section175 gross examples.
pub(in crate::tests) fn conservation_livestock_return(
    crops: i64,
    gain: i64,
    purpose: Option<crate::LivestockPurpose>,
) -> FederalDraft {
    let mut draft = conservation_return();
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    farm.accounting =
        FarmAccounting::Cash { receipts: vec![FarmReceipt::Raised { amount: crops * 100 }] };
    data.expenses.clear();
    let mut asset = farmer_gross_asset(gain, crate::Livestock::CattleOrHorse);
    asset.id = draft.next_id;
    draft.next_id += 1;
    let crate::AssetDisposal::Disposed { how: crate::DisposalKind::Sold { livestock, .. }, .. } =
        &mut asset.disposal
    else {
        unreachable!()
    };
    *livestock = crate::SoldLivestock::CattleOrHorse { purpose };
    data.assets = vec![asset];
    draft.screening.as_mut().unwrap().section1231 =
        Some(crate::Section1231History { regular: vec![], amt: vec![] });
    draft
}

// Complete synthetic crop-share use records; no eligibility is inferred from the form.
pub(in crate::tests) fn farm_rental_property_use() -> crate::FarmRentalPropertyUse {
    crate::FarmRentalPropertyUse::OnlyThisRental {
        tenant_connection: Some(crate::FarmRentalTenantConnection::NoOwnershipOrServices),
        value_enhancement: Some(crate::FarmRentalValueEnhancement::NoParticipation),
    }
}
pub(in crate::tests) fn farm_rental_passive_property_return() -> FederalDraft {
    farm_rental_passive_property_fixture(true)
}
// The ordinary Farming asset editor supports the same-AMT source projection;
// the distinct native financial fixture above retains separate AMT schedules.
pub(in crate::tests) fn farm_rental_passive_property_editor_return() -> FederalDraft {
    farm_rental_passive_property_fixture(false)
}
fn farm_rental_passive_property_fixture(separate_depreciation: bool) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 13_000_000, 0));
    let mut data = farm_rental_data(8_000, 0);
    data.active_participation = false;
    // A purchased pre-September-2017 machine first placed in service in 2023:
    // five-year, half-year MACRS, no section179/bonus. Original annual schedules
    // retain REG 200% DB prior $27,704.28 and AMT 150% DB prior $22,704.29.
    // In 2026 both switch to SL over the remaining 2.5 years: REG $4,482.49,
    // AMT $6,482.49. Actual repairs $17,517.51 plus those native deductions
    // give total expenses $22,000/$24,000 against $8,000 production shares.
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Repairs,
        if separate_depreciation { 1_751_751 } else { 2_200_000 },
    )];
    if separate_depreciation {
        let mut machine = bonus_equipment(1, 100);
        machine.id = d.next_id;
        d.next_id += 1;
        machine.cost = 3_891_051;
        machine.acquired = 20160901;
        machine.placed = 20230101;
        machine.history = EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation: 2_770_428,
            amt: EquipmentAmtHistory::Separate {
                prior_depreciation: 2_270_429,
                schedule: EquipmentSchedule::Manual,
            },
            mid_quarter: false,
            schedule: EquipmentSchedule::Manual,
        };
        data.assets.push(machine);
    }
    let mut sale = farm_land_sale(20160616, 20260616);
    sale.proceeds = 11_000_000;
    sale.selling = 0;
    sale.conservation = 0;
    sale.amt = FarmLandAmt::Different { basis: 9_800_000, conservation: 0 };
    set_land_sales(&mut data.farm, vec![sale], &mut d.next_id);
    data.farm.land_sales[0].farm_rental_use = Some(farm_rental_property_use());
    add(&mut d, income(FederalIncome::FarmRental { data }));
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    d
}

pub(in crate::tests) fn farm_rental_passive_asset_return() -> FederalDraft {
    let mut d = farm_rental_passive_property_editor_return();
    let FederalIncome::FarmRental { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.farm.land_sales.clear();
    let mut asset = bonus_equipment(10_000, 100);
    asset.id = d.next_id;
    d.next_id += 1;
    asset.acquired = 20150101;
    asset.placed = 20150101;
    asset.method = EquipmentMethod::Declining150;
    asset.history = EquipmentHistory::Continuing {
        initial_section179: 0,
        carryforward: 0,
        prior_depreciation: 1_000_000,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    asset.disposal = AssetDisposal::Disposed {
        date: 20260616,
        how: DisposalKind::Sold {
            proceeds: 1_000_000,
            selling_expenses: 0,
            payment: SalePayment::InFull,
            buyer: Buyer::Unrelated,
            livestock: crate::SoldLivestock::No,
        },
    };
    asset.farm_rental_use = Some(farm_rental_property_use());
    data.assets.push(asset);
    d
}
