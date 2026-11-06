// schedule_a items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(crate) fn salt_cap_current_year(status: FederalStatus, magi: i64) -> i64 {
    // Section 164(b)(7) phases down the full cap, floors it at $10,000, and
    // only then halves the result for a married-separate return.
    let threshold = if status == FederalStatus::Separate {
        crate::year_constants::SALT_PHASEDOWN_THRESHOLD_SEPARATE_DOLLARS
    } else {
        crate::year_constants::SALT_PHASEDOWN_THRESHOLD_DOLLARS
    };
    let full = phase::floor(
        crate::year_constants::SALT_REDUCED_CAP_FLOOR_DOLLARS,
        crate::year_constants::SALT_CAP_DOLLARS - fraction(positive(magi - threshold), 30, 100),
    );
    if status == FederalStatus::Separate { fraction(full, 1, 2) } else { full }
}
pub(crate) fn final_annuity_losses(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let mut total = 0;
    let Some(deductions) = &d.deductions else {
        return 0;
    };
    let Some(people) = &d.people else {
        return 0;
    };
    for loss in &deductions.annuity_losses {
        #[cfg(test)]
        let source = alloc::format!("Annuity loss: {}", loss.income_id);
        let Some(entry) = d.income.iter().find(|entry| entry.id == loss.income_id) else {
            crate::issue_localized!(
                r,
                &crate::IssueContext::income_id(loss.income_id).manual(),
                &source,
                "ISSUE_SELECT_THE_RETIREMENT_INCOME_ENTRY_FOR_THE_ENDED_CONTRACT_6F15745F",
                "Select the retirement income entry for the ended contract.",
            );
            continue;
        };
        let Some(data) = entry.income.retirement() else {
            crate::issue_localized!(
                r,
                &crate::interview::income_issue_context(entry).manual(),
                &source,
                "ISSUE_UNRECOVERED_ANNUITY_COST_MUST_LINK_TO_RETIREMENT_INCOME_42116AC6",
                "Unrecovered annuity cost must link to retirement income.",
            );
            continue;
        };
        let person = if entry.owner == crate::DocumentOwner::Yours {
            Some(&people.taxpayer)
        } else {
            people.spouse.as_ref()
        };
        let basis = data.pension_basis();
        let recovery = basis.as_ref().map_or_else(
            || {
                let earlier = match data.taxable {
                    Some(crate::RetirementTaxable::GeneralAnnuity { data: annuity }) => {
                        annuity.earlier_recovery()
                    }
                    Some(crate::RetirementTaxable::VariableAnnuity { data: annuity }) => {
                        annuity.earlier_recovery()
                    }
                    _ => 0,
                };
                earlier + data.gross - data.taxable_income()
            },
            |(basis, _)| {
                basis.earlier_recovery() + basis.remaining() - basis.remaining_after_payments()
            },
        );
        let basis_agrees = basis.as_ref().is_none_or(|(basis, _)| {
            let recovered = match basis.history {
                crate::PensionHistory::New { duration: crate::PensionDuration::Fixed { .. } } => {
                    return false;
                }
                crate::PensionHistory::New { .. } => 0,
                crate::PensionHistory::Prior { recovered, .. } => recovered,
            };
            basis.started == loss.started
                && basis.cost == Some(loss.cost)
                && recovered == loss.recovered
        });
        // Payments not confirmed to stop at death: no deduction.
        let confirmed = loss.eligible;
        let death_agrees =
            person.and_then(|p| p.death_date()).is_some_and(|date| date >= loss.started);
        let treatment_agrees = !data.kind.ira()
            && (data.ordinary_distribution == Some(RetirementTreatment::Ordinary)
                || matches!(data.ordinary_distribution, Some(RetirementTreatment::Exception {
                    distributed, facts: RetirementException::Death { died },
                    second: None,
                    early_simple: false,
                }) if died <= distributed && person.and_then(|p| p.death_date()).is_some_and(|date| distributed <= date)))
            && matches!(
                data.taxable,
                Some(crate::RetirementTaxable::Payer)
                    | Some(crate::RetirementTaxable::Simplified { .. })
                    | Some(crate::RetirementTaxable::GeneralAnnuity { .. })
                    | Some(crate::RetirementTaxable::VariableAnnuity { .. })
                    | Some(crate::RetirementTaxable::Railroad { .. })
                    | Some(crate::RetirementTaxable::ChildAnnuity { .. })
            );
        let worksheet_agrees = basis_agrees
            && match &data.taxable {
                Some(crate::RetirementTaxable::GeneralAnnuity { data: annuity }) => {
                    person
                        .and_then(|p| p.death_date())
                        .is_some_and(|death| annuity.ends_at_death(death))
                        && annuity.started == loss.started
                        && annuity.cost == loss.cost
                        && annuity.recovered == loss.recovered
                }
                Some(crate::RetirementTaxable::VariableAnnuity { data: annuity }) => {
                    let recovered = match annuity.history {
                        crate::VariableHistory::New { .. } => 0,
                        crate::VariableHistory::Prior { recovered, .. } => recovered,
                    };
                    person
                        .and_then(|p| p.death_date())
                        .is_some_and(|death| annuity.ends_at_death(death))
                        && annuity.started == loss.started
                        && annuity.cost == loss.cost
                        && recovered == loss.recovered
                }
                Some(crate::RetirementTaxable::ChildAnnuity { data: child }) => {
                    child.started == loss.started
                        && child.cost == loss.cost
                        && child.recovered == loss.recovered
                        && child.ended.is_some_and(|ended| {
                            person
                                .and_then(|p| p.death_date())
                                .is_some_and(|date| ended / 100 == date / 100)
                        })
                }
                _ => true,
            };
        let recovery_agrees =
            recovery >= 0 && recovery + loss.recovered + loss.refunds <= loss.cost;
        let valid = confirmed
            && loss.started > 19860701
            && death_agrees
            && treatment_agrees
            && worksheet_agrees
            && recovery_agrees;
        let source_context = crate::interview::income_issue_context(entry);
        let retirement_editable = matches!(
            data.taxable,
            Some(crate::RetirementTaxable::Payer)
                | Some(crate::RetirementTaxable::Simplified { .. })
        );
        let mut contexts = Vec::new();
        if !death_agrees || loss.started <= 19860701 {
            // The ordinary People editor has no actual death date; an old
            // starting date is a true unsupported contract, not a fact to change.
            contexts.push(source_context.clone().manual());
        }
        if !treatment_agrees {
            contexts.push(if retirement_editable {
                source_context.clone().corrective(crate::TaxSection::Retirement)
            } else {
                source_context.clone().manual()
            });
        }
        if !worksheet_agrees && death_agrees && !data.kind.ira() {
            contexts.push(if retirement_editable {
                source_context.clone().alternatives(&[
                    crate::TaxSection::OtherItemized,
                    crate::TaxSection::Retirement,
                ])
            } else {
                source_context.clone().corrective(crate::TaxSection::OtherItemized)
            });
        }
        if !recovery_agrees {
            contexts.push(if death_agrees && !data.kind.ira() {
                source_context.clone().corrective(crate::TaxSection::OtherItemized)
            } else {
                source_context.clone().manual()
            });
        }
        for context in contexts {
            crate::require_localized!(
                r,
                !confirmed,
                &context,
                &source,
                "ISSUE_CONFIRM_POST_JULY_1_1986_ANNUITIZATION_DEATH_OF_THE_LINKED_FINAL_AN_6DE6B545",
                "Confirm post-July 1, 1986 annuitization, death of the linked final annuitant, ordinary or inherited periodic pension/annuity treatment and complete nonduplicated investment recovery. Payments must cease because of death, not expiration of a fixed period independent of life. Starting date, original net investment and prior recovery must agree with the linked worksheet. A temporary child annuity must end in the death month and agree with the deduction records.",
            );
        }
        let amount = if valid { loss.amount(recovery) } else { 0 };
        line_cents(
            r,
            &alloc::format!("annuity_loss_{}", entry.id),
            amount,
            &["deductions.annuity_losses", "income.retirement.taxable"],
            "https://www.irs.gov/publications/p939",
            msg!("EXPLAIN_ANNUITY_LOSS_N"),
        );
        total += amount;
    }
    dollars(total)
}

pub(super) struct MortgageDeductionIn<'a> {
    pub(super) agi: i64,
    pub(super) foreign_magi: i64,
    pub(super) office_schedule_a: &'a crate::home_office::ScheduleAShares,
    pub(super) rental_personal: &'a [crate::kernel::ratio::exact::Exact; 2],
}

pub(super) struct MortgageDeductionOut {
    pub(super) mortgage_credit_current: i64,
    pub(super) mortgage_8a: i64,
    pub(super) mortgage_8b: i64,
    pub(super) mortgage_8c: i64,
    pub(super) mortgage_8d: i64,
    pub(super) amt_mortgage: i64,
    pub(super) mortgage_deductible_interest: i64,
}

pub(super) fn mortgage_deduction_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    i: MortgageDeductionIn<'_>,
) -> core::ops::ControlFlow<(), MortgageDeductionOut> {
    let &Pass { d, c, ded, context, separate, selected_regular, office_context, .. } = pass;
    let MortgageDeductionIn { agi, foreign_magi, office_schedule_a, rental_personal } = i;
    let mortgage_credit_current = mortgage_credit_line3(c, r);
    // Pub 936 Table 1 limits each line's interest; the Form 8396 line 3 credit
    // then reduces the deduction, taken from line 8a before line 8b.
    let mortgage_limited =
        crate::mortgage_limit::calculate(&ded.mortgage_loans().unwrap_or_default(), separate, r);
    for (index, loan) in ded.mortgage_loans().unwrap_or_default().iter().enumerate() {
        if let crate::MortgageInterestRefund::Yes {
            origin: Some(crate::MortgageRefundOrigin::Deducted(worksheet)),
            ..
        } = &loan.refund_of_interest
            && worksheet.ordinary
            && crate::other_income::worksheets(d)
                .iter()
                .any(|found| found.year == worksheet.year && found.facts_differ)
        {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::Mortgage)
                    .item("mortgage.loans", index),
                "Form 1098 box 4",
                "ISSUE_RECOVERY_REFUND_YEAR_FACTS_DIFFER",
                "Recoveries of the same year's deductions must use the same itemized deductions, standard deduction and taxable income from that year's return.",
            );
        }
    }
    let rental_mortgage = crate::extended_rules::home_mortgage_shares(context.forms(d));
    let loan_allowances =
        crate::mortgage_limit::loan_allowances(&ded.mortgage_loans().unwrap_or_default(), separate);
    // Remove the rental share in cents before rounding each personal Schedule A line.
    let mut personal_interest = [0; 2];
    let mut personal_points = [0; 2];
    for l in &loan_allowances {
        personal_interest[usize::from(!l.reported_on_1098)] += l.regular_interest_cents;
        personal_points[usize::from(!l.points_reported_on_1098)] += l.regular_points_cents;
    }
    // Keep fractional cents until each personal form line is complete.
    let personal_interest_cents: [_; 2] = core::array::from_fn(|index| {
        phase::floor(
            exact::whole(personal_interest[index] - rental_mortgage[index])
                - &office_schedule_a.interest_cents[index],
            exact::whole(0),
        )
    });
    let personal_points_cents: [_; 2] = core::array::from_fn(|index| {
        phase::floor(
            exact::whole(personal_points[index] - rental_mortgage[index + 2])
                - &office_schedule_a.points_cents[index],
            exact::whole(0),
        )
    });
    let mortgage_8a_limited =
        if rental_mortgage[0] == 0 && office_schedule_a.interest_cents[0] == exact::whole(0) {
            mortgage_limited.interest[0]
        } else {
            match exact::dollars(&personal_interest_cents[0])
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
            {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        };
    let mortgage_8b_limited =
        if rental_mortgage[1] == 0 && office_schedule_a.interest_cents[1] == exact::whole(0) {
            mortgage_limited.interest[1]
        } else {
            match exact::dollars(&personal_interest_cents[1])
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
            {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        };
    let personal_points = [
        if rental_mortgage[2] == 0 && office_schedule_a.points_cents[0] == exact::whole(0) {
            mortgage_limited.points[0]
        } else {
            match exact::dollars(&personal_points_cents[0])
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
            {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        },
        if rental_mortgage[3] == 0 && office_schedule_a.points_cents[1] == exact::whole(0) {
            mortgage_limited.points[1]
        } else {
            match exact::dollars(&personal_points_cents[1])
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
            {
                Ok(value) => value,
                Err(issue) => {
                    crate::section179_return::Issue::from(issue).report(r);
                    r.state = ResultState::Unsupported;
                    return core::ops::ControlFlow::Break(());
                }
            }
        },
    ];
    match crate::home_office::schedule_a_checks_with_section179(
        d,
        r,
        crate::home_office::ScheduleAFacts {
            magi: foreign_magi,
            limited_interest: mortgage_8a_limited + mortgage_8b_limited,
            paid_interest: dollars(
                ded.mortgage_loans()
                    .unwrap_or_default()
                    .iter()
                    .map(crate::MortgageLoan::interest_after_refund)
                    .sum(),
            ),
            loan_interest_cents: ded
                .mortgage_loans()
                .unwrap_or_default()
                .iter()
                .map(|l| l.interest)
                .sum(),
            property_taxes_cents: ded.property_taxes(),
            mortgage_credit: mortgage_credit_current,
        },
        selected_regular,
        Some(office_context),
    ) {
        Ok(value) => value,
        Err(issue) => {
            crate::section179_return::Issue::from(issue).report(r);
            r.state = ResultState::Unsupported;
            return core::ops::ControlFlow::Break(());
        }
    };
    // Points a Form 1098 reports (box 6) join the interest on line 8a; the Form 8396
    // credit reduces only the box 1 interest. A linked rental bears the rental
    // share of the credit on its certified loan (section 163(g)).
    let mortgage_8a = if rental_mortgage[..4].iter().any(|v| *v != 0)
        || office_schedule_a
            .interest_cents
            .iter()
            .chain(&office_schedule_a.points_cents)
            .any(|value| *value != exact::whole(0))
    {
        let rental_credit = crate::extended_rules::rental_credit_share(
            context.forms(d),
            mortgage_credit_current * 100,
        );
        let amount = phase::floor(
            &personal_interest_cents[0]
                - exact::whole(mortgage_credit_current * 100 - rental_credit),
            exact::whole(0),
        ) + &personal_points_cents[0];
        match exact::dollars(&amount).ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
        {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        }
    } else {
        positive(mortgage_8a_limited - mortgage_credit_current) + personal_points[0]
    };
    let mortgage_8b =
        positive(mortgage_8b_limited - positive(mortgage_credit_current - mortgage_8a_limited));
    // Points: deducted in full in the year paid when confirmed, else amortized over the
    // loan term, times Table 1 line 14 (mortgage_limit.rs). Line 8c holds the points no
    // Form 1098 reports; line 8d the box 5 insurance premiums after the AGI phase-out.
    let mortgage_8c = personal_points[1];
    let source_personal_insurance =
        loan_allowances.iter().map(|l| l.regular_insurance_cents).sum::<i64>() - rental_mortgage[4];
    let personal_insurance = phase::floor(
        exact::whole(source_personal_insurance) - &office_schedule_a.insurance_premiums_cents,
        exact::whole(0),
    );
    let mortgage_8d = if office_schedule_a.insurance_premiums_cents == exact::whole(0) {
        crate::mortgage_limit::insurance_phaseout(
            positive(source_personal_insurance),
            agi,
            separate,
        )
    } else {
        match exact::dollars(&crate::mortgage_limit::insurance_phaseout_exact(
            &personal_insurance,
            agi,
            separate,
        ))
        .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
        {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        }
    };
    let vehicle_insurance = loan_allowances
        .iter()
        .map(|l| l.regular_insurance_cents - l.amt_insurance_cents)
        .sum::<i64>();
    let rental_vehicle_insurance =
        crate::extended_rules::home_vehicle_insurance_share(context.forms(d));
    let amt_mortgage = if rental_mortgage[5] == 0 {
        mortgage_limited.amt_dwelling_interest
    } else {
        dollars(positive(
            loan_allowances
                .iter()
                .map(|l| {
                    l.regular_interest_cents - l.amt_interest_cents + l.regular_points_cents
                        - l.amt_points_cents
                })
                .sum::<i64>()
                - rental_mortgage[5],
        ))
    } + if rental_vehicle_insurance == 0 {
        crate::mortgage_limit::vehicle_insurance_premiums(
            &ded.mortgage_loans().unwrap_or_default(),
            agi,
            separate,
        )
    } else {
        crate::mortgage_limit::insurance_phaseout(
            positive(vehicle_insurance - rental_vehicle_insurance),
            agi,
            separate,
        )
    };
    let mortgage_deductible_interest = mortgage_8a + mortgage_8b + mortgage_8c + mortgage_8d;
    if !ded.mortgage_loans().unwrap_or_default().is_empty() || rental_personal[0] != exact::whole(0)
    {
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleA, "8a"),
            "mortgage_interest_8a",
            mortgage_8a,
            &["deductions.mortgage"],
            "https://www.irs.gov/instructions/i1040sca",
            msg!("EXPLAIN_MORTGAGE_INTEREST_8A"),
        );
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleA, "8b"),
            "mortgage_interest_8b",
            mortgage_8b,
            &["deductions.mortgage"],
            "https://www.irs.gov/instructions/i1040sca",
            msg!("EXPLAIN_MORTGAGE_INTEREST_8B"),
        );
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleA, "8c"),
            "mortgage_points_8c",
            mortgage_8c,
            &["deductions.mortgage"],
            "https://www.irs.gov/instructions/i1040sca",
            msg!("EXPLAIN_MORTGAGE_POINTS_8C"),
        );
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleA, "8d"),
            "mortgage_insurance_premiums_8d",
            mortgage_8d,
            &["deductions.mortgage", "agi"],
            "https://www.irs.gov/pub/irs-dft/p936--dft.pdf",
            msg!("EXPLAIN_MORTGAGE_INSURANCE_PREMIUMS_8D"),
        );
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleA, "8e"),
            "mortgage_interest_8e",
            mortgage_deductible_interest,
            &["deductions.mortgage"],
            "https://www.irs.gov/instructions/i1040sca",
            msg!("EXPLAIN_MORTGAGE_INTEREST_8E"),
        );
    }
    line(
        r,
        "mortgage_credit_current_potential",
        mortgage_credit_current,
        &["credits.mortgage_credit.current"],
        "https://www.irs.gov/pub/irs-dft/f8396--dft.pdf",
        msg!("EXPLAIN_MORTGAGE_CREDIT_CURRENT_POTENTIAL"),
    );
    core::ops::ControlFlow::Continue(MortgageDeductionOut {
        mortgage_credit_current,
        mortgage_8a,
        mortgage_8b,
        mortgage_8c,
        mortgage_8d,
        amt_mortgage,
        mortgage_deductible_interest,
    })
}

pub(super) struct ScheduleADetailReportingIn<'a> {
    pub(super) use_itemized: bool,
    pub(super) office_schedule_a: &'a crate::home_office::ScheduleAShares,
    pub(super) charity: &'a crate::charity::Worksheet,
    pub(super) salt_eligible: &'a crate::kernel::ratio::exact::Exact,
    pub(super) portfolio_interest: [i64; 2],
    pub(super) investment_property_taxes: i64,
    pub(super) estate_state_income_tax: i64,
    pub(super) estate_state_sales_tax: i64,
    pub(super) charity_deduction: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) gambling_loss: i64,
    pub(super) educator_itemized: i64,
    pub(super) annuity_loss: i64,
    pub(super) bond_premium: i64,
    pub(super) debt_instrument_loss: i64,
    pub(super) impairment_work: i64,
    pub(super) medical_before_limit: i64,
    pub(super) agi: i64,
    pub(super) medical_deduction: i64,
    pub(super) salt: i64,
    pub(super) gst_tax: i64,
    pub(super) foreign_tax_deductible: i64,
    pub(super) mortgage_deductible_interest: i64,
    pub(super) standard: i64,
    pub(super) itemized: i64,
    pub(super) itemized_reduction: i64,
    pub(super) mortgage_8a: i64,
    pub(super) mortgage_8b: i64,
    pub(super) mortgage_8c: i64,
    pub(super) mortgage_8d: i64,
    pub(super) ordinary_casualty_deduction: i64,
    pub(super) amt_mortgage: i64,
    pub(super) amt_related_adjustment: i64,
    pub(super) amt_medical_deduction: i64,
}

pub(super) fn schedule_a_detail_reporting_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    input: ScheduleADetailReportingIn<'_>,
) -> core::ops::ControlFlow<(), i64> {
    let &Pass { ded, repayment, .. } = pass;
    let ScheduleADetailReportingIn {
        use_itemized,
        office_schedule_a,
        charity,
        salt_eligible,
        portfolio_interest,
        investment_property_taxes,
        estate_state_income_tax,
        estate_state_sales_tax,
        charity_deduction,
        qualified_net_casualty,
        gambling_loss,
        educator_itemized,
        annuity_loss,
        bond_premium,
        debt_instrument_loss,
        impairment_work,
        medical_before_limit,
        agi,
        medical_deduction,
        salt,
        gst_tax,
        foreign_tax_deductible,
        mortgage_deductible_interest,
        standard,
        itemized,
        itemized_reduction,
        mortgage_8a,
        mortgage_8b,
        mortgage_8c,
        mortgage_8d,
        ordinary_casualty_deduction,
        amt_mortgage,
        amt_related_adjustment,
        amt_medical_deduction,
    } = input;
    // Schedule A line detail remains before §68; line 18 is the allowed total after it.
    let claim = |amount| if use_itemized { amount } else { 0 };
    let real_estate = match exact::dollars(
        &(exact::whole(ded.property_taxes())
            + exact::whole(office_schedule_a.whole_home_taxes_cents)
            - &office_schedule_a.taxes
            + exact::whole(t.estate_final_real_estate_tax)),
    )
    .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
    {
        Ok(value) => value,
        Err(issue) => {
            crate::section179_return::Issue::from(issue).report(r);
            r.state = ResultState::Unsupported;
            return core::ops::ControlFlow::Break(());
        }
    } + investment_property_taxes;
    let personal_property =
        dollars(ded.personal_property_taxes() + t.estate_final_personal_property_tax);
    let income_sales = dollars(
        ded.state_local_taxes() - ded.property_taxes() - ded.personal_property_taxes()
            + estate_state_income_tax
            + estate_state_sales_tax,
    );
    let current_charity = charity.current_cash() + charity.current_noncash();
    let carry_charity = positive(charity_deduction - current_charity);
    let other_itemized_total = qualified_net_casualty
        + gambling_loss
        + educator_itemized
        + repayment.deduction
        + annuity_loss
        + dollars(t.inherited_estate_tax + t.estate_interest_ird_tax)
        + bond_premium
        + debt_instrument_loss
        + impairment_work;
    emit_lines(
        r,
        &mut (),
        [
            (
                "1",
                "medical_expenses",
                match exact::dollars(
                    &(exact::whole(claim(medical_before_limit)) * exact::whole(100)),
                )
                .filter(|amount| amount.checked_mul(100).is_some())
                {
                    Some(amount) => amount,
                    None => {
                        crate::issue_localized!(
                            r,
                            &crate::IssueContext::Section(crate::TaxSection::Medical),
                            "Medical expenses",
                            "SCHEDULE_A_MEDICAL_SUBTOTAL_UNAVAILABLE",
                            "The required Schedule A medical expense subtotal cannot be represented in its native amount. No final tax can be reported.",
                        );
                        withhold_required_source_result(r);
                        return core::ops::ControlFlow::Break(());
                    }
                },
            ),
            ("2", "medical_agi", claim(agi)),
            ("3", "medical_floor", claim(fraction(positive(agi), 75, 1000))),
            ("4", "medical_deduction", claim(medical_deduction)),
            ("5a", "salt_income_or_sales_tax", claim(income_sales)),
            ("5b", "salt_real_estate_tax", claim(real_estate)),
            ("5c", "salt_personal_property_tax", claim(personal_property)),
            (
                "5d",
                "salt_before_limit",
                if use_itemized {
                    match exact::dollars(
                        &((salt_eligible + exact::whole(investment_property_taxes))
                            * exact::whole(100)),
                    )
                    .filter(|amount| amount.checked_mul(100).is_some())
                    {
                        Some(amount) => amount,
                        None => {
                            crate::issue_localized!(
                                r,
                                &crate::IssueContext::Section(crate::TaxSection::Taxes),
                                "State and local taxes",
                                "SALT_AMOUNT_UNAVAILABLE",
                                "The required Schedule A state and local tax subtotal cannot be represented in its native amount. No final tax can be reported.",
                            );
                            withhold_required_source_result(r);
                            return core::ops::ControlFlow::Break(());
                        }
                    }
                } else {
                    0
                },
            ),
            ("5e", "salt_deduction", claim(salt + investment_property_taxes)),
            ("6", "other_taxes_deduction", claim(gst_tax + foreign_tax_deductible)),
            (
                "7",
                "taxes_deduction",
                claim(salt + investment_property_taxes + gst_tax + foreign_tax_deductible),
            ),
            (
                "10",
                "interest_deduction",
                claim(mortgage_deductible_interest + portfolio_interest[0]),
            ),
            ("11", "charity_cash_gifts", claim(charity.current_cash())),
            ("12", "charity_noncash_gifts", claim(charity.current_noncash())),
            ("13", "charity_current_allowed", claim(charity_deduction - carry_charity)),
            ("14", "charity_carryover_allowed", claim(carry_charity)),
            ("15", "charity_deduction", claim(charity_deduction)),
            (
                "17c",
                "standard_with_disaster_loss",
                if !use_itemized && qualified_net_casualty > 0 { standard } else { 0 },
            ),
            ("17f", "bond_premium_deduction", claim(bond_premium)),
            ("17g", "debt_instrument_loss_deduction", claim(debt_instrument_loss)),
            ("17j", "impairment_work_deduction", claim(impairment_work)),
            (
                "17z",
                "other_itemized_total",
                if use_itemized {
                    other_itemized_total
                } else if qualified_net_casualty > 0 {
                    standard + qualified_net_casualty
                } else {
                    0
                },
            ),
            (
                "18",
                "itemized_deductions_allowed",
                match exact::dollars(
                    &((if use_itemized {
                        exact::whole(itemized) - exact::whole(itemized_reduction)
                    } else if qualified_net_casualty > 0 {
                        exact::whole(standard) + exact::whole(qualified_net_casualty)
                    } else {
                        exact::whole(0)
                    }) * exact::whole(100)),
                )
                .filter(|amount| amount.checked_mul(100).is_some())
                {
                    Some(amount) => amount,
                    None => {
                        crate::issue_localized!(
                            r,
                            &crate::IssueContext::Section(crate::TaxSection::Deductions),
                            "Itemized deductions",
                            "SCHEDULE_A_ITEMIZED_TOTAL_UNAVAILABLE",
                            "The required Schedule A itemized deduction total cannot be represented in its native amount. No final tax can be reported.",
                        );
                        withhold_required_source_result(r);
                        return core::ops::ControlFlow::Break(());
                    }
                },
            ),
        ],
        |(number, id, amount), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::ScheduleA, number)),
                id,
                LineValue::MoneyDollars(amount),
                &["deductions"],
                "https://www.irs.gov/pub/irs-dft/f1040sa--dft.pdf",
                crate::Message::new(
                    alloc::format!("EXPLAIN_{}", id.to_ascii_uppercase()),
                    alloc::vec![],
                ),
            ))
        },
    );
    for (number, id, amount) in [
        ("8a", "mortgage_interest_8a", claim(mortgage_8a)),
        ("8b", "mortgage_interest_8b", claim(mortgage_8b)),
        ("8c", "mortgage_points_8c", claim(mortgage_8c)),
        ("8d", "mortgage_insurance_premiums_8d", claim(mortgage_8d)),
        ("8e", "mortgage_interest_8e", claim(mortgage_deductible_interest)),
        ("9", "investment_interest_deduction", claim(portfolio_interest[0])),
        ("16", "personal_casualty_deduction", claim(ordinary_casualty_deduction)),
        ("17b", "qualified_net_disaster_loss", qualified_net_casualty),
        (
            "17e",
            "inherited_estate_tax_deduction",
            claim(dollars(t.inherited_estate_tax + t.estate_interest_ird_tax)),
        ),
        ("17k", "educator_itemized_deduction", claim(educator_itemized)),
    ] {
        if let Some(row) = r.lines.iter_mut().find(|row| row.id == id) {
            row.form_line = Some(FormLine::new(TaxForm::ScheduleA, number));
            row.amount = amount * 100;
        } else {
            form_line(
                r,
                FormLine::new(TaxForm::ScheduleA, number),
                id,
                amount,
                &["deductions"],
                "https://www.irs.gov/pub/irs-dft/f1040sa--dft.pdf",
                crate::Message::new(
                    alloc::format!("EXPLAIN_{}", id.to_ascii_uppercase()),
                    alloc::vec![],
                ),
            );
        }
    }
    line(
        r,
        "amt_mortgage_dwelling_interest",
        claim(amt_mortgage),
        &["deductions.mortgage"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_MORTGAGE_DWELLING_INTEREST"),
    );
    form_line(
        r,
        FormLine::new(TaxForm::Form6251, "3"),
        "amt_related_adjustments",
        claim(amt_mortgage)
            + amt_related_adjustment
            + claim(medical_deduction - amt_medical_deduction),
        &["deductions.mortgage", "income.business.assets", "adjustments"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_RELATED_ADJUSTMENTS"),
    );
    core::ops::ControlFlow::Continue(income_sales)
}

pub(super) struct ScheduleASourcesIn<'a> {
    pub(super) agi: i64,
    pub(super) foreign_magi: i64,
    pub(super) office_current: &'a crate::home_office::HomeContext,
}

pub(super) struct ScheduleASourcesOut {
    pub(super) estate_state_income_tax: i64,
    pub(super) estate_state_sales_tax: i64,
    pub(super) salt: i64,
    pub(super) gst_tax: i64,
    pub(super) bond_premium: i64,
    pub(super) debt_instrument_loss: i64,
    pub(super) impairment_work: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) office_schedule_a: crate::home_office::ScheduleAShares,
    pub(super) rental_personal: [crate::kernel::ratio::exact::Exact; 2],
    pub(super) schedule_a_state_local: crate::kernel::ratio::exact::Exact,
    pub(super) salt_eligible: crate::kernel::ratio::exact::Exact,
}

pub(super) fn schedule_a_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    office_next: &mut Option<crate::home_office::HomeContext>,
    i: ScheduleASourcesIn<'_>,
) -> core::ops::ControlFlow<(), ScheduleASourcesOut> {
    let &Pass { d, ded, s, j, context, selected_regular, office_context, .. } = pass;
    let ScheduleASourcesIn { agi, foreign_magi, office_current } = i;
    // Mandatory state disability and family-leave contributions printed in W-2 box 14 are state
    // income tax (Pub. 17, Taxes You May Deduct); they join the final-year K-1 amount on line 5a.
    let w2_state_disability: i64 = d
        .income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::Wage { data: crate::WageSource::W2(source) } => {
                Some(source.state_disability_cents())
            }
            _ => None,
        })
        .sum();
    let estate_state_income_tax = if ded.state_income_tax_elected() {
        t.estate_final_state_income_tax + w2_state_disability
    } else {
        0
    };
    let estate_state_sales_tax =
        if ded.state_sales_tax_elected() { t.estate_final_state_sales_tax } else { 0 };
    let estate_salt_cents = match crate::kernel::ratio::checked_sum([
        t.estate_final_personal_property_tax,
        t.estate_final_real_estate_tax,
        estate_state_income_tax,
        estate_state_sales_tax,
    ])
    .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
    {
        Ok(value) => value,
        Err(issue) => {
            crate::section179_return::Issue::from(issue).report(r);
            r.state = ResultState::Unsupported;
            return core::ops::ControlFlow::Break(());
        }
    };
    let salt_cap = salt_cap_current_year(s.filing_status(), foreign_magi);
    if !d.home_sites.is_empty() {
        let office = match crate::home_office::next_context_with_claims(
            context.forms(d),
            agi,
            foreign_magi,
            salt_cap * 100,
            estate_salt_cents,
            selected_regular,
            office_context,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        };
        if !office_current.stable_with(&office) {
            *office_next = Some(office);
        }
    }
    // Form 8829 on the itemizing route: the office's share of the home's
    // real estate taxes and mortgage interest leaves Schedule A.
    let office_schedule_a = match crate::home_office::schedule_a_shares(d) {
        Ok(value) => value,
        Err(issue) => {
            crate::section179_return::Issue::from(issue).report(r);
            r.state = ResultState::Unsupported;
            return core::ops::ControlFlow::Break(());
        }
    };
    let rental_personal = crate::extended_rules::home_personal_shares(d);
    let schedule_a_state_local = exact::whole(ded.state_local_taxes())
        + exact::whole(office_schedule_a.whole_home_taxes_cents)
        + &rental_personal[1]
        - &office_schedule_a.taxes;
    let salt_eligible = exact::rounded_exact(
        &((&schedule_a_state_local + exact::whole(estate_salt_cents)) / exact::whole(100)),
    );
    let salt = exact::rounded(&phase::cap(salt_eligible.clone(), [exact::whole(salt_cap)]))
        .expect("SALT deduction is bounded by its statutory native cap");
    let other_line = |kind: usize| {
        dollars(
            ded.other_itemized_lines()
                .iter()
                .filter(|row| {
                    matches!(
                        (kind, row),
                        (0, crate::OtherItemizedLine::GenerationSkippingTax { .. })
                            | (1, crate::OtherItemizedLine::BondPremium { .. })
                            | (2, crate::OtherItemizedLine::DebtInstrumentLoss { .. })
                            | (3, crate::OtherItemizedLine::ImpairmentWork { .. })
                    )
                })
                .map(crate::OtherItemizedLine::amount)
                .sum(),
        )
    };
    let gst_tax = other_line(0);
    let bond_premium = dollars(
        ded.other_itemized_lines()
            .iter()
            .filter(|row| matches!(row, crate::OtherItemizedLine::BondPremium { .. }))
            .map(crate::OtherItemizedLine::amount)
            .sum::<i64>()
            + crate::investment_statements::computed_bond_premium(d),
    );
    if let Some(key) =
        crate::investment_statements::manual_bond_premium_rule(d, ded.other_itemized_lines())
    {
        crate::require_localized!(
            r,
            false,
            &crate::IssueContext::Section(crate::TaxSection::OtherItemized),
            "Bond premium",
            key,
            "Check the bond-premium source identity and remove a duplicate computed deduction.",
        );
    }
    let debt_instrument_loss = other_line(2);
    let impairment_work = other_line(3);
    crate::require_localized!(
        r,
        j || !ded.other_itemized_lines().iter().any(|row| {
            matches!(
                row,
                crate::OtherItemizedLine::ImpairmentWork { owner: crate::Owner::Spouse, .. }
            )
        }),
        &crate::IssueContext::Section(crate::TaxSection::OtherItemized),
        "Other itemized deductions",
        "MESSAGE_YOUR_SPOUSE_S_EXPENSES_BELONG_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
        "Your spouse's expenses belong on this return only when you file jointly.",
    );
    let investment_property_taxes = dollars(ded.investment_property_taxes());
    core::ops::ControlFlow::Continue(ScheduleASourcesOut {
        estate_state_income_tax,
        estate_state_sales_tax,
        salt,
        gst_tax,
        bond_premium,
        debt_instrument_loss,
        impairment_work,
        investment_property_taxes,
        office_schedule_a,
        rental_personal,
        schedule_a_state_local,
        salt_eligible,
    })
}

pub(super) struct DeductionElectionIn<'a> {
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) amt_health: &'a crate::health::HealthResult,
    pub(super) marketplace_medical: &'a crate::kernel::ratio::exact::Exact,
    pub(super) personal_casualties: &'a [crate::PersonalCasualty],
    pub(super) foreign_tax_deduction: &'a crate::extended_rules::ForeignTaxDeduction,
    pub(super) portfolio_interest: [i64; 2],
    pub(super) casualty_capital: [i64; 2],
    pub(super) agi: i64,
    pub(super) estate_medical: i64,
    pub(super) gst_tax: i64,
    pub(super) foreign_tax_deductible: i64,
    pub(super) bond_premium: i64,
    pub(super) debt_instrument_loss: i64,
    pub(super) impairment_work: i64,
    pub(super) educator_itemized: i64,
    pub(super) annuity_loss: i64,
    pub(super) gambling_loss: i64,
    pub(super) salt: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) mortgage_deductible_interest: i64,
    pub(super) charity_deduction: i64,
    pub(super) standard_cash_allowed: i64,
    pub(super) standard: i64,
    pub(super) estate_state_income_tax: i64,
    pub(super) estate_state_sales_tax: i64,
}

pub(super) struct DeductionElectionOut {
    pub(super) medical_before_limit: i64,
    pub(super) medical_deduction: i64,
    pub(super) amt_medical_deduction: i64,
    pub(super) casualty_net: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) ordinary_casualty_deduction: i64,
    pub(super) casualty_deduction: i64,
    pub(super) itemized: i64,
    pub(super) cash_standard: i64,
    pub(super) claimed_casualty: i64,
    pub(super) use_itemized: bool,
}

pub(super) fn deduction_election_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: DeductionElectionIn<'_>,
) -> core::ops::ControlFlow<(), DeductionElectionOut> {
    let &Pass { d, ded, basis, repayment, j, c, screen, allocated_entries, context, .. } = pass;
    let force_itemized = context.force_itemized;
    let DeductionElectionIn {
        health,
        amt_health,
        marketplace_medical,
        personal_casualties,
        foreign_tax_deduction,
        portfolio_interest,
        casualty_capital,
        agi,
        estate_medical,
        gst_tax,
        foreign_tax_deductible,
        bond_premium,
        debt_instrument_loss,
        impairment_work,
        educator_itemized,
        annuity_loss,
        gambling_loss,
        salt,
        investment_property_taxes,
        mortgage_deductible_interest,
        charity_deduction,
        standard_cash_allowed,
        standard,
        estate_state_income_tax,
        estate_state_sales_tax,
    } = i;
    let medical_before_limit = match crate::health::medical_total(
        &[
            ded.medical(),
            estate_medical,
            basis.public_safety.medical,
            health.medical,
            crate::social_documents::medical(d),
        ],
        marketplace_medical,
    ) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    let medical_deduction = positive(medical_before_limit - fraction(positive(agi), 75, 1000));
    // Section 162(l)(3): the premium owner supplies its separate unused
    // AMT medical amount. The 213 floor still uses regular AGI.
    let amt_medical_before_limit = match crate::health::medical_total(
        &[
            ded.medical(),
            estate_medical,
            basis.public_safety.medical,
            amt_health.medical,
            crate::social_documents::medical(d),
        ],
        marketplace_medical,
    ) {
        Ok(value) => value,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    let amt_medical_deduction =
        positive(amt_medical_before_limit - fraction(positive(agi), 75, 1000));
    // Section 165(h)(1),(6) reduces ordinary events by $100 and qualifying
    // major-disaster events by $500. Qualified net loss bypasses the 10%-AGI
    // reduction and survives a standard-deduction election under section 63(b)(8).
    // Line 11 compares each qualified event's losses with the gains of all the
    // events, so the gains come first.
    let casualty = crate::casualty::section_a(personal_casualties, agi);
    let casualty_gains = casualty.gains;
    let undeclared_casualty_after_events = casualty.undeclared_after_events;
    let qualified_casualty = casualty.qualified_after_events;
    let casualty_after_events = casualty.after_events;
    let casualty_net = casualty.net;
    let qualified_net_casualty = casualty.qualified_net;
    let ordinary_casualty_deduction = casualty.ordinary_deduction;
    let casualty_deduction = casualty.deduction;
    let itemized = gst_tax
        + foreign_tax_deductible
        + bond_premium
        + debt_instrument_loss
        + impairment_work
        + repayment.deduction
        + educator_itemized
        + dollars(t.inherited_estate_tax + t.estate_interest_ird_tax)
        + annuity_loss
        + gambling_loss
        + medical_deduction
        + salt
        + investment_property_taxes
        + portfolio_interest[0]
        + mortgage_deductible_interest
        + charity_deduction
        + casualty_deduction;
    let cash_standard = if d.spouse_itemizes() {
        0
    } else {
        // Section 170(p) excludes the floor and carryover provisions, not
        // the current cash contribution-base limit in 170(b)(1)(G)(i).
        phase::cap(
            standard_cash_allowed,
            [if j {
                crate::year_constants::CHARITY_NONITEMIZER_JOINT_DOLLARS
            } else {
                crate::year_constants::CHARITY_NONITEMIZER_SINGLE_DOLLARS
            }],
        )
    };
    // High-income itemized/QBI/AMT coordination is explicitly screened below before publishing a result.
    r.automatic_itemizes = itemized > standard + qualified_net_casualty + cash_standard;
    let use_itemized = force_itemized.unwrap_or(match d.deduction_choice() {
        DeductionChoice::Itemized => true,

        DeductionChoice::Automatic => itemized > standard + qualified_net_casualty + cash_standard,
    });
    if use_itemized {
        let other_use = ded.mortgage_loans().unwrap_or_default().iter().any(|loan| {
            let crate::MortgageLoanDetails::FirstLender(ledger) = &loan.details else {
                return false;
            };
            match &ledger.purpose {
                crate::MortgageUse::Home => false,
                crate::MortgageUse::Refinance(value) => value.other > 0,
                crate::MortgageUse::Partly(value) => value.other > 0,
                crate::MortgageUse::Other => true,
            }
        });
        for (number, id, checked) in [
            ("8", "itemized_mortgage_other_use_box", other_use),
            (
                "19",
                "itemized_below_standard_box",
                d.deduction_choice() == DeductionChoice::Itemized && itemized < standard,
            ),
        ] {
            crate::line_emission::form_value(
                r,
                FormLine::new(TaxForm::ScheduleA, number),
                id,
                LineValue::Text,
                &["deductions"],
                "https://www.irs.gov/pub/irs-dft/f1040sa--dft.pdf",
                if checked { msg!("PRINT_BOX_CHECKED") } else { msg!("PRINT_BOX_NOT_CHECKED") },
            );
        }
    }
    if !use_itemized && matches!(c.foreign_tax, ForeignTaxClaim::Deduction { .. }) {
        note_once_scoped(
            r,
            msg!("WARN_FOREIGN_TAX_DEDUCTION_UNUSED"),
            crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                .alternatives(&[crate::TaxSection::Deductions]),
        );
    }
    if !use_itemized
        && matches!(c.foreign_tax, ForeignTaxClaim::Ordinary { .. })
        && foreign_tax_deductible > 0
    {
        note_once_scoped(
            r,
            msg!("WARN_FTC_SANCTIONED_COUNTRY_TAX"),
            crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                .alternatives(&[crate::TaxSection::Deductions]),
        );
    }
    if foreign_tax_deduction.accrued_sanctioned {
        note_once_scoped(
            r,
            msg!("WARN_FOREIGN_ACCRUED_SANCTIONED_TAX_NOT_DEDUCTED"),
            crate::IssueContext::Section(crate::TaxSection::MoreCredits),
        );
    }
    if use_itemized {
        for id in &foreign_tax_deduction.missing_k1 {
            if let Some(entry) = allocated_entries.iter().find(|entry| entry.id == *id) {
                let context = crate::interview::income_issue_context(entry)
                    .alternatives(&[crate::TaxSection::MoreCredits]);
                note_once_scoped(
                    r,
                    Message::new(
                        "WARN_FOREIGN_K1_TAX_SOURCE_MISSING",
                        alloc::vec![MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
                    ),
                    context,
                );
            }
        }
    }
    let claimed_casualty = if use_itemized { casualty_deduction } else { qualified_net_casualty };
    if !ded.personal_casualties.is_empty() {
        for (index, event) in ded.personal_casualties.iter().enumerate() {
            match event.source_complete {
                true => {}
                false => note_once_scoped(
                    r,
                    Message::new(
                        "WARN_CASUALTY_LOSS_LEFT_OUT",
                        alloc::vec![MessageArg::Text(event.label.clone())],
                    ),
                    crate::IssueContext::Section(crate::TaxSection::Deductions)
                        .item("personal_casualties", index),
                ),
            }
        }
        crate::require_localized!(
            r,
            claimed_casualty == 0
                || (matches!(
                    c.foreign_tax,
                    crate::ForeignTaxClaim::No | crate::ForeignTaxClaim::Deduction { .. }
                ) && !screen.atnol.present()
                    && estate_final_nol(d).is_none()),
            &crate::IssueContext::Section(crate::TaxSection::Deductions),
            "Personal casualty coordination",
            "ISSUE_A_CLAIMED_CASUALTY_LOSS_WITH_FORM_1116_A_PRIOR_AMT_OPERATING_LOSS_O_AD243AFA",
            "A claimed casualty loss with Form 1116, a prior AMT operating loss or an inherited estate/trust NOL needs coordinated source and loss-absorption worksheets.",
        );
        line(
            r,
            "form_4684_personal_loss_after_events",
            casualty_after_events,
            &["deductions.personal_casualties"],
            "https://www.irs.gov/pub/irs-dft/f4684--dft.pdf",
            msg!("EXPLAIN_FORM_N_PERSONAL_LOSS_AFTER_EVENTS"),
        );
        if undeclared_casualty_after_events > 0 {
            line(
                r,
                "form_4684_undeclared_loss_after_events",
                undeclared_casualty_after_events,
                &["deductions.personal_casualties"],
                "https://www.irs.gov/instructions/i4684",
                msg!("EXPLAIN_FORM_N_UNDECLARED_LOSS_AFTER_EVENTS"),
            );
        }
        line(
            r,
            "form_4684_personal_gains",
            casualty_gains,
            &["deductions.personal_casualties"],
            "https://www.irs.gov/pub/irs-dft/f4684--dft.pdf",
            msg!("EXPLAIN_FORM_N_PERSONAL_GAINS"),
        );
        for (id, amount, explanation_key) in [
            ("form_4684_personal_short_net", casualty_capital[0], "EXPLAIN_CASUALTY_SHORT_NET"),
            ("form_4684_personal_long_net", casualty_capital[1], "EXPLAIN_CASUALTY_LONG_NET"),
        ] {
            if amount != 0 {
                line(
                    r,
                    id,
                    amount,
                    &["form_4684_personal_gains", "form_4684_personal_loss_after_events"],
                    "https://www.irs.gov/instructions/i4684",
                    Message::new(explanation_key, alloc::vec![]),
                );
            }
        }
        line(
            r,
            "form_4684_personal_net_after_gains",
            casualty_net,
            &["form_4684_personal_loss_after_events", "form_4684_personal_gains"],
            "https://www.irs.gov/pub/irs-dft/f4684--dft.pdf",
            msg!("EXPLAIN_FORM_N_PERSONAL_NET_AFTER_GAINS"),
        );
        line(
            r,
            "form_4684_personal_loss_after_agi",
            ordinary_casualty_deduction,
            &["form_4684_personal_net_after_gains", "agi"],
            "https://www.irs.gov/pub/irs-dft/f4684--dft.pdf",
            msg!("EXPLAIN_FORM_N_PERSONAL_LOSS_AFTER_AGI"),
        );
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleA, "16"),
            "personal_casualty_deduction",
            claimed_casualty,
            &[
                "form_4684_personal_loss_after_agi",
                "qualified_net_disaster_loss",
                "deductions.choice",
            ],
            "https://www.irs.gov/pub/irs-dft/f4684--dft.pdf",
            msg!("EXPLAIN_PERSONAL_CASUALTY_DEDUCTION"),
        );
        if qualified_casualty > 0 {
            line(
                r,
                "qualified_net_disaster_loss",
                qualified_net_casualty,
                &["deductions.personal_casualties"],
                "https://uscode.house.gov/view.xhtml?edition=prelim&num=0&req=granuleid%3AUSC-prelim-title26-section165",
                msg!("EXPLAIN_QUALIFIED_NET_DISASTER_LOSS"),
            );
        }
    }
    if use_itemized && t.estate_final_state_income_tax > 0 && estate_state_income_tax == 0 {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Final-year K-1 state income tax was not deducted because the return did not elect state/local income taxes on Schedule A line 5a. Select the income-tax branch to compare it with a sales-tax election.", crate::Message::new("NOTE_FINAL_YEAR_K_1_STATE_INCOME_TAX_WAS_NOT_DEDUCTED_BECAUSE_THE_RET_83F83D7C", alloc::vec![])));
    }
    if use_itemized && t.estate_final_state_sales_tax > 0 && estate_state_sales_tax == 0 {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Final-year K-1 general sales tax was not deducted because the return did not elect state/local general sales taxes on Schedule A line 5a. Select the sales-tax branch to compare it with an income-tax election.", crate::Message::new("NOTE_FINAL_YEAR_K_1_GENERAL_SALES_TAX_WAS_NOT_DEDUCTED_BECAUSE_THE_RE_60B8F2C2", alloc::vec![])));
    }
    core::ops::ControlFlow::Continue(DeductionElectionOut {
        medical_before_limit,
        medical_deduction,
        amt_medical_deduction,
        casualty_net,
        qualified_net_casualty,
        ordinary_casualty_deduction,
        casualty_deduction,
        itemized,
        cash_standard,
        claimed_casualty,
        use_itemized,
    })
}

pub(super) struct DeductionAssemblyIn {
    pub(super) use_itemized: bool,
    pub(super) itemized: i64,
    pub(super) standard: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) cash_standard: i64,
    pub(super) annuity_loss: i64,
    pub(super) gambling: i64,
    pub(super) gambling_loss: i64,
}

pub(super) struct DeductionAssemblyOut {
    pub(super) deduction: i64,
}

pub(super) fn deduction_assembly_phase(
    r: &mut FederalResult,
    t: &Totals,
    input: DeductionAssemblyIn,
) -> DeductionAssemblyOut {
    let DeductionAssemblyIn {
        use_itemized,
        itemized,
        standard,
        qualified_net_casualty,
        cash_standard,
        annuity_loss,
        gambling,
        gambling_loss,
    } = input;
    let deduction =
        if use_itemized { itemized } else { standard + qualified_net_casualty + cash_standard };
    form_line(
        r,
        FormLine::new(TaxForm::ScheduleA, "17i"),
        "unrecovered_annuity_deduction",
        if use_itemized { annuity_loss } else { 0 },
        &["deductions.annuity_losses", "income.retirement"],
        "https://www.irs.gov/publications/p939",
        msg!("EXPLAIN_UNRECOVERED_ANNUITY_DEDUCTION"),
    );
    form_line(
        r,
        FormLine::new(TaxForm::Schedule1, "8b"),
        "gambling_winnings",
        gambling,
        &["income.gambling"],
        "https://www.irs.gov/taxtopics/tc419",
        msg!("EXPLAIN_GAMBLING_WINNINGS"),
    );
    form_line(
        r,
        FormLine::new(TaxForm::ScheduleA, "17a"),
        "gambling_loss_deduction",
        if use_itemized { gambling_loss } else { 0 },
        &["income.gambling.losses", "deductions.choice"],
        "https://www.irs.gov/publications/p505",
        msg!("EXPLAIN_GAMBLING_LOSS_DEDUCTION"),
    );
    form_line(
        r,
        FormLine::new(TaxForm::ScheduleA, "17e"),
        "inherited_estate_tax_deduction",
        if use_itemized { dollars(t.inherited_estate_tax) } else { 0 },
        &["income.inherited_ira.estate_tax", "deductions.choice"],
        "https://www.irs.gov/publications/p559",
        msg!("EXPLAIN_INHERITED_ESTATE_TAX_DEDUCTION"),
    );
    line(
        r,
        "estate_trust_ird_estate_tax_deduction",
        if use_itemized { dollars(t.estate_interest_ird_tax) } else { 0 },
        &["income.estate_trust_portfolio.box_10_interest_ird_estate_tax", "deductions.choice"],
        "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
        msg!("EXPLAIN_ESTATE_TRUST_IRD_ESTATE_TAX_DEDUCTION"),
    );
    DeductionAssemblyOut { deduction }
}
