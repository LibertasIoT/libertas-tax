// Coverage: Form 8958 — Calculation for every income kind (audited 2026-10-10, unit B9-3; Pub.
//   555, 26 U.S.C. 66 and 879, 1402(a)(5)).
//   Lines: a married-filing-separately return in a community-property state or country reports
//   half of each community item and all of its own separate items; every income row answers
//   Community, Separate or Partly (a percent), and the spouse's documents are entered under
//   "Your spouse". One factor per item applies where the item enters the return totals
//   (federal_rules.rs Totals): income, gains and withholding follow the factor, while the
//   employee's Social Security and Medicare wages, the operator's self-employment tax, IRA
//   compensation and the earned-income credit's earnings stay with the person who earned them.
//   IRA, Coverdell and HSA distributions, Social Security and ABLE payments are the owner's
//   separate income whatever the row says. Income from separate property is community income in
//   Idaho, Louisiana, Texas and Wisconsin (and a country whose law says so) for interest,
//   dividends, rents and royalties; gains follow how the property is held. Spouses living apart
//   all year (section 66(a)) and a nonresident alien spouse (section 879) give earned income,
//   business and partnership income to the spouse who earned it; other community income is still
//   halved. The tax withheld on a Form 1099 routed to a business follows the business's share, not
//   the document's (Reg. 1.31-1(a); withholding_share, F14), and Form 8958 lists a routed amount
//   once, as the destination's row. Not covered: section 66(b) and (c) relief (Form 8857), Alaska, South Dakota and
//   Tennessee elective regimes, registered domestic partners, Form 2555 (section 911(b)(2)(C)).
//   Evidence: src/tests/attribution/family.rs community_* (fixtures F4), federal_rules.rs Totals scaling;
//   f8958, Pub. 555.
//   Unit21 completes whole-interest source allocation before each return's
//   basis, at-risk, PAL, section179, QBI, capital, credit and AMT owners.
//   Actual-partner SE uses the shared annual owner and Form8958 source attribution;
//   church/clergy earnings stay outside partner source tax and deduction alike
//   on both returns, preserving each annual amount's independent rounding.
//   unresolved binding negative-box14 sources explicitly refuse both counterparts.
//   Evidence: family::unit21_, farm_rental::unit21_, interview::unit21_, and
//   schedules_1_3::assert_form_8958_divided_sources_conserve.
use crate::{
    Answer, CommunityProperty, CommunityRule, CommunityShare, CommunityState, FederalIncome,
    FederalSetup, FederalStatus, Owner, RetirementKind,
};

pub(crate) use crate::community_share::{Divide, Share, divide_enum, divide_record};

// How one kind of income divides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    // Always the owner's: IRAs, Coverdell and HSA accounts, Social Security,
    // ABLE accounts (Pub. 555 "Withdrawals from IRAs and Coverdell ESAs").
    Owners,
    // Pay for personal services, a business or a partnership share
    // (section 879(a)(1) and (2)).
    Earned,
    // Interest, dividends, rents and royalties: income from property.
    Property,
    // A partnership share follows property income under the normal halves
    // rule and the partner under section 879(a)(2).
    Partnership,
    // Everything else, divided like any community item.
    Other,
}

fn kind(income: &FederalIncome) -> Kind {
    match income {
        FederalIncome::Retirement { data: source }
            if {
                let data = &crate::FederalRetirement::from(source);
                data.kind == RetirementKind::Ira
            } =>
        {
            Kind::Owners
        }
        FederalIncome::Roth { .. }
        | FederalIncome::InheritedIra { .. }
        | FederalIncome::SocialSecurity { .. }
        | FederalIncome::EducationDistribution { .. }
        | FederalIncome::EducationAccountExpenses { .. }
        | FederalIncome::MedicalDistribution { .. }
        | FederalIncome::Able { .. }
        | FederalIncome::TrumpDeath { .. }
        | FederalIncome::TrumpAbleRollover { .. }
        | FederalIncome::BondRedemption { .. }
        | FederalIncome::SavingsBonds { .. } => Kind::Owners,
        FederalIncome::Wage { .. }
        | FederalIncome::EmployeeWages { .. }
        | FederalIncome::Business { .. }
        | FederalIncome::Retirement { .. } => Kind::Earned,
        FederalIncome::Partnership { .. } => Kind::Partnership,
        FederalIncome::Interest { .. }
        | FederalIncome::Oid { .. }
        | FederalIncome::Dividend { .. }
        | FederalIncome::Rental { .. }
        | FederalIncome::FarmRental { .. }
        | FederalIncome::Scorp { .. }
        | FederalIncome::EstateTrustPortfolio { .. }
        | FederalIncome::NonbusinessProperty { .. } => Kind::Property,
        // Project native components at their normal collector boundary;
        // printed broker paper is never a divided source record.
        FederalIncome::Broker { .. } => Kind::Other,
        _ => Kind::Other,
    }
}

// The share of an item the person who owns the document reports, as a
// fraction (numerator over denominator).
fn owner_part(
    state: CommunityState,
    rule: CommunityRule,
    kind: Kind,
    answer: CommunityShare,
    gains: bool,
) -> (i64, i64) {
    if kind == Kind::Owners {
        return (1, 1);
    }
    // Section 879(a): earned income goes to the spouse who earned it when the
    // spouses lived apart all year or one is a nonresident alien.
    let earner = (kind == Kind::Earned || (kind == Kind::Partnership && !gains))
        && rule != CommunityRule::Halves;
    let halved = (1, 2);
    let community = if earner { (1, 1) } else { halved };
    // Income from separate property is community income in some places, but
    // not under section 879.
    let separate = if !gains
        && matches!(kind, Kind::Property | Kind::Partnership)
        && rule == CommunityRule::Halves
        && state.separate_income_is_community()
    {
        halved
    } else {
        (1, 1)
    };
    match answer {
        CommunityShare::NotNeeded | CommunityShare::Separate => separate,
        CommunityShare::Community => community,
        CommunityShare::Partly { percent } => {
            let p = i64::from(percent);
            // The community part (p of 100) at its share, the rest at its own.
            let (cn, cd) = community;
            let (sn, sd) = separate;
            // (p/100) * cn/cd + ((100 - p)/100) * sn/sd over a common base.
            let den = 100 * cd * sd;
            (p * cn * sd + (100 - p) * sn * cd, den)
        }
    }
}

// These kinds run their own limits on the filer's divided source record.
pub(crate) fn self_divided(income: &FederalIncome) -> bool {
    matches!(
        income,
        FederalIncome::Rental { .. }
            | FederalIncome::FarmRental { .. }
            | FederalIncome::Partnership { .. }
            | FederalIncome::Scorp { .. }
            | FederalIncome::EstateTrustPortfolio { .. }
    )
}

// Borrow the accepted source when it is already wholly this return's. A divided
// value is transient; accepted answers, identity and the normal lifecycle stay intact.
pub(crate) fn record<'a>(
    d: &crate::FederalDraft,
    entry: &'a crate::FederalIncomeEntry,
) -> alloc::borrow::Cow<'a, FederalIncome> {
    let Some(share) = d.setup.as_ref().and_then(|setup| entry_share(setup, entry)) else {
        return alloc::borrow::Cow::Borrowed(&entry.income);
    };
    if !share.whole()
        && let FederalIncome::Rental { data } = &entry.income
    {
        return alloc::borrow::Cow::Owned(FederalIncome::Rental { data: data.divided(share) });
    }
    divided(share, &entry.income)
}
pub(crate) fn divided(
    share: Share,
    income: &FederalIncome,
) -> alloc::borrow::Cow<'_, FederalIncome> {
    if share.whole() || !self_divided(income) {
        return alloc::borrow::Cow::Borrowed(income);
    }
    alloc::borrow::Cow::Owned(match income {
        FederalIncome::Rental { data } => FederalIncome::Rental { data: data.divided(share) },
        FederalIncome::FarmRental { data } => {
            FederalIncome::FarmRental { data: data.divided(share) }
        }
        FederalIncome::Partnership { data } => {
            let mut data = data.divided(share);
            // A valid positive source credit can allocate to zero cents. It
            // contributes no claim row, while its accepted source stays intact.
            data.credits.retain(|credit| credit.amount != 0);
            data.investment_property.retain(|property| match property {
                crate::K1InvestmentProperty::RehabilitationRental { expenditures, .. }
                | crate::K1InvestmentProperty::RehabilitationOther { expenditures, .. } => {
                    *expenditures != 0
                }
                crate::K1InvestmentProperty::Energy { basis, .. }
                | crate::K1InvestmentProperty::CleanElectricity { basis, .. } => *basis != 0,
            });
            if data.prior_fuel_credit.as_ref().is_some_and(|prior| prior.claimed_credit == 0) {
                data.prior_fuel_credit = None;
            }
            FederalIncome::Partnership { data }
        }
        FederalIncome::Scorp { data } => FederalIncome::Scorp { data: data.divided(share) },
        FederalIncome::EstateTrustPortfolio { data } => {
            let mut data = data.divided(share);
            data.statements.reported_credits.retain(|credit| credit.paper.amount != 0);
            data.statements.investment_property.retain(|item| match &item.property {
                crate::K1InvestmentProperty::RehabilitationRental { expenditures, .. }
                | crate::K1InvestmentProperty::RehabilitationOther { expenditures, .. } => {
                    *expenditures != 0
                }
                crate::K1InvestmentProperty::Energy { basis, .. }
                | crate::K1InvestmentProperty::CleanElectricity { basis, .. } => *basis != 0,
            });
            FederalIncome::EstateTrustPortfolio { data }
        }
        _ => unreachable!("self-divided source kind"),
    })
}
// Each consumer borrows whole entries and owns only transient divided entries.
// Source IDs, answers and owner facts remain those of the accepted document.
pub(crate) fn entry<'a>(
    d: &crate::FederalDraft,
    entry: &'a crate::FederalIncomeEntry,
) -> alloc::borrow::Cow<'a, crate::FederalIncomeEntry> {
    match record(d, entry) {
        alloc::borrow::Cow::Borrowed(_) => alloc::borrow::Cow::Borrowed(entry),
        alloc::borrow::Cow::Owned(income) => {
            let mut allocated = entry.clone();
            allocated.income = income;
            alloc::borrow::Cow::Owned(allocated)
        }
    }
}
pub(crate) fn entries(
    d: &crate::FederalDraft,
) -> alloc::vec::Vec<alloc::borrow::Cow<'_, crate::FederalIncomeEntry>> {
    d.income.iter().map(|source| entry(d, source)).collect()
}

// Direct rental-property gains require the property basis and price to use
// one property fraction, even where the rental income has a different one.
pub(crate) fn property_entry<'a>(
    d: &crate::FederalDraft,
    entry: &'a crate::FederalIncomeEntry,
) -> alloc::borrow::Cow<'a, crate::FederalIncomeEntry> {
    if !matches!(entry.income, FederalIncome::Rental { .. } | FederalIncome::FarmRental { .. }) {
        return self::entry(d, entry);
    }
    let Some(share) = d.setup.as_ref().and_then(|setup| entry_share(setup, entry)) else {
        return alloc::borrow::Cow::Borrowed(entry);
    };
    match divided(share.for_gains(), &entry.income) {
        alloc::borrow::Cow::Borrowed(_) => alloc::borrow::Cow::Borrowed(entry),
        alloc::borrow::Cow::Owned(income) => {
            let mut allocated = entry.clone();
            allocated.income = income;
            alloc::borrow::Cow::Owned(allocated)
        }
    }
}
pub(crate) fn property_entries(
    d: &crate::FederalDraft,
) -> alloc::vec::Vec<alloc::borrow::Cow<'_, crate::FederalIncomeEntry>> {
    d.income.iter().map(|entry| property_entry(d, entry)).collect()
}

pub(crate) fn professional_owner(setup: Option<&FederalSetup>, owner: Owner) -> Owner {
    if setup.is_some_and(applies) { Owner::Taxpayer } else { owner }
}

pub(crate) fn share(
    setup: &FederalSetup,
    owner: Owner,
    answer: CommunityShare,
    income: &FederalIncome,
) -> Option<Share> {
    share_of_kind(setup, owner, answer, kind(income))
}

// Pub. 550 assigns jointly held investment income by the spouses' ownership;
// Pub. 555 then applies the state's community or separate-property treatment.
// Compose those fractions before the existing source rounding boundary.
pub(crate) fn entry_share(
    setup: &FederalSetup,
    entry: &crate::FederalIncomeEntry,
) -> Option<Share> {
    entry_share_of_kind(setup, entry, kind(&entry.income))
}

fn entry_share_of_kind(
    setup: &FederalSetup,
    entry: &crate::FederalIncomeEntry,
    kind: Kind,
) -> Option<Share> {
    let assignment = entry.accepted_assignment();
    if assignment.owner != crate::DocumentRecipient::Joint {
        // A separate return may record either spouse's financial account.
        // Outside community property, only its owner's return reports it.
        if setup.filing_status() == FederalStatus::Separate
            && !matches!(setup.community_property, CommunityProperty::Yes { .. })
            && matches!(
                assignment.owner,
                crate::DocumentRecipient::Taxpayer | crate::DocumentRecipient::Spouse
            )
            && matches!(
                entry.income,
                FederalIncome::Interest { .. }
                    | FederalIncome::Dividend { .. }
                    | FederalIncome::ReturnOfCapital { .. }
            )
        {
            return Some(Share {
                owner_num: 1,
                owner_den: 1,
                gain_num: 1,
                gain_den: 1,
                filer_owns: assignment.owner == crate::DocumentRecipient::Taxpayer,
            });
        }
        return share_of_kind(setup, entry.owner.owned(), entry.community, kind);
    }
    if setup.filing_status() != FederalStatus::Separate {
        return None;
    }
    let ([taxpayer, spouse], denominator) = assignment.owner_weights()?;
    let part = |gains| {
        let (numerator, divisor) = match setup.community_property {
            CommunityProperty::Yes { state, rule } => {
                owner_part(state, rule, kind, entry.community, gains)
            }
            CommunityProperty::NotAnswered | CommunityProperty::No => (1, 1),
        };
        (taxpayer * numerator + spouse * (divisor - numerator), denominator * divisor)
    };
    let (owner_num, owner_den) = part(false);
    let (gain_num, gain_den) = part(true);
    Some(Share { owner_num, owner_den, gain_num, gain_den, filer_owns: true })
}

// The share of the tax withheld on an item. It follows the income it was
// withheld from (Reg. 1.31-1(a)): a Form 1099 whose amount is routed to a
// business is withheld from that business's earnings, which section 879(a)
// gives to the spouse who earned them, whatever share the document's other
// boxes divide by. Any other item's withholding divides like the item.
pub(crate) fn withholding_share(
    d: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<Share> {
    entry_share_of_kind(d.setup.as_ref()?, entry, withholding_kind(d, entry, source_claims))
}

fn withholding_kind(
    d: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Kind {
    let routed_to_business = crate::information_returns::flows(&entry.income).iter().any(|flow| {
        crate::information_returns::resolve_flow(d, entry, flow, source_claims)
            .is_some_and(|(kind, _)| kind == crate::information_returns::Kind::Business)
    });
    if routed_to_business { Kind::Earned } else { kind(&entry.income) }
}

// Source character follows the resolved activity; an unresolved source stays unknown.
pub(crate) fn refund_earned(
    d: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<bool> {
    match withholding_kind(d, entry, source_claims) {
        Kind::Earned => Some(true),
        Kind::Owners | Kind::Property => Some(false),
        Kind::Other if matches!(entry.income, FederalIncome::Sale { .. }) => Some(false),
        Kind::Other
            if matches!(&entry.income, FederalIncome::Broker { data }
            if data.projection().receipt.is_none_or(|receipt| receipt.amount == 0)) =>
        {
            Some(false)
        }
        Kind::Other | Kind::Partnership => None,
    }
}

fn share_of_kind(
    setup: &FederalSetup,
    owner: Owner,
    answer: CommunityShare,
    kind: Kind,
) -> Option<Share> {
    if setup.filing_status() != FederalStatus::Separate {
        return None;
    }
    let CommunityProperty::Yes { state, rule } = setup.community_property else {
        return None;
    };
    let (owner_num, owner_den) = owner_part(state, rule, kind, answer, false);
    let (gain_num, gain_den) = owner_part(state, rule, kind, answer, true);
    Some(Share { owner_num, owner_den, gain_num, gain_den, filer_owns: owner == Owner::Taxpayer })
}

// A return that divides community income.
pub(crate) fn applies(setup: &FederalSetup) -> bool {
    setup.filing_status() == FederalStatus::Separate
        && matches!(setup.community_property, CommunityProperty::Yes { .. })
}

// The answers a row of a return like this may give (the variant indexes of
// CommunityShare): the three answers on a community-property separate return,
// the one "not needed" everywhere else.
pub(crate) fn allowed_answers(setup: Option<&FederalSetup>) -> alloc::vec::Vec<i32> {
    if setup.is_some_and(applies) {
        alloc::vec![
            libertas_macros::variant_index!(CommunityShare::Community) as i32,
            libertas_macros::variant_index!(CommunityShare::Separate) as i32,
            libertas_macros::variant_index!(CommunityShare::Partly) as i32,
        ]
    } else {
        alloc::vec![libertas_macros::variant_index!(CommunityShare::NotNeeded) as i32]
    }
}

// The variant index an answer has in the allowed set.
pub(crate) fn index(answer: CommunityShare) -> i32 {
    (match answer {
        CommunityShare::NotNeeded => libertas_macros::variant_index!(CommunityShare::NotNeeded),
        CommunityShare::Community => libertas_macros::variant_index!(CommunityShare::Community),
        CommunityShare::Separate => libertas_macros::variant_index!(CommunityShare::Separate),
        CommunityShare::Partly { .. } => libertas_macros::variant_index!(CommunityShare::Partly),
    }) as i32
}

// The rule of a row's answer on this return, checked again on save: a share
// among the answers the return offers, and a percent from 1 to 99.
pub(crate) fn row_issue(
    setup: Option<&FederalSetup>,
    answer: CommunityShare,
) -> Option<crate::validation::ValidationIssue> {
    if !allowed_answers(setup).contains(&index(answer)) {
        return Some(
            crate::validation::ValidationIssue::new("COMMUNITY_ANSWER_NOT_ALLOWED").at_record(),
        );
    }
    match answer {
        CommunityShare::Partly { percent } if !(1..=99).contains(&percent) => Some(
            crate::validation::ValidationIssue::new("COMMUNITY_PERCENT_OUT_OF_RANGE")
                .at_field("percent"),
        ),
        _ => None,
    }
}

// Property-source classification for the Form 8379 refund overlay. This does
// not divide the separately earned Part III amounts or change joint taxation.
pub(crate) fn refund_community_percent(
    income: &FederalIncome,
    answer: CommunityShare,
    state: u16,
) -> Option<u32> {
    if kind(income) == Kind::Owners {
        return Some(0);
    }
    if matches!(state, 3 | 4 | 7 | 9) && kind(income) == Kind::Property {
        return Some(100);
    }
    match answer {
        CommunityShare::Community => Some(100),
        CommunityShare::Separate => Some(0),
        CommunityShare::Partly { percent } if (1..=99).contains(&percent) => Some(percent),
        _ => None,
    }
}

// The property view completes actual sale proceeds against property basis.
// Only its computed year-end event enters the income-view Form6198 plan.
// This is not an undivided SE ledger run or a replacement source answer.
pub(crate) fn partner_items<'a>(
    d: &crate::FederalDraft,
    id: i64,
    data: &'a crate::PartnerK1,
    source: &crate::PartnerActivityRecord,
) -> Result<crate::partnership::PartnerItems<'a>, (&'static str, &'static str)> {
    let mut items = data.items_for_record(source)?;
    let Some(entry) = d.income.iter().find(|entry| entry.id == id) else {
        return Ok(items);
    };
    let Some(share) = d.setup.as_ref().and_then(|setup| entry_share(setup, entry)) else {
        return Ok(items);
    };
    if share.owner_num * share.gain_den == share.gain_num * share.owner_den {
        return Ok(items);
    }
    let FederalIncome::Partnership { data: whole } = &entry.income else {
        return Ok(items);
    };
    let property = whole.divided(share.for_gains());
    let property_source = if core::ptr::eq(source, &data.regular) {
        &property.regular
    } else {
        property.alternative.as_ref().unwrap_or(&property.regular)
    };
    let property_items = property.items_for_record(property_source)?;
    let plan = property_source.plan(&property_items)?;
    let basis = property_source.basis.result_with(
        property_source
            .ordinary
            .checked_add(
                property_items
                    .k1()
                    .checked_box_one()
                    .ok_or(crate::partnership::PartnerPlan::RANGE)?,
            )
            .ok_or(crate::partnership::PartnerPlan::RANGE)?,
        property_source.exempt,
        property_source.nondeductible,
        &plan,
    )?;
    items.property_disposal = Some(plan.disposal_of(&basis));
    Ok(items)
}
pub(crate) fn partner_pool(
    d: &crate::FederalDraft,
    id: i64,
    data: &crate::PartnerK1,
    source: &crate::PartnerActivityRecord,
) -> Result<crate::partnership::PartnerPoolResult, (&'static str, &'static str)> {
    data.pool_with_items(source, &partner_items(d, id, data, source)?)
}
// Retain the existing selected-system native result, before return179/PAL.
pub(crate) fn partner_risk(
    d: &crate::FederalDraft,
    id: i64,
    data: &crate::PartnerK1,
    source: &crate::PartnerActivityRecord,
) -> Result<crate::partnership::PartnerOrdinaryResult, (&'static str, &'static str)> {
    source.activity_result(&partner_items(d, id, data, source)?)
}
pub(crate) fn partner_ledger(
    d: &crate::FederalDraft,
    id: i64,
    data: &crate::PartnerK1,
    source: &crate::PartnerActivityRecord,
) -> Result<crate::k1_loss::LossLedger, (&'static str, &'static str)> {
    partner_risk(d, id, data, source).map(|value| value.ledger)
}
pub(crate) fn partner_capital(
    d: &crate::FederalDraft,
    id: i64,
    data: &crate::PartnerK1,
    source: &crate::PartnerActivityRecord,
) -> Option<[i64; 2]> {
    let ledger = partner_ledger(d, id, data, source).ok()?;
    let [short, long] = data.capital_gains_by_term();
    Some([
        short - ledger.risk_allowed[crate::k1_loss::K1LossClass::ShortTerm].get(),
        long - ledger.risk_allowed[crate::k1_loss::K1LossClass::LongTerm].get(),
    ])
}
pub(crate) fn partner_facts<'a>(
    d: &crate::FederalDraft,
    entry: &'a crate::FederalIncomeEntry,
    data: &crate::PartnerK1,
) -> Result<crate::k1_totals::Facts<'a>, (&'static str, &'static str)> {
    let mut facts = data.totals_facts(entry)?;
    for (source, flows, disposal) in [
        (&data.regular, &mut facts.flows, &mut facts.disposal),
        (
            data.alternative.as_ref().unwrap_or(&data.regular),
            &mut facts.amt_flows,
            &mut facts.amt_disposal,
        ),
    ] {
        match partner_items(d, entry.id, data, source) {
            Ok(items) => (*flows, *disposal) = data.outcome_with_items(source, &items)?,
            Err(error) if error.0 == crate::k1_activity::ordinary::RANGE_KEY => return Err(error),
            Err(_) => {}
        }
    }
    Ok(facts)
}

fn scorp_event_source(
    d: &crate::FederalDraft,
    id: i64,
    amt: bool,
) -> Option<crate::scorp_amt::ScorpSource<'static>> {
    let entry = d.income.iter().find(|entry| entry.id == id)?;
    let share = d.setup.as_ref().and_then(|setup| entry_share(setup, entry))?;
    if share.owner_num * share.gain_den == share.gain_num * share.owner_den {
        return None;
    }
    let FederalIncome::Scorp { data: whole } = &entry.income else {
        return None;
    };
    let property = crate::ScorpK1::from(whole).divided(share.for_gains());
    let property = if amt {
        property
            .amt
            .as_ref()
            .and_then(|record| record.source(&property).ok())
            .unwrap_or_else(|| property.into())
    } else {
        property.into()
    };
    Some(property)
}

pub(crate) fn scorp_events(
    d: &crate::FederalDraft,
    id: i64,
    amt: bool,
) -> Result<Option<crate::scorp::ScorpEvents>, (&'static str, &'static str)> {
    let Some(property) = scorp_event_source(d, id, amt) else {
        return Ok(None);
    };
    let basis = property.basis_result()?;
    Ok(Some(crate::scorp::ScorpEvents {
        disposal: property.disposal(&basis),
        loans: property.loan_gains(&basis)?,
    }))
}
// PRINT uses the same property-income versus gain source as actual loan gains.
// A regular community allocation cannot substitute for the gain allocation.
pub(crate) fn scorp_loan_rows(
    d: &crate::FederalDraft,
    id: i64,
    source: &crate::ScorpK1,
) -> Result<alloc::vec::Vec<crate::scorp::RepaymentRow>, (&'static str, &'static str)> {
    let property = scorp_event_source(d, id, false);
    let source =
        property.as_ref().map(crate::scorp_amt::ScorpSource::from).unwrap_or_else(|| source.into());
    source.loan_rows(&source.basis_result()?)
}

pub(crate) fn scorp_pool<'a>(
    d: &crate::FederalDraft,
    id: i64,
    data: impl Into<crate::scorp_amt::ScorpSource<'a>>,
    amt: bool,
) -> Result<crate::scorp::ScorpActivity, (&'static str, &'static str)> {
    data.into().pool_activity_with_events(scorp_events(d, id, amt)?.as_ref())
}
pub(crate) fn scorp_risk<'a>(
    d: &crate::FederalDraft,
    id: i64,
    data: impl Into<crate::scorp_amt::ScorpSource<'a>>,
    amt: bool,
) -> Result<crate::scorp_risk::ScorpRiskResult, (&'static str, &'static str)> {
    data.into().risk_with_events(scorp_events(d, id, amt)?.as_ref())
}
pub(crate) fn scorp_ledger<'a>(
    d: &crate::FederalDraft,
    id: i64,
    data: impl Into<crate::scorp_amt::ScorpSource<'a>>,
    amt: bool,
) -> Result<crate::k1_loss::LossLedger, (&'static str, &'static str)> {
    scorp_risk(d, id, data, amt).map(|value| value.ledger)
}
pub(crate) fn scorp_facts<'a>(
    d: &crate::FederalDraft,
    entry: &'a crate::FederalIncomeEntry,
    data: &crate::ScorpK1,
) -> crate::k1_totals::Facts<'a> {
    let mut facts = data.totals_facts(entry);
    if let Ok(events) = scorp_events(d, entry.id, false) {
        (facts.flows, facts.disposal) = data.outcome_with_events(events.as_ref());
    }
    if let Some(source) = data.amt.as_ref().and_then(|record| record.source(data).ok())
        && let Ok(events) = scorp_events(d, entry.id, true)
    {
        (facts.amt_flows, facts.amt_disposal) = source.outcome_with_events(events.as_ref());
    }
    facts
}

// Allocate an amount produced by a physical-source owner (miles, vehicle
// caps, or other source-level elections) without changing its factual inputs.
pub(crate) fn farm_rental_amount(
    d: &crate::FederalDraft,
    id: i64,
    amount: impl FnOnce(&crate::FederalFarmRental) -> i64,
) -> Option<i64> {
    let entry = d.income.iter().find(|entry| entry.id == id)?;
    let share = d.setup.as_ref().and_then(|setup| entry_share(setup, entry))?;
    if share.whole() {
        return None;
    }
    let FederalIncome::FarmRental { data } = &entry.income else {
        return None;
    };
    Some(share.filer(amount(data)))
}

pub(crate) fn income_amount(d: &crate::FederalDraft, id: i64, amount: i64) -> i64 {
    d.income
        .iter()
        .find(|entry| entry.id == id)
        .and_then(|entry| d.setup.as_ref().and_then(|setup| entry_share(setup, entry)))
        .map_or(amount, |share| share.filer(amount))
}
// A linked property adjustment follows the same property view as its parcel.
// Other income kinds retain their existing source policy.
pub(crate) fn property_amount(d: &crate::FederalDraft, id: i64, amount: i64) -> i64 {
    d.income
        .iter()
        .find(|entry| entry.id == id)
        .filter(|entry| {
            matches!(entry.income, FederalIncome::Rental { .. } | FederalIncome::FarmRental { .. })
        })
        .and_then(|entry| d.setup.as_ref().and_then(|setup| entry_share(setup, entry)))
        .map_or(amount, |share| share.gain(amount))
}
pub(crate) fn partner_se(
    d: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
    data: &crate::PartnerK1,
    suspended: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    section179: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
) -> Result<crate::partnership::PartnerSeResult, (&'static str, &'static str)> {
    if !d.setup.as_ref().is_some_and(applies) {
        return data.se_after_passive(suspended, section179);
    }
    let original = d
        .income
        .iter()
        .find(|source| source.id == entry.id)
        .expect("allocated entry retains its source");
    let FederalIncome::Partnership { data: whole } = &original.income else {
        unreachable!("same source kind")
    };
    let pool = partner_pool(d, entry.id, data, &data.regular)?;
    whole.se_after_passive_with_limits(data, pool, suspended, section179)
}

// Form 8958 line 8 uses source income before the spouse's basis, at-risk
// and passive limits. It never substitutes the post-limit preview amount.
pub(crate) fn item_amount(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    entry: &crate::FederalIncomeEntry,
    home_use: Option<&crate::extended_rules::HomeUseContext>,
) -> Option<i64> {
    let d = form_context.draft;
    Some(match &entry.income {
        FederalIncome::Rental { .. } => {
            crate::extended_rules::whole_rental_net(form_context, entry, home_use).ok()?
        }
        FederalIncome::FarmRental { data } => data
            .net(
                d,
                usize::from(entry.owner == crate::DocumentOwner::Spouses),
                entry.id,
                crate::depreciation::mid_quarter(d),
                false,
            )
            .ok()?,
        FederalIncome::Partnership { data } => {
            data.regular.ordinary
                + data.regular.rentals.real_estate
                + data.regular.rentals.other
                + data.guaranteed_services
                + data.guaranteed_capital
        }
        FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            data.ordinary + data.rentals.real_estate + data.rentals.other
        }
        FederalIncome::EstateTrustPortfolio { data } => {
            let data = &crate::FederalEstateTrustPortfolioK1::from(data);
            data.interest
                + data.ordinary_dividends
                + data.short_gain
                + data.long_gain
                + data.royalty_income
                - data.box_9_royalty_depreciation
                - data.box_9_royalty_depletion
                - data.box_9_royalty_amortization
                + data.activities.activities.iter().map(|a| a.net(false)).sum::<i64>()
        }
        _ => 0,
    })
}

// Lines9/10 allocate the actual partner's annual SE amounts, including the
// wage base shared by all of that person's businesses/K-1s. The ordinary
// return coordinator supplies the filer's selected-method tax and attribution;
// the counterpart uses the same actual-method owner on already limited
// sources. Neither source set creates another limitation ledger.
// The selected allocation and HOME context belong to the invoking return pass.

#[allow(clippy::too_many_arguments)]
pub(crate) fn partner_se_lines_with_section179_with_claims(
    d: &crate::FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    own: (i64, &alloc::collections::BTreeMap<i64, i64>),
    counterpart_payroll: [i64; 2],
    qbi_loss_sources: &alloc::collections::BTreeSet<i64>,
    selected: Option<&crate::Section179PreparedAllocation>,
    result: &mut crate::FederalResult,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<(), crate::Section179PreparedIssue> {
    use crate::kernel::lines::{LineValue, row};
    use crate::line_emission::emit;
    let (own_tax_cents, own_half_cents) = own;
    let Some(setup) = &d.setup else { return Ok(()) };
    if !applies(setup) || !pool.inputs_complete {
        return Ok(());
    }
    carry_notes(d, pool, qbi_loss_sources, result);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let sources =
            if owner == Owner::Taxpayer { &pool.partner_se } else { &pool.counterpart_partner_se };
        if sources.is_empty() {
            continue;
        }
        let (tax, half, weights) = if owner == Owner::Taxpayer {
            let weights = own_half_cents.clone();
            let source_half = weights.values().sum::<i64>();
            // Tax and deductible-half totals each retain the annual owner's
            // rounding; deriving tax from the rounded half introduces drift.
            (own_tax_cents, source_half, weights)
        } else {
            let actual = match crate::partnership::owner_actual_se_with_section179_with_claims(
                crate::related_forms::RelatedFormContext::original(d, source_claims),
                owner,
                counterpart_payroll[0],
                counterpart_payroll[1],
                sources,
                selected,
                home,
            ) {
                Ok(actual) => actual,
                Err(crate::partnership::OwnerSeIssue::Prepared(issue)) => return Err(issue),
                Err(crate::partnership::OwnerSeIssue::Rule((key, _message))) => {
                    crate::issue_localized!(
                        result,
                        &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                        "Counterpart partnership self-employment",
                        key,
                        _message,
                    );
                    result.state = crate::ResultState::Unsupported;
                    return Ok(());
                }
            };
            let clergy = crate::clergy::calculate_with_section179_with_claims(
                d,
                owner,
                selected,
                home,
                source_claims,
            )?;
            let nonbusiness = crate::federal_rules::church_employee_se(counterpart_payroll[1])
                + crate::kernel::ratio::fraction(
                    crate::kernel::ratio::positive(crate::kernel::ratio::dollars(
                        clergy.extra_se(),
                    )),
                    9235,
                    10000,
                );
            let [tax, half] = crate::business_rules::business_se_totals(
                actual.tax,
                actual.half,
                actual.net,
                nonbusiness,
            )
            .map(|amount| amount * 100);
            let weights = crate::business_rules::half_se_allocation(half, actual.attribution_cents);
            (tax, half, weights)
        };
        let taxes = crate::business_rules::half_se_allocation(
            tax,
            weights.iter().map(|(id, amount)| (*id, *amount)),
        );
        let halves = crate::business_rules::half_se_allocation(half, weights);
        for (id, _) in sources {
            let Some(entry) = d.income.iter().find(|entry| entry.id == *id) else {
                continue;
            };
            let label = crate::interview::income_display_arg(entry);
            let rows = [
                (
                    "se_deduction",
                    "9",
                    halves.get(id).copied().unwrap_or(0),
                    "EXPLAIN_FORM8958_SE_DEDUCTION_TOTAL",
                    "EXPLAIN_FORM8958_SE_DEDUCTION_THIS_RETURN",
                ),
                (
                    "se_tax",
                    "10",
                    taxes.get(id).copied().unwrap_or(0),
                    "EXPLAIN_FORM8958_SE_TAX_TOTAL",
                    "EXPLAIN_FORM8958_SE_TAX_THIS_RETURN",
                ),
            ]
            .into_iter()
            .flat_map(|(kind, line, full, total_key, this_key)| {
                [
                    (kind, line, "total", full, total_key),
                    (
                        kind,
                        line,
                        "this_return",
                        if owner == Owner::Taxpayer { full } else { 0 },
                        this_key,
                    ),
                ]
            });
            emit(result, &mut (), rows, |(kind, line, suffix, amount, key), _, _| {
                Some(row(
                    Some(crate::FormLine::new(crate::TaxForm::Form8958, line)),
                    &alloc::format!("form8958_{kind}_{id}_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &["income.partnership", "setup.community_property"],
                    "https://www.irs.gov/pub/irs-pdf/f8958.pdf",
                    crate::Message::new(key, alloc::vec![label.clone()]),
                ))
            });
        }
    }
    Ok(())
}

fn carry_notes(
    d: &crate::FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    qbi_loss_sources: &alloc::collections::BTreeSet<i64>,
    result: &mut crate::FederalResult,
) {
    let Some(setup) = &d.setup else { return };
    for entry in d.income.iter().filter(|entry| self_divided(&entry.income)) {
        let Some(share) = entry_share(setup, entry) else {
            continue;
        };
        if share.whole() || share.none() {
            continue;
        }
        let carried = (qbi_loss_sources.contains(&entry.id)
            && result
                .lines
                .iter()
                .any(|line| line.id == "qbi_loss_carryforward" && line.amount > 0))
            || pool.k1_ledgers.iter().any(|(id, ledger)| *id == entry.id && ledger.held_back())
            || pool
                .estate_activities
                .iter()
                .any(|activity| activity.source == entry.id && activity.suspended > 0)
            || result.lines.iter().any(|line| {
                line.amount > 0
                    && [
                        alloc::format!("rental_{}_suspended_loss", entry.id),
                        alloc::format!("amt_rental_{}_suspended_loss", entry.id),
                        alloc::format!("farm_rental_{}_suspended_loss", entry.id),
                        alloc::format!("amt_farm_rental_{}_suspended_loss", entry.id),
                        alloc::format!("partnership_{}_section179_carryforward", entry.id),
                        alloc::format!("scorp_{}_section179_carryforward", entry.id),
                    ]
                    .contains(&line.id)
            });
        if carried {
            crate::federal_rules::note_once_scoped(
                result,
                crate::Message::new(
                    "NOTE_COMMUNITY_CARRY_IS_THIS_RETURNS_SHARE",
                    alloc::vec![crate::interview::income_display_arg(entry)],
                ),
                crate::interview::income_issue_context(entry),
            );
        }
    }
}

// Allocation provenance is emitted once from the whole supported source,
// after the normal calculation, using the shared line-emission kernel.

pub(crate) fn explain_with_claims(
    d: &crate::FederalDraft,
    result: &mut crate::FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) {
    use crate::kernel::lines::{LineValue, row};
    use crate::line_emission::emit;
    let Some(setup) = &d.setup else { return };
    if !applies(setup) || result.state != crate::ResultState::EstimatedForSupportedScope {
        return;
    }
    for entry in &d.income {
        if !self_divided(&entry.income) {
            continue;
        }
        let Some(share) = entry_share(setup, entry) else {
            continue;
        };
        let label = crate::interview::income_display_arg(entry);
        let Some(total) = item_amount(
            crate::related_forms::RelatedFormContext::original(d, source_claims),
            entry,
            result.rental_home_context.as_ref(),
        ) else {
            crate::extended_rules::expense_output_unavailable(result, Some(entry.id));
            return;
        };
        let (withholding, mut portfolio_gain, foreign_taxes) = match &entry.income {
            FederalIncome::Partnership { data } => data
                .separate
                .as_ref()
                .map_or((0, 0, 0), |v| (v.backup_withholding, v.short + v.long, v.foreign_taxes)),
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                (
                    data.withholding,
                    data.capital.as_ref().map_or(0, |v| v.short + v.long),
                    data.foreign_taxes,
                )
            }
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                (data.backup_withholding, 0, data.foreign_statement.as_ref().map_or(0, |v| v.tax))
            }
            _ => (0, 0, 0),
        };
        // The source gain column also includes coded capital/1256 items and
        // current entity property amounts before this return's limits/lookback.
        // Section1250 rate components are already included and never added twice.
        let coded_gain = match &entry.income {
            FederalIncome::Partnership { data } => {
                let flows = match data.regular.source_k1_items(&data.items()).and_then(|items| {
                    items.checked_flows_whole().ok_or(crate::partnership::PartnerPlan::RANGE)
                }) {
                    Ok(flows) => flows,
                    Err((key, _message)) => {
                        crate::issue_localized!(
                            result,
                            &crate::interview::income_issue_context(entry),
                            "K-1 income",
                            key,
                            _message
                        );
                        return;
                    }
                };
                let Some(capital) = flows.checked_capital() else {
                    crate::extended_rules::expense_output_unavailable(result, Some(entry.id));
                    return;
                };
                let Some(gain) = crate::kernel::ratio::checked_sum([
                    capital[0],
                    capital[1],
                    flows.section1256,
                    data.regular
                        .business_property(&data.items())
                        .ok()
                        .flatten()
                        .map_or(0, |property| property.net()),
                    data.regular.rentals.real_estate_property.net,
                    data.regular.rentals.other_property.net,
                ]) else {
                    crate::extended_rules::expense_output_unavailable(result, Some(entry.id));
                    return;
                };
                gain
            }
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                let flows = data.k1_items().flows_whole();
                flows.capital().iter().sum::<i64>()
                    + flows.section1256
                    + data.business_property().ok().flatten().map_or(0, |property| property.net())
                    + data.rentals.real_estate_property.net
                    + data.rentals.other_property.net
            }
            _ => 0,
        };
        let Some(projected_gain) = crate::kernel::ratio::checked_sum([portfolio_gain, coded_gain])
        else {
            let (key, _message) = crate::partnership::PartnerPlan::RANGE;
            crate::issue_localized!(
                result,
                &crate::interview::income_issue_context(entry),
                "K-1 income",
                key,
                _message
            );
            return;
        };
        portfolio_gain = projected_gain;
        let credits = match &entry.income {
            FederalIncome::Partnership { data } => {
                data.credits.iter().map(|credit| credit.amount).sum::<i64>()
                    + data
                        .investment_property
                        .iter()
                        .map(|property| property.current_year_credit() * 100)
                        .sum::<i64>()
                    + data
                        .fuel_tax_credit
                        .as_ref()
                        .map_or(0, crate::PartnerFuelTaxCredit::credit_cents)
            }
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                data.credits.iter().map(|credit| credit.amount).sum::<i64>()
                    + data
                        .investment_property
                        .iter()
                        .map(|property| property.current_year_credit() * 100)
                        .sum::<i64>()
            }
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                data.reported_credits.iter().map(|credit| credit.amount).sum::<i64>()
                    + data
                        .investment_property
                        .iter()
                        .map(|property| property.property.current_year_credit() * 100)
                        .sum::<i64>()
            }
            _ => 0,
        };
        let portfolio = match &entry.income {
            FederalIncome::Partnership { data } => data
                .separate
                .as_ref()
                .map_or([0; 3], |source| [source.interest, source.dividends, source.royalties]),
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                [data.interest, data.dividends, data.royalties]
            }
            _ => [0; 3],
        };
        let royalty_expenses = match &entry.income {
            FederalIncome::Partnership { data } => data
                .deductions
                .iter()
                .filter(|deduction| {
                    deduction.code == crate::PartnerDeductionCode::RoyaltyDeductions
                })
                .map(|deduction| deduction.amount)
                .sum::<i64>(),
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                data.deductions
                    .iter()
                    .filter(|deduction| {
                        deduction.code == crate::ScorpDeductionCode::RoyaltyDeductions
                    })
                    .map(|deduction| deduction.amount)
                    .sum::<i64>()
            }
            _ => 0,
        };
        let property_gain = match &entry.income {
            FederalIncome::Rental { data } => {
                match crate::asset_sale::rental_sale_amount(
                    crate::related_forms::RelatedFormContext::original(d, source_claims),
                    entry.id,
                    data,
                    crate::depreciation::mid_quarter(d),
                    false,
                    result.rental_home_context.as_ref(),
                ) {
                    Ok(value) => value,
                    Err(issue) => {
                        issue.report(entry.id, result);
                        return;
                    }
                }
            }
            FederalIncome::FarmRental { data } => {
                let land = crate::farm_land::source_with_claims(
                    d,
                    entry.id,
                    &data.farm,
                    false,
                    source_claims,
                )
                .map_or(0, |gain| gain.gain);
                let assets = crate::asset_sale::business_assets(
                    &data.assets,
                    crate::depreciation::mid_quarter(d),
                    false,
                )
                .map_or(0, |gain| gain.gain);
                (land + assets) * 100
            }
            FederalIncome::Partnership { data } => {
                let disposal = match data.outcome(&data.regular) {
                    Ok((_, disposal)) => disposal,
                    Err((key, _message)) => {
                        crate::issue_localized!(
                            result,
                            &crate::interview::income_issue_context(entry),
                            "K-1 income",
                            key,
                            _message
                        );
                        return;
                    }
                };
                disposal.sale_gain() + disposal.distribution_gain
            }
            FederalIncome::Scorp { data } => {
                let data = &crate::ScorpK1::from(data);
                data.basis_result().ok().map_or(0, |basis| {
                    let disposal = data.disposal(&basis);
                    let loans = data.loan_gains(&basis).unwrap_or([0; 3]);
                    disposal.sale_gain() + disposal.distribution_gain + loans[1] + loans[2]
                })
            }
            _ => 0,
        };
        // Distributive-share capital items follow the income fraction;
        // external property gains follow the independent property fraction.
        let Some((gain, gain_share)) =
            crate::kernel::ratio::checked_sum([portfolio_gain, property_gain]).zip(
                crate::kernel::ratio::checked_sum([
                    share.filer(portfolio_gain),
                    share.gain(property_gain),
                ]),
            )
        else {
            let (key, _message) = crate::partnership::PartnerPlan::RANGE;
            crate::issue_localized!(
                result,
                &crate::interview::income_issue_context(entry),
                "K-1 income",
                key,
                _message
            );
            return;
        };
        let mut values = alloc::vec![
            (alloc::format!("form8958_{}_total", entry.id), total, "EXPLAIN_FORM8958_N_TOTAL"),
            (
                alloc::format!("form8958_{}_this_return", entry.id),
                share.filer(total),
                "EXPLAIN_FORM8958_N_THIS_RETURN"
            ),
        ];
        for (prefix, full, this, key_total, key_this) in [
            (
                "gain",
                gain,
                gain_share,
                "EXPLAIN_FORM8958_GAIN_N_TOTAL",
                "EXPLAIN_FORM8958_GAIN_N_THIS_RETURN",
            ),
            (
                "withholding",
                withholding,
                share.filer(withholding),
                "EXPLAIN_FORM8958_WITHHOLDING_N_TOTAL",
                "EXPLAIN_FORM8958_WITHHOLDING_N_THIS_RETURN",
            ),
            (
                "credit",
                foreign_taxes + credits,
                share.filer(foreign_taxes + credits),
                "EXPLAIN_FORM8958_CREDIT_N_TOTAL",
                "EXPLAIN_FORM8958_CREDIT_N_THIS_RETURN",
            ),
        ] {
            if full != 0 {
                values.push((
                    alloc::format!("form8958_{prefix}_{}_total", entry.id),
                    full,
                    key_total,
                ));
                values.push((
                    alloc::format!("form8958_{prefix}_{}_this_return", entry.id),
                    this,
                    key_this,
                ));
            }
        }
        for (kind, amount, total_key, this_key) in [
            (
                "interest",
                portfolio[0],
                "EXPLAIN_FORM8958_INTEREST_TOTAL",
                "EXPLAIN_FORM8958_INTEREST_THIS_RETURN",
            ),
            (
                "dividends",
                portfolio[1],
                "EXPLAIN_FORM8958_DIVIDENDS_TOTAL",
                "EXPLAIN_FORM8958_DIVIDENDS_THIS_RETURN",
            ),
            (
                "royalties",
                portfolio[2] - royalty_expenses,
                "EXPLAIN_FORM8958_ROYALTIES_TOTAL",
                "EXPLAIN_FORM8958_ROYALTIES_THIS_RETURN",
            ),
        ] {
            if amount != 0 {
                values.push((
                    alloc::format!("form8958_{kind}_{}_total", entry.id),
                    amount,
                    total_key,
                ));
                values.push((
                    alloc::format!("form8958_{kind}_{}_this_return", entry.id),
                    share.filer(amount),
                    this_key,
                ));
            }
        }
        emit(result, &mut (), values, |(id, amount, key), _, _| {
            let line = if id.starts_with("form8958_gain_") {
                "6"
            } else if id.starts_with("form8958_withholding_") {
                "11"
            } else if id.starts_with("form8958_credit_") {
                "12"
            } else if id.starts_with("form8958_interest_") {
                "2"
            } else if id.starts_with("form8958_dividends_") {
                "3"
            } else {
                "8"
            };
            Some(row(
                Some(crate::FormLine::new(crate::TaxForm::Form8958, line)),
                &id,
                LineValue::MoneyCents(amount),
                &["income", "setup.community_property"],
                "https://www.irs.gov/pub/irs-pdf/f8958.pdf",
                crate::Message::new(key, alloc::vec![label.clone()]),
            ))
        });
        if !share.whole() && !share.none() {
            crate::federal_rules::note_once_scoped(
                result,
                crate::Message::new("NOTE_FORM_8958_ALLOCATION", alloc::vec![]),
                crate::interview::income_issue_context(entry),
            );
        }
    }
}

// Monetary owner outputs retain their original precision. In particular,
// annual Form4797 Dollars enter working cents once, before person attribution.
// These transient rows never change the accepted physical property records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SourceAmount {
    pub whole: i64,
    pub filer: i64,
    pub counterpart: i64,
}
impl SourceAmount {
    pub(crate) fn income(d: &crate::FederalDraft, id: i64, whole: i64) -> Self {
        let filer = income_amount(d, id, whole);
        Self { whole, filer, counterpart: whole - filer }
    }
    pub(crate) fn property(d: &crate::FederalDraft, id: i64, whole: i64) -> Self {
        let filer = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .and_then(|entry| d.setup.as_ref().and_then(|setup| entry_share(setup, entry)))
            .map_or(whole, |share| share.gain(whole));
        Self { whole, filer, counterpart: whole - filer }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PropertyCents {
    pub gain: i64,
    pub recapture: i64,
    pub ordinary: i64,
    pub section1231: i64,
    pub unrecaptured: i64,
}
impl PropertyCents {
    pub(crate) fn annual(
        value: crate::farm_land::LandGain,
    ) -> Result<Self, crate::kernel::loss_limits::LossLimitError> {
        let lift = crate::extended_rules::annual_dollars_to_working_cents;
        Ok(Self {
            gain: lift(value.gain)?,
            recapture: lift(value.recapture)?,
            ordinary: lift(value.ordinary)?,
            section1231: lift(value.section1231)?,
            unrecaptured: lift(value.unrecaptured)?,
        })
    }
    pub(crate) fn limited(
        value: &crate::business_risk::PropertySource,
    ) -> Result<Self, crate::kernel::loss_limits::LossLimitError> {
        let [ordinary, section1231] = value.allowed_cents;
        let lift = crate::extended_rules::annual_dollars_to_working_cents;
        Ok(Self {
            gain: ordinary
                .checked_add(section1231)
                .ok_or(crate::kernel::loss_limits::LossLimitError::Overflow)?,
            ordinary,
            section1231,
            recapture: lift(value.original.dollars.recapture)?,
            unrecaptured: lift(value.original.dollars.unrecaptured)?,
        })
    }
    fn columns(self) -> [i64; 5] {
        [self.gain, self.recapture, self.ordinary, self.section1231, self.unrecaptured]
    }
    fn from_columns(value: [i64; 5]) -> Self {
        Self {
            gain: value[0],
            recapture: value[1],
            ordinary: value[2],
            section1231: value[3],
            unrecaptured: value[4],
        }
    }
    pub(crate) fn for_source(
        d: &crate::FederalDraft,
        id: i64,
        value: crate::farm_land::LandGain,
        physical: bool,
    ) -> Result<Self, crate::activity_risk::SourceIssue> {
        Self::annual(value)
            .and_then(|value| if physical { value.projected(d, id) } else { Ok(value) })
            .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))
    }
    fn projected(
        self,
        d: &crate::FederalDraft,
        id: i64,
    ) -> Result<Self, crate::kernel::loss_limits::LossLimitError> {
        let project = |value| SourceAmount::property(d, id, value).filer;
        let gain = project(self.gain);
        let ordinary = project(self.ordinary);
        let high_end = self
            .ordinary
            .checked_add(self.unrecaptured)
            .ok_or(crate::kernel::loss_limits::LossLimitError::Overflow)?;
        Ok(Self {
            gain,
            recapture: project(self.recapture),
            ordinary,
            section1231: gain - ordinary,
            unrecaptured: project(high_end) - ordinary,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PropertySourceProjection {
    pub income: i64,
    pub component: Option<crate::asset_sale::DirectBusinessPropertyId>,
    pub whole: Option<PropertyCents>,
    pub filer: PropertyCents,
    pub counterpart: Option<PropertyCents>,
    // Native realized-gain and recapture detail operands, distinct from recognized gain.
    pub asset_detail: Option<[i64; 3]>,
    pub installment: bool,
    pub owner_reported: bool,
}
impl PropertySourceProjection {
    fn new(
        d: &crate::FederalDraft,
        income: i64,
        component: Option<crate::asset_sale::DirectBusinessPropertyId>,
        value: crate::farm_land::LandGain,
        physical: bool,
    ) -> Result<Self, crate::activity_risk::SourceIssue> {
        let whole = PropertyCents::annual(value)
            .map_err(|error| crate::activity_risk::SourceIssue::new(income, error.into()))?;
        Self::from_cents(d, income, component, whole, physical)
    }
    fn from_cents(
        d: &crate::FederalDraft,
        income: i64,
        component: Option<crate::asset_sale::DirectBusinessPropertyId>,
        whole: PropertyCents,
        physical: bool,
    ) -> Result<Self, crate::activity_risk::SourceIssue> {
        let filer = if physical {
            whole
                .projected(d, income)
                .map_err(|error| crate::activity_risk::SourceIssue::new(income, error.into()))?
        } else {
            whole
        };
        let counterpart = PropertyCents {
            gain: whole.gain - filer.gain,
            recapture: whole.recapture - filer.recapture,
            ordinary: whole.ordinary - filer.ordinary,
            section1231: whole.section1231 - filer.section1231,
            unrecaptured: whole.unrecaptured - filer.unrecaptured,
        };
        Ok(Self {
            income,
            component,
            whole: physical.then_some(whole),
            filer,
            counterpart: physical.then_some(counterpart),
            asset_detail: None,
            installment: false,
            owner_reported: false,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BusinessIncomeProjection {
    pub whole: Option<i64>,
    pub filer: i64,
    pub counterpart: Option<i64>,
}
impl From<SourceAmount> for BusinessIncomeProjection {
    fn from(value: SourceAmount) -> Self {
        Self { whole: Some(value.whole), filer: value.filer, counterpart: Some(value.counterpart) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BusinessSourceProjection {
    pub income: BusinessIncomeProjection,
    pub operator_earnings: i64,
    pub conservation_deductions: [i64; 2],
    pub qbi_income_cents: i64,
    pub qbi_wages_cents: i64,
    pub qbi_property_cents: i64,
}
impl BusinessSourceProjection {
    pub(crate) fn qualified_income(&self, d: &crate::FederalDraft, id: i64, amount: i64) -> i64 {
        SourceAmount::income(d, id, amount + self.conservation_deductions[0]).filer
            - self.conservation_deductions[1]
    }
    pub(crate) fn bind_qbi(
        &mut self,
        d: &crate::FederalDraft,
        id: i64,
        income: i64,
        wages: i64,
        property: i64,
    ) {
        self.qbi_income_cents = income;
        self.qbi_wages_cents = SourceAmount::income(d, id, wages).filer;
        self.qbi_property_cents = SourceAmount::income(d, id, property).filer;
    }
    pub(crate) fn produce(
        d: &crate::FederalDraft,
        id: i64,
        operator: i64,
        alternative: bool,
        claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Result<Self, crate::Section179PreparedIssue> {
        let mut income = BusinessIncomeProjection::from(SourceAmount::income(d, id, operator));
        let mut conservation_deductions = [0; 2];
        let conservation = d.income.iter().any(|entry| {
            entry.id == id
                && matches!(&entry.income,
            FederalIncome::Business { data } if matches!(&data.activity,
                crate::BusinessActivity::Farming { data } if data.conservation.is_some()))
        });
        if conservation && d.setup.as_ref().is_some_and(applies) {
            let unavailable = || {
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    id,
                    "WARN_FARM_CONSERVATION_GROSS_SOURCE",
                )
            };
            let operator_deduction = crate::farm_conservation::operator_deduction_with_claims(
                d,
                id,
                alternative,
                claims,
            )
            .ok_or_else(unavailable)?;
            let filer_deduction =
                crate::farm_conservation::income_deduction_with_claims(d, id, alternative, claims)
                    .ok_or_else(unavailable)?;
            conservation_deductions = [operator_deduction, filer_deduction];
            let other_deduction = crate::farm_conservation::counterpart_deduction_with_claims(
                d,
                id,
                alternative,
                claims,
            );
            let before = SourceAmount::income(d, id, operator + operator_deduction);
            income = BusinessIncomeProjection {
                whole: other_deduction.map(|other| before.whole - filer_deduction - other),
                filer: before.filer - filer_deduction,
                counterpart: other_deduction.map(|other| before.counterpart - other),
            };
        }
        Ok(Self {
            income,
            operator_earnings: operator,
            conservation_deductions,
            qbi_income_cents: income.filer,
            qbi_wages_cents: 0,
            qbi_property_cents: 0,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SourceProjection {
    pub alternative: bool,
    pub business: alloc::collections::BTreeMap<i64, BusinessSourceProjection>,
    pub direct: alloc::vec::Vec<PropertySourceProjection>,
    pub native_partial_gains: alloc::vec::Vec<(i64, i64)>,
    pub native_issue: Option<crate::activity_risk::SourceIssue>,
    pub components: alloc::collections::BTreeMap<
        i64,
        Result<alloc::vec::Vec<PropertySourceProjection>, &'static str>,
    >,
}
impl SourceProjection {
    pub(crate) fn produce(
        d: &crate::FederalDraft,
        pool: &crate::extended_rules::RentalResult,
        alternative: bool,
        claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Result<Self, crate::activity_risk::SourceIssue> {
        let (physical, mut parts, partial, native_issue) =
            crate::asset_sale::physical_allowed_direct_source_packet_with_claims(
                d,
                pool,
                alternative,
                claims,
            )?;
        let mut result = Self { alternative, native_issue, ..Default::default() };
        for (id, gain) in partial {
            let cents = crate::extended_rules::annual_dollars_to_working_cents(gain)
                .map_err(|error| crate::activity_risk::SourceIssue::new(id, error.into()))?;
            result.native_partial_gains.push((id, cents));
        }
        for (id, value) in physical {
            let business = d.income.iter().any(|entry| {
                entry.id == id && matches!(entry.income, FederalIncome::Business { .. })
            });
            result.direct.push(PropertySourceProjection::new(d, id, None, value, business)?);
        }
        for (&id, cached) in &pool.direct_business_property {
            parts.insert(id, Ok(cached.iter().map(|part| part.original).collect()));
        }
        for (id, parts) in parts {
            let rows = match parts {
                Ok(parts) => {
                    let mut rows = alloc::vec::Vec::new();
                    for part in parts {
                        let mut row = if let Some(cached) = pool
                            .direct_business_property
                            .get(&id)
                            .and_then(|rows| rows.iter().find(|row| row.original.id == part.id))
                        {
                            let value = PropertyCents::limited(cached).map_err(|error| {
                                crate::activity_risk::SourceIssue::new(id, error.into())
                            })?;
                            if matches!(
                                part.id,
                                crate::asset_sale::DirectBusinessPropertyId::Partial(_)
                            ) {
                                result.native_partial_gains.push((id, value.gain));
                            }
                            PropertySourceProjection::from_cents(d, id, Some(part.id), value, true)?
                        } else {
                            PropertySourceProjection::new(d, id, Some(part.id), part.dollars, true)?
                        };
                        row.owner_reported = pool.direct_business_property.contains_key(&id);
                        if let Some(native) = part.report.filter(|_| !row.owner_reported) {
                            let lift = crate::extended_rules::annual_dollars_to_working_cents;
                            let realized = native
                                .gain
                                .checked_sub(native.recognized())
                                .and_then(|gain| gain.checked_add(native.postponed))
                                .ok_or_else(|| {
                                    crate::activity_risk::SourceIssue::new(
                                        id,
                                        crate::activity_risk::ProjectionIssue::Source(
                                            "DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE",
                                        ),
                                    )
                                })?;
                            let mut details = [0; 3];
                            for (axis, amount) in
                                [realized, native.postponed, native.recapture_1245]
                                    .into_iter()
                                    .enumerate()
                            {
                                details[axis] = lift(amount).map_err(|error| {
                                    crate::activity_risk::SourceIssue::new(id, error.into())
                                })?;
                            }
                            row.asset_detail = Some(details);
                            row.installment = native.installment.is_some();
                        }
                        rows.push(row);
                    }
                    let rows = Self::project_components(d, id, rows);
                    rows.and_then(|rows| {
                        let mut totals = [0_i64; 5];
                        let mut whole = [0_i64; 5];
                        for row in &rows {
                            for (sum, amount) in whole.iter_mut().zip(
                                row.whole
                                    .ok_or("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED")?
                                    .columns(),
                            ) {
                                *sum = sum
                                    .checked_add(amount)
                                    .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
                            }
                            for (total, value) in totals.iter_mut().zip(row.filer.columns()) {
                                *total = total
                                    .checked_add(value)
                                    .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
                            }
                        }
                        if pool.direct_business_property.contains_key(&id) {
                            result.direct.push(
                                PropertySourceProjection::from_cents(
                                    d,
                                    id,
                                    None,
                                    PropertyCents::from_columns(whole),
                                    true,
                                )
                                .map_err(|_| "DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?,
                            );
                        }
                        if let Some(source) =
                            result.direct.iter_mut().find(|source| source.income == id)
                        {
                            source.filer.unrecaptured = totals[4];
                            if let Some(whole) = source.whole {
                                source.counterpart.as_mut().unwrap().unrecaptured =
                                    whole.unrecaptured - totals[4];
                            }
                            if source.filer.columns() != totals {
                                return Err("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE");
                            }
                        } else if totals != [0; 5] {
                            return Err("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE");
                        }
                        Ok(rows)
                    })
                }
                Err(key) => Err(key),
            };
            if pool.direct_business_property.contains_key(&id) {
                rows.as_ref().map_err(|key| {
                    crate::activity_risk::SourceIssue::new(
                        id,
                        crate::activity_risk::ProjectionIssue::Source(key),
                    )
                })?;
            }
            result.components.insert(id, rows);
        }
        Ok(result)
    }
    fn project_components(
        d: &crate::FederalDraft,
        id: i64,
        mut parts: alloc::vec::Vec<PropertySourceProjection>,
    ) -> Result<alloc::vec::Vec<PropertySourceProjection>, &'static str> {
        parts.sort_by_key(|part| part.component);
        let mut checked = [0_i64; 5];
        let mut checked_details = [0_i64; 3];
        for part in &parts {
            if let Some(details) = part.asset_detail {
                for (total, amount) in checked_details.iter_mut().zip(details) {
                    *total = total
                        .checked_add(amount)
                        .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
                }
            }
            for (total, amount) in checked
                .iter_mut()
                .zip(part.whole.ok_or("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED")?.columns())
            {
                *total = total
                    .checked_add(amount)
                    .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
            }
        }
        let mut section_prefix = 0_i64;
        for part in &parts {
            section_prefix = section_prefix
                .checked_add(part.whole.unwrap().section1231)
                .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
            checked[2]
                .checked_add(section_prefix)
                .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
        }
        let mut prefixes: [crate::kernel::ratio::Prefix; 5] =
            core::array::from_fn(|_| Default::default());
        let mut details: [crate::kernel::ratio::Prefix; 2] =
            core::array::from_fn(|_| Default::default());
        let mut section_start = checked[2];
        for part in &mut parts {
            let whole = part.whole.ok_or("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED")?.columns();
            let mut filer = core::array::from_fn(|axis| {
                prefixes[axis].next(whole[axis], |amount| {
                    if axis == 3 {
                        SourceAmount::property(d, id, checked[2] + amount).filer
                            - SourceAmount::property(d, id, checked[2]).filer
                    } else {
                        SourceAmount::property(d, id, amount).filer
                    }
                })
            });
            let high_end = section_start
                .checked_add(whole[4])
                .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
            filer[4] = SourceAmount::property(d, id, high_end).filer
                - SourceAmount::property(d, id, section_start).filer;
            section_start = section_start
                .checked_add(whole[3])
                .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
            filer[0] = filer[2]
                .checked_add(filer[3])
                .ok_or("DIRECT_ACTIVITY_PROPERTY_SOURCE_UNAVAILABLE")?;
            part.filer = PropertyCents::from_columns(filer);
            if let Some(native) = part.asset_detail {
                let projected = core::array::from_fn::<_, 2, _>(|axis| {
                    details[axis]
                        .next(native[axis], |amount| SourceAmount::property(d, id, amount).filer)
                });
                let recapture1245 = crate::kernel::ratio::allocate_remaining(
                    part.filer.recapture,
                    &[native[2], whole[1] - native[2]],
                )[0];
                part.asset_detail = Some([projected[0], projected[1], recapture1245]);
            }
            part.counterpart = Some(PropertyCents::from_columns(core::array::from_fn(|axis| {
                whole[axis] - filer[axis]
            })));
        }
        Ok(parts)
    }
    pub(crate) fn bind_business(
        pool: &mut crate::extended_rules::RentalResult,
        sources: &alloc::collections::BTreeMap<i64, [BusinessSourceProjection; 2]>,
        alternative: bool,
    ) {
        if let Some(packet) = &mut pool.source_projection
            && let Ok(projection) = alloc::sync::Arc::make_mut(packet)
        {
            projection.business =
                sources.iter().map(|(id, rows)| (*id, rows[usize::from(alternative)])).collect();
        }
    }
    pub(crate) fn report(
        &self,
        d: &crate::FederalDraft,
        result: &mut crate::FederalResult,
    ) -> Result<(), crate::activity_risk::SourceIssue> {
        if self.alternative || !d.setup.as_ref().is_some_and(applies) {
            return Ok(());
        }
        if let Some(issue) = &self.native_issue {
            return Err(issue.clone());
        }
        for entry in &d.income {
            let FederalIncome::Business { data } = &entry.income else {
                continue;
            };
            let issue = |key| {
                crate::activity_risk::SourceIssue::new(
                    entry.id,
                    crate::activity_risk::ProjectionIssue::Source(key),
                )
            };
            let Some(parts) = self.components.get(&entry.id) else {
                continue;
            };
            let parts = parts.as_ref().map_err(|key| issue(key))?;
            for part in parts {
                // The actual limited owner already settled these annual report
                // rows; working cents above remain separate from that boundary.
                if part.owner_reported {
                    continue;
                }
                use crate::asset_sale::DirectBusinessPropertyId as Id;
                let (prefix, partial, land_long) = match part
                    .component
                    .ok_or_else(|| issue("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED"))?
                {
                    Id::Asset(id) | Id::Partial(id) => {
                        let index =
                            data.assets.iter().position(|asset| asset.id == id).ok_or_else(
                                || issue("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED"),
                            )?;
                        (
                            alloc::format!("business_{}_asset_{index}", entry.id),
                            matches!(part.component, Some(Id::Partial(_))),
                            false,
                        )
                    }
                    Id::Land { sale, parcel } => {
                        let crate::BusinessActivity::Farming { data: farm } = &data.activity else {
                            return Err(issue("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED"));
                        };
                        let index = farm
                            .land_sales
                            .iter()
                            .position(|row| row.id == sale && row.basis_property == parcel)
                            .ok_or_else(|| issue("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED"))?;
                        let parcel =
                            farm.land_basis.iter().find(|row| row.id == parcel).ok_or_else(
                                || issue("DIRECT_BUSINESS_PROPERTY_IDENTITY_REQUIRED"),
                            )?;
                        (
                            alloc::format!("farm_{}_land_{index}", entry.id),
                            false,
                            parcel.acquisition == crate::FarmLandAcquisition::Inherited
                                || crate::dates::held_more_than_one_year(
                                    parcel.acquired,
                                    farm.land_sales[index].sold,
                                ),
                        )
                    }
                };
                let value = part.filer;
                let mut set = |suffix: &str, amount| {
                    let id = alloc::format!("{prefix}_{suffix}");
                    if let Some(row) = result.lines.iter_mut().find(|row| row.id == id) {
                        row.amount = amount;
                    }
                };
                if partial {
                    set("partial_gain", value.gain);
                    continue;
                }
                if let Some([realized, postponed, recapture1245]) = part.asset_detail {
                    set("disposal_gain", value.gain + realized);
                    set("disposal_postponed", postponed);
                    set(
                        "disposal_ordinary",
                        if part.installment { value.recapture } else { value.ordinary },
                    );
                    set("disposal_section1231", value.section1231);
                    set("disposal_1245_recapture", recapture1245);
                    set("disposal_1250_recapture", value.recapture - recapture1245);
                    set("disposal_installment_ordinary", value.ordinary - value.recapture);
                    set("disposal_installment_gain", value.section1231);
                    set("disposal_form6252_recapture", value.recapture);
                    set("disposal_form6252_income", value.gain - value.recapture);
                    set("disposal_form6252_ordinary", value.ordinary - value.recapture);
                    set("disposal_form6252_gain", value.section1231);
                    set("disposal_form6252_unrecaptured_1250", value.unrecaptured);
                } else {
                    set("gain", value.gain);
                    set("ordinary", value.ordinary);
                    set("section1231", value.section1231);
                    set("unrecaptured", value.unrecaptured);
                    set("conservation_recapture", value.recapture);
                    set(
                        "cost_sharing_recapture",
                        if land_long { value.ordinary - value.recapture } else { 0 },
                    );
                    set("form6252_recapture", value.recapture);
                    set("form6252_income", value.gain - value.recapture);
                    set("form6252_ordinary", value.ordinary - value.recapture);
                    set("form6252_gain", value.section1231);
                }
            }
            if let Some(source) = self.direct.iter().find(|source| source.income == entry.id)
                && let Some(whole) = source.whole
            {
                let label = crate::interview::income_display_arg(entry);
                for (suffix, amount, key) in [
                    ("total", whole.gain, "EXPLAIN_FORM8958_GAIN_N_TOTAL"),
                    ("this_return", source.filer.gain, "EXPLAIN_FORM8958_GAIN_N_THIS_RETURN"),
                    (
                        "ordinary_this_return",
                        source.filer.ordinary,
                        "EXPLAIN_FORM8958_GAIN_ORDINARY",
                    ),
                    (
                        "section1231_this_return",
                        source.filer.section1231,
                        "EXPLAIN_FORM8958_GAIN_SECTION1231",
                    ),
                ] {
                    crate::line_emission::push_line(
                        result,
                        Some(crate::FormLine::new(crate::TaxForm::Form8958, "6")),
                        &alloc::format!("form8958_gain_{}_{}", entry.id, suffix),
                        amount,
                        &["income", "setup.community_property"],
                        "https://www.irs.gov/pub/irs-pdf/f8958.pdf",
                        crate::Message::new(key, alloc::vec![label.clone()]),
                    );
                }
            }
        }
        Ok(())
    }
    pub(crate) fn cent_direct(&self) -> alloc::vec::Vec<(i64, PropertyCents)> {
        self.direct.iter().map(|row| (row.income, row.filer)).collect()
    }
}

pub(crate) fn source_projection<'a>(
    d: &crate::FederalDraft,
    pool: &'a crate::extended_rules::RentalResult,
    alternative: bool,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<alloc::borrow::Cow<'a, SourceProjection>, crate::activity_risk::SourceIssue> {
    if let Some(result) = &pool.source_projection {
        return match result.as_ref() {
            Ok(value) if value.alternative == alternative => {
                Ok(alloc::borrow::Cow::Borrowed(value))
            }
            Err(issue) => Err(issue.clone()),
            _ => SourceProjection::produce(d, pool, alternative, claims)
                .map(alloc::borrow::Cow::Owned),
        };
    }
    SourceProjection::produce(d, pool, alternative, claims).map(alloc::borrow::Cow::Owned)
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_enum!(Answer { records: [], units: [Yes, No, NotSure], tuples: [] });
