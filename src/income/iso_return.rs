pub(crate) fn reconcile_wages(d: &crate::FederalDraft, r: &mut crate::FederalResult) {
    for owner in [crate::Owner::Taxpayer, crate::Owner::Spouse] {
        let mut reported = 0;
        let mut wages = 0;
        for entry in
            d.income.iter().filter(|entry| entry.owner == crate::DocumentOwner::from(owner))
        {
            match &entry.income {
                crate::FederalIncome::EsppSale { data } if data.in_wages => {
                    reported += data.compensation()
                }
                crate::FederalIncome::IsoSale { data } if data.in_wages => {
                    reported += data.compensation()
                }
                crate::FederalIncome::EmployeeStockSale { data }
                    if data.acquired / 10000 == 2026 =>
                {
                    reported += data.compensation()
                }
                crate::FederalIncome::Wage { data } => {
                    wages += crate::FederalWage::from(data).wages
                }
                _ => {}
            }
        }
        if reported > wages {
            for entry in
                d.income.iter().filter(|entry| entry.owner == crate::DocumentOwner::from(owner))
            {
                let context = crate::interview::income_issue_context(entry);
                let context = match &entry.income {
                    crate::FederalIncome::IsoSale { data }
                        if data.in_wages && data.compensation() > 0 =>
                    {
                        context.alternatives(&[
                            crate::TaxSection::InvestmentSales,
                            crate::TaxSection::Wages,
                        ])
                    }
                    // Each kind has its Investment sales page, and the W-2 is
                    // the Wages page's.
                    crate::FederalIncome::EsppSale { data }
                        if data.in_wages && data.compensation() > 0 =>
                    {
                        context.alternatives(&[
                            crate::TaxSection::InvestmentSales,
                            crate::TaxSection::Wages,
                        ])
                    }
                    crate::FederalIncome::EmployeeStockSale { data }
                        if data.acquired / 10000 == 2026 && data.compensation() > 0 =>
                    {
                        context.alternatives(&[
                            crate::TaxSection::InvestmentSales,
                            crate::TaxSection::Wages,
                        ])
                    }
                    _ => continue,
                };
                crate::issue_localized!(
                    r,
                    &context,
                    "ISO compensation",
                    "ISSUE_COMBINED_STOCK_COMPENSATION_MARKED_AS_ALREADY_REPORTED_CANNOT_EXCEE_0E5F6C00",
                    "Combined stock compensation marked as already reported cannot exceed this filer's entered W-2 box 1 wages. Reconcile employer reporting before calculating.",
                );
            }
        }
    }
}

// A printed ISO sale (FederalIncome::IsoSaleSource) and its answers on
// Investment sales; absent answers are not answered yet.
pub(crate) fn answers(draft: &crate::FederalDraft, id: i64) -> Option<&crate::IsoSaleAnswers> {
    draft.screening.as_ref()?.iso_sale_answers.iter().find(|row| row.document_id == id)
}

// The complete lot, or the first answer it still needs.
pub(crate) fn resolve(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Result<crate::IsoSale, crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue as Issue;
    let crate::FederalIncome::IsoSaleSource { data: paper } = &entry.income else {
        unreachable!("a printed ISO sale")
    };
    let answer = answers(draft, entry.id);
    let more = answer
        .and_then(|answer| answer.more.as_ref())
        .ok_or_else(|| Issue::new("ISO_SALE_ADJUSTMENTS_NEEDED").at_field("more"))?;
    let mut sale = paper.lot(more);
    if !sale.valid() {
        return Err(Issue::new(
            "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
        )
        .at_field("more"));
    }
    if sale.confirmed && sale.compensation() > 0 {
        sale.in_wages = answer
            .and_then(|answer| answer.wage_inclusion)
            .ok_or_else(|| Issue::new("ISO_WAGE_INCLUSION_NEEDED").at_field("wage_inclusion"))?;
    }
    Ok(sale)
}

// The W-2 question applies only to a sale with compensation.
pub(crate) fn wage_needed(draft: &crate::FederalDraft, entry: &crate::FederalIncomeEntry) -> bool {
    let crate::FederalIncome::IsoSaleSource { data: paper } = &entry.income else {
        return false;
    };
    answers(draft, entry.id).and_then(|answer| answer.more.as_ref()).is_some_and(|more| {
        let sale = paper.lot(more);
        sale.confirmed && sale.compensation() > 0
    })
}

// Answers follow their printed sales: a removed sale's answers go, and a
// W-2 answer goes once the sale has no compensation.
pub(crate) fn reconcile(draft: &mut crate::FederalDraft) {
    let keep: alloc::vec::Vec<(i64, bool)> = draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, crate::FederalIncome::IsoSaleSource { .. }))
        .map(|entry| (entry.id, wage_needed(draft, entry)))
        .collect();
    if let Some(screening) = &mut draft.screening {
        screening.iso_sale_answers.retain(|row| keep.iter().any(|(id, _)| *id == row.document_id));
        for row in &mut screening.iso_sale_answers {
            if keep.iter().any(|(id, wage)| *id == row.document_id && !*wage) {
                row.wage_inclusion = None;
            }
        }
    }
}

// A complete sale saved before its answers moved to Investment sales becomes
// its printed sale on the next Document center save: its own answers come
// with it, so nothing it already said is asked again.
pub(crate) fn carry_lot_answers(before: &crate::FederalDraft, draft: &mut crate::FederalDraft) {
    let carried: alloc::vec::Vec<crate::IsoSaleAnswers> = draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, crate::FederalIncome::IsoSaleSource { .. }))
        .filter(|entry| answers(draft, entry.id).is_none())
        .filter_map(|entry| {
            let crate::FederalIncome::IsoSale { data } =
                &before.income.iter().find(|old| old.id == entry.id)?.income
            else {
                return None;
            };
            Some(crate::IsoSaleAnswers {
                document_id: entry.id,
                more: Some(crate::IsoShareSaleMore {
                    selling_fees: data.selling_fees,
                    acquisition_costs: data.acquisition_costs,
                    option_cost: data.option_cost,
                    special: !data.confirmed,
                }),
                wage_inclusion: (data.compensation() > 0 && data.confirmed)
                    .then_some(data.in_wages),
            })
        })
        .collect();
    if !carried.is_empty() {
        crate::interview::fill_unasked_screening(draft);
        draft.screening.as_mut().unwrap().iso_sale_answers.extend(carried);
    }
}
