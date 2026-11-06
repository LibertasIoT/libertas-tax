use crate::validation::ValidationIssue;
// Canonical projections of the supported calculation working copy.
// Ownership and adjustments are applied by their owners before this projection.
// IRC 6049; IRS 2026 draft Schedule B pp. B-1–B-3 and Pub.550 (2025) pp.25–27.
use crate::{FederalIncome, FederalIncomeEntry};
mod annual;
pub use crate::annual_debt_source::{
    AnnualDebtRecord, AnnualDebtRecords, AnnualDebtTreatment, DebtComponents, DebtPremiumOffset,
    OriginalDebtIssue, PremiumHistory,
};
pub use crate::debt_reporting_source::DebtReporting;
pub use crate::investment_allocation_source::{
    DividendNominee, InterestAdjustments, InterestNominee, OidNominee, SellerMortgageParty,
};
pub(crate) use crate::investment_validation_source::{
    div_issue, div_paper_issue, div_printed_issue, div_rule, div_stored_issue, int_issue,
    int_printed_issue, int_rule, int_stored_issue, oid_issue, oid_rule, oid_stored_issue,
};
pub use annual::{BondPremiumInstrument, BondPremiumSource};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct InvestmentComponents {
    pub interest: i64,
    pub exempt_interest: i64,
    pub private_activity: i64,
    pub ordinary_dividends: i64,
    pub qualified_dividends: i64,
    pub capital_distributions: i64,
    pub unrecaptured_distributions: i64,
    pub collectible_distributions: i64,
    pub section_1202_gain: i64,
    pub section199a: i64,
    pub foreign_tax: i64,
    pub withholding: i64,
}

// A fund's per-person issuer limit applies after the gain follows ownership.
pub(crate) fn allocated_fund_gain(
    gain: i64,
    exclusion: Option<crate::FundQsbsExclusion>,
    share: Option<crate::community::Share>,
    separate: bool,
) -> crate::qsbs::FundGain {
    crate::qsbs::fund_gain(share.map_or(gain, |share| share.gain(gain)), exclusion, separate)
}

// Call only on the central unknowns::supported working copy: accepted input
// identities/owners are unchanged, but death/nominee allocations have been applied.
// Return-level Form 8815 exclusion remains its owner's separate output.
pub(crate) fn components(entry: &FederalIncomeEntry) -> Option<InvestmentComponents> {
    let mut result = InvestmentComponents::default();
    match &entry.income {
        FederalIncome::Interest { data } => {
            let data = &crate::FederalInterest::from(data);
            result.interest = data.net_taxable();
            result.exempt_interest = data.net_exempt();
            result.private_activity = data
                .adjustments
                .as_ref()
                .and_then(|a| a.instrument_records.as_ref())
                .map_or(data.private_activity, |r| r.private_activity());
            result.foreign_tax = data
                .adjustments
                .as_ref()
                .and_then(|a| a.instrument_records.as_ref())
                .map_or(data.foreign_tax, |r| r.foreign_tax());
            result.withholding = data.withholding;
        }
        FederalIncome::Oid { data } => {
            let data = &crate::FederalOid::from(data);
            result.interest = data.net_taxable();
            result.exempt_interest =
                data.adjustments.as_ref().and_then(|a| a.instrument_records.as_ref()).map_or_else(
                    || crate::kernel::ratio::positive(data.exempt - data.tax_exempt_bond_premium),
                    |r| r.exempt(),
                );
            result.private_activity = data
                .adjustments
                .as_ref()
                .and_then(|a| a.instrument_records.as_ref())
                .map_or(data.private_activity, |r| r.private_activity());
            result.withholding = data.withholding;
        }
        FederalIncome::Dividend { data } => {
            let data = &crate::FederalDividend::from(data);
            result.ordinary_dividends = data.ordinary;
            result.qualified_dividends = data.qualified;
            result.capital_distributions = data.capital_distributions;
            result.unrecaptured_distributions = data.unrecaptured_distributions;
            result.collectible_distributions = data.collectible_distributions;
            result.section_1202_gain = data.section_1202_gain;
            result.section199a = data.section199a.qualified();
            result.exempt_interest = data.exempt_dividends;
            result.private_activity = data.private_activity_dividends;
            result.foreign_tax = data.foreign_tax;
            result.withholding = data.withholding;
        }
        _ => return None,
    }
    Some(result)
}

// Called after final-return allocation and before community-property allocation.
// Invalid personal facts are retained for normal stored-data validation, never repaired.
pub(crate) fn apply(
    d: &mut crate::FederalDraft,
    notes: &mut alloc::vec::Vec<crate::federal_model::ReviewNote>,
) -> bool {
    let mut financial_omission = false;
    let bond_sources = crate::savings_bonds::sources(&d.income);
    let election = d.screening.as_ref().and_then(|s| s.taxable_premium_election);
    for entry in &mut d.income {
        let context = crate::interview::income_issue_context(entry);
        let mut nominee = false;
        let mut premium_left_out = false;
        if let Some((keys, omitted)) = apply_annual(
            &mut entry.income,
            d.screening.as_ref().and_then(|s| s.taxable_premium_election),
        ) {
            financial_omission |= omitted;
            for key in keys {
                let message = crate::Message::new(key, alloc::vec::Vec::new());
                notes.push(crate::federal_model::ReviewNote::scoped(
                    #[cfg(test)]
                    message.english(),
                    message,
                    context.clone(),
                ));
            }
            continue;
        }
        match &mut entry.income {
            FederalIncome::Interest { data: crate::InterestSource::Int1099(data) }
                if int_rule(&crate::FederalInterest::from(&*data)).is_none() =>
            {
                if let Some(part) = data.nominee.take() {
                    nominee = [
                        part.taxable,
                        part.us_obligations,
                        part.exempt,
                        part.private_activity,
                        part.foreign_tax,
                        part.bond_premium,
                        part.treasury_bond_premium,
                        part.tax_exempt_bond_premium,
                        part.accrued_interest,
                        part.market_discount,
                    ]
                    .iter()
                    .any(|x| *x != 0);
                    data.form.interest_income -= part.taxable;
                    data.form.us_savings_bond_and_treasury_interest -= part.us_obligations;
                    data.form.tax_exempt_interest -= part.exempt;
                    data.form.specified_private_activity_bond_interest -= part.private_activity;
                    data.form.foreign_tax_paid -= part.foreign_tax;
                    data.form.bond_premium -= part.bond_premium;
                    data.form.bond_premium_on_treasury_obligations -= part.treasury_bond_premium;
                    data.form.bond_premium_on_tax_exempt_bond -= part.tax_exempt_bond_premium;
                    data.accrued_interest -= part.accrued_interest;
                    data.form.market_discount -= part.market_discount;
                }
                let a = data.adjustments.take().unwrap_or_default();
                data.form.us_savings_bond_and_treasury_interest -= a.previously_reported;
                if a.reporting == DebtReporting::Net {
                    data.form.bond_premium = 0;
                    data.form.bond_premium_on_treasury_obligations = 0;
                    data.form.bond_premium_on_tax_exempt_bond = 0;
                    if election != Some(true)
                        && let Some(offset) = a.net_premium_offset
                    {
                        data.form.interest_income += offset.ordinary;
                        data.form.us_savings_bond_and_treasury_interest += offset.treasury;
                        premium_left_out = offset.ordinary + offset.treasury > 0;
                    }
                } else if election != Some(true) {
                    premium_left_out =
                        data.form.bond_premium + data.form.bond_premium_on_treasury_obligations > 0;
                    data.form.bond_premium = 0;
                    data.form.bond_premium_on_treasury_obligations = 0;
                }
            }
            FederalIncome::Dividend { data }
                if div_rule(&crate::FederalDividend::from(&*data)).is_none() =>
            {
                if let Some(part) = data.nominee.take() {
                    nominee = [
                        part.ordinary,
                        part.qualified,
                        part.capital,
                        part.unrecaptured,
                        part.collectible,
                        part.section_1202,
                        part.section_199a,
                        part.exempt,
                        part.private_activity,
                        part.foreign_tax,
                        part.section_897_ordinary,
                        part.section_897_capital,
                    ]
                    .iter()
                    .any(|x| *x != 0);
                    crate::dividend_source::reduce_box(&mut data.form.ordinary, part.ordinary);
                    crate::dividend_source::reduce_box(&mut data.form.qualified, part.qualified);
                    crate::dividend_source::reduce_box(
                        &mut data.form.capital_distributions,
                        part.capital,
                    );
                    crate::dividend_source::reduce_box(
                        &mut data.form.unrecaptured_distributions,
                        part.unrecaptured,
                    );
                    crate::dividend_source::reduce_box(
                        &mut data.form.collectible_distributions,
                        part.collectible,
                    );
                    crate::dividend_source::reduce_box(
                        &mut data.form.section_1202_gain,
                        part.section_1202,
                    );
                    if data.form.section199a.unwrap_or(0) != 0 {
                        crate::dividend_source::reduce_box(
                            &mut data.form.section199a,
                            part.section_199a,
                        );
                    }
                    crate::dividend_source::reduce_box(
                        &mut data.form.exempt_dividends,
                        part.exempt,
                    );
                    crate::dividend_source::reduce_box(
                        &mut data.form.private_activity_dividends,
                        part.private_activity,
                    );
                    crate::dividend_source::reduce_box(
                        &mut data.form.foreign_tax,
                        part.foreign_tax,
                    );
                    crate::dividend_source::reduce_box(
                        &mut data.form.section_897_ordinary,
                        part.section_897_ordinary,
                    );
                    crate::dividend_source::reduce_box(
                        &mut data.form.section_897_capital,
                        part.section_897_capital,
                    );
                }
            }
            FederalIncome::Oid { data } if oid_rule(&crate::FederalOid::from(&*data)).is_none() => {
                if let Some(part) = data.nominee.take() {
                    nominee = [
                        part.oid,
                        part.other_periodic_interest,
                        part.market_discount,
                        part.acquisition_premium,
                        part.bond_premium,
                        part.exempt,
                        part.tax_exempt_bond_premium,
                        part.private_activity,
                        part.treasury_oid,
                    ]
                    .iter()
                    .any(|x| *x != 0);
                    data.form.original_issue_discount_for_the_year -= part.oid;
                    data.form.other_periodic_interest -= part.other_periodic_interest;
                    data.form.market_discount -= part.market_discount;
                    data.form.acquisition_premium -= part.acquisition_premium;
                    data.form.bond_premium -= part.bond_premium;
                    data.form.tax_exempt_oid -= part.exempt;
                    data.tax_exempt_bond_premium -= part.tax_exempt_bond_premium;
                    data.private_activity -= part.private_activity;
                    data.form.original_issue_discount_on_us_treasury_obligations -=
                        part.treasury_oid;
                }
                let a = data.adjustments.take().unwrap_or_default();
                if a.reporting == DebtReporting::Net {
                    data.form.acquisition_premium = 0;
                    data.form.bond_premium = 0;
                    data.tax_exempt_bond_premium = 0;
                    if election != Some(true)
                        && let Some(offset) = a.net_premium_offset
                    {
                        data.form.original_issue_discount_for_the_year += offset.ordinary;
                        data.form.original_issue_discount_on_us_treasury_obligations +=
                            offset.treasury;
                        premium_left_out = offset.ordinary + offset.treasury > 0;
                    }
                } else if election != Some(true) {
                    premium_left_out = data.form.bond_premium > 0;
                    data.form.bond_premium = 0;
                }
            }
            _ => {}
        }
        if premium_left_out {
            financial_omission = true;
            let message = crate::Message::new("WARN_DEBT_PREMIUM_ELECTION_LEFT_OUT", alloc::vec![]);
            notes.push(crate::federal_model::ReviewNote::scoped(
                #[cfg(test)]
                message.english(),
                message,
                context.clone(),
            ));
        }
        if nominee {
            let message = crate::Message::new("NOTE_INVESTMENT_NOMINEE", alloc::vec![]);
            let note = crate::federal_model::ReviewNote::scoped(
                #[cfg(test)]
                message.english(),
                message,
                context,
            );
            if !notes.iter().any(|old| old.same_notice(&note)) {
                notes.push(note);
            }
        }
    }
    financial_omission | crate::savings_bonds::normalize(&mut d.income, &bond_sources, notes)
}

pub(crate) fn annual_records(income: &FederalIncome) -> Option<&AnnualDebtRecords> {
    match income {
        FederalIncome::Interest { data } => data.adjustments(),
        FederalIncome::Oid { data } => data.adjustments.as_ref(),
        _ => None,
    }
    .and_then(|a| a.instrument_records.as_ref())
}
pub(crate) fn annual_ids(income: &FederalIncome) -> alloc::vec::Vec<i64> {
    annual_records(income)
        .map_or_else(alloc::vec::Vec::new, |a| a.records.iter().map(|r| r.id).collect())
}
pub(crate) fn annual_ids_mut(income: &mut FederalIncome) -> alloc::vec::Vec<&mut i64> {
    let adjustments = match income {
        FederalIncome::Interest { data } => data.adjustments_mut(),
        FederalIncome::Oid { data } => data.adjustments.as_mut(),
        _ => None,
    };
    adjustments
        .and_then(|a| a.instrument_records.as_mut())
        .map_or_else(alloc::vec::Vec::new, |a| a.records.iter_mut().map(|r| &mut r.id).collect())
}
// Normalize a calculation copy only. Keep instrument facts so every downstream
// owner uses this one per-instrument calculation, not a second premium engine.
fn apply_annual(
    income: &mut FederalIncome,
    election: Option<bool>,
) -> Option<(alloc::vec::Vec<&'static str>, bool)> {
    let oid_source = matches!(income, FederalIncome::Oid { .. });
    let valid = match income {
        FederalIncome::Interest { data } => {
            int_rule(&crate::FederalInterest::from(&*data)).is_none()
        }
        FederalIncome::Oid { data } => oid_rule(&crate::FederalOid::from(&*data)).is_none(),
        _ => false,
    };
    if !valid || annual_records(income).is_none() {
        return None;
    }
    let adjustments = match income {
        FederalIncome::Interest { data } => data.adjustments_mut(),
        FederalIncome::Oid { data } => data.adjustments.as_mut(),
        _ => None,
    }
    .expect("annual statement");
    let records = adjustments.instrument_records.as_mut().expect("annual records");
    let mut notices = alloc::vec::Vec::new();
    let mut financial_omission = false;
    for r in &mut records.records {
        if r.original_issue
            .as_ref()
            .is_some_and(|issue| issue.de_minimis() && !issue.elect_de_minimis_oid)
        {
            notices.push("NOTE_DEBT_DE_MINIMIS_OID");
        }
        let n = r.nominee.is_some();
        let a = r.after_death.is_some();
        r.amounts = r.retained();
        r.nominee = None;
        r.after_death = None;
        r.previously_reported = 0;
        if n {
            notices.push("NOTE_INVESTMENT_NOMINEE");
        }
        if a {
            notices.push("NOTE_DEBT_AFTER_DEATH");
        }
        if r.treatment == AnnualDebtTreatment::Contingent {
            financial_omission = true;
            notices.push("ISSUE_DEBT_CONTINGENT_INCOME");
        }
        if r.reporting == DebtReporting::Net
            && election != Some(true)
            && let Some(offset) = &r.net_premium_offset
        {
            if oid_source {
                r.amounts.oid += offset.ordinary;
            } else {
                r.amounts.taxable += offset.ordinary;
            }
            if oid_source {
                r.amounts.treasury_oid += offset.treasury;
            } else {
                r.amounts.us_obligations += offset.treasury;
            }
            if offset.ordinary + offset.treasury > 0 {
                financial_omission = true;
                notices.push("WARN_DEBT_PREMIUM_ELECTION_LEFT_OUT");
            }
            r.net_premium_offset = Some(DebtPremiumOffset::default());
        }
        let taxable_premium = r.amounts.bond_premium + r.amounts.treasury_bond_premium;
        let carry = r.premium_history.as_ref().map_or(0, |h| h.premium_carry);
        if r.reporting == DebtReporting::Gross
            && taxable_premium + carry > 0
            && election != Some(true)
        {
            r.amounts.bond_premium = 0;
            r.amounts.treasury_bond_premium = 0;
            r.premium_history = None;
            financial_omission = true;
            notices.push("WARN_DEBT_PREMIUM_ELECTION_LEFT_OUT");
        }
        let available = crate::kernel::ratio::positive(
            r.amounts.taxable
                + r.amounts.oid
                + r.amounts.other_periodic_interest
                + r.amounts.us_obligations
                + r.amounts.treasury_oid
                - r.amounts.acquisition_premium
                - r.amounts.treasury_acquisition_premium
                - r.amounts.accrued_interest
                - r.amounts.treasury_accrued_interest,
        );
        if r.reporting == DebtReporting::Gross
            && taxable_premium > available
            && r.premium_history.is_none()
        {
            financial_omission = true;
            notices.push("WARN_DEBT_PREMIUM_HISTORY_LEFT_OUT");
        }
        if r.reporting == DebtReporting::Net && carry > 0 {
            financial_omission = true;
            notices.push("WARN_DEBT_NET_CARRY_LEFT_OUT");
        }
        if r.disposed_on.is_some()
            && (taxable_premium > 0
                || r.premium_history.as_ref().is_some_and(|h| h.premium_carry > 0))
        {
            financial_omission = true;
            notices.push("WARN_DEBT_TERMINAL_PREMIUM_LEFT_OUT");
        }
        if let Some(value) = r.private_activity_after_premium {
            r.amounts.private_activity = value;
        }
    }
    let records = records.clone();
    match income {
        FederalIncome::Interest { data: crate::InterestSource::Int1099(data) } => {
            data.form.interest_income = records.records.iter().map(|r| r.amounts.taxable).sum();
            data.form.us_savings_bond_and_treasury_interest =
                records.records.iter().map(|r| r.amounts.us_obligations).sum();
            data.form.tax_exempt_interest = records.records.iter().map(|r| r.amounts.exempt).sum();
            data.form.specified_private_activity_bond_interest =
                records.records.iter().map(|r| r.amounts.private_activity).sum();
            data.form.foreign_tax_paid =
                records.records.iter().map(|r| r.amounts.foreign_tax).sum();
            data.form.market_discount =
                records.records.iter().map(|r| r.amounts.market_discount).sum();
            data.form.bond_premium = records.records.iter().map(|r| r.amounts.bond_premium).sum();
            data.form.bond_premium_on_treasury_obligations =
                records.records.iter().map(|r| r.amounts.treasury_bond_premium).sum();
            data.form.bond_premium_on_tax_exempt_bond =
                records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum();
            data.accrued_interest = records
                .records
                .iter()
                .map(|r| r.amounts.accrued_interest + r.amounts.treasury_accrued_interest)
                .sum();
        }
        FederalIncome::Oid { data } => {
            data.form.original_issue_discount_for_the_year =
                records.records.iter().map(|r| r.amounts.oid).sum();
            data.form.other_periodic_interest =
                records.records.iter().map(|r| r.amounts.other_periodic_interest).sum();
            data.form.original_issue_discount_on_us_treasury_obligations =
                records.records.iter().map(|r| r.amounts.treasury_oid).sum();
            data.form.tax_exempt_oid = records.records.iter().map(|r| r.amounts.exempt).sum();
            data.private_activity =
                records.records.iter().map(|r| r.amounts.private_activity).sum();
            data.form.market_discount =
                records.records.iter().map(|r| r.amounts.market_discount).sum();
            data.form.bond_premium = records
                .records
                .iter()
                .map(|r| r.amounts.bond_premium + r.amounts.treasury_bond_premium)
                .sum();
            data.form.acquisition_premium = records
                .records
                .iter()
                .map(|r| r.amounts.acquisition_premium + r.amounts.treasury_acquisition_premium)
                .sum();
            data.tax_exempt_bond_premium =
                records.records.iter().map(|r| r.amounts.tax_exempt_bond_premium).sum();
        }
        _ => unreachable!(),
    }
    notices.sort_unstable();
    notices.dedup();
    Some((notices, financial_omission))
}

// Both functions consume the central supported working copy. They never
// recompute ownership and expose real accepted child identities only.
pub(crate) fn computed_bond_premium(d: &crate::FederalDraft) -> i64 {
    d.income
        .iter()
        .filter_map(|e| annual_records(&e.income))
        .flat_map(|r| &r.records)
        .map(AnnualDebtRecord::excess_deduction)
        .sum()
}
pub(crate) fn bond_instruments(d: &crate::FederalDraft) -> alloc::vec::Vec<BondPremiumInstrument> {
    let mut notes = alloc::vec::Vec::new();
    let working = crate::unknowns::supported(d, &mut notes);
    let working = working.as_ref().unwrap_or(d);
    working
        .income
        .iter()
        .flat_map(|entry| {
            annual_records(&entry.income).into_iter().flat_map(move |rows| {
                rows.records.iter().filter(|r| r.amounts.exempt == 0).map(move |r| {
                    BondPremiumInstrument {
                        id: r.id,
                        label: crate::Message::new(
                            "BOND_PREMIUM_INSTRUMENT_LABEL",
                            alloc::vec![
                                crate::MessageArg::Text(entry.label.clone()),
                                crate::MessageArg::Text(r.instrument.clone())
                            ],
                        )
                        .encode(),
                        computed_deduction: r.excess_deduction(),
                    }
                })
            })
        })
        .collect()
}
pub(crate) fn manual_bond_premium_rule(
    d: &crate::FederalDraft,
    lines: &[crate::OtherItemizedLine],
) -> Option<&'static str> {
    manual_bond_premium_issue(d, lines).map(|issue| issue.key)
}

pub(crate) fn manual_bond_premium_issue(
    d: &crate::FederalDraft,
    lines: &[crate::OtherItemizedLine],
) -> Option<ValidationIssue> {
    let sources = bond_instruments(d);
    let mut seen = alloc::collections::BTreeSet::new();
    for (index, row) in lines.iter().enumerate() {
        let crate::OtherItemizedLine::BondPremium { source, .. } = row else {
            continue;
        };
        match source {
            BondPremiumSource::OtherDocumented { instrument } => {
                if instrument.trim().is_empty() {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_DESCRIPTION_NEEDED")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                }
            }
            BondPremiumSource::AcceptedAnnualInstrument { instrument } => {
                if !seen.insert(*instrument) {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_SOURCE_TWICE")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                }
                let Some(source) = sources.iter().find(|s| s.id == *instrument) else {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_SOURCE_MISSING")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                };
                if source.computed_deduction > 0 {
                    return Some(
                        ValidationIssue::new("BOND_PREMIUM_COMPUTED_SOURCE_TWICE")
                            .at_field("instrument")
                            .in_field("source")
                            .in_list("other_lines", index),
                    );
                }
            }
        }
    }
    None
}

pub(crate) fn unresolved_contingent(income: &FederalIncome) -> bool {
    annual_records(income).is_some_and(|rows| {
        rows.records.iter().any(|r| r.treatment == AnnualDebtTreatment::Contingent)
    })
}

// A net report does not establish the taxable gross amount when the election
// is absent or declined. Only an actual retained applied-offset fact can do so.
pub(crate) fn net_income_unresolved(income: &FederalIncome, election: Option<bool>) -> bool {
    if election == Some(true) {
        return false;
    }
    if let Some(rows) = annual_records(income) {
        return rows.records.iter().any(|r| {
            r.reporting == DebtReporting::Net
                && r.net_premium_offset.is_none()
                && (r.amounts.exempt == 0
                    || r.amounts.taxable
                        + r.amounts.us_obligations
                        + r.amounts.oid
                        + r.amounts.other_periodic_interest
                        + r.amounts.bond_premium
                        + r.amounts.treasury_bond_premium
                        != 0
                    || r.amounts.treasury_oid != 0)
        });
    }
    match income {
        FederalIncome::Interest { data } => {
            let data = &crate::FederalInterest::from(data);
            data.adjustments.as_ref().is_some_and(|a| {
                a.reporting == DebtReporting::Net
                    && a.net_premium_offset.is_none()
                    && (data.exempt == 0
                        || data.taxable
                            + data.us_obligations
                            + data.bond_premium
                            + data.treasury_bond_premium
                            > 0)
            })
        }
        FederalIncome::Oid { data } => {
            let data = &crate::FederalOid::from(data);
            data.adjustments.as_ref().is_some_and(|a| {
                a.reporting == DebtReporting::Net
                    && a.net_premium_offset.is_none()
                    && (data.exempt == 0
                        || data.oid + data.other_periodic_interest + data.bond_premium > 0
                        || data.treasury_oid != 0)
            })
        }
        _ => false,
    }
}

/// Interest received from financing the buyer of your home
/// Schedule B line 1: enter interest separately from repayment of principal.
/// Buyer facts are personal records, not a Form 1098 mortgage-expense import.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SellerMortgageInterest {
    /// Buyer name, taxpayer identification number and address
    pub buyer: SellerMortgageParty,
    /// Interest received in 2026, excluding principal
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
}

/// Interest on a private loan you made
/// Use your payment ledger and loan agreement. Enter taxable interest received in 2026 separately from repayment of principal, once per borrower. Do not repeat a 1099-INT or a seller-mortgage interest record. Below-market or deferred-payment loans may require taxable imputed interest or original issue discount: establish that amount from your tax records before entering it; the App does not derive it from principal or an interest rate. See Publication 550.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PrivateLoanInterest {
    /// Borrower name
    #[libertas_size(min = 1)]
    pub borrower: alloc::string::String,
    /// Taxable interest for 2026, excluding principal
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// The loan financed property the buyer used as a personal residence
    /// Schedule B lists this seller-financed interest first and needs the buyer's name, taxpayer identification number and address. Use the sale and loan records.
    /// [NullText]
    /// No seller financing of the buyer's personal residence
    /// [SetText]
    /// Yes; the buyer's taxpayer identification number and address are
    pub seller_mortgage: Option<PrivateLoanResidence>,
}

/// Buyer identification for a seller-financed personal residence
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PrivateLoanResidence {
    /// Buyer's Social Security number or taxpayer identification number
    /// Enter all nine digits, including leading zeroes, without punctuation.
    // Legal identifier: nine decimal digits in a U.S. taxpayer identification number.
    #[libertas_size(min = 9, max = 9)]
    #[libertas_pattern(r"[0-9]{9}")]
    pub buyer_tin: alloc::string::String,
    /// Buyer address
    #[libertas_size(min = 1)]
    pub buyer_address: alloc::string::String,
}
impl PrivateLoanInterest {
    pub(crate) fn buyer(&self) -> Option<SellerMortgageParty> {
        self.seller_mortgage.as_ref().map(|v| SellerMortgageParty {
            buyer: self.borrower.clone(),
            buyer_tin: v.buyer_tin.clone(),
            buyer_address: v.buyer_address.clone(),
        })
    }
}
