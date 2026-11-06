use crate::tests::*;

pub(in crate::tests) struct FixtureEdit<'a, S, N> {
    source: &'a mut S,
    before: N,
    value: Option<N>,
    save: fn(&mut S, &N, N),
}
impl<S, N> core::ops::Deref for FixtureEdit<'_, S, N> {
    type Target = N;
    fn deref(&self) -> &N {
        self.value.as_ref().unwrap()
    }
}
impl<S, N> core::ops::DerefMut for FixtureEdit<'_, S, N> {
    fn deref_mut(&mut self) -> &mut N {
        self.value.as_mut().unwrap()
    }
}
impl<S, N> Drop for FixtureEdit<'_, S, N> {
    fn drop(&mut self) {
        (self.save)(self.source, &self.before, self.value.take().unwrap());
    }
}
macro_rules! copy_changed {
    ($before:ident, $after:ident, $to:expr, $from:expr, $($field:ident),+ $(,)?) => {
        $(if $before.$field != $after.$field { $to.$field = $from.$field.clone(); })+
    };
}

pub(in crate::tests) fn edit_wage(
    source: &mut WageSource,
) -> FixtureEdit<'_, WageSource, FederalWage> {
    let before = FederalWage::from(&*source);
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source,
        save: |source, before, after| {
            let replacement = wage_source(after.clone());
            let (WageSource::W2(source), WageSource::W2(new)) = (&mut *source, replacement) else {
                *source = wage_source(after);
                return;
            };
            copy_changed!(
                before,
                after,
                source.form,
                new.form,
                wages,
                withholding,
                social_wages,
                social_tax,
                medicare_wages,
                medicare_tax
            );
            if before.social_tips != after.social_tips {
                source.form.social_tips = new.form.social_tips;
            }
            if before.care_benefits != after.care_benefits {
                source.form.care_benefits = new.form.care_benefits;
            }
            if before.statutory_employee != after.statutory_employee {
                source.form.statutory = new.form.statutory;
            }
            if before.retirement_plan != after.retirement_plan {
                source.form.retirement_plan = new.form.retirement_plan;
            }
            if before.payroll != after.payroll {
                source.answers.coverage = new.answers.coverage.clone();
            }
            copy_changed!(
                before,
                after,
                source.answers,
                new.answers,
                disability,
                abroad,
                deferred_pension,
                inmate_pay,
                medicaid_waiver,
                statutory_business,
                paying_scorporation,
                coverage
            );
            if before.employee_tips != after.employee_tips {
                source.answers.tip_records = new.answers.tip_records;
                source.answers.noncash_tips = new.answers.noncash_tips;
                source.form.allocated_tips = new.form.allocated_tips;
            }
            if before.work_deductions != after.work_deductions
                || before.uncollected_tax != after.uncollected_tax
                || before.other_box12 != after.other_box12
            {
                source.form.box12 = new.form.box12;
            }
            if before.work_deductions != after.work_deductions {
                source.answers.tips_listed_occupation = new.answers.tips_listed_occupation;
                source.answers.overtime_flsa_required = new.answers.overtime_flsa_required;
            }
            // A W-2 with tips carries its tip-occupation answer; a fixture that
            // states none claims no qualified-tips deduction.
            if before.employee_tips != after.employee_tips
                && source.answers.tips_listed_occupation.is_none()
                && (source.form.allocated_tips.unwrap_or(0) > 0
                    || !source.answers.tip_months().is_empty())
            {
                source.answers.tips_listed_occupation =
                    Some(crate::WageDeductionEligibility::Ineligible);
            }
        },
    }
}

pub(in crate::tests) fn edit_interest(
    source: &mut InterestSource,
) -> FixtureEdit<'_, InterestSource, FederalInterest> {
    let before = FederalInterest::from(&*source);
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source,
        save: |source, before, after| {
            let replacement = interest_source(after.clone());
            let (InterestSource::Int1099(source), InterestSource::Int1099(new)) =
                (&mut *source, replacement)
            else {
                *source = interest_source(after);
                return;
            };
            copy_changed!(
                before,
                after,
                source,
                new,
                nominee,
                adjustments,
                after_death,
                accrued_interest,
                special
            );
            macro_rules! paper_changed {
            ($($native:ident => $paper:ident),+ $(,)?) => {
                $(if before.$native != after.$native { source.form.$paper = new.form.$paper.clone(); })+
            };
        }
            paper_changed!(taxable => interest_income, investment_expenses => investment_expenses,
            market_discount => market_discount, foreign_country => foreign_country_or_us_territory,
            early_penalty => early_withdrawal_penalty, us_obligations => us_savings_bond_and_treasury_interest,
            withholding => federal_income_tax_withheld, foreign_tax => foreign_tax_paid,
            exempt => tax_exempt_interest, private_activity => specified_private_activity_bond_interest,
            bond_premium => bond_premium, treasury_bond_premium => bond_premium_on_treasury_obligations,
            tax_exempt_bond_premium => bond_premium_on_tax_exempt_bond);
            if before.printed_facts != after.printed_facts {
                source.form.corrected = new.form.corrected;
                source.form.payer_telephone = new.form.payer_telephone;
                source.form.payer_rtn = new.form.payer_rtn;
                source.form.fatca_filing_requirement = new.form.fatca_filing_requirement;
                source.form.account_number = new.form.account_number;
                source.form.tax_exempt_and_tax_credit_bond_cusip_no =
                    new.form.tax_exempt_and_tax_credit_bond_cusip_no;
            }
        },
    }
}

pub(in crate::tests) fn edit_dividend(
    source: &mut DividendSource,
) -> FixtureEdit<'_, DividendSource, FederalDividend> {
    let before = FederalDividend::from(&*source);
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source,
        save: |source, before, after| {
            let new = dividend_source(after.clone());
            copy_changed!(
                before,
                after,
                source.form,
                new.form,
                nondividend_distributions,
                cash_liquidation,
                noncash_liquidation,
                section_897_ordinary,
                section_897_capital,
                foreign_country,
                ordinary,
                qualified,
                capital_distributions,
                collectible_distributions,
                unrecaptured_distributions,
                section_1202_gain,
                withholding,
                exempt_dividends,
                private_activity_dividends,
                foreign_tax
            );
            copy_changed!(
                before,
                after,
                source,
                new,
                capital_returns,
                nominee,
                after_death,
                section_1202_exclusion,
                special
            );
            if before.section199a != after.section199a {
                source.form.section199a = new.form.section199a;
                source.reit_eligible = new.reit_eligible;
            }
        },
    }
}

pub(in crate::tests) fn edit_retirement(
    source: &mut RetirementSource,
) -> FixtureEdit<'_, RetirementSource, FederalRetirement> {
    let before = FederalRetirement::from(&*source);
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source,
        save: |source, before, mut after| {
            if before.kind != after.kind {
                if before.ira == after.ira {
                    after.ira = after.kind.ira();
                }
                if before.box7 == after.box7 {
                    if before.kind == RetirementKind::Nonqualified {
                        after.box7 &= !(DistributionCodes::CODE_D.0
                            | DistributionCodes::CODE_F.0
                            | DistributionCodes::CODE_W.0);
                        if after.box7 == 0 {
                            after.box7 = DistributionCodes::CODE_7.0;
                        }
                    }
                    if before.kind == RetirementKind::Inherited {
                        after.box7 &= !DistributionCodes::CODE_4.0;
                        after.box7 |= DistributionCodes::CODE_7.0;
                    }
                    if after.kind == RetirementKind::Inherited {
                        after.box7 &= !(DistributionCodes::CODE_1.0
                            | DistributionCodes::CODE_2.0
                            | DistributionCodes::CODE_3.0
                            | DistributionCodes::CODE_7.0);
                        after.box7 |= DistributionCodes::CODE_4.0;
                    }
                }
            }
            let replacement = retirement_source(after.clone());
            let (RetirementSource::R1099(source), RetirementSource::R1099(new)) =
                (&mut *source, replacement)
            else {
                *source = retirement_source(after);
                return;
            };
            copy_changed!(
                before,
                after,
                source.form,
                new.form,
                gross,
                withholding,
                box_3,
                box_6,
                box_8a,
                box_8b_percent,
                box_9a_percent,
                box_9b,
                box_10,
                ira,
                trump
            );
            if before.box_2a != after.box_2a {
                source.form.taxable = new.form.taxable;
            }
            if before.box7 != after.box7
                || before.kind != after.kind
                || before.ordinary_distribution != after.ordinary_distribution
            {
                source.form.box7 = new.form.box7;
                source.form.ira = new.form.ira;
                source.form.payment_date = new.form.payment_date;
            }
            if before.form_boxes != after.form_boxes {
                source.form.taxable_not_determined = new.form.taxable_not_determined;
                source.form.total_distribution = new.form.total_distribution;
                source.form.employee_contributions = new.form.employee_contributions;
                source.form.excess_contribution_earnings = new.form.excess_contribution_earnings;
                source.form.first_roth_year = new.form.first_roth_year;
                source.form.fatca = new.form.fatca;
                source.form.payment_date = new.form.payment_date;
            }
            let code_owns_taxable = |taxable: &Option<RetirementTaxable>| {
                matches!(
                    taxable,
                    Some(
                        RetirementTaxable::Inherited { .. }
                            | RetirementTaxable::PropertyProceeds { .. }
                    )
                )
            };
            if before.ordinary_distribution != after.ordinary_distribution
                || before.disability_wages != after.disability_wages
                || before.kind != after.kind
                || (before.taxable != after.taxable
                    && (code_owns_taxable(&before.taxable) || code_owns_taxable(&after.taxable)))
            {
                source.code = new.code;
            }
            if before.taxable != after.taxable {
                // Choosing the payer's amount in a native fixture replaces its worksheet.
                // An explicitly changed paper checkbox is still copied as supplied.
                source.form.taxable_not_determined =
                    if matches!(new.taxable_method.as_ref(), Some(TaxableMethod::BoxTwoA))
                        && before.form_boxes.taxable_not_determined
                            == after.form_boxes.taxable_not_determined
                    {
                        false
                    } else {
                        new.form.taxable_not_determined
                    };
                source.taxable_method = new.taxable_method;
                source.ira_basis = new.ira_basis;
            }
            copy_changed!(
                before,
                after,
                source,
                new,
                disability,
                box_10_basis,
                disaster,
                after_death
            );
        },
    }
}

pub(in crate::tests) fn edit_roth(
    source: &mut R1099Source,
) -> FixtureEdit<'_, R1099Source, RothWithdrawal> {
    let before = RothWithdrawal::from(&*source);
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source,
        save: |source, before, after| {
            let new = roth_source(after.clone());
            copy_changed!(
                before,
                after,
                source.form,
                new.form,
                gross,
                withholding,
                box_3,
                box_6,
                box_8a,
                box_8b_percent,
                box_9a_percent,
                box_9b,
                box_10,
                box7,
                ira,
                trump
            );
            if before.box_2a != after.box_2a {
                source.form.taxable = new.form.taxable;
            }
            if before.form_boxes != after.form_boxes {
                source.form.taxable_not_determined = new.form.taxable_not_determined;
                source.form.total_distribution = new.form.total_distribution;
                source.form.employee_contributions = new.form.employee_contributions;
                source.form.excess_contribution_earnings = new.form.excess_contribution_earnings;
                source.form.first_roth_year = new.form.first_roth_year;
                source.form.fatca = new.form.fatca;
                source.form.payment_date = new.form.payment_date;
            }
            if before.code != after.code
                || before.exception != after.exception
                || before.repaid != after.repaid
                || before.distributed != after.distributed
                || before.inherited != after.inherited
                || before.rolled_over != after.rolled_over
                || before.sequence != after.sequence
            {
                source.code = new.code;
            }
            copy_changed!(before, after, source, new, disaster);
        },
    }
}

pub(in crate::tests) fn edit_scorp(
    source: &mut ScorpSource,
) -> FixtureEdit<'_, ScorpSource, ScorpK1> {
    let before = ScorpK1::from(&*source);
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source,
        save: |source, before, after| {
            let old = scorp_source(before.clone());
            let new = scorp_source(after);
            let before = &old.paper;
            let after = &new.paper;
            copy_changed!(
                before,
                after,
                source.paper,
                new.paper,
                reporting_period,
                final_k1,
                amended_k1,
                corporation,
                irs_center,
                corporation_shares,
                shareholder,
                reporting_party,
                shareholder_type,
                allocation_percentage,
                shareholder_shares,
                loans,
                ordinary,
                rental_real_estate,
                other_rental,
                interest,
                dividends,
                qualified_dividends,
                royalties,
                short,
                long,
                collectibles,
                unrecaptured,
                section1231,
                section179,
                k3_attached,
                multiple_at_risk,
                multiple_passive
            );
            let before = &old.statements;
            let after = &new.statements;
            copy_changed!(
                before,
                after,
                source.statements,
                new.statements,
                qbi,
                rentals,
                property,
                income,
                credits,
                investment_property,
                section179_sales,
                private_activity,
                schedule_k3,
                situations
            );
            let before = &old.personal;
            let after = &new.personal;
            copy_changed!(
                before,
                after,
                source.personal,
                new.personal,
                participation,
                basis,
                amt,
                disposition,
                special,
                qbi_group,
                qbi_records,
                stock_term,
                trading,
                rental_participation,
                rental_carryover_active,
                rental_disposed,
                section179_carryover,
                section179_carryover_sold,
                amt_section179_income_carry,
                investment_confirmed,
                housing_confirmed,
                credit_origins,
                investment_forms,
                k3_withholding,
                research_credit_line,
                health_insurance,
                exact
            );
            patch_code_rows(&mut source.paper.income, &old.paper.income, &new.paper.income);
            patch_code_rows(
                &mut source.paper.deductions,
                &old.paper.deductions,
                &new.paper.deductions,
            );
            patch_code_rows(&mut source.paper.credits, &old.paper.credits, &new.paper.credits);
            patch_code_rows(&mut source.paper.amt, &old.paper.amt, &new.paper.amt);
            patch_code_rows(&mut source.paper.basis, &old.paper.basis, &new.paper.basis);
            patch_code_rows(
                &mut source.paper.information,
                &old.paper.information,
                &new.paper.information,
            );
        },
    }
}

fn patch_code_rows(target: &mut Vec<K1CodeRow>, before: &[K1CodeRow], after: &[K1CodeRow]) {
    let mut codes = Vec::new();
    for row in before.iter().chain(after) {
        if !codes.contains(&row.code) {
            codes.push(row.code.clone());
        }
    }
    for code in codes {
        let old: Vec<_> = before.iter().filter(|row| row.code == code).cloned().collect();
        let new: Vec<_> = after.iter().filter(|row| row.code == code).cloned().collect();
        if old != new {
            let at = target.iter().position(|row| row.code == code).unwrap_or(target.len());
            target.retain(|row| row.code != code);
            target.splice(at..at, new);
        }
    }
}

pub(in crate::tests) fn edit_estate(
    source: &mut EstateTrustSource,
) -> FixtureEdit<'_, EstateTrustSource, FederalEstateTrustPortfolioK1> {
    let before = FederalEstateTrustPortfolioK1::from(&*source);
    FixtureEdit { value: Some(before.clone()), before, source, save: patch_estate_source }
}

fn patch_estate_source(
    source: &mut EstateTrustSource,
    before: &FederalEstateTrustPortfolioK1,
    after: FederalEstateTrustPortfolioK1,
) {
    let old = estate_source(before.clone());
    let new = estate_source(after);
    let before = &old.form;
    let after = &new.form;
    copy_changed!(
        before,
        after,
        source.form,
        new.form,
        interest,
        ordinary_dividends,
        qualified_dividends,
        short_gain,
        long_gain,
        rate_28_gain,
        unrecaptured_gain,
        portfolio_income,
        business_income,
        rental_real_estate_income,
        other_rental_income,
        estate_tax_deduction,
        final_k1,
        final_form1041,
        filed_1041t,
        began,
        ended
    );
    if before.estate_trust.federal_id != after.estate_trust.federal_id {
        source.form.estate_trust.federal_id = new.form.estate_trust.federal_id.clone();
    }
    patch_estate_code_rows(&mut source.form.deductions, &old.form.deductions, &new.form.deductions);
    patch_estate_code_rows(
        &mut source.form.final_deductions,
        &old.form.final_deductions,
        &new.form.final_deductions,
    );
    patch_estate_code_rows(&mut source.form.amt, &old.form.amt, &new.form.amt);
    patch_estate_code_rows(&mut source.form.credits, &old.form.credits, &new.form.credits);
    patch_estate_code_rows(
        &mut source.form.information,
        &old.form.information,
        &new.form.information,
    );
    let before = &old.statements;
    let after = &new.statements;
    copy_changed!(
        before,
        after,
        source.statements,
        new.statements,
        final_character,
        amt_reconciliation,
        investment_income,
        reported_credits,
        foreign_statement,
        investment_property,
        section199a_dividends,
        code_i_dividends_only,
        box_5_royalty_only,
        box_9_royalty_sound_recording_only,
        box_9_royalty_cost_depletion_only,
        box_9_royalty_amortization_only,
        activities,
        qbi,
        box_10_interest_ird_only,
        final_67e_no_niit_allocation,
        final_personal_property_tax,
        final_real_estate_tax,
        final_state_income_tax,
        final_state_sales_tax,
        final_taxes_only,
        final_carry_same_tax_character,
        amt_box12a_standalone,
        amt_box12_components_confirmed,
        amt_box12e_included_in_d,
        amt_box12f_included_in_d,
        private_activity_interest,
        final_nol,
        niit_adjustment,
        unentered
    );
    let before = &old.answers;
    let after = &new.answers;
    copy_changed!(
        before,
        after,
        source.answers,
        new.answers,
        final_year_successor,
        final_mortgages,
        complete_portfolio_only,
        entity_period
    );
}

fn patch_estate_code_rows(
    target: &mut Vec<EstateTrustCodeRow>,
    before: &[EstateTrustCodeRow],
    after: &[EstateTrustCodeRow],
) {
    let mut codes = Vec::new();
    for row in before.iter().chain(after) {
        if !codes.contains(&row.code) {
            codes.push(row.code.clone());
        }
    }
    for code in codes {
        let old: Vec<_> = before.iter().filter(|row| row.code == code).cloned().collect();
        let new: Vec<_> = after.iter().filter(|row| row.code == code).cloned().collect();
        if old != new {
            let at = target.iter().position(|row| row.code == code).unwrap_or(target.len());
            target.retain(|row| row.code != code);
            target.splice(at..at, new);
        }
    }
}

pub(in crate::tests) fn edit_estate_document(
    document: &mut K1EstateTrustDocument,
) -> FixtureEdit<'_, K1EstateTrustDocument, K1EstateTrust> {
    let before = super::form_1041_k1::estate_fixture_page(document)
        .unwrap_or_else(|| estate_snapshot(&estate_fixture_source(document)));
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source: document,
        save: |document, before, after| {
            let native_after = after.clone();
            let mut source = estate_fixture_source(document);
            patch_estate_source(&mut source, &before.record(), after.record());
            copy_changed!(before, after, source.form, after, estate_trust, fiduciary, beneficiary);
            let old = K1EstateTrustDocument::from_source(&estate_source(before.record()));
            let new = K1EstateTrustDocument::from_source(&source);
            let before = &old.statements;
            let after = &new.statements;
            copy_changed!(
                before,
                after,
                document.statements,
                new.statements,
                final_character,
                amt_other_in_a,
                amt_adjustments,
                investment_income,
                reported_credits,
                foreign_statement,
                investment_property,
                activities,
                qbi,
                private_activity_interest,
                section199a_dividends,
                box_5_royalty_only,
                box_9_royalty_sound_recording_only,
                box_9_royalty_cost_depletion_only,
                box_9_royalty_amortization_only,
                box_10_interest_ird_only,
                final_67e_no_niit_allocation,
                final_personal_property_tax,
                final_real_estate_tax,
                final_state_income_tax,
                final_state_sales_tax,
                final_taxes_only,
                final_carry_same_tax_character,
                amt_box12a_standalone,
                amt_box12_components_confirmed,
                amt_box12e_included_in_d,
                amt_box12f_included_in_d,
                code_i_dividends_only,
                final_nol,
                niit_adjustment
            );
            document.form = source.form.clone();
            super::form_1041_k1::remember_estate_fixture(
                document.clone(),
                source,
                Some(native_after),
            );
        },
    }
}

pub(in crate::tests) fn edit_mortgage(
    source: &mut Mortgage1098Source,
) -> FixtureEdit<'_, Mortgage1098Source, MortgageLoan> {
    let before = mortgage_loan(&*source);
    FixtureEdit {
        value: Some(before.clone()),
        before,
        source,
        save: |source, before, after| {
            let new = mortgage_loan_source(after.clone());
            if before.interest != after.interest {
                source.form.box_1 = new.form.box_1;
            }
            if before.outstanding_principal != after.outstanding_principal {
                source.form.box_2 = new.form.box_2;
            }
            if before.incurred != after.incurred {
                source.form.box_3 = new.form.box_3;
                source.debt_date = new.debt_date;
            }
            if before.mortgage_insurance_premiums != after.mortgage_insurance_premiums {
                source.form.box_5 = new.form.box_5;
            }
            if before.refund_of_interest != after.refund_of_interest {
                source.form.box_4 = new.form.box_4;
                source.refund_origin = new.refund_origin;
            }
            if before.points != after.points {
                source.form.box_6 = new.form.box_6;
                source.points = new.points;
            }
            copy_changed!(before, after, source, new, id, property, lender, details);
            if before.printed != after.printed {
                source.form.lender = new.form.lender;
                source.form.borrower = new.form.borrower;
                source.assigned = new.assigned;
                source.owner_source = new.owner_source;
                source.joint_share = new.joint_share;
            }
        },
    }
}
