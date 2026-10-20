// Coverage: Pub. 517 clergy and religious workers — Unit41 R6–R14.
//   ClergyRecord owns minister/religious-order/practitioner status, approved
//   Form4361 and Form4029; MinisterHousing owns actual parsonage/allowance facts.
//   W2Coverage separates ordinary, church employee and ministerial wages;
//   BusinessDetails marks ministerial fees, notary net and optional actual gross.
//   IRC107 housing exclusion/excess and Pub517 Worksheets1–3 expense allocation
//   feed ordinary income, QBI and business deductions; the full housing/employee
//   expense amounts separately feed SE. Religious exemptions retain ordinary
//   income and remove only their statutory SE/credit source portions. Totalization
//   removes SE tax; retained EIC/ACTC treatment follows the approved statutory
//   reading of1401(c),1402(a),32(c)(2),24(d), with the broad IRS worksheet wording
//   explicitly qualified in the private source audit. Form2441 excludes exempt
//   ministerial SE/housing and retains ordinary taxable wages.
//   Tests: tests/income/abroad_clergy.rs F1/F4/F5 and independent exemption, credit,
//   notary/totalization overlap, page/save/restart and owner-isolation controls;
//   tests/returns.rs minister-parsonage-and-fees external fixture.
//   Outside this record: retired-minister pension housing designations and
//   refunds of earlier SE tax. Vow-of-poverty order income belongs to the order
//   and is not entered; services outside the order use their ordinary source.
use crate::kernel::lines::{LineValue, emit, row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use crate::{BusinessActivity, FederalDraft, FederalIncome, FederalResult, Owner};
use alloc::{collections::BTreeMap, format};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Minister, religious worker or Form 4029 exemption
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ClergyRecord {
    /// Your status
    pub role: ClergyRole,
}

/// Status for self-employment tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ClergyRole {
    /// Minister of a church (ordained, commissioned or licensed), including a cantor or Christian Science reader
    /// Your pay for ministerial services, including as the church's employee, is self-employment income (Schedule SE line 2) unless your Form 4361 exemption is approved.
    Minister {
        /// Housing the church provides or designates
        #[libertas_default(None)]
        housing: MinisterHousing,
        /// Your Form 4361 exemption from self-employment tax is approved
        /// The IRS returned your Form 4361 marked approved; it is irrevocable and covers only ministerial earnings. An application still pending or denied: No (the tax is due).
        form_4361: bool,
        /// Unreimbursed expenses of your ministerial work as a church employee
        /// Not deductible for income tax; subtracted from your self-employment earnings. Leave zero if none or if your Form 4361 is approved.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        employee_expenses: i64,
    },
    /// Member of a religious order who has not taken a vow of poverty
    ReligiousOrderMember {
        /// Your Form 4361 exemption from self-employment tax is approved
        form_4361: bool,
        /// Unreimbursed expenses of your work for the order as its employee
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        employee_expenses: i64,
    },
    /// Christian Science practitioner
    ChristianSciencePractitioner {
        /// Your Form 4361 exemption from self-employment tax is approved
        form_4361: bool,
        /// Unreimbursed employee expenses of your practice
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        employee_expenses: i64,
    },
    /// Member of a recognized religious sect with an approved Form 4029
    /// No self-employment tax on any of your self-employment income. Wages are free of Social Security and Medicare tax only when the employer also holds an approved Form 4029 (its W-2 shows "Form 4029" in box 14 and no boxes 3 to 6).
    ReligiousSect,
}

/// Housing from the church
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MinisterHousing {
    /// None
    None,
    /// A parsonage the church provides
    Parsonage {
        /// Fair rental value of the parsonage for 2026, furnished
        /// What it would cost to rent a comparable furnished home in a similar location. Excluded from income; added to your self-employment earnings.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        fair_rental_value: i64,
        /// Allowance designated for utilities
        /// The amount the church designated in advance for utility costs; W-2 box 1 leaves it out. Zero if none.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        utility_allowance: i64,
        /// Utilities you actually paid
        /// Any allowance above this is taxable (Form 1040 line 1h "Excess allowance").
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        utilities_paid: i64,
    },
    /// A housing or rental allowance the church designated
    Allowance {
        /// Amount officially designated in advance as a housing or rental allowance for 2026, including any utility part
        /// From the church's resolution, budget or your contract; W-2 box 1 leaves it out.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        designated: i64,
        /// Amount you used in 2026 to rent or provide your home, including utilities
        /// Rent or mortgage payments, repairs, furnishings, utilities and similar costs of the home.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        spent: i64,
        /// Fair rental value of the home for 2026, furnished, plus the cost of utilities
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        fair_rental_value: i64,
    },
}

/// Clergy record for one person on this return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ClergyClaim {
    /// Person whose ministerial services or approved religious exemption this records
    pub owner: Owner,
    /// Ministerial status and its applicable housing and exemption facts
    pub role: ClergyRole,
}

impl ClergyRole {
    fn housing_amounts(&self) -> [i64; 3] {
        match self {
            Self::Minister { housing, .. } => housing.amounts(),
            _ => [0; 3],
        }
    }

    pub(crate) fn valid(&self) -> bool {
        let amount = |v: i64| (0..=100000000000).contains(&v);
        amount(self.employee_expenses())
            && match self {
                Self::Minister {
                    housing:
                        MinisterHousing::Parsonage {
                            fair_rental_value,
                            utility_allowance,
                            utilities_paid,
                        },
                    ..
                } => [*fair_rental_value, *utility_allowance, *utilities_paid]
                    .into_iter()
                    .all(amount),
                Self::Minister {
                    housing: MinisterHousing::Allowance { designated, spent, fair_rental_value },
                    ..
                } => [*designated, *spent, *fair_rental_value].into_iter().all(amount),
                _ => true,
            }
    }
    pub(crate) fn exempt(&self) -> bool {
        match self {
            Self::Minister { form_4361, .. }
            | Self::ReligiousOrderMember { form_4361, .. }
            | Self::ChristianSciencePractitioner { form_4361, .. } => *form_4361,
            Self::ReligiousSect => false,
        }
    }
    fn employee_expenses(&self) -> i64 {
        match self {
            Self::Minister { employee_expenses, .. }
            | Self::ReligiousOrderMember { employee_expenses, .. }
            | Self::ChristianSciencePractitioner { employee_expenses, .. } => *employee_expenses,
            Self::ReligiousSect => 0,
        }
    }
}
impl MinisterHousing {
    // All source and intermediate amounts are cents. Each existing downstream
    // form applies its own whole-dollar boundary, never an extra rounding here.
    fn amounts(&self) -> [i64; 3] {
        match *self {
            Self::None => [0; 3],
            Self::Parsonage { fair_rental_value, utility_allowance, utilities_paid } => {
                let utilities = phase::cap(utility_allowance, [utilities_paid]);
                [
                    fair_rental_value + utilities,
                    utility_allowance - utilities,
                    fair_rental_value + utility_allowance,
                ]
            }
            Self::Allowance { designated, spent, fair_rental_value } => {
                let excluded = phase::cap(designated, [spent, fair_rental_value]);
                [excluded, designated - excluded, designated]
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct ClergyResult {
    pub se_line_2: i64,
    pub housing_tax_free: i64,
    pub housing_excess: i64,
    pub nondeductible_by_business: BTreeMap<i64, i64>,
    pub exempt: bool,
    pub sect: bool,
    pub employee_expenses: i64,
    pub wages: i64,
    pub business_profit: i64,
    tax_free_total: i64,
    record_present: bool,
}

pub(crate) fn business(data: &crate::FederalBusiness) -> bool {
    data.ministerial && matches!(data.activity, BusinessActivity::Ordinary { .. })
}

#[cfg(test)]
pub(crate) fn calculate(d: &FederalDraft, owner: Owner) -> ClergyResult {
    match calculate_with_section179(d, owner, None, None) {
        Ok(value) => value,
        Err(_) => unreachable!("legacy clergy calculation has no selected allocation"),
    }
}

pub(crate) fn calculate_with_section179(
    d: &FederalDraft,
    owner: Owner,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<ClergyResult, crate::Section179PreparedIssue> {
    let role = d
        .screening
        .as_ref()
        .and_then(|s| s.clergy.iter().find(|c| c.owner == owner))
        .map(|c| &c.role);
    let mut out = ClergyResult { record_present: role.is_some(), ..Default::default() };
    if let Some(role) = role {
        out.exempt = role.exempt();
        out.sect = matches!(role, ClergyRole::ReligiousSect);
        out.employee_expenses = role.employee_expenses();
    }
    let housing = role.map_or([0; 3], ClergyRole::housing_amounts);
    out.housing_tax_free = housing[0];
    out.housing_excess = housing[1];
    out.wages = d
        .income
        .iter()
        .filter(|e| e.owner == owner)
        .filter_map(|e| match &e.income {
            FederalIncome::Wage { data } if data.coverage == crate::W2Coverage::Ministerial => {
                Some(data.wages)
            }
            _ => None,
        })
        .sum();
    let businesses: alloc::vec::Vec<_> = d
        .income
        .iter()
        .filter(|e| e.owner == owner)
        .filter_map(|e| match &e.income {
            FederalIncome::Business { data } if business(data) => Some((e.id, data)),
            _ => None,
        })
        .collect();
    out.tax_free_total = out.wages
        + businesses.iter().map(|(_, data)| data.receipts()).sum::<i64>()
        + housing[0]
        + housing[1];
    for (id, data) in businesses {
        let expenses = crate::section179_return::expenses(d, id, data, selected, home)?;
        out.business_profit += data.receipts() - expenses
            + selected.map_or(0, |allocation| allocation.ordinary_risk_adjustment(id));
        let disallowed = if out.tax_free_total > 0 {
            fraction(expenses, housing[0], out.tax_free_total)
        } else {
            0
        };
        out.nondeductible_by_business.insert(id, disallowed);
    }
    out.se_line_2 = out.wages + housing[2] + out.business_profit - out.employee_expenses;
    Ok(out)
}

impl ClergyResult {
    pub(crate) fn nonbusiness_gross(&self) -> i64 {
        if self.exempt || self.sect {
            0
        } else {
            self.wages + self.housing_tax_free + self.housing_excess
        }
    }
    pub(crate) fn extra_se(&self) -> i64 {
        if self.exempt || self.sect { 0 } else { self.se_line_2 - self.business_profit }
    }
}
// Only the new source exclusions; CRP/statutory wages retain their existing owners.

pub(crate) fn business_se_exclusion_with_section179(
    d: &FederalDraft,
    id: i64,
    data: &crate::FederalBusiness,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let Some(entry) = d.income.iter().find(|e| e.id == id) else {
        return match selected {
            Some(_) => Err(crate::Section179PreparedIssue::UnknownSource(id)),
            None => Ok(0),
        };
    };
    if let Some(allocation) = selected {
        // Admit the actual direct source through its canonical selected owner.
        allocation.allowed(id)?;
    }
    let facts = calculate_with_section179(d, entry.owner, selected, home)?;
    let profit = data.earned_receipts()
        + crate::federal_rules::farm_routed_income(
            d,
            usize::from(entry.owner == Owner::Spouse),
            id,
        )?
        - crate::section179_return::expenses(d, id, data, selected, home)?
        + selected.map_or(0, |allocation| allocation.ordinary_risk_adjustment(id));
    Ok(
        if facts.sect
            || (facts.exempt && business(data))
            || matches!(
                data.source,
                crate::BusinessSource::ForeignServices { totalization_exempt: true, .. }
            )
        {
            profit
        } else {
            data.notary_profit
        },
    )
}

pub(crate) fn earned_adjustments_with_section179(
    d: &FederalDraft,
    owner: Owner,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<(i64, i64), crate::Section179PreparedIssue> {
    let mut compensation = 0;
    let mut totalization = 0;
    for e in d.income.iter().filter(|e| e.owner == owner) {
        if let FederalIncome::Business { data } = &e.income {
            let excluded = business_se_exclusion_with_section179(d, e.id, data, selected, home)?;
            compensation += excluded + nondeductible_with_section179(d, e.id, selected, home)?;
            if matches!(
                data.source,
                crate::BusinessSource::ForeignServices { totalization_exempt: true, .. }
            ) {
                // Totalization cannot restore a notary or approved religious
                // service exemption into a credit's earned-income base.
                if !(calculate_with_section179(d, owner, selected, home)?.sect
                    || calculate_with_section179(d, owner, selected, home)?.exempt
                        && business(data))
                {
                    totalization += excluded - data.notary_profit;
                }
            }
        }
    }
    Ok((compensation, totalization))
}

// Before179 cannot call calculate: its expense allocation reads section179::expenses.
// Zero is certified only by the same owner's absent tax-free housing numerator.
pub(crate) fn before179_nondeductible(
    d: &FederalDraft,
    owner: Owner,
    data: &crate::FederalBusiness,
) -> Result<i64, &'static str> {
    let excluded = d
        .screening
        .as_ref()
        .and_then(|screening| screening.clergy.iter().find(|claim| claim.owner == owner))
        .map_or(0, |claim| claim.role.housing_amounts()[0]);
    if business(data) && excluded != 0 { Err("CLERGY_BEFORE179_EXPENSE_SOURCE") } else { Ok(0) }
}

// The single tax-free expense-allocation owner; callers keep their actual tax,
// SE, QBI or health responsibility and never substitute a second ratio.

pub(crate) fn nondeductible_with_section179(
    d: &FederalDraft,
    id: i64,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let Some(entry) = d.income.iter().find(|e| e.id == id) else {
        return match selected {
            Some(_) => Err(crate::Section179PreparedIssue::UnknownSource(id)),
            None => Ok(0),
        };
    };
    if let Some(allocation) = selected {
        // Admit the actual direct source through its canonical selected owner.
        allocation.allowed(id)?;
    }
    Ok(calculate_with_section179(d, entry.owner, selected, home)?
        .nondeductible_by_business
        .get(&id)
        .copied()
        .unwrap_or(0))
}

pub(crate) fn lines(r: &mut FederalResult, owner: Owner, facts: &ClergyResult) {
    if !facts.record_present && facts.wages != 0 {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("WARN_CLERGY_RECORD_MISSING", alloc::vec::Vec::new()),
            crate::IssueContext::owner(crate::TaxSection::Wages, owner),
        );
    }
    if facts.wages == 0
        && facts.business_profit == 0
        && facts.housing_tax_free == 0
        && facts.housing_excess == 0
        && !facts.exempt
        && !facts.sect
    {
        return;
    }
    let person = if owner == Owner::Spouse { "spouse" } else { "taxpayer" };
    let percent = if facts.tax_free_total > 0 {
        fraction(facts.housing_tax_free, 10000, facts.tax_free_total)
    } else {
        0
    };
    emit(
        r,
        &mut (),
        [
            (
                "housing_excluded",
                None,
                LineValue::MoneyDollars(dollars(facts.housing_tax_free)),
                "EXPLAIN_CLERGY_HOUSING_EXCLUDED",
            ),
            (
                "housing_excess",
                Some(crate::FormLine::new(crate::TaxForm::Form1040, "1h")),
                LineValue::MoneyDollars(dollars(facts.housing_excess)),
                "EXPLAIN_CLERGY_HOUSING_EXCESS",
            ),
            (
                "tax_free_percent",
                None,
                LineValue::RatioTenThousandths(percent),
                "EXPLAIN_CLERGY_TAX_FREE_PERCENT",
            ),
            (
                "se_earnings",
                (!facts.exempt && !facts.sect)
                    .then(|| crate::FormLine::new(crate::TaxForm::ScheduleSe, "2")),
                LineValue::MoneyDollars(dollars(facts.se_line_2)),
                "EXPLAIN_CLERGY_SE_EARNINGS",
            ),
            (
                "nondeductible_expenses",
                None,
                LineValue::MoneyDollars(dollars(facts.nondeductible_by_business.values().sum())),
                "EXPLAIN_CLERGY_NONDEDUCTIBLE_EXPENSES",
            ),
        ],
        |(suffix, form, value, key), _, _| {
            Some(row(
                form,
                &format!("clergy_{person}_{suffix}"),
                value,
                &["screening.clergy", "income.wages.coverage", "income.business.ministerial"],
                "https://www.irs.gov/publications/p517",
                crate::Message::new(key, alloc::vec::Vec::new()),
            ))
        },
    );
}

// Print metadata is carried by ordinary explanation arguments; printed-form
// production remains with PRINT. The tax and the saved source facts stay here.
#[derive(Default)]
pub(crate) struct SeExemption {
    pub line_a: bool,
    pub box_index: i64,
    pub notary_profit: i64,
    pub totalization: bool,
}
impl SeExemption {
    pub(crate) fn any(&self) -> bool {
        self.line_a || self.box_index != 0 || self.notary_profit != 0 || self.totalization
    }
    pub(crate) fn code(&self) -> &'static str {
        match (self.notary_profit != 0, self.totalization) {
            (true, true) => "EN; EAS",
            (true, false) => "EN",
            (false, true) => "EAS",
            _ => "",
        }
    }
}

pub(crate) fn se_exemption_with_section179(
    d: &FederalDraft,
    owner: Owner,
    net: i64,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<SeExemption, crate::Section179PreparedIssue> {
    let clergy = calculate_with_section179(d, owner, selected, home)?;
    let mut out = SeExemption::default();
    for e in d.income.iter().filter(|e| e.owner == owner) {
        if let FederalIncome::Business { data } = &e.income {
            out.notary_profit += data.notary_profit;
            out.totalization |= matches!(
                data.source,
                crate::BusinessSource::ForeignServices { totalization_exempt: true, .. }
            );
        }
    }
    if clergy.sect {
        out.box_index = 2;
    } else if net < 400 {
        out.box_index = if clergy.exempt {
            1
        } else if out.notary_profit != 0 || out.totalization {
            3
        } else {
            0
        };
    } else {
        out.line_a = clergy.exempt;
    }
    Ok(out)
}
