use crate::tests::*;
pub(in crate::tests) fn foreign_pay(wages: i64) -> FederalIncome {
    FederalIncome::ForeignEmployment { data: ForeignWages { wages: wages * 100 } }
}
// A bona fide residence that began in 2020 and continues.
pub(in crate::tests) fn residence() -> ForeignQualification {
    ForeignQualification::Residence {
        began: 20200101,
        ended: None,
        basis: crate::ResidenceBasis::Citizen,
        nonresident_statement: false,
    }
}
// A Form 2555 record electing the exclusion, with no housing.
pub(in crate::tests) fn foreign_claim(owner: Owner) -> ForeignEarnedClaim {
    ForeignEarnedClaim {
        owner,
        qualification: residence(),
        tax_home: true,
        revocation: crate::ExclusionRevocation::None,
        prior_year_pay: 0,
        prior_year_history: None,
        multi_year_pay: vec![],
        exclusion: true,
        housing: ForeignHousing::None,
    }
}
// The person's Form 2555 record, replacing any earlier one.
pub(in crate::tests) fn set_claim(d: &mut FederalDraft, claim: ForeignEarnedClaim) {
    let screening = d.screening.as_mut().unwrap();
    screening.foreign_earned.retain(|c| c.owner != claim.owner);
    screening.foreign_earned.push(claim);
}
// The record of this person, for a test to change.
pub(in crate::tests) fn claim_mut(d: &mut FederalDraft, owner: Owner) -> &mut ForeignEarnedClaim {
    d.screening
        .as_mut()
        .unwrap()
        .foreign_earned
        .iter_mut()
        .find(|c| c.owner == owner)
        .expect("a Form 2555 record")
}

pub(in crate::tests) fn foreign_high_tax_return(tax: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10000000, 0);
    add(&mut d, income(foreign_pay(60000)));
    add(&mut d, income(savings_income(40000, 0)));
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        sources: vec![
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Country A employer".into(),
                category: ForeignTaxCategory::Employment,
                income: 6000000,
                tax: 0,
            },
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Country B bank".into(),
                category: ForeignTaxCategory::Passive {
                    sales: vec![],
                    withholding: ForeignWithholding::AtLeast15,
                    qualified: 0,
                },
                income: 4000000,
                tax: tax * 100,
            },
        ],
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    d
}

pub(in crate::tests) fn set_foreign_amt_election(d: &mut FederalDraft, year: u32, confirmed: bool) {
    if let ForeignTaxClaim::Ordinary { amt_limitation, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *amt_limitation = ForeignAmtLimitation::Simplified { first_claim_year: year, confirmed };
    } else {
        panic!("ordinary foreign credit fixture");
    }
}

pub(in crate::tests) fn wholly_foreign_credit_return(
    status: FederalStatus,
    wages: i64,
) -> FederalDraft {
    let mut d = complete(status);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income.clear();
    add(&mut d, income(foreign_pay(wages)));
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign employer".into(),
            category: ForeignTaxCategory::Employment,
            income: wages * 100,
            tax: 40000000,
        }],
        amt_limitation: ForeignAmtLimitation::Normal,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    d
}

// One column of a retained Schedule B (Form 1116): the shape the foreign credit
// tests were written in, turned into the schedules the claim stores.
#[derive(Clone, Debug)]
pub(in crate::tests) struct ForeignCreditBalance {
    pub(in crate::tests) year: u32,
    pub(in crate::tests) category: ForeignCreditCategory,
    pub(in crate::tests) regular: i64,
    pub(in crate::tests) amt: i64,
}
pub(in crate::tests) fn foreign_balance(
    year: u32,
    category: ForeignCreditCategory,
    regular: i64,
    amt: i64,
) -> ForeignCreditBalance {
    ForeignCreditBalance { year, category, regular: regular * 100, amt: amt * 100 }
}
pub(in crate::tests) fn foreign_history_of(
    balances: &[ForeignCreditBalance],
) -> ForeignCreditHistory {
    let opening = |carryover| ForeignCarryoverOpening {
        carryover,
        carryback_adjustment: 0,
        redetermination: 0,
        other_adjustments: 0,
    };
    let mut schedules: alloc::vec::Vec<ForeignCarryoverSchedule> = alloc::vec::Vec::new();
    for balance in balances {
        let column = ForeignCarryoverColumn {
            year: balance.year,
            regular: opening(balance.regular),
            amt: opening(balance.amt),
        };
        match schedules.iter_mut().find(|s| s.category == balance.category) {
            Some(schedule) => schedule.columns.push(column),
            None => schedules.push(ForeignCarryoverSchedule {
                category: balance.category,
                columns: vec![column],
            }),
        }
    }
    ForeignCreditHistory::Retained(ForeignCarryoverRecords { schedules, same_filing_unit: true })
}
pub(in crate::tests) fn set_foreign_history(
    d: &mut FederalDraft,
    balances: alloc::vec::Vec<ForeignCreditBalance>,
) {
    if let ForeignTaxClaim::Ordinary { history, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *history = foreign_history_of(&balances);
    } else {
        panic!("ordinary foreign-credit fixture");
    }
}

pub(in crate::tests) fn set_foreign_carryback(d: &mut FederalDraft, value: ForeignCarryback) {
    if let ForeignTaxClaim::Ordinary { carryback, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *carryback = value;
    } else {
        panic!("ordinary foreign-credit fixture");
    }
}
pub(in crate::tests) fn prior_foreign_credit(
    category: ForeignCreditCategory,
    regular: [i64; 2],
    amt: [i64; 2],
) -> ForeignPriorCredit {
    ForeignPriorCredit {
        category,
        regular: ForeignPriorLimit { limit: regular[0] * 100, taxes: regular[1] * 100 },
        amt: ForeignPriorLimit { limit: amt[0] * 100, taxes: amt[1] * 100 },
    }
}

pub(in crate::tests) fn foreign_medical_itemizing(
    d: &mut FederalDraft,
    medical: i64,
    sales: i64,
    property: i64,
) {
    let ded = d.deductions.as_mut().unwrap();
    let facts = itemized(ded);
    facts.medical = medical * 100;
    facts.personal_tax = PersonalTaxElection::Sales { paid: sales * 100 };
    facts.personal_property_taxes = property * 100;
    ded.choice = DeductionSelection::Itemized { facts: facts.clone() };
}

pub(in crate::tests) fn foreign_mortgage(d: &mut FederalDraft, interest: i64) {
    let v = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(v, interest * 100);
}

pub(in crate::tests) fn foreign_state_payment(
    label: &str,
    paid: i64,
    domestic: i64,
    foreign: i64,
) -> ForeignStatePayment {
    ForeignStatePayment {
        label: label.into(),
        paid: paid * 100,
        domestic: domestic * 100,
        foreign: if foreign == 0 {
            vec![]
        } else {
            vec![ForeignStateIncome { source: "Foreign employer".into(), income: foreign * 100 }]
        },
    }
}
pub(in crate::tests) fn set_foreign_state(
    d: &mut FederalDraft,
    payments: alloc::vec::Vec<ForeignStatePayment>,
) {
    if let ForeignTaxClaim::Ordinary { state_taxes, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *state_taxes = ForeignStateTaxes::Records { payments, confirmed: true };
    } else {
        panic!("ordinary foreign fixture");
    }
}
pub(in crate::tests) fn foreign_state_return() -> FederalDraft {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 25000);
    add(&mut d, wage(3, Owner::Taxpayer, 7500000, 0));
    foreign_medical_itemizing(&mut d, 0, 0, 0);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 600000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    set_foreign_state(&mut d, vec![foreign_state_payment("State A", 6000, 75000, 15000)]);
    d
}

pub(in crate::tests) fn foreign_asset(
    label: &str,
    source: InvestmentAssetSource,
    beginning: i64,
    ending: i64,
) -> ForeignInvestmentAsset {
    ForeignInvestmentAsset {
        label: label.into(),
        source,
        basis: InvestmentAssetBasis::Average {
            beginning: beginning * 100,
            ending: ending * 100,
            amt_beginning: beginning * 100,
            amt_ending: ending * 100,
        },
    }
}
pub(in crate::tests) fn foreign_asset_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 9000000, 0);
    add(&mut d, income(savings_income(10000, 0)));
    foreign_medical_itemizing(&mut d, 0, 0, 0);
    investment_borrowing(&mut d, 2000, 0, 0);
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign deposits".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::None,
                qualified: 0,
            },
            income: 600000,
            tax: 60000,
        }],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::TaxBook {
            assets: vec![
                foreign_asset("Domestic deposits", InvestmentAssetSource::Domestic, 20000, 60000),
                foreign_asset(
                    "Foreign deposits",
                    InvestmentAssetSource::Foreign { source: "Foreign deposits".into() },
                    80000,
                    40000,
                ),
            ],
        },
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    d
}

pub(in crate::tests) fn daily_basis(date: u32, regular: i64, amt: i64) -> InvestmentBasisChange {
    InvestmentBasisChange { date, regular: regular * 100, amt: amt * 100 }
}

pub(in crate::tests) fn foreign_passive_loss_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 9810000, 0);
    add(&mut d, income(savings_income(1900, 0)));
    foreign_medical_itemizing(&mut d, 0, 0, 0);
    investment_borrowing(&mut d, 1600, 0, 0);
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        sources: vec![
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Other tax".into(),
                category: ForeignTaxCategory::Passive {
                    sales: vec![],
                    withholding: ForeignWithholding::None,
                    qualified: 0,
                },
                income: 40000,
                tax: 4000,
            },
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Low withholding".into(),
                category: ForeignTaxCategory::Passive {
                    sales: vec![],
                    withholding: ForeignWithholding::Below15,
                    qualified: 0,
                },
                income: 20000,
                tax: 1000,
            },
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "High withholding".into(),
                category: ForeignTaxCategory::Passive {
                    sales: vec![],
                    withholding: ForeignWithholding::AtLeast15,
                    qualified: 0,
                },
                income: 130000,
                tax: 32500,
            },
        ],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::TaxBook {
            assets: vec![
                foreign_asset(
                    "Other tax asset",
                    InvestmentAssetSource::Foreign { source: "Other tax".into() },
                    70000,
                    70000,
                ),
                foreign_asset(
                    "High withholding asset",
                    InvestmentAssetSource::Foreign { source: "High withholding".into() },
                    90000,
                    90000,
                ),
            ],
        },
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    d
}

pub(in crate::tests) fn foreign_loss_system(
    overall: i64,
    separate: i64,
    domestic: i64,
    extra: i64,
) -> ForeignLossSystem {
    ForeignLossSystem {
        overall: overall * 100,
        separate: separate * 100,
        domestic: domestic * 100,
        additional_recapture: extra * 100,
    }
}
pub(in crate::tests) fn foreign_loss_records(
    d: &mut FederalDraft,
    accounts: alloc::vec::Vec<ForeignLossAccount>,
) {
    if let ForeignTaxClaim::Ordinary { loss_history, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *loss_history = ForeignLossHistory::Retained { accounts, same_filing_unit: true };
    } else {
        panic!("ordinary credit required");
    }
}

// A housing claim for the whole of 2026 in one location.
pub(in crate::tests) fn housing_claim(
    expenses: i64,
    location: ForeignHousingLocation,
    prior: Option<HousingCarryover>,
) -> ForeignHousing {
    ForeignHousing::Claim {
        stays: vec![ForeignHousingStay { location, starts_on: 20260101, ends_on: 20261231 }],
        expenses: expenses * 100,
        nontaxable_allowances: 0,
        prior,
        household: ForeignHousingHousehold::OnlyClaim,
    }
}
// The taxpayer's Form 2555 gains a housing claim (the exclusion stays as it
// is; a record is created, excluding, when there is none).
pub(in crate::tests) fn employee_housing(
    d: &mut FederalDraft,
    expenses: i64,
    location: ForeignHousingLocation,
) {
    if d.screening.as_ref().unwrap().foreign_earned.iter().all(|c| c.owner != d.income[0].owner) {
        set_claim(d, foreign_claim(d.income[0].owner));
    }
    let owner = d.income[0].owner;
    claim_mut(d, owner).housing = housing_claim(expenses, location, None);
}

// A business run abroad (no Form 2555 record: nothing of it is excluded).
pub(in crate::tests) fn foreign_service_business(profit: i64) -> FederalIncome {
    let FederalIncome::Business { mut data } = cash_business(profit) else { unreachable!() };
    data.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
    FederalIncome::Business { data }
}

pub(in crate::tests) fn combined_foreign_wage_business(
    wages: i64,
    status: FederalStatus,
) -> FederalDraft {
    let mut d = empty_return(status);
    add(&mut d, income(foreign_pay(wages)));
    let FederalIncome::Business { mut data } = foreign_service_business(100000) else {
        unreachable!()
    };
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Service costs".into() },
        2500000,
    ));
    add(&mut d, income(FederalIncome::Business { data }));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    d
}

pub(in crate::tests) fn excluded_foreign_business(receipts: i64, expenses: i64) -> FederalIncome {
    let FederalIncome::Business { mut data } = foreign_service_business(receipts) else {
        unreachable!()
    };
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Foreign service costs".into() },
        expenses * 100,
    ));
    FederalIncome::Business { data }
}

// The taxpayer's self-employed housing claim: a Form 2555 record holding it
// (housing only when there is no record yet).
pub(in crate::tests) fn self_employed_housing(
    d: &mut FederalDraft,
    prior: Option<HousingCarryover>,
) {
    if d.screening.as_ref().unwrap().foreign_earned.iter().all(|c| c.owner != d.income[0].owner) {
        let mut claim = foreign_claim(d.income[0].owner);
        claim.exclusion = false;
        set_claim(d, claim);
    }
    let owner = d.income[0].owner;
    claim_mut(d, owner).housing = housing_claim(50000, ForeignHousingLocation::General, prior);
}

// Unit k3: a K-1's Schedule K-3 and its box 21 / box 16 code F foreign taxes.
// The page maps field for field onto the stored record and reopens unchanged;
// the K-1 import never reads the K-3 and the K-3 import never reads the K-1.
pub(in crate::tests) fn k3_every_field() -> ScheduleK3 {
    use crate::tests::foreign::foreign::{k3_answer, k3_blank, k3_row};
    let mut k3 = k3_blank();
    k3.countries = vec![
        k3_row("JA", crate::K3Category::Passive, 6_000, 1_000, 900),
        k3_row("UK", crate::K3Category::Passive, 1_000, 0, 0),
        k3_row("CA", crate::K3Category::General, 20_000, 0, 3_000),
    ];
    k3.withholding = vec![
        k3_answer("JA", ForeignWithholding::AtLeast15),
        k3_answer("UK", ForeignWithholding::None),
    ];
    k3.us_gross = 10_050_000;
    k3.passive = crate::K3CategoryAmounts {
        deductions: 100_000,
        interest: 20_000,
        reduction: 10_000,
        capital: Default::default(),
    };
    k3.general = crate::K3CategoryAmounts {
        deductions: 600_000,
        interest: 30_000,
        reduction: 0,
        capital: Default::default(),
    };
    k3.partner_deductions = 70_000;
    k3.partner_interest.business = 800_000;
    k3.partner_interest.passive_activity = 40_000;
    k3.partner_interest.business_assets =
        crate::K3Assets { us: 30_000_000, branch: 0, passive: 5_000_000, general: 10_000_000 };
    k3.partner_interest.passive_assets =
        crate::K3Assets { us: 1_000_000, branch: 0, passive: 2_000_000, general: 0 };
    k3.items = vec![
        crate::K3Item::OtherCategories,
        crate::K3Item::ForeignRateGroups,
        crate::K3Item::Research,
        crate::K3Item::OilAndGas,
        crate::K3Item::Loans,
        crate::K3Item::DisallowedDeduction,
        crate::K3Item::ForeignCorporations,
    ];
    k3
}

pub(in crate::tests) fn foreign_income_owner_fixture() -> FederalDraft {
    let mut d = injured_return((200_000, 100_000), (50_000, 0), 0, allocation());
    add(&mut d, income(savings_income(5000, 0)));
    let mut second = income(savings_income(15_000, 0));
    second.owner = Owner::Spouse;
    add(&mut d, second);
    d
}

pub(in crate::tests) fn add_foreign_owner_income(d: &mut FederalDraft, owner: Owner, wages: i64) {
    let mut entry = income(foreign_pay(wages));
    entry.owner = owner;
    add(d, entry);
}
