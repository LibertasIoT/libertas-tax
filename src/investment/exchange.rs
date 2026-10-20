// Coverage: Form 8824 — Supported (unit B2, 2026-10-09; 2026 draft form of 2026-06-09, 2025
//   instructions of 2025-12-02, Pub 544 ch. 1, Pub 946 ch. 4, Reg. 1.1031(k)-1 and 1.168(i)-6).
//   Lines: a whole rental property exchanged for one other real property in 2026
//   (RentalSale::Exchanged on the rental page), directly or through a qualified intermediary.
//   Part I: line 4 the transfer date, line 5 the identification date and line 6 the receipt date
//   (45 days / 180 days / the return due date, with the extension answer after April 15, 2027);
//   lines 1-3 and 8 are printing only and not stored. Part II: line 7 the related-party choice,
//   lines 9-10 the 2026 disposal, line 11 the exception. Part III: lines 15-25a, with the
//   given-up rental's basis derived from its building records (asset_sale::accumulated through
//   the transfer month): line 15 cash, other property and net liabilities less expenses, 16-17,
//   18 basis plus net amounts paid plus unused expenses, 19-24, 25 the basis of the property
//   received and 25a its building part; line 21 is always zero (post-1986 straight-line
//   section 1250 property has no additional depreciation; Form 4797 line 17 in the 2026 draft),
//   line 22 is section 1231 gain on Form 4797 line 5 with its unrecaptured section 1250 gain
//   (the smaller of line 22 and the building gain, limited by depreciation), and the replacement
//   continues on the same rental entry (Reg. 1.168(i)-6: the exchanged basis keeps the
//   remaining recovery period of the building given up, the excess basis is new property placed
//   in service on its first rental day; Form 4562 line 17 and lines 19i/19j; section 1223 holding
//   period; section 469 history stays with the activity; QBI UBIA carries over). A failed
//   exchange (identification after 45 days, receipt after 180 days or the return due date, a
//   home or foreign replacement, a related seller or a related person's disposal within two
//   years without an exception) is taxed as a sale through the rental sale split and the
//   replacement's basis is its value (Reg. 1.1031(k)-1(b)(1), section 1031(f)(4)). An earlier
//   related-party exchange (last transfer in 2024 or 2025) whose property was disposed of in 2026
//   within two years recognizes its deferred gain on the Investment sales kind
//   (FederalScreening.related_exchanges) by character: Schedule D lines 4/11, or its own
//   section 1231 source on Form 4797 line 5 with the netting, lookback, rate classes, NIIT and
//   passive treatment of the installment earlier-year sources. Part IV (section 1043) and
//   exchanges of business, farm or investment real property that is not a rental stay
//   RareSituation::LikeKindExchange. Refused by name through RentalExchange.special: other
//   property given up (lines 12-14), more than one property, reverse and build-to-suit
//   exchanges, an intermediary's default, a note received (section 453(f)(6)); also a
//   replacement used in the filer's business, the replacement disposed of in 2026 (section
//   1031(f)(1)(C)(ii)), a tolled two-year period (section 1031(g)) or ordinary recapture
//   (RelatedExchange.special), a rental with personal use, a building held one year or less, and a
//   replacement property type the rental entry does not calculate. The 1.168(i)-6(i) election
//   out is not offered. A rental acquired in a pre-2026 exchange is left out by the rental scope
//   confirmation (WARN_RENTAL_LEFT_OUT); its carryover record is a later unit.
//   STORAGE_FORMAT: set by the dispatcher. Tests (hand-computed): src/tests/investment/exchange.rs
//   exchange_no_boot_carries_basis_and_depreciates_the_excess,
//   exchange_boot_with_liabilities_is_unrecaptured_section_1250_gain,
//   exchange_commercial_building_that_lost_value_has_no_unrecaptured_gain,
//   exchange_related_party_boundaries_and_exceptions,
//   exchange_earlier_year_related_follow_up_two_year_boundary,
//   exchange_deadlines_forty_five_one_eighty_and_the_extension,
//   exchange_consistency_checks_and_refusals; src/tests/investment/interview.rs exchange_* (pages).
//   https://www.irs.gov/pub/irs-dft/f8824--dft.pdf
//   https://www.irs.gov/instructions/i8824
//   https://www.irs.gov/publications/p544
//   https://www.irs.gov/publications/p946
use crate::dates::{add_years, days_between};
use crate::extended_rules::depreciation_for_year;
use crate::federal_rules::valid_date;
use crate::kernel::cost_recovery::historical::rental_elapsed_half_months;
use crate::kernel::lines::push_line;
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FederalDraft, FederalRental, FederalResult, FormLine, Owner, RentalBuilding, TaxForm};
use crate::{IssueContext, Message, MessageArg, RentalPropertyType};
use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

type Refusal = (&'static str, &'static str);

const SOURCE: &str = "https://www.irs.gov/forms-pubs/about-form-8824";

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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub cash_received: i64,
    /// Value of other things you received (line 15)
    /// Furniture, appliances or anything else that is not real property, at fair market value. A note from the buyer is under Something else applies.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub other_received: i64,
    /// Loans on this rental taken over or paid off in the exchange (line 15)
    /// Mortgages and other debts secured by this rental that the other side took over or that were paid from the exchange, from your closing statement.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub debt_relieved: i64,
    /// Loans you took on for the new property (line 18)
    /// Mortgages on the new property that you took over or took it subject to, and new loans you used to pay for it at closing.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub debt_assumed: i64,
    /// Cash you added (line 18)
    /// Your own money paid toward the new property at closing, beyond the exchange funds.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub cash_paid: i64,
    /// Exchange expenses (lines 15 and 18)
    /// Commissions, legal, title, recording and intermediary fees on both closings. Not loan costs, prorated rent or property tax, or deposits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub expenses: i64,
    /// Part of the exchange value for this rental's land
    /// Use the closing statement or an appraisal. The land is not depreciated, so its gain is figured separately.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 1)]
    pub value: i64,
    /// Part of that value for the land
    /// Use the closing statement, an appraisal or the property-tax assessment ratio. Enter the whole value for vacant land.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

/// Related-party exchange from 2024 or 2025
/// One for each like-kind exchange with a related person whose last transfer was in 2024 or 2025 and whose property was sold or given away in 2026. Use the Form 8824 you filed for the year of the exchange.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct RelatedExchange {
    /// Property you gave up (Form 8824 line 1)
    #[libertas_size(min = 1)]
    pub property: String,
    /// Date of the last transfer in the exchange
    /// The later of lines 4 and 6 of that Form 8824.
    #[libertas_date_only]
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 20240101, max = 20251231)]
    pub last_transfer: u32,
    /// Date the related person sold or gave away the property they got from you (line 9)
    /// If you sold or gave away the property you received, choose Something else applies.
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub disposed: u32,
    /// Exception (line 11)
    pub exception: RelatedException,
    /// Deferred gain (line 24)
    /// Line 24 of the Form 8824 for the year of the exchange. Enter zero for a deferred loss: a loss on an exchange with a related person is never deductible.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub deferred: i64,
    /// What you gave up
    /// For a building or other depreciated real property, its unrecaptured section 1250 gain is the smaller of its depreciation and its gain, less any part you already reported.
    pub kind: crate::PropertyCharacter,
    /// Something else applies
    /// You sold or gave away the property you received; a put, an option or a short sale reduced either of your risk of loss; or that exchange's gain included depreciation recapture taxed as ordinary income.
    #[libertas_default(false)]
    pub special: bool,
}

/// Related-party exchange from an earlier year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EarlierRelatedExchange {
    /// Filer
    pub owner: Owner,
    /// Exchange
    pub exchange: RelatedExchange,
}

// Why an exchange is taxed as a sale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SaleReason {
    IdentifiedLate,
    ReceivedLate,
    Personal,
    Foreign,
    Related,
}
impl SaleReason {
    fn key(self) -> &'static str {
        match self {
            Self::IdentifiedLate => "EXCHANGE_REASON_IDENTIFIED_LATE",
            Self::ReceivedLate => "EXCHANGE_REASON_RECEIVED_LATE",
            Self::Personal => "EXCHANGE_REASON_PERSONAL",
            Self::Foreign => "EXCHANGE_REASON_FOREIGN",
            Self::Related => "EXCHANGE_REASON_RELATED",
        }
    }
}

// What the facts make of the exchange.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    // Section 1031 applies. `kept` is a related-party exchange both sides
    // held; `unreviewed` one accepted on the no-tax-avoidance exception.
    Exchange { kept: bool, unreviewed: bool },
    // No like-kind property was received: the exchange is a sale. `shown`
    // keeps Form 8824 Part III lines 15-24 (a related person's disposal).
    Sale { reason: SaleReason, shown: bool },
}

// Form 8824 Part III, whole dollars, for the rental given up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Form8824 {
    pub line15: i64,
    pub line16: i64,
    pub line17: i64,
    pub line18: i64,
    pub line19: i64,
    pub line20: i64,
    pub line21: i64,
    pub line22: i64,
    pub line23: i64,
    pub line24: i64,
    pub line25: i64,
    pub line25a: i64,
    // Adjusted basis of the rental given up (land and buildings), and its
    // parts: the buildings' cost, their depreciation allowed or allowable
    // through the transfer month and the land cost.
    pub adjusted: i64,
    pub building_cost: i64,
    pub depreciation: i64,
    pub land_cost: i64,
    // The amount realized (the exchange value of the rental after
    // expenses), the building gain on the Form 4797 Part III split and
    // the unrecaptured section 1250 gain limit.
    pub realized: i64,
    pub building_gain: i64,
    pub unrecaptured_limit: i64,
    // Reg. 1.168(i)-6(b)(7)-(8): the exchanged basis and the excess basis
    // of the buildings received.
    pub exchanged_basis: i64,
    pub excess_basis: i64,
}

// The replacement's 2026 depreciation, cents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Replacement {
    pub carryover: i64,
    pub excess: i64,
}
impl Replacement {
    pub(crate) fn total(&self) -> i64 {
        self.carryover + self.excess
    }
}

fn invalid() -> Refusal {
    (
        "ISSUE_EXCHANGE_DATES_OR_AMOUNTS_INVALID",
        crate::fixture_english!(
            "Enter the exchange dates in order, amounts within the supported limits, and a land part no larger than the value you received for this rental."
        ),
    )
}

// Half-months the building was in service through the transfer month.
fn elapsed(b: &RentalBuilding, transferred: u32) -> i64 {
    let first = rental_elapsed_half_months(b.in_service, b.in_service / 10000);
    let window = 2 * i64::from(transferred / 100 % 100) - 1;
    if transferred / 10000 <= b.in_service / 10000 {
        first.min(window)
    } else {
        rental_elapsed_half_months(b.in_service, transferred / 10000) - 24 + window
    }
}

// All rental consumers select the original clock through its shared owner.
fn period(
    kind: &RentalPropertyType,
    placed: u32,
    source: Option<&crate::Commercial1993Transition>,
    amt: bool,
) -> Result<i64, Refusal> {
    crate::historical_recovery::rental_building_recovery(kind, placed, source, amt)
        .map(|recovery| recovery.half_months())
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
    fn days_after_transfer(&self, date: u32) -> i32 {
        days_between(self.transferred, date)
    }
    // Section 1031(a)(3), (f) and (h): whether like-kind property was received.
    pub(crate) fn outcome(&self) -> Outcome {
        use crate::year_constants::{
            EXCHANGE_IDENTIFICATION_DAYS, EXCHANGE_RECEIPT_DAYS, EXCHANGE_RETURN_DUE_DATE,
        };
        let sale = |reason, shown| Outcome::Sale { reason, shown };
        match &self.replacement.used {
            ReplacementUse::Foreign => return sale(SaleReason::Foreign, false),
            ReplacementUse::Personal => return sale(SaleReason::Personal, false),
            _ => {}
        }
        // Property received within 45 days is treated as identified.
        if self.days_after_transfer(self.identified) > EXCHANGE_IDENTIFICATION_DAYS
            && self.days_after_transfer(self.received) > EXCHANGE_IDENTIFICATION_DAYS
        {
            return sale(SaleReason::IdentifiedLate, false);
        }
        if self.days_after_transfer(self.received) > EXCHANGE_RECEIPT_DAYS
            || (self.received > EXCHANGE_RETURN_DUE_DATE && self.extended != Some(true))
        {
            return sale(SaleReason::ReceivedLate, false);
        }
        let excepted = |exception: RelatedException| match exception {
            RelatedException::None => None,
            RelatedException::NoTaxAvoidance => Some(true),
            _ => Some(false),
        };
        match &self.party {
            ExchangeParty::Unrelated => Outcome::Exchange { kept: false, unreviewed: false },
            ExchangeParty::Related { disposal: RelatedDisposal::Kept } => {
                Outcome::Exchange { kept: true, unreviewed: false }
            }
            // Refused by name; the exchange itself is unchanged.
            ExchangeParty::Related { disposal: RelatedDisposal::YouDisposed } => {
                Outcome::Exchange { kept: false, unreviewed: false }
            }
            ExchangeParty::Related { disposal: RelatedDisposal::TheyDisposed { exception } } => {
                match excepted(*exception) {
                    None => sale(SaleReason::Related, true),
                    Some(unreviewed) => Outcome::Exchange { kept: false, unreviewed },
                }
            }
            ExchangeParty::RelatedSeller { exception } => match excepted(*exception) {
                None => sale(SaleReason::Related, false),
                Some(unreviewed) => Outcome::Exchange { kept: false, unreviewed },
            },
        }
    }
    // The exchange facts this App does not calculate; None when supported.
    pub(crate) fn refusal(&self, rental: &FederalRental) -> Option<Refusal> {
        if !self.valid() {
            return Some(invalid());
        }
        if self.special {
            return Some((
                "ISSUE_EXCHANGE_SOMETHING_ELSE",
                crate::fixture_english!(
                    "This like-kind exchange is not calculated: other property given up, more than one property, a reverse or build-to-suit exchange, an intermediary's default or a note received need their own statements (Form 8824 lines 12–18, Publication 544)."
                ),
            ));
        }
        if self.received < self.transferred {
            return Some((
                "ISSUE_EXCHANGE_RECEIVED_BEFORE_TRANSFER",
                crate::fixture_english!(
                    "Receiving the new property before giving this rental up is a reverse exchange through an accommodation titleholder, which is not calculated (Revenue Procedure 2000-37)."
                ),
            ));
        }
        if matches!(self.party, ExchangeParty::Related { disposal: RelatedDisposal::YouDisposed }) {
            return Some((
                "ISSUE_EXCHANGE_REPLACEMENT_DISPOSED",
                crate::fixture_english!(
                    "You sold or gave away in 2026 the property you received from a related person; that sale and the exchange gain it triggers are not calculated (section 1031(f), Form 8824 line 10)."
                ),
            ));
        }
        if self.replacement.used == ReplacementUse::Business {
            return Some((
                "ISSUE_EXCHANGE_REPLACEMENT_IN_BUSINESS",
                crate::fixture_english!(
                    "The new property is used in your own business; its depreciation belongs to that business and is not calculated (Form 8824, Form 4562)."
                ),
            ));
        }
        self.form(rental, false).err()
    }
    // Form 8824 Part III for the rental given up. `amt` figures the basis
    // and the depreciation allowed under the alternative minimum tax.
    pub(crate) fn form(&self, v: &FederalRental, amt: bool) -> Result<Form8824, Refusal> {
        let mut cost = 0;
        let mut land = 0;
        let mut depreciation = 0;
        for b in &v.buildings {
            cost += b.cost - b.land;
            land += b.land;
            depreciation +=
                crate::asset_sale::accumulated(b, amt, &v.property_type, self.transferred)?;
        }
        let (building_cost, land_cost, depreciation) =
            (dollars(cost), dollars(land), dollars(depreciation));
        let adjusted = land_cost + building_cost - depreciation;
        let (cash, other) = (dollars(self.cash_received), dollars(self.other_received));
        let (relieved, assumed) = (dollars(self.debt_relieved), dollars(self.debt_assumed));
        let (paid, expenses) = (dollars(self.cash_paid), dollars(self.expenses));
        let value = dollars(self.replacement.value);
        // Instructions lines 15 and 18: liabilities assumed, cash paid
        // and other property given up offset liabilities relieved, never
        // cash received; expenses reduce line 15 first.
        let received = cash + other + positive(relieved - (assumed + paid));
        let line15 = positive(received - expenses);
        let used_on_15 = phase::cap(expenses, [received]);
        let line16 = value;
        let line17 = line15 + line16;
        let line18 = adjusted + positive(assumed + paid - relieved) + (expenses - used_on_15);
        let line19 = line17 - line18;
        let line20 = positive(phase::cap(line15, [line19]));
        // Post-1986 straight-line section 1250 property has no additional
        // depreciation, so no ordinary income under the recapture rules.
        let line21 = 0;
        let line22 = positive(line20 - line21);
        let line23 = line21 + line22;
        let line24 = line19 - line23;
        let line25 = line18 + line23 - line15;
        let building_value = value - dollars(self.replacement.land);
        let line25a = if value > 0 { fraction(line25, building_value, value) } else { 0 };
        // Reg. 1.168(i)-6(b)(7): the exchanged basis is the lesser of the
        // basis of the property received and the given-up buildings'
        // adjusted depreciable basis; the rest is excess basis.
        let depreciable = building_cost - depreciation;
        let exchanged_basis = phase::bound(line25a, 0, positive(depreciable));
        let excess_basis = positive(line25a - exchanged_basis);
        let realized = line19 + adjusted;
        let land_value = dollars(self.land_value);
        if realized < 0 || land_value > realized {
            return Err(invalid());
        }
        let building_gain = (realized - land_value) - depreciable;
        Ok(Form8824 {
            line15,
            line16,
            line17,
            line18,
            line19,
            line20,
            line21,
            line22,
            line23,
            line24,
            line25,
            line25a,
            adjusted,
            building_cost,
            depreciation,
            land_cost,
            realized,
            building_gain,
            unrecaptured_limit: phase::cap(phase::floor(building_gain, 0), [depreciation]),
            exchanged_basis,
            excess_basis,
        })
    }
    // The exchange as a disposition of the rental, for the Form 4797 netting:
    // a failed exchange recognizes the whole gain through the rental sale
    // split, a qualifying one only line 23.
    pub(crate) fn gain(
        &self,
        v: &FederalRental,
        amt: bool,
    ) -> Result<crate::asset_sale::RentalGain, Refusal> {
        let form = self.form(v, amt)?;
        let sale = matches!(self.outcome(), Outcome::Sale { .. });
        let land_value = dollars(self.land_value);
        Ok(crate::asset_sale::RentalGain {
            realized: form.realized - land_value,
            basis: form.building_cost,
            depreciation: form.depreciation,
            building: form.building_gain,
            land_realized: land_value,
            land_basis: form.land_cost,
            land: land_value - form.land_cost,
            unrecaptured: if sale {
                form.unrecaptured_limit
            } else {
                phase::cap(form.unrecaptured_limit, [form.line22])
            },
            recognized: if sale { form.line19 } else { form.line23 },
            gross: if sale {
                phase::floor(form.building_gain, 0)
                    + phase::floor(
                        crate::kernel::carry::native_balance(land_value, form.land_cost),
                        0,
                    )
            } else {
                phase::floor(form.line23, 0)
            },
            // An exchange of a rental with a rehabilitation credit is refused.
            additional: 0,
            ordinary: 0,
            installment: None,
            ..Default::default()
        })
    }
    // Whether the exchange is a fully taxable disposition that releases the
    // activity's suspended passive losses (section 469(g)(1)(A), Pub 925):
    // a sale, but not to a related person.
    pub(crate) fn releases_losses(&self) -> bool {
        matches!(
            self.outcome(),
            Outcome::Sale { reason, .. } if reason != SaleReason::Related
        )
    }
    // The replacement's 2026 depreciation, cents. The exchanged basis keeps
    // the remaining recovery period of each building given up (Reg.
    // 1.168(i)-6(c)(3)-(5): the longer period when the new building has
    // one); the excess basis, or the whole cost after a failed exchange, is
    // new property placed in service on its first rental day.
    pub(crate) fn replacement_depreciation(
        &self,
        v: &FederalRental,
        amt: bool,
    ) -> Result<Replacement, Refusal> {
        let ReplacementUse::Rental { ref kind, in_service } = self.replacement.used else {
            return Ok(Replacement::default());
        };
        if in_service / 10000 != 2026 || !valid_date(in_service) || !self.valid() {
            return Ok(Replacement::default());
        }
        let synthetic = |cost, land| RentalBuilding {
            commercial_1993_transition: None,
            casualty: None,
            id: 0,
            energy: None,
            sale_history: None,
            credit_reduction: 0,
            cost,
            land,
            in_service,
            acquired: crate::RentalAcquisition::Purchased,
            rehabilitation: None,
            partial: None,
        };
        let new = |cost, land| {
            depreciation_for_year(
                &synthetic(cost, land),
                amt,
                kind,
                2026,
                None,
                crate::kernel::source_fact::SourceFact::NotReported,
            )
        };
        if matches!(self.outcome(), Outcome::Sale { .. }) {
            return Ok(Replacement {
                carryover: 0,
                excess: new(self.replacement.value, self.replacement.land)?,
            });
        }
        // A source failure remains unavailable for replacement depreciation.
        let form = self.form(v, amt)?;
        let half_months = rental_elapsed_half_months(in_service, in_service / 10000);
        let mut weights = Vec::new();
        for b in &v.buildings {
            let given_up =
                crate::asset_sale::accumulated(b, amt, &v.property_type, self.transferred)?;
            let lifetime =
                period(&v.property_type, b.in_service, b.commercial_1993_transition.as_ref(), amt)?;
            weights.push((
                phase::floor(b.cost - b.land - given_up, 0),
                phase::floor(lifetime, period(kind, in_service, None, amt)?)
                    - elapsed(b, self.transferred),
            ));
        }
        let total: i64 = weights.iter().map(|w| w.0).sum();
        let mut carryover = 0;
        for (basis, remaining) in weights {
            if total > 0 && remaining > 0 {
                let part = fraction(form.exchanged_basis * 100, basis, total);
                carryover += fraction(part, half_months.min(remaining), remaining);
            }
        }
        Ok(Replacement {
            carryover,
            excess: if form.excess_basis > 0 { new(form.excess_basis * 100, 0)? } else { 0 },
        })
    }
    // Unadjusted basis immediately after acquisition of the building held at
    // the end of 2026, cents (Reg. 1.199A-2(c)(3)): the buildings given up
    // keep theirs in proportion to the exchanged basis and their placed-in-
    // service dates, the excess basis or the whole cost after a failed
    // exchange is new.
    pub(crate) fn ubia(&self, v: &FederalRental) -> Result<i64, Refusal> {
        let ReplacementUse::Rental { in_service, .. } = self.replacement.used else {
            return Ok(0);
        };
        if self.received > 20261231 || in_service > 20261231 || !self.valid() {
            return Ok(0);
        }
        if matches!(self.outcome(), Outcome::Sale { .. }) {
            return Ok(self.replacement.value - self.replacement.land);
        }
        let form = self.form(v, false)?;
        let depreciable = form.building_cost - form.depreciation;
        let carried = if depreciable > 0 {
            v.buildings.iter().try_fold(0, |carried, b| {
                let lifetime = period(
                    &v.property_type,
                    b.in_service,
                    b.commercial_1993_transition.as_ref(),
                    false,
                )?;
                Ok::<_, Refusal>(
                    carried
                        + if rental_elapsed_half_months(b.in_service, 2026) < lifetime {
                            fraction(b.cost - b.land, form.exchanged_basis, depreciable)
                        } else {
                            0
                        },
                )
            })?
        } else {
            0
        };
        Ok(carried + form.excess_basis * 100)
    }
}

fn name_arg(id: i64, label: &str) -> MessageArg {
    if label.trim().is_empty() {
        MessageArg::Nested(Box::new(IssueContext::income_id(id).label()))
    } else {
        MessageArg::Text(label.into())
    }
}

fn row(
    r: &mut FederalResult,
    form: TaxForm,
    line: &str,
    id: &str,
    amount: i64,
    explanation: Message,
) {
    push_line(
        r,
        Some(FormLine::new(form, line)),
        id,
        amount * 100,
        &["income.rental.sale"],
        SOURCE,
        explanation,
    );
}

fn note(r: &mut FederalResult, id: i64, key: &str, args: Vec<MessageArg>) {
    crate::federal_rules::note_once_scoped(r, Message::new(key, args), IssueContext::income_id(id));
}

impl RentalExchange {
    // The Form 8824 rows and notes of an exchanged rental that passed the
    // refusals; a failed exchange adds its sale rows in asset_sale.
    pub(crate) fn report(&self, r: &mut FederalResult, id: i64, label: &str, v: &FederalRental) {
        let Ok(form) = self.form(v, false) else {
            return;
        };
        let name = name_arg(id, label);
        match self.outcome() {
            Outcome::Sale { reason, shown } => {
                note(
                    r,
                    id,
                    "WARN_EXCHANGE_TAXED_AS_SALE",
                    vec![name, MessageArg::Resource(reason.key().into())],
                );
                if shown {
                    self.part_three(r, id, &form, false);
                }
            }
            Outcome::Exchange { kept, unreviewed } => {
                if kept {
                    note(r, id, "NOTE_RELATED_EXCHANGE_TWO_YEARS", vec![name.clone()]);
                }
                if unreviewed {
                    note(r, id, "WARN_EXCHANGE_NO_TAX_AVOIDANCE", vec![name.clone()]);
                }
                if matches!(self.replacement.used, ReplacementUse::Investment) && v.history.any() {
                    note(r, id, "NOTE_EXCHANGE_LOSSES_STAY_SUSPENDED", vec![name]);
                }
                self.part_three(r, id, &form, true);
                if form.line22 > 0 {
                    row(
                        r,
                        TaxForm::Form4797,
                        "5",
                        &format!("rental_{id}_exchange_section1231"),
                        form.line22,
                        msg!("EXPLAIN_EXCHANGE_SECTION1231"),
                    );
                    push_line(
                        r,
                        None,
                        &format!("rental_{id}_exchange_unrecaptured_1250"),
                        phase::cap(form.unrecaptured_limit, [form.line22]) * 100,
                        &["income.rental.sale"],
                        "https://www.irs.gov/instructions/i1040sd",
                        msg!("EXPLAIN_N_UNRECAPTURED_N"),
                    );
                }
            }
        }
    }
    // Lines 15-24, and 25 and 25a for the property that keeps its basis.
    fn part_three(&self, r: &mut FederalResult, id: i64, f: &Form8824, basis: bool) {
        let mut lines = vec![
            ("15", "line15", f.line15, msg!("EXPLAIN_EXCHANGE_LINE15")),
            ("16", "line16", f.line16, msg!("EXPLAIN_EXCHANGE_LINE16")),
            ("17", "line17", f.line17, msg!("EXPLAIN_EXCHANGE_LINE17")),
            ("18", "line18", f.line18, msg!("EXPLAIN_EXCHANGE_LINE18")),
            ("19", "line19", f.line19, msg!("EXPLAIN_EXCHANGE_LINE19")),
            ("20", "line20", f.line20, msg!("EXPLAIN_EXCHANGE_LINE20")),
            ("21", "line21", f.line21, msg!("EXPLAIN_EXCHANGE_LINE21")),
            ("22", "line22", f.line22, msg!("EXPLAIN_EXCHANGE_LINE22")),
            ("23", "line23", f.line23, msg!("EXPLAIN_EXCHANGE_LINE23")),
            ("24", "line24", f.line24, msg!("EXPLAIN_EXCHANGE_LINE24")),
        ];
        if basis {
            lines.push(("25", "line25", f.line25, msg!("EXPLAIN_EXCHANGE_LINE25")));
            lines.push(("25a", "line25a", f.line25a, msg!("EXPLAIN_EXCHANGE_LINE25A")));
        }
        for (line, suffix, amount, explanation) in lines {
            row(
                r,
                TaxForm::Form8824,
                line,
                &format!("rental_{id}_exchange_{suffix}"),
                amount,
                explanation,
            );
        }
    }
}

// ---- Related-party exchanges from an earlier year ----

impl RelatedExchange {
    pub(crate) fn valid(&self) -> bool {
        !self.property.is_empty()
            && (20240101..=20251231).contains(&self.last_transfer)
            && valid_date(self.last_transfer)
            && (20260101..=20261231).contains(&self.disposed)
            && valid_date(self.disposed)
            && self.last_transfer < self.disposed
            && (0..=100_000_000_000).contains(&self.deferred)
            && match self.kind {
                crate::PropertyCharacter::BusinessProperty { unrecaptured, .. } => {
                    (0..=100_000_000_000).contains(&unrecaptured)
                }
                _ => true,
            }
    }
    #[cfg(test)]
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.last_transfer >= self.disposed).then_some(
            crate::validation::ValidationIssue::new("RELATED_EXCHANGE_DISPOSED_BEFORE_TRANSFER")
                .at_field("disposed"),
        )
    }
    pub(crate) fn supported(&self) -> bool {
        self.valid() && !self.special
    }
    // Section 1031(f)(1): the disposition is before the date two years after
    // the last transfer (the anniversary itself is outside), and no
    // exception applies.
    pub(crate) fn taxable(&self) -> bool {
        self.exception == RelatedException::None && self.disposed < add_years(self.last_transfer, 2)
    }
    // The deferred gain recognized in 2026 and its unrecaptured section 1250
    // part, whole dollars.
    pub(crate) fn recognized(&self) -> (i64, i64) {
        if !self.taxable() {
            return (0, 0);
        }
        let gain = dollars(self.deferred);
        let unrecaptured = match self.kind {
            crate::PropertyCharacter::BusinessProperty { unrecaptured, .. } => {
                phase::cap(dollars(unrecaptured), [gain])
            }
            _ => 0,
        };
        (gain, unrecaptured)
    }
}

fn earlier(draft: &FederalDraft) -> impl Iterator<Item = (usize, &RelatedExchange)> {
    draft
        .screening
        .iter()
        .flat_map(|s| s.related_exchanges.iter())
        .map(|v| &v.exchange)
        .enumerate()
        .filter(|(_, v)| v.supported())
}

// Schedule D gains in cents, [short-term, long-term].
// Preserve the retained filer alongside the very same recognized gain.
pub(crate) fn related_capital_sources(draft: &FederalDraft) -> ([i64; 2], [[i64; 2]; 2]) {
    let mut total = [0, 0];
    let mut owners = [[0; 2]; 2];
    for record in
        draft.screening.iter().flat_map(|s| &s.related_exchanges).filter(|v| v.exchange.supported())
    {
        let exchange = &record.exchange;
        let (gain, _) = exchange.recognized();
        let term = match exchange.kind {
            crate::PropertyCharacter::ShortTermCapital => 0,
            crate::PropertyCharacter::LongTermCapital => 1,
            crate::PropertyCharacter::BusinessProperty { .. } => continue,
        };
        total[term] += gain * 100;
        owners[usize::from(record.owner == crate::Owner::Spouse)][term] += gain * 100;
    }
    (total, owners)
}

// Distinct from the earlier-year installment sources' ids (-1, -2, ...) and
// from every income entry's.
fn source_id(index: usize) -> i64 {
    -1001 - index as i64
}

// The same native identity used by related_sources, with the retained filer.
// Ownership needs no second recognition or return calculation.
pub(crate) fn related_source_owner(draft: &FederalDraft, id: i64) -> Option<crate::Owner> {
    draft
        .screening
        .as_ref()?
        .related_exchanges
        .iter()
        .enumerate()
        .find(|(index, record)| {
            source_id(*index) == id
                && record.exchange.supported()
                && matches!(record.exchange.kind, crate::PropertyCharacter::BusinessProperty { .. })
        })
        .map(|(_, record)| record.owner)
}

// Business and rental property as section 1231 sources (Form 4797 line 5),
// with whether each is a business you materially participated in.
pub(crate) fn related_sources(
    draft: &FederalDraft,
) -> Vec<(i64, crate::farm_land::LandGain, bool)> {
    earlier(draft)
        .filter_map(|(index, exchange)| {
            let crate::PropertyCharacter::BusinessProperty { used, .. } = &exchange.kind else {
                return None;
            };
            let (gain, unrecaptured) = exchange.recognized();
            (gain != 0).then(|| {
                (
                    source_id(index),
                    crate::farm_land::LandGain {
                        gain,
                        percent: 0,
                        recapture: 0,
                        ordinary: 0,
                        section1231: gain,
                        unrecaptured,
                    },
                    *used == crate::PropertyUse::Business,
                )
            })
        })
        .collect()
}

// The earlier-year exchanges' rows and refusals. `pool` is the regular-tax
// passive pool; `character` the regular section 1231 result.
pub(crate) fn report(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    r: &mut FederalResult,
) {
    let Some(screening) = draft.screening.as_ref() else {
        return;
    };
    let context = IssueContext::Section(crate::TaxSection::InvestmentSales);
    for (index, value) in screening.related_exchanges.iter().enumerate() {
        let exchange = &value.exchange;
        #[cfg(test)]
        let section = format!("Related-party exchange: {}", exchange.property);
        if !exchange.valid() {
            crate::issue_localized!(
                r,
                &context,
                &section,
                "ISSUE_RELATED_EXCHANGE_DATES_OR_AMOUNTS_INVALID",
                "Enter the last transfer date in 2024 or 2025, the date of the disposition in 2026 and a deferred gain within the supported limits (Form 8824 lines 4, 6, 9 and 24).",
            );
            continue;
        }
        if exchange.special {
            crate::issue_localized!(
                r,
                &context,
                &section,
                "ISSUE_RELATED_EXCHANGE_SOMETHING_ELSE",
                "This related-party exchange is not calculated: your own disposition of the property you received, a reduced risk of loss or ordinary depreciation recapture need their own worksheets (Form 8824 Parts II and III, section 1031(f) and (g)).",
            );
            continue;
        }
        let (gain, unrecaptured) = exchange.recognized();
        if gain == 0 {
            continue;
        }
        let n = index + 1;
        let input = "screening.related_exchanges";
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form8824, "24")),
            &format!("related_exchange_{n}_gain"),
            gain * 100,
            &[input],
            SOURCE,
            msg!("EXPLAIN_RELATED_EXCHANGE_GAIN"),
        );
        if let crate::PropertyCharacter::BusinessProperty { used, .. } = &exchange.kind {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4797, "5")),
                &format!("related_exchange_{n}_section1231"),
                gain * 100,
                &[input],
                "https://www.irs.gov/instructions/i4797",
                msg!("EXPLAIN_EXCHANGE_SECTION1231"),
            );
            if unrecaptured > 0 {
                push_line(
                    r,
                    None,
                    &format!("related_exchange_{n}_unrecaptured_1250"),
                    unrecaptured * 100,
                    &[input],
                    "https://www.irs.gov/instructions/i1040sd",
                    msg!("EXPLAIN_N_UNRECAPTURED_N"),
                );
            }
            match used {
                crate::PropertyUse::Rental if pool.passive_loss > 0 => {
                    crate::issue_localized!(
                        r,
                        &context,
                        &section,
                        "ISSUE_RELATED_EXCHANGE_GAIN_FROM_A_RENTAL_WITH_UNALLOWED_PASSIVE_LOSSES",
                        "Gain from an earlier related-party exchange of rental or passive-activity property is passive income that allows part of this return's unallowed passive losses (Form 8582 lines 1 through 4), which is not calculated.",
                    );
                }
                crate::PropertyUse::Business
                    if character.is_some_and(|v| v.net > 0 && v.ordinary > 0) =>
                {
                    crate::issue_localized!(
                        r,
                        &context,
                        &section,
                        "ISSUE_RELATED_EXCHANGE_BUSINESS_GAIN_WITH_SECTION_1231_LOOKBACK",
                        "Business-property losses from 2021–2025 make part of this year's section 1231 gain ordinary. The part from an earlier related-party exchange of business property needs that business's qualified business income allocation (Form 8995 line 1), which is not calculated.",
                    );
                }
                _ => {}
            }
        }
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
