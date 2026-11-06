//! Employee dependent-care compensation in the before-section179 income view.
//! Reg.1.179-2(c)(1), IRC129(b)/(e)(2) and IRC32(c)(2): the earned-income
//! exclusion is distinct from the taxable-income ordering in Reg.1.179-2(c)(5).
//! https://www.ecfr.gov/current/title-26/section-1.179-2
use super::{Section179PreparedAllocation, Section179PreparedIssue as Issue};
use crate::{FederalDraft, FederalIncome, FederalResult};
use alloc::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CareCompensation {
    pub taxable_dollars: [i64; 2],
}
impl CareCompensation {
    pub(super) fn cents(self) -> Result<i64, Issue> {
        crate::kernel::ratio::checked_sum(self.taxable_dollars)
            .and_then(|amount| amount.checked_mul(100))
            .ok_or(Issue::AggregateAmountOverflow)
    }
}

pub(crate) fn needed(d: &FederalDraft) -> Result<bool, Issue> {
    let w2_benefits = d
        .income
        .iter()
        .any(|e| matches!(&e.income, FederalIncome::Wage { data } if crate::FederalWage::from(data).care_benefits > 0));
    match &d.credits {
        Some(credits) => Ok(credits.care.has_benefits(d)),
        None if w2_benefits => Err(Issue::MissingReturnContext),
        None => Ok(false),
    }
}

fn boundary(id: i64) -> Issue {
    Issue::SourceBoundaryUnavailable(id, "SECTION179_CARE_EARNED_SOURCE_REQUIRED")
}

// These native local-income categories can be employee compensation. The
// section179 income owner has no supplier for them yet, even when Form2441 does.
fn local_compensation(kind: &crate::OtherIncomeKind) -> bool {
    matches!(
        kind,
        crate::OtherIncomeKind::HouseholdWages
            | crate::OtherIncomeKind::StrikeBenefits
            | crate::OtherIncomeKind::InmatePay
            | crate::OtherIncomeKind::MedicaidWaiver { in_your_home: false }
    )
}

// Reg.1.179-2(c)(1) and the K-1 box 14 instructions: the partner's SE source
// before any section 179 deduction. The suspended passive loss belongs to the
// shared Form 8582 owner; accept the source only when no suspension that owner
// could assign changes its SE earnings.
fn partner_se(
    id: i64,
    data: &crate::PartnerK1,
) -> Result<crate::partnership::PartnerSeResult, Issue> {
    use crate::kernel::carry::Amount;
    let rule = |(key, _): (&'static str, &'static str)| Issue::SourceBoundaryUnavailable(id, key);
    let pool = data.pool(&data.regular).map_err(rule)?;
    let se = |suspended: i64| {
        let suspended = Amount::new(suspended).map_err(|_| Issue::AmountOverflow(id))?;
        data.se_after_passive(suspended, Amount::default()).map_err(rule)
    };
    let least = se(0)?;
    let most = se((-pool.passive).max(0)).map_err(|_| boundary(id))?;
    if data.publicly_traded
        || (least.earnings, least.earnings_cents) != (most.earnings, most.earnings_cents)
    {
        return Err(boundary(id));
    }
    Ok(least)
}

// Employee stock compensation not in W-2 box 1, under the conditions of the
// native ESPP and ISO collectors (Totals::iso_unreported).
pub(super) fn unreported_compensation(income: &FederalIncome) -> i64 {
    match income {
        FederalIncome::EsppSale { data } if !data.special && !data.in_wages => data.compensation(),
        FederalIncome::IsoSale { data } if data.confirmed && !data.in_wages => data.compensation(),
        _ => 0,
    }
}

fn collect_other(
    source: &crate::FederalIncomeEntry,
    kind: &crate::OtherIncomeKind,
    amount: i64,
    payroll: &mut crate::federal_rules::Totals,
    r: &mut FederalResult,
) -> Result<(), Issue> {
    if amount != 0 && local_compensation(kind) {
        return Err(boundary(source.id));
    }
    crate::other_income::collect(
        &mut crate::other_income::Ctx {
            totals: payroll,
            result: r,
            id: source.id,
            owner: usize::from(source.owner == crate::DocumentOwner::Spouses),
            context: crate::interview::income_issue_context(source),
        },
        kind,
        amount,
    );
    Ok(())
}

pub(crate) fn calculate(
    d: &FederalDraft,
    selected: &Section179PreparedAllocation,
    claims: &BTreeMap<i64, i64>,
    employee_compensation: &crate::employee_expenses::Before179Compensation,
) -> Result<CareCompensation, Issue> {
    let setup = d.setup.as_ref().ok_or(Issue::MissingReturnContext)?;
    let people = d.people.as_ref().ok_or(Issue::MissingReturnContext)?;
    let credits = d.credits.as_ref().ok_or(Issue::MissingReturnContext)?;
    let screening = d.screening.as_ref().ok_or(Issue::MissingReturnContext)?;
    d.adjustments.as_ref().ok_or(Issue::MissingReturnContext)?;
    if selected.income.system != super::Section179IncomeSystem::Regular
        || !credits.care.own_business.is_empty()
        || crate::adoption::has_benefits(&credits.adoptions)
        || crate::community::applies(setup)
        || !screening.clergy.is_empty()
        || screening.optional_se.iter().any(|e| e.farm || e.nonfarm.is_some())
    {
        return Err(boundary(0));
    }
    // Check every origin before collecting a subset of Totals. The established
    // unearned arms do not contribute to the native Form2441 earned expression.
    let mut partners = alloc::vec::Vec::new();
    for entry in &d.income {
        match &entry.income {
            FederalIncome::Business { data } => {
                if selected.participation.get(entry.id) != crate::participation::Participation::Yes
                    || !data.all_at_risk()
                    || !matches!(data.source, crate::BusinessSource::UnitedStates)
                    || !matches!(
                        data.activity,
                        crate::BusinessActivity::Ordinary { .. }
                            | crate::BusinessActivity::Farming { .. }
                    )
                    || crate::clergy::business(data)
                {
                    return Err(boundary(entry.id));
                }
            }
            FederalIncome::Wage { data } => {
                let data = crate::FederalWage::from(data);
                if data.coverage != crate::W2Coverage::Ordinary
                    || data.abroad != 0
                    || data.statutory_employee
                    || data.inmate_pay != 0
                    || data.deferred_pension != 0
                    || data.disability.amount() != 0
                {
                    return Err(boundary(entry.id));
                }
            }
            // A 1099-NEC answered as employee pay, or a Form 8919 record, with
            // a Form 8919 reason: Form 1040 line 1g wages, earned under IRC129(e)(2).
            FederalIncome::EmployeeWages { data } if data.reason().is_some() => {}
            FederalIncome::Retirement { data: source }
                if {
                    let data = &crate::FederalRetirement::from(source);
                    !data.reports_disability_wages()
                        && (data.kind.ira()
                            || !matches!(
                                data.ordinary_distribution,
                                Some(crate::RetirementTreatment::Corrective { .. })
                            ))
                } => {}
            FederalIncome::Broker { data }
                if crate::broker_calculation::known_components(data, true, true) => {}
            FederalIncome::Interest { .. }
            | FederalIncome::Oid { .. }
            | FederalIncome::Dividend { .. }
            | FederalIncome::SocialSecurity { .. }
            | FederalIncome::Gambling { .. }
            | FederalIncome::BondRedemption { .. }
            | FederalIncome::SavingsBonds { .. }
            | FederalIncome::Section1256 { .. }
            | FederalIncome::EducationDistribution { .. }
            | FederalIncome::EducationAccountExpenses { .. }
            | FederalIncome::Royalty { .. }
            | FederalIncome::NonbusinessProperty { .. }
            | FederalIncome::WorthlessSecurity { .. }
            | FederalIncome::NonbusinessBadDebt { .. }
            | FederalIncome::ReturnOfCapital { .. }
            | FederalIncome::UndistributedGain { .. }
            | FederalIncome::JuryDuty { .. }
            | FederalIncome::Alimony { .. }
            | FederalIncome::Scholarship { .. }
            | FederalIncome::OtherIncome { .. }
            | FederalIncome::Nec1099 { .. }
            | FederalIncome::Misc1099 { .. }
            | FederalIncome::K1099 { .. }
            | FederalIncome::Cooperative { .. }
            | FederalIncome::GovernmentPayment { .. }
            | FederalIncome::Sale { .. }
            | FederalIncome::NuaSale { .. }
            | FederalIncome::HealthLoanAssistance { .. }
            | FederalIncome::WildfireRelief { .. }
            | FederalIncome::DisasterRelief { .. }
            | FederalIncome::EstateTrustPortfolio { .. }
            | FederalIncome::PersonalDebt { .. }
            | FederalIncome::InheritedIra { .. }
            | FederalIncome::Roth { .. }
            | FederalIncome::LtcBenefits { .. }
            | FederalIncome::Able { .. }
            | FederalIncome::TrumpDeath { .. }
            | FederalIncome::TrumpAbleRollover { .. }
            | FederalIncome::SmallBusinessStock { .. }
            | FederalIncome::QualifiedSmallBusinessSale { .. }
            | FederalIncome::HomeSale { .. }
            | FederalIncome::OpportunityFund { .. }
            | FederalIncome::OpportunityFundDeferral { .. }
            | FederalIncome::Rental { .. }
            | FederalIncome::PropertyAcquisition { .. }
            | FederalIncome::FarmRental { .. }
            | FederalIncome::MedicalDistribution { .. }
            | FederalIncome::LtcInsured { .. }
            | FederalIncome::InstallmentSale { .. }
            | FederalIncome::CasualtyLivingInsurance { .. }
            | FederalIncome::DisasterMitigation { .. } => {}
            FederalIncome::Partnership { data } => {
                partners.push((entry.id, partner_se(entry.id, data)?))
            }
            FederalIncome::PartnershipActivities { data }
                if matches!(data.ordinary_self_employment(), Ok(None)) => {}
            // An S-corporation share adds no earned income. Employee stock
            // compensation is in W-2 box 1 or, when not, collected below.
            FederalIncome::Scorp { .. }
            | FederalIncome::ScorpActivities { .. }
            | FederalIncome::EmployeeStockSale { .. }
            | FederalIncome::EsppSaleSource { .. }
            | FederalIncome::EsppSale { .. }
            | FederalIncome::IsoSale { .. } => {}
            _ => return Err(boundary(entry.id)),
        }
    }
    let joint = crate::federal_rules::joint(setup.filing_status());
    let mut r = crate::federal_rules::boxed(|| FederalResult::empty(d.revision));
    let mut earned =
        super::owner_deductions::regular_earned_sources(d, selected, claims, &partners, &mut r)?;
    for (wages, excess) in earned.payroll.wages.iter_mut().zip(employee_compensation.by_owner) {
        *wages = wages.checked_add(excess).ok_or(Issue::AggregateAmountOverflow)?;
    }
    for entry in &d.income {
        match &entry.income {
            FederalIncome::OtherIncome { data } => {
                collect_other(entry, &data.kind, data.amount, &mut earned.payroll, &mut r)?;
            }
            FederalIncome::Nec1099 { data } => {
                let data = crate::FederalForm1099Nec::from(data);
                if let Some(crate::IncomeUse::Income { kind }) = &data.belongs {
                    collect_other(entry, kind, data.compensation, &mut earned.payroll, &mut r)?;
                }
            }
            FederalIncome::Misc1099 { data } => {
                let data = &crate::Form1099Misc::from(data);
                if data.deferred != 0 {
                    return Err(boundary(entry.id));
                }
                if let Some(crate::IncomeUse::Income { kind }) = &data.other_use {
                    collect_other(entry, kind, data.other, &mut earned.payroll, &mut r)?;
                }
            }
            FederalIncome::EsppSale { .. } | FederalIncome::IsoSale { .. } => {
                let total = &mut earned.payroll.iso_unreported
                    [usize::from(entry.owner == crate::DocumentOwner::Spouses)];
                *total = total
                    .checked_add(unreported_compensation(&entry.income))
                    .ok_or(Issue::AggregateAmountOverflow)?;
            }
            _ => {}
        }
    }
    earned.payroll.moving_excess = crate::moving::figure(d, &mut r, joint).excess;
    earned.payroll.deferral_excess = [
        crate::elective_deferrals::excess(d, crate::Owner::Taxpayer, people.taxpayer.birth_date)
            .taxable,
        people.spouse.as_ref().filter(|_| joint).map_or(0, |spouse| {
            crate::elective_deferrals::excess(d, crate::Owner::Spouse, spouse.birth_date).taxable
        }),
    ];
    // Both owners expose these earned wages, but the current section179
    // starting-income collector does not yet include their compensation.
    if earned.payroll.moving_excess.iter().chain(&earned.payroll.deferral_excess).any(|v| *v != 0) {
        return Err(boundary(0));
    }
    let family = crate::federal_rules::family(d, &mut r, people);
    let input = core::array::from_fn(|i| {
        crate::federal_rules::earned_before_special_adjustments(
            &earned.payroll,
            i,
            earned.profit[i],
            earned.half[i],
        )
    });
    let context = crate::forms::form_2441::Form2441Context {
        draft: d,
        setup,
        credits,
        joint,
        care_people: family.care_people,
        care_expenses: family.care_expenses,
        earned: crate::federal_rules::care_earned_input(&earned.payroll, input),
        // These source classes were explicitly excluded above.
        optional_eic: [0; 2],
        deduction_owners: [0; 2],
    };
    let care =
        crate::forms::form_2441::compute(&context, &mut r).ok_or(Issue::AggregateAmountOverflow)?;
    if !r.issues.is_empty() {
        return Err(Issue::OwnerIssues(core::mem::take(&mut r.issues)));
    }
    Ok(CareCompensation { taxable_dollars: care.taxable })
}
