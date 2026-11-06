use super::*;

#[test]
fn unit13_resolved_personal_gross_and_nol_preserve_opposite_native_categories() {
    for cents in [49, 50, 51] {
        let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
        let mut entry = income(personal_rent(0, 0));
        let FederalIncome::NonbusinessProperty { receipts, .. } = &mut entry.income else {
            panic!()
        };
        *receipts = cents;
        add(&mut draft, entry);
        source_owned_nol(&mut draft, 100, Owner::Spouse);
        let gross = i64::from(cents >= 50);
        assert_resolved_income_sources(
            draft,
            &[TaxSection::OtherIncome, TaxSection::SelfEmployment],
            [gross, -1],
            |result| {
                assert_eq!(value(result, "nonbusiness_personal_rent"), gross);
                assert_eq!(value(result, "prior_nol_deduction"), 1);
                assert_eq!(value(result, "agi"), 80_000 + gross - 1);
            },
        );
    }
}

#[test]
fn unit13_resolved_final_rental_loss_and_personal_gross_keep_native_owners() {
    for cents in [49, 50, 51] {
        let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
        // Actual residential basis275000 /27.5 yields10000 depreciation;
        // rent10000 minus1 paid expense minus10000 depreciation is−1.
        let property = rental(10_000, 1);
        add(&mut draft, income(FederalIncome::Rental { data: property }));
        let mut entry = income(personal_rent(0, 0));
        entry.owner = crate::DocumentOwner::Spouses;
        let FederalIncome::NonbusinessProperty { receipts, .. } = &mut entry.income else {
            panic!()
        };
        *receipts = cents;
        add(&mut draft, entry);
        let gross = i64::from(cents >= 50);
        assert_resolved_income_sources(
            draft,
            &[TaxSection::OtherIncome, TaxSection::Rentals],
            [-1, gross],
            |result| {
                assert_eq!(value(result, "schedule_e_rental_royalty_total"), -1);
                assert_eq!(value(result, "nonbusiness_personal_rent"), gross);
                assert_eq!(value(result, "agi"), 80_000 - 1 + gross);
            },
        );
    }
}

#[test]
fn unit13_resolved_ltc_and_nol_keep_separate_native_annual_boundaries() {
    for cents in [49, 50, 51] {
        let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
        let periods =
            [(20260601, 20260630, 4), (20260701, 20260731, 5)].map(|(starts, ends, source)| {
                let received = 1_500_000 + if source == 4 { cents } else { 0 };
                CareBenefitPeriod {
                    starts,
                    ends,
                    received,
                    services: 0,
                    reimbursements: 0,
                    recipient: CareBenefitRecipient::Insured { other_payees: false },
                    injured_payees: Some(InjuredLtcPeriodPayees::Payments {
                        payments: vec![InjuredLtcPeriodPayment { source, amount: received }],
                    }),
                }
            });
        add(
            &mut draft,
            income(ltc_record(LtcPeriods::Several {
                method: CarePeriodMethod::Contract,
                periods: periods.to_vec(),
            })),
        );
        let mut first = income(ltc_form(15_000, Some(3)));
        let FederalIncome::LtcBenefits { data } = &mut first.income else { panic!() };
        data.form.benefits = Some(data.form.benefits.unwrap_or(0) + cents);
        add(&mut draft, first);
        let mut second = income(ltc_form(15_000, Some(3)));
        second.owner = crate::DocumentOwner::Spouses;
        add(&mut draft, second);
        source_owned_nol(&mut draft, 377_100, Owner::Taxpayer);
        let extra = i64::from(cents >= 50);
        assert_resolved_income_sources(
            draft,
            &[TaxSection::OtherIncome, TaxSection::SelfEmployment],
            [2100 + extra - 3771, 1670],
            |result| {
                assert_eq!(value(result, "taxable_ltc_benefits"), 3770 + extra);
                assert_eq!(value(result, "prior_nol_deduction"), 3771);
                assert_eq!(value(result, "agi"), 79_999 + extra);
            },
        );
    }
}

#[test]
fn unit13_resolved_missing_nol_owner_is_not_a_known_zero_source() {
    let mut draft = injured_return((60_000, 5_000), (20_000, 500), 1_000, allocation());
    let mut entry = income(personal_rent(0, 0));
    let FederalIncome::NonbusinessProperty { receipts, .. } = &mut entry.income else { panic!() };
    *receipts = 50;
    add(&mut draft, entry);
    source_owned_nol(&mut draft, 100, Owner::Spouse);
    let NolHistory::Records { years, .. } = &mut draft.screening.as_mut().unwrap().nol else {
        panic!()
    };
    years[0].injured_owners = None;
    let result = supported(&draft);
    assert_eq!(value(&result, "nonbusiness_personal_rent"), 1);
    assert_eq!(value(&result, "prior_nol_deduction"), 1);
    assert_eq!(value(&result, "agi"), 80_000);
    assert_eq!(result.tax, Some(524_300));
    assert_eq!(result.refund, Some(125_700));
    assert_eq!(estimate(&restart(draft.clone()).draft), result);
    assert!(noted(&result, "ISSUE_8379_OTHER_INCOME_UNATTRIBUTABLE"));
    assert!(!result.lines.iter().any(|line| line.id == "form_8379_line_13b_b"));
}
