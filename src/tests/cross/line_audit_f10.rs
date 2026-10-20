use crate::tests::support::builders::survey::survey_uncatalogued_document as unanswered_document;
// Fix unit F10 (retirement, PTC and penalty line audit, 2026-10-10): one regression per
// defect, with the audit's fixture and its hand-worked expected figures.
use super::*;
extern crate std;

fn amt(r: &FederalResult, id: &str) -> Option<i64> {
    r.lines.iter().find(|l| l.id == id).map(|l| l.amount)
}
fn qdd(
    amount: i64,
    distributed: u32,
    repaid: i64,
    repaid_in_return_year: i64,
) -> DisasterDistribution {
    DisasterDistribution::Qualified {
        fema: 4900,
        began: 20260310,
        declared: 20260320,
        distributed,
        amount,
        earlier: 0,
        spread: DisasterSpread::AllInReturnYear,
        repaid: crate::DisasterRepayment::Yes {
            total: repaid,
            in_return_year: repaid_in_return_year,
        },
    }
}

#[test]
fn d8606_1_line_6_is_less_a_current_year_repayment_of_a_current_year_disaster_distribution() {
    // i8606 (2026 draft) Line 6 example plus $10,000 prior basis: a $20,000 qualified
    // disaster distribution on May 6, 2026, $10,000 repaid on November 25, 2026, value
    // $50,000 at year end. Line 6 = 40,000; line 9 = 60,000; line 12 = 20,000 x 10,000 /
    // 60,000 = 3,333; line 14 = 6,667; line 15a = 15b = 16,667.
    let mut d = complete(FederalStatus::Single);
    let mut e = basis_distribution(Owner::Taxpayer, 20_000, RetirementTreatment::Early);
    if let FederalIncome::Retirement { data } = &mut e.income {
        data.box7 = crate::DistributionCodes::CODE_1.0;
        data.disaster = qdd(2_000_000, 20260506, 1_000_000, 1_000_000);
    }
    add(&mut d, e);
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 10_000, 50_000)];
    let r = supported(&d);
    // Form 8606 line 13 (basis recovered) $3,333 and line 14 (carried forward) $6,667,
    // not $2,857 and $7,143 from the unreduced $50,000 value.
    assert_eq!(amt(&r, "taxpayer_ira_basis_recovered"), Some(333_300));
    assert_eq!(amt(&r, "taxpayer_ira_basis_carryforward"), Some(666_700));
    assert_eq!(amt(&r, "taxpayer_ira_basis_taxable"), Some(1_666_700));
    // Form 8915-F lines 20/22/24 = 16,667; line 25 is the repaid share of that taxable
    // amount (10,000 x 16,667 / 20,000 = 8,333.50); line 26 is the rest.
    assert_eq!(amt(&r, "form_8915_taxpayer_2026_line_22"), Some(1_666_700));
    assert_eq!(amt(&r, "form_8915_taxpayer_2026_line_25"), Some(833_350));
    assert_eq!(amt(&r, "form_8915_taxpayer_2026_line_26"), Some(833_350));
    assert_eq!(return_line(&r, "4b"), 833_400);
    // A repayment made after December 31, 2026 is not in the year-end value.
    if let FederalIncome::Retirement { data } = &mut d.income[1].income {
        data.disaster = qdd(2_000_000, 20260506, 1_000_000, 0);
    }
    let r = supported(&d);
    assert_eq!(amt(&r, "taxpayer_ira_basis_recovered"), Some(285_700));
}

#[test]
fn d8915_1_final_return_remainder_for_a_prior_year_form_of_an_earlier_disaster() {
    // A filer who died on 2026-10-15 spread a 2025 qualified distribution for a 2024
    // disaster: 2025 Form 8915-F (2024 disasters) line 22 = $5,000, so 2025, 2026 and 2027
    // each take $5,000. The final return takes the 2026 installment and the 2027 remainder.
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().disaster_carries = vec![DisasterCarry {
        owner: Owner::Taxpayer,
        form: EarlierForm8915 {
            year: Form8915Year::Filed2025 { line_13: 0, line_14: 0, line_24: 500_000, line_25: 0 },
            disaster_year: 2024,
            line_11: 0,
            line_22: 500_000,
            repayments: Form8915Repayments::default(),
        },
    }];
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, estate_medical: 0, date: 20261015, ordinary: true });
    let r = estimate(&d);
    // Worksheet 4: 2025 line 22 is 2026's installment, the same again is the 2027 remainder.
    assert_eq!(amt(&r, "form_8915_taxpayer_2024_line_23"), Some(500_000));
    assert_eq!(amt(&r, "form_8915_taxpayer_2024_line_24"), Some(1_000_000));
    assert_eq!(amt(&r, "form_8915_taxpayer_2024_line_26"), Some(1_000_000));
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    // Alive: the 2027 installment waits for 2027.
    d.people.as_mut().unwrap().taxpayer.died = None;
    let r = estimate(&d);
    assert_eq!(amt(&r, "form_8915_taxpayer_2024_line_26"), Some(500_000));
}

#[test]
fn code_g_box_2a_is_taxable_for_a_rollover_to_a_roth() {
    // retirement-special-codes F2: a 401(k) Form 1099-R with box 1 30,000, box 2a 30,000
    // and code G (an in-plan Roth rollover). Line 5a 30,000, line 5b 30,000, and no
    // Form 5329 tax (sections 402A(c)(4)(B), 408A(d)(3)(A)(ii)).
    use crate::interview::document_income;
    use crate::tests::cross::interview::r1099;
    let code = DistributionCode::DirectRollover { to: RolloverTarget::Roth };
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19840701;
    add(
        &mut d,
        income(document_income(&IncomeDocument::R1099(r1099(3_000_000, code.clone(), false)))),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 3_000_000);
    assert_eq!(return_line(&r, "5b"), 3_000_000);
    assert_eq!(amt(&r, "early_distribution_tax").unwrap_or(0), 0);
    // The person chose the Roth destination, so there is nothing to assume and no note.
    assert!(!noted(&r, "WARN_CODE_G_BOX_2A_TAXED_AS_ROTH_ROLLOVER"));
    // (b) After-tax money sent to a Roth IRA prints box 2a 0: line 5a 10,000, line 5b 0.
    let mut after_tax = r1099(1_000_000, code, false);
    after_tax.taxable = 0;
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(document_income(&IncomeDocument::R1099(after_tax))));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 1_000_000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert!(!noted(&r, "WARN_CODE_G_BOX_2A_TAXED_AS_ROTH_ROLLOVER"));
}

fn ira_early(birth: u32, box7: u32, treatment: RetirementTreatment) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    add(
        &mut d,
        income(FederalIncome::Retirement {
            data: FederalRetirement {
                box_9b: None,
                box_10: 0,
                disability: DisabilityIncome::None,
                kind: RetirementKind::Ira,
                disability_wages: crate::DisabilityWages::None,
                gross: 1_000_000,
                taxable: RetirementTaxable::Payer { amount: 1_000_000 },
                withholding: 0,
                box7,
                ordinary_distribution: treatment,
                disaster: crate::DisasterDistribution::No,
                after_death: None,
            },
        }),
    );
    d
}
fn form_5329_line(r: &FederalResult, suffix: &str) -> i64 {
    amt(r, &alloc::format!("early_taxpayer_{suffix}")).unwrap_or(0) / 100
}

#[test]
fn d5329_1_exception_12_applies_at_59_and_a_half_whatever_the_code() {
    // Born 1960-01-01 (59 and a half on 2019-07-01): code 1 and code S are early only by
    // the payer's mistake. i5329 exception 12: line 1 $10,000, line 2 $10,000, line 4 $0.
    let r = supported(&ira_early(
        19600101,
        crate::DistributionCodes::CODE_1.0,
        RetirementTreatment::Early,
    ));
    assert_eq!(form_5329_line(&r, "taxable"), 10_000);
    assert_eq!(form_5329_line(&r, "exceptions"), 10_000);
    assert_eq!(form_5329_line(&r, "tax"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    let r = supported(&ira_early(
        19600101,
        crate::DistributionCodes::CODE_S.0,
        RetirementTreatment::EarlySimple,
    ));
    assert_eq!(form_5329_line(&r, "exceptions"), 10_000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    // A 46-year-old still owes the 10% (and 25% for code S).
    let r = supported(&ira_early(
        19800101,
        crate::DistributionCodes::CODE_1.0,
        RetirementTreatment::Early,
    ));
    assert_eq!(value(&r, "early_distribution_tax"), 1_000);
    let r = supported(&ira_early(
        19800101,
        crate::DistributionCodes::CODE_S.0,
        RetirementTreatment::EarlySimple,
    ));
    assert_eq!(value(&r, "early_distribution_tax"), 2_500);
}

#[test]
fn d5329_2_periodic_payments_exception_02_and_the_modification_refusal() {
    // Born 1976-03-01, IRA payments of $10,000 under a substantially equal series, code 2.
    let series = |eligible: bool, modified: bool| RetirementTreatment::Exception {
        second: None,
        distributed: 20260615,
        facts: RetirementException::PeriodicPayments {
            first_payment: 20230201,
            eligible,
            modified,
        },
        early_simple: false,
    };
    let r =
        supported(&ira_early(19760301, crate::DistributionCodes::CODE_2.0, series(true, false)));
    assert_eq!(form_5329_line(&r, "taxable"), 10_000);
    assert_eq!(form_5329_line(&r, "exceptions"), 10_000);
    assert_eq!(form_5329_line(&r, "tax"), 0);
    // No applies no exception: the 10% tax.
    let r =
        supported(&ira_early(19760301, crate::DistributionCodes::CODE_2.0, series(false, false)));
    assert_eq!(form_5329_line(&r, "tax"), 1_000);

    // A modification inside the five years adds a recapture tax this App cannot compute.
    let r = estimate(&ira_early(19760301, crate::DistributionCodes::CODE_2.0, series(true, true)));
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_SEPP_RECAPTURE"));
}

fn item(owner: Owner, document: IncomeDocument) -> IncomeDocumentItem {
    unanswered_document(owner, 0, document)
}
fn retirement_page(s: &mut Interview) -> P {
    match crate::tests::cross::interview::from_review(s, TaxSection::Retirement) {
        Reply::Edit(request @ P::SaveIncomes { .. }) => request,
        other => panic!("expected the retirement topic, got {other:?}"),
    }
}
fn save_documents_on_page(s: &mut Interview, documents: Vec<IncomeDocumentItem>) {
    let mut request = retirement_page(s);
    let P::SaveIncomes { documents: page, .. } = &mut request else { unreachable!() };
    page.documents = documents;
    let reply = send(s, request);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}

#[test]
fn d5329_3_a_disabled_roth_owner_has_form_5329_lines_1_and_2() {
    // Born 1980-01-01, code T for disability since 2025-01-01, $8,000 on 2026-03-10; Roth
    // history first year 2024 with $5,000 of regular contributions. Form 8606 line 25c is
    // $3,000: Form 1040 line 4b $3,000; Form 5329 line 1 $3,000, line 2 $3,000 (exception 03),
    // line 4 $0.
    let code = DistributionCode::RothIra {
        sequence: None,
        rolled_over: 0,
        roth: RothIraCode::ExceptionApplies {
            distributed: 20260310,
            reason: RothReason::Disabled { since: 20250101 },
        },
    };
    let roth = R1099 {
        state_details: Default::default(),
        gross: 800_000,
        taxable: 800_000,
        withholding: 0,
        box7: box7_of(&code),
        allowed_treatments: DistributionCodes::treatments(box7_of(&code), false, false),
        allowed_methods: vec![0],
        code,
        ira: false,
        trump: false,
        more: RMore::default(),
        disaster: DisasterDistribution::No,
        public_safety: vec![],
        after_death: None,
    };
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800101;
    let mut s = Interview::new(TEST_USER, d);
    save_documents_on_page(
        &mut s,
        vec![
            item(Owner::Taxpayer, IncomeDocument::R1099(roth)),
            item(
                Owner::Taxpayer,
                IncomeDocument::RothHistory(RothIraHistory {
                    first_year: 2024,
                    amounts: Some(RothOrdering { regular: 500_000, conversions: vec![] }),
                }),
            ),
        ],
    );
    let r = supported(&s.draft);
    assert_eq!(return_line(&r, "4b"), 300_000);
    assert_eq!(form_5329_line(&r, "taxable"), 3_000);
    assert_eq!(form_5329_line(&r, "exceptions"), 3_000);
    assert_eq!(form_5329_line(&r, "tax"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}

#[test]
fn d5329_4_code_s_kept_is_taxed_at_25_percent_and_a_simple_to_simple_rollover_is_tax_free() {
    // Born 1980-01-01, code S, box 1 and box 2a $10,000 (B11 F12). Kept: line 4b $10,000, line 4
    // $2,500. $6,000 rolled into another SIMPLE IRA (section 408(d)(3)(G)) is tax-free: line 4b
    // $4,000 and the 25% on it, $1,000.
    let r = supported(&ira_early(
        19800101,
        crate::DistributionCodes::CODE_S.0,
        RetirementTreatment::EarlySimple,
    ));
    assert_eq!(return_line(&r, "4b"), 1_000_000);
    assert_eq!(form_5329_line(&r, "taxable"), 10_000);
    assert_eq!(form_5329_line(&r, "tax"), 2_500);
    let r = supported(&ira_early(
        19800101,
        crate::DistributionCodes::CODE_S.0,
        RetirementTreatment::Rollover {
            destination: RolloverDestination::SimpleIra,
            amount: 600_000,
            early: true,
        },
    ));
    assert_eq!(return_line(&r, "4b"), 400_000);
    assert_eq!(form_5329_line(&r, "taxable"), 4_000);
    assert_eq!(form_5329_line(&r, "tax"), 1_000);
    assert_eq!(value(&r, "early_distribution_tax"), 1_000);
    // Code 1 keeps the rollover tax-free.
    let r = supported(&ira_early(
        19800101,
        crate::DistributionCodes::CODE_1.0,
        RetirementTreatment::Rollover {
            destination: RolloverDestination::Ira,
            amount: 600_000,
            early: true,
        },
    ));
    assert_eq!(return_line(&r, "4b"), 400_000);
    assert_eq!(form_5329_line(&r, "tax"), 400);
}

#[test]
fn d5329_5_a_first_year_minimum_is_not_a_current_year_shortfall() {
    // Born 1953-05-01 turns 73 in 2026: the first requirement is due April 1, 2027. The
    // December 31, 2025 balance $265,000 gives $10,000 (Table III 26.5); nothing taken by
    // December 31, 2026. Line 55 is $0 and the minimum is still shown.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19530501;
    let mut entry = rmd_record(0, 0);
    entry.required = RmdMinimum::OwnIras {
        period: RmdPeriod::Current,
        accounts: vec![RmdIra { balance: 26_500_000, rollover: 0, spouse: None }],
    };
    d.adjustments.as_mut().unwrap().required_distributions = vec![entry];
    let r = supported(&d);
    assert_eq!(value(&r, "rmd_0_account_0_minimum"), 10_000);
    assert_eq!(value(&r, "rmd_0_deferred_to_following_year"), 10_000);
    assert_eq!(value(&r, "required_distribution_tax"), 0);
    // The year after (born 1952) the same balance is due by December 31.
    d.people.as_mut().unwrap().taxpayer.birth_date = 19520501;
    let r = supported(&d);
    assert_eq!(value(&r, "required_distribution_tax"), 2_598);
}

#[test]
fn d8962_1_household_income_includes_a_form_8814_childs_income() {
    // Joint, wages 60,000, one child (born 2016-06-15) with 5,000 of interest on Form 8814:
    // line 4 5,000 (over 1,350), line 5 2,700, line 12 2,300, AGI 62,300. 2025 i8962 line 2b:
    // the lesser of line 4 and line 5 = 2,700, so line 3 = 65,000; line 4 = 26,650 (family of
    // 3); 243%; applicable 6.60% + 43 x 1.84 / 50 = 8.18% -> line 8a 5,317; premiums 1,500
    // and benchmark 1,200 for 12 months: line 24 = min(18,000, 14,400 - 5,317) = 9,083.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 6_000_000, 0)];
    let id = d.next_id;
    add_child(&mut d);
    d.dependents[0].birth_date = 20160615;
    add(&mut d, child_election(id, vec![child_interest(5_000)], 0, OtherParentChoice::NotNeeded));
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 1500, 1200, 0)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 62_300);
    assert_eq!(value(&r, "marketplace_household_income"), 65_000);
    assert_eq!(value(&r, "marketplace_annual_contribution"), 5_317);
    assert_eq!(value(&r, "premium_tax_credit"), 9_083);
}

#[test]
fn d8962_2_the_excess_advance_payment_is_form_8962_line_27() {
    // 2026 draft Form 8962: "27 Excess advance payment of PTC ... Enter the difference here
    // and on Schedule 2 (Form 1040), line 1a"; lines 28 and 29 are reserved.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 600, 500)).collect());
    let r = supported(&d);
    assert_eq!(value(&r, "premium_credit_repayment"), 866);
    assert_eq!(printed_line(&r, "premium_credit_repayment"), (TaxForm::Form8962, "27".into()));
}

fn ai_period(agi: i64) -> AnnualizedPeriod {
    AnnualizedPeriod {
        agi,
        itemized: 0,
        qbi: 0,
        additional: 0,
        self_employment: 0,
        social_wages: 0,
    }
}

#[test]
fn d2210_1_schedule_ai_line_18_takes_a_december_childs_credit_only_in_the_last_column() {
    // MFJ, wages $200,000 with no withholding; a child born 2026-12-01 (CTC $2,200); 2025 tax
    // $30,000 on AGI $180,000; Schedule AI AGI by period 35,000 / 60,000 / 100,000 / 200,000.
    // Line 18 is 0 / 0 / 0 / 2,200 (i2210 line 18: credits for events in the column's months).
    // Line 21 2,956.50 / 6,309.00 / 10,354.50 / 21,726.00; line 27 2,956.50 / 3,352.50 /
    // 4,045.50 / 11,371.50; with 6% / 7% / 7% / 7% the penalty is $755.64 (line 38 $756).
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 0)];
    d.next_id = 2;
    let mut kid = child(d.next_id);
    d.next_id += 1;
    kid.birth_date = 20261201;
    kid.residency = DependentResidency::WholeYearException;
    d.dependents.push(kid);
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 3_000_000,
        agi: Some(18_000_000),
        spouse_tax: 0,
    });
    pay.annualized = true;
    pay.annualized_income = Some(AnnualizedIncome {
        through_march: ai_period(3_500_000),
        through_may: ai_period(6_000_000),
        through_august: ai_period(10_000_000),
        full_year: ai_period(20_000_000),
    });
    let r = supported(&d);
    assert_eq!(amt(&r, "form_2210_ai_line_21_a"), Some(295_650));
    assert_eq!(amt(&r, "form_2210_ai_line_21_b"), Some(630_900));
    assert_eq!(amt(&r, "form_2210_ai_line_21_c"), Some(1_035_450));
    assert_eq!(amt(&r, "form_2210_ai_line_21_d"), Some(2_172_600));
    assert_eq!(amt(&r, "form_2210_line_19"), Some(75_564));
    assert_eq!(return_line(&r, "38"), 75_600);
}

#[test]
fn d2210_3_the_trump_account_line_63_tax_is_on_form_2210_line_2_and_form_1040_es_line_12b() {
    // Single, $81,100 wages, no withholding, a Trump account with $20 of line 63 tax and a $24
    // line 61 excise, 2025 tax $9,000. Line 2 $20 (the excise is not listed), line 4 $9,038,
    // line 9 $8,134.20 (90%), penalty $373.62. 1040-ES line 12b: the tax on Form 1040 line 24
    // less the listed items, $9,038.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20090704;
    d.income = vec![wage(1, Owner::Taxpayer, 8_110_000, 0)];
    d.screening.as_mut().unwrap().trump_accounts.push(TrumpAccount {
        owner: Owner::Taxpayer,
        contributions: 720_000,
        exempt: 120_000,
        returns: vec![TrumpExcessReturn {
            principal: 60_000,
            earnings: 2_000,
            received: 20261201,
            deadline: 20270415,
            confirmed: true,
        }],
        confirmed: true,
    });
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year =
        Some(PriorYearTax { full_year: true, tax: 900_000, agi: Some(10_000_000), spouse_tax: 0 });
    pay.next_year =
        NextYearEstimateRecord::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No };
    let r = supported(&d);
    assert_eq!(amt(&r, "form_2210_line_2"), Some(2_000));
    assert_eq!(amt(&r, "form_2210_line_4"), Some(903_800));
    assert_eq!(amt(&r, "form_2210_line_9"), Some(813_420));
    assert_eq!(amt(&r, "form_2210_line_19"), Some(37_362));
    assert_eq!(amt(&r, "form_1040_es_line_12b"), Some(903_800));
}

#[test]
fn d2210_4_household_tax_stays_on_line_2_when_estimated_payments_were_required() {
    // Single, $50,000 wages with no withholding, a household employer with $20,000 of wages
    // (Schedule H $3,102), 2025 tax $10,000, $1,000 paid on each due date. Without the
    // household tax line 4 is $3,823, over $1,000, so section 6654(e) does not apply and
    // section 3510(b)(2) does not exclude it: line 2 $3,102, line 4 $6,925, line 9 $6,232.50,
    // installments $1,558.13, penalty $102.54 (line 38 $103).
    let mut d = with_household(household_employer(20_000));
    d.income = vec![wage(1, Owner::Taxpayer, 5_000_000, 0)];
    d.next_id = 2;
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 1_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    pay.estimated_payments = [20260415, 20260615, 20260915, 20270115]
        .iter()
        .map(|&date| DatedPayment {
            payer: crate::PaymentPayer::Joint,
            refund_property: None,
            date,
            amount: 100_000,
        })
        .collect();
    pay.refresh_estimated_projection();
    let r = supported(&d);
    assert_eq!(amt(&r, "form_2210_line_2"), Some(310_200));
    assert_eq!(amt(&r, "form_2210_line_4"), Some(692_500));
    assert_eq!(amt(&r, "form_2210_line_9"), Some(623_250));
    assert_eq!(amt(&r, "form_2210_line_19"), Some(10_254));
    assert_eq!(return_line(&r, "38"), 10_300);
    // With no tax for a full 2025 year section 6654(e)(2) applies: the household tax is out of
    // line 2 and there is no penalty.
    d.payments.as_mut().unwrap().prior_year =
        Some(PriorYearTax { full_year: true, tax: 0, agi: Some(10_000_000), spouse_tax: 0 });
    let r = supported(&d);
    assert_eq!(amt(&r, "form_2210_line_2"), Some(0));
    assert_eq!(amt(&r, "form_2210_line_19"), Some(0));
}

#[test]
fn d2210f_2_the_two_thirds_test_counts_a_gain_netted_against_a_loss() {
    // Single, a Schedule F with $50,000 on line 9 and no expenses, a long-term sale with a
    // $40,000 gain and one with a $40,000 loss (Form 1040 line 7 is $0), 2025 tax $9,000.
    // Gross income is $90,000 and farming $50,000 (55.6% < 66 2/3%): Form 2210 applies. Line 4
    // $9,730, line 9 $8,757, installments $2,189.25, penalty $402.23 (line 38 $402).
    let mut d = complete(FederalStatus::Single);
    d.income = vec![];
    add(&mut d, income(raised_farm(50_000, 0)));
    add(&mut d, income(ordinary_investment_sale(50_000, 10_000, GainTerm::Long)));
    add(&mut d, income(ordinary_investment_sale(10_000, 50_000, GainTerm::Long)));
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year =
        Some(PriorYearTax { full_year: true, tax: 900_000, agi: Some(10_000_000), spouse_tax: 0 });
    pay.next_year = NextYearEstimateRecord::NotNeeded;
    let r = supported(&d);
    assert_eq!(amt(&r, "farming_fishing_gross_income_return_year"), Some(5_000_000));
    assert_eq!(amt(&r, "gross_income_return_year"), Some(9_000_000));
    assert_eq!(amt(&r, "form_2210_line_4"), Some(973_000));
    assert_eq!(amt(&r, "form_2210_line_9"), Some(875_700));
    assert_eq!(amt(&r, "form_2210_line_19"), Some(40_223));
    assert_eq!(return_line(&r, "38"), 40_200);
    assert!(amt(&r, "form_2210_f_line_16").is_none());
}

#[test]
fn a_child_born_in_december_with_all_days_alive_in_the_home_meets_the_residence_test() {
    // Born 2026-12-01 and in the home for the 31 days alive: the child tax credit, not the
    // $500 credit for other dependents. 10 of the 31 days is not more than half.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 8_000_000, 0)];
    d.next_id = 2;
    let mut kid = child(d.next_id);
    d.next_id += 1;
    kid.birth_date = 20261201;
    kid.residency = DependentResidency::Days { days: 31 };
    d.dependents.push(kid);
    let r = supported(&d);
    assert_eq!(value(&r, "child_credit"), 2_200);
    d.dependents[0].residency = DependentResidency::Days { days: 10 };
    let r = estimate(&d);
    assert_eq!(value(&r, "child_credit"), 500);
}
