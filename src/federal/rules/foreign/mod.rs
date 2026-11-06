use super::*;

// Form 2555 lines 31 and 38: the qualifying days in 2026 of a bona fide
// residence (its dates clipped to the year) or of the 12-month physical
// presence period, which ends starts_on + 364 days later.
pub(crate) fn exclusion_days(
    q: &ForeignQualification,
) -> Result<i64, (&'static str, &'static str)> {
    match q {
        ForeignQualification::PriorYearOnly => Ok(0),
        // The 330-day test and the residence confirmation are answered before
        // the rules run (unknowns::foreign_claims leaves a failed or unconfirmed
        // claim out with its notice), so only an invalid stored period is
        // reported here.
        ForeignQualification::Residence { began, ended, .. } => {
            if !valid_date(*began) || ended.is_some_and(|e| !valid_date(e) || e <= *began) {
                return Err((
                    "ISSUE_FOREIGN_RESIDENCE_PERIOD_INVALID",
                    crate::fixture_english!(
                        "Enter valid dates for the bona fide residence, ending after it began."
                    ),
                ));
            }
            let (start, last) = period_ordinals(q);
            Ok(i64::from(max(0, min(last + 1, ordinal(20270101)) - start)))
        }
        ForeignQualification::Presence { starts_on, foreign_days } => {
            if !valid_birth(*starts_on) || *starts_on < 20250102 {
                return Err((
                    "ISSUE_FOREIGN_PRESENCE_PERIOD_INVALID",
                    crate::fixture_english!(
                        "Choose a valid 12-month presence period overlapping 2026."
                    ),
                ));
            }
            let start = ordinal(*starts_on);
            // Every period a 2026 return can name has 365 days: starts run
            // from 2025-01-02 to 2026-12-31 and none contains a February 29.
            let end = start + 365;
            if *foreign_days > end - start {
                return Err((
                    "ISSUE_FOREIGN_PRESENCE_PERIOD_INVALID",
                    crate::fixture_english!(
                        "Choose a valid 12-month presence period overlapping 2026."
                    ),
                ));
            }
            Ok(i64::from(max(0, min(end, ordinal(20270101)) - max(start, ordinal(20260101)))))
        }
    }
}

// The first and last day (ordinals) of a qualifying period, the first day
// clipped up to January 1, 2026: the bounds a housing stay must stay inside.
pub(crate) fn period_ordinals(q: &ForeignQualification) -> (i32, i32) {
    let year_start = ordinal(20260101);
    match q {
        ForeignQualification::PriorYearOnly => (ordinal(20260101), ordinal(20260101) - 1),
        ForeignQualification::Residence { began, ended, .. } => {
            (max(ordinal(*began), year_start), ended.map_or(ordinal(20261231), ordinal))
        }
        ForeignQualification::Presence { starts_on, .. } => {
            let start = ordinal(*starts_on);
            (max(start, year_start), start + 364)
        }
    }
}

#[derive(Default)]
pub(crate) struct ForeignResult {
    // Form 1040 line 1h: pay from a foreign employer with no W-2.
    pub wages: [i64; 2],
    // Form 2555 Part IV line 19: W-2 box 1 pay for work abroad plus the
    // foreign-employer pay. A W-2's box 1 is already in the wages total.
    pub earned_wages: [i64; 2],
    pub excluded: [i64; 2],
    pub gross_excluded: [i64; 2],
    pub prior_excluded: [i64; 2],
    pub housing_deduction: [i64; 2],
    pub housing_exclusion: [i64; 2],
    pub elected: [bool; 2],
}

impl ForeignResult {
    pub(crate) fn election(&self) -> bool {
        self.elected.iter().any(|elected| *elected)
    }
}

// The person's Form 2555 record, index 0 the taxpayer and 1 the spouse.
pub(crate) fn foreign_claim(d: &FederalDraft, owner: usize) -> Option<&crate::ForeignEarnedClaim> {
    d.screening
        .as_ref()?
        .foreign_earned
        .iter()
        .find(|claim| usize::from(claim.owner == Owner::Spouse) == owner)
}

pub(crate) fn foreign(d: &FederalDraft, r: &mut FederalResult) -> ForeignResult {
    foreign_for_owners(d, r, [true; 2])
}

// An upstream compensation consumer needs only the owners supplying that
// income. Shared housing still uses the actual household's complete claims.
pub(crate) fn foreign_for_owners(
    d: &FederalDraft,
    r: &mut FederalResult,
    owners: [bool; 2],
) -> ForeignResult {
    let mut result = ForeignResult::default();
    let mut abroad = [0; 2];
    for e in &d.income {
        let i = usize::from(e.owner == crate::DocumentOwner::Spouses);
        match &e.income {
            FederalIncome::ForeignEmployment { data: v } => result.wages[i] += v.wages,
            FederalIncome::Wage { data: v } => abroad[i] += crate::FederalWage::from(v).abroad,
            _ => {}
        }
    }
    let claims = [foreign_claim(d, 0), foreign_claim(d, 1)];
    // W-2 box 1 pay for work abroad is a Form 2555 fact: it is read only for a
    // person who has a Form 2555 record.
    for (i, (((pay, earned), abroad), claim)) in
        result.wages.iter_mut().zip(&mut result.earned_wages).zip(abroad).zip(claims).enumerate()
    {
        let gross = *pay + if claim.is_some() { abroad } else { 0 };
        *earned = if !owners[i] {
            0
        } else {
            claim.map_or_else(
                || dollars(gross),
                |c| {
                    if matches!(c.qualification, ForeignQualification::PriorYearOnly) {
                        0
                    } else {
                        c.employee_pay(gross).0
                    }
                },
            )
        };
        *pay = dollars(*pay);
    }
    if owners == [true; 2]
        || claims.iter().enumerate().any(|(i, claim)| {
            owners[i]
                && claim.is_some_and(|claim| {
                    matches!(claim.housing, crate::ForeignHousing::Claim { .. })
                })
        })
    {
        crate::foreign_housing::check_households(
            claims.map(|claim| claim.map(|c| &c.housing)),
            d.setup.as_ref().unwrap().filing_status() == FederalStatus::Separate,
            r,
        );
    }
    let mut housing = [None, None];
    let mut housing_excluded = [0; 2];
    for i in 0..2 {
        if !owners[i] {
            continue;
        }
        let Some(claim) = claims[i] else { continue };
        let barred = claim.revocation.barred();
        if claim.exclusion && barred[0] {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FOREIGN_ELECTION_BARRED"),
                crate::IssueContext::owner(crate::TaxSection::WorkAbroad, claim.owner),
            );
        }
        if matches!(claim.housing, crate::ForeignHousing::Claim { .. }) && barred[1] {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FOREIGN_HOUSING_ELECTION_BARRED"),
                crate::IssueContext::owner(crate::TaxSection::WorkAbroad, claim.owner),
            );
        }
        let context = crate::IssueContext::owner(crate::TaxSection::WorkAbroad, claim.owner);
        let business_funding = d.income.iter().any(|entry| usize::from(entry.owner == crate::DocumentOwner::Spouses) == i && matches!(&entry.income, FederalIncome::Business { data } if matches!(data.source, crate::BusinessSource::ForeignServices { .. })));
        result.elected[i] = (claim.exclusion && !barred[0])
            || (matches!(claim.housing, crate::ForeignHousing::Claim { .. })
                && (!barred[1] || business_funding));
        let wages = result.earned_wages[i];
        // A self-employed person's housing is figured with the business
        // receipts in foreign_business::exclude.
        if !business_funding && matches!(claim.housing, crate::ForeignHousing::Claim { .. }) {
            housing[i] = Some(&claim.housing);
            housing_excluded[i] =
                claim.housing.calculate(&claim.qualification, wages, wages, i == 1, r).exclusion;
            if barred[1] {
                housing_excluded[i] = 0;
            }
        }
        let receipts: i64 = d
            .income
            .iter()
            .filter(|entry| usize::from(entry.owner == crate::DocumentOwner::Spouses) == i)
            .filter_map(|entry| match &entry.income {
                FederalIncome::Business { data }
                    if matches!(data.source, crate::BusinessSource::ForeignServices { .. }) =>
                {
                    Some(data.receipts())
                }
                _ => None,
            })
            .sum();
        // Part IV line 26: foreign earned income.
        line(
            r,
            &alloc::format!("foreign_earned_{}_income", if i == 1 { "spouse" } else { "taxpayer" }),
            wages + dollars(receipts),
            &["screening.foreign_earned", "income.foreign_employment"],
            FOREIGN,
            msg!("EXPLAIN_FOREIGN_EARNED_N_N_INCOME"),
        );
        if claim.exclusion && !barred[0] {
            let gross: i64 = d
                .income
                .iter()
                .filter(|e| e.owner == crate::DocumentOwner::from(claim.owner))
                .filter_map(|e| match &e.income {
                    FederalIncome::Wage { data } => Some(crate::FederalWage::from(data).abroad),
                    FederalIncome::ForeignEmployment { data } => Some(data.wages),
                    _ => None,
                })
                .sum();
            result.prior_excluded[i] = claim.employee_pay(gross).1;
            if claim.prior_year_history.is_some() {
                line(
                    r,
                    &alloc::format!(
                        "foreign_prior_year_{}_exclusion",
                        if i == 1 { "spouse" } else { "taxpayer" }
                    ),
                    result.prior_excluded[i],
                    &[
                        "screening.foreign_earned.prior_year_pay",
                        "screening.foreign_earned.prior_year_history",
                        "screening.foreign_earned.multi_year_pay",
                    ],
                    FOREIGN,
                    msg!("EXPLAIN_FOREIGN_PRIOR_YEAR_EXCLUSION"),
                );
            }
            match exclusion_days(&claim.qualification) {
                Ok(days) => {
                    result.excluded[i] = phase::cap(
                        positive(wages - housing_excluded[i]),
                        [fraction(
                            crate::year_constants::FOREIGN_EARNED_INCOME_EXCLUSION_DOLLARS,
                            days,
                            365,
                        )],
                    );
                    let person = if i == 1 { "spouse" } else { "taxpayer" };
                    line(
                        r,
                        &alloc::format!("foreign_earned_{person}_days"),
                        days,
                        &["screening.foreign_earned"],
                        FOREIGN,
                        msg!("EXPLAIN_FOREIGN_EARNED_N_N_DAYS"),
                    );
                }
                Err((key, _message)) => {
                    #[cfg(test)]
                    let message = _message;
                    crate::issue_localized!(r, &context, "Foreign exclusion", key, message,)
                }
            }
        }
    }
    line(
        r,
        "foreign_wages",
        result.wages.iter().sum(),
        &["income.foreign_employment"],
        FOREIGN,
        msg!("EXPLAIN_FOREIGN_WAGES"),
    );
    result.gross_excluded = result.excluded;
    result.housing_exclusion = housing_excluded;
    for (i, amount) in housing_excluded.iter().enumerate() {
        result.excluded[i] += amount;
        if let Some(crate::ForeignHousing::Claim { prior: Some(prior), .. }) = housing[i] {
            // Section 1.911-4(e)(2): prior carry needs remaining foreign income,
            // not a current self-employment housing amount.
            result.housing_deduction[i] = crate::foreign_housing::deduction(
                0,
                result.earned_wages[i] - result.excluded[i],
                Some(prior),
                if i == 1 { "spouse" } else { "taxpayer" },
                r,
            );
        }
    }
    result
}

#[cfg(test)]
pub(crate) const FTC_NO_SOURCE: &str = "Form 1116 Part II: enter at least one foreign income source, a carryover history or a foreign loss account.";

pub(crate) use crate::form_1116::CreditBasis;

// Retained payee-document taxes in cents on the centrally supported working
// copy: Form1099-INT/DIV components and qualified estate/trust statements.
// The common consumer serves the statement election and the deduction.
pub(crate) fn payee_document_taxes(d: &FederalDraft) -> i64 {
    payee_document_tax_sources(d).iter().map(|(_, _, tax)| tax).sum()
}

// One native extraction owns both the joint total and actual payee sources.
// Values retain the centrally supported statement's actual cents.
fn payee_document_tax_sources(d: &FederalDraft) -> alloc::vec::Vec<(i64, Owner, i64)> {
    crate::community::entries(d)
        .iter()
        .map(|entry| {
            let tax = match &entry.income {
                FederalIncome::Interest { .. } | FederalIncome::Dividend { .. } => {
                    crate::investment_statements::components(entry).unwrap().foreign_tax
                }
                FederalIncome::EstateTrustPortfolio { data } => data
                    .statements
                    .foreign_statement
                    .as_ref()
                    .filter(|s| {
                        s.treatment == crate::EstateForeignTreatment::QualifiedPassivePayeeStatement
                            || (matches!(
                                s.treatment,
                                crate::EstateForeignTreatment::PassivePortfolio {
                                    basis: crate::ForeignTaxBasis::Paid,
                                    ..
                                }
                            ) && matches!(
                                d.credits.as_ref().map(|c| &c.foreign_tax),
                                Some(crate::ForeignTaxClaim::Deduction { .. })
                            ) && crate::estate_foreign::admitted(d, entry.id))
                    })
                    .map_or(0, |s| s.tax),
                _ => 0,
            };
            (entry.id, entry.owner.owned(), tax)
        })
        .collect()
}

// The section 904(j) election additionally uses its existing eligible K-3
// projection, alongside the statements typed on More credits.
pub(crate) fn payee_statement_taxes(d: &FederalDraft) -> i64 {
    let k3: i64 =
        crate::schedule_k3::credit(&crate::schedule_k3::attached(d), false, &[]).tax.iter().sum();
    payee_document_taxes(d) + k3
}

// Schedule A line 6, before rounding or the itemized-deduction choice. The
// independent K-3 traversal includes every category, not the 904(j) filter.
#[derive(Default)]
pub(crate) struct ForeignTaxDeduction {
    pub tax: i64,
    pub investment: i64,
    // Actual source cents for the same native investment-tax deduction.
    // Aggregate tax rows without a payee remain unavailable, never zero.
    pub investment_owners: Option<[i64; 2]>,
    pub investment_capacities: Option<crate::kernel::attribution::ExpenseSourceCents>,
    pub portfolio: i64,
    pub unallocated_tax: i64,
    pub unallocated_k3: alloc::vec::Vec<i64>,
    pub missing_k1: alloc::vec::Vec<i64>,
    pub accrued_sanctioned: bool,
    pub sanctioned: alloc::collections::BTreeMap<alloc::string::String, i64>,
    earned: [i64; 2],
    earned_countries: [alloc::vec::Vec<(alloc::string::String, i64)>; 2],
}

// Source-only tax projection shared with early royalty interest routing.
// No AGI-dependent tax, exclusion or basis result is inferred here.
pub(crate) fn direct_foreign_tax_deduction(d: &FederalDraft) -> ForeignTaxDeduction {
    let claim = &d.credits.as_ref().unwrap().foreign_tax;
    let chosen = matches!(claim, ForeignTaxClaim::Deduction { .. });
    let sanctioned = matches!(claim, ForeignTaxClaim::Ordinary { .. });
    let mut out = ForeignTaxDeduction {
        investment_owners: Some([0; 2]),
        investment_capacities: Some(crate::kernel::attribution::ExpenseSourceCents::known([0; 2])),
        ..Default::default()
    };
    if !chosen && !sanctioned {
        return out;
    }
    if chosen {
        for (_, owner, tax) in payee_document_tax_sources(d) {
            out.investment += tax;
            out.investment_owners.as_mut().unwrap()[usize::from(owner == Owner::Spouse)] += tax;
            out.investment_capacities.as_mut().unwrap().add(
                tax,
                Some(if owner == Owner::Spouse {
                    crate::DeductionOwners::Spouse
                } else {
                    crate::DeductionOwners::Taxpayer
                }),
            );
        }
        out.tax = out.investment;
        out.portfolio = out.investment;
        if let ForeignTaxClaim::Deduction { taxes } = claim {
            for row in taxes {
                match row.income {
                    crate::ForeignTaxIncome::Earned { owner } => {
                        out.earned[usize::from(owner == Owner::Spouse)] += row.tax
                    }
                    crate::ForeignTaxIncome::Investment => {
                        out.investment_capacities.as_mut().unwrap().add(row.tax, row.owners);
                        match (row.owners, &mut out.investment_owners) {
                            (Some(crate::DeductionOwners::Taxpayer), Some(owners)) => {
                                owners[0] += row.tax
                            }
                            (Some(crate::DeductionOwners::Spouse), Some(owners)) => {
                                owners[1] += row.tax
                            }
                            _ => out.investment_owners = None,
                        }
                        out.tax += row.tax;
                        out.investment += row.tax;
                        out.portfolio += row.tax;
                    }
                    crate::ForeignTaxIncome::Passive => {
                        out.investment_capacities.as_mut().unwrap().add(row.tax, row.owners);
                        match (row.owners, &mut out.investment_owners) {
                            (Some(crate::DeductionOwners::Taxpayer), Some(owners)) => {
                                owners[0] += row.tax
                            }
                            (Some(crate::DeductionOwners::Spouse), Some(owners)) => {
                                owners[1] += row.tax
                            }
                            _ => out.investment_owners = None,
                        }
                        out.tax += row.tax;
                        out.investment += row.tax;
                    }
                    crate::ForeignTaxIncome::Other => out.tax += row.tax,
                }
            }
        }
    } else if let ForeignTaxClaim::Ordinary { basis, sources, .. } = claim {
        // Section 901(j)(3) expressly removes the section 275 bar. Credit
        // source rows already hold the tax, so never add the 1099 boxes again.
        for row in sources
            .iter()
            .filter(|row| crate::year_constants::is_section_901j_country(&row.country))
        {
            // An accrual credit election does not establish cash payment for
            // the separate section 901(j)(3) deduction. Entity K-3 taxes below
            // retain the entity's paid/accrued treatment under 1.702-1(a)(6).
            if *basis == crate::ForeignTaxBasis::Accrued {
                out.accrued_sanctioned |= row.tax > 0;
                if matches!(
                    row.category,
                    crate::ForeignTaxCategory::Passive { .. }
                        | crate::ForeignTaxCategory::PriorPassiveTax { .. }
                ) {
                    out.investment_owners = None;
                    out.unallocated_tax += row.tax;
                    out.investment_capacities = None;
                }
                continue;
            }
            match row.category {
                crate::ForeignTaxCategory::Employment
                | crate::ForeignTaxCategory::Business { .. } => {
                    let owner = usize::from(row.owner == Owner::Spouse);
                    out.earned[owner] += row.tax;
                    out.earned_countries[owner].push((row.country.to_ascii_uppercase(), row.tax));
                }
                crate::ForeignTaxCategory::Passive { .. }
                | crate::ForeignTaxCategory::PriorPassiveTax { .. } => {
                    out.tax += row.tax;
                    out.investment += row.tax;
                    out.portfolio += row.tax;
                    if let Some(owners) = &mut out.investment_owners {
                        owners[usize::from(row.owner == Owner::Spouse)] += row.tax;
                    }
                    if let Some(capacities) = &mut out.investment_capacities {
                        capacities.add(
                            row.tax,
                            Some(if row.owner == Owner::Spouse {
                                crate::DeductionOwners::Spouse
                            } else {
                                crate::DeductionOwners::Taxpayer
                            }),
                        );
                    }
                    *out.sanctioned.entry(row.country.to_ascii_uppercase()).or_default() += row.tax;
                }
                crate::ForeignTaxCategory::PriorEmployeeWageTax { .. } => {
                    out.tax += row.tax;
                    *out.sanctioned.entry(row.country.to_ascii_uppercase()).or_default() += row.tax;
                }
            }
        }
    }
    out
}

pub(crate) fn foreign_tax_deduction(
    d: &FederalDraft,
    foreign: &ForeignResult,
    businesses: &[crate::foreign_business::BusinessExclusion; 2],
    limits: &[crate::schedule_k3::K3Limits],
    pool: &RentalResult,
) -> ForeignTaxDeduction {
    let claim = &d.credits.as_ref().unwrap().foreign_tax;
    let chosen = matches!(claim, ForeignTaxClaim::Deduction { .. });
    let sanctioned = matches!(claim, ForeignTaxClaim::Ordinary { .. });
    let mut out = direct_foreign_tax_deduction(d);
    if !chosen && !sanctioned {
        return out;
    }
    for (owner, business) in businesses.iter().enumerate() {
        let income = foreign.earned_wages[owner] + positive(business.earned_net)
            - foreign.housing_deduction[owner]
            - business.housing_deduction;
        let retained = out.earned[owner]
            - crate::form_1116::excluded_earned_tax(
                out.earned[owner],
                foreign.excluded[owner],
                income,
            );
        out.tax += retained;
        let weights: alloc::vec::Vec<i64> =
            out.earned_countries[owner].iter().map(|(_, tax)| *tax).collect();
        for ((country, _), share) in out.earned_countries[owner]
            .iter()
            .zip(crate::kernel::ratio::allocate_prefix(retained, &weights))
        {
            *out.sanctioned.entry(country.clone()).or_default() += share;
        }
    }
    let mut position = 0;
    for entry in &crate::community::entries(d) {
        let k3 = match &entry.income {
            FederalIncome::Partnership { data } => data.schedule_k3.clone(),
            FederalIncome::Scorp { data } => crate::ScorpK1::from(data).schedule_k3,
            _ => None,
        };
        let allowed =
            pool.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map_or(0, |(_, ledger)| {
                ledger.basis_allowed[crate::k1_loss::K1LossClass::ForeignTaxes].get()
            });
        let Some(k3) = k3 else {
            if chosen && allowed > 0 {
                out.investment_owners = None;
                out.missing_k1.push(entry.id);
                out.investment_capacities = None;
                out.unallocated_tax += allowed;
            }
            continue;
        };
        let limit = limits.get(position).copied().unwrap_or(crate::schedule_k3::K3Limits::FULL);
        position += 1;
        let tax: i64 = k3
            .countries
            .iter()
            .filter(|row| chosen || crate::year_constants::is_section_901j_country(&row.country))
            .map(|row| fraction(row.tax, limit.tax.0, limit.tax.1))
            .sum();
        out.tax += tax;
        if sanctioned {
            for row in k3
                .countries
                .iter()
                .filter(|row| crate::year_constants::is_section_901j_country(&row.country))
            {
                *out.sanctioned.entry(row.country.to_ascii_uppercase()).or_default() +=
                    fraction(row.tax, limit.tax.0, limit.tax.1);
            }
        }
        // Released basis carries have no row in this year's K-3. Likewise a
        // K-1 tax not covered by the K-3 has no source character to invent.
        let all_current: i64 =
            k3.countries.iter().map(|row| fraction(row.tax, limit.tax.0, limit.tax.1)).sum();
        // The K-1 ledger owns whole-dollar allowances, while K-3 rows retain
        // cents. Compare in that owner's units; rounding a complete 99-cent
        // source to a dollar must not invent a missing one-cent carry.
        let missing = positive(allowed - dollars(all_current) * 100);
        if chosen && missing > 0 {
            out.investment_owners = None;
            out.missing_k1.push(entry.id);
            out.investment_capacities = None;
            out.unallocated_tax += missing;
        }
        // A Form 1116 category does not establish section 1411 activity
        // attribution. Keep the Schedule A deduction; leave only the uncertain
        // NIIT benefit out with a source-specific notice when it can matter.
        if tax > 0 {
            out.investment_owners = None;
            out.unallocated_k3.push(entry.id);
            out.investment_capacities = None;
            out.unallocated_tax += tax;
        }
    }
    out
}

pub(crate) fn foreign_credit(
    d: &FederalDraft,
    basis: CreditBasis,
    r: &mut FederalResult,
) -> (i64, i64, crate::injured_spouse::income_categories::NativeForeignCreditSources) {
    use crate::injured_spouse::income_categories::{
        NativeForeignCreditCapacity, NativeForeignCreditSources,
    };
    match &d.credits.as_ref().unwrap().foreign_tax {
        ForeignTaxClaim::No | ForeignTaxClaim::Deduction { .. } => (
            0,
            0,
            NativeForeignCreditSources {
                joint: 0,
                owners: Some([0; 2]),
                capacity: Some(NativeForeignCreditCapacity::PayeeCents([0; 2])),
            },
        ),
        ForeignTaxClaim::Statements { eligible } => {
            // Payee-statement taxes: the 1099 boxes on the Interest and
            // dividends entries and the Schedule K-3 taxes
            // (a K-3 with only passive rows is a payee statement). A K-1 tax
            // without a K-3 counts toward the $300/$600 test but is never
            // credited (its income category is unknown).
            let k3_entries = crate::schedule_k3::attached_sources(d);
            // One actual source traversal supplies both the election total
            // and payees. K-3 context stays bound to its own native rows.
            let mut document_sources = payee_document_tax_sources(d);
            let document_tax: i64 = document_sources.iter().map(|(_, _, tax)| tax).sum();
            let k3 = crate::schedule_k3::owned_credit(&k3_entries, false, &[]);
            let k3_tax: i64 = k3.tax.iter().sum();
            document_sources.extend(k3.rows.iter().filter_map(|row| {
                let (id, owner) = row.source?;
                Some((id, owner, row.tax - row.reduction))
            }));
            let documents = document_tax + k3_tax;
            let taxes = documents + crate::schedule_k3::k1_taxes_without_k3(d);
            let limit = if d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint {
                crate::year_constants::FOREIGN_DE_MINIMIS_JOINT_DOLLARS * 100
            } else {
                crate::year_constants::FOREIGN_DE_MINIMIS_DOLLARS * 100
            };
            crate::require_localized!(
                r,
                *eligible == Some(true)
                    && taxes <= limit
                    && (documents > 0 || !k3_entries.is_empty())
                    && k3_entries.iter().all(|k3| k3.schedule.passive_only())
                    && (taxes == 0
                        || basis.portfolio > 0
                        || k3_entries.iter().any(|k3| k3.schedule.taxes() > 0))
                    && !d.income.iter().any(|e| match &e.income {
                        FederalIncome::ForeignEmployment { .. } => true,
                        FederalIncome::Wage { data } => crate::FederalWage::from(data).abroad > 0,
                        _ => false,
                    }),
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign tax credit",
                "ISSUE_THE_PAYEE_STATEMENT_ELECTION_REQUIRES_ONLY_STATEMENT_REPORTED_PASSI_61D2975F",
                "The payee-statement election requires only statement-reported passive foreign income and taxes at or below $300 ($600 jointly); other claims require Form 1116.",
            );
            let credit = d
                .credits
                .as_ref()
                .unwrap()
                .foreign_tax
                .statement_credit(basis.regular, documents)
                .expect("payee-statement credit branch");
            let source_cents = document_sources
                .iter()
                .try_fold([0_i64; 2], |mut sums, (_, owner, tax)| {
                    if *tax == 0 {
                        return Some(sums);
                    }
                    if *tax < 0 {
                        return None;
                    }
                    let index = match owner {
                        Owner::Taxpayer => 0,
                        Owner::Spouse => 1,
                    };
                    sums[index] = sums[index].checked_add(*tax)?;
                    Some(sums)
                })
                .filter(|owners| owners[0].checked_add(owners[1]) == Some(documents));
            let owners = if credit == 0 {
                Some([0; 2])
            } else {
                source_cents.and_then(|cents| match cents {
                    [first, 0] if first > 0 => Some([credit, 0]),
                    [0, second] if second > 0 => Some([0, credit]),
                    _ => {
                        // Only a fully allowed source pool has a conserving
                        // rounded-pair projection. A binding joint cap cannot
                        // invent post-limit owners from gross source ratios.
                        let first = usize::from(
                            d.injured_spouse
                                .as_ref()
                                .is_some_and(|answer| answer.injured == Owner::Spouse),
                        );
                        crate::kernel::ratio::project_rounded_pair(credit, cents, first)
                    }
                })
            };
            (
                credit,
                credit,
                NativeForeignCreditSources {
                    joint: credit,
                    owners,
                    capacity: source_cents.map(NativeForeignCreditCapacity::PayeeCents),
                },
            )
        }
        ForeignTaxClaim::Ordinary {
            basis: tax_basis,
            sources,
            situations: _,
            history,
            carryback,
            amt_limitation,
            interest_allocation,
            investment_assets,
            loss_history,
            state_taxes,
            rate_adjustment,
        } => crate::form_1116::ordinary(
            d,
            &crate::form_1116::Claim {
                basis: tax_basis,
                sources,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets,
                loss_history,
                state_taxes,
                rate_adjustment,
            },
            &basis,
            r,
        ),
    }
}

pub(crate) fn valid_credit(claim: &ForeignTaxClaim) -> bool {
    let amount = |v: i64| (0..=100_000_000_000).contains(&v);
    match claim {
        ForeignTaxClaim::No => true,
        ForeignTaxClaim::Deduction { taxes } => {
            taxes.len() <= 50
                && taxes.iter().all(|row| {
                    !row.label.trim().is_empty() && (1..=100_000_000_000).contains(&row.tax)
                })
        }
        ForeignTaxClaim::Statements { .. } => true,
        ForeignTaxClaim::Ordinary {
            sources,
            amt_limitation,
            history,
            carryback,
            state_taxes,
            loss_history,
            investment_assets,
            ..
        } => {
            history.valid()
                && state_taxes.valid()
                && loss_history.valid()
                && investment_assets.valid()
                && carryback.valid()
                && amt_limitation.valid()
                && sources.len() <= 100
                && sources.iter().all(|s| {
                    !s.label.trim().is_empty()
                        && crate::form_1116::country_size_valid(&s.country)
                        && amount(s.income)
                        && amount(s.tax)
                        && amount(s.qualified())
                        && s.qualified() <= s.income
                        && match s.category {
                            ForeignTaxCategory::PriorEmployeeWageTax { income_year, .. }
                            | ForeignTaxCategory::PriorPassiveTax { income_year, .. } => {
                                (1900..=2025).contains(&income_year) && s.income == 0 && s.tax > 0
                            }
                            _ => true,
                        }
                })
        }
    }
}
