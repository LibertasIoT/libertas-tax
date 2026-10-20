// Coverage: Schedule K-1 (1065) and (1120-S) boxes 2 and 3 — Supported (unit k1-rentals,
//   2026-10-09; i1065sk1 2025 final (no 2026 revision), i1120ssk 2026 draft of 9/18/26, f8582 2026
//   draft of 9/18/26, i8582 2025, Pub 925 2025, i1040se 2026 draft, IRC 469(c)(2), (c)(7), (f),
//   (i)(1)–(3), (i)(6)(A), (i)(6)(C)).
//   Lines: box 2 net rental real estate income (loss) and box 3 other net rental income (loss), each
//   one passive activity with its own 2025 Form 8582 carryover, per tax system (K1RentalAmounts on
//   PartnerActivityRecord, ScorpK1 and ScorpAmt). Income raises outside basis (partner worksheet
//   line 4; Form 7203 lines 3b/3c) and Form 6198 activity income; a loss is its own basis class
//   (Form 7203 lines 36/37) and Form 6198 loss of the K-1's one at-risk activity. Form 8582: box 2
//   with active participation in Part IV with the special allowance (MAGI phase-out), box 2
//   without it and box 3 in Part V (extended_rules.rs rentals_for, PassiveEntry.pass_through).
//   A limited partner never actively participates (section 469(i)(6)(C)): PARTNER_LIMITED_NOT_ACTIVE
//   on both sides and PartnerK1::valid. A real estate professional who materially participated
//   (K1RentalParticipation::Professional, with the return's recorded status) makes box 2
//   nonpassive; its carryover is released against the activity's own income first (section
//   469(f)), MAGI excludes its allowed loss (i8582 line 6), the recorded NIIT safe harbor reverses
//   it as for a Schedule E rental. Schedule E line 28: {kind}_N_rental_real_estate_net/_suspended
//   and {kind}_N_other_rental_net/_suspended (amt_ twins). QBI when the section 199A statement
//   lists the activity: the allowed net with its wages and UBIA (PassiveEntry.uniform_qbi).
//   Unit B8 (2026-10-10): a rental loss held back by the basis or at-risk limits carries by activity
//   (K1LossClass::RentalRealEstate and OtherRental; K1LossCarryforwards.real_estate/other_rental) and the
//   activity's 2026 loss is what both limits allowed, a released carry included (activities reads the
//   LossLedger). Refused by name: the professional answer without the recorded status (ISSUE_K1_RENTAL_PROFESSIONAL_STATUS), rentals
//   of a publicly traded partnership (ISSUE_K1_RENTAL_PTP), an installment/exchange/casualty/theft
//   or related-buyer rental disposition (PartnerK1Item::RentalDisposition,
//   ScorpK1Item::RentalDisposition), box 22/18 and a box 23/19
//   statement with two activities of one kind (MultipleActivities). The carryover follows the
//   current participation answer (i8582 Part IV column (c) asks for both years; improvement log).
//   Evidence: k1_rental.rs, partnership.rs (result_with, calculate_with), scorp_capital.rs
//   (class_input), scorp_risk.rs, extended_rules.rs, k1_page.rs (K1RentalBoxes); tests/passthrough/k1_rental.rs.
//   B9-7 R6: entity rental-property sections join the shared Section1231 class at basis/risk,
//   with ISSUE_K1_RENTAL_PROPERTY_LIMITED if either holds back a current rental-property loss.
//   Separate ordinary/property PALs enter each rental activity's existing character allocator.
//   A fully taxable unrelated full sale releases its losses; overall gain offsets other PALs,
//   while a professional's former-passive activity stays nonpassive. Allowed property parts
//   aggregate once per K-1 in Form 4797; QBI uses return-wide character in the same rental QBI
//   activity, and NIIT retains passive origin unless the professional safe harbor applies.
//   A professional full sale keeps section469(g) prior losses as NIIT deductions, separate
//   from the own-income section469(f) release (i8960 2025, entire-interest dispositions).
//   Evidence: tests/passthrough/k1_rental.rs and tests/passthrough/interview.rs k1_rental_property_*; Pub925/i8582 2025.
//   Full B9-7/Unit4 acceptance and mixed property EBL allocation remain open.
use crate::kernel::phase;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Property of a rental activity sold by the entity
/// The parts of the section 1231 and unrecaptured section 1250 boxes the statement assigns to this activity, for this tax system.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1RentalProperty {
    /// Net section 1231 gain (loss) of this activity
    /// The part of Form 1065 box 10 (Form 1120-S box 9) from this activity. Enter a loss as a negative amount.
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub net: i64,
    /// Unrecaptured section 1250 gain of this activity
    /// The part of the business-property amount in Form 1065 box 9c (Form 1120-S box 8c) from this activity.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
    /// Section 1231 loss of this activity carried from 2025 because of the passive activity rules
    /// The part of the activity's 2025 Form 8582 unallowed loss that was a Form 4797 loss. It is not part of the ordinary rental carryover. Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub passive_loss: i64,
}

/// Rental activity
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum K1RentalActivity {
    /// The rental real estate activity — box 2
    RealEstate,
    /// The other rental activity — box 3
    Other,
}

/// Rental activities of this K-1 in one tax system — boxes 2 and 3
/// Each box is one activity. Enter the printed amount, a loss as a negative amount, and zero for a blank box. For alternative minimum tax use the amounts after that system's adjustments.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1RentalAmounts {
    /// Net rental real estate income (loss) — box 2
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub real_estate: i64,
    /// Rental real estate loss carried from 2025 because of the passive activity rules
    /// From your 2025 Form 8582 for this activity. It already passed the basis and at-risk limits. Leave out a section 1231 part; it is entered with the activity's property. Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub real_estate_carryover: i64,
    /// Property of the rental real estate activity sold
    pub real_estate_property: K1RentalProperty,
    /// Other net rental income (loss) — box 3
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub other: i64,
    /// Other rental loss carried from 2025 because of the passive activity rules
    /// From your 2025 Form 8582 for this activity. It already passed the basis and at-risk limits. Leave out a section 1231 part; it is entered with the activity's property. Zero when none was carried.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_carryover: i64,
    /// Property of the other rental activity sold
    pub other_property: K1RentalProperty,
}

/// How you took part in the rental real estate — box 2
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
#[libertas_inline_choices]
pub enum K1RentalParticipation {
    /// You did not actively participate
    /// Also choose this for a limited partner's interest, when you and your spouse own less than 10% of the activity, or when box 2 is blank. A loss then offsets only passive income.
    #[default]
    NotActive,
    /// You actively participated
    /// You or your spouse made management decisions in a significant and bona fide sense, such as approving tenants, setting rents or approving repairs, and together owned at least 10% of the activity all year. A limited partner cannot. Up to $25,000 of a loss can then offset your other income.
    Active,
    /// Real estate professional and material participant, without current active-participation eligibility
    /// Your professional status is recorded on People, and you materially participated in this activity, counting your spouse's work. Choose this when you did not make significant bona fide management decisions or you and your spouse did not own at least 10% by value throughout 2026. A limited partner may choose this only when a limited-partner material-participation test is met. Current income or loss is nonpassive; remaining prior passive losses cannot use the special allowance.
    Professional,
    /// Real estate professional and material participant, with current active-participation eligibility
    /// Your professional status is recorded on People, and you materially participated in this activity. You or your spouse also made significant bona fide management decisions and together owned at least 10% by value throughout 2026; you are not a limited partner. Remaining prior passive losses may use the special allowance only when you actively participated in the loss year too.
    ProfessionalActive,
}

impl K1RentalParticipation {
    pub(crate) fn professional(self) -> bool {
        matches!(self, Self::Professional | Self::ProfessionalActive)
    }

    pub(crate) fn active(self) -> bool {
        matches!(self, Self::Active | Self::ProfessionalActive)
    }
}

/// Section 199A amounts of a rental activity
/// From the K-1's section 199A statement when it lists the activity as a trade or business. Its qualified business income is the activity's amount after the loss limits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1RentalQbi {
    /// W-2 wages
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub wages: i64,
    /// Unadjusted basis of qualified property (UBIA)
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub property: i64,
}

/// Your facts about this K-1's rental activities
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct K1RentalFacts {
    /// How you took part in the rental real estate — box 2
    #[libertas_default(NotActive)]
    pub participation: K1RentalParticipation,
    /// Section 199A amounts of the rental real estate activity
    /// Absent when the statement does not list the box 2 activity as a trade or business.
    pub real_estate_qbi: Option<K1RentalQbi>,
    /// Section 199A amounts of the other rental activity
    /// Absent when the statement does not list the box 3 activity as a trade or business.
    pub other_qbi: Option<K1RentalQbi>,
    /// You actively participated in the year the box 2 loss carried from 2025 arose
    /// Form 8582 counts a prior-year unallowed loss toward the special allowance only for a year in which you actively participated. Leave on when there is no carried loss.
    #[libertas_default(true)]
    pub carryover_active: bool,
    /// Rental activities whose property the entity sold in full in 2026
    /// All of the activity's property, sold or exchanged for cash or other property in a fully taxable sale to a buyer not related to you or to the entity (not an installment sale, a like-kind exchange or a casualty). The activity's suspended passive losses are then allowed. Leave empty when none.
    /// ----
    /// Activity
    #[libertas_unordered]
    // Tax structure: only box 2 real estate and box 3 other rental activities.
    #[libertas_size(max = 2)]
    pub disposed: alloc::vec::Vec<K1RentalActivity>,
}

impl Default for K1RentalFacts {
    fn default() -> Self {
        Self {
            participation: K1RentalParticipation::NotActive,
            real_estate_qbi: None,
            other_qbi: None,
            carryover_active: true,
            disposed: alloc::vec::Vec::new(),
        }
    }
}

// Arithmetic safety: income, losses and grouped_former_passive retain native i64
// additions/subtractions before owner loss limits and cents transfers.
// Widen only with those actual receivers, including their regular/AMT paths.
const MONEY: core::ops::RangeInclusive<i64> = 0..=100_000_000_000;
const SIGNED: core::ops::RangeInclusive<i64> = -100_000_000_000..=100_000_000_000;

impl K1RentalAmounts {
    pub(crate) fn valid(&self) -> bool {
        SIGNED.contains(&self.real_estate)
            && SIGNED.contains(&self.other)
            && MONEY.contains(&self.real_estate_carryover)
            && MONEY.contains(&self.other_carryover)
            && self.properties().into_iter().all(K1RentalProperty::valid)
    }

    // Whether this K-1 reports or carries a rental activity at all.
    pub(crate) fn any(&self) -> bool {
        *self != Self::default()
    }

    pub(crate) fn real_estate_present(&self) -> bool {
        self.real_estate != 0
            || self.real_estate_carryover != 0
            || self.real_estate_property.has_items()
    }

    // Partner basis worksheet line 4 and Form 7203 lines 3b/3c; also the
    // Form 6198 income of the K-1's one at-risk activity. Actual source cents.
    pub(crate) fn income(&self) -> i64 {
        phase::floor(self.real_estate, 0)
            + phase::floor(self.other, 0)
            + self.properties().into_iter().map(|p| phase::floor(p.net, 0)).sum::<i64>()
    }

    // The two loss classes (Form 7203 lines 36 and 37; the partner basis
    // worksheet's Part II), box 2 first. Actual source cents.
    pub(crate) fn losses(&self) -> [i64; 2] {
        [phase::floor(-self.real_estate, 0), phase::floor(-self.other, 0)]
    }

    pub(crate) fn properties(&self) -> [&K1RentalProperty; 2] {
        [&self.real_estate_property, &self.other_property]
    }

    pub(crate) fn passive_history(&self) -> bool {
        self.real_estate_carryover != 0
            || self.other_carryover != 0
            || self.properties().iter().any(|p| p.passive_loss != 0)
    }

    pub(crate) fn drop_passive_history(&mut self) {
        self.real_estate_carryover = 0;
        self.other_carryover = 0;
        self.real_estate_property.passive_loss = 0;
        self.other_property.passive_loss = 0;
    }

    pub(crate) fn property_losses(&self) -> [i64; 2] {
        self.properties().map(|p| phase::floor(-p.net, 0))
    }

    // The one Section1231 class includes business and both rental parts.
    // A rental property's suspension cannot be saved as business history.
    // Reuse the class allocator to identify whether either rental part is
    // actually held back, including source-cent residual allocation.
    pub(crate) fn property_allowed(
        &self,
        ledger: &crate::k1_loss::LossLedger,
    ) -> Result<[i64; 2], (&'static str, &'static str)> {
        use crate::k1_loss::K1LossClass::Section1231;
        let losses = self.property_losses();
        if losses == [0, 0] {
            return Ok(losses);
        }
        let total =
            ledger.basis_allowed[Section1231].get() + ledger.basis_suspended[Section1231].get();
        let other = total - losses.iter().sum::<i64>();
        if other < 0 {
            return Err(PROPERTY_LIMITED);
        }
        let basis = crate::kernel::ratio::allocate_remaining(
            ledger.basis_allowed[Section1231].get(),
            &[other, losses[0], losses[1]],
        );
        let total =
            ledger.risk_allowed[Section1231].get() + ledger.risk_suspended[Section1231].get();
        let risk = crate::kernel::ratio::allocate_remaining(
            ledger.risk_allowed[Section1231].get(),
            &[total - basis[1] - basis[2], basis[1], basis[2]],
        );
        if basis[1..] != losses || risk[1..] != losses {
            return Err(PROPERTY_LIMITED);
        }
        Ok(losses)
    }
}

impl K1RentalProperty {
    pub(crate) fn valid(&self) -> bool {
        SIGNED.contains(&self.net)
            && MONEY.contains(&self.unrecaptured)
            && MONEY.contains(&self.passive_loss)
    }

    pub(crate) fn has_items(&self) -> bool {
        *self != Self::default()
    }
}

impl K1RentalFacts {
    pub(crate) fn valid(&self) -> bool {
        self.disposed.iter().enumerate().all(|(i, v)| !self.disposed[..i].contains(v))
            && [&self.real_estate_qbi, &self.other_qbi]
                .into_iter()
                .flatten()
                .all(|qbi| MONEY.contains(&qbi.wages) && MONEY.contains(&qbi.property))
    }
}

// One rental activity of a K-1 entering the shared Form 8582 pool, whole
// source cents: the passive net (current amount less the activity's carryover,
// after any own-income release), the nonpassive amount of a professional's
// activity, and whether its loss may use the special allowance.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RentalActivity {
    pub real_estate: bool,
    pub passive: i64,
    pub nonpassive: i64,
    pub prior_released: i64,
    // The prior unallowed loss left in the pool (column (c)).
    pub prior: i64,
    pub allowance: bool,
    // The part of `passive` that is a carryover from a year without active
    // participation, which the special allowance does not admit.
    pub ineligible_prior: i64,
    pub material: bool,
    pub qbi: Option<K1RentalQbi>,
    pub characters: Option<crate::scorp::ActivityCharacters>,
    pub property_gain: i64,
    pub released: bool,
    // Prior PAL released by section 469(g), after the same activity's
    // section 469(f) own-income release. It remains a NIIT deduction even
    // when the professional safe harbor excludes current income/loss.
    pub niit_disposition_loss: [i64; 2],
}

impl RentalActivity {
    pub(crate) fn current_and_prior(&self) -> ([i64; 2], [i64; 2]) {
        self.characters.map_or(
            ([self.nonpassive + self.prior_released, 0], [self.prior + self.prior_released, 0]),
            |parts| {
                (
                    [
                        parts.nonpassive[0] + parts.prior_released[0],
                        parts.nonpassive[1] + parts.prior_released[1],
                    ],
                    [
                        parts.prior_released[0] - parts.passive[0],
                        parts.prior_released[1] - parts.passive[1],
                    ],
                )
            },
        )
    }

    pub(crate) fn grouped_former_passive(
        &mut self,
        parts: crate::scorp::ActivityCharacters,
        income_release: [i64; 2],
        carryover_active: bool,
    ) {
        let (_, prior) = self.current_and_prior();
        self.passive = parts.passive.iter().sum();
        self.nonpassive = parts.nonpassive.iter().sum();
        self.prior_released = parts.prior_released.iter().sum();
        self.prior = -self.passive;
        self.ineligible_prior = if self.allowance && !carryover_active { self.prior } else { 0 };
        if self.characters.is_some() {
            self.characters = Some(parts);
        }
        if self.released {
            self.niit_disposition_loss =
                [prior[0] - income_release[0], prior[1] - income_release[1]];
        }
    }
}

pub(crate) const PROPERTY_LIMITED: (&str, &str) = (
    "ISSUE_K1_RENTAL_PROPERTY_LIMITED",
    crate::fixture_english!(
        "A rental activity's section 1231 loss held back by basis or at-risk limits needs an activity-specific property carryover that this K-1 record does not calculate."
    ),
);

pub(crate) const PROFESSIONAL_STATUS: (&str, &str) = (
    "ISSUE_K1_RENTAL_PROFESSIONAL_STATUS",
    crate::fixture_english!(
        "A Schedule K-1 box 2 rental real estate activity in which a real estate professional materially participated needs that status recorded for its owner on the People page (Real estate professional)."
    ),
);
pub(crate) const PTP: (&str, &str) = (
    "ISSUE_K1_RENTAL_PTP",
    crate::fixture_english!(
        "Schedule K-1 (1065) boxes 2 and 3 of a publicly traded partnership are limited by the passive activity rules separately from your other activities, which this App does not calculate."
    ),
);

// The K-1's rental activities in one tax system, in box order; an activity
// with no amount and no carryover is not listed. A limited partner never
// actively participates (section 469(i)(6)(C)); a professional's activity is
// a former passive activity for its carryover (section 469(f)(1)): the
// carryover first offsets the activity's own income and the rest stays
// passive; the allowance additionally requires current and loss-year active participation.
pub(crate) fn activities(
    amounts: &K1RentalAmounts,
    facts: &K1RentalFacts,
    ledger: &crate::k1_loss::LossLedger,
    limited: bool,
    professional_status: bool,
    disposed: bool,
    group_disposed: bool,
) -> Result<alloc::vec::Vec<RentalActivity>, (&'static str, &'static str)> {
    use crate::k1_loss::K1LossClass;
    amounts.property_allowed(ledger)?;
    let mut list = alloc::vec::Vec::new();
    for (real_estate, current, carryover, property, qbi, class, activity) in [
        (
            true,
            amounts.real_estate,
            amounts.real_estate_carryover,
            &amounts.real_estate_property,
            &facts.real_estate_qbi,
            K1LossClass::RentalRealEstate,
            K1RentalActivity::RealEstate,
        ),
        (
            false,
            amounts.other,
            amounts.other_carryover,
            &amounts.other_property,
            &facts.other_qbi,
            K1LossClass::OtherRental,
            K1RentalActivity::Other,
        ),
    ] {
        let allowed = ledger.risk_allowed[class].get();
        if current == 0 && carryover == 0 && allowed == 0 && !property.has_items() {
            continue;
        }
        let current = [phase::floor(current, 0) - allowed, property.net];
        let prior = [carryover, property.passive_loss];
        let participation =
            if real_estate { facts.participation } else { K1RentalParticipation::NotActive };
        let material = participation.professional();
        if material && !professional_status {
            return Err(PROFESSIONAL_STATUS);
        }
        let released = (disposed || facts.disposed.contains(&activity))
            && (!real_estate || !material || group_disposed);
        // A passive activity sold in full with an overall loss leaves the
        // pool. An overall gain offsets other PALs. A former passive
        // activity remains nonpassive (i8582, entire dispositions).
        let combined = current.iter().sum::<i64>() - prior.iter().sum::<i64>();
        let parts = if released && !material {
            if combined <= 0 {
                crate::scorp::ActivityCharacters {
                    passive: [0, 0],
                    nonpassive: [current[0] - prior[0], current[1] - prior[1]],
                    prior_released: prior,
                }
            } else {
                crate::scorp::ActivityCharacters {
                    passive: [current[0] - prior[0], current[1] - prior[1]],
                    nonpassive: [0, 0],
                    prior_released: [0, 0],
                }
            }
        } else {
            crate::scorp::ActivityCharacters::from_current_and_prior(
                current, prior, material, released,
            )?
        };
        let prior_released = parts.prior_released.iter().sum::<i64>();
        let niit_disposition_loss = if material && released {
            let former = crate::scorp::ActivityCharacters::from_current_and_prior(
                current, prior, true, false,
            )?;
            [prior[0] - former.prior_released[0], prior[1] - former.prior_released[1]]
        } else {
            [0, 0]
        };
        let prior_pool = prior.iter().sum::<i64>() - prior_released;
        let allowance = participation.active() && !limited && !released;
        list.push(RentalActivity {
            real_estate,
            passive: parts.passive.iter().sum(),
            nonpassive: parts.nonpassive.iter().sum(),
            prior_released,
            prior: prior_pool,
            allowance,
            ineligible_prior: if allowance && !facts.carryover_active { prior_pool } else { 0 },
            material,
            qbi: qbi.clone(),
            characters: property.has_items().then_some(parts),
            property_gain: phase::floor(property.net, 0),
            released,
            niit_disposition_loss,
        });
    }
    Ok(list)
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1RentalAmounts {
    amounts: [
        real_estate,
        real_estate_carryover,
        real_estate_property,
        other,
        other_carryover,
        other_property
    ],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1RentalProperty {
    amounts: [net, unrecaptured, passive_loss],
    gains: [],
    facts: [],
});

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1RentalQbi { amounts: [wages, property], gains: [], facts: [] });

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1RentalFacts {
    amounts: [real_estate_qbi, other_qbi],
    gains: [],
    facts: [participation, carryover_active, disposed],
});
