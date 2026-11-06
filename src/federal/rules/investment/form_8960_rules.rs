// form_8960 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Persisted selectors name the person by identity (the EnumSource key), never by position.
// Rebuild editor choices by identity and retain only referenced missing people. This keeps
// incomplete relationships repairable without growing a history of every deleted person.
// Form 8960 Parts I, II and III for an individual, from the amounts the
// return already uses for the tax. Line 4a is Schedule 1 lines 3, 5 and 6;
// line 4b reverses the nonpassive business share; line 5a is Form 1040 line 7
// plus Schedule 1 line 4; line 7 holds the Schedule 1 line 8 investment
// items and the fiduciary's box 14 code H. The investment-related itemized
// deductions after the aggregate cap are spread over lines 9a, 9b and 10 in
// proportion (Part IV of the lines 9 and 10 worksheet). Line 9c is zero:
// P.L. 119-21 section 70110 made the miscellaneous-deduction repeal permanent.
pub(super) struct Form8960Amounts {
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) annuities: i64,
    pub(super) schedule_1_business: i64,
    pub(super) business_adjustment: i64,
    pub(super) gains: i64,
    pub(super) excluded_gains: i64,
    pub(super) other: i64,
    pub(super) investment_interest: i64,
    pub(super) taxes: i64,
    pub(super) estate_tax: i64,
    pub(super) allowed: i64,
    pub(super) savings_penalty: i64,
    pub(super) excess_deduction: i64,
    pub(super) modified_agi: i64,
    pub(super) threshold: i64,
    pub(super) nii: i64,
    pub(super) tax: i64,
}

pub(super) fn form_8960_lines(r: &mut FederalResult, a: Form8960Amounts) {
    let line_4c = a.schedule_1_business + a.business_adjustment;
    let line_5d = a.gains + a.excluded_gains;
    let line_8 = a.interest + a.dividends + a.annuities + line_4c + line_5d + a.other;
    let before = a.investment_interest + a.taxes + a.estate_tax;
    let share = |amount: i64| {
        if before == 0 { 0 } else { fraction(amount, a.allowed, before) }
    };
    let line_9a = share(a.investment_interest);
    let estate_tax = share(a.estate_tax);
    let line_9b = a.allowed - line_9a - estate_tax;
    let line_9d = line_9a + line_9b;
    // Reg. 1.1411-4(g)(4)(iii): a beneficiary's qualifying final excess
    // deduction is a separate line 10 deduction, not an itemized deduction
    // subject to the lines 9/10 itemized cap or a box 14 H line 7 adjustment.
    let line_10 = a.savings_penalty + estate_tax + a.excess_deduction;
    let line_11 = line_9d + line_10;
    let line_12 = positive(line_8 - line_11);
    let line_15 = positive(a.modified_agi - a.threshold);
    debug_assert_eq!(line_12, a.nii);
    if line_15 == 0 {
        // Who Must File (i8960): attach Form 8960 whenever modified AGI is greater than the
        // threshold, even with no investment income and no tax.
        return;
    }
    emit_lines(
        r,
        &mut (),
        [
            ("1", "form_8960_taxable_interest", a.interest, msg!("EXPLAIN_N_14")),
            ("2", "form_8960_ordinary_dividends", a.dividends, msg!("EXPLAIN_N_15")),
            ("3", "form_8960_annuities", a.annuities, msg!("EXPLAIN_N_16")),
            ("4a", "form_8960_business_rental_income", a.schedule_1_business, msg!("EXPLAIN_4A")),
            (
                "4b",
                "form_8960_nonpassive_business_adjustment",
                a.business_adjustment,
                msg!("EXPLAIN_4B"),
            ),
            ("4c", "form_8960_business_investment_income", line_4c, msg!("EXPLAIN_4C")),
            ("5a", "form_8960_property_gains", a.gains, msg!("EXPLAIN_5A")),
            ("5b", "form_8960_excluded_gains", a.excluded_gains, msg!("EXPLAIN_5B")),
            ("5d", "form_8960_net_gains", line_5d, msg!("EXPLAIN_5D")),
            ("7", "form_8960_other_modifications", a.other, msg!("EXPLAIN_N_17")),
            ("8", "form_8960_investment_income", line_8, msg!("EXPLAIN_N_18")),
            ("9a", "form_8960_investment_interest", line_9a, msg!("EXPLAIN_9A")),
            ("9b", "form_8960_income_and_property_taxes", line_9b, msg!("EXPLAIN_9B")),
            ("9c", "form_8960_miscellaneous_investment_expenses", 0, msg!("EXPLAIN_9C")),
            ("9d", "form_8960_investment_expenses", line_9d, msg!("EXPLAIN_9D")),
            ("10", "form_8960_additional_modifications", line_10, msg!("EXPLAIN_N_19")),
            ("11", "form_8960_deductions_and_modifications", line_11, msg!("EXPLAIN_N_20")),
            ("12", "form_8960_net_investment_income", line_12, msg!("EXPLAIN_N_21")),
            ("13", "form_8960_modified_agi", a.modified_agi, msg!("EXPLAIN_N_22")),
            ("14", "form_8960_threshold", a.threshold, msg!("EXPLAIN_N_23")),
            ("15", "form_8960_modified_agi_excess", line_15, msg!("EXPLAIN_N_24")),
            (
                "16",
                "form_8960_smaller_amount",
                phase::cap(line_12, [line_15]),
                msg!("EXPLAIN_N_25"),
            ),
            ("17", "form_8960_net_investment_income_tax", a.tax, msg!("EXPLAIN_N_26")),
        ],
        |(line, id, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form8960, line)),
                id,
                LineValue::MoneyDollars(amount),
                &["income", "deductions", "agi"],
                "https://www.irs.gov/instructions/i8960",
                explanation,
            ))
        },
    );
}

pub(super) struct NiitIn<'a> {
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) property_sources_known_absent: bool,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) source_claims: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) niit_capital_net: i64,
    pub(super) niit_capital: i64,
    pub(super) resident_niit_prior: Option<i64>,
    pub(super) separate_niit: bool,
    pub(super) resident: usize,
    pub(super) direct_property_sources: &'a [(i64, crate::community::PropertyCents)],
    pub(super) niit_magi: i64,
    pub(super) niit_threshold: i64,
    pub(super) claimed_casualty: i64,
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) property_net: i64,
    pub(super) loan_gain: i64,
    pub(super) hobby: i64,
    pub(super) nonresident_amounts: [i64; 4],
    pub(super) use_itemized: bool,
    pub(super) salt: i64,
    pub(super) salt_eligible: &'a Exact,
    pub(super) agi: i64,
    pub(super) savings_penalty: i64,
    pub(super) foreign_tax_deduction: &'a crate::extended_rules::ForeignTaxDeduction,
    pub(super) foreign_tax_deductible: i64,
    pub(super) foreign_investment_tax: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) investment_interest_deduction: i64,
    pub(super) deduction: i64,
    pub(super) stock_loss: i64,
    pub(super) itemized_reduction: i64,
    pub(super) gst_tax: i64,
    pub(super) bond_premium: i64,
    pub(super) debt_instrument_loss: i64,
    pub(super) impairment_work: i64,
    pub(super) educator_itemized: i64,
    pub(super) annuity_loss: i64,
    pub(super) gambling_loss: i64,
    pub(super) medical_deduction: i64,
    pub(super) mortgage_deductible_interest: i64,
    pub(super) charity_deduction: i64,
    pub(super) casualty_deduction: i64,
    pub(super) schedule_a_state_local: &'a Exact,
    pub(super) rental_personal: &'a [Exact; 2],
    pub(super) office_schedule_a: &'a crate::home_office::ScheduleAShares,
    pub(super) estate_state_income_tax: i64,
    pub(super) estate_state_sales_tax: i64,
    pub(super) investment_unavailable: bool,
}

pub(super) struct NiitOut {
    pub(super) prior_niit_difference: i64,
    pub(super) estate_niit_prior_sources_known: bool,
    pub(super) self_rental_current_components_zero: bool,
    pub(super) property_niit: Option<crate::section1231::PropertyNiit>,
    pub(super) niit_casualty_loss: i64,
    pub(super) passive_business_niit: i64,
    pub(super) income_tax_expense: Option<i64>,
    pub(super) niit_itemized: i64,
    pub(super) niit_deduction_components: [i64; 5],
    pub(super) injured_niit_itemized:
        alloc::boxed::Box<crate::injured_spouse::niit_deductions::NiitItemizedDeductions>,
    pub(super) estate_niit_excess_deduction: i64,
    pub(super) nii: i64,
    pub(super) niit: i64,
}

pub(super) fn niit_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: NiitIn<'_>,
) -> Result<Option<NiitOut>, crate::Section179PreparedIssue> {
    let &Pass { d, context, allocated_entries, ded, repayment, .. } = pass;
    let NiitIn {
        property_character,
        property_sources_known_absent,
        rental,
        source_claims,
        niit_capital_net,
        niit_capital,
        resident_niit_prior,
        separate_niit,
        resident,
        direct_property_sources,
        niit_magi,
        niit_threshold,
        claimed_casualty,
        interest,
        dividends,
        property_net,
        loan_gain,
        hobby,
        nonresident_amounts,
        use_itemized,
        salt,
        salt_eligible,
        agi,
        savings_penalty,
        foreign_tax_deduction,
        foreign_tax_deductible,
        foreign_investment_tax,
        investment_property_taxes,
        investment_interest_deduction,
        deduction,
        stock_loss,
        itemized_reduction,
        gst_tax,
        bond_premium,
        debt_instrument_loss,
        impairment_work,
        educator_itemized,
        annuity_loss,
        gambling_loss,
        medical_deduction,
        mortgage_deductible_interest,
        charity_deduction,
        casualty_deduction,
        schedule_a_state_local,
        rental_personal,
        office_schedule_a,
        estate_state_income_tax,
        estate_state_sales_tax,
        investment_unavailable,
    } = i;
    let niit_savings_penalty = savings_penalty;
    let mut prior_niit_difference = crate::extended_rules::prior_niit_difference(d, r);
    let estate_niit_prior_sources_known = allocated_entries.iter().all(|entry| {
        let FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
            return true;
        };
        let data = &crate::FederalEstateTrustPortfolioK1::from(data);
        let carry = data.final_short_carry + data.final_long_carry;
        // An absent final-character statement with a positive succeeded
        // attribute is unknown, even though it supplies no native operand.
        data.final_character
            .as_ref()
            .map_or(carry == 0, |finals| finals.capital.niit_difference.is_some() || carry == 0)
    });
    // Reg. 1.642(h)-1(b) preserves the succeeded loss character. Feed the
    // fiduciary's retained excluded-loss balance into the same net-gains
    // worksheet as personal carryovers; never reconstruct the estate return.
    for entry in allocated_entries {
        if let FederalIncome::EstateTrustPortfolio { data } = &entry.income
            && let data = &crate::FederalEstateTrustPortfolioK1::from(data)
            && let Some(finals) = &data.final_character
        {
            let carry = data.final_short_carry + data.final_long_carry;
            let difference = finals.capital.niit_difference.unwrap_or_else(|| {
                if carry > 0 {
                    note_once_scoped(
                        r,
                        msg!("WARN_ESTATE_CAPITAL_NIIT_LEFT_OUT"),
                        crate::interview::income_issue_context(entry),
                    );
                }
                // Omit only the unknown investment-tax loss benefit. Ordinary
                // and AMT carryovers and the accepted source stay unchanged.
                carry
            });
            prior_niit_difference += dollars(difference);
        }
    }
    if let Some(prior) = resident_niit_prior {
        prior_niit_difference = prior;
    }
    let self_rental_excluded = property_character.as_ref().map_or(
        Ok(crate::asset_sale::SelfRentalNiit {
            excluded: [0, 0],
            current_components_known_zero: property_sources_known_absent,
        }),
        |character| {
            crate::asset_sale::self_rental_niit_with_claims(d, rental, character, source_claims)
        },
    );
    let self_rental_current_components_zero =
        self_rental_excluded.as_ref().is_ok_and(|value| value.current_components_known_zero);
    let self_rental_excluded = match self_rental_excluded {
        Ok(value) => value.excluded,
        Err(error) => {
            error.report_at(
                r,
                &crate::IssueContext::Section(crate::TaxSection::Rentals),
                #[cfg(test)]
                "Self-rental disposition investment tax",
            );
            if error.is_required_source() {
                return Ok(None);
            }
            [0, 0]
        }
    };
    let property_niit = if let Some(character) = property_character.as_ref() {
        match crate::section1231::property_niit_with_claims(
            context.forms(d),
            rental,
            character,
            niit_capital_net,
            niit_capital,
            prior_niit_difference,
            -dollars(t.nua_excluded_gain) - self_rental_excluded[1],
        ) {
            Ok(mut value) => {
                value.adjustment -= self_rental_excluded[0];
                Some(value)
            }
            Err(error) => {
                error.report_at(
                    r,
                    &crate::IssueContext::Section1231,
                    #[cfg(test)]
                    "Property investment tax",
                );
                if error.is_required_source() {
                    return Ok(None);
                }
                None
            }
        }
    } else {
        let [adjustment, next_capital_adjustment] = crate::capital::niit_adjustment(
            prior_niit_difference,
            -dollars(t.nua_excluded_gain) - self_rental_excluded[1],
            niit_capital_net,
            niit_capital,
        );
        let passive_property_ordinary_cents = crate::kernel::ratio::checked_sum(
            direct_property_sources
                .iter()
                .filter(|(id, _)| {
                    crate::section1231::direct_property_is_passive(context.forms(d), *id)
                })
                .map(|(_, value)| value.ordinary),
        )
        .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
        Some(crate::section1231::PropertyNiit {
            adjustment: adjustment + dollars(passive_property_ordinary_cents),
            next_capital_adjustment,
            current_components_known_zero: property_sources_known_absent
                && passive_property_ordinary_cents == 0,
        })
    };
    if let Some(value) = &property_niit {
        emit_lines(
            r,
            &mut (),
            [
                (
                    "niit_disposition_adjustment",
                    value.adjustment,
                    msg!("EXPLAIN_NIIT_DISPOSITION_ADJUSTMENT"),
                ),
                (
                    "niit_capital_carry_adjustment",
                    value.next_capital_adjustment,
                    msg!("EXPLAIN_NIIT_CAPITAL_CARRY_ADJUSTMENT"),
                ),
            ],
            |(id, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["income.scorp.section1231", "income.nua_sale"],
                    "https://www.irs.gov/instructions/i8960",
                    explanation,
                ))
            },
        );
    }
    // Section 1411(c)(4) and proposed Regulations section 1.1411-7: gain
    // on S-corporation stock of a non-trading business may be partly
    // excluded from net investment income; only its absence is calculated.
    crate::require_localized!(
        r,
        t.k1_business_gain == 0 || niit_magi <= niit_threshold,
        &crate::IssueContext::Section(crate::TaxSection::PassThrough),
        "K-1 gain on the interest",
        "ISSUE_K1_GAIN_NIIT_ADJUSTMENT",
        "A gain on a partnership interest or S corporation shares — a distribution above your basis or a sale — from a business you materially participate in, with modified AGI above the net investment income tax threshold, needs the Form 8960 line 5c adjustment for the entity's business property (proposed Regulations section 1.1411-7), which this App does not calculate.",
    );
    // Form 8960 line 7: the personal casualty and theft loss taken into account in taxable
    // income (Schedule A line 16, or the qualified disaster loss added to the standard
    // deduction) reduces the gain on property (Reg. 1.1411-4(d)(3)(i), (f)(4)(i)).
    let niit_casualty_loss = claimed_casualty;
    let passive_business_niit = dollars(t.passive_business_niit.values().sum());
    let gross_nii = positive(
        passive_business_niit
            + interest
            + dividends
            + niit_capital
            + t.child.investment
            + rental.income
            + rental.scorp_niit_adjustment
            + rental.partner_niit_adjustment
            + rental.rental_niit_adjustment
            + rental.estate_trust_niit_adjustment
            + property_niit.as_ref().map_or(0, |value| value.adjustment)
            + property_net
            + loan_gain
            + hobby
            + dollars(t.substitute)
            + dollars(t.k1_line_7)
            + dollars(t.k1_nii_excluded)
            + dollars(t.nonqualified_annuity)
            + dollars(t.estate_niit_line_7)
            - niit_casualty_loss
            - nonresident_amounts[1]
            - nonresident_amounts[2],
    );
    let income_tax_expense = if use_itemized {
        ded.income_tax_niit(salt, salt_eligible, gross_nii, agi)
    } else {
        Some(0)
    };
    if use_itemized && niit_magi > niit_threshold && gross_nii > niit_savings_penalty {
        for id in &foreign_tax_deduction.unallocated_k3 {
            if let Some(entry) = allocated_entries.iter().find(|entry| entry.id == *id) {
                let context = crate::interview::income_issue_context(entry)
                    .alternatives(&[crate::TaxSection::MoreCredits]);
                note_once_scoped(
                    r,
                    Message::new(
                        "WARN_FOREIGN_K3_NIIT_TAX_NOT_ALLOCATED",
                        alloc::vec![MessageArg::Nested(alloc::boxed::Box::new(context.label()))],
                    ),
                    context,
                );
            }
        }
    }
    line(
        r,
        "foreign_income_tax_deduction",
        if use_itemized { foreign_tax_deductible } else { 0 },
        &[
            "credits.foreign_tax",
            "income.interest.foreign_tax",
            "income.dividend.foreign_tax",
            "income.estate_trust_portfolio.foreign_statement",
            "income.schedule_k3",
        ],
        "https://www.irs.gov/publications/p514",
        msg!("EXPLAIN_FOREIGN_INCOME_TAX_DEDUCTION"),
    );
    line(
        r,
        "foreign_investment_tax_deduction",
        if use_itemized { foreign_investment_tax } else { 0 },
        &[
            "credits.foreign_tax",
            "income.interest.foreign_tax",
            "income.dividend.foreign_tax",
            "income.estate_trust_portfolio.foreign_statement",
        ],
        "https://www.irs.gov/instructions/i8960",
        msg!("EXPLAIN_FOREIGN_INVESTMENT_TAX_DEDUCTION"),
    );
    let known_niit_itemized = if use_itemized {
        phase::cap(
            investment_property_taxes
                + investment_interest_deduction
                + dollars(t.estate_interest_ird_tax)
                + foreign_investment_tax,
            [deduction],
        )
    } else {
        0
    };
    // Allocation not resolved: none of the income tax reduces investment
    // income (the larger NIIT), with a warning where it could matter.
    if !(income_tax_expense.is_some()
        || (use_itemized && known_niit_itemized == deduction)
        || gross_nii <= savings_penalty + known_niit_itemized + stock_loss
        || niit_magi <= niit_threshold)
    {
        note_once(r, msg!("WARN_NIIT_STATE_TAX_NOT_ALLOCATED"));
    }
    // Final regulation 1.1411-4(f)(7)(ii) uses an aggregate lesser-of,
    // not the proportional disallowance from the proposed regulation. The
    // 2026 section 68 no longer excludes investment interest or other items.
    let niit_itemized_before_limit = if use_itemized {
        investment_property_taxes
            + investment_interest_deduction
            + income_tax_expense.unwrap_or(0)
            + foreign_investment_tax
            + dollars(t.estate_interest_ird_tax)
    } else {
        0
    };
    let niit_itemized =
        phase::cap(niit_itemized_before_limit, [if use_itemized { deduction } else { 0 }]);
    let niit_itemized_input = crate::injured_spouse::niit_deductions::NativeDeductionOperands {
        itemizing: use_itemized,
        // Actual ScheduleA component composition from this invocation.
        // Equal aggregate totals alone do not establish a common pool.
        portfolio_only_itemized: use_itemized
            && itemized_reduction == 0
            && gst_tax == 0
            && foreign_tax_deductible == 0
            && bond_premium == 0
            && debt_instrument_loss == 0
            && impairment_work == 0
            && repayment.deduction == 0
            && educator_itemized == 0
            && dollars(t.inherited_estate_tax + t.estate_interest_ird_tax) == 0
            && annuity_loss == 0
            && gambling_loss == 0
            && medical_deduction == 0
            && salt == 0
            && investment_property_taxes == 0
            && mortgage_deductible_interest == 0
            && charity_deduction == 0
            && casualty_deduction == 0,
        // Actual native ScheduleA component inventory proves that
        // the allowed itemized pool contains only these NII expenses.
        // Equal aggregate totals alone are not evidence of identity.
        same_itemized_pool: use_itemized
            && itemized_reduction == 0
            && gst_tax == 0
            && foreign_tax_deduction.tax == foreign_tax_deduction.investment
            && bond_premium == 0
            && debt_instrument_loss == 0
            && impairment_work == 0
            && repayment.deduction == 0
            && educator_itemized == 0
            && t.inherited_estate_tax == 0
            && annuity_loss == 0
            && gambling_loss == 0
            && medical_deduction == 0
            && income_tax_expense == Some(salt)
            && (salt == 0
                || (ded.state_income_tax_elected()
                    && *schedule_a_state_local == exact::whole(ded.income_tax_paid())
                    && ded.property_taxes() == 0
                    && ded.personal_property_taxes() == 0
                    && rental_personal[1] == exact::whole(0)
                    && office_schedule_a.taxes == exact::whole(0)
                    && t.estate_final_personal_property_tax == 0
                    && t.estate_final_real_estate_tax == 0
                    && estate_state_income_tax == 0
                    && estate_state_sales_tax == 0))
            && mortgage_deductible_interest == 0
            && charity_deduction == 0
            && casualty_deduction == 0,
        whole_itemized_pool: use_itemized
            && !investment_unavailable
            && itemized_reduction == 0
            && gst_tax == 0
            && foreign_tax_deduction.tax == foreign_tax_deduction.investment
            && bond_premium == 0
            && debt_instrument_loss == 0
            && impairment_work == 0
            && repayment.deduction == 0
            && educator_itemized == 0
            && t.inherited_estate_tax == 0
            && annuity_loss == 0
            && gambling_loss == 0
            && medical_deduction == 0
            && mortgage_deductible_interest == 0
            && charity_deduction == 0
            && casualty_deduction == 0,
        whole_itemized_dollars: if use_itemized { deduction } else { 0 },
        whole_income_tax_dollars: if salt == 0 {
            Some(0)
        } else if ded.state_income_tax_elected()
            && *schedule_a_state_local == exact::whole(ded.income_tax_paid())
            && ded.property_taxes() == 0
            && ded.personal_property_taxes() == 0
            && rental_personal[1] == exact::whole(0)
            && office_schedule_a.taxes == exact::whole(0)
            && t.estate_final_personal_property_tax == 0
            && t.estate_final_real_estate_tax == 0
            && estate_state_income_tax == 0
            && estate_state_sales_tax == 0
        {
            Some(salt)
        } else {
            None
        },
        allowed_dollars: niit_itemized,
        // Same condition as the native allowed-total emitter below.
        // A withheld provisional zero is not an established amount.
        allowed_available: !use_itemized
            || income_tax_expense.is_some()
            || known_niit_itemized == deduction,
        before_dollars: [
            investment_property_taxes,
            investment_interest_deduction,
            income_tax_expense.unwrap_or(0),
            foreign_investment_tax,
            dollars(t.estate_interest_ird_tax),
        ],
        income_tax_available: income_tax_expense.is_some(),
        foreign_cents: foreign_tax_deduction.investment_owners,
        foreign_capacities: foreign_tax_deduction.investment_capacities.clone(),
        estate_ird_cents: t.injured_niit_deduction_cents.estate_ird,
        current_k1_interest_cents: t.injured_niit_deduction_cents.k1_interest,
    };
    let injured_niit_itemized = alloc::boxed::Box::new(if separate_niit {
        crate::injured_spouse::niit_deductions::NiitItemizedDeductions::from_native_for_first(
            d,
            niit_itemized_input,
            resident,
        )
    } else {
        crate::injured_spouse::niit_deductions::NiitItemizedDeductions::from_native(
            d,
            niit_itemized_input,
        )
    });
    let niit_components = if separate_niit {
        let owners =
            injured_niit_itemized.owners_dollars.zip(injured_niit_itemized.before_owners_dollars);
        let Some((allowed, components)) = owners else {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::OtherItemized),
                "Net investment income tax deductions",
                "ISSUE_NIIT_NONRESIDENT_SPOUSE_DOCUMENTS_NEED_THE_JOINT_ELECTION",
                "Establish the resident spouse's allowed investment deductions before calculating separate net investment income tax figures.",
            );
            return Ok(None);
        };
        (allowed[resident], components[resident])
    } else {
        (
            niit_itemized,
            [
                investment_property_taxes,
                investment_interest_deduction,
                income_tax_expense.unwrap_or(0),
                if use_itemized { foreign_investment_tax } else { 0 },
                dollars(t.estate_interest_ird_tax),
            ],
        )
    };
    let (niit_itemized, niit_deduction_components) = niit_components;
    let niit_itemized_before_limit = if separate_niit {
        niit_deduction_components.iter().sum::<i64>()
    } else {
        niit_itemized_before_limit
    };
    if income_tax_expense.is_some() {
        line(
            r,
            "niit_income_tax_deduction",
            niit_deduction_components[2],
            &["deductions.choice", "income", "agi", "itemized_deduction_reduction"],
            "https://www.irs.gov/instructions/i8960",
            msg!("EXPLAIN_NIIT_INCOME_TAX_DEDUCTION"),
        );
    }
    for (id, amount) in [
        ("niit_itemized_deductions", niit_itemized),
        ("niit_itemized_deduction_reduction", niit_itemized_before_limit - niit_itemized),
    ] {
        if income_tax_expense.is_none()
            && (id == "niit_itemized_deduction_reduction" || known_niit_itemized != deduction)
        {
            continue;
        }
        line(
            r,
            id,
            amount,
            &["deductions", "itemized_deduction_reduction"],
            "https://www.irs.gov/irb/2013-51_IRB",
            msg!("EXPLAIN_FEDERAL_RULES_15"),
        );
    }
    // Section 529A distributions are income under section 72's allocation
    // method, not annuity payments or a pass-through of the program's
    // investments. Their taxable earnings raise Form 8960 MAGI through AGI,
    // but are not one of section 1411(c)(1)(A)'s net-investment-income items.
    let estate_niit_excess_deduction = dollars(t.estate_niit_excess_deduction);
    let nii = positive(
        gross_nii
            - niit_savings_penalty
            - niit_itemized
            - stock_loss
            - estate_niit_excess_deduction,
    );
    let niit = fraction(phase::cap(nii, [positive(niit_magi - niit_threshold)]), 38, 1000);
    Ok(Some(NiitOut {
        prior_niit_difference,
        estate_niit_prior_sources_known,
        self_rental_current_components_zero,
        property_niit,
        niit_casualty_loss,
        passive_business_niit,
        income_tax_expense,
        niit_itemized,
        niit_deduction_components,
        injured_niit_itemized,
        estate_niit_excess_deduction,
        nii,
        niit,
    }))
}

pub(super) fn nonqualified_annuity_output_phase(r: &mut FederalResult, t: &Totals) {
    line(
        r,
        "nonqualified_annuity_investment_income",
        dollars(t.nonqualified_annuity),
        &["income.retirement.kind", "income.retirement.taxable"],
        "https://www.irs.gov/instructions/i8960",
        msg!("EXPLAIN_NONQUALIFIED_ANNUITY_INVESTMENT_INCOME"),
    );
}

pub(super) struct NiitReportingIn<'a> {
    pub(super) use_itemized: bool,
    pub(super) deduction: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) investment_unavailable: bool,
    pub(super) investment_interest_deduction: i64,
    pub(super) income_tax_expense: Option<i64>,
    pub(super) salt: i64,
    pub(super) income_sales: i64,
    pub(super) foreign_investment_tax: i64,
    pub(super) investment_facts: Option<&'a crate::InvestmentInterest>,
    pub(super) property_niit: &'a Option<crate::section1231::PropertyNiit>,
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) nonresident_amounts: [i64; 4],
    pub(super) business: i64,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) royalties: i64,
    pub(super) passive_business_niit: i64,
    pub(super) capital: i64,
    pub(super) property_ordinary: i64,
    pub(super) loan_gain: i64,
    pub(super) stock_loss: i64,
    pub(super) property_net: i64,
    pub(super) niit_casualty_loss: i64,
    pub(super) hobby: i64,
    pub(super) niit_itemized: i64,
    pub(super) niit_deduction_components: [i64; 5],
    pub(super) savings_penalty: i64,
    pub(super) estate_niit_excess_deduction: i64,
    pub(super) niit_magi: i64,
    pub(super) niit_threshold: i64,
    pub(super) nii: i64,
    pub(super) niit: i64,
}

pub(super) fn niit_output_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: NiitReportingIn<'_>,
) {
    let &Pass { ded, .. } = pass;
    let NiitReportingIn {
        use_itemized,
        deduction,
        investment_property_taxes,
        investment_unavailable,
        investment_interest_deduction,
        income_tax_expense,
        salt,
        income_sales,
        foreign_investment_tax,
        investment_facts,
        property_niit,
        interest,
        dividends,
        nonresident_amounts,
        business,
        rental,
        royalties,
        passive_business_niit,
        capital,
        property_ordinary,
        loan_gain,
        stock_loss,
        property_net,
        niit_casualty_loss,
        hobby,
        niit_itemized,
        niit_deduction_components,
        savings_penalty,
        estate_niit_excess_deduction,
        niit_magi,
        niit_threshold,
        nii,
        niit,
    } = i;
    r.expense_roles = Some(alloc::boxed::Box::new(crate::deduction_owners::ExpenseRoleDollars {
        itemizing: use_itemized,
        itemized_dollars: if use_itemized { deduction } else { 0 },
        property_dollars: if use_itemized { investment_property_taxes } else { 0 },
        interest_dollars: (!investment_unavailable).then_some(investment_interest_deduction),
        connected_dollars: income_tax_expense,
        salt_dollars: if use_itemized { salt } else { 0 },
        income_tax_category_dollars: if use_itemized && ded.state_income_tax_elected() {
            income_sales
        } else {
            0
        },
        foreign_dollars: if use_itemized { foreign_investment_tax } else { 0 },
        current_paid_dollars: investment_facts.map(|facts| dollars(facts.paid)),
        retained_carry_dollars: investment_facts.map(|facts| dollars(facts.carryover)),
    }));
    if let (Some(property), Some(_)) = (property_niit, income_tax_expense) {
        form_8960_lines(
            r,
            Form8960Amounts {
                interest: interest - nonresident_amounts[1],
                dividends: dividends - nonresident_amounts[2],
                annuities: dollars(t.nonqualified_annuity),
                schedule_1_business: business + rental.income + royalties,
                business_adjustment: rental.scorp_niit_adjustment
                    + rental.partner_niit_adjustment
                    + rental.rental_niit_adjustment
                    + rental.estate_trust_niit_adjustment
                    + passive_business_niit
                    - business,
                gains: capital + property_ordinary + loan_gain - stock_loss,
                excluded_gains: property.adjustment - property_ordinary
                    + dollars(t.k1_nii_excluded),
                other: dollars(t.estate_niit_line_7) + property_net
                    - royalties
                    - niit_casualty_loss
                    + hobby
                    + dollars(t.substitute)
                    + dollars(t.k1_line_7)
                    + t.child.investment,
                investment_interest: niit_deduction_components[1],
                taxes: niit_deduction_components[0]
                    + niit_deduction_components[2]
                    + niit_deduction_components[3],
                estate_tax: niit_deduction_components[4],
                allowed: niit_itemized,
                savings_penalty,
                excess_deduction: estate_niit_excess_deduction,
                modified_agi: niit_magi,
                threshold: niit_threshold,
                nii,
                tax: niit,
            },
        );
    }
}
