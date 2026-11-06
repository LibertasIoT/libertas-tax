use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Sold or stopped using in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum AssetDisposal {
    /// Still used in this business at the end of 2026
    No,
    /// Sold, scrapped or stopped using in 2026
    /// A trade-in, an exchange or a casualty or theft is not entered here; choose it under Situations people often miss. A sale of the whole business is entered under Sale of the business, beside each asset sold here.
    Disposed {
        /// Date
        #[libertas_date_only]
        // Tax structure: this source event must occur on or before the end of the 2026 return year.
        #[libertas_number(min = 20260101, max = 20261231)]
        date: u32,
        /// What happened
        how: DisposalKind,
    },
}

/// What happened
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DisposalKind {
    /// Sold
    Sold {
        /// Sale price
        /// Cash plus the value of anything else you received, including the buyer's note and a loan the buyer took over.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        proceeds: i64,
        /// Selling expenses
        /// Commissions, advertising and similar costs of the sale.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        selling_expenses: i64,
        /// How you were paid
        #[libertas_default(InFull)]
        payment: crate::SalePayment,
        /// Buyer
        #[libertas_default(Unrelated)]
        buyer: Buyer,
        /// Livestock used for draft, breeding, dairy or sport
        /// Choose cattle or horses, or other livestock, only for animals you kept for those purposes. They are business property after 24 months (cattle and horses) or 12 months (other livestock), not after one year. Choose Not livestock for everything else.
        #[libertas_default(No)]
        livestock: SoldLivestock,
    },
    /// Condemned, or sold under threat of condemnation
    /// Taken by a government or a utility with the power to condemn, or sold to it after it told you it would take the property (section 1033). A loss is deducted as on a sale. A gain is taxed only as far as the award exceeds what you spent, or will spend within two years after the end of 2026 (three for real property), on property similar in service or use; the depreciation recapture is limited to the gain taxed.
    Condemned {
        /// Award
        /// The amount awarded or paid for the property, including severance damages, before expenses.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        award: i64,
        /// Expenses of obtaining the award
        /// Legal, appraisal and similar costs.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        expenses: i64,
        /// Cost of replacement property
        /// What you paid, or will pay within the replacement period, for property similar in service or use. Zero if you are not replacing it; the whole gain is then taxed. If the replacement is land only, the depreciation recapture may be larger than this estimate.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        replacement_cost: i64,
    },
    /// Scrapped or abandoned, nothing received
    Scrapped,
    /// Kept for personal use
    /// Depreciation stops at the date; no gain or loss. A vehicle, or property with a section 179 deduction, whose business use this year is 50% or less has its earlier excess depreciation or section 179 benefit added back (Form 4797 Part IV); enter its earlier business use below.
    PersonalUse,
}

/// Buyer
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum Buyer {
    /// Not related to you
    Unrelated,
    /// Your spouse, brother, sister, parent, grandparent, child or grandchild
    /// A loss on a sale to a relative is not deductible (section 267); a gain is taxed as usual.
    Relative,
    /// A corporation or partnership you own more than half of, or a trust or estate you or your spouse benefit from
    /// A gain on depreciable property sold to such a buyer is ordinary income (section 1239); a loss is not deductible.
    ControlledEntity,
}

/// Part of a building replaced or removed in 2026 (partial disposition)
/// The part's own cost is taken out of the building and its remaining basis is a loss, or a gain if you received more for it, on Form 4797 line 2 (line 11 if held one year or less), with the totals on lines 1b and 1c. The building continues on its cost less this part.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PartialDisposition {
    /// Date replaced, removed or sold
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Original cost of the part
    /// From your records. For a replaced component whose original cost is not known, use the replacement cost discounted to the year the building was placed in service with the Producer Price Index for Finished Goods (Regulations section 1.168(i)-8(f)(3)); keep the computation with the return.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub cost_of_part: i64,
    /// Amount received for the part
    /// Zero for a part scrapped or replaced.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub proceeds: i64,
}

/// Livestock used for draft, breeding, dairy or sport
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum Livestock {
    /// Not livestock
    No,
    /// Cattle or horses
    /// Held 24 months or more from the date you acquired them.
    CattleOrHorse,
    /// Other livestock
    /// Hogs, sheep, goats, mules and similar animals, but not poultry; held 12 months or more from the date you acquired them.
    Other,
}
/// Actual use of the livestock sold
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum LivestockPurpose {
    /// Draft work
    Draft,
    /// Breeding
    Breeding,
    /// Dairy production
    Dairy,
    /// Sport
    Sport,
}

/// Livestock classification and actual use
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum SoldLivestock {
    /// Not livestock
    No,
    /// Cattle or horses
    CattleOrHorse {
        /// Actual use from the property's records
        /// [NullText]
        /// The livestock's use is unavailable
        /// [SetText]
        /// The livestock was used for
        purpose: Option<LivestockPurpose>,
    },
    /// Other livestock
    /// Hogs, sheep, goats, mules and similar animals, but not poultry.
    Other {
        /// Actual use from the property's records
        /// [NullText]
        /// The livestock's use is unavailable
        /// [SetText]
        /// The livestock was used for
        purpose: Option<LivestockPurpose>,
    },
}
impl SoldLivestock {
    pub(crate) fn classification(self) -> Livestock {
        match self {
            Self::No => Livestock::No,
            Self::CattleOrHorse { .. } => Livestock::CattleOrHorse,
            Self::Other { .. } => Livestock::Other,
        }
    }
    pub(crate) fn purpose(self) -> crate::kernel::source_fact::SourceFact<LivestockPurpose> {
        use crate::kernel::source_fact::SourceFact;
        match self {
            Self::No => SourceFact::NotReported,
            Self::CattleOrHorse { purpose } | Self::Other { purpose } => {
                purpose.map_or(SourceFact::Unavailable, SourceFact::Reported)
            }
        }
    }
}
impl From<Livestock> for SoldLivestock {
    fn from(value: Livestock) -> Self {
        match value {
            Livestock::No => Self::No,
            Livestock::CattleOrHorse => Self::CattleOrHorse { purpose: None },
            Livestock::Other => Self::Other { purpose: None },
        }
    }
}

impl Livestock {
    // Section 1231(b)(3): the holding period is counted in months from the
    // acquisition date, so the anniversary itself qualifies (ordinary property
    // must be held more than one year). Whether a sale on `date` of an animal
    // acquired on `acquired` has been held long enough.
    pub(crate) fn held_long_enough(self, acquired: u32, date: u32) -> bool {
        match self {
            Self::No => crate::dates::held_more_than_one_year(acquired, date),
            Self::Other => date >= crate::dates::add_years(acquired, 1),
            Self::CattleOrHorse => date >= crate::dates::add_years(acquired, 2),
        }
    }
}

/// Sold, exchanged or kept in 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum RentalSale {
    /// Kept
    /// Not sold or exchanged in 2026.
    No,
    /// Sold the whole property in 2026
    /// The building, its land and every improvement. A sale of part of the property is entered on the building row as a partial disposition; an exchange for other real property is the next choice. If the property was your main home, enter the sale on Sale of your main home instead.
    Sold {
        /// Date sold
        #[libertas_date_only]
        // Tax structure: this source event must occur on or before the end of the 2026 return year.
        #[libertas_number(min = 20260101, max = 20261231)]
        date: u32,
        /// Sale price
        /// Your share of the contract price, including the buyer's note and any loan the buyer took over (Form 1099-S box 2).
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        price: i64,
        /// Part of the sale price for the land
        /// Use the closing statement or an appraisal. The land is not depreciated, so its gain is figured separately.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        land_price: i64,
        /// Selling expenses
        /// Commissions, legal fees, title and transfer costs you paid at closing.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        selling_expenses: i64,
        /// How you were paid
        #[libertas_default(InFull)]
        payment: crate::SalePayment,
        /// Buyer
        #[libertas_default(Unrelated)]
        buyer: Buyer,
        /// Personal-use share of the property over the years you owned it
        /// Zero for a property you never used personally. Otherwise the average of the personal-use shares of your Schedule E worksheets for the years you owned it (personal days over the days used). That share of the price, expenses and cost is the personal part: its gain is a capital gain and its loss is not deductible.
        // Tax structure: this rental sale must retain a business-use share, so personal use is below 100%.
        #[libertas_number(min = 0, max = 99)]
        #[libertas_default(0)]
        #[libertas_physical_unit("percent")]
        personal_share_percent: u32,
    },
    /// Condemned, or sold under threat of condemnation
    /// Taken by a government or a utility with the power to condemn, or sold to it after it told you it would take the property (section 1033). A loss is deducted as on a sale. A gain is taxed only as far as the award exceeds what you spent, or will spend within three years after the end of 2026, on real property similar in use; the depreciation recapture is limited to the gain taxed. The land part of the award is figured separately because land is not depreciated.
    Condemned {
        /// Date taken or sold
        #[libertas_date_only]
        // Tax structure: this source event must occur on or before the end of the 2026 return year.
        #[libertas_number(min = 20260101, max = 20261231)]
        date: u32,
        /// Award
        /// The amount awarded or paid for the property, including severance damages, before expenses.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        award: i64,
        /// Part of the award for the land
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        land_award: i64,
        /// Expenses of obtaining the award
        /// Legal, appraisal and similar costs.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        expenses: i64,
        /// Cost of replacement property
        /// What you paid, or will pay within three years after the end of 2026, for real property similar in use. Zero if you are not replacing it; the whole gain is then taxed. If the replacement is land only, the depreciation recapture may be larger than this estimate.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        replacement_cost: i64,
    },
    /// Exchanged for other real property in 2026 (like-kind exchange, Form 8824)
    /// The building, its land and every improvement, given up for real property you hold for rent or investment, directly or through a qualified intermediary. Choose this once you have received the new property; until then, enter the transfer as Sold. After the exchange, enter the new property's rent and expenses on this rental; its depreciation is figured from the exchange.
    Exchanged(crate::RentalExchange),
}

impl AssetDisposal {
    pub(crate) fn date(&self) -> Option<u32> {
        match self {
            Self::No => None,
            Self::Disposed { date, .. } => Some(*date),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::No => true,
            Self::Disposed { date, how } => {
                (20260101..=20261231).contains(date)
                    && crate::dates::source_valid_date(*date)
                    && how.valid()
            }
        }
    }
}

// Shared financial facts; each source owner validates its own event date.
impl DisposalKind {
    pub(crate) fn valid(&self) -> bool {
        match self {
            DisposalKind::Sold { proceeds, selling_expenses, payment, buyer, .. } => {
                [proceeds, selling_expenses].iter().all(|v| (0..=100000000000).contains(*v))
                    && payment.valid(*proceeds)
                    && (*buyer == Buyer::Unrelated || !payment.installment())
            }
            DisposalKind::Condemned { award, expenses, replacement_cost } => {
                [award, expenses, replacement_cost].iter().all(|v| (0..=100000000000).contains(*v))
            }
            DisposalKind::Scrapped | DisposalKind::PersonalUse => true,
        }
    }
}

impl RentalSale {
    /// The like-kind exchange of this rental, when it was exchanged: the one
    /// test the rental rules make of how the property left (the property
    /// received continues the rental), so an exchange is read here and
    /// nowhere by matching `sale` again.
    pub(crate) fn exchange(&self) -> Option<&crate::RentalExchange> {
        match self {
            Self::Exchanged(exchange) => Some(exchange),
            Self::No | Self::Sold { .. } | Self::Condemned { .. } => None,
        }
    }
    pub(crate) fn date(&self) -> Option<u32> {
        match self {
            Self::No => None,
            Self::Sold { date, .. } | Self::Condemned { date, .. } => Some(*date),
            Self::Exchanged(v) => Some(v.transferred),
        }
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::No => true,
            Self::Exchanged(v) => v.valid(),
            Self::Sold {
                date,
                price,
                land_price,
                selling_expenses,
                payment,
                buyer,
                personal_share_percent,
            } => {
                (20260101..=20261231).contains(date)
                    && crate::dates::source_valid_date(*date)
                    && (0..=100000000000).contains(price)
                    && (0..=*price).contains(land_price)
                    && (0..=100000000000).contains(selling_expenses)
                    && payment.valid(*price)
                    && (*buyer == Buyer::Unrelated || !payment.installment())
                    && *personal_share_percent <= 99
            }
            Self::Condemned { date, award, land_award, expenses, replacement_cost } => {
                (20260101..=20261231).contains(date)
                    && crate::dates::source_valid_date(*date)
                    && (0..=100000000000).contains(award)
                    && (0..=*award).contains(land_award)
                    && (0..=100000000000).contains(expenses)
                    && (0..=100000000000).contains(replacement_cost)
            }
        }
    }
}

// Half-months of the year of disposition allowed by the convention, out of
// 24 (Pub 946 ch. 4): half-year 12; mid-quarter 3, 9, 15 or 21 by the
// quarter of disposition; mid-month 2 x month - 1.
pub(crate) fn disposal_half_months(date: u32, convention: Convention) -> i64 {
    let month = i64::from(date / 100 % 100).clamp(1, 12);
    match convention {
        Convention::HalfYear => 12,
        Convention::MidQuarter => 3 + 6 * ((month - 1) / 3),
        Convention::MidMonth => 2 * month - 1,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Convention {
    HalfYear,
    MidQuarter,
    MidMonth,
}

// Allocate source amounts; retained character, identity and confirmations stay facts.
crate::community::divide_record!(PartialDisposition {
    amounts: [cost_of_part, proceeds],
    gains: [],
    facts: [date],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(RentalSale {
    records: [
        Sold {
            amounts: [price, land_price, selling_expenses, payment],
            facts: [date, buyer, personal_share_percent]
        },
        Condemned { amounts: [award, land_award, expenses, replacement_cost], facts: [date] },
    ],
    units: [No],
    tuples: [Exchanged],
});

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_enum!(AssetDisposal {
    records: [Disposed { amounts: [how], facts: [date] }],
    units: [No],
    tuples: [],
});

crate::community::divide_enum!(DisposalKind {
    records: [
        Sold { amounts: [proceeds, selling_expenses, payment, buyer, livestock], facts: [] },
        Condemned { amounts: [award, expenses, replacement_cost], facts: [] }
    ],
    units: [Scrapped, PersonalUse],
    tuples: [],
});

crate::community::divide_enum!(SoldLivestock {
    records: [
        CattleOrHorse { amounts: [], facts: [purpose] },
        Other { amounts: [], facts: [purpose] }
    ],
    units: [No],
    tuples: [],
});

crate::community::divide_enum!(Livestock {
    records: [],
    units: [No, CattleOrHorse, Other],
    tuples: [],
});

crate::community::divide_enum!(Buyer {
    records: [],
    units: [Unrelated, Relative, ControlledEntity],
    tuples: [],
});
