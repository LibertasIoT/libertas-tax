// 2026 W-2 instructions box 13; Schedule C/SE instructions and Pub. 596 Worksheet B.
// The saved W-2 owns the association. This is a working-copy route, never another stored ledger.
use crate::{FederalDraft, FederalIncome, FederalIncomeEntry, FederalResult, FederalWage};
use alloc::vec::Vec;

pub(crate) fn business(d: &FederalDraft, id: i64) -> bool {
    d.income.iter().any(|e| matches!(&e.income, FederalIncome::Wage { data } if { let data = FederalWage::from(data); data.statutory_employee && data.statutory_business == Some(id) }))
}
// Before the calculation copy routes box 1, the expenses-only foreign C has
// zero receipts. Its linked W-2 still supplies the person's foreign earnings.

pub(crate) fn foreign_gross_with_claims(
    d: &FederalDraft,
    e: &FederalIncomeEntry,
    w: &FederalWage,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    if w.statutory_employee
        && wage_rule_with_claims(d, e, w, false, source_claims).is_none()
        && d.income.iter().any(|target| {
            Some(target.id) == w.statutory_business
                && matches!(&target.income, FederalIncome::Business { data }
                    if matches!(data.source, crate::BusinessSource::ForeignServices { .. }))
        })
    {
        w.wages
    } else {
        0
    }
}

fn wage_rule_with_claims(
    d: &FederalDraft,
    e: &FederalIncomeEntry,
    w: &FederalWage,
    routed: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<&'static str> {
    if w.abroad != 0 {
        return Some("W2_STATUTORY_ABROAD_USE_BUSINESS");
    }
    let id = w.statutory_business?;
    let Some(target) = d.income.iter().find(|v| v.id == id) else {
        return Some("ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME");
    };
    if e.owner != target.owner {
        return Some("ISSUE_STATUTORY_BUSINESS_OTHER_OWNER");
    }
    let FederalIncome::Business { data } = &target.income else {
        return Some("ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME");
    };
    let expected = if routed {
        d.income
            .iter()
            .filter_map(|e| match &e.income {
                FederalIncome::Wage { data }
                    if {
                        let data = FederalWage::from(data);
                        data.statutory_employee && data.statutory_business == Some(id)
                    } =>
                {
                    Some(FederalWage::from(data).wages)
                }
                _ => None,
            })
            .sum()
    } else {
        0
    };
    if !matches!(data.activity, crate::BusinessActivity::Ordinary { receipts, .. } if receipts == expected)
        || data.receipts() != expected
        || crate::information_returns::routed_to_with_claims(d, id, source_claims) != 0
        || data.health_insurance != crate::BusinessHealthInsurance::None
        || data.retirement != crate::BusinessRetirement::None
    {
        return Some("ISSUE_STATUTORY_BUSINESS_HAS_OTHER_INCOME");
    }
    None
}

pub(crate) fn route_with_claims(
    d: &mut FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) {
    let adds: Vec<_> = d
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Wage { data }
                if {
                    let data = FederalWage::from(data);
                    data.statutory_employee
                        && wage_rule_with_claims(d, e, &data, false, source_claims).is_none()
                } =>
            {
                let data = FederalWage::from(data);
                data.statutory_business.map(|id| (id, data.wages))
            }
            _ => None,
        })
        .collect();
    for (id, amount) in adds {
        if let Some(FederalIncomeEntry { income: FederalIncome::Business { data }, .. }) =
            d.income.iter_mut().find(|e| e.id == id)
            && let crate::BusinessActivity::Ordinary { receipts, .. } = &mut data.activity
        {
            *receipts += amount;
        }
    }
}

pub(crate) fn validate_with_claims(
    d: &FederalDraft,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) {
    for e in &d.income {
        if let FederalIncome::Wage { data } = &e.income {
            let data = &FederalWage::from(data);
            if !data.statutory_employee {
                continue;
            }
            if let Some(key) = wage_rule_with_claims(d, e, data, true, source_claims) {
                crate::issue_localized!(
                    r,
                    &crate::interview::income_issue_context(e).alternatives(&[
                        crate::TaxSection::SelfEmployment,
                        crate::TaxSection::Wages,
                    ]),
                    "Statutory employee Schedule C",
                    key,
                    "Select a Schedule C of the same owner containing only this statutory job's expenses, no other receipts, routed forms or owner health/retirement deductions.",
                );
            }
        }
    }
}
pub(crate) fn omitted(
    original: &FederalDraft,
    d: &mut FederalDraft,
    notes: &mut Vec<crate::federal_model::ReviewNote>,
) -> bool {
    let mut financial_omission = false;
    let removed: Vec<_> = original
        .income
        .iter()
        .filter(|e| {
            matches!(e.income, FederalIncome::Business { .. })
                && !d.income.iter().any(|v| v.id == e.id)
        })
        .map(|e| e.id)
        .collect();
    for e in &mut d.income {
        if let FederalIncome::Wage { data: crate::WageSource::W2(source) } = &mut e.income
            && source.form.statutory
            && source.answers.statutory_business.is_some_and(
                |crate::StatutoryBusiness::Business { source }| removed.contains(&source),
            )
        {
            financial_omission = true;
            source.form.wages = Some(0);
            source.answers.abroad = None;
            source.answers.deferred_pension = None;
            source.answers.inmate_pay = 0;
            source.answers.medicaid_waiver = 0;
            source.answers.statutory_business = None;
            let context = crate::interview::income_issue_context(e);
            notes.push(crate::federal_model::ReviewNote::scoped(#[cfg(test)] "Statutory employee pay is left out with its unsupported Schedule C; withholding and payroll boxes are retained.",crate::Message::new("WARN_STATUTORY_PAY_LEFT_OUT",alloc::vec![]),context));
        }
    }
    financial_omission
}
