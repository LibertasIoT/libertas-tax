use super::*;

#[test]
fn long_term_care_pools_by_identity_never_by_equal_names() {
    // Row 213. You and your spouse type the same name; the premiums still pool per person, each
    // with the age limit from their own birth date (you 50: $930; your spouse 71: $6,200).
    let mut d = complete(FederalStatus::Joint);
    set_age(&mut d, -1, 50);
    set_age(&mut d, -2, 71);
    let people = d.people.as_mut().unwrap();
    people.taxpayer.first_name = "Pat".into();
    people.taxpayer.last_name = "Sample".into();
    let spouse = people.spouse.as_mut().unwrap();
    spouse.first_name = "Pat".into();
    spouse.last_name = "Sample".into();
    let expenses = itemized(d.deductions.as_mut().unwrap());
    expenses.medical = 2000000;
    expenses.long_term_care = vec![personal_care(-1, 700000), personal_care(-2, 700000)];
    let r = supported(&d);
    // The pools run in identity order: your spouse (-2), then you (-1).
    assert_eq!(value(&r, "long_term_care_1_medical"), 6200);
    assert_eq!(value(&r, "long_term_care_2_medical"), 930);
}

#[test]
fn unconfirmed_health_service_loan_assistance_is_other_income_not_canceled_debt() {
    // Row 263 (Pub. 970 chapter 5). Assistance not confirmed as excluded under section
    // 108(f)(4) is Schedule 1 line 8z other income, on its own line, and not line 8c.
    let mut d = complete(FederalStatus::Single);
    let mut entry = health_loan_assistance(HealthLoanProgram::National, 0);
    let FederalIncome::HealthLoanAssistance { data } = &mut entry.income else { panic!() };
    data.amount = 500_000;
    data.confirmed = false;
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_health_loan_assistance"), 5000);
    assert!(r.lines.iter().all(|l| l.id != "taxable_canceled_debt"));
}

#[test]
fn a_code_3_pension_asks_the_minimum_retirement_age_and_the_early_picker_does_not() {
    // Row 293. The wage part of a disability pension is stored on the record, from the code 3
    // answer; the Early picker's disability exception carries no such question.
    let code_3 = crate::tests::cross::interview::r1099(
        1_000_000,
        DistributionCode::Disability {
            began: 20240301,
            wages: DisabilityWages::Part { amount: 300_000 },
        },
        false,
    );
    let FederalIncome::Retirement { data } = code_3.into_source().income() else { panic!() };
    let data = FederalRetirement::from(&data);
    assert_eq!(data.disability_wages, DisabilityWages::Part { amount: 300_000 });
    assert!(matches!(
        data.ordinary_distribution,
        Some(RetirementTreatment::Exception {
            facts: RetirementException::Disability { began: 20240301 },
            ..
        })
    ));
    assert_eq!(data.disability_wages(1_000_000), 300_000);
    let early = crate::tests::cross::interview::r1099(
        1_000_000,
        DistributionCode::Early {
            what_happened: Disposition::Kept,
            exception: EarlyException::Applies {
                second: None,
                repaid: None,
                distributed: 20260601,
                facts: RetirementException::Disability { began: 20260101 },
            },
        },
        true,
    );
    let FederalIncome::Retirement { data } = early.into_source().income() else { panic!() };
    let data = FederalRetirement::from(&data);
    assert_eq!(data.disability_wages, DisabilityWages::None);
    assert_eq!(data.disability_wages(1_000_000), 0);
}

#[test]
fn a_disability_pension_before_minimum_retirement_age_does_not_demand_a_disabled_person() {
    // Row 294. Wages on line 1h from a code 3 pension are not a Schedule R claim: only the
    // Schedule R amount asks the People page to say disabled. The person here is not disabled
    // and claims no Schedule R credit, so the estimate stands.
    let mut d = complete(FederalStatus::Single);
    let mut e = income(disability_pension(17000));
    let data = e.income.r1099_source_mut().unwrap();
    data.disability = Some(DisabilityIncome::None);
    add(&mut d, e);
    assert!(!d.people.as_ref().unwrap().taxpayer.disabled);
    let r = supported(&d);
    assert!(!r.issues.iter().any(|i| i.section == "Disability retirement"));
    // The Schedule R amount still needs the People page.
    let data = d.income[1].income.r1099_source_mut().unwrap();
    data.disability = Some(DisabilityIncome::Qualifying { amount: 1_700_000, eligible: true });
    unsupported(&d, "Disability retirement");
}

#[test]
fn form_8379_allocates_the_amount_on_the_return_not_the_row_preview() {
    // Row 295. Line 13b allocates each document's own amount on the return: a confirmed
    // (excluded) health-service payment, a qualified relief payment and a hobby the estimate
    // leaves out allocate zero; an unconfirmed payment its amount.
    let relief = |qualified| FederalIncome::WildfireRelief {
        data: FederalWildfireRelief {
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            paid: 20260310,
            declared: 20250108,
            reference: "FEMA-FM-5000".into(),
            amount: 2_500_000,
            withholding: 50_000,
            qualified,
        },
    };
    let cents = |income: &FederalIncome| -> Vec<(crate::injured_spouse::Part13b, Option<i64>)> {
        crate::injured_spouse::parts_13b(income, false)
    };
    let excluded = cents(&relief(true));
    assert_eq!(excluded, vec![(crate::injured_spouse::Part13b::Rest, Some(0))]);
    let taxed = cents(&relief(false));
    assert_eq!(taxed.len(), 1);
    assert!(taxed[0].1.is_some_and(|amount| amount > 0));
    let FederalIncome::HealthLoanAssistance { mut data } =
        health_loan_assistance(HealthLoanProgram::National, 0).income
    else {
        unreachable!()
    };
    data.amount = 500_000;
    data.confirmed = true;
    assert_eq!(
        cents(&FederalIncome::HealthLoanAssistance { data: data.clone() }),
        vec![(crate::injured_spouse::Part13b::Rest, Some(0))]
    );
    data.confirmed = false;
    assert_eq!(
        cents(&FederalIncome::HealthLoanAssistance { data }),
        vec![(crate::injured_spouse::Part13b::Rest, Some(500_000))]
    );
}

#[test]
fn an_inherited_ira_payment_has_its_own_kind_and_is_taxed_as_box_2a_without_a_record() {
    // Rows 303 and 305. A 1099-R with code 4 and box 7b is the Inherited kind (an IRA for the
    // box 7b checkbox, never the filer's own IRA), and keeps the payer's box 2a: with no
    // Inherited IRA record to use, the payment is taxed as printed, with a note.
    let mut row = crate::tests::cross::interview::r1099(
        1_000_000,
        DistributionCode::Inherited { account: None },
        true,
    );
    row.source_mut().form.taxable = Some(900_000);
    let FederalIncome::Retirement { data } = row.clone().into_source().income() else { panic!() };
    let data = FederalRetirement::from(&data);
    assert_eq!(data.kind, RetirementKind::Inherited);
    assert!(data.kind.ira());
    assert_ne!(data.kind, RetirementKind::Ira);
    assert_eq!(
        (data.taxable.clone(), data.box_2a),
        (Some(RetirementTaxable::Inherited { account: None }), 900_000)
    );
    // The page reopens at box 2a as the payer printed it.
    let entry = income(row.clone().into_source().income());
    let Some(item) = crate::interview::document_item(&entry, &[], false) else {
        panic!("the document reopens")
    };
    let IncomeSource::Retirement { value: crate::RetirementDocument::R1099(reopened) } =
        item.document
    else {
        panic!()
    };
    assert_eq!((reopened.gross, reopened.taxable), (1_000_000, Some(900_000)));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(row.into_source().income()));
    let r = supported(&d);
    assert!(noted(&r, "WARN_INHERITED_IRA_RECORD_MISSING"), "{r:?}");
    assert_eq!(return_line(&r, "4a"), 1_000_000);
    assert_eq!(return_line(&r, "4b"), 900_000);
}

#[test]
fn the_sole_ira_branches_have_no_property_sale_and_the_general_one_keeps_its_rules() {
    // Row 306 and B11 R15. The SIMPLE IRA branch (code S) offers Kept or a rollover into another
    // SIMPLE IRA, the only tax-free move in the first two years; it keeps the 25% rate.
    assert_eq!(
        crate::tests::retirement::interview::retirement_treatment(
            DistributionCode::EarlySimple {
                what_happened: SimpleDisposition::Kept { exception: EarlyException::None }
            },
            true
        ),
        RetirementTreatment::EarlySimple
    );
    assert_eq!(
        crate::tests::retirement::interview::retirement_treatment(
            DistributionCode::EarlySimple {
                what_happened: SimpleDisposition::RolledOver { amount: 400_000 },
            },
            true
        ),
        RetirementTreatment::Rollover {
            destination: RolloverDestination::SimpleIra,
            amount: 400_000,
            early: true,
        }
    );
}
