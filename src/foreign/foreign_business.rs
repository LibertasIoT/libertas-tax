use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line;
use crate::{BusinessSource, FederalDraft, FederalIncome, FederalResult};

#[derive(Default)]
pub(crate) struct BusinessExclusion {
    pub gross_adjustment: i64,
    pub net_adjustment: i64,
    pub business_net: i64,
    // Foreign service receipts less expenses and half-SE, before Form 2555.
    pub earned_net: i64,
    pub elected: bool,
    pub business_gross: i64,
    pub related: i64,
    pub housing_deduction: i64,
    pub housing_exclusion: i64,
    pub statutory_gross_excluded: i64,
    pub statutory_net_excluded: i64,
}

impl BusinessExclusion {
    pub(crate) fn allocate(&self, receipts: &[i64], expenses: &[i64]) -> alloc::vec::Vec<i64> {
        let gross = crate::kernel::ratio::allocate_remaining(
            self.business_gross + self.housing_deduction,
            receipts,
        );
        let related = crate::kernel::ratio::allocate_remaining(self.related, expenses);
        gross.into_iter().zip(related).map(|(g, e)| g - e).collect()
    }
}

// Gross and net Form 2555 amounts differ: Schedule C and half-SE remain
// reported in full; line 44 restores their excluded-income share in line 45.
// `foreign_wages` is Part IV line 19 in whole dollars (W-2 pay for work abroad
// plus foreign-employer pay); `employee_exclusion` is the exclusion
// extended_rules::foreign figured on that pay alone.

// Forward the existing invocation operands and its borrowed home context together.

#[allow(clippy::too_many_arguments)]
pub(crate) fn exclude_with_section179_with_claims(
    d: &FederalDraft,
    owner: usize,
    half_by_business: &alloc::collections::BTreeMap<i64, i64>,
    foreign_wages: i64,
    employee_exclusion: i64,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<BusinessExclusion, crate::Section179PreparedIssue> {
    let businesses: alloc::vec::Vec<_> = d
        .income
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::Business { data }
                if usize::from(entry.owner == crate::DocumentOwner::Spouses) == owner =>
            {
                Some((entry.id, data))
            }
            _ => None,
        })
        .collect();
    if !businesses.iter().any(|(_, b)| matches!(b.source, BusinessSource::ForeignServices { .. })) {
        return Ok(BusinessExclusion::default());
    }
    // The businesses marked as run abroad take the person's exclusion and
    // housing only through a Form 2555 record; without one they are taxed.
    let Some(claim) = crate::extended_rules::foreign_claim(d, owner) else {
        return Ok(BusinessExclusion::default());
    };
    let context = crate::IssueContext::owner(crate::TaxSection::WorkAbroad, claim.owner);
    let qualification = &claim.qualification;
    let housing =
        matches!(claim.housing, crate::ForeignHousing::Claim { .. }).then_some(&claim.housing);
    crate::require_localized!(
        r,
        housing.is_none()
            || !businesses.iter().any(|(id, b)| {
                matches!(b.source, BusinessSource::ForeignServices { .. })
                    && crate::statutory_wages::business(d, *id)
            }),
        &context,
        "Statutory employee foreign housing",
        "ISSUE_STATUTORY_FOREIGN_HOUSING",
        "Foreign housing with statutory employee pay needs the employee funding worksheet; the self-employed housing worksheet does not apply to that pay.",
    );
    let barred = claim.revocation.barred();
    let earned_election = claim.exclusion && !barred[0];
    let mut receipts = 0;
    let mut half_se = 0;
    let mut expenses = 0;
    let mut compatible = true;
    for (id, business) in &businesses {
        compatible &= matches!(business.health_insurance, crate::BusinessHealthInsurance::None)
            && matches!(business.retirement, crate::BusinessRetirement::None);
        if business.source == BusinessSource::UnitedStates {
            continue;
        }
        half_se += half_by_business.get(id).copied().unwrap_or(0);
        let cost = crate::section179_return::expenses_with_claims(
            d,
            *id,
            business,
            selected,
            home,
            source_claims,
        )?;
        receipts += business.receipts();
        expenses +=
            cost - selected.map_or(0, |allocation| allocation.ordinary_limit_adjustment(*id));
    }
    let receipts = dollars(receipts);
    let expenses = dollars(expenses);
    crate::require_localized!(
        r,
        compatible,
        &context.clone().alternatives(&[
            crate::TaxSection::SelfEmployedBenefits,
            crate::TaxSection::WorkAbroad,
        ]),
        "Foreign business exclusion",
        "ISSUE_ALL_THIS_PERSON_S_FOREIGN_SERVICE_BUSINESSES_AND_EMPLOYEE_PAY_MUST_B07F118D",
        "No owner health or retirement deductions with a business run abroad. Additional related deductions need coordinated worksheets; separately entered U.S. businesses remain outside the exclusion.",
    );
    let days = match crate::extended_rules::exclusion_days(qualification) {
        Ok(days) => days,
        Err((key, _message)) => {
            #[cfg(test)]
            let message = _message;

            crate::issue_localized!(r, &context, "Foreign business exclusion", key, message,);
            return Ok(BusinessExclusion { elected: true, ..BusinessExclusion::default() });
        }
    };
    let total_income = receipts + foreign_wages;
    let housing_amounts = housing
        .map(|h| h.calculate(qualification, foreign_wages, total_income, owner == 1, r))
        .unwrap_or_default();
    let housing_exclusion = if barred[1] { 0 } else { housing_amounts.exclusion };
    let gross = if earned_election {
        phase::cap(
            phase::floor(total_income - housing_exclusion, 0),
            [fraction(crate::year_constants::FOREIGN_EARNED_INCOME_EXCLUSION_DOLLARS, days, 365)],
        )
    } else {
        0
    };
    // Form 2555 line 44: deductions allocable to the excluded income, in
    // proportion to the foreign earned income exclusion (line 42) over foreign
    // earned income (line 27). Pub. 54 "Items Related to Excluded Income" and
    // Reg. 1.911-6(a) Example 5 use the exclusion alone in the numerator, so
    // the housing exclusion does not enter it (checked 2026-10-09; the map's
    // row 5 defect was not confirmed).
    let related =
        if total_income == 0 { 0 } else { fraction(expenses + half_se, gross, total_income) };
    let business_gross =
        if total_income == 0 { 0 } else { fraction(gross, receipts, total_income) };
    let business_net = business_gross - related;
    let employee_gross = gross - business_gross;
    let net = gross - related;
    let person = if owner == 0 { "taxpayer" } else { "spouse" };
    let housing_deduction = if let Some(crate::ForeignHousing::Claim { prior, .. }) = housing {
        let amount = housing_amounts.amount - housing_amounts.exclusion;
        crate::require_localized!(
            r,
            receipts > 0 || amount == 0,
            &context,
            "Foreign housing funding",
            "ISSUE_CREATING_A_NEW_HOUSING_CARRYOVER_WITHOUT_CURRENT_FOREIGN_SERVICE_EA_61F4C7F6",
            "Creating a new housing carryover without current foreign service earnings needs separate eligibility review. A prior carryover can expire when no income capacity remains.",
        );
        crate::foreign_housing::deduction(
            amount,
            total_income - housing_exclusion - gross,
            prior.as_ref(),
            person,
            r,
        )
    } else {
        0
    };
    emit_lines(
        r,
        &mut (),
        [
            ("gross_exclusion", business_gross),
            ("disallowed_deductions", related),
            ("net_exclusion", business_net),
        ],
        |(suffix, amount), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("foreign_business_{person}_{suffix}"),
                LineValue::MoneyDollars(amount),
                &["income.business.source", "income.business.expenses", "self_employment_tax"],
                "https://www.irs.gov/instructions/i2555",
                msg!("EXPLAIN_FOREIGN_BUSINESS_N_N"),
            ))
        },
    );
    if foreign_wages > 0 {
        line(
            r,
            &alloc::format!("foreign_employee_{person}_gross_exclusion"),
            employee_gross,
            &["income.foreign_employment", "income.business.source"],
            "https://www.irs.gov/instructions/i2555",
            msg!("EXPLAIN_FOREIGN_EMPLOYEE_N_GROSS_EXCLUSION"),
        );
    }
    let mut result = BusinessExclusion {
        gross_adjustment: gross - employee_exclusion,
        net_adjustment: net + housing_exclusion - employee_exclusion,
        housing_exclusion,
        business_net,
        earned_net: receipts - expenses - half_se,
        housing_deduction,
        business_gross,
        related,
        elected: earned_election || housing.is_some(),
        ..BusinessExclusion::default()
    };
    // Keep the existing owner's receipt/expense allocations. Statutory box 1
    // is compensation at gross, while its C profit is excluded from SE profit.
    // Therefore the compensation projection removes its gross exclusion, not
    // a second share of ordinary self-employment net income.
    let foreign_businesses: alloc::vec::Vec<_> = businesses
        .iter()
        .filter(|(_, b)| matches!(b.source, BusinessSource::ForeignServices { .. }))
        .collect();
    let receipts: alloc::vec::Vec<_> =
        foreign_businesses.iter().map(|(_, b)| b.receipts()).collect();
    let expenses: alloc::vec::Vec<_> = foreign_businesses
        .iter()
        .map(|(id, b)| {
            crate::section179_return::expenses_with_claims(d, *id, b, selected, home, source_claims)
                .map(|expense| {
                    expense
                        - selected.map_or(0, |allocation| allocation.ordinary_limit_adjustment(*id))
                })
        })
        .collect::<Result<_, _>>()?;
    let gross = crate::kernel::ratio::allocate_remaining(business_gross, &receipts);
    let net = result.allocate(&receipts, &expenses);
    for (index, (id, _)) in foreign_businesses.into_iter().enumerate() {
        if crate::statutory_wages::business(d, *id) {
            result.statutory_gross_excluded += gross[index];
            result.statutory_net_excluded += net[index];
        }
    }
    Ok(result)
}
