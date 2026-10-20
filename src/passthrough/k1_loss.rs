// Coverage: Form 7203 Part III and the partner basis worksheet Part II — Supported for the loss
//   classes below (unit B8, 2026-10-10; i7203 2022, i1065sk1 2025, i1120ssk 2026 draft of 9/18/26,
//   i6198 Nov. 2025, Pub 925, 26 CFR 1.704-1(d), 1.1366-2(a), 1.179-3(h), 1.469-2T(d)).
//   Lines: the loss class catalog (K1LossClass: ordinary, the two rental classes, short- and
//   long-term capital, collectibles, section 1231, section 179, cash gifts, foreign taxes), the
//   per-K-1 ledger of what each limit allowed and held back (LossLedger: basis, then at-risk;
//   the passive stage follows in the Form 8582 pool), and the stored carryover records of the
//   classes that had none (K1LossCarry, K1LossCarryforwards, PartnerCapitalCarry). One pro rata
//   allocation shares each limit among the current share and the carried amount of every class
//   (Reg. 1.704-1(d)(2), 1.1366-2(a)(5)); a held-back amount keeps its character and is
//   incurred again in the next year (Reg. 1.1366-2(a)(3), section 704(d)(2)).
//   Adding a class: one K1LossClass member, its place in ALL, at_risk() and key(), and its
//   resource name K1_LOSS_CLASS_<KEY>; the ledger, both stages and the notes read ALL.
//   Evidence: k1_loss.rs, partnership.rs (calculate_with), scorp_risk.rs (calculate),
//   partnership_basis.rs, scorp_basis.rs; tests/passthrough/k1_loss.rs.
//   R9 sale AMT comparison uses each tax system's source-item allocation.
//   Its Form 4797 sale-loss share has the existing disposition adjustment;
//   preferred-stock and nonportfolio capital code losses use that same
//   existing Schedule D adjustment; other residual codes retain the refusal.
//   Evidence: tests/passthrough/k1_sale_amt.rs; i6251 lines 2k, 2m and 2n (2025 final).
use crate::kernel::carry::{Amount, Cents};
use crate::kernel::loss_limits::{LossLimitError, risk};
use crate::kernel::phase;
#[cfg(test)]
use crate::kernel::ratio::dollars;
use crate::kernel::source_fact::SourceFact;
use crate::scorp_basis::BasisError;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Form 7203 Part III line order; also the allocation order (rule 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum K1LossClass {
    // 7203 line 35; worksheet 15a; Form 6198 line 1
    Ordinary,
    // 36; 15b; line 4
    RentalRealEstate,
    // 37; 15c; line 4
    OtherRental,
    // 38; 15e; line 2a
    ShortTerm,
    // 38; 15f (S corporation: other than collectibles); line 2a
    LongTerm,
    // 38, S corporation only; line 2a
    Collectibles,
    // 39 (S corporation), partner worksheet 15g; line 2b
    Section1231,
    // 40; 15h; line 2a/2b (box 11 codes C, Q, R, S and box 10 codes C, S
    // losses, unit B9-1)
    OtherLoss,
    // 41; 15i; outside the Form 6198 stage (rule 7)
    Section179,
    // 42; 15j; not an activity deduction
    Charity,
    // 42; 15j; the 30%, 50% and 20% classes of box 13 (12) B through G;
    // not an activity deduction
    OtherCharity,
    // 43; 15k; Form 6198 line 4 after Form 4952
    InvestmentInterest,
    // 44-45; 15l, 15o, 15p; the other box 13 (12) codes and the gambling
    // losses
    OtherDeductions,
    // 46; 15d; not an activity deduction
    ForeignTaxes,
}

impl K1LossClass {
    // The catalog in allocation order. A class is one entry here and in the
    // two matches below; its slot is its position, which the test
    // `catalog_slots_are_positions` keeps true.
    pub(crate) const ALL: [Self; 14] = [
        Self::Ordinary,
        Self::RentalRealEstate,
        Self::OtherRental,
        Self::ShortTerm,
        Self::LongTerm,
        Self::Collectibles,
        Self::Section1231,
        Self::OtherLoss,
        Self::Section179,
        Self::Charity,
        Self::OtherCharity,
        Self::InvestmentInterest,
        Self::OtherDeductions,
        Self::ForeignTaxes,
    ];
    pub(crate) const COUNT: usize = Self::ALL.len();

    const fn slot(self) -> usize {
        self as usize
    }

    // Whether the class is a loss item of the K-1's one Form 6198 activity.
    pub(crate) const fn at_risk(self) -> bool {
        !matches!(self, Self::Section179 | Self::Charity | Self::OtherCharity | Self::ForeignTaxes)
    }

    // Resource suffix of the class name (K1_LOSS_CLASS_<KEY>) in a note.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Ordinary => "K1_LOSS_CLASS_ORDINARY",
            Self::RentalRealEstate => "K1_LOSS_CLASS_RENTAL_REAL_ESTATE",
            Self::OtherRental => "K1_LOSS_CLASS_OTHER_RENTAL",
            Self::ShortTerm => "K1_LOSS_CLASS_SHORT_TERM",
            Self::LongTerm => "K1_LOSS_CLASS_LONG_TERM",
            Self::Collectibles => "K1_LOSS_CLASS_COLLECTIBLES",
            Self::Section1231 => "K1_LOSS_CLASS_SECTION1231",
            Self::OtherLoss => "K1_LOSS_CLASS_OTHER_LOSS",
            Self::Section179 => "K1_LOSS_CLASS_SECTION179",
            Self::Charity => "K1_LOSS_CLASS_CHARITY",
            Self::OtherCharity => "K1_LOSS_CLASS_OTHER_CHARITY",
            Self::InvestmentInterest => "K1_LOSS_CLASS_INVESTMENT_INTEREST",
            Self::OtherDeductions => "K1_LOSS_CLASS_OTHER_DEDUCTIONS",
            Self::ForeignTaxes => "K1_LOSS_CLASS_FOREIGN_TAXES",
        }
    }

    // The classes that already report their own 2027 carries (ordinary on
    // both kinds; an S corporation's capital and section 1231 classes) keep
    // those notes; every other class gets NOTE_K1_LOSS_CARRYFORWARD.
    pub(crate) const fn has_own_note(self, partner: bool) -> bool {
        match self {
            Self::Ordinary => true,
            Self::ShortTerm | Self::LongTerm | Self::Collectibles | Self::Section1231 => !partner,
            _ => false,
        }
    }
}

// Actual nonnegative source cents by class. Unit-bearing cells make old
// annual-dollar producers fail at this boundary instead of changing scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LossAmounts([Amount<Cents>; K1LossClass::COUNT]);

impl Index<K1LossClass> for LossAmounts {
    type Output = Amount<Cents>;
    fn index(&self, class: K1LossClass) -> &Self::Output {
        &self.0[class.slot()]
    }
}
impl IndexMut<K1LossClass> for LossAmounts {
    fn index_mut(&mut self, class: K1LossClass) -> &mut Self::Output {
        &mut self.0[class.slot()]
    }
}
impl LossAmounts {
    pub(crate) fn of(items: &[(K1LossClass, Amount<Cents>)]) -> Self {
        let mut amounts = Self::default();
        for (class, amount) in items {
            amounts[*class] = Amount::new(amounts[*class].get() + amount.get())
                .expect("validated class source total");
        }
        amounts
    }
    pub(crate) fn as_slice(&self) -> &[Amount<Cents>] {
        &self.0
    }
    pub(crate) fn from_slice(values: &[Amount<Cents>]) -> Self {
        let mut amounts = Self::default();
        for (slot, value) in amounts.0.iter_mut().zip(values) {
            *slot = *value;
        }
        amounts
    }
    pub(crate) fn total(&self) -> Amount<Cents> {
        Amount::new(self.0.iter().map(Amount::get).sum()).expect("validated class total")
    }
    pub(crate) fn any(&self) -> bool {
        self.0.iter().any(|v| v.get() != 0)
    }
    pub(crate) fn valid(&self) -> bool {
        true // Amount<Cents> establishes nonnegativity at construction.
    }
    pub(crate) fn plus(&self, other: &Self) -> Self {
        let mut out = *self;
        for (slot, value) in out.0.iter_mut().zip(other.0) {
            *slot = Amount::new(slot.get() + value.get()).expect("validated class source total");
        }
        out
    }
    pub(crate) fn minus(&self, other: &Self) -> Self {
        let mut out = *self;
        for (slot, value) in out.0.iter_mut().zip(other.0) {
            *slot = crate::kernel::carry::closing_values::<Cents>(slot.get(), value.get())
                .expect("allowed class amount is bounded by its source");
        }
        out
    }
    // Proportional allocation keeps the existing catalog order and algorithm.
    pub(crate) fn allocate(&self, amount: Amount<Cents>) -> Self {
        let amount = phase::cap(amount.get(), [self.total().get()]);
        let weights: Vec<_> = self.0.iter().map(Amount::get).collect();
        let values: Vec<_> = crate::kernel::ratio::allocate_remaining(amount, &weights)
            .into_iter()
            .map(|v| Amount::new(v).expect("nonnegative class allocation"))
            .collect();
        Self::from_slice(&values)
    }
    #[cfg(test)]
    pub(crate) fn limit(&self, amount: Amount<Cents>) -> (Self, Self) {
        let allowed = self.allocate(amount);
        (allowed, self.minus(&allowed))
    }
}

// What one K-1 brings to the loss limits in one tax system, by class: this
// year's share, the amount carried by the basis limit and the amount carried
// by the at-risk limit (which already passed basis). Actual source cents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ClassInput {
    pub current: LossAmounts,
    pub basis_carry: LossAmounts,
    pub risk_carry: LossAmounts,
}

impl ClassInput {
    // The amounts the basis stage allocates: current plus carried.
    pub(crate) fn basis(&self) -> LossAmounts {
        self.current.plus(&self.basis_carry)
    }

    // Whether the stored record carries anything a new activity cannot have.
    pub(crate) fn carries(&self) -> bool {
        self.basis_carry.any() || self.risk_carry.any()
    }
}

// One tax system's result for one K-1: what each limit allowed and held back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LossLedger {
    pub basis_allowed: LossAmounts,
    pub basis_suspended: LossAmounts,
    // The basis-allowed amount and the at-risk carry, after Form 6198.
    pub risk_allowed: LossAmounts,
    // Zero for the classes outside Form 6198.
    pub risk_suspended: LossAmounts,
    // Nondeductible expenses the at-risk limit holds back: not a loss class,
    // but a balance of the same stage.
    pub nondeductible_suspended: Amount<Cents>,
    // The third limit: what the passive activity rules hold back of each class
    // (Form 8582, or a publicly traded partnership's own limit under section
    // 469(k)). The pool fills it after its allocation; the two stages above
    // never read it, because a class reaches the passive stage only as far as
    // the at-risk stage allowed it.
    pub passive_suspended: LossAmounts,
}

impl LossLedger {
    // Whether a basis or at-risk limit holds back any class this year.
    pub(crate) fn limited(&self) -> bool {
        self.basis_suspended.any()
            || self.risk_suspended.any()
            || self.nondeductible_suspended.get() != 0
    }

    // Whether any of the three limits holds back any class this year.
    pub(crate) fn held_back(&self) -> bool {
        self.limited() || self.passive_suspended.any()
    }
}

// The at-risk stage over the classes (Form 6198 Parts I and IV): the
// basis-allowed amounts plus the at-risk carries of every at-risk class and
// the nondeductible expenses share the income and the amount at risk. The
// classes outside the activity pass through unchanged.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RiskStage {
    pub allowed: LossAmounts,
    pub suspended: LossAmounts,
    pub nondeductible_allowed: Amount<Cents>,
    pub nondeductible_suspended: Amount<Cents>,
    // The sum of every allowed loss and expense, for the year-end amount.
    pub allowed_total: Amount<Cents>,
}

pub(crate) fn limit_classes(
    income: Amount<Cents>,
    capacity: Amount<Cents>,
    basis_allowed: &LossAmounts,
    risk_carry: &LossAmounts,
    nondeductible: Amount<Cents>,
) -> Result<RiskStage, BasisError> {
    // Keep the native ordinary/expense/catalog order, with the two genuine
    // loss sources separate until the shared owner's checked addition.
    let mut slots = alloc::vec![Some(K1LossClass::Ordinary), None];
    slots.extend(
        K1LossClass::ALL
            .into_iter()
            .filter(|class| *class != K1LossClass::Ordinary && class.at_risk())
            .map(Some),
    );
    let entries = slots
        .iter()
        .map(|slot| {
            let (basis, prior) = match slot {
                Some(class) => (basis_allowed[*class], risk_carry[*class]),
                None => (nondeductible, Amount::default()),
            };
            Ok(risk::RiskEntry::<_, Cents> { key: *slot, basis_allowed: basis, prior_risk: prior })
        })
        .collect::<Result<Vec<_>, BasisError>>()?;
    let bypass = K1LossClass::ALL
        .into_iter()
        .filter(|class| !class.at_risk())
        .map(|class| Ok((Some(class), basis_allowed[class])))
        .collect::<Result<Vec<_>, BasisError>>()?;
    let SourceFact::Reported(limited) =
        risk::settle_at_risk(SourceFact::Reported(risk::RiskInput {
            entries: &entries,
            bypass: &bypass,
            income,
            capacity: capacity.get(),
        }))
        .map_err(|error| match error {
            LossLimitError::AtRiskRecapture => BasisError::AtRiskRecapture,
            _ => BasisError::Invalid,
        })?
    else {
        unreachable!("reported at-risk classes")
    };
    let mut stage = RiskStage {
        allowed: *basis_allowed,
        suspended: LossAmounts::default(),
        nondeductible_allowed: Amount::default(),
        nondeductible_suspended: Amount::default(),
        allowed_total: limited.gross_allowed,
    };
    for class in limited.classes {
        match class.key {
            Some(key) => {
                stage.allowed[*key] = class.allowed;
                stage.suspended[*key] = class.suspended;
            }
            None => {
                stage.nondeductible_allowed = class.allowed;
                stage.nondeductible_suspended = class.suspended;
            }
        }
    }
    Ok(stage)
}

impl LossLedger {
    // The ledger of one K-1 from its basis result and at-risk stage.
    pub(crate) fn of(
        basis_allowed: LossAmounts,
        basis_suspended: LossAmounts,
        stage: &RiskStage,
    ) -> Self {
        Self {
            basis_allowed,
            basis_suspended,
            risk_allowed: stage.allowed,
            risk_suspended: stage.suspended,
            nondeductible_suspended: stage.nondeductible_suspended,
            passive_suspended: LossAmounts::default(),
        }
    }

    // The ledger of a K-1 whose at-risk stage is skipped (question C9): every
    // basis-allowed amount passes, nothing is held back.
    pub(crate) fn without_risk_stage(
        basis_allowed: LossAmounts,
        basis_suspended: LossAmounts,
    ) -> Self {
        Self {
            basis_allowed,
            basis_suspended,
            risk_allowed: basis_allowed,
            risk_suspended: LossAmounts::default(),
            nondeductible_suspended: Amount::default(),
            passive_suspended: LossAmounts::default(),
        }
    }
}

// Arithmetic safety: LossAmounts::total/plus retain native i64
// additions in source cents.
// Widen only with those actual receivers, including their regular/AMT paths.
const MONEY: core::ops::RangeInclusive<i64> = 0..=100_000_000_000;

/// Loss carried because of your basis or the at-risk limits
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1LossCarry {
    /// Carried because of your basis
    /// The carryforward column of your 2025 basis worksheet for this kind of loss: Form 7203 Part III column (e), or column E of the partner basis worksheet's Part II. Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis: i64,
    /// Carried because of the at-risk limits
    /// The part of this kind of loss your 2025 Form 6198 did not allow. It already passed your basis; do not include it above. Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk: i64,
}

impl K1LossCarry {
    pub(crate) fn valid(&self) -> bool {
        MONEY.contains(&self.basis) && MONEY.contains(&self.risk)
    }
}

/// Losses and deductions carried because of your basis or amount at risk
/// For this tax system. Rental losses carry by basis and by the at-risk limits; the section 179 deduction, the cash gifts and the foreign taxes carry by basis only. The ordinary business loss, and an S corporation's capital and section 1231 losses, keep their own carryover records.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1LossCarryforwards {
    /// Rental real estate loss — box 2
    pub real_estate: K1LossCarry,
    /// Other rental loss — box 3
    pub other_rental: K1LossCarry,
    /// Section 179 deduction carried because of your basis
    /// Not the Form 4562 line 13 carryover, which is entered with the box. Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section179: i64,
    /// Cash contributions (60%) carried because of your basis
    /// Not a Schedule A carryover. Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub charity: i64,
    /// Foreign taxes carried because of your basis
    /// Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub foreign_taxes: i64,
}

impl K1LossCarryforwards {
    pub(crate) fn valid(&self) -> bool {
        self.real_estate.valid()
            && self.other_rental.valid()
            && MONEY.contains(&self.section179)
            && MONEY.contains(&self.charity)
            && MONEY.contains(&self.foreign_taxes)
    }

    // Whether a rental or section 179 carry could reach the return through
    // Form 8582 or Form 4562 (a K-1 amount that waits for those forms).
    pub(crate) fn waits_for_return_forms(&self) -> bool {
        self.real_estate != K1LossCarry::default()
            || self.other_rental != K1LossCarry::default()
            || self.section179 != 0
    }

    // The carries of the classes this record holds, by limit.
    pub(crate) fn basis(&self) -> LossAmounts {
        LossAmounts::of(&[
            (
                K1LossClass::RentalRealEstate,
                Amount::new(self.real_estate.basis).expect("validated retained source"),
            ),
            (
                K1LossClass::OtherRental,
                Amount::new(self.other_rental.basis).expect("validated retained source"),
            ),
            (
                K1LossClass::Section179,
                Amount::new(self.section179).expect("validated retained source"),
            ),
            (K1LossClass::Charity, Amount::new(self.charity).expect("validated retained source")),
            (
                K1LossClass::ForeignTaxes,
                Amount::new(self.foreign_taxes).expect("validated retained source"),
            ),
        ])
    }

    pub(crate) fn risk(&self) -> LossAmounts {
        LossAmounts::of(&[
            (
                K1LossClass::RentalRealEstate,
                Amount::new(self.real_estate.risk).expect("validated retained source"),
            ),
            (
                K1LossClass::OtherRental,
                Amount::new(self.other_rental.risk).expect("validated retained source"),
            ),
        ])
    }
}

/// Capital losses carried because of your basis or amount at risk — boxes 8 and 9a
/// For this tax system. Each keeps its short-term or long-term character.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PartnerCapitalCarry {
    /// Short-term capital loss — box 8
    pub short: K1LossCarry,
    /// Long-term capital loss — box 9a
    pub long: K1LossCarry,
}

impl PartnerCapitalCarry {
    pub(crate) fn valid(&self) -> bool {
        self.short.valid() && self.long.valid()
    }

    pub(crate) fn basis(&self) -> LossAmounts {
        LossAmounts::of(&[
            (
                K1LossClass::ShortTerm,
                Amount::new(self.short.basis).expect("validated retained source"),
            ),
            (
                K1LossClass::LongTerm,
                Amount::new(self.long.basis).expect("validated retained source"),
            ),
        ])
    }

    pub(crate) fn risk(&self) -> LossAmounts {
        LossAmounts::of(&[
            (
                K1LossClass::ShortTerm,
                Amount::new(self.short.risk).expect("validated retained source"),
            ),
            (
                K1LossClass::LongTerm,
                Amount::new(self.long.risk).expect("validated retained source"),
            ),
        ])
    }
}

// One review note per class this unit adds whose basis or at-risk carry to
// 2027 is not zero: the document, the class and the two amounts.
// An AMT note is left out for a class whose carries equal the regular ones
// (`regular` is the regular ledger then): the regular note already says it.
pub(crate) fn carry_notes(
    entry: &crate::FederalIncomeEntry,
    partner: bool,
    amt: bool,
    ledger: &LossLedger,
    regular: Option<&LossLedger>,
    ended: bool,
    result: &mut crate::FederalResult,
) {
    for class in K1LossClass::ALL {
        // After the sale of the whole interest nothing carries: a basis
        // carry can never be allowed (Reg. 1.704-1(d)(1), 1.1366-2(a)(6)(i))
        // and the gain released the at-risk carries (unit B9-1). The regular
        // note of a class that has its own says it carried; this one ends it.
        if ended && !amt {
            if ledger.basis_suspended[class].get() != 0 || ledger.risk_suspended[class].get() != 0 {
                let message = crate::Message::new(
                    "NOTE_K1_SUSPENDED_LOSS_ENDS",
                    alloc::vec![
                        crate::interview::income_display_arg(entry),
                        crate::MessageArg::Resource(class.key().into()),
                    ],
                );
                result
                    .review_notes
                    .push(crate::federal_model::ReviewNote::localized(
                        #[cfg(test)]
                        alloc::format!(
                            "Document {}: {} that your basis or amount at risk held back can't be deducted after the sale.",
                            entry.id,
                            class.english(),
                        ),
                        message,
                    ));
            }
            continue;
        }
        if class.has_own_note(partner) {
            continue;
        }
        let (basis, risk) =
            (ledger.basis_suspended[class].get(), ledger.risk_suspended[class].get());
        if basis == 0 && risk == 0 {
            continue;
        }
        if regular.is_some_and(|regular| {
            (regular.basis_suspended[class].get(), regular.risk_suspended[class].get())
                == (basis, risk)
        }) {
            continue;
        }
        result
            .review_notes
            .push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)]
                alloc::format!(
                    "Document {}: {} carried to 2027 — ${} because of your basis, ${} because of the at-risk limits.{}",
                    entry.id,
                    class.english(),
                    dollars(basis), dollars(risk),
                    if amt { " (AMT)" } else { "" },
                ),
                crate::Message::new(
                    if amt {
                        "NOTE_K1_AMT_CLASS_CARRYFORWARD"
                    } else {
                        "NOTE_K1_LOSS_CARRYFORWARD"
                    },
                    alloc::vec![
                        crate::interview::income_display_arg(entry),
                        crate::MessageArg::Resource(class.key().into()),
                        crate::MessageArg::usd_cents(basis),
                        crate::MessageArg::usd_cents(risk),
                    ],
                ),
            ));
    }
}

#[cfg(test)]
impl K1LossClass {
    pub(crate) fn english(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary business loss",
            Self::RentalRealEstate => "rental real estate loss",
            Self::OtherRental => "other rental loss",
            Self::ShortTerm => "short-term capital loss",
            Self::LongTerm => "long-term capital loss",
            Self::Collectibles => "collectibles loss",
            Self::Section1231 => "section 1231 loss",
            Self::OtherLoss => "other income loss",
            Self::Section179 => "section 179 deduction",
            Self::Charity => "cash contributions",
            Self::OtherCharity => "other contributions",
            Self::InvestmentInterest => "investment interest expense",
            Self::OtherDeductions => "other deductions",
            Self::ForeignTaxes => "foreign taxes",
        }
    }
}

// The App has existing projections for Schedule C/E/F amounts and Schedule D/
// Form 4797 amounts. Other classes still lack a complete AMT projection when
// their basis allowances differ; this is an App boundary, not an IRS exclusion.
// Mapped capital codes reuse the Schedule D adjustment (i6251 line 2k,
// 2025 final; correction 2026-10-19), with each system's own basis/risk ledger.
pub(crate) fn amt_limit_differs(
    regular: &LossLedger,
    amt: &LossLedger,
    regular_items: &crate::k1_items::K1Items,
    amt_items: &crate::k1_items::K1Items,
) -> bool {
    // Code I has a Schedule E destination and therefore a Form 6251 line 2n
    // adjustment. Remove only its share of this class's basis allowance;
    // other deductions keep their existing refusal. Sale losses have a
    // Form 4797 destination and reach line 2k. Compare the residual code
    // losses, not the whole OtherLoss class; each tax system has its own
    // sale amounts and therefore its own per-item basis allocation. Preferred
    // stock (Q) and nonportfolio capital (S) already reach Schedule D through
    // k1_totals::apply in both systems. Subtract individual loss rows, so a
    // positive code cannot hide another code's unmapped loss.
    let unmapped = |ledger: &LossLedger, items: &crate::k1_items::K1Items| {
        let basis_only = LossLedger { risk_allowed: ledger.basis_allowed, ..*ledger };
        let flows = items.flows(&basis_only);
        let mut allowed = ledger.basis_allowed;
        allowed[K1LossClass::OtherDeductions] = crate::kernel::carry::closing_values::<Cents>(
            allowed[K1LossClass::OtherDeductions].get(),
            flows.royalty_deductions,
        )
        .expect("mapped deductions are inside their class");
        allowed[K1LossClass::OtherLoss] = crate::kernel::carry::closing_values::<Cents>(
            allowed[K1LossClass::OtherLoss].get(),
            flows.sales.iter().map(|(_, _, cents)| phase::floor(-cents, 0)).sum::<i64>()
                + flows
                    .rows
                    .iter()
                    .filter_map(|(_, kind, cents)| {
                        use crate::k1_items::K1ItemKind;
                        matches!(
                            kind,
                            K1ItemKind::PreferredShort
                                | K1ItemKind::PreferredLong
                                | K1ItemKind::NonportfolioShort
                                | K1ItemKind::NonportfolioLong
                        )
                        .then_some(phase::floor(-cents, 0))
                    })
                    .sum::<i64>(),
        )
        .expect("mapped losses are inside their class");
        allowed
    };
    let regular = unmapped(regular, regular_items);
    let amt = unmapped(amt, amt_items);

    [
        K1LossClass::Charity,
        K1LossClass::Section179,
        K1LossClass::OtherLoss,
        K1LossClass::OtherCharity,
        K1LossClass::InvestmentInterest,
        K1LossClass::OtherDeductions,
    ]
    .into_iter()
    .any(|class| regular[class] != amt[class])
}

#[cfg(test)]
pub(crate) const AMT_LIMIT_DIFFERS: &str = crate::fixture_english!(
    "Your AMT basis allows a different section 179 deduction, contribution, other-income loss, investment interest expense or other deduction than your regular basis; this App does not yet calculate the AMT adjustment for that difference."
);

#[cfg(test)]
impl LossAmounts {
    // The three capital categories of an S corporation, in catalog order.
    pub(crate) fn capital(&self) -> [i64; 3] {
        [
            self[K1LossClass::ShortTerm].get(),
            self[K1LossClass::LongTerm].get(),
            self[K1LossClass::Collectibles].get(),
        ]
    }
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1LossCarry { amounts: [basis, risk], gains: [], facts: [] });

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1LossCarryforwards {
    amounts: [real_estate, other_rental, section179, charity, foreign_taxes],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartnerCapitalCarry {
    amounts: [short, long],
    gains: [],
    facts: [],
});

#[cfg(test)]
mod source_cent_tests {
    use super::*;
    use crate::kernel::source_fact::SourceFact;

    #[test]
    fn basis_risk_and_passive_stages_keep_cent_conservation() {
        let amount = |value| Amount::<Cents>::new(value).unwrap();
        let losses = LossAmounts::of(&[
            (K1LossClass::Ordinary, amount(200)),
            (K1LossClass::RentalRealEstate, amount(200)),
        ]);
        let (basis_allowed, basis_held) = losses.limit(amount(300));
        assert_eq!(basis_allowed[K1LossClass::Ordinary].get(), 150);
        assert_eq!(basis_allowed[K1LossClass::RentalRealEstate].get(), 150);
        assert_eq!(basis_held.total().get(), 100);
        let empty = LossAmounts::default();
        let first = LossAmounts::of(&[(K1LossClass::Ordinary, amount(150))]);
        let second = LossAmounts::of(&[(K1LossClass::Ordinary, amount(150))]);
        let first = limit_classes(amount(0), amount(100), &first, &empty, amount(0)).unwrap();
        let second = limit_classes(amount(0), amount(50), &second, &empty, amount(0)).unwrap();
        assert_eq!(first.allowed_total.get(), 100);
        assert_eq!(second.allowed_total.get(), 50);
        assert_eq!(first.suspended.total().get() + second.suspended.total().get(), 150);
        let activities = [
            crate::kernel::loss_limits::PassiveActivity {
                key: 1,
                net: -100,
                rental_allowance: false,
                ineligible_prior: 0,
            },
            crate::kernel::loss_limits::PassiveActivity {
                key: 2,
                net: -50,
                rental_allowance: false,
                ineligible_prior: 0,
            },
        ];
        let SourceFact::Reported(pool) = crate::kernel::loss_limits::passive_pool(
            SourceFact::Reported(crate::kernel::loss_limits::PoolInput {
                activities: &activities,
                other_passive_income: amount(100),
                allowance: amount(0),
            }),
        )
        .unwrap() else {
            panic!("reported sources stay reported")
        };
        assert_eq!(pool.allowed_loss(), 100);
        assert_eq!(pool.held_loss(), 50);
        assert_eq!(pool.suspended(), &[33, 17]);
        assert_eq!(pool.annual_suspensions().unwrap(), [1, 0]);
    }
}
