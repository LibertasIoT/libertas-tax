use crate::tests::*;

pub(in crate::tests) fn partner_basis_record() -> PartnerBasisRecord {
    PartnerBasisRecord {
        opening: 0,
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
    basis.opening = 1000000;
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
        amt_section179_income_carry: None,
        reporting_period: None,
        section179_sales: alloc::vec::Vec::new(),
        qbi_aggregation_group: None,
        section179_carryover_sold: 0,
        schedule_k3: None,
        rental_facts: Default::default(),
        income_source: None,
        gross_personal_services: None,
        guaranteed_services: 0,
        guaranteed_capital: 0,
        health_guaranteed: 0,
        health_distributions: 0,
        alternative: Some(regular.clone()),
        regular,
        participation: PartnerParticipation::General { facts },
        trade_kind: ScorpTradeKind::Ordinary,
        limited_for_se: true,
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
        disposition: crate::K1Disposition::Kept,
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
    partner.limited_for_se = false;
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
    data.regular.basis.opening = 30000000;
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
        record.basis.opening = 10_000_000;
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
        |_, _, _| Some(Section179OtherLimitedDeduction { system, amount_cents: 0 }),
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
        page.interest =
            PartnerInterest::General { participation: Participation::Over500Hours { hours: 600 } };
        page.basis.start = start.clone();
        page.qbi = PartnerQbi::NoStatement;
        crate::tests::cross::interview::document(0, IncomeDocument::K1Partnership(page))
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
            Some(crate::Section179OtherLimitedDeduction { system, amount_cents: 0 })
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
    page.interest =
        PartnerInterest::General { participation: Participation::Over500Hours { hours: 600 } };
    page.basis.start = k1_1065_continuing(
        10_000_000,
        ScorpRiskStart::Detailed { capacity: 10_000_000, profit: 0, loss: 0 },
    );
    page.qbi = PartnerQbi::NoStatement;
    partner_benefits_session(vec![page])
}
