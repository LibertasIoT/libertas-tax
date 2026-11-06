use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// How you were paid
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SalePayment {
    /// All in 2026
    InFull,
    /// Installment sale: the buyer pays part after 2026
    /// The gain is taxed as the payments arrive; depreciation recapture is taxed in 2026. Include the buyer's note in the sale price, and report the interest the buyer pays as interest income.
    Installment {
        /// Loans the buyer took over
        /// Mortgages and other debts on the property that the buyer assumed or took the property subject to. Not a new loan the buyer got from a bank or from you.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        assumed_debt: i64,
        /// Principal received in 2026
        /// The down payment and the principal of every 2026 payment on the buyer's note, without interest. Include sale money paid at closing to your lender or for your selling costs.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        payments: i64,
        /// Interest the buyer's note charges
        interest: NoteInterest,
        /// Something else applies
        /// A sale by a dealer of property held for customers; a price that depends on later events; a note you pledged as security for a loan; or a note you sold, canceled or gave away in 2026.
        #[libertas_default(false)]
        special: bool,
    },
}

/// Interest the buyer's note charges
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum NoteInterest {
    /// At least the applicable federal rate
    /// The rate the IRS publishes each month for the note's term, when the sale contract was signed.
    Adequate,
    /// A lower rate, or no interest
    Low,
}

impl NoteInterest {
    // Sections 483/1274 are shared by every source's installment note.
    pub(crate) fn refusal(self, price: i64) -> Option<(&'static str, &'static str)> {
        if self == NoteInterest::Low && price > 300_000 {
            return Some((
                "ISSUE_INSTALLMENT_UNSTATED_INTEREST",
                crate::fixture_english!(
                    "An installment note charging less than the applicable federal rate has unstated interest: part of the sale price is interest income (Form 6252 line 5; sections 483 and 1274, Publication 537), which is not calculated."
                ),
            ));
        }
        None
    }
}

impl SalePayment {
    pub(crate) fn installment(&self) -> bool {
        matches!(self, Self::Installment { .. })
    }
    // The payment with the loans the buyer took over and the principal
    // received scaled (cents) to the part of the sale one Form 6252 reports.
    pub(crate) fn scaled(&self, part: impl Fn(i64) -> i64) -> Self {
        match self {
            Self::InFull => Self::InFull,
            Self::Installment { assumed_debt, payments, interest, special } => Self::Installment {
                assumed_debt: part(*assumed_debt),
                payments: part(*payments),
                interest: *interest,
                special: *special,
            },
        }
    }
    // The server twin of INSTALLMENT_PAID_ABOVE_PRICE.
    pub(crate) fn issue(&self, price: i64) -> Option<crate::validation::ValidationIssue> {
        match self {
            Self::Installment { assumed_debt, payments, .. }
                if i128::from(*assumed_debt) + i128::from(*payments) > i128::from(price) =>
            {
                Some(
                    crate::validation::ValidationIssue::new("INSTALLMENT_PAID_ABOVE_PRICE")
                        .at_field("payments"),
                )
            }
            _ => None,
        }
    }
    pub(crate) fn valid(&self, price: i64) -> bool {
        match self {
            Self::InFull => true,
            Self::Installment { assumed_debt, payments, .. } => {
                (0..=100000000000).contains(assumed_debt)
                    && (0..=100000000000).contains(payments)
                    && assumed_debt + payments <= price
            }
        }
    }
    // The refusals every installment fold shares, by the sale price in
    // cents; None when supported or paid in full.
    pub(crate) fn refusal(&self, price: i64) -> Option<(&'static str, &'static str)> {
        let Self::Installment { interest, special, .. } = self else {
            return None;
        };
        if !self.valid(price) {
            return Some((
                "ISSUE_INSTALLMENT_PAYMENTS_EXCEED_PRICE",
                crate::fixture_english!(
                    "Installment sale: the loans the buyer took over and the principal received in 2026 cannot exceed the sale price (Form 6252 lines 5, 6 and 21)."
                ),
            ));
        }
        if *special {
            return Some((
                "ISSUE_INSTALLMENT_SPECIAL_TRANSACTION",
                crate::fixture_english!(
                    "An installment sale by a dealer, with a price that depends on later events, or with a note pledged as security or disposed of in 2026 is not calculated (Form 6252 line 4 and Part III; sections 453(b), 453A(d) and 453B)."
                ),
            ));
        }
        interest.refusal(price)
    }
}

// Community-property projection; accepted source facts are retained.

crate::community::divide_enum!(SalePayment {
    records: [Installment { amounts: [assumed_debt, payments], facts: [interest, special] }],
    units: [InFull],
    tuples: [],
});
