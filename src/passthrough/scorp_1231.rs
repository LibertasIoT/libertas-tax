use crate::k1_loss::K1LossClass;
use crate::kernel::phase;
use crate::scorp_basis::BasisResult;
use crate::{FederalResult, Message, MessageArg, ScorpK1};
use libertas_macros::*;

/// K-1 business-property gains, losses and suspensions
/// Keep these amounts separate from box 1 ordinary business income and portfolio capital boxes. Section 1231 character is determined after owner loss limits and return-wide netting. Complete supported source records produce return totals and separate regular/AMT histories; unresolved allocations and special transactions require further worksheets. These amounts belong to the declared ordinary business and at-risk activity; a sale of the owner's interest, liquidation, installment transaction needing additional records, casualty/theft sub-netting, section 179 property requiring owner-level recapture, related-party recharacterization and unentered transactions require their separate source worksheets. Do not infer QBI or final capital character from Form 1065 box 10 or Form 1120-S box 9 alone.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1Section1231 {
    /// Net section 1231 gain or loss of the box 1 trade or business
    /// Form 1065 box 10 or Form 1120-S box 9, after the entity's ordinary depreciation recapture and before the owner's loss limitations. Rental activity parts belong with that activity's amounts. Do not repeat it in box 1 or portfolio capital.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub net: i64,
    /// Business-asset unrecaptured section 1250 gain — Form 1065 box 9c or Form 1120-S box 8c
    /// From this activity's business-property statement, for line 5 of the section 1250 worksheet. This is a rate component, not additional income or basis. It may exceed the net section 1231 amount when other business-property losses offset gains. Fund/trust and partnership-interest portfolio components belong in the separate capital record.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub unrecaptured: i64,
    /// Prior section 1231 loss suspended by owner basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis_loss: i64,
    /// Prior section 1231 loss suspended only by at-risk limits
    /// Already passed basis limits. Exclude basis-suspended and passive-suspended amounts.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub risk_loss: i64,
    /// Prior section 1231 loss suspended only by passive-activity limits
    /// Already passed basis and at-risk limits; not part of either preceding balance or the separate ordinary passive-loss history.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub passive_loss: i64,
}

impl K1Section1231 {
    // Arithmetic safety: business_property and report_section1231_in combine native property
    // losses with rental/basis/risk totals before cents transfers.
    pub(crate) fn valid(&self) -> bool {
        (-100000000000..=100000000000).contains(&self.net)
            && [self.unrecaptured, self.basis_loss, self.risk_loss, self.passive_loss]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
    }
    pub(crate) fn has_items(&self) -> bool {
        self.net != 0
            || self.unrecaptured != 0
            || self.basis_loss != 0
            || self.risk_loss != 0
            || self.passive_loss != 0
    }
    pub(crate) fn gain(&self) -> i64 {
        phase::floor(self.net, 0)
    }
}

impl ScorpK1 {
    pub(crate) fn section1231_gain(&self) -> i64 {
        self.business_property().ok().flatten().map_or(0, |v| v.gain)
    }
    pub(crate) fn report_section1231_in(
        &self,
        basis: &BasisResult,
        id: i64,
        amt: bool,
        result: &mut FederalResult,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) {
        let Ok(Some(source)) = self.business_property() else {
            return;
        };
        let risk = match self
            .risk
            .calculate_in(self, basis, None, capacity)
            .map_err(|error| error.message())
        {
            Ok(v) => v,
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    result,
                    &crate::IssueContext::income_id(id),
                    "S-corporation section 1231",
                    key,
                    message,
                );
                return;
            }
        };
        let rental_losses = self.rentals.property_losses().iter().sum::<i64>();
        let business_allowed =
            risk.ledger.risk_allowed[K1LossClass::Section1231].get() - rental_losses;
        let message = Message::new(
            if amt { "NOTE_SCORP_AMT_SECTION1231" } else { "NOTE_SCORP_REGULAR_SECTION1231" },
            alloc::vec![
                MessageArg::Number(id),
                MessageArg::usd_cents(source.gain - business_allowed),
                MessageArg::usd_cents(basis.suspended[K1LossClass::Section1231].get()),
                MessageArg::usd_cents(risk.ledger.risk_suspended[K1LossClass::Section1231].get()),
                MessageArg::usd_cents(source.passive_loss),
                MessageArg::usd_cents(source.unrecaptured),
            ],
        );
        result.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)]
            message.english(),
            message,
        ));
        if !amt && self.property_tracking() && self.qbi_left_out().is_none() {
            let source = self.qbi_projection();
            source.property_qbi.report(&source, basis, id, None, result);
        }
    }
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(K1Section1231 {
    amounts: [net, unrecaptured, basis_loss, risk_loss, passive_loss],
    gains: [],
    facts: [],
});
