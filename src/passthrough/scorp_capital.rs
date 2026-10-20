use crate::k1_loss::{ClassInput, K1LossClass};
use crate::kernel::carry::{Amount, Cents};
use crate::kernel::phase;
use crate::scorp_basis::BasisResult;
use crate::{FederalResult, ScorpK1};
use libertas_macros::*;

fn capital_note(key: &str, id: i64, cents: &[i64]) -> crate::federal_model::ReviewNote {
    let mut args = alloc::vec![crate::MessageArg::Number(id)];
    args.extend(cents.iter().copied().map(crate::MessageArg::usd_cents));
    let message = crate::Message::new(key, args);
    crate::federal_model::ReviewNote::localized(
        #[cfg(test)]
        message.english(),
        message,
    )
}

/// K-1 portfolio capital amounts and retained losses
/// Keep these records separate from the return's Schedule D capital-loss carryforward. Amounts still suspended here have not yet reached Schedule D.
/// Confirm the entered box 8b amount, retained collectible losses and identified portfolio box 8c components are complete. All entered box 8c gains belong to box 8a and are portfolio income. Business-asset section 1250/section 1231 amounts belong only in the separate business-property record. No qualified-small-business-stock adjustments, section 1256 contracts, foreign adjustments, nonportfolio business gains or other special capital treatment. These portfolio amounts are not QBI or passive operating income. Confirm allocation to the same at-risk activity as the other declared K-1 amounts. Additional cases require separate worksheets.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpCapital {
    /// Box 7 net short-term capital gain or loss
    #[libertas_money("USD", 2)]
    pub short: i64,
    /// Box 8a net long-term capital gain or loss
    /// Total including the separately identified collectible and portfolio section 1250 components.
    #[libertas_money("USD", 2)]
    pub long: i64,
    /// Box 8b collectibles gain or loss included in box 8a
    /// Enter the signed component, not additional income. It may exceed the net box 8a gain when other long-term assets lost money.
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Box 8c portfolio gain from partnership-interest sales
    /// From the attached source statement: unrecaptured section 1250 gain included in box 8a, reported on line 10 of its worksheet. Exclude the corporation's business-asset gains; a bare box 8c amount is not enough to identify this category.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_partnership: i64,
    /// Box 8c portfolio gain from estates, trusts, mutual funds or REITs
    /// From the attached source statement: unrecaptured section 1250 gain included in box 8a, reported on line 11 of its worksheet. Not additional income. Do not repeat either component as another income document.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unrecaptured_fund: i64,
    /// Prior short-term loss suspended by shareholder basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis_short: i64,
    /// Prior total long-term loss suspended by shareholder basis
    /// Include the separately identified collectible portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis_long: i64,
    /// Collectible portion of prior basis-suspended long-term loss
    /// Included in the preceding total, not an additional loss.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub basis_collectibles: i64,
    /// Prior short-term loss suspended only by at-risk limits
    /// Already passed basis; do not include it in basis-suspended losses.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub risk_short: i64,
    /// Prior total long-term loss suspended only by at-risk limits
    /// Include the separately identified collectible portion.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub risk_long: i64,
    /// Collectible portion of prior at-risk-suspended long-term loss
    /// Included in the preceding total and already passed basis limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub risk_collectibles: i64,
}

impl ScorpCapital {
    // Arithmetic safety: current_losses, losses and capital_result_in combine signed native
    // i64 components before k1_totals cents transfers.
    pub(crate) fn valid(&self) -> bool {
        [self.short, self.long, self.collectibles]
            .iter()
            .all(|v| (-100000000000..=100000000000).contains(v))
            && [
                self.unrecaptured_partnership,
                self.unrecaptured_fund,
                self.basis_short,
                self.basis_long,
                self.basis_collectibles,
                self.risk_short,
                self.risk_long,
                self.risk_collectibles,
            ]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
            && self.basis_collectibles <= self.basis_long
            && self.risk_collectibles <= self.risk_long
    }
    // Sections 1.1366-1(a)(2)(i), 1.1366-2(a)(3)-(5), and 1.1367-1(f):
    // retain each holding-period category, increase basis for income first,
    // and share the basis limit among current and carried loss items. Do not
    // offset a capital loss against another category before this allocation.
    // Internal order: short-term, other long-term, long-term collectibles.
    // Subtract rounded components from rounded totals to conserve each total.
    fn current(&self) -> [i64; 3] {
        let collectibles = self.collectibles;
        [self.short, self.long - collectibles - self.unrecaptured(), collectibles]
    }
    pub(crate) fn unrecaptured(&self) -> i64 {
        self.unrecaptured_partnership + self.unrecaptured_fund
    }
    pub(crate) fn gains(&self) -> [i64; 3] {
        let mut gains = self.current().map(|v| phase::floor(v, 0));
        // The 25% category is gain-only. It funds the loss limits separately
        // from negative other-long-term items, then remains included once in
        // the total noncollectible long-term result returned to Schedule D.
        gains[1] += self.unrecaptured();
        gains
    }
    // This year's loss in each category (short-term, other long-term,
    // collectibles).
    pub(crate) fn current_losses(&self) -> [i64; 3] {
        let current = self.current();
        [phase::floor(-current[0], 0), phase::floor(-current[1], 0), phase::floor(-current[2], 0)]
    }
    // The basis carryover of each category.
    pub(crate) fn basis_carry(&self) -> [i64; 3] {
        let collectibles = self.basis_collectibles;
        [self.basis_short, self.basis_long - collectibles, collectibles]
    }
    #[cfg(test)]
    pub(crate) fn losses(&self) -> [i64; 3] {
        let (current, carry) = (self.current_losses(), self.basis_carry());
        [current[0] + carry[0], current[1] + carry[1], current[2] + carry[2]]
    }
    pub(crate) fn prior_risk(&self) -> [i64; 3] {
        let collectibles = self.risk_collectibles;
        [self.risk_short, self.risk_long - collectibles, collectibles]
    }
}

impl ScorpK1 {
    // Prior basis, at-risk or passive losses this K-1 carries into 2026.
    pub(crate) fn suspended_history(&self) -> bool {
        self.prior_loss != 0
            || self.risk.prior_loss != 0
            || self.passive.loss != 0
            || self.rentals.passive_history()
            || self.amt.as_ref().is_some_and(|amt| amt.rentals.passive_history())
            || self.capital.as_ref().is_some_and(|v| {
                [
                    v.basis_short,
                    v.basis_long,
                    v.basis_collectibles,
                    v.risk_short,
                    v.risk_long,
                    v.risk_collectibles,
                ]
                .iter()
                .any(|amount| *amount != 0)
            })
            || self
                .section1231
                .as_ref()
                .is_some_and(|v| v.basis_loss != 0 || v.risk_loss != 0 || v.passive_loss != 0)
    }
    // A deceased shareholder's suspended losses are not carried onto the final
    // return (Pub. 559 "Passive activity rules" and S corporation losses).
    pub(crate) fn drop_suspended_history(&mut self) {
        self.prior_loss = 0;
        self.risk.prior_loss = 0;
        self.passive.loss = 0;
        self.rentals.drop_passive_history();
        if let Some(amt) = &mut self.amt {
            amt.rentals.drop_passive_history();
        }
        if let Some(v) = self.capital.as_mut() {
            v.basis_short = 0;
            v.basis_long = 0;
            v.basis_collectibles = 0;
            v.risk_short = 0;
            v.risk_long = 0;
            v.risk_collectibles = 0;
        }
        if let Some(v) = self.section1231.as_mut() {
            v.basis_loss = 0;
            v.risk_loss = 0;
            v.passive_loss = 0;
        }
    }
    pub(crate) fn capital_gains(&self) -> i64 {
        self.capital.as_ref().map_or(0, |v| v.gains().iter().sum())
    }
    // The classes this K-1 brings to the loss limits in this tax system: its
    // share of each loss and deduction, and what its records carry (Form 7203
    // Part III columns (a) and (b); Form 6198 lines 1, 2a, 2b and 4).
    pub(crate) fn class_input(&self) -> ClassInput {
        use K1LossClass::*;
        let mut input = ClassInput::default();
        input.current[Ordinary] = Amount::<Cents>::new(phase::floor(-self.ordinary, 0))
            .expect("validated source class cents");
        input.basis_carry[Ordinary] =
            Amount::<Cents>::new(self.prior_loss).expect("validated source class cents");
        input.risk_carry[Ordinary] =
            Amount::<Cents>::new(self.risk.prior_loss).expect("validated source class cents");
        if let Some(capital) = &self.capital {
            for (class, (current, (carry, risk))) in
                [ShortTerm, LongTerm, Collectibles].into_iter().zip(
                    capital
                        .current_losses()
                        .into_iter()
                        .zip(capital.basis_carry().into_iter().zip(capital.prior_risk())),
                )
            {
                input.current[class] =
                    Amount::<Cents>::new(current).expect("validated source class cents");
                input.basis_carry[class] =
                    Amount::<Cents>::new(carry).expect("validated source class cents");
                input.risk_carry[class] =
                    Amount::<Cents>::new(risk).expect("validated source class cents");
            }
        }
        if let Ok(Some(property)) = self.business_property() {
            input.current[Section1231] =
                Amount::<Cents>::new(property.loss).expect("validated source class cents");
            input.basis_carry[Section1231] =
                Amount::<Cents>::new(property.basis_loss).expect("validated source class cents");
            input.risk_carry[Section1231] =
                Amount::<Cents>::new(property.risk_loss).expect("validated source class cents");
        }
        input.current[Section1231] = Amount::<Cents>::new(
            input.current[Section1231].get() + (self.rentals.property_losses().iter().sum::<i64>()),
        )
        .expect("validated source class cents");
        // Section 1366(d)(1) limits the box 12 code A gift with the losses;
        // it is a deduction item of its own class in Form 7203 Part III.
        input.current[Charity] =
            Amount::<Cents>::new(self.cash_charity).expect("validated source class cents");
        // Box 11 section 179 is likewise a Part III deduction class of its
        // own (T2-29b); Form 4562 Part I and the at-risk test follow in
        // section179.rs.
        input.current[Section179] =
            Amount::<Cents>::new(self.section179).expect("validated source class cents");
        // Form 7203 Part III line 46: the box 16 code F foreign taxes.
        input.current[ForeignTaxes] =
            Amount::<Cents>::new(self.foreign_taxes).expect("validated source class cents");
        // Form 7203 Part III lines 36 and 37: the box 2 and box 3 rental
        // losses, each its own class.
        let [real_estate, other] = self.rentals.losses();
        input.current[RentalRealEstate] =
            Amount::<Cents>::new(real_estate).expect("validated source class cents");
        input.current[OtherRental] =
            Amount::<Cents>::new(other).expect("validated source class cents");
        input.basis_carry = input.basis_carry.plus(&self.carried.basis());
        input.risk_carry = input.risk_carry.plus(&self.carried.risk());
        // Unit B9-1: the box 10 and box 12 codes bring their losses and
        // deductions to the classes they join.
        input.current = input.current.plus(&self.k1_items().current());
        input
    }
    pub(crate) fn capital_result_in(
        &self,
        basis: &BasisResult,
        id: i64,
        amt: bool,
        result: &mut FederalResult,
        events: Option<&crate::scorp::ScorpEvents>,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<[i64; 4], (&'static str, &'static str)> {
        let Some(capital) = &self.capital else {
            return Ok([0; 4]);
        };
        let risk = self
            .risk
            .calculate_in(self, basis, events, capacity)
            .map_err(|error| error.message())?;
        let mut net = capital.gains();
        let ledger = &risk.ledger;
        for (term, (class, (regular_key, amt_key))) in
            [K1LossClass::ShortTerm, K1LossClass::LongTerm, K1LossClass::Collectibles]
                .into_iter()
                .zip([
                    ("NOTE_SCORP_CAPITAL_REGULAR_SHORT", "NOTE_SCORP_CAPITAL_AMT_SHORT"),
                    ("NOTE_SCORP_CAPITAL_REGULAR_LONG", "NOTE_SCORP_CAPITAL_AMT_LONG"),
                    (
                        "NOTE_SCORP_CAPITAL_REGULAR_COLLECTIBLES",
                        "NOTE_SCORP_CAPITAL_AMT_COLLECTIBLES",
                    ),
                ])
                .enumerate()
        {
            net[term] -= ledger.risk_allowed[class].get();
            result.review_notes.push(capital_note(
                if amt { amt_key } else { regular_key },
                id,
                &[net[term], basis.suspended[class].get(), ledger.risk_suspended[class].get()],
            ));
        }
        if capital.collectibles != 0
            || capital.basis_collectibles != 0
            || capital.risk_collectibles != 0
        {
            result.review_notes.push(capital_note(
                if amt {
                    "NOTE_SCORP_CAPITAL_AMT_TOTAL_LONG"
                } else {
                    "NOTE_SCORP_CAPITAL_REGULAR_TOTAL_LONG"
                },
                id,
                &[
                    net[1] + net[2],
                    basis.suspended[K1LossClass::LongTerm].get()
                        + basis.suspended[K1LossClass::Collectibles].get(),
                    ledger.risk_suspended[K1LossClass::LongTerm].get()
                        + ledger.risk_suspended[K1LossClass::Collectibles].get(),
                ],
            ));
        }
        if capital.unrecaptured() != 0 {
            result.review_notes.push(capital_note(
                if amt { "NOTE_SCORP_CAPITAL_AMT_1250" } else { "NOTE_SCORP_CAPITAL_REGULAR_1250" },
                id,
                &[capital.unrecaptured_partnership, capital.unrecaptured_fund],
            ));
        }
        // Short-term, noncollectible long-term (including 1250), collectibles,
        // and the 1250 rate component already included in the second entry.
        Ok([net[0], net[1], net[2], capital.unrecaptured()])
    }
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(ScorpCapital {
    amounts: [
        short,
        long,
        collectibles,
        unrecaptured_partnership,
        unrecaptured_fund,
        basis_short,
        basis_long,
        basis_collectibles,
        risk_short,
        risk_long,
        risk_collectibles
    ],
    gains: [],
    facts: [],
});
