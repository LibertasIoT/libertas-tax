use crate::k1_activity::*;
use crate::tests::*;
pub(crate) fn portfolio() -> K1StatementPortfolio {
    K1StatementPortfolio {
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
    }
}
pub(crate) fn items() -> K1StatementItems {
    K1StatementItems::Partnership { income: Vec::new(), deductions: Vec::new() }
}
pub(crate) fn amounts(operating: i64, charity: i64) -> K1StatementAmounts {
    K1StatementAmounts {
        operating,
        exempt_interest: 0,
        exempt_other: 0,
        nondeductible: 0,
        portfolio: portfolio(),
        operating_carry: K1RetainedLossCarry { basis: Some(0), risk: Some(0) },
        passive_operating: Some(0),
        property: None,
        section179: 0,
        section179_basis_carry: Some(0),
        section179_business_carry: Some(0),
        section179_sold_carry: Some(0),
        cash_charity: charity,
        charity_basis_carry: Some(0),
        foreign_taxes: 0,
        foreign_basis_carry: Some(0),
        items: items(),
        capital_carry: K1StatementCapitalCarry::Partnership {
            amounts: K1StatementPartnerCapitalCarry {
                short: K1RetainedLossCarry { basis: Some(0), risk: Some(0) },
                long: K1RetainedLossCarry { basis: Some(0), risk: Some(0) },
            },
        },
    }
}
pub(crate) fn entity_only() -> K1EntityOnlyAmounts {
    K1EntityOnlyAmounts {
        exempt_interest: 0,
        exempt_other: 0,
        nondeductible: 0,
        cash_charity: 0,
        charity_basis_carry: Some(0),
        foreign_taxes: 0,
        foreign_basis_carry: Some(0),
        portfolio: portfolio(),
        items: items(),
        capital_carry: K1StatementCapitalCarry::Partnership {
            amounts: K1StatementPartnerCapitalCarry {
                short: K1RetainedLossCarry { basis: Some(0), risk: Some(0) },
                long: K1RetainedLossCarry { basis: Some(0), risk: Some(0) },
            },
        },
    }
}
pub(crate) fn component(
    id: i64,
    name: &str,
    operating: i64,
    charity: i64,
    risk: i64,
    passive: i64,
) -> K1StatementActivity {
    K1StatementActivity {
        id,
        name: name.into(),
        kind: K1StatementKind::TradeBusiness,
        personal_services: K1ActivityPersonalSource::Unavailable,
        self_employment: K1StatementSe::Unavailable,
        qualified_current_controls: K1QualifiedCurrentControls { ordinary: None, property: None },
        regular: Some(amounts(operating, charity)),
        alternative: K1StatementAmt::SameAsRegular,
        risk_group: Some(risk),
        passive_group: Some(passive),
    }
}
pub(crate) fn risk(id: i64, name: &str) -> K1RiskGroup {
    K1RiskGroup {
        id,
        name: name.into(),
        source: K1RiskSource::Unavailable,
        investment_parts: Some(K1RiskInvestmentParts {
            contributions: 0,
            distributions: 0,
            loan_advances: 0,
            loan_repayments: 0,
        }),
    }
}
pub(crate) fn passive(id: i64, name: &str) -> K1PassiveGroup {
    K1PassiveGroup {
        id,
        name: name.into(),
        participation: K1ActivityParticipation::Unavailable,
        disposed: false,
    }
}
pub(crate) fn block(id: i64, name: &str) -> K1QbiBlock {
    K1QbiBlock {
        id,
        name: name.into(),
        reported_aggregate: false,
        statement: K1ReportedQbiStatement::Unavailable,
        business_names: None,
        allocations: None,
        qbi_aggregation_group: None,
    }
}
pub(crate) fn source() -> K1ActivitySources {
    K1ActivitySources {
        components: alloc::vec![
            component(11, "Actual A", 400_000, 400_000, 21, 31),
            component(12, "Actual B", -600_000, 0, 22, 32)
        ],
        risk_groups: alloc::vec![risk(21, "Risk A"), risk(22, "Risk B")],
        passive_groups: alloc::vec![passive(31, "Passive A"), passive(32, "Passive B")],
        qbi_blocks: alloc::vec![block(41, "Business A"), block(42, "Business B")],
        investment_property_sources: None,
        section179_sale_sources: None,
        entity_only: Some(entity_only()),
        alternative_entity_only: K1EntityOnlyAmt::SameAsRegular,
    }
}
pub(crate) fn current() -> K1CurrentStatementTotals {
    K1CurrentStatementTotals {
        ordinary: -200_000,
        rental_real_estate: 0,
        other_rental: 0,
        property: None,
        portfolio: portfolio(),
        exempt_interest: 0,
        exempt_other: 0,
        nondeductible: 0,
        section179: 0,
        cash_charity: 400_000,
        foreign_taxes: 0,
        items: items(),
    }
}
pub(crate) fn parent() -> PartnerActivityK1 {
    PartnerActivityK1 {
        entity: PartnerEntitySource {
            amt_section179_income_carry: None,
            reporting_period: None,
            section179_sales: Vec::new(),
            regular: PartnerEntityBasis {
                opening: Some(200_000),
                contributions: 0,
                opening_liabilities: 0,
                ending_liabilities: 0,
                assumed: 0,
                transferred: 0,
                distributions: 0,
                prior_nondeductible: Some(0),
                complete: true,
            },
            alternative: None,
            publicly_traded: false,
            final_k1: false,
            interest_term: None,
            disposition: crate::K1Disposition::Kept,
            sale: crate::PartnerSaleParts::default(),
            schedule_k3: None,
            credits: Vec::new(),
            investment_property: Vec::new(),
            investment_confirmed: None,
            housing_confirmed: None,
        },
        current: current(),
        alternative_current: None,
        activities: K1ComponentStatement::Entered { source: source() },
        person: None,
        unentered: Vec::new(),
        special: Vec::new(),
        health_insurance: None,
        retirement: None,
    }
}
pub(crate) fn history(
    original: i64,
    remaining: i64,
    qualified: i64,
    remaining_qualified: i64,
) -> K1ComponentQualified {
    K1ComponentQualified::Partnership {
        current_income: 0,
        current_property: 0,
        records: crate::partnership_qbi::PartnerQbiRecords {
            qualified_se_gross: 0,
            history: Some(crate::PartnerQbiHistory {
                basis: alloc::vec![crate::ScorpLossYear {
                    year: 2020,
                    original,
                    remaining,
                    qualified,
                    remaining_qualified
                }],
                risk: Vec::new(),
                passive: Vec::new(),
            }),
        },
    }
}
pub(crate) fn histories() -> K1ActivitySources {
    let mut source = source();
    source.components[0].regular = Some(amounts(0, 0));
    source.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(5_000);
    source.qbi_blocks[0].allocations = Some(alloc::vec![K1QbiAllocation {
        component: 11,
        qualified: history(10_000, 5_000, 4_000, 2_000)
    }]);
    source.qbi_blocks[1].allocations = Some(alloc::vec![K1QbiAllocation {
        component: 11,
        qualified: history(10_000, 5_000, 3_000, 1_500)
    }]);
    source
}
pub(crate) fn signed_block(source: &mut K1ActivitySources, index: usize, amount: i64) {
    source.qbi_blocks[index].statement = K1ReportedQbiStatement::Partnership {
        statement: Some(crate::PartnerQbiStatement {
            income: amount,
            wages: 0,
            property: 0,
            service: false,
            section1231_qbi: 0,
            reit_dividends: 0,
            ptp_income: 0,
        }),
    };
    source.qbi_blocks[index].allocations = Some(alloc::vec![K1QbiAllocation {
        component: 11,
        qualified: K1ComponentQualified::Partnership {
            current_income: amount,
            current_property: 0,
            records: crate::partnership_qbi::PartnerQbiRecords::default()
        }
    }]);
}
pub(crate) fn scorp_items() -> K1StatementItems {
    K1StatementItems::Scorp { income: Vec::new(), deductions: Vec::new() }
}
pub(crate) fn scorp_carry() -> K1StatementCapitalCarry {
    K1StatementCapitalCarry::Scorp {
        amounts: K1StatementScorpCapitalCarry {
            basis_short: Some(0),
            basis_long: Some(0),
            basis_collectibles: Some(0),
            risk_short: Some(0),
            risk_long: Some(0),
            risk_collectibles: Some(0),
        },
    }
}
pub(crate) fn fresh_income(scorp: bool) -> FederalIncome {
    let mut partner = parent();
    let K1ComponentStatement::Entered { source } = &mut partner.activities else { unreachable!() };
    for component in &mut source.components {
        component.id = 0;
        component.risk_group = None;
        component.passive_group = None;
    }
    for risk in &mut source.risk_groups {
        risk.id = 0;
        risk.investment_parts = None;
    }
    for passive in &mut source.passive_groups {
        passive.id = 0;
    }
    for block in &mut source.qbi_blocks {
        block.id = 0;
    }
    if !scorp {
        return FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(partner) };
    }
    scorp_income(partner)
}
fn scorp_income(mut partner: PartnerActivityK1) -> FederalIncome {
    let K1ComponentStatement::Entered { source } = &mut partner.activities else { unreachable!() };
    for component in &mut source.components {
        component.personal_services = K1ActivityPersonalSource::NotApplicable;
        component.self_employment = K1StatementSe::NotApplicable;
        let amounts = component.regular.as_mut().unwrap();
        amounts.items = scorp_items();
        amounts.capital_carry = scorp_carry();
    }
    let entity_only = source.entity_only.as_mut().unwrap();
    entity_only.items = scorp_items();
    entity_only.capital_carry = scorp_carry();
    partner.current.items = scorp_items();
    FederalIncome::ScorpActivities {
        data: ScorpActivityK1 {
            entity: ScorpEntitySource {
                amt_section179_income_carry: None,
                reporting_period: None,
                section179_sales: Vec::new(),
                regular: ScorpEntityBasis {
                    opening: Some(200_000),
                    additions: 0,
                    distributions: 0,
                    prior_nondeductible: Some(0),
                    losses_first: false,
                    complete: true,
                },
                alternative: None,
                loans: Vec::new(),
                final_k1: false,
                stock_term: None,
                disposition: K1Disposition::Kept,
                schedule_k3: None,
                credits: Vec::new(),
                investment_property: Vec::new(),
                investment_confirmed: None,
                housing_confirmed: None,
            },
            current: partner.current,
            alternative_current: None,
            activities: partner.activities,
            person: None,
            unentered: Vec::new(),
            special: Vec::new(),
            health_insurance: None,
        },
    }
}
pub(crate) fn input(value: &FederalIncome) -> IncomeDocument {
    match value {
        FederalIncome::PartnershipActivities { data } => {
            IncomeDocument::K1PartnershipActivities(alloc::boxed::Box::new(
                K1PartnershipActivitiesPage::from_record("Synthetic actual partnership", data),
            ))
        }
        FederalIncome::ScorpActivities { data } => IncomeDocument::K1ScorpActivities(
            K1ScorpActivitiesPage::from_record("Synthetic actual corporation", data),
        ),
        _ => unreachable!(),
    }
}
pub(crate) fn page(value: &mut IncomeDocument) -> &mut K1ComponentDetailsPage {
    match value {
        IncomeDocument::K1PartnershipActivities(v) => &mut v.activities,
        IncomeDocument::K1ScorpActivities(v) => &mut v.activities,
        _ => unreachable!(),
    }
}
pub(crate) fn graph(value: &mut IncomeDocument) -> &mut K1ActivitySourcesPage {
    let K1ComponentStatementPage::Entered { source } = &mut page(value).statement else {
        unreachable!()
    };
    source
}
pub(crate) fn documents(request: &mut P) -> &mut Vec<IncomeDocumentItem> {
    let P::SaveIncomes { documents, .. } = request else { unreachable!() };
    &mut documents.documents
}
pub(crate) fn nullable_sources(income: &mut FederalIncome) {
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = None;
            data.entity.regular.prior_nondeductible = None;
            data.entity.regular.complete = false;
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = None;
            data.entity.regular.prior_nondeductible = None;
            data.entity.regular.complete = false;
        }
        _ => unreachable!(),
    }
    let scorp = matches!(income, FederalIncome::ScorpActivities { .. });
    let (activities, current, alternative_current) = match income {
        FederalIncome::PartnershipActivities { data } => {
            (&mut data.activities, &mut data.current, &mut data.alternative_current)
        }
        FederalIncome::ScorpActivities { data } => {
            (&mut data.activities, &mut data.current, &mut data.alternative_current)
        }
        _ => unreachable!(),
    };
    let K1ComponentStatement::Entered { source } = activities else { unreachable!() };
    for component in &mut source.components {
        let amounts = component.regular.as_mut().unwrap();
        amounts.operating_carry.risk = None;
        amounts.section179_basis_carry = None;
        amounts.charity_basis_carry = None;
    }
    let amounts = source.components[0].regular.as_mut().unwrap();
    amounts.operating_carry.basis = Some(10_000);
    amounts.passive_operating = Some(5_000);
    amounts.section179_business_carry = Some(3_000);
    amounts.section179_sold_carry = None;
    amounts.foreign_basis_carry = Some(4_000);
    amounts.property = Some(K1StatementProperty {
        net: 12_345,
        unrecaptured: 54_321,
        basis_loss: None,
        risk_loss: Some(2_000),
        passive_loss: None,
    });
    match &mut amounts.capital_carry {
        K1StatementCapitalCarry::Partnership { amounts } => {
            amounts.short.basis = Some(100);
            amounts.short.risk = None;
            amounts.long.basis = None;
            amounts.long.risk = Some(200);
        }
        K1StatementCapitalCarry::Scorp { amounts } => {
            amounts.basis_short = Some(100);
            amounts.basis_long = None;
            amounts.basis_collectibles = Some(10);
            amounts.risk_short = None;
            amounts.risk_long = Some(200);
            amounts.risk_collectibles = Some(100);
        }
        _ => unreachable!(),
    }
    let mut alternative = amounts.clone();
    alternative.property.as_mut().unwrap().net = -12_345;
    alternative.operating_carry.basis = None;
    alternative.operating_carry.risk = Some(2_500);
    source.components[0].alternative = K1StatementAmt::Separate { amounts: alternative };
    current.property = Some(K1CurrentProperty { net: 12_345, unrecaptured: 54_321 });
    let mut amt = current.clone();
    amt.property.as_mut().unwrap().net = -12_345;
    *alternative_current = Some(amt);
    // The pooled worksheet establishes50000c, not an allocation to either child.
    // All other literal zero facts here are explicitly established by this fixture.
    source.risk_groups[0].name = "Actual pooled retained worksheet".into();
    source.risk_groups[0].source = match scorp {
        false => K1RiskSource::Partnership {
            regular: PartnerRiskRecord {
                start: ScorpRiskStart::Unknown,
                opening_debt: 0,
                ending_debt: 0,
                excluded_contributions: 0,
                excluded_distributions: 0,
                protected: 0,
                released: 0,
                prior_loss: 50_000,
                prior_nondeductible: 0,
                deducted: 0,
                recaptured: 0,
                complete: false,
            },
            alternative: None,
        },
        true => K1RiskSource::Scorp {
            regular: ScorpRisk {
                start: ScorpRiskStart::Unknown,
                excluded_funding: 0,
                protected: 0,
                released: 0,
                excluded_returns: 0,
                prior_loss: 50_000,
                prior_nondeductible: 0,
                deducted: 0,
                recaptured: 0,
                complete: false,
            },
            alternative: None,
        },
    };
}
pub(crate) fn bind_properties(document: &mut IncomeDocument) {
    let ids: Vec<_> = match document {
        IncomeDocument::K1PartnershipActivities(v) => {
            v.entity.investment_property.iter().map(crate::K1InvestmentProperty::id).collect()
        }
        IncomeDocument::K1ScorpActivities(v) => {
            v.entity.investment_property.iter().map(crate::K1InvestmentProperty::id).collect()
        }
        _ => unreachable!(),
    };
    let source = graph(document);
    source.investment_property_sources = Some(
        ids.iter()
            .zip(&source.components)
            .map(|(property, component)| K1InvestmentPropertySourcePage {
                property: *property,
                component: component.id,
            })
            .collect(),
    );
}
pub(crate) fn current_credit_sources() -> Vec<crate::K1Credit> {
    use crate::K1CreditCode::*;
    [
        WorkOpportunity,
        EmployerTipFica,
        PensionStartup,
        AutoEnrollment,
        MilitarySpouse,
        SmallEmployerHealth,
        PaidLeave,
        LowIncomeHousingJ5,
        LowIncomeHousingOther,
        Research,
    ]
    .into_iter()
    .map(|code| crate::K1Credit {
        code,
        line: (code == Research).then_some(crate::BusinessCredit::ResearchSmallBusiness),
        amount: 12_345,
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
    })
    .collect()
}

pub(crate) fn sale_fields(
    document: &mut IncomeDocument,
) -> (&mut Vec<K1Section179Sale>, &mut Option<K1EntityTaxPeriod>) {
    match document {
        IncomeDocument::K1PartnershipActivities(value) => {
            (&mut value.section179_sales, &mut value.reporting_period)
        }
        IncomeDocument::K1ScorpActivities(value) => {
            (&mut value.section179_sales, &mut value.reporting_period)
        }
        _ => unreachable!(),
    }
}
pub(crate) fn current_sale() -> K1Section179Sale {
    K1Section179Sale {
        id: 0,
        description: "Actual physical sale".into(),
        acquired: 20240117,
        sold: 20260523,
        regular: Some(K1Section179SaleAmounts {
            price: 100_001,
            cost: 200_002,
            depreciation: 50_003,
            section179: 25_004,
        }),
        alternative: K1Section179SaleAmtSource::Unavailable,
    }
}
pub(crate) fn add_sale_sources(document: &mut IncomeDocument) {
    let (sales, period) = sale_fields(document);
    sales.push(current_sale());
    *period = Some(K1EntityTaxPeriod { began: 20250701, ended: 20260630 });
}
pub(crate) fn bind_sales(document: &mut IncomeDocument) {
    let ids: Vec<_> = sale_fields(document).0.iter().map(|sale| sale.id).collect();
    let source = graph(document);
    source.section179_sale_sources = Some(
        ids.iter()
            .zip(&source.components)
            .map(|(sale, component)| K1Section179SaleSourcePage {
                sale: *sale,
                component: component.id,
            })
            .collect(),
    );
}

pub(crate) fn amt_income_carry(
    document: &mut IncomeDocument,
) -> &mut Option<Section179IncomeCarryHistory> {
    match document {
        IncomeDocument::K1PartnershipActivities(value) => {
            &mut value.entity.amt_section179_income_carry
        }
        IncomeDocument::K1ScorpActivities(value) => &mut value.entity.amt_section179_income_carry,
        _ => unreachable!(),
    }
}

/// Empty construction choices for a fresh synthetic income request.
pub(in crate::tests) fn empty_k1_activity_choices()
-> alloc::boxed::Box<crate::K1ActivityChoiceContext> {
    alloc::boxed::Box::default()
}

pub(crate) fn activity_health_plan(scorp: bool, wage: i64) -> SeHealth {
    SeHealth::Plan {
        establishment: if scorp {
            HealthEstablishmentPage::Scorporation {
                payment: EntityPremiumPayment::EntityPaid,
                wage,
            }
        } else {
            HealthEstablishmentPage::Partnership { payment: EntityPremiumPayment::EntityPaid }
        },
        months: vec![BusinessHealthMonth {
            month: 1,
            premiums: 35_000,
            child_increase: 0,
            employer_eligible: false,
            reimbursed: None,
        }],
        long_term_care: Vec::new(),
    }
}

pub(crate) fn current_fuel_source() -> PartnerFuelTaxCredit {
    PartnerFuelTaxCredit {
        fuels: FederalFuelLines {
            gasoline_off_highway: Some(fuel_use(100_000, 20_000)),
            ..Default::default()
        },
        statement_complete: true,
        farm_activity: false,
    }
}

pub(crate) fn prior_fuel_source() -> PartnerPriorFuelCredit {
    PartnerPriorFuelCredit {
        claimed_credit: 10_000,
        filed_in_return_year: true,
        cash_method: true,
        full_prior_tax_benefit: true,
        partial_prior_year_benefit: None,
        prior_niit_recovery_excluded: true,
        not_reported_elsewhere: true,
    }
}

/// Original ordinary multi-activity source, including its original validation assertion.
pub(crate) fn ordinary_multi_parent() -> PartnerActivityK1 {
    let mut value = parent();
    value.current.cash_charity = 0;
    value.entity.regular.opening = Some(100_000);
    value.entity.alternative = Some(value.entity.regular.clone());
    value.entity.alternative.as_mut().unwrap().opening = Some(300_000);
    value.alternative_current = Some(value.current.clone());
    value.health_insurance = Some(BusinessHealthInsurance::None);
    value.retirement = Some(PartnerRetirement::None);
    value.person = Some(PartnerPersonSource {
        guaranteed_services: Some(0),
        guaranteed_capital: Some(0),
        pensions: Some(0),
        health_guaranteed: Some(0),
        health_distributions: Some(0),
        deferred_compensation_tax: Some(0),
        crp_payments: Some(0),
        limited_for_se: Some(true),
        reported_se: Some(0),
        se_history: K1RetainedSeHistory { basis: Some(0), risk: Some(0), passive: Some(0) },
        se_gross: Some(vec![]),
        fuel_tax_credit: Some(K1CurrentFuelSource::NotReported),
        prior_fuel_credit: Some(K1PriorFuelSource::NotReported),
    });
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for component in &mut source.components {
        component.personal_services = K1ActivityPersonalSource::NotApplicable;
        component.self_employment = K1StatementSe::NotApplicable;
        component.regular.as_mut().unwrap().cash_charity = 0;
    }
    let losses = source.components[1].regular.as_mut().unwrap();
    losses.operating_carry = K1RetainedLossCarry { basis: Some(100_000), risk: Some(100_000) };
    losses.passive_operating = Some(200_000);
    let mut alternative = losses.clone();
    alternative.passive_operating = Some(100_000);
    source.components[1].alternative = K1StatementAmt::Separate { amounts: alternative };
    for (index, group) in source.risk_groups.iter_mut().enumerate() {
        let mut regular = partner_risk_record();
        regular.start = ScorpRiskStart::Detailed {
            capacity: if index == 0 { 0 } else { 300_000 },
            profit: 0,
            loss: 0,
        };
        regular.prior_loss = if index == 0 { 0 } else { 100_000 };
        let mut alternative = regular.clone();
        if index == 1 {
            alternative.start = ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 };
        }
        group.source = K1RiskSource::Partnership { regular, alternative: Some(alternative) };
    }
    for (index, group) in source.passive_groups.iter_mut().enumerate() {
        group.participation = K1ActivityParticipation::Partnership {
            facts: PartnerParticipation::Limited {
                minutes: if index == 0 { 0 } else { 30_001 },
                prior_ten: 0,
                personal_service: false,
                service_years: 0,
            },
            trade_kind: ScorpTradeKind::Ordinary,
        };
    }
    for block in &mut source.qbi_blocks {
        block.statement = K1ReportedQbiStatement::Partnership { statement: None };
        block.allocations = Some(vec![]);
    }
    source.qbi_blocks[1].allocations = Some(vec![K1QbiAllocation {
        component: source.components[1].id,
        qualified: K1ComponentQualified::Partnership {
            current_income: 0,
            current_property: 0,
            records: crate::partnership_qbi::PartnerQbiRecords {
                qualified_se_gross: 0,
                history: Some(PartnerQbiHistory {
                    basis: vec![loss_year(2025, 1_000, 0, 1_000, 0)],
                    risk: vec![loss_year(2025, 1_000, 0, 1_000, 0)],
                    passive: vec![loss_year(2025, 2_000, 0, 2_000, 0)],
                }),
            },
        },
    }]);
    assert_eq!(value.rule(), None);
    value
}

/// Explicit synthetic original-year qualified losses; current B is documented zero.
pub(crate) fn ordinary_multi_retained_qbi_parent() -> PartnerActivityK1 {
    let mut value = ordinary_multi_parent();
    value.current.ordinary = 400_000;
    value.alternative_current = Some(value.current.clone());
    value.entity.alternative = Some(value.entity.regular.clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    let component = &mut source.components[1];
    let regular = component.regular.as_mut().unwrap();
    regular.operating = 0;
    regular.passive_operating = Some(0);
    component.alternative = K1StatementAmt::Separate { amounts: regular.clone() };
    component.qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 0, loss: 0 });
    let K1RiskSource::Partnership { regular, alternative } = &mut source.risk_groups[1].source
    else {
        unreachable!()
    };
    *alternative = Some(regular.clone());
    let K1ComponentQualified::Partnership { records, .. } =
        &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2025, 1_000, 500, 1_000, 500)],
        risk: vec![loss_year(2025, 1_000, 800, 1_000, 800)],
        passive: vec![],
    });
    assert_eq!(value.rule(), None);
    value
}

/// Explicit synthetic current statements: A earns4000/qualified3000, B earns2000/qualified0.
pub(crate) fn ordinary_multi_current_qbi_parent() -> PartnerActivityK1 {
    ordinary_multi_current_qbi_parent_with_statements([Some((300_000, 0)), None])
}

/// Forward actual statement income and W-2 wages in cents to the existing native sources.
pub(crate) fn ordinary_multi_current_qbi_parent_with_statements(
    statements: [Option<(i64, i64)>; 2],
) -> PartnerActivityK1 {
    let mut value = ordinary_multi_parent();
    value.current.ordinary = 600_000;
    value.alternative_current = Some(value.current.clone());
    let K1ComponentStatement::Entered { source: component_source } = &mut value.activities else {
        unreachable!()
    };
    for (index, component) in component_source.components.iter_mut().enumerate() {
        let income = if index == 0 { 400_000 } else { 200_000 };
        component.regular = Some(amounts(income, 0));
        component.alternative = K1StatementAmt::Separate { amounts: amounts(income, 0) };
        component.qualified_current_controls.ordinary =
            Some(K1CurrentSourceParts { income, loss: 0 });
    }
    for group in &mut component_source.risk_groups {
        let K1RiskSource::Partnership { regular, alternative } = &mut group.source else {
            unreachable!()
        };
        regular.prior_loss = 0;
        alternative.as_mut().unwrap().prior_loss = 0;
    }
    for group in &mut component_source.passive_groups {
        group.participation = K1ActivityParticipation::Partnership {
            facts: PartnerParticipation::Limited {
                minutes: 30_001,
                prior_ten: 0,
                personal_service: false,
                service_years: 0,
            },
            trade_kind: ScorpTradeKind::Ordinary,
        };
    }
    for (index, block) in component_source.qbi_blocks.iter_mut().enumerate() {
        let component = &component_source.components[index];
        let statement = statements[index];
        let qualified = statement.map_or(0, |(income, _)| income);
        block.reported_aggregate = false;
        block.qbi_aggregation_group = None;
        block.business_names =
            Some(vec![crate::K1QbiBusinessName { name: component.name.clone() }]);
        block.statement = K1ReportedQbiStatement::Partnership {
            statement: statement.map(|(income, wages)| crate::PartnerQbiStatement {
                income,
                wages,
                property: 0,
                service: false,
                section1231_qbi: 0,
                reit_dividends: 0,
                ptp_income: 0,
            }),
        };
        block.allocations = Some(vec![K1QbiAllocation {
            component: component.id,
            qualified: K1ComponentQualified::Partnership {
                current_income: qualified,
                current_property: 0,
                records: crate::partnership_qbi::PartnerQbiRecords {
                    qualified_se_gross: 0,
                    history: Some(PartnerQbiHistory {
                        basis: vec![],
                        risk: vec![],
                        passive: vec![],
                    }),
                },
            },
        }]);
    }
    assert_eq!(value.rule(), None);
    value
}

/// Original S-corporation fixture, configured on an accepted source without changing identities.
pub(crate) fn scorp_ordinary_multi_baseline(mut value: ScorpActivityK1) -> ScorpActivityK1 {
    value.current.cash_charity = 0;
    value.entity.regular.opening = Some(100_000);
    value.entity.alternative = Some(value.entity.regular.clone());
    value.entity.alternative.as_mut().unwrap().opening = Some(200_000);
    value.entity.loans = vec![ScorpEntityLoan {
        regular: ScorpLoan {
            kind: ScorpLoanKind::FormalNote,
            repayment_term: ScorpNoteHolding::NoRepayment,
            principal: 500_000,
            basis: 100_000,
            advances: 0,
            repayments: 0,
        },
        alternative_basis: Some(50_000),
    }];
    value.alternative_current = Some(value.current.clone());
    value.person = Some(ScorpPersonSource { withholding: Some(0), private_activity: Some(0) });
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    // The first native Save allocated these durable group IDs; selecting
    // them here models the source editor's explicit associations.
    for (index, component) in source.components.iter_mut().enumerate() {
        component.risk_group = Some(source.risk_groups[index].id);
        component.passive_group = Some(source.passive_groups[index].id);
        component.regular.as_mut().unwrap().cash_charity = 0;
    }
    let loss = source.components[1].regular.as_mut().unwrap();
    loss.operating_carry = K1RetainedLossCarry { basis: Some(100_000), risk: Some(100_000) };
    loss.passive_operating = Some(200_000);
    let mut alternative = loss.clone();
    alternative.passive_operating = Some(100_000);
    source.components[1].alternative = K1StatementAmt::Separate { amounts: alternative };
    for (index, group) in source.risk_groups.iter_mut().enumerate() {
        let mut regular = scorp_risk_history();
        regular.start = ScorpRiskStart::Detailed {
            capacity: if index == 0 { 0 } else { 250_000 },
            profit: 0,
            loss: 0,
        };
        regular.prior_loss = if index == 0 { 0 } else { 100_000 };
        regular.deducted = 0;
        let mut alternative = regular.clone();
        if index == 1 {
            alternative.start = ScorpRiskStart::Detailed { capacity: 450_000, profit: 0, loss: 0 };
        }
        group.source = K1RiskSource::Scorp { regular, alternative: Some(alternative) };
        group.investment_parts = Some(K1RiskInvestmentParts {
            contributions: 0,
            distributions: 0,
            loan_advances: 0,
            loan_repayments: 0,
        });
    }
    for (index, group) in source.passive_groups.iter_mut().enumerate() {
        let mut facts = participation_facts();
        facts.minutes = if index == 0 { 0 } else { 30_001 };
        group.participation =
            K1ActivityParticipation::Scorp { facts, trade_kind: ScorpTradeKind::Ordinary };
    }
    for block in &mut source.qbi_blocks {
        block.statement = K1ReportedQbiStatement::Scorp { statement: None };
        block.allocations = Some(vec![]);
    }
    source.qbi_blocks[1].allocations = Some(vec![K1QbiAllocation {
        component: source.components[1].id,
        qualified: K1ComponentQualified::Scorp {
            current_income: 0,
            ordinary: ScorpLossQbi {
                current: 0,
                history: vec![loss_year(2025, 1_000, 0, 1_000, 0)],
                at_risk: vec![loss_year(2025, 1_000, 0, 1_000, 0)],
                passive: vec![loss_year(2025, 2_000, 0, 2_000, 0)],
            },
            property: ScorpPropertyQbi::default(),
        },
    }]);
    value
}

/// Explicit positive-current source facts; original qualified and loss histories are empty.
pub(crate) fn current_scorp_qbi_source(value: ScorpActivityK1) -> ScorpActivityK1 {
    current_scorp_qbi_source_with_statements(value, [Some((300_000, 0)), None])
}

/// Forward actual statement income and W-2 wages in cents to the existing native sources.
pub(crate) fn current_scorp_qbi_source_with_statements(
    mut value: ScorpActivityK1,
    statements: [Option<(i64, i64)>; 2],
) -> ScorpActivityK1 {
    value.current.ordinary = 600_000;
    value.alternative_current = Some(value.current.clone());
    value.entity.alternative = Some(value.entity.regular.clone());
    value.entity.loans.clear();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for (index, component) in source.components.iter_mut().enumerate() {
        let income = [400_000, 200_000][index];
        let mut regular = amounts(income, 0);
        regular.items = scorp_items();
        regular.capital_carry = scorp_carry();
        component.alternative = K1StatementAmt::Separate { amounts: regular.clone() };
        component.regular = Some(regular);
        component.qualified_current_controls.ordinary =
            Some(K1CurrentSourceParts { income, loss: 0 });
    }
    for group in &mut source.risk_groups {
        let mut regular = scorp_risk_history();
        regular.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
        regular.prior_loss = 0;
        regular.deducted = 0;
        group.source = K1RiskSource::Scorp { alternative: Some(regular.clone()), regular };
    }
    for group in &mut source.passive_groups {
        let mut facts = participation_facts();
        facts.minutes = 30_001;
        group.participation =
            K1ActivityParticipation::Scorp { facts, trade_kind: ScorpTradeKind::Ordinary };
    }
    for (index, block) in source.qbi_blocks.iter_mut().enumerate() {
        let component = &source.components[index];
        let statement = statements[index];
        let qualified = statement.map_or(0, |(income, _)| income);
        block.reported_aggregate = false;
        block.qbi_aggregation_group = None;
        block.business_names = Some(vec![K1QbiBusinessName { name: component.name.clone() }]);
        block.statement = K1ReportedQbiStatement::Scorp {
            statement: statement.map(|(income, wages)| K1ScorpBlockStatement {
                income,
                section1231_qbi: 0,
                wages,
                property: 0,
                service: false,
                reit_dividends: 0,
                ptp_income: 0,
            }),
        };
        block.allocations = Some(vec![K1QbiAllocation {
            component: component.id,
            qualified: K1ComponentQualified::Scorp {
                current_income: qualified,
                ordinary: ScorpLossQbi::default(),
                property: ScorpPropertyQbi {
                    gain: 0,
                    qualified_1250: Some(0),
                    losses: ScorpLossQbi::default(),
                    complete: true,
                },
            },
        }]);
    }
    value
}

pub(crate) fn ordinary_multi_current_scorp_qbi_parent() -> ScorpActivityK1 {
    let FederalIncome::ScorpActivities { mut data } = scorp_income(parent()) else {
        unreachable!()
    };
    data.health_insurance = Some(BusinessHealthInsurance::None);
    current_scorp_qbi_source(scorp_ordinary_multi_baseline(data))
}

pub(crate) fn qbi_aggregation_facts() -> QbiAggregationPage {
    QbiAggregationPage {
        facts: QbiAggregationFactsPage {
            name: "Actual elected A+B".into(),
            common_ownership: true,
            majority_year_last_day: true,
            same_tax_year: true,
            same_or_customary_products: true,
            shared_facilities_or_elements: true,
            coordinated_or_interdependent: false,
            election: QbiElectionHistory::FirstTimelyOriginalReturn,
            irs_history: QbiIrsDisaggregationHistory::None,
            disclosure: vec![],
        },
        members: vec![],
        disclosure_sources: vec![],
        businesses: None,
    }
}

// Explicit native source facts for the financial property cohort. The initial
// Save owns every identity; this only completes those accepted source records.
pub(crate) fn financial_property_sources(income: &mut FederalIncome) {
    let scorp = matches!(income, FederalIncome::ScorpActivities { .. });
    let (activities, current, alternative_current, properties) = match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(100_000);
            data.entity.alternative = Some(data.entity.regular.clone());
            data.health_insurance = Some(BusinessHealthInsurance::None);
            data.retirement = Some(PartnerRetirement::None);
            data.person = Some(PartnerPersonSource {
                guaranteed_services: Some(0),
                guaranteed_capital: Some(0),
                pensions: Some(0),
                health_guaranteed: Some(0),
                health_distributions: Some(0),
                deferred_compensation_tax: Some(0),
                crp_payments: Some(0),
                limited_for_se: Some(true),
                reported_se: Some(0),
                se_history: K1RetainedSeHistory { basis: Some(0), risk: Some(0), passive: Some(0) },
                se_gross: Some(Vec::new()),
                fuel_tax_credit: Some(K1CurrentFuelSource::NotReported),
                prior_fuel_credit: Some(K1PriorFuelSource::NotReported),
            });
            (
                &mut data.activities,
                &mut data.current,
                &mut data.alternative_current,
                &mut data.entity.investment_property,
            )
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(100_000);
            data.entity.alternative = Some(data.entity.regular.clone());
            data.health_insurance = Some(BusinessHealthInsurance::None);
            data.person =
                Some(ScorpPersonSource { withholding: Some(0), private_activity: Some(0) });
            (
                &mut data.activities,
                &mut data.current,
                &mut data.alternative_current,
                &mut data.entity.investment_property,
            )
        }
        _ => unreachable!(),
    };
    current.cash_charity = 0;
    *alternative_current = Some(current.clone());
    assert_eq!(properties.len(), 2);
    for (property, amount) in properties.iter_mut().zip([20_000, 40_000]) {
        let K1InvestmentProperty::Energy { own_form, .. } = property else { unreachable!() };
        *own_form = Some(OwnInvestmentCredit { amount, reduced: true });
    }
    let K1ComponentStatement::Entered { source } = activities else { unreachable!() };
    for index in 0..source.components.len() {
        let component = &mut source.components[index];
        let operating = [400_000, -600_000][index];
        let mut regular = amounts(operating, 0);
        if scorp {
            regular.items = scorp_items();
            regular.capital_carry = scorp_carry();
        }
        component.regular = Some(regular);
        component.alternative = K1StatementAmt::SameAsRegular;
        component.personal_services = K1ActivityPersonalSource::NotApplicable;
        component.self_employment = K1StatementSe::NotApplicable;
        component.qualified_current_controls = K1QualifiedCurrentControls {
            ordinary: Some(K1CurrentSourceParts {
                income: operating.max(0),
                loss: (-operating).max(0),
            }),
            property: None,
        };
        component.risk_group = Some(source.risk_groups[index].id);
        component.passive_group = Some(source.passive_groups[index].id);
        let risk = &mut source.risk_groups[index];
        let start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
        risk.source = if scorp {
            let mut regular = scorp_risk_history();
            regular.start = start;
            regular.prior_loss = 0;
            regular.deducted = 0;
            K1RiskSource::Scorp { alternative: Some(regular.clone()), regular }
        } else {
            let mut regular = partner_risk_record();
            regular.start = start;
            K1RiskSource::Partnership { alternative: Some(regular.clone()), regular }
        };
        risk.investment_parts = Some(K1RiskInvestmentParts {
            contributions: 0,
            distributions: 0,
            loan_advances: 0,
            loan_repayments: 0,
        });
        let mut participation = participation_facts();
        participation.minutes = 30_001;
        source.passive_groups[index].participation = if scorp {
            K1ActivityParticipation::Scorp {
                facts: participation,
                trade_kind: ScorpTradeKind::Ordinary,
            }
        } else {
            K1ActivityParticipation::Partnership {
                facts: PartnerParticipation::Limited {
                    minutes: 30_001,
                    prior_ten: 0,
                    personal_service: false,
                    service_years: 0,
                },
                trade_kind: ScorpTradeKind::Ordinary,
            }
        };
        let block = &mut source.qbi_blocks[index];
        block.reported_aggregate = false;
        block.qbi_aggregation_group = None;
        block.business_names = Some(vec![K1QbiBusinessName { name: component.name.clone() }]);
        block.statement = if scorp {
            K1ReportedQbiStatement::Scorp { statement: None }
        } else {
            K1ReportedQbiStatement::Partnership { statement: None }
        };
        block.allocations = Some(vec![K1QbiAllocation {
            component: component.id,
            qualified: if scorp {
                K1ComponentQualified::Scorp {
                    current_income: 0,
                    ordinary: ScorpLossQbi::default(),
                    property: ScorpPropertyQbi {
                        gain: 0,
                        qualified_1250: Some(0),
                        losses: ScorpLossQbi::default(),
                        complete: true,
                    },
                }
            } else {
                K1ComponentQualified::Partnership {
                    current_income: 0,
                    current_property: 0,
                    records: crate::partnership_qbi::PartnerQbiRecords {
                        qualified_se_gross: 0,
                        history: Some(PartnerQbiHistory {
                            basis: Vec::new(),
                            risk: Vec::new(),
                            passive: Vec::new(),
                        }),
                    },
                }
            },
        }]);
    }
}

/// Actual cash advanced to the existing loan, explicitly assigned to activity B.
pub(crate) fn scorp_cash_advance(value: &mut ScorpActivityK1, advance: i64) {
    value.entity.loans[0].regular.advances = advance;
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.risk_groups[1].investment_parts.as_mut().unwrap().loan_advances = advance;
}

/// Actual principal repayment assigned to B; formal notes retain their source term.
pub(crate) fn scorp_cash_repayment(value: &mut ScorpActivityK1, repayment: i64) {
    let loan = &mut value.entity.loans[0].regular;
    loan.repayments = repayment;
    loan.repayment_term = if repayment == 0 {
        ScorpNoteHolding::NoRepayment
    } else {
        ScorpNoteHolding::Single { term: GainTerm::Long }
    };
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.risk_groups[1].investment_parts.as_mut().unwrap().loan_repayments = repayment;
}

/// Two actual reduced repaid notes whose source cannot determine restoration priority.
pub(crate) fn scorp_repayment_priority(value: &mut ScorpActivityK1, alternative_only: bool) {
    value.entity.loans[0].regular.basis = if alternative_only { 500_000 } else { 100_000 };
    value.entity.loans[0].alternative_basis = Some(100_000);
    value.entity.loans.push(value.entity.loans[0].clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.risk_groups[1].investment_parts.as_mut().unwrap().loan_repayments = 200_000;
    // $5,000 current income less $1,000 retained basis loss gives the
    // existing Form7203 owner $4,000, against two $4,000 repaid deficits.
    if alternative_only {
        value.alternative_current.as_mut().unwrap().ordinary = 500_000;
        let K1StatementAmt::Separate { amounts } = &mut source.components[1].alternative else {
            unreachable!()
        };
        amounts.operating = 100_000;
    } else {
        value.current.ordinary = 500_000;
        source.components[1].regular.as_mut().unwrap().operating = 100_000;
    }
}

/// Independent outside-all-activities tax-exempt source controls.
pub(crate) fn entity_exempt_other_fields(
    income: &mut FederalIncome,
) -> (&mut K1ActivitySources, &mut K1CurrentStatementTotals, &mut Option<K1CurrentStatementTotals>)
{
    let (activities, current, alternative) = match income {
        FederalIncome::PartnershipActivities { data } => {
            (&mut data.activities, &mut data.current, &mut data.alternative_current)
        }
        FederalIncome::ScorpActivities { data } => {
            (&mut data.activities, &mut data.current, &mut data.alternative_current)
        }
        _ => unreachable!(),
    };
    let K1ComponentStatement::Entered { source } = activities else { unreachable!() };
    (source, current, alternative)
}

pub(crate) fn set_entity_exempt_other(income: &mut FederalIncome, regular: i64, amt: i64) {
    let (source, current, alternative) = entity_exempt_other_fields(income);
    let amounts = source.entity_only.as_mut().unwrap();
    amounts.exempt_other = regular;
    let mut separate = amounts.clone();
    separate.exempt_other = amt;
    source.alternative_entity_only = K1EntityOnlyAmt::Separate { amounts: separate };
    current.exempt_other = regular;
    alternative.as_mut().unwrap().exempt_other = amt;
}

pub(crate) fn set_entity_exempt_interest(income: &mut FederalIncome, regular: i64, amt: i64) {
    let (source, current, alternative) = entity_exempt_other_fields(income);
    source.entity_only.as_mut().unwrap().exempt_interest = regular;
    let amounts = match &mut source.alternative_entity_only {
        K1EntityOnlyAmt::Separate { amounts } => amounts,
        _ => unreachable!("fixture keeps independent AMT source amounts"),
    };
    amounts.exempt_interest = amt;
    current.exempt_interest = regular;
    alternative.as_mut().unwrap().exempt_interest = amt;
}

pub(crate) fn entity_exempt_other_output(
    income: &FederalIncome,
    amt: bool,
) -> Result<crate::k1_activity::ordinary::Output, crate::k1_activity::ordinary::SourceError> {
    match income {
        FederalIncome::PartnershipActivities { data } => data.ordinary_source(amt),
        FederalIncome::ScorpActivities { data } => data.ordinary_source(amt),
        _ => unreachable!(),
    }
}

pub(crate) fn entity_exempt_other_rule(income: &FederalIncome) -> Option<&'static str> {
    crate::k1_activity::income_issue(income).map(|issue| issue.key)
}

/// Reuse the native ordinary fixtures, retaining their accepted source identities.
pub(crate) fn configure_entity_exempt_other(income: &mut FederalIncome) {
    let scorp = matches!(income, FederalIncome::ScorpActivities { .. });
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(100_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(50_000);
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(100_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(50_000);
            data.entity.loans.clear();
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_exempt_other_fields(income);
    current.ordinary = -400_000;
    *alternative = Some(current.clone());
    for component in &mut source.components {
        let mut values = amounts(-200_000, 0);
        if scorp {
            values.items = scorp_items();
            values.capital_carry = scorp_carry();
        }
        component.regular = Some(values);
        component.alternative = K1StatementAmt::SameAsRegular;
    }
    for block in &mut source.qbi_blocks {
        block.allocations = Some(vec![]);
    }
    for group in &mut source.risk_groups {
        match &mut group.source {
            K1RiskSource::Partnership { regular, alternative } => {
                regular.start =
                    ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
                regular.prior_loss = 0;
                *alternative = Some(regular.clone());
            }
            K1RiskSource::Scorp { regular, alternative } => {
                regular.start =
                    ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
                regular.prior_loss = 0;
                *alternative = Some(regular.clone());
            }
            _ => unreachable!(),
        }
    }
    for group in &mut source.passive_groups {
        match &mut group.participation {
            K1ActivityParticipation::Partnership { facts, .. } => {
                *facts = PartnerParticipation::Limited {
                    minutes: 30_001,
                    prior_ten: 0,
                    personal_service: false,
                    service_years: 0,
                };
            }
            K1ActivityParticipation::Scorp { facts, .. } => {
                facts.minutes = 30_001;
            }
            _ => unreachable!(),
        }
    }
    set_entity_exempt_other(income, 100_000, 100_000);
    assert_eq!(entity_exempt_other_rule(income), None);
}

/// The retained passive source has an actual original-year zero-qualified partition.
pub(crate) fn entity_exempt_other_passive_history(income: &mut FederalIncome) {
    let scorp = matches!(income, FederalIncome::ScorpActivities { .. });
    let (source, _, _) = entity_exempt_other_fields(income);
    for (component, block) in source.components.iter_mut().zip(&mut source.qbi_blocks) {
        component.regular.as_mut().unwrap().passive_operating = Some(20_000);
        let passive = vec![loss_year(2025, 200, 0, 200, 0)];
        let qualified = if scorp {
            K1ComponentQualified::Scorp {
                current_income: 0,
                ordinary: ScorpLossQbi { current: 0, history: vec![], at_risk: vec![], passive },
                property: ScorpPropertyQbi::default(),
            }
        } else {
            K1ComponentQualified::Partnership {
                current_income: 0,
                current_property: 0,
                records: crate::partnership_qbi::PartnerQbiRecords {
                    qualified_se_gross: 0,
                    history: Some(PartnerQbiHistory { basis: vec![], risk: vec![], passive }),
                },
            }
        };
        block.allocations = Some(vec![K1QbiAllocation { component: component.id, qualified }]);
    }
}

/// Actual component-attributed exempt sources, retaining independently selected AMT parts.
pub(crate) fn set_component_exempt_other(
    income: &mut FederalIncome,
    regular: [i64; 2],
    amt: [i64; 2],
) {
    let (source, current, alternative) = entity_exempt_other_fields(income);
    for (index, component) in source.components.iter_mut().enumerate() {
        let values = component.regular.as_mut().unwrap();
        values.exempt_other = regular[index];
        let mut separate = values.clone();
        separate.exempt_other = amt[index];
        component.alternative = K1StatementAmt::Separate { amounts: separate };
    }
    current.exempt_other =
        regular.iter().sum::<i64>() + source.entity_only.as_ref().unwrap().exempt_other;
    let K1EntityOnlyAmt::Separate { amounts } = &source.alternative_entity_only else {
        unreachable!("fixture retains independent AMT entity-only amounts")
    };
    alternative.as_mut().unwrap().exempt_other = amt.iter().sum::<i64>() + amounts.exempt_other;
}

/// Reuse the entity-only fixture's accepted identities; only the actual source attribution changes.
pub(crate) fn configure_component_exempt_other(income: &mut FederalIncome) {
    set_entity_exempt_other(income, 0, 0);
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(200_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(200_000);
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(200_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(200_000);
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_exempt_other_fields(income);
    current.ordinary = -200_000;
    alternative.as_mut().unwrap().ordinary = -200_000;
    for component in &mut source.components {
        component.regular.as_mut().unwrap().operating = -100_000;
    }
    for group in &mut source.risk_groups {
        match &mut group.source {
            K1RiskSource::Partnership { regular, alternative } => {
                regular.start = ScorpRiskStart::Detailed { capacity: 25_000, profit: 0, loss: 0 };
                *alternative = Some(regular.clone());
            }
            K1RiskSource::Scorp { regular, alternative } => {
                regular.start = ScorpRiskStart::Detailed { capacity: 25_000, profit: 0, loss: 0 };
                *alternative = Some(regular.clone());
            }
            _ => unreachable!(),
        }
    }
    set_component_exempt_other(income, [50_001, 0], [30_001, 0]);
}

/// Current nonpassive §179 source, with explicit zero-qualified statement facts.
pub(crate) fn configure_section179(income: &mut FederalIncome, regular: [i64; 2], amt: [i64; 2]) {
    let scorp = matches!(income, FederalIncome::ScorpActivities { .. });
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(1_000_000_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(1_000_000_000);
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(1_000_000_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(1_000_000_000);
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_exempt_other_fields(income);
    source.entity_only.as_mut().unwrap().exempt_other = 0;
    source.alternative_entity_only = K1EntityOnlyAmt::SameAsRegular;
    current.exempt_other = 0;
    current.ordinary = 200_000;
    current.section179 = regular.into_iter().sum();
    *alternative = Some(current.clone());
    alternative.as_mut().unwrap().section179 = amt.into_iter().sum();
    for (index, component) in source.components.iter_mut().enumerate() {
        let mut values = amounts(100_000, 0);
        if scorp {
            values.items = scorp_items();
            values.capital_carry = scorp_carry();
        }
        values.section179 = regular[index];
        component.regular = Some(values.clone());
        values.section179 = amt[index];
        component.alternative = K1StatementAmt::Separate { amounts: values };
    }
    for group in &mut source.risk_groups {
        match &mut group.source {
            K1RiskSource::Partnership { regular, alternative } => {
                regular.start =
                    ScorpRiskStart::Detailed { capacity: 1_000_000_000, profit: 0, loss: 0 };
                *alternative = Some(regular.clone());
            }
            K1RiskSource::Scorp { regular, alternative } => {
                regular.start =
                    ScorpRiskStart::Detailed { capacity: 1_000_000_000, profit: 0, loss: 0 };
                *alternative = Some(regular.clone());
            }
            _ => unreachable!(),
        }
    }
}

pub(crate) fn before_section179(
    income: &FederalIncome,
    amt: bool,
) -> Result<crate::k1_activity::ordinary::Output, crate::k1_activity::ordinary::SourceError> {
    match income {
        FederalIncome::PartnershipActivities { data } => data.before_section179(amt),
        FederalIncome::ScorpActivities { data } => data.before_section179(amt),
        _ => unreachable!(),
    }
}

/// Huge legal current source whose actual emitted basis allowance is explicit.
pub(crate) fn section179_huge_source(income: &mut FederalIncome, opening: i64) {
    let gross = i64::MAX / 2 + 1;
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(opening);
            data.entity.alternative.as_mut().unwrap().opening = Some(opening);
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_exempt_other_fields(income);
    current.ordinary = 0;
    current.section179 = gross;
    *alternative = Some(current.clone());
    for (index, component) in source.components.iter_mut().enumerate() {
        let values = component.regular.as_mut().unwrap();
        values.operating = 0;
        values.section179 = if index == 0 { gross } else { 0 };
        component.alternative = K1StatementAmt::SameAsRegular;
    }
}
