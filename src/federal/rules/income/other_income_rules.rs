// other_income items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_hobby_sales(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalHobbySales,
) -> Result<(), CollectFailure> {
    let Source { context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    crate::require_localized!(
        r,
        data.inventory.eligible(),
        context,
        source,
        "ISSUE_RESOLVE_HOBBY_VERSUS_BUSINESS_ASSET_CLASSIFICATION_AND_INVENTORY_AC_5BD6F771",
        "Resolve the inventory accounting before reporting these hobby sales.",
    );
    t.hobby += data.income();
    Ok(())
}

pub(super) fn collect_health_loan_assistance(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::HealthLoanAssistance,
) -> Result<(), CollectFailure> {
    let Source { e, context, .. } = *src;
    if data.confirmed {
        data.report(r, e.id);
        t.health_loan_interest += data.interest;
    } else {
        // Unconfirmed program assistance is not excluded: it is
        // canceled-debt income, and the interest it paid stays
        // in the student-loan interest deduction.
        note_once_scoped(
            r,
            Message::new(
                "WARN_HEALTH_LOAN_ASSISTANCE_ASSUMED_TAXABLE",
                alloc::vec![crate::interview::income_display_arg(e)],
            ),
            context.clone(),
        );
        t.health_assistance += data.amount;
    }
    Ok(())
}

pub(super) fn collect_personal_debt(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::PersonalDebtSource,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, owner, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    let data = &crate::PersonalDebtCancellation::from(data);
    let ssn = d
        .people
        .as_ref()
        .and_then(|p| if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() })
        .is_some_and(|p| p.valid_ssn);
    let taxable = data.taxable(
        r,
        e.id,
        ssn,
        e.owner.owned(),
        #[cfg(test)]
        source,
    );
    t.personal_debt += taxable;
    t.injured_other_sources.insert(e.id, Some(exact::whole(taxable)));
    for (index, sale) in data
        .sales(
            d,
            r,
            e.id,
            #[cfg(test)]
            source,
        )
        .iter()
        .enumerate()
    {
        line_cents(
            r,
            &if index == 0 && data.sale().is_some() {
                alloc::format!("debt_{}_sale_gain", e.id)
            } else {
                alloc::format!("debt_{}_transfer_gain", e.id)
            },
            sale.gain(),
            &["income.personal_debt"],
            "https://www.irs.gov/taxtopics/tc409",
            msg!("EXPLAIN_DEBT_N_SALE_GAIN"),
        );
        collect_sale(
            t,
            sale,
            e,
            r,
            context,
            #[cfg(test)]
            source,
        );
    }
    Ok(())
}

pub(super) fn collect_property_acquisition(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::PropertyAcquisitionSource,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let Source { source, .. } = *src;
    let data = &crate::PropertyAcquisition::from(data);
    if let Some(sale) = data.sale(
        d,
        r,
        e.id,
        #[cfg(test)]
        source,
    ) {
        line_cents(
            r,
            &alloc::format!("acquisition_{}_gain", e.id),
            sale.gain(),
            &["income.personal_debt"],
            "https://www.irs.gov/publications/p4681",
            msg!("EXPLAIN_ACQUISITION_N_GAIN"),
        );
        collect_sale(
            t,
            &sale,
            e,
            r,
            context,
            #[cfg(test)]
            source,
        );
    }
    Ok(())
}

pub(super) fn collect_gambling(
    _cx: &mut Collect<'_>,
    _src: &Source<'_, '_>,
    t: &mut Totals,
    _r: &mut FederalResult,
    data: &crate::GamblingSource,
) -> Result<(), CollectFailure> {
    let v = FederalGambling::from(data);
    t.gambling += v.winnings;
    t.gambling_losses += v.losses;
    t.withholding_other += v.withholding;
    Ok(())
}

pub(super) struct OtherIncomeIn {
    pub(super) medicaid_waiver_excluded: i64,
    pub(super) medical_earnings: i64,
}

pub(super) struct OtherIncomeOut {
    pub(super) care_benefits: i64,
    pub(super) personal_debt: i64,
    pub(super) prizes: i64,
    pub(super) alaska: i64,
    pub(super) olympic_prize: i64,
    pub(super) digital: i64,
    pub(super) deferred_pension: i64,
    pub(super) inmate_pay: i64,
    pub(super) other_8z: i64,
    pub(super) not_for_profit: i64,
    pub(super) other_line_8: i64,
    pub(super) casualty_living_insurance: i64,
    pub(super) taxed_relief: i64,
    pub(super) health_assistance: i64,
    pub(super) scholarship_inclusions: [i64; 2],
    pub(super) scholarship: i64,
    pub(super) jury: i64,
    pub(super) jury_remitted: i64,
    pub(super) taxable_grants: i64,
    pub(super) family_leave: i64,
}

pub(super) fn other_income_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    i: OtherIncomeIn,
) -> OtherIncomeOut {
    let &Pass { c, .. } = pass;
    let OtherIncomeIn { medicaid_waiver_excluded, medical_earnings } = i;
    let care_benefits = dollars(t.care_benefits);
    line(
        r,
        "taxable_ltc_benefits",
        care_benefits,
        &["income.ltc_benefits"],
        "https://www.irs.gov/instructions/i8853",
        msg!("EXPLAIN_TAXABLE_LTC_BENEFITS"),
    );
    let personal_debt = dollars(t.personal_debt);
    line(
        r,
        "personal_debt_cancellation",
        personal_debt,
        &["income.personal_debt"],
        "https://www.irs.gov/publications/p4681",
        msg!("EXPLAIN_PERSONAL_DEBT_CANCELLATION"),
    );
    let prizes = dollars(t.prizes);
    line(
        r,
        "taxable_prizes",
        prizes,
        &["income.other_income"],
        "https://www.irs.gov/publications/p525",
        msg!("EXPLAIN_TAXABLE_PRIZES"),
    );
    // Schedule 1 lines 8g, 8m, 8t, 8u, 8v and 8z that have no form of their own
    // (Other income and the Form 1099 destinations), 8j beyond hobby goods, and
    // the Form 1099-K amounts that were not income (the top of Schedule 1).
    // Line 8m is shown in full; line 24c takes the excluded part below.
    let alaska = dollars(t.alaska);
    let olympic_prize = dollars(t.olympic);
    let digital = dollars(t.digital);
    let deferred_pension = dollars(t.deferred_pension.iter().sum());
    let inmate_pay = dollars(t.inmate_pay.iter().sum());
    let other_8z = dollars(t.other_8z + t.recovery_8z);
    let not_for_profit = dollars(t.not_for_profit);
    emit_lines(
        r,
        &mut (),
        [
            ("8g", "alaska_dividends", alaska, msg!("EXPLAIN_ALASKA_DIVIDENDS")),
            ("8m", "olympic_prizes", olympic_prize, msg!("EXPLAIN_OLYMPIC_PRIZES")),
            ("8t", "deferred_pension", deferred_pension, msg!("EXPLAIN_DEFERRED_PENSION")),
            ("8u", "inmate_pay", inmate_pay, msg!("EXPLAIN_INMATE_PAY")),
            ("8v", "digital_asset_income", digital, msg!("EXPLAIN_DIGITAL_ASSET_INCOME")),
        ],
        |(reference, id, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Schedule1, reference)),
                id,
                LineValue::MoneyDollars(amount),
                &["income.other_income", "income.wages"],
                "https://www.irs.gov/instructions/i1040gi",
                explanation,
            ))
        },
    );
    // The top-of-Schedule 1 entry has no line number: a plain row.
    line(
        r,
        "form_1099k_not_income",
        dollars(t.not_income_1099k),
        &["income.form_1099k"],
        "https://www.irs.gov/instructions/i1040gi",
        msg!("EXPLAIN_FORM_1099K_NOT_INCOME"),
    );
    // The new lines other than the Olympic prize, whose exclusion is decided
    // once the adjusted gross income without it is known.
    let other_line_8 = alaska + deferred_pension + inmate_pay + digital + other_8z + not_for_profit
        - medicaid_waiver_excluded;
    form_line(
        r,
        FormLine::new(TaxForm::Schedule1, "8s"),
        "medicaid_waiver_excluded",
        -medicaid_waiver_excluded,
        &["income.wages", "income.other_income"],
        "https://www.irs.gov/instructions/i1040gi",
        msg!("EXPLAIN_MEDICAID_WAIVER_EXCLUDED"),
    );
    let casualty_living_insurance = dollars(t.casualty_living_insurance);
    line(
        r,
        "taxable_casualty_living_insurance",
        casualty_living_insurance,
        &["income.casualty_living_insurance"],
        "https://www.irs.gov/publications/p547",
        msg!("EXPLAIN_TAXABLE_CASUALTY_LIVING_INSURANCE"),
    );
    let taxed_relief = dollars(t.taxed_relief);
    line(
        r,
        "taxable_relief_payments",
        taxed_relief,
        &["income.wildfire_relief", "income.disaster_relief"],
        "https://www.irs.gov/instructions/i1040s1",
        msg!("EXPLAIN_TAXABLE_RELIEF_PAYMENTS"),
    );
    // Unconfirmed health-service loan repayment assistance: Schedule 1 line 8z.
    let health_assistance = dollars(t.health_assistance);
    line(
        r,
        "taxable_health_loan_assistance",
        health_assistance,
        &["income.health_loan_assistance"],
        "https://www.irs.gov/publications/p970",
        msg!("EXPLAIN_TAXABLE_HEALTH_LOAN_ASSISTANCE"),
    );
    // All are Schedule 1 line 8z other income; every total below adds
    // them together.
    let casualty_living_insurance =
        casualty_living_insurance + taxed_relief + medical_earnings + health_assistance;
    // Pub. 970 chapter 2: the part of the box 5 grant that a filer or spouse
    // student includes in income to raise the education credit is taxable
    // scholarship; a dependent student reports it on their own return.
    let mut scholarship_inclusions = [0; 2];
    for e in &c.education_inputs() {
        if matches!(e.student, -1 | -2) {
            let included = e.expenses.included();
            t.scholarship += included;
            scholarship_inclusions[usize::from(e.student == -2)] += included;
        }
    }
    let scholarship = dollars(t.scholarship);
    form_line(
        r,
        FormLine::new(TaxForm::Schedule1, "8r"),
        "taxable_scholarships",
        scholarship,
        &["income.scholarship"],
        "https://www.irs.gov/publications/p970",
        msg!("EXPLAIN_TAXABLE_SCHOLARSHIPS"),
    );
    let jury = dollars(t.jury);
    let jury_remitted = dollars(t.jury_remitted);
    form_line(
        r,
        FormLine::new(TaxForm::Schedule1, "8h"),
        "jury_pay",
        jury,
        &["income.jury_duty.received"],
        "https://www.irs.gov/publications/p525",
        msg!("EXPLAIN_JURY_PAY"),
    );
    form_line(
        r,
        FormLine::new(TaxForm::Schedule1, "24a"),
        "jury_pay_remitted",
        jury_remitted,
        &["income.jury_duty.remitted"],
        "https://www.irs.gov/publications/p525",
        msg!("EXPLAIN_JURY_PAY_REMITTED"),
    );
    let taxable_grants = dollars(t.taxable_grants);
    line(
        r,
        "taxable_grants_1099g",
        taxable_grants,
        &["income.unemployment"],
        "https://www.irs.gov/instructions/i1099g",
        msg!("EXPLAIN_TAXABLE_GRANTS_1099G"),
    );
    let family_leave = dollars(t.family_leave);
    line(
        r,
        "family_leave_1099g",
        family_leave,
        &["income.unemployment"],
        "https://www.irs.gov/instructions/i1099g",
        msg!("EXPLAIN_FAMILY_LEAVE_1099G"),
    );
    OtherIncomeOut {
        care_benefits,
        personal_debt,
        prizes,
        alaska,
        olympic_prize,
        digital,
        deferred_pension,
        inmate_pay,
        other_8z,
        not_for_profit,
        other_line_8,
        casualty_living_insurance,
        taxed_relief,
        health_assistance,
        scholarship_inclusions,
        scholarship,
        jury,
        jury_remitted,
        taxable_grants,
        family_leave,
    }
}

pub(super) struct PersonalSalesReportingOut {
    pub(super) hobby: i64,
}

pub(super) fn personal_sales_reporting_phase(
    r: &mut FederalResult,
    t: &Totals,
) -> PersonalSalesReportingOut {
    line(
        r,
        "personal_sale_disallowed_loss",
        dollars(t.personal_sale_loss),
        &["income.sales"],
        "https://www.irs.gov/businesses/what-to-do-with-form-1099-k",
        msg!("EXPLAIN_PERSONAL_SALE_DISALLOWED_LOSS"),
    );
    let hobby = dollars(t.hobby);
    // Confirmed hobby goods income is property income, not compensation.
    // It still participates in AGI/MAGI; the IRA engine uses its own earned pool.
    // Not-for-profit Form 1099 amounts join line 8j but are not
    // net investment income.
    form_line(
        r,
        FormLine::new(TaxForm::Schedule1, "8j"),
        "hobby_sales_income",
        hobby + dollars(t.not_for_profit),
        &["income.hobby_sales"],
        "https://www.irs.gov/pub/taxpros/2023ntf-11-comingled-1099-k.pdf",
        msg!("EXPLAIN_HOBBY_SALES_INCOME"),
    );
    PersonalSalesReportingOut { hobby }
}
