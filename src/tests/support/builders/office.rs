use crate::tests::*;

// Plain fixture data preserves the original worksheet inputs. Only the
// canonical HomeOfficeSite is stored; these types contain no tax calculation.
#[derive(Clone, Debug)]
pub(in crate::tests) enum HomeFixture {
    Owned {
        mortgage_interest: i64,
        mortgage_insurance: i64,
        real_estate_taxes: i64,
        basis: i64,
        land: i64,
        first_used: u32,
    },
    Rented {
        rent: i64,
    },
}

#[derive(Clone, Debug)]
pub(in crate::tests) struct OfficeFixture {
    pub office_area: u32,
    pub home_area: u32,
    pub home: HomeFixture,
    pub insurance: i64,
    pub repairs: i64,
    pub utilities: i64,
    pub other: i64,
    pub direct: i64,
    pub prior_operating: i64,
    pub prior_depreciation: i64,
    pub prior_casualty: i64,
}

pub(in crate::tests) fn actual_office(
    area: u32,
    home_area: u32,
    home: HomeFixture,
) -> OfficeFixture {
    OfficeFixture {
        office_area: area,
        home_area,
        home,
        insurance: 0,
        repairs: 0,
        utilities: 0,
        other: 0,
        direct: 0,
        prior_operating: 0,
        prior_depreciation: 0,
        prior_casualty: 0,
    }
}

pub(in crate::tests) fn owned_office_home(
    interest: i64,
    premiums: i64,
    taxes: i64,
    basis: i64,
    land: i64,
    first_used: u32,
) -> HomeFixture {
    HomeFixture::Owned {
        mortgage_interest: interest,
        mortgage_insurance: premiums,
        real_estate_taxes: taxes,
        basis,
        land,
        first_used,
    }
}

// 2026-10-17 synthetic source completion for the approved separate-carry
// questions: these records explicitly have $2,000 depreciation carry and no
// casualty carry. This is not a classification of an unknown legacy balance.
pub(in crate::tests) fn simplified_claim(area: u32) -> HomeOfficeClaim {
    HomeOfficeClaim {
        owner: Owner::Taxpayer,
        starts: 20260101,
        ends: 20261231,
        use_kind: OfficeQualifiedUse::PrincipalPlace,
        method: OfficeMethod::Simplified {
            months: (1..=12)
                .map(|month| OfficeSimplifiedMonth {
                    month,
                    exclusive_days: 20,
                    exclusive_area: area,
                    shared_days: 0,
                    shared_area: 0,
                })
                .collect(),
        },
        activities: vec![],
        prior_operating: 100_000,
        prior_depreciation: 200_000,
        prior_casualty: 0,
    }
}

pub(in crate::tests) fn add_office_claim(
    draft: &mut FederalDraft,
    source: i64,
    mut claim: HomeOfficeClaim,
    area: u32,
) -> i64 {
    let owner = draft.income.iter().find(|entry| entry.id == source).unwrap().owner;
    claim.owner = owner;
    claim.activities = vec![OfficeActivity {
        activity: source,
        gross_share: 10000,
        expense_period_share: 10000,
        attribution_reason: alloc::string::String::new(),
        deduction_parts: 1,
    }];
    let id = draft.next_id;
    draft.next_id += 1;
    draft.home_sites.push(HomeOfficeSite {
        id,
        address: "Synthetic office home".into(),
        home_area: area.max(2000),
        paid_by: if owner == Owner::Spouse {
            HomeCostOwners::Spouse
        } else {
            HomeCostOwners::Taxpayer
        },
        mortgage_loans: vec![],
        tenure: HomeTenure::Rented,
        claims: vec![claim],
        costs: vec![],
        casualties: vec![],
    });
    id
}

pub(in crate::tests) fn add_simplified_office(
    draft: &mut FederalDraft,
    source: i64,
    area: u32,
) -> i64 {
    add_office_claim(draft, source, simplified_claim(area), area)
}

pub(in crate::tests) fn office_claim_mut(
    draft: &mut FederalDraft,
    source: i64,
) -> &mut HomeOfficeClaim {
    draft
        .home_sites
        .iter_mut()
        .flat_map(|site| &mut site.claims)
        .find(|claim| claim.activities.iter().any(|activity| activity.activity == source))
        .unwrap()
}

pub(in crate::tests) fn add_actual_office(
    draft: &mut FederalDraft,
    source: i64,
    fixture: OfficeFixture,
) -> i64 {
    let mut claim = simplified_claim(fixture.office_area);
    claim.prior_operating = fixture.prior_operating;
    claim.prior_depreciation = fixture.prior_depreciation;
    claim.prior_casualty = fixture.prior_casualty;
    claim.method =
        OfficeMethod::Actual { exclusive_area: fixture.office_area, shared_daycare_area: 0 };
    let id = add_office_claim(draft, source, claim, fixture.home_area);
    let site_index = draft.home_sites.len() - 1;
    draft.home_sites[site_index].home_area = fixture.home_area;
    let owner = draft.home_sites[site_index].claims[0].owner;
    let mut first = 20260101;
    let mut costs = vec![];
    match fixture.home {
        HomeFixture::Owned {
            mortgage_interest,
            mortgage_insurance,
            real_estate_taxes,
            basis,
            land,
            first_used,
        } => {
            first = first.max(first_used);
            let business_basis = crate::kernel::ratio::fraction(
                basis - land,
                i64::from(fixture.office_area),
                i64::from(fixture.home_area),
            );
            // Added history facts are synthetic completions, populated through
            // the existing table owner; expected current-year figures remain
            // the independent hand-computed assertions in the topic tests.
            let prior: i64 = (first_used / 10000..2026)
                .filter_map(|year| {
                    crate::depreciation::nonresidential_rate(
                        first_used / 100 % 100,
                        year - first_used / 10000,
                    )
                })
                .map(|rate| crate::kernel::ratio::fraction(business_basis, rate, 100000))
                .sum();
            draft.home_sites[site_index].tenure = HomeTenure::Owned {
                building: Some(alloc::boxed::Box::new(OfficeBuilding {
                    description: "Original home".into(),
                    first_used,
                    stopped: None,
                    adjusted_basis: basis,
                    conversion_value: basis,
                    land_basis: land,
                    land_value: land,
                    schedule: OfficeRecoveryHistory {
                        recovery: OfficeRecovery::Macrs39,
                        convention: OfficeOldConvention::MidMonth,
                        accumulated: prior,
                        straight_line_from: None,
                        simplified_years: vec![],
                    },
                    prior_post_1997: prior,
                    prior_pre_1997: 0,
                    production: None,
                })),
                improvements: vec![],
            };
            if mortgage_interest != 0 || mortgage_insurance != 0 {
                let loan_id = draft.next_id;
                draft.next_id += 1;
                draft.deductions.as_mut().unwrap().mortgage.push(MortgageLoan {
                    id: loan_id,
                    property: "Synthetic office home".into(),
                    lender: MortgageLender::Institution,
                    interest: mortgage_interest,
                    mortgage_insurance_premiums: mortgage_insurance,
                    outstanding_principal: 10_000_000,
                    incurred: first_used,
                    details: MortgageLoanDetails::FirstLender(MortgageLedger {
                        home: MortgageHome::Main,
                        purpose: MortgageUse::Home,
                        average: MortgageAverageBalance::Statements { average_balance: 10_000_000 },
                    }),
                    refund_of_interest: MortgageInterestRefund::No,
                    points: MortgagePoints::No,
                });
                draft.home_sites[site_index].mortgage_loans.push(loan_id);
            }
            costs.push((HomeCostKind::PropertyTax, real_estate_taxes, HomeCostScope::WholeHome));
        }
        HomeFixture::Rented { rent } => {
            costs.push((HomeCostKind::Rent, rent, HomeCostScope::WholeHome))
        }
    }
    draft.home_sites[site_index].claims[0].starts = first;
    costs.extend([
        (HomeCostKind::Insurance, fixture.insurance, HomeCostScope::WholeHome),
        (HomeCostKind::Repair, fixture.repairs, HomeCostScope::WholeHome),
        (HomeCostKind::Utilities, fixture.utilities, HomeCostScope::WholeHome),
        (HomeCostKind::Other, fixture.other, HomeCostScope::WholeHome),
        (HomeCostKind::Repair, fixture.direct, HomeCostScope::Exclusive { owner }),
    ]);
    draft.home_sites[site_index].costs = costs
        .into_iter()
        .filter(|(_, amount, _)| *amount != 0)
        .map(|(kind, amount, scope)| HomeCost {
            kind,
            amount,
            scope,
            starts: first,
            ends: 20261231,
        })
        .collect();
    id
}

pub(in crate::tests) fn office_return(profit: i64, office: OfficeFixture) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income.clear();
    office_business(draft, profit, office)
}

// 2026-10-17 paired audit: editor controls originally retain complete()'s
// $60,000 W-2 and withholding; worksheet-only controls intentionally do not.
pub(in crate::tests) fn office_interview_return(
    profit: i64,
    office: OfficeFixture,
) -> FederalDraft {
    office_business(complete(FederalStatus::Single), profit, office)
}

fn office_business(mut draft: FederalDraft, profit: i64, office: OfficeFixture) -> FederalDraft {
    add(&mut draft, income(cash_business(profit)));
    let source = draft.income.last().unwrap().id;
    add_actual_office(&mut draft, source, office);
    draft
}

pub(in crate::tests) fn interview_office() -> OfficeFixture {
    let mut office = actual_office(
        200,
        2_000,
        owned_office_home(1_000_000, 0, 300_000, 25_000_000, 5_000_000, 20210601),
    );
    office.insurance = 100_000;
    office.utilities = 240_000;
    office
}

pub(in crate::tests) fn office_line_id(
    result: &FederalResult,
    suffix: &str,
) -> alloc::string::String {
    result
        .lines
        .iter()
        .find(|line| {
            line.id.starts_with("home_")
                && (line.id.ends_with(&alloc::format!("_form_8829_{suffix}"))
                    || line.id.ends_with(&alloc::format!("_worksheet_{suffix}")))
        })
        .unwrap()
        .id
        .clone()
}

pub(in crate::tests) fn standard_office() -> OfficeFixture {
    let mut office = actual_office(
        300,
        2_000,
        owned_office_home(1_200_000, 0, 400_000, 30_000_000, 6_000_000, 20200315),
    );
    office.insurance = 120_000;
    office.repairs = 80_000;
    office.utilities = 300_000;
    office.direct = 40_000;
    office
}

pub(in crate::tests) fn itemizing_office(first_used: u32) -> OfficeFixture {
    let mut office = actual_office(
        250,
        2_500,
        owned_office_home(1_500_000, 0, 600_000, 40_000_000, 10_000_000, first_used),
    );
    office.insurance = 100_000;
    office.utilities = 240_000;
    office
}

// Original worksheet totals are recorded once: the site's home tax stays on
// the site and only the remainder is personal Schedule A property tax.
pub(in crate::tests) fn office_itemize(
    d: &mut FederalDraft,
    income_tax: i64,
    total_property: i64,
    interest: i64,
    premiums: i64,
) {
    let home_taxes: i64 = d
        .home_sites
        .iter()
        .flat_map(|site| &site.costs)
        .filter(|cost| matches!(cost.kind, HomeCostKind::PropertyTax))
        .map(|cost| cost.amount)
        .sum();
    let other_property = total_property
        .checked_sub(home_taxes)
        .filter(|amount| *amount >= 0)
        .expect("worksheet total includes the once-entered home tax");
    let linked = d
        .home_sites
        .iter()
        .flat_map(|site| site.mortgage_loans.iter())
        .copied()
        .collect::<Vec<_>>();
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_tax = PersonalTaxElection::Income {
        paid: income_tax,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    itemized(ded).real_estate_taxes = other_property;
    if let Some(loan) = ded.mortgage.iter_mut().find(|loan| linked.contains(&loan.id)) {
        loan.interest = interest;
        loan.mortgage_insurance_premiums = premiums;
    } else {
        assert_eq!((interest, premiums), (0, 0), "nonzero mortgage facts need their source");
    }
    let facts = itemized(ded).clone();
    ded.choice = DeductionSelection::Itemized { facts };
}
