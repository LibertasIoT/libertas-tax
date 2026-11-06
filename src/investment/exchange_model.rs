use crate::RentalPropertyType;
use crate::dates::source_valid_date as valid_date;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Like-kind exchange of this rental
/// Use your exchange agreement, your intermediary's statement and the closing statements of both properties.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct RentalExchange {
    /// Date you gave up this rental (Form 8824 line 4)
    /// The closing date on which the deed passed to the buyer or your intermediary (Form 1099-S box 1).
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub transferred: u32,
    /// Date you identified the new property in writing (line 5)
    /// The date of the signed written identification you sent to your intermediary or the other party, describing the property clearly and naming no more than three properties, or properties worth no more than twice this rental. If you received the new property within 45 days after you gave this rental up, enter the date you received it.
    #[libertas_date_only]
    // Tax structure: a 2026 section 1031 transfer can receive or identify replacement property no later than its 180-day window ending June 29, 2027.
    #[libertas_number(min = 20250101, max = 20270629)]
    pub identified: u32,
    /// Date you received the new property (line 6)
    #[libertas_date_only]
    // Tax structure: a 2026 section 1031 transfer can receive or identify replacement property no later than its 180-day window ending June 29, 2027.
    #[libertas_number(min = 20260101, max = 20270629)]
    pub received: u32,
    /// You extend your 2026 return to October 15, 2027
    /// Answer only when you received the new property after April 15, 2027: the exchange then counts only if your 2026 return is due later because you filed an extension (Form 4868).
    /// [NullText]
    /// Not needed: replacement property received by April 15, 2027
    /// [SetText]
    /// Extended the 2026 return to October 15, 2027
    #[libertas_default($null)]
    pub extended: Option<bool>,
    /// The property you received
    pub replacement: ExchangeReplacement,
    /// Cash you received (line 15)
    /// Cash paid to you at either closing or released to you by your intermediary during the exchange.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_received: i64,
    /// Value of other things you received (line 15)
    /// Furniture, appliances or anything else that is not real property, at fair market value. A note from the buyer is under Something else applies.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_received: i64,
    /// Loans on this rental taken over or paid off in the exchange (line 15)
    /// Mortgages and other debts secured by this rental that the other side took over or that were paid from the exchange, from your closing statement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub debt_relieved: i64,
    /// Loans you took on for the new property (line 18)
    /// Mortgages on the new property that you took over or took it subject to, and new loans you used to pay for it at closing.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub debt_assumed: i64,
    /// Cash you added (line 18)
    /// Your own money paid toward the new property at closing, beyond the exchange funds.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_paid: i64,
    /// Exchange expenses (lines 15 and 18)
    /// Commissions, legal, title, recording and intermediary fees on both closings. Not loan costs, prorated rent or property tax, or deposits.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub expenses: i64,
    /// Part of the exchange value for this rental's land
    /// Use the closing statement or an appraisal. The land is not depreciated, so its gain is figured separately.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub land_value: i64,
    /// Who was on the other side
    pub party: ExchangeParty,
    /// Something else applies
    /// You also gave up property other than this rental and cash, such as stock, equipment or a note; you gave up or received more than one property; the exchange was a reverse or build-to-suit exchange through an exchange accommodation titleholder; your intermediary defaulted; or you received a note.
    #[libertas_default(false)]
    pub special: bool,
}

/// The property you received
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ExchangeReplacement {
    /// Address (Form 8824 line 2)
    /// Needed only for a printed return.
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub address: String,
    /// Value of the real property you received (line 16)
    /// Its fair market value on the day you received it, normally the price on its closing statement, without closing costs.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub value: i64,
    /// Part of that value for the land
    /// Use the closing statement, an appraisal or the property-tax assessment ratio. Enter the whole value for vacant land.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub land: i64,
    /// How you use it
    pub used: ReplacementUse,
}

/// How you use the property you received
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum ReplacementUse {
    /// As a rental
    /// Enter its rent and expenses on this rental from the day it was ready to rent.
    Rental {
        /// Type of property
        #[libertas_default(SingleFamily)]
        kind: RentalPropertyType,
        /// Date first available to rent
        #[libertas_date_only]
        in_service: u32,
    },
    /// Held for investment, not rented
    /// For example, vacant land you hold for its value.
    Investment,
    /// In your own business
    Business,
    /// As your home or a vacation home
    /// The exchange does not qualify, so it is taxed as a sale.
    Personal,
    /// Real property outside the United States
    /// U.S. and foreign real property are not like kind, so the exchange is taxed as a sale.
    Foreign,
}

/// Who was on the other side
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum ExchangeParty {
    /// Unrelated people or companies
    /// Directly or through a qualified intermediary, and the new property did not come from a relative or a related company.
    Unrelated,
    /// A related person, directly or through an intermediary (Form 8824 Part II)
    /// Your spouse, child, grandchild, parent, grandparent, brother or sister, or a corporation, partnership, trust or estate that you or they own more than 50% of.
    Related {
        /// What happened to the exchanged properties in 2026
        #[libertas_default(Kept)]
        disposal: RelatedDisposal,
    },
    /// A related person sold you the new property through your intermediary
    /// They were paid cash for it while you exchanged through the intermediary (section 1031(f)(4)). Unless an exception applies, the exchange is taxed as a sale.
    RelatedSeller {
        /// Exception (line 11)
        exception: RelatedException,
    },
}

/// What happened to the exchanged properties in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum RelatedDisposal {
    /// Both of you still held them at the end of 2026 (lines 9 and 10 No)
    Kept,
    /// The related person sold or gave away the property they got from you (line 9)
    TheyDisposed {
        /// Exception (line 11)
        exception: RelatedException,
    },
    /// You sold or gave away the property you received (line 10)
    YouDisposed,
}

/// Exception (Form 8824 line 11)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum RelatedException {
    /// None of these
    None,
    /// After the death of either of you (11a)
    Death,
    /// An involuntary conversion, such as a condemnation, first threatened after the exchange (11b)
    InvoluntaryConversion,
    /// Neither the exchange nor the disposition had tax avoidance as a main purpose (11c)
    /// You attach an explanation to your return, and the IRS must agree.
    NoTaxAvoidance,
}

impl RentalExchange {
    fn money(&self) -> bool {
        [
            self.cash_received,
            self.other_received,
            self.debt_relieved,
            self.debt_assumed,
            self.cash_paid,
            self.expenses,
            self.land_value,
        ]
        .iter()
        .all(|v| (0..=100_000_000_000).contains(v))
    }
    // The server twin of the structural client rules and ranges.
    pub(crate) fn valid(&self) -> bool {
        let replacement = &self.replacement;
        self.money()
            && (20260101..=20261231).contains(&self.transferred)
            && valid_date(self.transferred)
            && (20250101..=20270629).contains(&self.identified)
            && valid_date(self.identified)
            && self.identified <= self.received
            && (20260101..=20270629).contains(&self.received)
            && valid_date(self.received)
            && (self.received <= crate::year_constants::EXCHANGE_RETURN_DUE_DATE
                || self.extended.is_some())
            && (1..=100_000_000_000).contains(&replacement.value)
            && (0..=replacement.value).contains(&replacement.land)
            && match replacement.used {
                ReplacementUse::Rental { in_service, .. } => {
                    valid_date(in_service)
                        && (self.received..=20271231).contains(&in_service)
                        && replacement.land < replacement.value
                }
                _ => true,
            }
            && self.land_value
                <= replacement.value + self.cash_received + self.other_received + self.debt_relieved
    }
    // The key of the first page rule the exchange breaks, in the order of the
    // client rules.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        let replacement = &self.replacement;
        if self.identified > self.received {
            return Some(
                crate::validation::ValidationIssue::new("EXCHANGE_IDENTIFIED_AFTER_RECEIVED")
                    .at_field("identified"),
            );
        }
        if replacement.land > replacement.value {
            return Some(
                crate::validation::ValidationIssue::new("EXCHANGE_LAND_ABOVE_VALUE")
                    .at_field("land")
                    .in_field("replacement"),
            );
        }
        if let ReplacementUse::Rental { in_service, .. } = replacement.used {
            if replacement.land >= replacement.value {
                return Some(
                    crate::validation::ValidationIssue::new("EXCHANGE_RENTAL_WITHOUT_BUILDING")
                        .at_field("land")
                        .in_field("replacement"),
                );
            }
            if in_service < self.received {
                return Some(
                    crate::validation::ValidationIssue::new("EXCHANGE_RENTED_BEFORE_RECEIVED")
                        .at_field("in_service")
                        .in_field("used")
                        .in_field("replacement"),
                );
            }
            if in_service > 20271231 {
                return Some(
                    crate::validation::ValidationIssue::new("EXCHANGE_ENTER_THE_RENTAL_DATE")
                        .at_field("in_service")
                        .in_field("used")
                        .in_field("replacement"),
                );
            }
        }
        if i128::from(self.land_value)
            > i128::from(replacement.value)
                + i128::from(self.cash_received)
                + i128::from(self.other_received)
                + i128::from(self.debt_relieved)
        {
            return Some(
                crate::validation::ValidationIssue::new("EXCHANGE_LAND_VALUE_ABOVE_TOTAL")
                    .at_field("land_value"),
            );
        }
        if self.received > crate::year_constants::EXCHANGE_RETURN_DUE_DATE
            && self.extended.is_none()
        {
            return Some(
                crate::validation::ValidationIssue::new("EXCHANGE_SAY_WHETHER_YOU_EXTEND")
                    .at_field("extended"),
            );
        }
        None
    }
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
}

// Community-property projection; accepted source facts are retained.

crate::community::divide_record!(RentalExchange {
    amounts: [
        replacement,
        cash_received,
        other_received,
        debt_relieved,
        debt_assumed,
        cash_paid,
        expenses,
        land_value
    ],
    gains: [],
    facts: [transferred, identified, received, extended, party, special],
});
crate::community::divide_record!(ExchangeReplacement {
    amounts: [value, land],
    gains: [],
    facts: [address, used],
});
