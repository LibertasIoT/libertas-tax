use crate::k1_page::K1Partnership;
use crate::tests::*;

pub(in crate::tests) fn partner_basis_record() -> PartnerBasisRecord {
    PartnerBasisRecord {
        opening: Some(0),
        contributions: 0,
        opening_liabilities: 0,
        ending_liabilities: 0,
        assumed: 0,
        transferred: 0,
        distributions: 0,
        prior_loss: 0,
        prior_nondeductible: 0,
        complete: true,
    }
}
pub(in crate::tests) fn partner_risk_record() -> PartnerRiskRecord {
    PartnerRiskRecord {
        start: ScorpRiskStart::New,
        opening_debt: 0,
        ending_debt: 0,
        excluded_contributions: 0,
        excluded_distributions: 0,
        protected: 0,
        released: 0,
        prior_loss: 0,
        prior_nondeductible: 0,
        deducted: 0,
        recaptured: 0,
        complete: true,
    }
}

pub(in crate::tests) fn partner_activity_record() -> PartnerActivityRecord {
    let mut basis = partner_basis_record();
    basis.opening = Some(1000000);
    let mut risk = partner_risk_record();
    risk.start = ScorpRiskStart::Detailed { capacity: 1000000, profit: 0, loss: 0 };
    PartnerActivityRecord {
        section1231: None,
        rentals: Default::default(),
        carried: Default::default(),
        capital_carried: Default::default(),
        ordinary: 100000,
        exempt: 0,
        nondeductible: 0,
        basis,
        risk,
        passive_loss: 150000,
    }
}

pub(in crate::tests) fn ordinary_partner_source(amount: i64, active: bool) -> PartnerK1 {
    let mut regular = partner_activity_record();
    regular.ordinary = amount * 100;
    regular.passive_loss = 0;
    let mut facts = participation_facts();
    if active {
        facts.minutes = 30001;
    }
    PartnerK1 {
        source: None,
        research_credit_line: None,
        amt_record_choice: None,

        amt_section179_income_carry: None,
        reporting_period: None,
        section179_sales: alloc::vec::Vec::new(),
        qbi_aggregation_group: None,
        section179_carryover_sold: 0,
        schedule_k3: None,
        rental_facts: crate::K1RentalFacts { participation: None, ..Default::default() },
        income_source: None,
        gross_personal_services: None,
        guaranteed_services: 0,
        guaranteed_capital: 0,
        health_guaranteed: 0,
        health_distributions: 0,
        alternative: Some(regular.clone()),
        regular,
        participation: Some(PartnerParticipation::General { facts }),
        trade_kind: ScorpTradeKind::Ordinary,
        limited_for_se: Some(true),
        reported_se: Some(0),
        se_history: PartnerSeHistory { basis: 0, risk: 0, passive: 0 },
        se_gross: None,
        qbi: None,
        qbi_records: Default::default(),
        health_insurance: BusinessHealthInsurance::None,
        pensions: 0,
        retirement: PartnerRetirement::None,
        fuel_tax_credit: None,
        prior_fuel_credit: None,
        final_k1: false,
        income: Vec::new(),
        deductions: Vec::new(),
        deferred_compensation_tax: 0,
        crp_payments: 0,
        interest_term: None,
        disposition: Some(crate::K1Disposition::Kept),
        sale: crate::PartnerSaleParts::default(),
        special: Vec::new(),
        publicly_traded: false,
        separate: None,
        section179: 0,
        section179_carryover: 0,
        credits: Vec::new(),
        unentered: vec![],
        investment_property: Vec::new(),
        investment_confirmed: None,
        housing_confirmed: None,
    }
}

pub(in crate::tests) fn partner_qbi_statement(income: i64) -> PartnerQbiStatement {
    PartnerQbiStatement {
        section1231_qbi: 0,
        ptp_income: 0,
        reit_dividends: 0,
        income: income * 100,
        wages: 0,
        property: 0,
        service: false,
    }
}

pub(in crate::tests) fn partner_health_source(premiums: i64, employer: bool) -> PartnerK1 {
    let mut partner = ordinary_partner_source(20000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(2000000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 2000000 });
    let qbi = partner_qbi_statement(20000);
    partner.qbi_records.qualified_se_gross = 2000000;
    partner.qbi = Some(qbi);
    if let FederalIncome::Business { data } = health_business(20000, premiums, employer) {
        partner.health_insurance = data.health_insurance;
        if let BusinessHealthInsurance::Plan { establishment, .. } = &mut partner.health_insurance {
            *establishment =
                HealthEstablishment::Partnership { payment: EntityPremiumPayment::EntityPaid };
        }
    }
    partner
}

pub(in crate::tests) fn partner_paid_health_source(premiums: i64, employer: bool) -> PartnerK1 {
    let mut data = partner_health_source(premiums, employer);
    data.health_guaranteed = premiums * 100;
    data.guaranteed_services = premiums * 100;
    data.reported_se = Some((20000 + premiums) * 100);
    data.se_gross = Some(PartnerSeGross::Nonfarm { gross: (20000 + premiums) * 100 });
    data
}

pub(in crate::tests) fn partner_distribution_health_source() -> PartnerK1 {
    let mut data = partner_health_source(6000, false);
    data.health_distributions = 600000;
    data.regular.basis.distributions = 600000;
    data.alternative.as_mut().unwrap().basis.distributions = 600000;
    data
}

pub(in crate::tests) fn partner_excess_loss_source(qualified: i64) -> PartnerK1 {
    let mut data = ordinary_partner_source(-300000, true);
    data.regular.basis.opening = Some(30000000);
    data.regular.risk.start = ScorpRiskStart::Detailed { capacity: 30000000, profit: 0, loss: 0 };
    data.alternative = Some(data.regular.clone());
    data.qbi = Some(partner_qbi_statement(-qualified));
    data
}

pub(in crate::tests) fn partner_uniform_profit_draft(profit_qualified: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    let mut profit = ordinary_partner_source(10000, true);
    profit.qbi = Some(partner_qbi_statement(profit_qualified));
    add(&mut d, income(FederalIncome::Partnership { data: profit }));
    d
}

// Shared PTP179 source scenario: actual regular and alternative ledgers;
// independent AMT income-carry history is supplied by each selected test.
pub(in crate::tests) fn ptp179_draft(profit: i64, elected: i64) -> FederalDraft {
    let mut partner = ordinary_partner_source(profit, true);
    partner.qbi = Some(partner_qbi_statement(profit));
    partner.section179 = elected * 100;
    partner.publicly_traded = true;
    for record in core::iter::once(&mut partner.regular).chain(partner.alternative.as_mut()) {
        record.basis.opening = Some(10_000_000);
        record.basis.distributions = 0;
        record.risk.start = ScorpRiskStart::Detailed { capacity: 10_000_000, profit: 0, loss: 0 };
    }
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    draft
}

pub(in crate::tests) fn selected_ptp179(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    k1: i64,
    ordinary_cents: i64,
) -> Result<Section179PreparedAllocation, Section179PreparedIssue> {
    crate::section179::section179_with_native_prepared_income_and_operating_context(
        draft,
        system,
        &[],
        &[Section179BusinessIncomeSource {
            source: k1,
            system,
            ordinary: Section179IncomeClass {
                current_net_cents: ordinary_cents,
                prior_allowed_loss_cents: 0,
            },
            section1231: Section179IncomeClass {
                current_net_cents: 0,
                prior_allowed_loss_cents: 0,
            },
        }],
        |_, _, _| {
            Some(Section179OtherLimitedDeduction {
                system,
                amount_cents: 0,
                coordinated_owner_sources: Default::default(),
            })
        },
    )
}

// Two actual fresh ordinary PTP pages; independent regular/AMT income carries.
pub(in crate::tests) fn d6_ptp_pages(start: BasisStart) -> Vec<IncomeDocumentItem> {
    [
        ("Synthetic D6 PTP A", 2_000_000, 500_000, 200_000, 100_000),
        ("Synthetic D6 PTP B", 1_000_000, 300_000, 0, 0),
    ]
    .into_iter()
    .map(|(name, ordinary, elected, regular_carry, amt_carry)| {
        let mut page = crate::tests::cross::interview::k1_1065(name);
        page.publicly_traded = true;
        page.ordinary = ordinary;
        page.more.section179 = elected;
        page.more.section179_carryover = regular_carry;
        // 2026-10-15: main supplies regular carry as an aggregate, with no
        // originating year. This fully allowed fixture needs no vintage order.
        // Genuine separately supplied prior AMT Form4562 facts. Its zero is
        // explicit, not a fallback to the regular carry or an absent history.
        page.more.amt_section179_income_carry = Some(crate::Section179IncomeCarryHistory {
            gross: amt_carry,
            sold: 0,
            vintages: if amt_carry == 0 {
                Vec::new()
            } else {
                vec![crate::Asset179Carryover { year: 2024, amount: amt_carry }]
            },
        });
        (page.participation, page.limited_for_se) = (
            Some(crate::PartnerParticipation::General {
                facts: (Participation::Over500Hours { hours: 600 }).record(),
            }),
            Some(false),
        );
        page.basis.start = start.clone();
        page.qbi = PartnerQbi::NoStatement;
        crate::tests::cross::interview::document(
            0,
            IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) },
        )
    })
    .collect()
}

pub(in crate::tests) fn d6_selected_source(
    draft: &FederalDraft,
    system: crate::Section179IncomeSystem,
) -> Result<crate::Section179PreparedAllocation, crate::Section179PreparedIssue> {
    let alternative = system == crate::Section179IncomeSystem::Alternative;
    let entries = crate::community::entries(draft);
    let mut sources = Vec::new();
    for entry in &entries {
        let FederalIncome::Partnership { data } = &entry.income else {
            // This finite source control has only the actual baseline W2 and
            // two K1s: no direct C/F income or other limited deduction source.
            assert!(matches!(entry.income, FederalIncome::Wage { .. }));
            continue;
        };
        let record = if alternative { data.alternative.as_ref().unwrap() } else { &data.regular };
        let pool = crate::community::partner_pool(draft, entry.id, data, record).unwrap();
        assert_eq!(record.passive_loss, 0);
        assert_eq!(pool.combined.passive, 0);
        assert_eq!(pool.combined.prior_released, 0);
        assert_eq!(pool.combined.prior_pool, 0);
        assert!(pool.combined.characters.is_none());
        sources.push(crate::Section179BusinessIncomeSource {
            source: entry.id,
            system,
            ordinary: crate::Section179IncomeClass {
                current_net_cents: pool.combined.nonpassive,
                prior_allowed_loss_cents: 0,
            },
            section1231: crate::Section179IncomeClass {
                current_net_cents: 0,
                prior_allowed_loss_cents: 0,
            },
        });
    }
    crate::section179::section179_with_native_prepared_income_and_operating_context(
        draft,
        system,
        &[],
        &sources,
        |_, _, _| {
            // Same finite owner domain as the assertions above: there is no
            // Business source whose home-office owner could claim this deduction.
            Some(crate::Section179OtherLimitedDeduction {
                system,
                amount_cents: 0,
                coordinated_owner_sources: Default::default(),
            })
        },
    )
}

// A prepared source seam, not a PAL ordering or whole-return routing choice.
pub(in crate::tests) fn section179_income_source(
    id: i64,
    current: i64,
    prior: i64,
    gain: i64,
    prior_gain: i64,
) -> crate::Section179BusinessIncomeSource {
    crate::Section179BusinessIncomeSource {
        source: id,
        system: crate::Section179IncomeSystem::Regular,
        ordinary: crate::Section179IncomeClass {
            current_net_cents: current,
            prior_allowed_loss_cents: prior,
        },
        section1231: crate::Section179IncomeClass {
            current_net_cents: gain,
            prior_allowed_loss_cents: prior_gain,
        },
    }
}

// One actual K-1 editor supplies the entity income and separate personal payments.
pub(in crate::tests) fn partner_service179_session(services: i64, capital: i64) -> Interview {
    use crate::tests::cross::interview::{k1_1065, k1_1065_continuing, partner_benefits_session};
    let mut page = k1_1065("Synthetic active service partner");
    page.ordinary = 500_000;
    page.services = services;
    page.capital = capital;
    page.more.section179 = 7_000_000;
    page.more.amt_section179_income_carry =
        Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: vec![] });
    (page.participation, page.limited_for_se) = (
        Some(crate::PartnerParticipation::General {
            facts: (Participation::Over500Hours { hours: 600 }).record(),
        }),
        Some(false),
    );
    page.basis.start = k1_1065_continuing(
        10_000_000,
        ScorpRiskStart::Detailed { capacity: 10_000_000, profit: 0, loss: 0 },
    );
    page.qbi = PartnerQbi::NoStatement;
    partner_benefits_session(vec![page])
}

// Source cents throughout; a continuing passive PTP with ample actual basis/risk.
pub(in crate::tests) fn ptp_rental_source(
    ordinary: i64,
    real_estate: i64,
    other: i64,
) -> PartnerK1 {
    let mut source = ordinary_partner_source(0, false);
    source.publicly_traded = true;
    source.regular.ordinary = ordinary;
    source.regular.basis.opening = Some(20_000_000);
    source.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 20_000_000, profit: 0, loss: 0 };
    source.regular.rentals.real_estate = real_estate;
    source.regular.rentals.other = other;
    source.rental_facts.participation = Some(crate::K1RentalParticipation::NotActive);
    source.alternative = Some(source.regular.clone());
    source
}

pub(in crate::tests) fn ptp_rental_draft(source: PartnerK1) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    draft
}

pub(in crate::tests) fn current_ptp_page(amount: i64, service: bool) -> K1Partnership {
    use crate::tests::cross::interview::{k1_1065, k1_1065_continuing};

    let mut page = k1_1065("Current ordinary PTP source");
    page.publicly_traded = true;
    page.ordinary = amount * 100;
    (page.participation, page.limited_for_se) =
        (Some((LimitedParticipation::Over500Hours { hours: 600 }).record()), Some(true));
    page.basis.start = k1_1065_continuing(
        1_000_000,
        ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 },
    );
    page.qbi = PartnerQbi::Statement {
        section1231_qbi: 0,
        ptp_income: 0,
        reit_dividends: 0,
        income: amount * 100,
        wages: 0,
        property: 0,
        service,
    };
    page
}

pub(in crate::tests) fn current_ptp_care_return(wages: i64) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    add_child(&mut draft);
    draft.dependents[0].set_care_expenses(400_000);
    document_care(&mut draft);
    draft.checklist.paid |= PaidTopics::CARE.0;
    draft
}

pub(in crate::tests) fn current_ptp_passive_history(amount: i64, service: bool) -> K1Partnership {
    let mut page = current_ptp_page(amount, service);
    let BasisStart::Continuing { carried, .. } = &mut page.basis.start else { unreachable!() };
    carried.passive_loss = 300_000;
    page.qbi_records.as_mut().unwrap().history = Some(PartnerQbiHistory {
        basis: vec![],
        risk: vec![],
        passive: vec![
            loss_year(2023, 2_000, 1_000, 2_000, 1_000),
            loss_year(2017, 1_000, 0, 1_000, 0),
        ],
    });
    page
}

pub(in crate::tests) fn current_ptp_ordinary_histories(service: bool) -> K1Partnership {
    let mut page = current_ptp_page(10_000, service);
    page.participation = Some((LimitedParticipation::Investor { prior_years: 0 }).record());
    page.limited_for_se = Some(true);
    let BasisStart::Continuing { carried, .. } = &mut page.basis.start else { unreachable!() };
    carried.basis_loss = 300_000;
    carried.risk_loss = 200_000;
    carried.passive_loss = 100_000;
    page.qbi_records.as_mut().unwrap().history = Some(PartnerQbiHistory {
        basis: vec![
            loss_year(2023, 2_000, 1_000, 2_000, 1_000),
            loss_year(2017, 1_000, 0, 1_000, 0),
        ],
        risk: vec![loss_year(2024, 2_000, 1_500, 2_000, 1_500)],
        passive: vec![loss_year(2025, 1_000, 500, 1_000, 500)],
    });
    page
}

// Construct one explicit synthetic paper from the shared scenario's known boxes.
// This helper supplies fixture facts; production never reconstructs paper from totals.
pub(in crate::tests) fn partnership_document(
    page: crate::k1_page::K1Partnership,
) -> crate::K1PartnershipDocument {
    use crate::partnership_source::{credit_code_name, deduction_code_name, income_code};
    let row = |code: &str, amount: Option<i64>| crate::K1CodeRow {
        code: code.into(),
        amount,
        text: if amount.is_none() { "STMT".into() } else { alloc::string::String::new() },
        statement: amount.is_none(),
    };
    let mut form = crate::K1PartnershipPaper {
        reporting_period: page.reporting_period,
        final_k1: page.final_k1,
        partnership: crate::PrintedParty::from_name(page.name.clone()),
        partner: page.printed.as_ref().map(|printed| printed.recipient.clone()).unwrap_or_default(),
        publicly_traded: page.publicly_traded,
        liabilities: crate::K1Liabilities {
            recourse: crate::K1MoneyPair {
                beginning: Some(page.liabilities_start),
                ending: Some(page.liabilities_end),
            },
            ..Default::default()
        },
        ordinary: Some(page.ordinary),
        rental_real_estate: Some(page.rentals.real_estate),
        other_rental: Some(page.rentals.other),
        services: Some(page.services),
        capital: Some(page.capital),
        total_guaranteed: Some(page.services + page.capital),
        interest: Some(page.more.interest),
        dividends: Some(page.more.dividends),
        qualified_dividends: Some(page.more.qualified_dividends),
        royalties: Some(page.more.royalties),
        short: Some(page.more.short),
        long: Some(page.more.long),
        collectibles: Some(page.more.collectibles),
        unrecaptured: Some(
            page.more.unrecaptured_partnership
                + page.more.unrecaptured_fund
                + page.more.unrecaptured_business
                + page.rentals.real_estate_unrecaptured
                + page.rentals.other_unrecaptured,
        ),
        section1231: Some(
            page.more.section1231
                + page.rentals.real_estate_section1231
                + page.rentals.other_section1231,
        ),
        section179: Some(page.more.section179),
        foreign_taxes: Some(page.more.foreign_taxes),
        k3_attached: matches!(&page.schedule_k3, crate::K3Attached::Attached(_)),
        ..Default::default()
    };
    form.income = page
        .more
        .income
        .iter()
        .map(|item| row(income_code(item.code), Some(item.amount)))
        .collect();
    form.deductions = page
        .more
        .deductions
        .iter()
        .map(|item| row(deduction_code_name(item.code), Some(item.amount)))
        .collect();
    for (code, amount) in [
        ("A", page.more.cash_charity),
        ("M", page.more.medical_insurance),
        ("R", page.more.pensions),
    ] {
        if amount != 0 {
            form.deductions.push(row(code, Some(amount)))
        }
    }
    form.self_employment.push(row("A", Some(page.se)));
    match page.se_gross {
        crate::SeGross::None => {}
        crate::SeGross::Farming { amount } => form.self_employment.push(row("B", Some(amount))),
        crate::SeGross::Nonfarm { amount } => form.self_employment.push(row("C", Some(amount))),
    }
    let credits: Vec<crate::K1Credit> = page.more.credits.iter().cloned().map(Into::into).collect();
    form.credits = credits
        .iter()
        .map(|credit| row(credit_code_name(&credit.code), Some(credit.amount)))
        .collect();
    if page.more.backup_withholding != 0 {
        form.credits.push(row("O", Some(page.more.backup_withholding)))
    }
    for (code, amount) in [
        ("A", page.more.exempt_interest),
        ("B", page.more.exempt_other),
        ("C", page.more.nondeductible),
    ] {
        if amount != 0 {
            form.exempt.push(row(code, Some(amount)))
        }
    }
    if page.distributions != 0 {
        form.distributions.push(row("A", Some(page.distributions)))
    }
    for (code, amount) in [
        ("AI", page.more.deferred_compensation_tax),
        ("AQ", page.more.crp_payments),
        ("AB", page.more.sale_ordinary),
        ("AC", page.more.sale_collectibles),
        ("AD", page.more.sale_unrecaptured),
    ] {
        if amount != 0 {
            form.information.push(row(code, Some(amount)))
        }
    }
    if matches!(page.qbi, crate::PartnerQbi::Statement { .. }) {
        form.information.push(row("Z", None))
    }
    if !page.more.section179_sales.is_empty() {
        form.information.push(row("L", None))
    }
    for property in &page.more.investment_property {
        match property {
            crate::K1InvestmentProperty::RehabilitationRental { .. } => {
                form.credits.push(row("E", None))
            }
            crate::K1InvestmentProperty::RehabilitationOther { .. } => {
                form.information.push(row("D", None))
            }
            crate::K1InvestmentProperty::Energy { .. } => form.information.push(row("E", None)),
            crate::K1InvestmentProperty::CleanElectricity { .. } => {
                form.information.push(row("AV", None))
            }
        }
    }
    let mut situations = Vec::new();
    for item in &page.more.unentered {
        match item {
            crate::PartnerK1Item::RentalDisposition => {
                situations.push(crate::PartnershipStatementSituation::RentalDisposition)
            }
            crate::PartnerK1Item::OtherIncome => {
                form.income.push(row("B", None));
            }
            crate::PartnerK1Item::OtherDeductions => form.deductions.push(row("O", None)),
            crate::PartnerK1Item::Credits => form.credits.push(row("H", None)),
            crate::PartnerK1Item::CreditRecapture => form.credits.push(row("AX", None)),
            crate::PartnerK1Item::Depletion => form.information.push(row("T", None)),
            crate::PartnerK1Item::AmtItems => form.amt.push(row("C", None)),
            crate::PartnerK1Item::PropertyDistribution => form.distributions.push(row("B", None)),
            crate::PartnerK1Item::OtherInformation => form.information.push(row("F", None)),
            crate::PartnerK1Item::CooperativeQbi => {
                situations.push(crate::PartnershipStatementSituation::CooperativeQbi)
            }
            crate::PartnerK1Item::MultipleActivities => {
                form.multiple_at_risk = true;
            }
        }
    }
    let statements = crate::PartnershipStatements {
        qbi: page.qbi.clone(),
        rentals: crate::PartnershipRentalStatement {
            real_estate_section1231: page.rentals.real_estate_section1231,
            real_estate_unrecaptured: page.rentals.real_estate_unrecaptured,
            other_section1231: page.rentals.other_section1231,
            other_unrecaptured: page.rentals.other_unrecaptured,
            real_estate_qbi: page.rentals.real_estate_qbi.clone(),
            other_qbi: page.rentals.other_qbi.clone(),
        },
        property: crate::PartnershipPropertyStatement {
            unrecaptured_partnership: page.more.unrecaptured_partnership,
            unrecaptured_fund: page.more.unrecaptured_fund,
            unrecaptured_business: page.more.unrecaptured_business,
            section1231_business: page.more.section1231,
        },
        income: page
            .more
            .income
            .iter()
            .map(|item| crate::PartnershipIncomeStatement {
                code: item.code,
                amount: (page
                    .more
                    .income
                    .iter()
                    .filter(|other| income_code(other.code) == income_code(item.code))
                    .count()
                    > 1)
                .then_some(item.amount),
            })
            .collect(),
        credits: credits
            .iter()
            .map(|credit| crate::PartnershipCreditStatement {
                code: credit.code,
                line: (credit.code != K1CreditCode::Research).then_some(credit.line).flatten(),
                amount: (credits.iter().filter(|other| other.code == credit.code).count() > 1)
                    .then_some(credit.amount),
            })
            .collect(),
        investment_property: page.more.investment_property.clone(),
        section179_sales: page.record().section179_sales,
        schedule_k3: match &page.schedule_k3 {
            crate::K3Attached::NotAttached => None,
            crate::K3Attached::Attached(k3) => Some(super::k3_paper::schedule_k3_paper(
                k3,
                crate::ScheduleK3Paper::Partnership(crate::K3PartnershipPaper {
                    entity: form.partnership.clone(),
                    recipient: form.partner.clone(),
                    ..Default::default()
                }),
            )),
        },
        situations,
    };
    crate::K1PartnershipDocument {
        form,
        statements,
        personal: Some(crate::PartnerPersonalRecords::from_page(&page)),
    }
}

impl From<crate::k1_page::K1Partnership> for crate::K1PartnershipDocument {
    fn from(page: crate::k1_page::K1Partnership) -> Self {
        partnership_document(page)
    }
}

// The shared synthetic scenarios provide their printed code and statement facts
// explicitly. Personal origins remain desired topic answers, never paper fields.
pub(in crate::tests) fn set_partnership_credits(
    document: &mut crate::K1PartnershipDocument,
    credits: &[crate::K1Credit],
) {
    document.form.credits = credits
        .iter()
        .map(|credit| crate::K1CodeRow {
            code: crate::partnership_source::credit_code_name(&credit.code).into(),
            amount: Some(credit.amount),
            text: alloc::string::String::new(),
            statement: credit.line.is_some(),
        })
        .collect();
    document.statements.credits = credits
        .iter()
        .map(|credit| crate::PartnershipCreditStatement {
            code: credit.code,
            line: (credit.code != K1CreditCode::Research).then_some(credit.line).flatten(),
            amount: (credits.iter().filter(|other| other.code == credit.code).count() > 1)
                .then_some(credit.amount),
        })
        .collect();
    document.personal.as_mut().unwrap().research_credit_line = credits
        .iter()
        .find(|credit| credit.code == K1CreditCode::Research)
        .and_then(|credit| credit.line);
    document.personal.as_mut().unwrap().credit_origins = credits
        .iter()
        .map(|credit| crate::PartnerCreditPersonal {
            code: credit.code,
            line: credit.line,
            carbon_oxide_origin: credit.carbon_oxide_origin.clone(),
            renewable_facility_origin: credit.renewable_facility_origin.clone(),
        })
        .collect();
}

pub(in crate::tests) fn push_partnership_credit(
    document: &mut crate::K1PartnershipDocument,
    credit: crate::K1Credit,
) {
    let mut credits: Vec<crate::K1Credit> =
        document.calculation_page().more.credits.into_iter().map(Into::into).collect();
    credits.push(credit);
    set_partnership_credits(document, &credits);
}

pub(in crate::tests) fn push_partnership_property(
    document: &mut crate::K1PartnershipDocument,
    property: crate::K1InvestmentProperty,
) {
    let (credit_box, code) = match &property {
        crate::K1InvestmentProperty::RehabilitationRental { .. } => (true, "E"),
        crate::K1InvestmentProperty::RehabilitationOther { .. } => (false, "D"),
        crate::K1InvestmentProperty::Energy { .. } => (false, "E"),
        crate::K1InvestmentProperty::CleanElectricity { .. } => (false, "AV"),
    };
    let rows = if credit_box { &mut document.form.credits } else { &mut document.form.information };
    if !rows.iter().any(|row| row.code == code) {
        rows.push(crate::K1CodeRow {
            code: code.into(),
            amount: None,
            text: "STMT".into(),
            statement: true,
        });
    }
    document.personal.as_mut().unwrap().investment_forms.push(crate::PartnerInvestmentPersonal {
        id: property.id(),
        own_form: property.own_form().cloned(),
    });
    document.statements.investment_property.push(property);
}

// Explicit setup for lifecycle scenarios whose native builders describe known
// synthetic paper facts. Native calculation-only fixtures remain source-free.
pub(in crate::tests) fn known_fact_partnership_draft(mut draft: FederalDraft) -> FederalDraft {
    for index in 0..draft.income.len() {
        let entry = &draft.income[index];
        let FederalIncome::Partnership { data } = &entry.income else { continue };
        if data.source.is_some() {
            continue;
        }
        let mut page = crate::k1_page::K1Partnership::from_record(&entry.label, data)
            .expect("synthetic scenario has representable known K-1 facts");
        page.printed = entry.printed.clone().or_else(|| {
            crate::tests::support::builders::fixture_document_names(
                &draft,
                if entry.owner == crate::DocumentOwner::Spouses { -2 } else { -1 },
            )
        });
        let assigned = page
            .printed
            .as_ref()
            .map(crate::PrintedDocumentNames::assignment)
            .filter(|assignment| assignment.owner.is_known());
        let document = partnership_document(page);
        let research_credit_line =
            document.personal.as_ref().and_then(|records| records.research_credit_line);
        let mut source = document.source();
        assigned
            .or_else(|| {
                crate::document_name_matcher::match_eligible_recipient(
                    &draft,
                    &source.paper.partner.name,
                    crate::document_name_matcher::RecipientDomain::Earner,
                )
                .and_then(crate::DocumentRecipient::from_key)
                .map(crate::document_names::Assignment::matched)
            })
            .unwrap_or_default()
            .apply_to(&mut source.assigned, &mut source.owner_source, &mut source.joint_share);
        let entry = &mut draft.income[index];
        entry.printed = Some(source.printed_names());
        let FederalIncome::Partnership { data } = &mut entry.income else { unreachable!() };
        data.research_credit_line = research_credit_line;
        data.amt_record_choice = document
            .personal
            .as_ref()
            .and_then(|personal| personal.amt.as_ref())
            .map(|amt| matches!(amt, crate::PartnerAmtRecords::Separate { .. }));
        data.source = Some(alloc::boxed::Box::new(source));
    }
    crate::tests::support::builders::known_fact_scorp_draft(draft)
}

pub(in crate::tests) fn complete_partnership_paper(
    draft: &FederalDraft,
) -> (crate::k1_page::K1Partnership, crate::K1PartnershipDocument) {
    use crate::tests::cross::interview::k1_1065;
    let mut calculation = k1_1065("Synthetic complete paper");
    calculation.reporting_period =
        Some(crate::K1EntityTaxPeriod { began: 20260101, ended: 20261231 });
    calculation.ordinary = 123_449;
    calculation.basis.start = crate::tests::cross::interview::k1_1065_continuing(
        0,
        ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 },
    );
    calculation.basis.contributions = 1_000_000;
    calculation.liabilities_start = 60_003;
    calculation.liabilities_end = 100_007;
    calculation.distributions = 60_005;
    calculation.more.credits = vec![
        K1Credit {
            code: K1CreditCode::EmployerTipFica,
            line: None,
            amount: 100_049,
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
        }
        .into(),
    ];
    calculation.more.credits.push(
        K1Credit {
            code: K1CreditCode::Research,
            line: Some(BusinessCredit::ResearchSmallBusiness),
            amount: 20_031,
            carbon_oxide_origin: None,
            renewable_facility_origin: None,
        }
        .into(),
    );
    let mut paper: crate::K1PartnershipDocument = calculation.clone().into();
    paper.form.amended_k1 = true;
    paper.form.partnership.federal_id = "00-1234567".into();
    paper.form.partnership.address = "17 Synthetic Lane, Sample, NY 10001".into();
    paper.form.partner.federal_id = "***-**-4322".into();
    paper.form.partner.address = "23 Synthetic Road, Sample, NY 10002".into();
    paper.form.partner.name =
        crate::tests::support::builders::fixture_document_names(draft, -1).unwrap().recipient.name;
    paper.form.irs_center = "Ogden".into();
    paper.form.entity_type = "Individual".into();
    paper.form.domestic_partner = true;
    paper.form.share.profit = crate::K1PercentagePair { beginning: Some(12.5), ending: Some(25.0) };
    paper.form.liabilities = crate::K1Liabilities {
        nonrecourse: crate::K1MoneyPair { beginning: Some(10_001), ending: Some(20_001) },
        qualified_nonrecourse: crate::K1MoneyPair { beginning: Some(20_001), ending: Some(30_002) },
        recourse: crate::K1MoneyPair { beginning: Some(30_001), ending: Some(50_004) },
    };
    paper.form.distributions = vec![
        crate::K1CodeRow {
            code: "A".into(),
            amount: Some(20_003),
            text: "Cash *".into(),
            statement: true,
        },
        crate::K1CodeRow {
            code: "F".into(),
            amount: Some(40_002),
            text: "Deemed money".into(),
            statement: false,
        },
    ];
    (calculation, paper)
}

pub(in crate::tests) fn partnership_orphan_income_statement() -> crate::K1PartnershipDocument {
    use crate::tests::cross::interview::k1_1065;
    let mut paper: crate::K1PartnershipDocument =
        k1_1065("Synthetic missing attached amount").into();
    paper.statements.income.push(crate::PartnershipIncomeStatement {
        code: PartnerIncomeCode::Section1256,
        amount: None,
    });
    paper
}

pub(in crate::tests) fn partnership_uncalculated_code_papers() -> [crate::K1PartnershipDocument; 11]
{
    use crate::tests::cross::interview::k1_1065;
    let mut deduction: crate::K1PartnershipDocument = k1_1065("Synthetic coded statements").into();
    deduction.form.deductions.push(crate::K1CodeRow {
        code: "U".into(),
        amount: Some(99),
        text: "Actual attached statement".into(),
        statement: true,
    });
    let mut activities: crate::K1PartnershipDocument =
        k1_1065("Synthetic two-activity statement").into();
    for (code, amount) in [("B", 100), ("C", 200)] {
        activities.form.self_employment.push(crate::K1CodeRow {
            code: code.into(),
            amount: Some(amount),
            text: "Two activities".into(),
            statement: true,
        });
    }
    let mut qbi: crate::K1PartnershipDocument = k1_1065("Synthetic missing QBI statement").into();
    qbi.statements.qbi = crate::PartnerQbi::NoStatement;
    let mut k3: crate::K1PartnershipDocument = k1_1065("Synthetic missing K-3").into();
    k3.form.k3_attached = true;
    let mut equivalents: crate::K1PartnershipDocument =
        k1_1065("Synthetic dividend equivalents").into();
    equivalents.form.dividend_equivalents = Some(12_345);
    let mut passive: crate::K1PartnershipDocument =
        k1_1065("Synthetic passive-activity statement reference").into();
    passive.form.multiple_passive = true;
    let mut self_employment: crate::K1PartnershipDocument =
        k1_1065("Synthetic unclassified self-employment row").into();
    self_employment.form.self_employment.push(crate::K1CodeRow {
        code: "D".into(),
        amount: Some(123),
        text: "Actual printed row".into(),
        statement: true,
    });
    let mut exempt: crate::K1PartnershipDocument =
        k1_1065("Synthetic unclassified exempt row").into();
    exempt.form.exempt.push(crate::K1CodeRow {
        code: "D".into(),
        amount: Some(123),
        text: "Actual printed row".into(),
        statement: true,
    });
    let mut investment_income: crate::K1PartnershipDocument =
        k1_1065("Synthetic printed investment income total").into();
    investment_income.form.information.push(crate::K1CodeRow {
        code: "A".into(),
        amount: Some(123),
        text: "Investment income".into(),
        statement: false,
    });
    let mut investment_expenses: crate::K1PartnershipDocument =
        k1_1065("Synthetic printed investment expense total").into();
    investment_expenses.form.information.push(crate::K1CodeRow {
        code: "B".into(),
        amount: Some(123),
        text: "Investment expenses".into(),
        statement: false,
    });
    let mut contributed: crate::K1PartnershipDocument =
        k1_1065("Synthetic printed property contribution").into();
    contributed.form.contributed_built_in_property = Some(true);
    [
        deduction,
        activities,
        qbi,
        k3,
        equivalents,
        passive,
        self_employment,
        exempt,
        investment_income,
        investment_expenses,
        contributed,
    ]
}

pub(in crate::tests) fn partnership_property_total_papers()
-> [(bool, crate::K1PartnershipDocument); 4] {
    use crate::tests::cross::interview::k1_1065;
    let rate = crate::EnergyRate {
        increased_rate: false,
        domestic_content: false,
        energy_community: false,
    };
    let properties = [
        crate::K1InvestmentProperty::RehabilitationRental {
            expenditures: 12_345,
            placed_year: 2026,
            id: 0,
            own_form: None,
        },
        crate::K1InvestmentProperty::RehabilitationOther {
            expenditures: 12_345,
            placed_year: 2026,
            id: 0,
            own_form: None,
        },
        crate::K1InvestmentProperty::Energy {
            basis: 12_345,
            rate: rate.clone(),
            id: 0,
            own_form: None,
        },
        crate::K1InvestmentProperty::CleanElectricity {
            basis: 12_345,
            rate,
            storage: false,
            id: 0,
            own_form: None,
        },
    ];
    let mut index = 0;
    properties.map(|property| {
        let credit_box = index == 0;
        index += 1;
        let mut paper: crate::K1PartnershipDocument =
            k1_1065("Synthetic actual investment-property statement").into();
        push_partnership_property(&mut paper, property);
        (credit_box, paper)
    })
}
