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
    CommunityProperty, CommunityRule, CommunityShare, CommunityState, FederalIncome, FederalSetup,
    FederalStatus, Owner, RetirementKind,
};

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
        FederalIncome::Retirement { data } if data.kind == RetirementKind::Ira => Kind::Owners,
        FederalIncome::Roth { .. }
        | FederalIncome::InheritedIra { .. }
        | FederalIncome::SocialSecurity { .. }
        | FederalIncome::EducationDistribution { .. }
        | FederalIncome::EducationAccountExpenses { .. }
        | FederalIncome::MedicalDistribution { .. }
        | FederalIncome::Able { .. }
        | FederalIncome::TrumpDeath { .. }
        | FederalIncome::TrumpAbleRollover { .. }
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

// What the filer reports of one income item on a community-property return:
// None when the return divides nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Share {
    // The owner's part as a fraction; the other spouse reports the rest.
    owner_num: i64,
    owner_den: i64,
    // Property gains follow ownership independently of separate-property income.
    gain_num: i64,
    gain_den: i64,
    // The filer owns the document.
    filer_owns: bool,
}

impl Share {
    // The whole item is the filer's: nothing is divided.
    pub(crate) fn whole(&self) -> bool {
        self.filer_owns && self.owner_num == self.owner_den && self.gain_num == self.gain_den
    }
    // The filer owns the document (and so the operator's own deductions).
    pub(crate) fn filer_owns(&self) -> bool {
        self.filer_owns
    }
    // Nothing of the item is the filer's.
    pub(crate) fn none(&self) -> bool {
        !self.filer_owns && self.owner_num == self.owner_den && self.gain_num == self.gain_den
    }
    // The filer's part of `amount`; the owner keeps an odd cent so the two
    // columns of Form 8958 add to the total.
    pub(crate) fn filer(&self, amount: i64) -> i64 {
        let owners = crate::kernel::ratio::fraction(amount, self.owner_num, self.owner_den);
        if self.filer_owns { owners } else { amount - owners }
    }
    // Preserve the owner-first cent boundary and the other spouse's exact
    // residual without narrowing an intermediate source expense total.
    pub(crate) fn filer_exact(
        &self,
        amount: &crate::kernel::ratio::exact::Exact,
    ) -> crate::kernel::ratio::exact::Exact {
        use crate::kernel::ratio::exact;
        let owners = exact::rounded_exact(
            &(amount * exact::ratio(self.owner_num, self.owner_den).expect("owner denominator")),
        );
        if self.filer_owns { owners } else { amount - owners }
    }
    pub(crate) fn gain(&self, amount: i64) -> i64 {
        self.for_gains().filer(amount)
    }
    pub(crate) fn for_gains(&self) -> Self {
        Self { owner_num: self.gain_num, owner_den: self.gain_den, ..*self }
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

// The field lists beside each source type classify money, property gains and
// facts exhaustively. A newly added field must be classified before compilation
// can succeed. The only arithmetic is Share's shared-kernel projection.
pub(crate) trait Divide: Clone {
    fn divided(&self, share: Share) -> Self;
}
impl Divide for i64 {
    fn divided(&self, share: Share) -> Self {
        share.filer(*self)
    }
}
impl<T: Divide> Divide for alloc::boxed::Box<T> {
    fn divided(&self, share: Share) -> Self {
        alloc::boxed::Box::new((**self).divided(share))
    }
}
impl<T: Divide> Divide for Option<T> {
    fn divided(&self, share: Share) -> Self {
        self.as_ref().map(|value| value.divided(share))
    }
}
impl<T: Divide> Divide for alloc::vec::Vec<T> {
    fn divided(&self, share: Share) -> Self {
        self.iter().map(|value| value.divided(share)).collect()
    }
}
macro_rules! divide_record {
    ($ty:ty { amounts: [$($amount:ident),* $(,)?], gains: [$($gain:ident),* $(,)?], facts: [$($fact:ident),* $(,)?] $(,)? }) => {
        impl crate::community::Divide for $ty {
            fn divided(&self, share: crate::community::Share) -> Self {
                let _ = share;
                let Self { $($amount,)* $($gain,)* $($fact,)* } = self;
                Self {
                    $($amount: crate::community::Divide::divided($amount, share),)*
                    $($gain: crate::community::Divide::divided($gain, share.for_gains()),)*
                    $($fact: $fact.clone(),)*
                }
            }
        }
    };
}
pub(crate) use divide_record;
macro_rules! divide_enum {
    ($ty:ident {
        records: [$($variant:ident { amounts: [$($amount:ident),* $(,)?], facts: [$($fact:ident),* $(,)?] }),* $(,)?],
        units: [$($unit:ident),* $(,)?],
        tuples: [$($tuple:ident),* $(,)?] $(,)?
    }) => {
        impl crate::community::Divide for $ty {
            fn divided(&self, share: crate::community::Share) -> Self {
                let _ = share;
                match self {
                    $(Self::$variant { $($amount,)* $($fact,)* } => Self::$variant {
                        $($amount: crate::community::Divide::divided($amount, share),)*
                        $($fact: $fact.clone(),)*
                    },)*
                    $(Self::$unit => Self::$unit,)*
                    $(Self::$tuple(value) => Self::$tuple(crate::community::Divide::divided(value, share)),)*
                }
            }
        }
    };
}
pub(crate) use divide_enum;

// Borrow the accepted source when it is already wholly this return's. A divided
// value is transient; accepted answers, identity and the normal lifecycle stay intact.
pub(crate) fn record<'a>(
    d: &crate::FederalDraft,
    entry: &'a crate::FederalIncomeEntry,
) -> alloc::borrow::Cow<'a, FederalIncome> {
    let Some(share) = d
        .setup
        .as_ref()
        .and_then(|setup| share(setup, entry.owner, entry.community, &entry.income))
    else {
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
        FederalIncome::Scorp { data } => {
            let mut data = data.divided(share);
            data.credits.retain(|credit| credit.amount != 0);
            data.investment_property.retain(|property| match property {
                crate::K1InvestmentProperty::RehabilitationRental { expenditures, .. }
                | crate::K1InvestmentProperty::RehabilitationOther { expenditures, .. } => {
                    *expenditures != 0
                }
                crate::K1InvestmentProperty::Energy { basis, .. }
                | crate::K1InvestmentProperty::CleanElectricity { basis, .. } => *basis != 0,
            });
            FederalIncome::Scorp { data }
        }
        FederalIncome::EstateTrustPortfolio { data } => {
            let mut data = data.divided(share);
            data.reported_credits.retain(|credit| credit.amount != 0);
            data.investment_property.retain(|item| match &item.property {
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
    let Some(share) = d
        .setup
        .as_ref()
        .and_then(|setup| share(setup, entry.owner, entry.community, &entry.income))
    else {
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

// The share of the tax withheld on an item. It follows the income it was
// withheld from (Reg. 1.31-1(a)): a Form 1099 whose amount is routed to a
// business is withheld from that business's earnings, which section 879(a)
// gives to the spouse who earned them, whatever share the document's other
// boxes divide by. Any other item's withholding divides like the item.
pub(crate) fn withholding_share(
    setup: &FederalSetup,
    owner: Owner,
    answer: CommunityShare,
    income: &FederalIncome,
) -> Option<Share> {
    share_of_kind(setup, owner, answer, withholding_kind(income))
}

fn withholding_kind(income: &FederalIncome) -> Kind {
    let routed_to_business = crate::information_returns::flows(income)
        .iter()
        .any(|flow| flow.kind == crate::information_returns::Kind::Business);
    if routed_to_business { Kind::Earned } else { kind(income) }
}

// R8 reuses the source character of the community owner (approved R8 and
// whole-return-situations 3D), not a non-W2-as-unearned inference. A mixed
// K1/otherwise unclassified source remains unknown in regimes needing it.
pub(crate) fn refund_earned(income: &FederalIncome) -> Option<bool> {
    match withholding_kind(income) {
        Kind::Earned => Some(true),
        Kind::Owners | Kind::Property => Some(false),
        Kind::Other if matches!(income, FederalIncome::Sale { .. }) => Some(false),
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
    let Some(share) = d
        .setup
        .as_ref()
        .and_then(|setup| share(setup, entry.owner, entry.community, &entry.income))
    else {
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
        property_source.ordinary + property_items.k1().box_one(),
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
    let (short, long) = data.separate.as_ref().map_or((0, 0), |v| (v.short, v.long));
    Some([
        crate::kernel::phase::floor(short, 0)
            - ledger.risk_allowed[crate::k1_loss::K1LossClass::ShortTerm].get(),
        crate::kernel::phase::floor(long, 0)
            - ledger.risk_allowed[crate::k1_loss::K1LossClass::LongTerm].get(),
    ])
}
pub(crate) fn partner_facts<'a>(
    d: &crate::FederalDraft,
    entry: &'a crate::FederalIncomeEntry,
    data: &crate::PartnerK1,
) -> crate::k1_totals::Facts<'a> {
    let mut facts = data.totals_facts(entry);
    for (source, flows, disposal) in [
        (&data.regular, &mut facts.flows, &mut facts.disposal),
        (
            data.alternative.as_ref().unwrap_or(&data.regular),
            &mut facts.amt_flows,
            &mut facts.amt_disposal,
        ),
    ] {
        if let Ok(items) = partner_items(d, entry.id, data, source) {
            (*flows, *disposal) = data.outcome_with_items(source, &items);
        }
    }
    facts
}

fn scorp_event_source(
    d: &crate::FederalDraft,
    id: i64,
    amt: bool,
) -> Option<crate::scorp_amt::ScorpSource<'static>> {
    let entry = d.income.iter().find(|entry| entry.id == id)?;
    let share = d
        .setup
        .as_ref()
        .and_then(|setup| share(setup, entry.owner, entry.community, &entry.income))?;
    if share.owner_num * share.gain_den == share.gain_num * share.owner_den {
        return None;
    }
    let FederalIncome::Scorp { data: whole } = &entry.income else {
        return None;
    };
    let property = whole.divided(share.for_gains());
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
    let share = d
        .setup
        .as_ref()
        .and_then(|setup| share(setup, entry.owner, entry.community, &entry.income))?;
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
        .and_then(|entry| {
            d.setup
                .as_ref()
                .and_then(|setup| share(setup, entry.owner, entry.community, &entry.income))
        })
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
        .and_then(|entry| {
            d.setup
                .as_ref()
                .and_then(|setup| share(setup, entry.owner, entry.community, &entry.income))
        })
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
    d: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Option<i64> {
    Some(match &entry.income {
        FederalIncome::Rental { .. } => crate::extended_rules::whole_rental_net(d, entry).ok()?,
        FederalIncome::FarmRental { data } => data
            .net(
                d,
                usize::from(entry.owner == Owner::Spouse),
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
            data.ordinary + data.rentals.real_estate + data.rentals.other
        }
        FederalIncome::EstateTrustPortfolio { data } => {
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
pub(crate) fn partner_se_lines_with_section179(
    d: &crate::FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    own: (i64, &alloc::collections::BTreeMap<i64, i64>),
    counterpart_payroll: [i64; 2],
    qbi_loss_sources: &alloc::collections::BTreeSet<i64>,
    selected: Option<&crate::Section179PreparedAllocation>,
    result: &mut crate::FederalResult,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<(), crate::Section179PreparedIssue> {
    use crate::kernel::lines::{LineValue, emit, row};
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
            let actual = match crate::partnership::owner_actual_se_with_section179(
                d,
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
            let clergy = crate::clergy::calculate_with_section179(d, owner, selected, home)?;
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
        let Some(share) = share(setup, entry.owner, entry.community, &entry.income) else {
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
pub(crate) fn explain(d: &crate::FederalDraft, result: &mut crate::FederalResult) {
    use crate::kernel::lines::{LineValue, emit, row};
    let Some(setup) = &d.setup else { return };
    if !applies(setup) || result.state != crate::ResultState::EstimatedForSupportedScope {
        return;
    }
    for entry in &d.income {
        if !self_divided(&entry.income) {
            continue;
        }
        let Some(share) = share(setup, entry.owner, entry.community, &entry.income) else {
            continue;
        };
        let label = crate::interview::income_display_arg(entry);
        let Some(total) = item_amount(d, entry) else {
            crate::extended_rules::expense_output_unavailable(result, Some(entry.id));
            return;
        };
        let (withholding, mut portfolio_gain, foreign_taxes) = match &entry.income {
            FederalIncome::Partnership { data } => data
                .separate
                .as_ref()
                .map_or((0, 0, 0), |v| (v.backup_withholding, v.short + v.long, v.foreign_taxes)),
            FederalIncome::Scorp { data } => (
                data.withholding,
                data.capital.as_ref().map_or(0, |v| v.short + v.long),
                data.foreign_taxes,
            ),
            FederalIncome::EstateTrustPortfolio { data } => {
                (data.backup_withholding, 0, data.foreign_statement.as_ref().map_or(0, |v| v.tax))
            }
            _ => (0, 0, 0),
        };
        // The source gain column also includes coded capital/1256 items and
        // current entity property amounts before this return's limits/lookback.
        // Section1250 rate components are already included and never added twice.
        portfolio_gain += match &entry.income {
            FederalIncome::Partnership { data } => {
                let flows = data.k1_items().flows_whole();
                flows.capital().iter().sum::<i64>()
                    + flows.section1256
                    + data
                        .regular
                        .business_property(&data.items())
                        .ok()
                        .flatten()
                        .map_or(0, |property| property.net())
                    + data.regular.rentals.real_estate_property.net
                    + data.regular.rentals.other_property.net
            }
            FederalIncome::Scorp { data } => {
                let flows = data.k1_items().flows_whole();
                flows.capital().iter().sum::<i64>()
                    + flows.section1256
                    + data.business_property().ok().flatten().map_or(0, |property| property.net())
                    + data.rentals.real_estate_property.net
                    + data.rentals.other_property.net
            }
            _ => 0,
        };
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
                data.credits.iter().map(|credit| credit.amount).sum::<i64>()
                    + data
                        .investment_property
                        .iter()
                        .map(|property| property.current_year_credit() * 100)
                        .sum::<i64>()
            }
            FederalIncome::EstateTrustPortfolio { data } => {
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
            FederalIncome::Scorp { data } => [data.interest, data.dividends, data.royalties],
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
            FederalIncome::Scorp { data } => data
                .deductions
                .iter()
                .filter(|deduction| deduction.code == crate::ScorpDeductionCode::RoyaltyDeductions)
                .map(|deduction| deduction.amount)
                .sum::<i64>(),
            _ => 0,
        };
        let property_gain = match &entry.income {
            FederalIncome::Rental { data } => {
                match crate::asset_sale::rental_sale_amount(
                    data,
                    crate::depreciation::mid_quarter(d),
                    false,
                ) {
                    Ok(value) => value,
                    Err(issue) => {
                        issue.report(entry.id, result);
                        return;
                    }
                }
            }
            FederalIncome::FarmRental { data } => {
                let land = crate::farm_land::source(d, entry.id, &data.farm, false)
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
                let disposal = data.outcome(&data.regular).1;
                disposal.sale_gain() + disposal.distribution_gain
            }
            FederalIncome::Scorp { data } => data.basis_result().ok().map_or(0, |basis| {
                let disposal = data.disposal(&basis);
                let loans = data.loan_gains(&basis).unwrap_or([0; 3]);
                disposal.sale_gain() + disposal.distribution_gain + loans[1] + loans[2]
            }),
            _ => 0,
        };
        // Distributive-share capital items follow the income fraction;
        // external property gains follow the independent property fraction.
        let gain = portfolio_gain + property_gain;
        let gain_share = share.filer(portfolio_gain) + share.gain(property_gain);
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
