// form_8912 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form8912In {
    pub(super) remaining_tax: i64,
    pub(super) child_nonrefund: i64,
    pub(super) adoption_used: i64,
    pub(super) mortgage_credit_used: i64,
    pub(super) dc_homebuyer_credit: i64,
    pub(super) residential_energy_credit: i64,
    pub(super) personal_refueling_credit: i64,
    pub(super) minimum_credit: i64,
    pub(super) general_business_credit: i64,
}

pub(super) fn form_8912_phase(pass: &Pass, r: &mut FederalResult, i: Form8912In) -> i64 {
    let &Pass { c, .. } = pass;
    let Form8912In {
        remaining_tax,
        child_nonrefund,
        adoption_used,
        mortgage_credit_used,
        dc_homebuyer_credit,
        residential_energy_credit,
        personal_refueling_credit,
        minimum_credit,
        general_business_credit,
    } = i;
    let bond_line1: i64 = c
        .tax_credit_bonds
        .iter()
        .filter(|b| !matches!(b.answers.origin, Some(crate::TaxCreditOrigin::EntityStatement(_))))
        .map(crate::FederalTaxCreditBond1097::current_dollars)
        .sum();
    let bond_line2: i64 = c
        .tax_credit_bonds
        .iter()
        .filter(|b| matches!(b.answers.origin, Some(crate::TaxCreditOrigin::EntityStatement(_))))
        .map(crate::FederalTaxCreditBond1097::current_dollars)
        .sum();
    let bond_line3: i64 =
        c.tax_credit_bonds.iter().map(crate::FederalTaxCreditBond1097::carry_dollars).sum();
    let bond_line4 = crate::tax_credit_bonds::available_dollars(&c.tax_credit_bonds);
    // Form 8912 lines 7–11: line 10b includes every other supported personal
    // nonrefundable credit; line 10d includes the prior-minimum-tax credit.
    // Form 8912 line 10c deducts the allowed Form 3800 credit before the bond credit.
    let bond_line11 = positive(
        remaining_tax
            - child_nonrefund
            - adoption_used
            - mortgage_credit_used
            - dc_homebuyer_credit
            - residential_energy_credit
            - personal_refueling_credit
            - minimum_credit
            - general_business_credit,
    );
    let bond_credit = phase::cap(bond_line4, [bond_line11]);
    let bond_allocations =
        crate::tax_credit_bonds::allocated_used_dollars(&c.tax_credit_bonds, bond_credit);
    if let Err(error) = &bond_allocations {
        let (key, _explanation) = error.message();
        require_message(
            r,
            false,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            #[cfg(test)]
            "Tax credit bond carry allocation",
            #[cfg(test)]
            alloc::format!(
                "Form 8912 line 12 is ${bond_credit} and line 4 is ${bond_line4}. {_explanation}"
            ),
            crate::Message::new(
                "ISSUE_BOND_ALLOCATION_SUMMARY",
                alloc::vec![
                    crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 0,
                        minor_units: bond_credit,
                    },
                    crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 0,
                        minor_units: bond_line4,
                    },
                    crate::MessageArg::Nested(alloc::boxed::Box::new(crate::Message::new(
                        key,
                        alloc::vec![],
                    ))),
                ],
            ),
        );
    }
    if !c.tax_credit_bonds.is_empty() {
        for (id, amount, explanation) in [
            ("form_8912_line1", bond_line1, msg!("EXPLAIN_FORM_N_LINE1")),
            ("form_8912_line_2", bond_line2, msg!("EXPLAIN_CREDIT_ENTITY_STATEMENTS")),
            ("form_8912_line_14", bond_line1, msg!("EXPLAIN_FORM_N_LINE1")),
            ("form_8912_line_20", bond_line2, msg!("EXPLAIN_CREDIT_ENTITY_STATEMENTS")),
            ("form_8912_line3", bond_line3, msg!("EXPLAIN_FORM_N_LINE3")),
            ("form_8912_line4", bond_line4, msg!("EXPLAIN_FORM_N_LINE4")),
            ("form_8912_line11", bond_line11, msg!("EXPLAIN_FORM_N_LINE11")),
            ("tax_credit_bond_credit", bond_credit, msg!("EXPLAIN_TAX_CREDIT_BOND_CREDIT")),
        ] {
            let reference = match id {
                "form_8912_line1" => "1",
                "form_8912_line_2" => "2",
                "form_8912_line_14" => "14",
                "form_8912_line_20" => "20",
                "form_8912_line3" => "3",
                "form_8912_line4" => "4",
                "form_8912_line11" => "11",
                "tax_credit_bond_credit" => "12",
                _ => unreachable!(),
            };
            form_line(
                r,
                FormLine::new(TaxForm::Form8912, reference),
                id,
                amount,
                &["credits.tax_credit_bonds", "income_tax", "credits"],
                "https://www.irs.gov/pub/irs-pdf/f8912.pdf",
                explanation,
            );
        }
        for (index, bond) in c.tax_credit_bonds.iter().enumerate() {
            let Some(answers) = bond.resolved_answers() else {
                continue;
            };
            let part = if matches!(answers.origin, crate::TaxCreditOrigin::EntityStatement(_)) {
                "iv"
            } else {
                "iii"
            };
            form_line(
                r,
                FormLine::new(TaxForm::Form8912, if part == "iv" { "18(f)" } else { "13(c)" }),
                &alloc::format!("tax_credit_bond_{part}_{}", index + 1),
                bond.current_dollars(),
                &["credits.tax_credit_bonds"],
                "https://www.irs.gov/instructions/i8912",
                Message::new(
                    "EXPLAIN_CREDIT_SOURCE_ROW",
                    alloc::vec![
                        MessageArg::Text(bond.form.issuer.name.clone()),
                        MessageArg::Text(bond.form.unique_identifier.clone().unwrap_or_default()),
                        MessageArg::Text(String::from(if part == "iv" {
                            "IV line 18(f)"
                        } else {
                            "III line 13(c)"
                        })),
                    ],
                ),
            );
        }
        if let Ok(used_by_bond) = bond_allocations {
            for (index, (bond, used)) in c.tax_credit_bonds.iter().zip(used_by_bond).enumerate() {
                let available = bond.current_dollars() + bond.carry_dollars();
                line(
                    r,
                    &alloc::format!("tax_credit_bond_used_{}", index + 1),
                    used,
                    &[
                        "credits.tax_credit_bonds.answers.allowed_credit_dollars",
                        "credits.tax_credit_bonds.answers.allowed_credit_confirmed",
                        "tax_credit_bond_credit",
                    ],
                    "https://www.irs.gov/instructions/i8912",
                    Message::new(
                        "EXPLAIN_TAX_CREDIT_BOND_USED_N",
                        alloc::vec![
                            MessageArg::Text(bond.form.issuer.name.clone()),
                            MessageArg::Text(
                                bond.form.unique_identifier.clone().unwrap_or_default()
                            )
                        ],
                    ),
                );
                line(
                    r,
                    &alloc::format!("tax_credit_bond_carry_2027_{}", index + 1),
                    crate::kernel::carry::closing_values::<crate::kernel::carry::Dollars>(
                        available, used,
                    )
                    .expect("validated same-bond credit allocation")
                    .get(),
                    &[
                        "credits.tax_credit_bonds.answers.allowed_credit_dollars",
                        "credits.tax_credit_bonds.answers.allowed_credit_confirmed",
                        "tax_credit_bond_credit",
                    ],
                    "https://www.irs.gov/instructions/i8912",
                    Message::new(
                        "EXPLAIN_TAX_CREDIT_BOND_CARRY_N_N",
                        alloc::vec![
                            MessageArg::Text(bond.form.issuer.name.clone()),
                            MessageArg::Text(
                                bond.form.unique_identifier.clone().unwrap_or_default()
                            )
                        ],
                    ),
                );
            }
        }
    }
    bond_credit
}

pub(super) struct TaxCreditBondSourceOut {
    pub(super) late_investment_sources: [[i64; 2]; 2],
    pub(super) investment_sources_known: [bool; 2],
}

pub(super) fn tax_credit_bond_source_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
) -> core::ops::ControlFlow<(), TaxCreditBondSourceOut> {
    let &Pass { c, s, p, .. } = pass;
    #[cfg(test)]
    let mut fixture_index = 0;
    for bond in &c.tax_credit_bonds {
        #[cfg(test)]
        let source = {
            fixture_index += 1;
            alloc::format!("Tax credit bond {fixture_index}")
        };
        crate::require_localized!(
            r,
            bond.resolved_answers()
                .is_none_or(|answers| answers.kind.issue_date_allowed(answers.issued_on))
                && (bond.owner == Owner::Taxpayer
                    || (joint(s.filing_status()) && p.spouse.is_some())),
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            &source,
            "ISSUE_FORM_8912_REQUIRES_A_PRE_2018_QUALIFIED_TAX_CREDIT_OR_HOLDER_CREDIT_57C707E1",
            "Form 8912 requires an eligible legacy bond issue date and a permitted filer owner. Credit limitations and required income are evaluated separately from these source facts.",
        );
    }
    let mut late_investment_sources = [[0; 2]; 2];
    let mut investment_sources_known = [true; 2];
    for bond in &c.tax_credit_bonds {
        let owner = usize::from(bond.owner == Owner::Spouse);
        late_investment_sources[0][owner] += bond.interest_income();
        late_investment_sources[1][owner] += bond.dividend_income();
        investment_sources_known[0] &= !bond.income_unknown();
    }
    let bond_interest = crate::tax_credit_bonds::current_interest_cents(&c.tax_credit_bonds);
    if checked_native_add(&mut t.interest, i128::from(bond_interest)).is_none() {
        CollectFailure::Broker(crate::IssueContext::Section(crate::TaxSection::InvestmentSales))
            .report(r);
        withhold_required_source_result(r);
        return core::ops::ControlFlow::Break(());
    }
    t.dividends += c
        .tax_credit_bonds
        .iter()
        .map(crate::FederalTaxCreditBond1097::dividend_income)
        .sum::<i64>();
    line_cents(
        r,
        "bond_credit_deemed_interest",
        bond_interest,
        &["credits.tax_credit_bonds.form.box1_credit"],
        "https://www.irs.gov/instructions/i8912",
        msg!("EXPLAIN_BOND_CREDIT_DEEMED_INTEREST"),
    );
    core::ops::ControlFlow::Continue(TaxCreditBondSourceOut {
        late_investment_sources,
        investment_sources_known,
    })
}
