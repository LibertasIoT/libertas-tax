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
            disposition: Some(crate::K1Disposition::Kept),
            sale: crate::PartnerSaleParts::default(),
            schedule_k3: None,
            credits: Vec::new(),
            investment_property: Vec::new(),
            investment_confirmed: None,
            housing_confirmed: None,
        },
        current: current(),
        amt: Vec::new(),
        depletion_sources: None,
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
                disposition: Some(K1Disposition::Kept),
                schedule_k3: None,
                credits: Vec::new(),
                investment_property: Vec::new(),
                investment_confirmed: None,
                housing_confirmed: None,
            },
            current: partner.current,
            amt: partner.amt,
            depletion_sources: partner.depletion_sources,
            alternative_current: None,
            activities: partner.activities,
            person: None,
            unentered: Vec::new(),
            special: Vec::new(),
            health_insurance: None,
        },
    }
}
pub(crate) fn input(value: &FederalIncome) -> crate::K1Document {
    match value {
        FederalIncome::PartnershipActivities { data } => {
            crate::K1Document::K1PartnershipActivities(alloc::boxed::Box::new(
                K1PartnershipActivitiesPage::from_record("Synthetic actual partnership", data),
            ))
        }
        FederalIncome::ScorpActivities { data } => crate::K1Document::K1ScorpActivities(
            K1ScorpActivitiesPage::from_record("Synthetic actual corporation", data),
        ),
        _ => unreachable!(),
    }
}
pub(crate) fn expected_input(
    draft: &FederalDraft,
    entry: &FederalIncomeEntry,
) -> crate::K1Document {
    let expected = crate::tests::support::builders::expected_named_source(
        draft,
        entry.owner.owned(),
        IncomeSource::K1 { value: input(&entry.income) },
    );
    let IncomeSource::K1 { value } = expected else { unreachable!() };
    value
}

pub(crate) fn expected_fixture_input(
    draft: &FederalDraft,
    value: &FederalIncome,
) -> crate::K1Document {
    expected_owned_fixture_input(draft, Owner::Taxpayer, value)
}

pub(crate) fn expected_owned_fixture_input(
    draft: &FederalDraft,
    owner: Owner,
    value: &FederalIncome,
) -> crate::K1Document {
    let expected = crate::tests::support::builders::expected_named_source(
        draft,
        owner,
        IncomeSource::K1 { value: input(value) },
    );
    let IncomeSource::K1 { value } = expected else { unreachable!() };
    value
}
pub(crate) fn page(value: &mut crate::K1Document) -> &mut K1ComponentDetailsPage {
    match value {
        crate::K1Document::K1PartnershipActivities(v) => &mut v.activities,
        crate::K1Document::K1ScorpActivities(v) => &mut v.activities,
        _ => unreachable!(),
    }
}
pub(crate) fn graph(value: &mut crate::K1Document) -> &mut K1ActivitySourcesPage {
    let K1ComponentStatementPage::Entered { source } = &mut page(value).statement else {
        unreachable!()
    };
    source
}
pub(crate) fn source_row(id: i64, document: crate::K1Document) -> crate::K1DocumentRow {
    crate::K1DocumentRow {
        id,
        document,
        owner: crate::DocumentOwner::Yours,
        community: CommunityShare::NotNeeded,
        refund_management: None,
        allowed_community: Vec::new(),
        preview: None,
        investment_caps: None,
        refund_management_needed: None,
    }
}
pub(crate) fn documents(request: &mut P) -> &mut Vec<crate::K1DocumentRow> {
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    &mut documents.common.k1.documents
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
pub(crate) fn bind_properties(document: &mut crate::K1Document) {
    let ids: Vec<_> = match document {
        crate::K1Document::K1PartnershipActivities(v) => {
            v.entity.investment_property.iter().map(crate::K1InvestmentProperty::id).collect()
        }
        crate::K1Document::K1ScorpActivities(v) => {
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
    document: &mut crate::K1Document,
) -> (&mut Vec<K1Section179Sale>, &mut Option<K1EntityTaxPeriod>) {
    match document {
        crate::K1Document::K1PartnershipActivities(value) => {
            (&mut value.section179_sales, &mut value.reporting_period)
        }
        crate::K1Document::K1ScorpActivities(value) => {
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
pub(crate) fn add_sale_sources(document: &mut crate::K1Document) {
    let (sales, period) = sale_fields(document);
    sales.push(current_sale());
    *period = Some(K1EntityTaxPeriod { began: 20250701, ended: 20260630 });
}
pub(crate) fn bind_sales(document: &mut crate::K1Document) {
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
    document: &mut crate::K1Document,
) -> &mut Option<Section179IncomeCarryHistory> {
    match document {
        crate::K1Document::K1PartnershipActivities(value) => {
            &mut value.entity.amt_section179_income_carry
        }
        crate::K1Document::K1ScorpActivities(value) => {
            &mut value.entity.amt_section179_income_carry
        }
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
        }
        .into(),
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

/// Actual positive current sources and independent original-year basis/risk qualification.
pub(crate) fn ordinary_multi_current_retained_qbi_parent(
    incomes: [i64; 2],
    statements: [Option<(i64, i64)>; 2],
) -> PartnerActivityK1 {
    let mut value = ordinary_multi_current_qbi_parent_with_statements([None, None]);
    value.current.ordinary = incomes.iter().sum();
    value.alternative_current = Some(value.current.clone());
    value.entity.regular.opening = Some(0);
    value.entity.alternative = Some(value.entity.regular.clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for (index, component) in source.components.iter_mut().enumerate() {
        let amounts = component.regular.as_mut().unwrap();
        amounts.operating = incomes[index];
        if index == 0 {
            amounts.operating_carry =
                K1RetainedLossCarry { basis: Some(100_000), risk: Some(100_000) };
        }
        component.alternative = K1StatementAmt::SameAsRegular;
        component.qualified_current_controls.ordinary =
            Some(K1CurrentSourceParts { income: incomes[index], loss: 0 });
        let block = &mut source.qbi_blocks[index];
        block.statement = K1ReportedQbiStatement::Partnership {
            statement: statements[index].map(|(income, wages)| PartnerQbiStatement {
                income,
                wages,
                property: 0,
                service: false,
                section1231_qbi: 0,
                reit_dividends: 0,
                ptp_income: 0,
            }),
        };
        let K1ComponentQualified::Partnership { current_income, records, .. } =
            &mut block.allocations.as_mut().unwrap()[0].qualified
        else {
            unreachable!()
        };
        *current_income = statements[index].map_or(0, |(income, _)| income);
        records.history = (index == 0).then(|| PartnerQbiHistory {
            basis: vec![loss_year(2025, 1_000, 500, 1_000, 500)],
            risk: vec![loss_year(2024, 1_000, 800, 1_000, 800)],
            passive: vec![],
        });
    }
    let K1RiskSource::Partnership { regular, alternative } = &mut source.risk_groups[0].source
    else {
        unreachable!()
    };
    regular.prior_loss = 100_000;
    *alternative = Some(regular.clone());
    assert_eq!(value.rule(), None);
    value
}

pub(crate) fn partnership_qualified_history(
    qualified: &mut K1ComponentQualified,
) -> &mut PartnerQbiHistory {
    let K1ComponentQualified::Partnership { records, .. } = qualified else { unreachable!() };
    records.history.as_mut().unwrap()
}

pub(crate) fn partner_activity_graph(value: &mut PartnerActivityK1) -> &mut K1ActivitySources {
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source
}

pub(crate) fn partner_group_material(source: &mut K1ActivitySources, material: bool) {
    for group in &mut source.passive_groups {
        group.participation = K1ActivityParticipation::Partnership {
            facts: PartnerParticipation::Limited {
                minutes: if material { 30_001 } else { 0 },
                prior_ten: 0,
                personal_service: false,
                service_years: 0,
            },
            trade_kind: ScorpTradeKind::Ordinary,
        };
    }
}

pub(crate) fn ordinary_multi_passive_retained_qbi_parent(
    incomes: [i64; 2],
    statements: [Option<(i64, i64)>; 2],
) -> PartnerActivityK1 {
    let mut value = ordinary_multi_current_retained_qbi_parent(incomes, statements);
    partner_group_material(partner_activity_graph(&mut value), false);
    value
}

// An actual opening at-risk amount permits both $1,000 histories to pass even
// when component A has only $1,000 current income. B retains its separate group.
pub(crate) fn partner_retained_opening_risk(source: &mut K1ActivitySources) {
    let K1RiskSource::Partnership { regular, alternative } = &mut source.risk_groups[0].source
    else {
        unreachable!()
    };
    regular.start = ScorpRiskStart::Detailed { capacity: 200_000, profit: 0, loss: 0 };
    *alternative = Some(regular.clone());
}

/// Forward actual statement income and W-2 wages in cents to the existing native sources.
pub(crate) fn ordinary_multi_current_qbi_parent_with_statements(
    statements: [Option<(i64, i64)>; 2],
) -> PartnerActivityK1 {
    ordinary_multi_current_partner_qbi_source_with_statements(ordinary_multi_parent(), statements)
}

pub(crate) fn ordinary_multi_current_partner_qbi_source(
    value: PartnerActivityK1,
) -> PartnerActivityK1 {
    ordinary_multi_current_partner_qbi_source_with_statements(value, [Some((300_000, 0)), None])
}

fn ordinary_multi_current_partner_qbi_source_with_statements(
    mut value: PartnerActivityK1,
    statements: [Option<(i64, i64)>; 2],
) -> PartnerActivityK1 {
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

// Actual native source cents, independent of any expected federal result.
// Zero qualified amounts select the native NoStatement representation.
pub(crate) struct CurrentLossQbiFacts {
    pub gross_loss: i64,
    pub qualified: [i64; 2],
    pub opening_basis: i64,
    pub loss_risk_capacity: i64,
}
impl Default for CurrentLossQbiFacts {
    fn default() -> Self {
        Self {
            gross_loss: 200_000,
            qualified: [400_000, -150_000],
            opening_basis: 1_000_000,
            loss_risk_capacity: 1_000_000,
        }
    }
}

// Reuse current-source construction for explicit material participation and
// established zero retained histories. Keep every existing source identity/link.
pub(crate) fn current_loss_partner_qbi_source(
    value: PartnerActivityK1,
    facts: &CurrentLossQbiFacts,
) -> PartnerActivityK1 {
    let mut value = ordinary_multi_current_partner_qbi_source(value);
    value.current.ordinary = 400_000 - facts.gross_loss;
    value.alternative_current = Some(value.current.clone());
    value.entity.regular.opening = Some(facts.opening_basis);
    value.entity.alternative = Some(value.entity.regular.clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    current_loss_qbi_sources(source, facts);
    value
}

pub(crate) fn current_loss_scorp_qbi_source(
    value: ScorpActivityK1,
    facts: &CurrentLossQbiFacts,
) -> ScorpActivityK1 {
    let mut value = current_scorp_qbi_source(value);
    value.current.ordinary = 400_000 - facts.gross_loss;
    value.alternative_current = Some(value.current.clone());
    value.entity.regular.opening = Some(facts.opening_basis);
    value.entity.alternative = Some(value.entity.regular.clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    current_loss_qbi_sources(source, facts);
    value
}

fn current_loss_qbi_sources(source: &mut K1ActivitySources, facts: &CurrentLossQbiFacts) {
    let component = &mut source.components[1];
    let regular = component.regular.as_mut().unwrap();
    regular.operating = -facts.gross_loss;
    component.alternative = K1StatementAmt::Separate { amounts: regular.clone() };
    component.qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 0, loss: facts.gross_loss });
    for (index, group) in source.risk_groups.iter_mut().enumerate() {
        let start = ScorpRiskStart::Detailed {
            capacity: if index == 0 { 1_000_000 } else { facts.loss_risk_capacity },
            profit: 0,
            loss: 0,
        };
        match &mut group.source {
            K1RiskSource::Partnership { regular, alternative } => {
                regular.start = start;
                *alternative = Some(regular.clone());
            }
            K1RiskSource::Scorp { regular, alternative } => {
                regular.start = start;
                *alternative = Some(regular.clone());
            }
            _ => unreachable!(),
        }
    }
    for (index, block) in source.qbi_blocks.iter_mut().enumerate() {
        let qualified = facts.qualified[index];
        let allocation = &mut block.allocations.as_mut().unwrap()[0];
        match (&mut block.statement, &mut allocation.qualified) {
            (
                K1ReportedQbiStatement::Partnership { statement },
                K1ComponentQualified::Partnership { current_income, .. },
            ) => {
                *current_income = qualified;
                *statement = (qualified != 0).then_some(PartnerQbiStatement {
                    income: qualified,
                    wages: 0,
                    property: 0,
                    service: false,
                    section1231_qbi: 0,
                    reit_dividends: 0,
                    ptp_income: 0,
                });
            }
            (
                K1ReportedQbiStatement::Scorp { statement },
                K1ComponentQualified::Scorp { current_income, ordinary, .. },
            ) => {
                *current_income = qualified.max(0);
                ordinary.current = (-qualified).max(0);
                *statement = (qualified != 0).then_some(K1ScorpBlockStatement {
                    income: qualified,
                    section1231_qbi: 0,
                    wages: 0,
                    property: 0,
                    service: false,
                    reit_dividends: 0,
                    ptp_income: 0,
                });
            }
            _ => unreachable!(),
        }
    }
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

/// Actual outside-all-activities interest, with existing accepted native IDs.
pub(crate) fn scorp_entity_interest_source(
    value: ScorpActivityK1,
    interest: i64,
    losses: [i64; 2],
    capacity: i64,
    passive_history: i64,
) -> ScorpActivityK1 {
    let mut value = current_scorp_qbi_source_with_statements(value, [None, None]);
    value.entity.regular.opening = Some(100_000);
    value.entity.alternative.as_mut().unwrap().opening = Some(50_000);
    value.current.ordinary = -losses.iter().sum::<i64>();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for (index, component) in source.components.iter_mut().enumerate() {
        let mut regular = amounts(-losses[index], 0);
        regular.items = scorp_items();
        regular.capital_carry = scorp_carry();
        regular.passive_operating = Some(if index == 1 { passive_history } else { 0 });
        component.regular = Some(regular);
        component.alternative = K1StatementAmt::SameAsRegular;
        component.qualified_current_controls.ordinary =
            Some(K1CurrentSourceParts { income: 0, loss: losses[index] });
        let K1RiskSource::Scorp { regular, alternative } = &mut source.risk_groups[index].source
        else {
            unreachable!()
        };
        regular.start = ScorpRiskStart::Detailed { capacity, profit: 0, loss: 0 };
        *alternative = Some(regular.clone());
        if index == 1 && passive_history != 0 {
            let K1ActivityParticipation::Scorp { facts, .. } =
                &mut source.passive_groups[index].participation
            else {
                unreachable!()
            };
            facts.minutes = 0;
            let K1ComponentQualified::Scorp { ordinary, .. } =
                &mut source.qbi_blocks[index].allocations.as_mut().unwrap()[0].qualified
            else {
                unreachable!()
            };
            let dollars = crate::kernel::ratio::dollars(passive_history);
            ordinary.passive = vec![loss_year(2025, dollars, 0, dollars, 0)];
        }
    }
    set_scorp_entity_interest(&mut value, interest);
    value
}

pub(crate) fn set_scorp_entity_interest(value: &mut ScorpActivityK1, interest: i64) {
    value.current.portfolio.interest = interest;
    value.alternative_current = Some(value.current.clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.entity_only.as_mut().unwrap().portfolio.interest = interest;
    source.alternative_entity_only = K1EntityOnlyAmt::SameAsRegular;
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

/// Borrow the actual native source and its independent printed controls.
pub(crate) fn entity_dividend_parts(
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

/// Reuse the original multi-activity source, with explicit clean or retained losses.
pub(crate) fn prepare_entity_dividends(income: &mut FederalIncome, losses: bool) {
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(100_000);
            data.entity.alternative = Some(data.entity.regular.clone());
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(100_000);
            data.entity.alternative = Some(data.entity.regular.clone());
            data.entity.loans.clear();
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_dividend_parts(income);
    for group in &mut source.risk_groups {
        match &mut group.source {
            K1RiskSource::Partnership { regular, alternative } => {
                regular.start = ScorpRiskStart::Detailed { capacity: 50_000, profit: 0, loss: 0 };
                regular.deducted = 0;
                regular.recaptured = 0;
                if !losses {
                    regular.prior_loss = 0;
                }
                *alternative = Some(regular.clone());
            }
            K1RiskSource::Scorp { regular, alternative } => {
                regular.start = ScorpRiskStart::Detailed { capacity: 50_000, profit: 0, loss: 0 };
                regular.deducted = 0;
                regular.recaptured = 0;
                if !losses {
                    regular.prior_loss = 0;
                }
                *alternative = Some(regular.clone());
            }
            _ => unreachable!(),
        }
    }
    for component in &mut source.components {
        let amounts = component.regular.as_mut().unwrap();
        if !losses {
            amounts.operating = 0;
            amounts.operating_carry = K1RetainedLossCarry { basis: Some(0), risk: Some(0) };
            amounts.passive_operating = Some(0);
            component.qualified_current_controls.ordinary =
                Some(K1CurrentSourceParts { income: 0, loss: 0 });
        }
        component.alternative = K1StatementAmt::Separate { amounts: amounts.clone() };
    }
    if !losses {
        current.ordinary = 0;
        for block in &mut source.qbi_blocks {
            block.allocations = Some(vec![]);
        }
    }
    *alternative = Some(current.clone());
    assert!(crate::k1_activity::income_issue(income).is_none());
}

/// Both printed controls reconcile to actual entity-only source cents.
pub(crate) fn set_entity_interest(income: &mut FederalIncome, regular: i64, amt: i64) {
    let (source, current, alternative) = entity_dividend_parts(income);
    source.entity_only.as_mut().unwrap().portfolio.interest = regular;
    let mut amounts = match &source.alternative_entity_only {
        K1EntityOnlyAmt::Separate { amounts } => amounts.clone(),
        K1EntityOnlyAmt::SameAsRegular => source.entity_only.as_ref().unwrap().clone(),
        K1EntityOnlyAmt::Unavailable => unreachable!(),
    };
    amounts.portfolio.interest = amt;
    source.alternative_entity_only = K1EntityOnlyAmt::Separate { amounts };
    current.portfolio.interest = regular;
    alternative.as_mut().unwrap().portfolio.interest = amt;
}

pub(crate) fn set_entity_dividends(
    income: &mut FederalIncome,
    regular: [i64; 2],
    alternative: Option<[i64; 2]>,
) {
    let (source, current, amt_current) = entity_dividend_parts(income);
    let values = source.entity_only.as_mut().unwrap();
    values.portfolio.dividends = regular[0];
    values.portfolio.qualified_dividends = regular[1];
    current.portfolio.dividends = regular[0];
    current.portfolio.qualified_dividends = regular[1];
    let amt = alternative.unwrap_or(regular);
    let mut amt_source = match &source.alternative_entity_only {
        K1EntityOnlyAmt::Separate { amounts } => amounts.clone(),
        _ => values.clone(),
    };
    amt_source.portfolio.dividends = amt[0];
    amt_source.portfolio.qualified_dividends = amt[1];
    source.alternative_entity_only = if alternative.is_some()
        || matches!(source.alternative_entity_only, K1EntityOnlyAmt::Separate { .. })
    {
        K1EntityOnlyAmt::Separate { amounts: amt_source }
    } else {
        K1EntityOnlyAmt::SameAsRegular
    };
    let amt_current = amt_current.as_mut().unwrap();
    amt_current.portfolio.dividends = amt[0];
    amt_current.portfolio.qualified_dividends = amt[1];
}

pub(crate) fn entity_dividend_output(
    income: &FederalIncome,
    alternative: bool,
) -> Result<crate::k1_activity::ordinary::Output, crate::k1_activity::ordinary::SourceError> {
    match income {
        FederalIncome::PartnershipActivities { data } => data.ordinary_source(alternative),
        FederalIncome::ScorpActivities { data } => data.ordinary_source(alternative),
        _ => unreachable!(),
    }
}

pub(crate) fn entity_dividend_index(draft: &FederalDraft) -> usize {
    draft
        .income
        .iter()
        .position(|entry| {
            matches!(
                &entry.income,
                FederalIncome::PartnershipActivities { .. } | FederalIncome::ScorpActivities { .. }
            )
        })
        .unwrap()
}

/// Existing portfolio borrowing uses the late Form4952 owner (no royalty link).
pub(crate) fn entity_dividend_borrowing(draft: &mut FederalDraft) {
    let deductions = draft.deductions.as_mut().unwrap();
    itemized(deductions).investment_income_tax = InvestmentIncomeTax::None;
    deductions.itemize_override = Some(ItemizeOverride::Itemize);
    deductions.investment_interest = Some(InvestmentBorrowing {
        loans: vec![native_investment_loan("Synthetic portfolio borrowing".into(), 300_000, 0, 0)],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Ordinary,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
}

/// Extend the shared empty fiduciary source with a reconciled qualified adjustment.
pub(crate) fn entity_dividend_estate(qualified: i64) -> FederalEstateTrustPortfolioK1 {
    let mut data = empty_estate_portfolio();
    data.ordinary_dividends = 100_000;
    data.qualified_dividends = qualified;
    data.amt_box12a = 10_000;
    data.amt_box12b = 10_000;
    data.amt_box12_components_confirmed = true;
    data.amt_reconciliation = Some(EstateAmtReconciliation { other_in_a: 0, adjustments: vec![] });
    data.investment_income = Some(EstateInvestmentIncome {
        regular: 100_000,
        alternative: 110_000,
        character: EstateInvestmentCharacter::Preferential {
            qualified,
            gains: 0,
            amt_qualified: qualified + 10_000,
            amt_gains: 0,
        },
    });
    assert!(data.valid());
    assert_eq!(crate::estate_trust::source_rule(&data), None);
    assert_eq!(crate::estate_trust::investment_adjustment(&data), Some([0; 2]));
    data
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
    let mut separate = match &source.alternative_entity_only {
        K1EntityOnlyAmt::Separate { amounts } => amounts.clone(),
        _ => amounts.clone(),
    };
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

/// Actual independent operating, net SE and gross SE source cents.
/// Reuse the fully allowed graph and its explicit empty histories.
pub(crate) fn ordinary_positive_se_parent(
    operating: [i64; 2],
    reported: [i64; 2],
    gross: [i64; 2],
) -> PartnerActivityK1 {
    let mut value = ordinary_multi_current_qbi_parent();
    value.current.ordinary = operating.into_iter().sum();
    value.alternative_current = Some(value.current.clone());
    let person = value.person.as_mut().unwrap();
    person.limited_for_se = Some(false);
    person.reported_se = Some(reported.into_iter().sum());
    person.se_gross = Some(vec![PartnerSeGross::Nonfarm { gross: gross.into_iter().sum() }]);
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for (index, component) in source.components.iter_mut().enumerate() {
        component.regular = Some(amounts(operating[index], 0));
        component.alternative = K1StatementAmt::SameAsRegular;
        component.qualified_current_controls.ordinary =
            Some(K1CurrentSourceParts { income: operating[index], loss: 0 });
        component.self_employment = K1StatementSe::Partnership {
            source: K1PartnerSeSource {
                reported: Some(reported[index]),
                history: K1RetainedSeHistory { basis: Some(0), risk: Some(0), passive: Some(0) },
                gross: Some(PartnerSeGross::Nonfarm { gross: gross[index] }),
                guaranteed_services: 0,
                guaranteed_capital: 0,
            },
        };
    }
    for block in &mut source.qbi_blocks {
        block.statement = K1ReportedQbiStatement::Partnership { statement: None };
        let K1ComponentQualified::Partnership { current_income, .. } =
            &mut block.allocations.as_mut().unwrap()[0].qualified
        else {
            unreachable!()
        };
        *current_income = 0;
    }
    assert_eq!(value.rule(), None);
    value
}

/// Distinct current taxable operations, net SE, actual SE gross, and QBI source parts.
pub(crate) fn ordinary_positive_se_qbi_parent() -> PartnerActivityK1 {
    let mut value =
        ordinary_positive_se_parent([400_000, 200_000], [120_000, 80_000], [900_000, 300_000]);
    let statements =
        ordinary_multi_current_qbi_parent_with_statements([Some((300_000, 0)), Some((100_000, 0))]);
    let K1ComponentStatement::Entered { source: statements } = statements.activities else {
        unreachable!()
    };
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.qbi_blocks = statements.qbi_blocks;
    for (block, gross) in source.qbi_blocks.iter_mut().zip([450_000, 75_000]) {
        let K1ComponentQualified::Partnership { records, .. } =
            &mut block.allocations.as_mut().unwrap()[0].qualified
        else {
            unreachable!()
        };
        records.qualified_se_gross = gross;
    }
    assert_eq!(value.rule(), None);
    value
}

/// Reuse the ordinary Schedule C fixture while supplying independent gross and net sources.
pub(crate) fn add_qbi_gross_business(draft: &mut FederalDraft, gross: i64, profit: i64) -> i64 {
    let mut source = cash_business(0);
    let FederalIncome::Business { data } = &mut source else { unreachable!() };
    let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity else { unreachable!() };
    *receipts = gross;
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Operating costs".into() },
        gross - profit,
    )];
    add(draft, income(source));
    draft.income.last().unwrap().id
}

// Existing current-source facts with explicit native participation and a distinct QBI amount.
pub(crate) fn ordinary_multi_passive_qbi_parent(
    qualified: i64,
    active: [bool; 2],
) -> PartnerActivityK1 {
    let mut value = ordinary_multi_current_qbi_parent();
    let K1ComponentStatement::Entered { source: component_source } = &mut value.activities else {
        unreachable!()
    };
    for (name, active) in ["Actual A", "Actual B"].into_iter().zip(active) {
        let group_id = component_source
            .components
            .iter()
            .find(|component| component.name == name)
            .unwrap()
            .passive_group
            .unwrap();
        let group =
            component_source.passive_groups.iter_mut().find(|group| group.id == group_id).unwrap();
        let K1ActivityParticipation::Partnership {
            facts: PartnerParticipation::Limited { minutes, .. },
            ..
        } = &mut group.participation
        else {
            unreachable!()
        };
        *minutes = if active { 30_001 } else { 0 };
    }
    let component_id = component_source
        .components
        .iter()
        .find(|component| component.name == "Actual A")
        .unwrap()
        .id;
    let block = component_source
        .qbi_blocks
        .iter_mut()
        .find(|block| {
            matches!(block.statement, K1ReportedQbiStatement::Partnership { statement: Some(_) })
        })
        .unwrap();
    let K1ReportedQbiStatement::Partnership { statement: Some(statement) } = &mut block.statement
    else {
        unreachable!()
    };
    statement.income = qualified;
    let allocation = block
        .allocations
        .as_mut()
        .unwrap()
        .iter_mut()
        .find(|allocation| allocation.component == component_id)
        .unwrap();
    let K1ComponentQualified::Partnership { current_income, .. } = &mut allocation.qualified else {
        unreachable!()
    };
    *current_income = qualified;
    assert_eq!(value.rule(), None);
    value
}

pub(crate) fn passive_scorp_qbi_source(
    value: ScorpActivityK1,
    qualified: i64,
    active: [bool; 2],
) -> ScorpActivityK1 {
    let mut value = current_scorp_qbi_source(value);
    let K1ComponentStatement::Entered { source: component_source } = &mut value.activities else {
        unreachable!()
    };
    for (name, active) in ["Actual A", "Actual B"].into_iter().zip(active) {
        let group_id = component_source
            .components
            .iter()
            .find(|component| component.name == name)
            .unwrap()
            .passive_group
            .unwrap();
        let group =
            component_source.passive_groups.iter_mut().find(|group| group.id == group_id).unwrap();
        let K1ActivityParticipation::Scorp { facts, .. } = &mut group.participation else {
            unreachable!()
        };
        facts.minutes = if active { 30_001 } else { 0 };
    }
    let component_id = component_source
        .components
        .iter()
        .find(|component| component.name == "Actual A")
        .unwrap()
        .id;
    let block = component_source
        .qbi_blocks
        .iter_mut()
        .find(|block| {
            matches!(block.statement, K1ReportedQbiStatement::Scorp { statement: Some(_) })
        })
        .unwrap();
    let K1ReportedQbiStatement::Scorp { statement: Some(statement) } = &mut block.statement else {
        unreachable!()
    };
    statement.income = qualified;
    let allocation = block
        .allocations
        .as_mut()
        .unwrap()
        .iter_mut()
        .find(|allocation| allocation.component == component_id)
        .unwrap();
    let K1ComponentQualified::Scorp { current_income, .. } = &mut allocation.qualified else {
        unreachable!()
    };
    *current_income = qualified;
    assert!(
        crate::k1_activity::income_issue(&FederalIncome::ScorpActivities { data: value.clone() })
            .is_none()
    );
    value
}

pub(crate) fn ordinary_multi_passive_scorp_qbi_parent(
    qualified: i64,
    active: [bool; 2],
) -> ScorpActivityK1 {
    passive_scorp_qbi_source(ordinary_multi_current_scorp_qbi_parent(), qualified, active)
}

/// Ordinary partnership sources with no retained basis/risk losses or qualified amounts.
/// Outside interest is separate from both groups' economic investment.
pub(crate) fn partner_entity_interest_source(
    interest: i64,
    operating: [i64; 2],
    capacity: i64,
    passive_history: i64,
) -> PartnerActivityK1 {
    let mut value = ordinary_multi_current_qbi_parent_with_statements([None, None]);
    value.entity.regular = PartnerEntityBasis {
        opening: Some(100_000),
        contributions: 0,
        opening_liabilities: 0,
        ending_liabilities: 0,
        assumed: 0,
        transferred: 0,
        distributions: 0,
        prior_nondeductible: Some(0),
        complete: true,
    };
    value.entity.alternative = Some(value.entity.regular.clone());
    value.entity.alternative.as_mut().unwrap().opening = Some(50_000);
    value.current.ordinary = operating.iter().sum();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for (index, component) in source.components.iter_mut().enumerate() {
        let mut regular = amounts(operating[index], 0);
        regular.passive_operating = Some(if index == 1 { passive_history } else { 0 });
        component.regular = Some(regular);
        component.alternative = K1StatementAmt::SameAsRegular;
        component.qualified_current_controls.ordinary = Some(K1CurrentSourceParts {
            income: operating[index].max(0),
            loss: (-operating[index]).max(0),
        });
        let mut risk = partner_risk_record();
        risk.start = ScorpRiskStart::Detailed { capacity, profit: 0, loss: 0 };
        source.risk_groups[index].source =
            K1RiskSource::Partnership { alternative: Some(risk.clone()), regular: risk };
        source.risk_groups[index].investment_parts = Some(K1RiskInvestmentParts {
            contributions: 0,
            distributions: 0,
            loan_advances: 0,
            loan_repayments: 0,
        });
        if index == 1 && passive_history != 0 {
            let K1ActivityParticipation::Partnership { facts, .. } =
                &mut source.passive_groups[index].participation
            else {
                unreachable!()
            };
            *facts = PartnerParticipation::Limited {
                minutes: 0,
                prior_ten: 0,
                personal_service: false,
                service_years: 0,
            };
            let K1ComponentQualified::Partnership { records, .. } =
                &mut source.qbi_blocks[index].allocations.as_mut().unwrap()[0].qualified
            else {
                unreachable!()
            };
            let dollars = crate::kernel::ratio::dollars(passive_history);
            records.history.as_mut().unwrap().passive =
                vec![loss_year(2025, dollars, 0, dollars, 0)];
        }
    }
    set_partner_entity_interest(&mut value, interest);
    value
}

pub(crate) fn set_partner_entity_interest(value: &mut PartnerActivityK1, interest: i64) {
    value.current.portfolio.interest = interest;
    value.alternative_current = Some(value.current.clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.entity_only.as_mut().unwrap().portfolio.interest = interest;
    source.alternative_entity_only = K1EntityOnlyAmt::SameAsRegular;
}

/// Farm code-B controls on the existing fully allowed positive-SE graph.
pub(crate) fn ordinary_farm_se_parent(
    operating: [i64; 2],
    reported: [i64; 2],
    gross: [i64; 2],
) -> PartnerActivityK1 {
    let mut value = ordinary_positive_se_parent(operating, reported, gross);
    value.person.as_mut().unwrap().se_gross =
        Some(vec![PartnerSeGross::Farm { gross: gross.into_iter().sum() }]);
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for (component, gross) in source.components.iter_mut().zip(gross) {
        let K1StatementSe::Partnership { source } = &mut component.self_employment else {
            unreachable!()
        };
        source.gross = Some(PartnerSeGross::Farm { gross });
    }
    assert_eq!(value.rule(), None);
    value
}

// Current investment interest with independent native basis and at-risk records.
pub(crate) fn k1_investment_interest_source(
    scorp: bool,
    paid: i64,
    basis: [i64; 2],
    risk: [i64; 2],
) -> FederalIncome {
    if scorp {
        let mut source = scorp_document();
        source.ordinary = 0;
        source.opening = Some(basis[0]);
        source.loans.clear();
        let mut participation = participation_facts();
        participation.minutes = 30_001;
        source.participation = Some(participation);
        source.risk.start = ScorpRiskStart::Detailed { capacity: risk[0], profit: 0, loss: 0 };
        source.deductions =
            vec![ScorpDeduction { code: ScorpDeductionCode::InvestmentInterest, amount: paid }];
        let mut alternative = amt_records(&source);
        alternative.stock = basis[1];
        alternative.risk.start = ScorpRiskStart::Detailed { capacity: risk[1], profit: 0, loss: 0 };
        source.amt = Some(alternative);
        FederalIncome::Scorp { data: scorp_source(source) }
    } else {
        let mut source = ordinary_partner_source(0, true);
        source.regular.basis.opening = Some(basis[0]);
        source.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: risk[0], profit: 0, loss: 0 };
        let mut alternative = source.regular.clone();
        alternative.basis.opening = Some(basis[1]);
        alternative.risk.start = ScorpRiskStart::Detailed { capacity: risk[1], profit: 0, loss: 0 };
        source.alternative = Some(alternative);
        source.deductions =
            vec![PartnerDeduction { code: PartnerDeductionCode::InvestmentInterest, amount: paid }];
        FederalIncome::Partnership { data: source }
    }
}

pub(crate) fn k1_investment_interest_return(
    scorp: bool,
    basis: [i64; 2],
    income_dollars: i64,
) -> FederalDraft {
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, wage(0, Owner::Taxpayer, 20_000_000, 0));
    add(&mut draft, income(savings_income(income_dollars, 0)));
    add(&mut draft, income(k1_investment_interest_source(scorp, 10_049, basis, [100_000; 2])));
    let deductions = draft.deductions.as_mut().unwrap();
    let facts = itemized(deductions).clone();
    {
        deductions.itemized = facts;
        deductions.itemize_override = Some(ItemizeOverride::Itemize);
    };
    draft
}

/// Reuse accepted source identities and the existing two independent activity groups.
pub(crate) fn prepare_entity_nondeductible(income: &mut FederalIncome, regular: i64, amt: i64) {
    configure_entity_exempt_other(income);
    set_entity_exempt_other(income, 0, 0);
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(150_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(150_000);
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(150_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(150_000);
            data.entity.regular.losses_first = false;
            data.entity.alternative.as_mut().unwrap().losses_first = false;
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_dividend_parts(income);
    for component in &mut source.components {
        component.regular.as_mut().unwrap().operating = -100_000;
        component.alternative = K1StatementAmt::SameAsRegular;
        component.qualified_current_controls.ordinary =
            Some(K1CurrentSourceParts { income: 0, loss: 100_000 });
    }
    current.ordinary = -200_000;
    *alternative = Some(current.clone());
    set_entity_nondeductible(income, regular, amt);
    assert!(crate::k1_activity::income_issue(income).is_none());
}

pub(crate) fn set_entity_nondeductible(income: &mut FederalIncome, regular: i64, amt: i64) {
    let (source, current, alternative) = entity_dividend_parts(income);
    source.entity_only.as_mut().unwrap().nondeductible = regular;
    let mut amounts = match &source.alternative_entity_only {
        K1EntityOnlyAmt::SameAsRegular => source.entity_only.as_ref().unwrap().clone(),
        K1EntityOnlyAmt::Separate { amounts } => amounts.clone(),
        K1EntityOnlyAmt::Unavailable => unreachable!(),
    };
    amounts.nondeductible = amt;
    source.alternative_entity_only = K1EntityOnlyAmt::Separate { amounts };
    current.nondeductible = regular;
    alternative.as_mut().unwrap().nondeductible = amt;
}

/// Actual outside-all-activities capital controls, preserving source cents.
pub(crate) fn set_entity_capital(income: &mut FederalIncome, regular: [i64; 2], amt: [i64; 2]) {
    let (source, current, alternative) = entity_exempt_other_fields(income);
    let value = source.entity_only.as_mut().unwrap();
    value.portfolio.short = regular[0];
    value.portfolio.long = regular[1];
    let mut separate = match &source.alternative_entity_only {
        K1EntityOnlyAmt::Separate { amounts } => amounts.clone(),
        _ => value.clone(),
    };
    separate.portfolio.short = amt[0];
    separate.portfolio.long = amt[1];
    source.alternative_entity_only = K1EntityOnlyAmt::Separate { amounts: separate };
    current.portfolio.short = regular[0];
    current.portfolio.long = regular[1];
    let alternative = alternative.as_mut().unwrap();
    alternative.portfolio.short = amt[0];
    alternative.portfolio.long = amt[1];
}

pub(crate) fn configure_entity_capital(income: &mut FederalIncome) {
    set_entity_exempt_other(income, 0, 0);
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(50_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(80_000);
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(50_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(80_000);
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_exempt_other_fields(income);
    for component in &mut source.components {
        component.regular.as_mut().unwrap().operating = -100_000;
    }
    current.ordinary = -200_000;
    alternative.as_mut().unwrap().ordinary = -200_000;
    set_entity_capital(income, [30_000, 70_000], [10_000, 30_000]);
}

pub(crate) fn zero_entity_capital_operating(income: &mut FederalIncome) {
    let (source, current, alternative) = entity_exempt_other_fields(income);
    for component in &mut source.components {
        component.regular.as_mut().unwrap().operating = 0;
    }
    current.ordinary = 0;
    alternative.as_mut().unwrap().ordinary = 0;
}

/// Retain an actual native loan identity; only its source-dollar controls differ.
pub(crate) fn entity_capital_debt(income: &mut FederalIncome, original: &FederalIncome) {
    let FederalIncome::ScorpActivities { data: original } = original else { unreachable!() };
    let FederalIncome::ScorpActivities { data } = income else { unreachable!() };
    let mut loans = scorp_ordinary_multi_baseline(original.clone()).entity.loans;
    let mut loan = loans.remove(0);
    loan.regular.principal = 50_000;
    loan.regular.basis = 50_000;
    loan.regular.advances = 0;
    loan.regular.repayments = 0;
    loan.alternative_basis = Some(50_000);
    data.entity.loans = vec![loan];
}

// Synthetic component capital sources. Every parameter is a source-cent scale:
// scale=1 exercises cents; scale=100 gives the corresponding dollar fixture.
pub(crate) fn component_capital_source(scorp: bool, scale: i64) -> FederalIncome {
    let partner = ordinary_multi_parent();
    let mut income = if scorp {
        let FederalIncome::ScorpActivities { data } = scorp_income(partner) else { unreachable!() };
        FederalIncome::ScorpActivities { data: scorp_ordinary_multi_baseline(data) }
    } else {
        FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(partner) }
    };
    configure_component_capital_source(&mut income, scale);
    income
}

pub(crate) fn configure_component_capital_source(income: &mut FederalIncome, scale: i64) {
    let scorp = matches!(income, FederalIncome::ScorpActivities { .. });
    match &mut *income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(0);
            data.entity.alternative.as_mut().unwrap().opening = Some(0);
            data.entity.regular.contributions = 1_200 * scale;
            data.entity.alternative.as_mut().unwrap().contributions = 1_200 * scale;
        }
        FederalIncome::ScorpActivities { data } => {
            data.health_insurance = Some(BusinessHealthInsurance::None);
            data.entity.regular.opening = Some(0);
            data.entity.alternative.as_mut().unwrap().opening = Some(0);
            data.entity.regular.additions = 1_200 * scale;
            data.entity.alternative.as_mut().unwrap().additions = 1_200 * scale;
            data.entity.loans.clear();
        }
        _ => unreachable!(),
    }
    let (source, current, alternative) = entity_exempt_other_fields(income);
    current.ordinary = 0;
    current.portfolio.short = -600 * scale;
    current.portfolio.long = -600 * scale;
    *alternative = Some(current.clone());
    alternative.as_mut().unwrap().ordinary = 700 * scale;
    source.components.truncate(1);
    source.risk_groups.truncate(1);
    source.passive_groups.truncate(1);
    source.qbi_blocks.truncate(1);
    let component = &mut source.components[0];
    let mut values = amounts(0, 0);
    values.portfolio = current.portfolio.clone();
    values.capital_carry = if scorp {
        values.items = scorp_items();
        K1StatementCapitalCarry::Scorp {
            amounts: K1StatementScorpCapitalCarry {
                basis_short: Some(400 * scale),
                basis_long: Some(400 * scale),
                risk_short: Some(100 * scale),
                risk_long: Some(100 * scale),
                basis_collectibles: Some(0),
                risk_collectibles: Some(0),
            },
        }
    } else {
        K1StatementCapitalCarry::Partnership {
            amounts: K1StatementPartnerCapitalCarry {
                short: K1RetainedLossCarry { basis: Some(400 * scale), risk: Some(100 * scale) },
                long: K1RetainedLossCarry { basis: Some(400 * scale), risk: Some(100 * scale) },
            },
        }
    };
    component.regular = Some(values.clone());
    values.operating = 700 * scale;
    component.alternative = K1StatementAmt::Separate { amounts: values };
    component.qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 0, loss: 0 });
    // Prior gross loss1400, basis600: each class retains400 at basis.
    // Actual prior line19b400/line21loss400/profit0 leaves no risk investment;
    // allowed200/class and suspended100/class explain deducted400/prior200.
    // Current contribution1200 includes actual excluded funding500: risk700.
    // Separate AMT current ordinary income700 raises its basis and risk income.
    source.risk_groups[0].investment_parts.as_mut().unwrap().contributions = 1_200 * scale;
    let start = ScorpRiskStart::Detailed { capacity: 400 * scale, profit: 0, loss: 400 * scale };
    source.risk_groups[0].source = if scorp {
        let mut regular = scorp_risk_history();
        regular.start = start;
        regular.prior_loss = 200 * scale;
        regular.deducted = 400 * scale;
        regular.excluded_funding = 500 * scale;
        let alternative = regular.clone();
        K1RiskSource::Scorp { regular, alternative: Some(alternative) }
    } else {
        let mut regular = partner_risk_record();
        regular.start = start;
        regular.prior_loss = 200 * scale;
        regular.deducted = 400 * scale;
        regular.excluded_contributions = 500 * scale;
        let alternative = regular.clone();
        K1RiskSource::Partnership { regular, alternative: Some(alternative) }
    };
    assert_eq!(entity_exempt_other_rule(income), None);
}

/// Existing two-activity sources with fully available current portfolio facts.
pub(crate) fn configure_component_capital_current(
    income: &mut FederalIncome,
    regular: [[i64; 2]; 2],
    alternative: [[i64; 2]; 2],
) {
    let scorp = matches!(income, FederalIncome::ScorpActivities { .. });
    entity_exempt_other_fields(income).1.portfolio = portfolio();
    configure_entity_exempt_other(income);
    set_entity_exempt_other(income, 0, 0);
    match income {
        FederalIncome::PartnershipActivities { data } => {
            data.entity.regular.opening = Some(100_000_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(100_000_000);
        }
        FederalIncome::ScorpActivities { data } => {
            data.entity.regular.opening = Some(100_000_000);
            data.entity.alternative.as_mut().unwrap().opening = Some(100_000_000);
        }
        _ => unreachable!(),
    }
    let (source, current, amt_current) = entity_exempt_other_fields(income);
    current.ordinary = 0;
    current.portfolio.short = regular.iter().map(|v| v[0]).sum();
    current.portfolio.long = regular.iter().map(|v| v[1]).sum();
    *amt_current = Some(current.clone());
    amt_current.as_mut().unwrap().portfolio.short = alternative.iter().map(|v| v[0]).sum();
    amt_current.as_mut().unwrap().portfolio.long = alternative.iter().map(|v| v[1]).sum();
    for (index, component) in source.components.iter_mut().enumerate() {
        let mut value = amounts(0, 0);
        if scorp {
            value.items = scorp_items();
            value.capital_carry = scorp_carry();
        }
        value.portfolio.short = regular[index][0];
        value.portfolio.long = regular[index][1];
        component.regular = Some(value.clone());
        value.portfolio.short = alternative[index][0];
        value.portfolio.long = alternative[index][1];
        component.alternative = K1StatementAmt::Separate { amounts: value };
        component.qualified_current_controls.ordinary =
            Some(K1CurrentSourceParts { income: 0, loss: 0 });
    }
    assert_eq!(entity_exempt_other_rule(income), None);
}

/// Current capital gain and independently documented basis-only prior losses.
pub(crate) fn configure_component_capital_positive_release(income: &mut FederalIncome) {
    configure_component_capital_source(income, 100);
    let (source, current, alternative) = entity_exempt_other_fields(income);
    current.portfolio.short = 100_000;
    current.portfolio.long = 0;
    *alternative = Some(current.clone());
    let regular = source.components[0].regular.as_mut().unwrap();
    regular.portfolio = current.portfolio.clone();
    match &mut regular.capital_carry {
        K1StatementCapitalCarry::Partnership { amounts } => {
            amounts.short = K1RetainedLossCarry { basis: Some(40_000), risk: Some(0) };
            amounts.long = K1RetainedLossCarry { basis: Some(0), risk: Some(0) };
        }
        K1StatementCapitalCarry::Scorp { amounts } => {
            amounts.basis_short = Some(40_000);
            amounts.basis_long = Some(0);
            amounts.risk_short = Some(0);
            amounts.risk_long = Some(0);
        }
        _ => unreachable!(),
    }
    let mut amt = regular.clone();
    match &mut amt.capital_carry {
        K1StatementCapitalCarry::Partnership { amounts } => amounts.short.basis = Some(20_000),
        K1StatementCapitalCarry::Scorp { amounts } => amounts.basis_short = Some(20_000),
        _ => unreachable!(),
    }
    source.components[0].alternative = K1StatementAmt::Separate { amounts: amt };
    match &mut source.risk_groups[0].source {
        K1RiskSource::Partnership { regular, alternative } => {
            regular.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
            regular.prior_loss = 0;
            regular.deducted = 0;
            regular.excluded_contributions = 0;
            *alternative = Some(regular.clone());
        }
        K1RiskSource::Scorp { regular, alternative } => {
            regular.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
            regular.prior_loss = 0;
            regular.deducted = 0;
            regular.excluded_funding = 0;
            *alternative = Some(regular.clone());
        }
        _ => unreachable!(),
    }
    assert_eq!(entity_exempt_other_rule(income), None);
}
