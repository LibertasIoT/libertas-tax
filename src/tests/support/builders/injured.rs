use crate::tests::cross::interview::document;
use crate::tests::*;

pub(in crate::tests) fn append_injured_two_period_ltc_sources(draft: &mut FederalDraft) {
    let periods =
        [(20260601, 20260630, 4), (20260701, 20260731, 5)].map(|(starts, ends, source)| {
            CareBenefitPeriod {
                starts,
                ends,
                received: 1_500_000,
                services: 0,
                reimbursements: 0,
                recipient: CareBenefitRecipient::Insured { other_payees: false },
                injured_payees: Some(InjuredLtcPeriodPayees::Payments {
                    payments: vec![InjuredLtcPeriodPayment { source, amount: 1_500_000 }],
                }),
            }
        });
    add(
        draft,
        income(ltc_record(LtcPeriods::Several {
            method: CarePeriodMethod::Contract,
            periods: periods.to_vec(),
        })),
    );
    add(draft, income(ltc_form(15_000, Some(3))));
    let mut spouse = income(ltc_form(15_000, Some(3)));
    spouse.owner = Owner::Spouse;
    add(draft, spouse);
}

pub(crate) const INJURED_INSURED_COUNT: usize = 12;
pub(crate) const INJURED_CONTRACT_DAYS: usize = 365;
pub(crate) const INJURED_DAILY_SOURCE: i64 = 50_000;
pub(crate) const INJURED_ANNUAL_SOURCE: i64 = 18_250_000;

pub(crate) fn injured_insured_documents() -> Vec<IncomeDocumentItem> {
    (0..INJURED_INSURED_COUNT)
        .map(|i| {
            document(
                0,
                IncomeDocument::LtcInsured(LtcInsured {
                    insured: LtcInsuredPerson::Other {
                        name: alloc::format!("Synthetic daily-contract insured {i}"),
                    },
                    periods: LtcPeriodsInput::One {
                        starts: 20260101,
                        ends: 20261231,
                        services: 0,
                        reimbursements: 0,
                        recipient: CareBenefitRecipient::Other {
                            insured_received: 0,
                            others_received: 0,
                        },
                    },
                }),
            )
        })
        .collect()
}

pub(crate) fn injured_daily_forms(rows: &mut Vec<IncomeDocumentItem>, ids: &[i64]) {
    for row in rows.iter_mut() {
        let IncomeDocument::LtcInsured(insured) = &mut row.document else {
            panic!("only actual insured rows before the forms")
        };
        insured.periods = LtcPeriodsInput::Several {
            method: CarePeriodMethod::Contract,
            periods: (0..INJURED_CONTRACT_DAYS)
                .map(|day| {
                    let date = crate::dates::add_days(20260101, day as i32);
                    CareBenefitPeriodInput {
                        starts: date,
                        ends: date,
                        received: 100_000,
                        services: 0,
                        reimbursements: 0,
                        recipient: CareBenefitRecipient::Other {
                            insured_received: 0,
                            others_received: 0,
                        },
                        injured_payees: None,
                    }
                })
                .collect(),
        };
    }
    for id in ids {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut source = document(
                0,
                IncomeDocument::Ltc1099(Ltc1099 {
                    benefits: INJURED_ANNUAL_SOURCE,
                    accelerated: 0,
                    interest: 0,
                    benefit_origin: Some(LtcPaymentOrigin::Insurance),
                    accelerated_origin: None,
                    paid: LtcPaidBasis::PerDiem,
                    contract: LtcContractPage::Qualified {
                        qualification: CareContractQualification::Qualified,
                    },
                    illness: LtcIllness::Chronic,
                    insured: Some(*id),
                }),
            );
            source.owner = owner;
            rows.push(source);
        }
    }
}

pub(crate) fn injured_period_payees(draft: &FederalDraft, rows: &mut [IncomeDocumentItem]) {
    for row in rows.iter_mut() {
        let IncomeDocument::LtcInsured(insured) = &mut row.document else {
            continue;
        };
        let forms = crate::care_benefits::payments(draft, row.id);
        assert_eq!(forms.len(), 2);
        assert_eq!(forms[0].1, INJURED_ANNUAL_SOURCE);
        assert_eq!(forms[1].1, INJURED_ANNUAL_SOURCE);
        assert_ne!(forms[0].0.owner, forms[1].0.owner);
        assert_ne!(forms[0].0.id, forms[1].0.id);
        assert!(forms.iter().all(|(e, _)| e.id > 0));
        let LtcPeriodsInput::Several { periods, .. } = &mut insured.periods else { unreachable!() };
        assert_eq!(periods.len(), INJURED_CONTRACT_DAYS);
        for period in periods {
            period.injured_payees = Some(InjuredLtcPeriodPayeesInput::Payments {
                payments: forms
                    .iter()
                    .map(|(e, _)| InjuredLtcPeriodPaymentInput {
                        source: e.id,
                        amount: INJURED_DAILY_SOURCE,
                    })
                    .collect(),
            });
        }
    }
}

// These fixtures contain only actual W-2/portfolio sources. All absent
// activity/account/credit source families are known empty, not guessed zero.
pub(in crate::tests) fn liability_portfolio_facts(
    d: &FederalDraft,
    capital: Option<[i64; 2]>,
) -> crate::injured_spouse::JointFacts<'_> {
    // Older synthetic contexts have no native retained-NIIT output packet.
    // They remain unknown, rather than acquiring an inferred zero.
    liability_portfolio_native_facts(d, capital, None)
}

// Each specialized fixture supplies its own captured/native packet after this
// ordinary companion seed. This is not permission to infer the specialized
// category from W-2 inputs or the return's printed total.
#[derive(Clone, Copy)]
pub(in crate::tests) enum SpecializedWageFixture {
    Inmate,
    Care,
    Adoption,
    Statutory,
}

pub(in crate::tests) fn liability_specialized_wage_facts<'a>(
    d: &'a FederalDraft,
    capital: Option<[i64; 2]>,
    specialization: SpecializedWageFixture,
) -> crate::injured_spouse::JointFacts<'a> {
    // Only the field whose actual output the caller handles is omitted from
    // the ordinary seed's closed-source guard. No estimate or Save uses this
    // companion draft; every other specialized/source guard remains intact.
    let mut companion = d.clone();
    for entry in &mut companion.income {
        if let FederalIncome::Wage { data } = &mut entry.income {
            match specialization {
                SpecializedWageFixture::Inmate => data.inmate_pay = 0,
                SpecializedWageFixture::Care => data.care_benefits = 0,
                SpecializedWageFixture::Adoption => {
                    data.other_box12.retain(|row| row.code != crate::W2Code::T);
                }
                SpecializedWageFixture::Statutory => data.statutory_employee = false,
            }
        }
    }
    let mut facts = liability_portfolio_native_facts(&companion, capital, None);
    // Preserve the original donor's ordinary W-2 source operand. The separate
    // inmate/care/adoption/statutory packet remains entirely the caller's;
    // absent or deliberately wrong packets are never repaired here.
    facts.w2_wages = liability_wage_source_cents(d);
    facts
}

fn liability_wage_source_cents(d: &FederalDraft) -> [i64; 2] {
    [Owner::Taxpayer, Owner::Spouse].map(|owner| {
        d.income
            .iter()
            .filter(|entry| entry.owner == owner)
            .map(|entry| match &entry.income {
                FederalIncome::Wage { data } if !data.statutory_employee => {
                    data.wages - data.deferred_pension - data.inmate_pay
                }
                _ => 0,
            })
            .sum()
    })
}

// The packet owns its arrays/boxes/maps and borrows only the static known-empty
// fixtures below. It does not borrow the input draft or native result.
pub(in crate::tests) fn liability_portfolio_native_facts(
    d: &FederalDraft,
    capital: Option<[i64; 2]>,
    native: Option<&FederalResult>,
) -> crate::injured_spouse::JointFacts<'static> {
    // The match below rejects LTC records; these fixtures have no LTC source.
    static NO_LTC_SOURCES: alloc::collections::BTreeMap<i64, i64> =
        alloc::collections::BTreeMap::new();
    let mut earned = [0; 2];
    let mut investment_sources = [[0; 2]; 2];
    let mut medicare_wages = [0; 2];
    let mut withholding = [[0; 2]; 3];
    assert!(d.dependents.is_empty());
    assert!(d.screening.as_ref().is_none_or(|s| s.clergy.is_empty()));
    for entry in &d.income {
        let owner = usize::from(entry.owner == Owner::Spouse);
        match &entry.income {
            FederalIncome::Wage { data } => {
                // This synthetic packet has no separately settled W-2
                // category outputs. Specialized wages need their native owner.
                assert!(
                    data.disability == crate::DisabilityIncome::None
                        && data.abroad == 0
                        && data.deferred_pension == 0
                        && data.inmate_pay == 0
                        && data.employee_tips.is_none()
                        && data.medicaid_waiver == 0
                        && data.care_benefits == 0
                        && !data.statutory_employee
                        && data.other_box12.iter().all(|row| {
                            row.amount == 0
                                || !matches!(row.code, crate::W2Code::II | crate::W2Code::T)
                        }),
                    "fixture requires its actual specialized W-2 source outputs"
                );
                earned[owner] += data.wages / 100;
                medicare_wages[owner] += data.medicare_wages;
                withholding[0][owner] += data.withholding;
            }
            FederalIncome::Interest { data } => {
                withholding[1][owner] += data.withholding;
                investment_sources[0][owner] += data.net_taxable();
            }
            FederalIncome::Dividend { data } => {
                withholding[1][owner] += data.withholding;
                investment_sources[1][owner] += data.ordinary;
            }
            FederalIncome::Sale { data } => withholding[1][owner] += data.withholding,
            FederalIncome::QualifiedSmallBusinessSale { data } => {
                withholding[1][owner] += data.withholding
            }
            _ => panic!("fixture requires its actual source owner's additional outputs"),
        }
    }
    // The exhaustive fixture match above rejects foreign-employer income;
    // screening has no Form2555 claim in these portfolio-only fixtures.
    assert!(d.screening.as_ref().is_none_or(|s| s.foreign_earned.is_empty()));
    extern crate std;
    static NO_FOREIGN: std::sync::LazyLock<crate::extended_rules::ForeignResult> =
        std::sync::LazyLock::new(Default::default);
    static NO_CHILD: std::sync::LazyLock<crate::federal_rules::ChildTotals> =
        std::sync::LazyLock::new(Default::default);
    static NO_K1_NII: std::sync::LazyLock<crate::injured_spouse::K1NiiSources> =
        std::sync::LazyLock::new(|| crate::injured_spouse::K1NiiSources {
            ordinary_cuts: Some([crate::kernel::carry::Amount::default(); 2]),
            ..Default::default()
        });
    static NO_RENTAL: std::sync::LazyLock<crate::extended_rules::RentalResult> =
        std::sync::LazyLock::new(|| crate::extended_rules::RentalResult {
            inputs_complete: true,
            farm_rental: 0, // The closed fixture match rejects every Form4835 source.
            ..Default::default()
        });
    // This helper's exhaustive whitelist above rejects every NUA, property,
    // K1 and estate current-exclusion source. Retain independent actual prior
    // facts and present native worksheet outputs; never default a missing row.
    let niit_retained = (|| {
        let native = native?;
        let mut prior_dollars = 0;
        for prior in &d.screening.as_ref()?.capital_history {
            prior_dollars += match prior.niit_difference {
                Some(amount) => crate::kernel::ratio::dollars(amount),
                None if crate::extended_rules::regular_capital_carry(prior) == [0; 2] => 0,
                None => return None,
            };
        }
        let source =
            |key: &str| native.lines.iter().find(|row| row.id == key).map(|row| row.amount / 100);
        Some(alloc::boxed::Box::new(crate::injured_spouse::niit_retained::RetainedNiitUse {
            prior_dollars,
            used_dollars: source("niit_disposition_adjustment")?,
            next_dollars: source("niit_capital_carry_adjustment")?,
            current_components_known_zero: true,
        }))
    })();
    // The screening guard excludes clergy and the closed income match rejects
    // pension records. Their known-empty annual categories now belong to the
    // same category packet as the actual native collector's contributions.
    use crate::injured_spouse::income_categories::{IncomeCategory, NativeIncomeCategories};
    let empty_categories = NativeIncomeCategories::collect(
        d,
        &[],
        &[
            (
                IncomeCategory::ClergyHousing,
                crate::kernel::ratio::exact::whole(0),
                0,
                Some([0; 2].map(crate::kernel::ratio::exact::whole)),
            ),
            (
                IncomeCategory::DisabilityPension,
                crate::kernel::ratio::exact::whole(0),
                0,
                Some([0; 2].map(crate::kernel::ratio::exact::whole)),
            ),
        ],
    );
    crate::injured_spouse::JointFacts {
        income_categories: alloc::boxed::Box::new(
            crate::injured_spouse::income_categories::NativeIncomeCategories {
                investment_sources: investment_sources
                    .map(|v| Some(v.map(crate::kernel::ratio::exact::whole))),
                //2026-10-14: this synthetic helper's exhaustive match above
                // rejects Hobby and every non-W2/portfolio source family.
                // Its genuinely empty Hobby category is explicitly known.
                niit_hobby: Some((
                    0,
                    core::array::from_fn(|_| crate::kernel::ratio::exact::whole(0)),
                )),
                niit_retained,
                // The exhaustive source whitelist above excludes all business/home fuel.
                fuel_credit_sources: Some(alloc::boxed::Box::new(
                    crate::injured_spouse::income_categories::NativeFuelCreditSources {
                        joint_cents: 0,
                        business_cents: [0; 2],
                        home_cents: 0,
                    },
                )),
                ..empty_categories
            },
        ),
        w2_wages: liability_wage_source_cents(d),
        k1_nii: &NO_K1_NII, // Fixture match rejects every K1/activity record.
        nii_rental: &NO_RENTAL,
        child: &NO_CHILD, // Exhaustive fixture match rejects child-election records.
        savings_bond_exclusion: Some([0; 2]), // Helper rejects Form8815 records.
        foreign: &NO_FOREIGN,
        foreign_business_housing: [0; 2],
        property_ordinary: Some([0; 2]),
        accounts: crate::injured_spouse::AccountSources::default(),
        // This helper rejects K-1 records above; there is no deferred tax source.
        deferred_compensation_tax: crate::injured_spouse::DeferredCompensationTaxSources {
            joint_dollars: 0,
            cents: [0; 2],
        },
        investment_recapture: [0; 2],
        mortgage_recapture: crate::mortgage_recapture::MortgageRecaptureSources {
            complete: true,
            ..Default::default()
        },
        // The baseline portfolio is unelected. Election fixtures supply the
        // native outputs and source capacity explicitly before projection.
        investment_election: crate::injured_spouse::InvestmentElectionSources {
            elected: [0; 2],
            capital: None,
        },
        unemployment: [0; 2], // The fixture match rejects unemployment records.
        k1_portfolio: [[0; 2]; 3],
        other_sources: alloc::collections::BTreeMap::new(),
        olympic: [0; 2],
        nii_substitute: [0; 2],
        capital: capital
            .map(|net| crate::injured_spouse::CapitalOwners::whole(net, [0; 2], [[0; 2]; 2])),
        eic_children: 0,
        ctc_ids: vec![],
        odc_ids: vec![],
        earned,
        se: [0; 2],
        half_se: [0; 2],
        received_alimony_recapture: [0; 2], // Fixture match rejects alimony.
        employee_expenses: [0; 2],
        ira: [0; 2],
        hsa: [0; 2],
        pension: [0; 2],
        vehicle_repayment: [0; 2],
        sep: [0; 2],
        health: [0; 2],
        educator: [0; 2],
        business: [0; 2],
        regular_nol: vec![],
        ltc: &[],
        nii_ltc_sources: &NO_LTC_SOURCES,
        fund_credit: [0; 2],
        qbi: None,
        care_sources: vec![],
        care_shared_expenses: 0,
        education_sources: &[],
        education_pools: [0; 3],
        withholding_categories: withholding,
        retirement: [[0; 2]; 2],
        nii_annuity: [0; 2],
        nii_savings_penalty: [0; 2],
        nii_royalties: 0,
        personal_expenses: Some([0; 2]), // Helper rejects personal-rental records above.
        medicare_withheld: [0; 2],
        excess_social: [0; 2],
        extension: 0,
        tips: [0; 2],
        overtime: [0; 2],
        medicare_wages,
        itemized: false,
    }
}

pub(in crate::tests) fn actual_liability_totals(
    r: &FederalResult,
) -> crate::injured_spouse::owners::LiabilityTotals {
    // Completed zero Schedule3 parts are omitted from the emitted form.
    let source =
        |id: &str| r.lines.iter().find(|row| row.id == id).map_or(0, |row| row.amount / 100);
    crate::injured_spouse::owners::LiabilityTotals {
        wages: return_line(r, "1a") / 100,
        interest: return_line(r, "2b") / 100,
        dividends: return_line(r, "3b") / 100,
        retirement: [return_line(r, "4b") / 100, return_line(r, "5b") / 100],
        social: return_line(r, "6b") / 100,
        capital: return_line(r, "7a") / 100,
        total_income: return_line(r, "9") / 100,
        adjustments: return_line(r, "10") / 100,
        standard_or_itemized: return_line(r, "12e") / 100,
        cash_charity: return_line(r, "12f") / 100,
        child_credit: return_line(r, "19") / 100,
        other_nonrefundable: return_line(r, "20") / 100
            - source("schedule_3_general_business_credit")
            - source("schedule_3_minimum_tax_credit")
            - source("schedule_3_tax_credit_bond_credit"),
        other_taxes: return_line(r, "23") / 100 + source("schedule_2_credit_repayments_total"),
        taxable: return_line(r, "15") / 100,
        qualified_dividends: return_line(r, "3a") / 100,
        calculation_amounts: [
            "account_spouse_tax",
            "account_taxpayer_tax",
            "additional_medicare_tax",
            "archer_contribution_1_deduction",
            "archer_contribution_2_deduction",
            "care_credit",
            "care_credit_before_prior",
            "care_expenses_smallest",
            "care_percent",
            "care_prior_worksheet_a_line10",
            "care_prior_worksheet_a_line12",
            "early_spouse_tax",
            "early_taxpayer_tax",
            "education_credit",
            "form_8960_deductions_and_modifications",
            "form_8960_net_investment_income",
            "jury_pay_remitted",
            "net_investment_income_tax",
            "niit_disposition_adjustment",
            "nonqualified_annuity_investment_income",
            "olympic_exclusion",
            "overtime_deduction",
            "prior_year_care_credit",
            "qbi_deduction",
            "rmd_0_tax",
            "rmd_1_tax",
            "schedule_1_alimony_paid",
            "schedule_1_archer_msa_deduction",
            "schedule_1_business_income",
            "schedule_1_early_withdrawal_penalty",
            "schedule_1_educator_expenses",
            "schedule_1_employee_business_expenses",
            "schedule_1_estate_deductions",
            "schedule_1_farm_income",
            "schedule_1_half_self_employment_tax",
            "schedule_1_hsa_deduction",
            "schedule_1_ira_deduction",
            "schedule_1_moving_expenses",
            "schedule_1_pension_contributions",
            "schedule_1_self_employed_health_insurance",
            "schedule_1_self_employed_retirement",
            "schedule_1_student_loan_interest",
            "schedule_2_credit_repayments_total",
            "schedule_2_premium_credit_repayment",
            "senior_deduction",
            "tips_deduction",
            "vehicle_interest_deduction",
        ]
        .into_iter()
        .map(|key| (key, source(key)))
        .collect(),
    }
}

pub(in crate::tests) fn allocation() -> InjuredSpouseAllocation {
    InjuredSpouseAllocation {
        injured: Owner::Taxpayer,
        debt: OffsetDebt::ChildSupport,
        obligated: false,
        community: InjuredCommunityFacts::No,
        student_loan_borrower: None,
        estimated: EstimatedSplit::IrsFormula,
        allocations: vec![],
        claimants: vec![],
    }
}

// A joint return: your W-2 and your spouse's W-2, with the injured spouse
// item chosen and the allocation answered.
pub(in crate::tests) fn injured_return(
    own: (i64, i64),
    spouse: (i64, i64),
    estimated: i64,
    a: InjuredSpouseAllocation,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![
        wage(1, Owner::Taxpayer, own.0 * 100, own.1 * 100),
        wage(2, Owner::Spouse, spouse.0 * 100, spouse.1 * 100),
    ];
    d.next_id = 3;
    d.payments.as_mut().unwrap().estimated = Some(estimated * 100);
    if estimated > 0 {
        d.payments.as_mut().unwrap().estimated_payments.push(crate::DatedPayment {
            date: 20260415,
            amount: estimated * 100,
            payer: PaymentPayer::Joint,
            refund_property: None,
        });
    }
    d.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
    d.injured_spouse = Some(a);
    d
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tests) enum RestRoundingSource {
    Interest,
    Dividend,
    Jury,
    Unemployment,
}

pub(in crate::tests) fn rest_rounding_source(
    kind: RestRoundingSource,
    owner: Owner,
    cents: i64,
) -> FederalIncomeEntry {
    let mut entry = match kind {
        RestRoundingSource::Interest => {
            income(FederalIncome::Interest { data: native_interest(cents, 0) })
        }
        RestRoundingSource::Dividend => {
            income(FederalIncome::Dividend { data: native_dividend(cents, 0, 0) })
        }
        RestRoundingSource::Jury => {
            income(FederalIncome::JuryDuty { data: JuryPay { received: cents, remitted: 0 } })
        }
        RestRoundingSource::Unemployment => {
            let mut entry = government_unemployment(0, &[], 0);
            let FederalIncome::Unemployment { data } = &mut entry.income else { unreachable!() };
            data.amount = cents;
            entry
        }
    };
    entry.owner = owner;
    entry
}

pub(in crate::tests) fn changed_filer_carry(draft: &mut FederalDraft, filer: CarryFiler) {
    draft.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
        carryforwards: vec![BusinessCreditCarryforward {
            credit: BusinessCredit::WorkOpportunity,
            origin_year: 2025,
            amount: 500_000,
            passive: None,
            filer,
            research_source: None,
        }],
        carrybacks: vec![],
        spouse_without_credits: false,
        credits: vec![],
    });
}

pub(in crate::tests) fn texas_interest_admission(withholding: i64) -> FederalDraft {
    let mut draft = complete(FederalStatus::Joint);
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    data.withholding = withholding;
    let mut allocation = InjuredSpouseAllocation::defaults();
    allocation.community =
        InjuredCommunityFacts::Yes { states: vec![7], debt_facts: InjuredDebtFacts::NotApplicable };
    draft.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
    draft.injured_spouse = Some(allocation);
    draft
}
