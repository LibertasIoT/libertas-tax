// Coverage: Form 6252 — Supported (2026-09-27; audited Missing 2026-09-24).
//   Lines: 1-18 and 19-26 for a 2026 installment sale entered on an existing sale fold (a
//   continuing business asset or actual-expense vehicle held more than one year, a whole rental
//   property, a farm-land parcel held more than one year): line 5 the fold's sale price, line 6
//   the loans the buyer took over, lines 8-10 the fold's basis and depreciation, line 11 its
//   selling expenses, line 12 the Form 4797 Part III line 33 section 1245 recapture (taxed in full
//   in 2026, section 453(i)), line 14 = gross profit (line 15 is always zero: no main home is sold
//   on these folds), line 17 debt above line 13, line 18 contract price, line 19 = 16/18 exact,
//   line 20 = line 17, line 21 principal received in 2026, line 24 = line 22 x line 19 in whole
//   dollars, line 25 the farm land's section 1252 recapture limited by line 24 (Form 4797 line
//   16), line 26 to Form 4797 line 4 (section 1231). The recognized amounts replace the fold's gain
//   at its source (asset_sale::sale_for / rental_sale, farm_land::calculate_for), so section 1231
//   netting and the lookback, Schedule D, QBI, NIIT, Form 4952, rental MAGI, section 469(g) and
//   AMT (its own basis and depreciation, hence its own gross profit) all read the 2026 recognized
//   gain. Unrecaptured section 1250 gain is recognized first (Schedule D instructions, Worksheet
//   line 4). A sale at no gain (line 14 zero or less) is not an installment sale: the fold's loss
//   is reported in full. Payments in 2026 on a sale from an earlier year (Part II lines 19-26,
//   line 20 zero, line 19 = the year-of-sale line 16 / line 18): the Investment sales kind
//   "Installment payments from an earlier-year sale" (FederalScreening.installment_payments) by
//   character: capital held more than one year to Schedule D line 11, one year or less to line
//   4 (both net investment income); business or rental property held more than one year to Form
//   4797 line 4 with its remaining unrecaptured section 1250 gain, as its own section 1231
//   source (earlier_sources) in the netting, lookback, rate classes, NIIT (excluded for a business
//   you materially participated in, included for a rental) and rental MAGI (business only).
//   Unstated interest (Pub 537; sections 483 and 1274): a note with at least the applicable
//   federal rate needs no adjustment; a lower rate is refused unless the sale price is $3,000
//   or less (section 483(d)(2)). Section 453A: 2026 non-farm sales above $150,000 whose notes
//   outstanding at the end of 2026 total more than $5,000,000 are refused (interest on deferred
//   tax). Refused by name: related-party sales (lines 3, 27-37), dealer sales, contingent-price
//   sales (line 4), pledged notes, notes disposed of in 2026 (section 453B) through "Something
//   else applies"; property also used personally, held one year or less, a rental with a loss on
//   the building or the land (a separate Form 6252 for each), a rental with suspended passive
//   losses or a current-year loss (section 469(g)(3)), a farm-land installment sale with a Form
//   1062 election; earlier-year business property held one year or less is not offered; an
//   earlier-year rental gain on a return with unallowed passive losses (Form 8582); an
//   earlier-year business gain when the section 1231 lookback recaptures part of this year's
//   gain (Form 8995 line 1). Current-year investment/personal sales use InvestmentInstallmentSale,
//   with raw source facts and native identity/owner/community retained in FederalIncomeEntry;
//   Form 6252 gain joins Schedule D line 4/11, collectible classes and taxable-gain NIIT/investment-only Form4952.
//   ElectOut or a nonpositive gain forwards the actual sale to Form 8949; personal/related losses
//   are nondeductible. Related-buyer acceleration and eligible pledges use the shared engine;
//   Part III income joins the same capital source once. Earlier-year business gains do not enter the section 179
//   business-income limit. Tests (hand-computed): src/tests/investment/business.rs
//   form_6252_equipment_installment_taxes_recapture_now_and_gain_by_payments,
//   form_6252_building_installment_recognizes_unrecaptured_1250_first,
//   form_6252_loans_above_basis_are_a_payment_in_the_year_of_sale,
//   form_6252_loss_sale_is_reported_in_full_and_unstated_interest_starts_above_3000,
//   form_6252_notes_above_5_million_owe_section_453a_interest,
//   form_6252_situations_outside_the_calculation_are_refused_by_name; src/tests/farm/farm.rs
//   form_6252_farm_land_installment_takes_section_1252_recapture_first; src/tests/investment/investment.rs
//   form_6252_rental_installment_sale_matches_hand_computation,
//   form_6252_rental_installment_refusals_are_named,
//   form_6252_earlier_year_payments_route_by_character,
//   form_6252_earlier_business_gain_is_not_investment_income_but_a_rental_gain_is,
//   form_6252_earlier_year_boundaries_and_refusals; src/tests/investment/interview.rs installment_* (pages).
//   https://www.irs.gov/pub/irs-dft/f6252--dft.pdf (2026 draft form with its instructions)
//   https://www.irs.gov/publications/p537
//   https://www.irs.gov/instructions/i1040sd (Unrecaptured Section 1250 Gain Worksheet line 4)
use crate::Owner;
pub use crate::installment_payment::{NoteInterest, SalePayment};
use crate::kernel::installment::{
    Acceleration, PriorSaleCents, Schedule, YearOfSaleDollars, payment_year, pledge_payment,
    related_disposition, year_of_sale,
};
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Installment payments from an earlier-year sale
/// One for each sale before 2026 you report on the installment method. Use the Form 6252 you filed for the year of the sale.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InstallmentPayments {
    /// Property sold
    #[libertas_size(min = 1)]
    pub property: String,
    /// Date sold
    #[libertas_date_only]
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 19870101, max = 20251231)]
    pub sold: u32,
    /// Gross profit
    /// Line 16 of the Form 6252 for the year of the sale.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub gross_profit: i64,
    /// Contract price
    /// Line 18 of the Form 6252 for the year of the sale.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub contract_price: i64,
    /// Payments received before 2026
    /// Lines 22 and 23 of your 2025 Form 6252 added together, without interest.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub earlier_payments: i64,
    /// Principal received in 2026
    /// Without interest; report the interest as interest income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub payments: i64,
    /// What you sold
    pub kind: PropertyCharacter,
    /// Something else applies
    /// A sale to a relative or to a company, partnership or trust you or they control; a dealer sale; a price that depends on later events; a note you pledged, sold, canceled or gave away; depreciation recapture still to report; collectibles or small business stock; a note charging less than the applicable federal rate; a gain that differs for the alternative minimum tax; or suspended passive losses from the property.
    #[libertas_default(false)]
    pub special: bool,
    /// Qualified business income of a business sale
    #[libertas_default(NotQualified)]
    pub qbi: InstallmentQbi,
    /// Original livestock classification
    /// Use the sale-year property records. Null means those records do not establish the classification.
    /// [NullText]
    /// The original livestock classification is unavailable
    /// [SetText]
    /// The original property classification is
    pub livestock: Option<crate::Livestock>,
}

/// Kind of property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PropertyCharacter {
    /// Investment property held more than one year
    /// Land, a private business interest or other property you held as an investment.
    LongTermCapital,
    /// Investment property held one year or less
    ShortTermCapital,
    /// Business or rental property held more than one year
    BusinessProperty {
        /// How you used it
        used: PropertyUse,
        /// Unrecaptured section 1250 gain not yet reported
        /// For a building or other depreciated real property: the smaller of its depreciation and its gain, less the gain you reported from this sale before 2026. Zero for land or equipment.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        unrecaptured: i64,
    },
}

/// How you used it
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum PropertyUse {
    /// In a business you materially participated in
    Business,
    /// As a rental or in another business you did not materially participate in
    Rental,
}

/// Qualified business income of the earlier sale
/// The stored answer; the page asks it with `SoldQbi`, which holds the business picker.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InstallmentQbi {
    /// Not from a qualified business
    NotQualified,
    /// From a qualified business; None is a business no longer on this return
    Qualified { business: Option<i64> },
}

/// Installment payments from an earlier-year sale
/// One for each sale before 2026 you report on the installment method. Use the Form 6252 you filed for the year of the sale.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InstallmentSale {
    /// Property sold
    #[libertas_size(min = 1)]
    pub property: String,
    /// Date sold
    #[libertas_date_only]
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 19870101, max = 20251231)]
    pub sold: u32,
    /// Gross profit
    /// Line 16 of the Form 6252 for the year of the sale.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub gross_profit: i64,
    /// Contract price
    /// Line 18 of the Form 6252 for the year of the sale.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub contract_price: i64,
    /// Payments received before 2026
    /// Lines 22 and 23 of your 2025 Form 6252 added together, without interest.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub earlier_payments: i64,
    /// Principal received in 2026
    /// Without interest; report the interest as interest income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub payments: i64,
    /// What you sold
    pub kind: SoldProperty,
    /// Something else applies
    /// A sale to a relative or to a company, partnership or trust you or they control; a dealer sale; a price that depends on later events; a note you pledged, sold, canceled or gave away; depreciation recapture still to report; collectibles or small business stock; a note charging less than the applicable federal rate; a gain that differs for the alternative minimum tax; or suspended passive losses from the property.
    #[libertas_default(false)]
    pub special: bool,
}

/// Kind of property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SoldProperty {
    /// Investment property held more than one year
    /// Land, a private business interest or other property you held as an investment.
    LongTermCapital,
    /// Investment property held one year or less
    ShortTermCapital,
    /// Business or rental property held more than one year
    BusinessProperty {
        /// How you used it
        used: SoldUse,
        /// Livestock classification from the original sale
        /// Use the sale-year property records and Form 4797. Draft, breeding, dairy and sporting livestock qualify for the farmer estimated-tax test. This does not change the gain or its tax character.
        /// [NullText]
        /// The original livestock classification is unavailable
        /// [SetText]
        /// The original property classification is
        livestock: Option<crate::Livestock>,
        /// Unrecaptured section 1250 gain not yet reported
        /// For a building or other depreciated real property: the smaller of its depreciation and its gain, less the gain you reported from this sale before 2026. Zero for land or equipment.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        unrecaptured: i64,
    },
}

/// How you used it
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SoldUse {
    /// In a business you materially participated in
    Business {
        /// Qualified business income
        /// Business-property losses from 2021–2025 can make part of this year's gain ordinary income; that part is qualified business income of the business that sold the property.
        #[libertas_default(NotQualified)]
        qbi: SoldQbi,
    },
    /// As a rental or in another business you did not materially participate in
    Rental,
}

/// Qualified business income of the earlier sale
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SoldQbi {
    /// Not from a qualified business
    /// The business was not a trade or business for the QBI deduction, or you do not claim it.
    NotQualified,
    /// From a qualified business
    Qualified {
        /// The business
        /// Choose the business on this return that sold the property; its W-2 wages and property count toward the limit. Leave empty when that business is no longer on this return: the ordinary part is then a business of its own with no wages or property for the limit.
        /// [NullText]
        /// The selling business is no longer on this return
        /// [SetText]
        /// The selling business on this return is
        #[libertas_enum_source("$.qbi_businesses")]
        business: Option<i64>,
    },
}

impl InstallmentSale {
    // The stored record of the page's answers.
    pub(crate) fn record(&self) -> InstallmentPayments {
        let (kind, qbi) = match &self.kind {
            SoldProperty::LongTermCapital => (PropertyCharacter::LongTermCapital, None),
            SoldProperty::ShortTermCapital => (PropertyCharacter::ShortTermCapital, None),
            SoldProperty::BusinessProperty { used, unrecaptured, .. } => match used {
                SoldUse::Business { qbi } => (
                    PropertyCharacter::BusinessProperty {
                        used: PropertyUse::Business,
                        unrecaptured: *unrecaptured,
                    },
                    Some(match qbi {
                        SoldQbi::NotQualified => InstallmentQbi::NotQualified,
                        SoldQbi::Qualified { business } => {
                            InstallmentQbi::Qualified { business: *business }
                        }
                    }),
                ),
                SoldUse::Rental => (
                    PropertyCharacter::BusinessProperty {
                        used: PropertyUse::Rental,
                        unrecaptured: *unrecaptured,
                    },
                    None,
                ),
            },
        };
        InstallmentPayments {
            property: self.property.clone(),
            sold: self.sold,
            gross_profit: self.gross_profit,
            contract_price: self.contract_price,
            earlier_payments: self.earlier_payments,
            payments: self.payments,
            kind,
            special: self.special,
            qbi: qbi.unwrap_or(InstallmentQbi::NotQualified),
            livestock: match self.kind {
                SoldProperty::BusinessProperty { livestock, .. } => livestock,
                _ => None,
            },
        }
    }
}

impl InstallmentPayments {
    // The page showing a stored record.
    pub(crate) fn page(&self) -> InstallmentSale {
        InstallmentSale {
            property: self.property.clone(),
            sold: self.sold,
            gross_profit: self.gross_profit,
            contract_price: self.contract_price,
            earlier_payments: self.earlier_payments,
            payments: self.payments,
            kind: match &self.kind {
                PropertyCharacter::LongTermCapital => SoldProperty::LongTermCapital,
                PropertyCharacter::ShortTermCapital => SoldProperty::ShortTermCapital,
                PropertyCharacter::BusinessProperty { used, unrecaptured } => {
                    SoldProperty::BusinessProperty {
                        livestock: self.livestock,
                        used: match used {
                            PropertyUse::Business => SoldUse::Business {
                                qbi: match &self.qbi {
                                    InstallmentQbi::NotQualified => SoldQbi::NotQualified,
                                    InstallmentQbi::Qualified { business } => {
                                        SoldQbi::Qualified { business: *business }
                                    }
                                },
                            },
                            PropertyUse::Rental => SoldUse::Rental,
                        },
                        unrecaptured: *unrecaptured,
                    }
                }
            },
            special: self.special,
        }
    }
}

/// Installment sale in 2026
/// Investment or personal-use property with a payment due after 2026. Enter exchange-traded securities, fully paid sales and losses as 1099-B sales. Enter a business, rental or farm asset's sale on that asset. Report actual interest separately as interest income.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentInstallmentSale {
    /// Property sold
    #[libertas_size(min = 1)]
    pub property: String,
    /// Date acquired
    #[libertas_date_only]
    pub acquired: u32,
    /// Date sold
    #[libertas_date_only]
    #[libertas_number(min = 20260101, max = 20261231)]
    pub sold: u32,
    /// What it was
    pub use_kind: InstallmentUse,
    /// Collectible capital asset
    /// Investment or personal-use art, antiques, metals, gems, stamps, coins and similar capital assets. Long-term gains have a maximum 28% rate.
    pub collectible: bool,
    /// Selling price (Form 6252 line 5)
    /// Cash, the buyer's note and loans the buyer took over, without interest. For a contingent price with a stated maximum, enter that maximum.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub price: i64,
    /// Loans the buyer took over (line 6)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub assumed_debt: i64,
    /// Adjusted basis (line 8)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Selling expenses (line 11)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub selling_expenses: i64,
    /// Principal received in 2026 (line 21)
    /// Without interest. Include the down payment and sale money paid at closing to your lender or for selling costs.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub payments: i64,
    /// Interest the buyer's note charges
    pub interest: NoteInterest,
    /// Buyer
    pub buyer: crate::Buyer,
    /// Did a related buyer resell or give away the property in 2026?
    pub second_disposition: SecondDisposition,
    /// Net proceeds of a loan secured by the buyer's note in 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub pledged: i64,
    /// How to report it
    pub method: InstallmentMethod,
    /// Something else applies
    pub other: InstallmentOther,
}

/// What it was
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum InstallmentUse {
    /// Investment property
    /// Land, a private business interest, a note or other property held for investment.
    Investment,
    /// Personal-use property
    /// A gain is taxable; a personal loss is not deductible.
    Personal,
}

/// How to report it
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum InstallmentMethod {
    /// Installment method: tax the gain as payments arrive
    Installment,
    /// Elect out: report the whole gain in 2026
    /// Report the sale this way on a timely filed return, including extensions. Revocation requires IRS consent.
    ElectOut,
}

/// Did a related buyer resell or give away the property in 2026?
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SecondDisposition {
    /// No
    None,
    /// Yes
    Resold {
        /// Date of the buyer's sale or gift
        #[libertas_date_only]
        on: u32,
        /// Amount the buyer realized, or fair market value for a gift
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount_realized: i64,
    },
    /// Yes, but a Form 6252 line 29 exception applies
    Excepted {
        /// Exception
        condition: ResaleException,
    },
}

/// Exception
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ResaleException {
    /// The first sale was stock sold to its issuing corporation (29b)
    IssuingCorporation,
    /// Involuntary conversion threatened after your sale (29c)
    InvoluntaryConversion,
    /// The buyer disposed of it after either party's death (29d)
    AfterDeath,
    /// You can establish to the IRS that neither sale principally avoided tax (29e)
    NoTaxAvoidance,
}

/// Something else applies
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum InstallmentOther {
    /// None of these
    NoneOfThese,
    /// Qualified small business stock (section 1202)
    QualifiedSmallBusinessStock,
    /// A contingent price with no stated maximum
    ContingentPriceNoMaximum,
    /// A dealer's sale of property held for customers
    DealerSale,
}

impl InvestmentInstallmentSale {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue as Issue;
        if self.property.trim().is_empty() {
            return Some(Issue::new("INSTALLMENT_PROPERTY_REQUIRED").at_field("property"));
        }
        if !crate::federal_rules::valid_date(self.acquired) || self.acquired > self.sold {
            return Some(Issue::new("INSTALLMENT_SOLD_BEFORE_ACQUIRED").at_field("acquired"));
        }
        if !crate::federal_rules::valid_date(self.sold)
            || !(20260101..=20261231).contains(&self.sold)
        {
            return Some(Issue::new("INSTALLMENT_SALE_DATE_INVALID").at_field("sold"));
        }
        for (field, amount) in [
            ("assumed_debt", self.assumed_debt),
            ("basis", self.basis),
            ("selling_expenses", self.selling_expenses),
            ("payments", self.payments),
            ("pledged", self.pledged),
        ] {
            if amount < 0 {
                return Some(Issue::new("INSTALLMENT_AMOUNT_NEGATIVE").at_field(field));
            }
        }
        if self.basis.checked_add(self.selling_expenses).is_none() {
            return Some(Issue::new("INSTALLMENT_AMOUNT_OVERFLOW").at_field("selling_expenses"));
        }
        if self.price <= 0 {
            return Some(Issue::new("INSTALLMENT_PRICE_REQUIRED").at_field("price"));
        }
        if i128::from(self.assumed_debt) + i128::from(self.payments) > i128::from(self.price) {
            return Some(Issue::new("INSTALLMENT_PAID_ABOVE_PRICE").at_field("payments"));
        }
        if let SecondDisposition::Resold { on, amount_realized } = self.second_disposition {
            if !crate::federal_rules::valid_date(on) || on < self.sold || on > 20261231 {
                return Some(
                    Issue::new("INSTALLMENT_RESALE_BEFORE_SALE")
                        .at_field("on")
                        .in_field("second_disposition"),
                );
            }
            if amount_realized < 0 {
                return Some(
                    Issue::new("INSTALLMENT_AMOUNT_NEGATIVE")
                        .at_field("amount_realized")
                        .in_field("second_disposition"),
                );
            }
        }
        if self.calculation().is_err() {
            return Some(Issue::new("INSTALLMENT_AMOUNT_OVERFLOW").at_field("price"));
        }
        None
    }

    pub(crate) fn refusal(&self) -> Option<(&'static str, &'static str)> {
        match self.other {
            InstallmentOther::QualifiedSmallBusinessStock => {
                return Some((
                    "ISSUE_INSTALLMENT_QSBS",
                    "Installment sales of qualified small business stock require the exclusion history; this sale is not calculated.",
                ));
            }
            InstallmentOther::ContingentPriceNoMaximum => {
                return Some((
                    "ISSUE_INSTALLMENT_NO_MAXIMUM_PRICE",
                    "A contingent installment price without a stated maximum requires a separate basis recovery schedule; this sale is not calculated.",
                ));
            }
            InstallmentOther::DealerSale => {
                return Some((
                    "ISSUE_INSTALLMENT_DEALER_SALE",
                    "Dealer sales of property held for customers cannot use this capital installment-sale worksheet.",
                ));
            }
            InstallmentOther::NoneOfThese => {}
        }
        self.interest.refusal(self.price)
    }

    // Source eligibility stays here; the shared engine owns every payment and gain formula.
    pub(crate) fn calculation(&self) -> Result<Option<CurrentSchedule>, ()> {
        if self.method == InstallmentMethod::ElectOut
            || i128::from(self.assumed_debt) + i128::from(self.payments) == i128::from(self.price)
        {
            return Ok(None);
        }
        let pledged = if self.price > 15_000_000 && self.use_kind == InstallmentUse::Investment {
            let unpaid = self
                .price
                .checked_sub(self.assumed_debt)
                .and_then(|value| value.checked_sub(self.payments))
                .ok_or(())?;
            pledge_payment(unpaid, self.pledged).ok_or(())?
        } else {
            0
        };
        let payments = self.payments.checked_add(pledged).ok_or(())?;
        let Some(form) = year_of_sale(YearOfSaleDollars {
            price: dollars(self.price),
            debt: dollars(self.assumed_debt),
            cost: dollars(self.basis),
            depreciation: 0,
            expenses: dollars(self.selling_expenses),
            recapture: 0,
            payments: dollars(payments),
            other_recapture: 0,
        }) else {
            return Ok(None);
        };
        // Form 6252's year-of-sale amounts have already crossed their dollar boundary.
        // Project that same line22 once, including excess debt and eligible pledge proceeds.
        let acceleration = if self.buyer != crate::Buyer::Unrelated
            && let SecondDisposition::Resold { amount_realized, .. } = self.second_disposition
        {
            // Line 32 compares reported lines 30 and 31. Give the engine the
            // same reported line 30, just as contract and paid principal above.
            let amount_realized = dollars(amount_realized).checked_mul(100).ok_or(())?;
            Some(
                related_disposition(
                    PriorSaleCents {
                        gross_profit: form.gross_profit.checked_mul(100).ok_or(())?,
                        contract_price: form.contract.checked_mul(100).ok_or(())?,
                        earlier_payments: 0,
                        payments: form
                            .excess_debt
                            .checked_add(form.payments)
                            .and_then(|value| value.checked_mul(100))
                            .ok_or(())?,
                    },
                    amount_realized,
                )
                .ok_or(())?,
            )
        } else {
            None
        };
        let current = CurrentSchedule { form, pledged, acceleration };
        // Every reported dollar amount must remain representable as Money cents.
        for amount in [
            form.price,
            form.debt,
            form.cost,
            form.expenses,
            form.cost + form.expenses,
            form.gross_profit,
            form.contract,
            form.excess_debt + form.payments,
            current.gain(),
        ] {
            amount.checked_mul(100).ok_or(())?;
        }
        Ok(Some(current))
    }
}

// Transient calculation output; raw answers alone are persisted.
#[derive(Clone, Copy)]
pub(crate) struct CurrentSchedule {
    pub form: Schedule,
    pub pledged: i64,
    pub acceleration: Option<Acceleration>,
}

impl CurrentSchedule {
    pub(crate) fn gain(self) -> i64 {
        self.form.gain + self.acceleration.map_or(0, |value| value.income_dollars)
    }
}

/// Installment sale from an earlier year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EarlierInstallment {
    /// Seller
    pub owner: Owner,
    /// Sale
    pub sale: InstallmentPayments,
}

impl SalePayment {
    // Lines 5-26 from the fold's whole-dollar amounts; None when paid in
    // full or when the sale has no gain.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn form(
        &self,
        price: i64,
        cost: i64,
        depreciation: i64,
        expenses: i64,
        recapture: i64,
        other_recapture: i64,
    ) -> Option<Schedule> {
        let Self::Installment { assumed_debt, payments, .. } = self else {
            return None;
        };
        year_of_sale(YearOfSaleDollars {
            price,
            debt: dollars(*assumed_debt),
            cost,
            depreciation,
            expenses,
            recapture,
            payments: dollars(*payments),
            other_recapture,
        })
    }
}

impl InstallmentPayments {
    pub(crate) fn livestock_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let invalid = match self.kind {
            PropertyCharacter::BusinessProperty { unrecaptured, .. } => {
                self.livestock.is_some_and(|value| value != crate::Livestock::No)
                    && unrecaptured != 0
            }
            _ => self.livestock.is_some(),
        };
        invalid.then(|| {
            crate::validation::ValidationIssue::new("INSTALLMENT_LIVESTOCK_CHARACTER")
                .at_field("kind")
        })
    }
    pub(crate) fn valid(&self) -> bool {
        !self.property.is_empty()
            && self.livestock_issue().is_none()
            && (19870101..=20251231).contains(&self.sold)
            && crate::federal_rules::valid_date(self.sold)
            && (1..=100000000000).contains(&self.gross_profit)
            && (1..=100000000000).contains(&self.contract_price)
            && (0..=100000000000).contains(&self.earlier_payments)
            && (0..=100000000000).contains(&self.payments)
            && self.gross_profit <= self.contract_price
            && self.earlier_payments + self.payments <= self.contract_price
            && match self.kind {
                PropertyCharacter::BusinessProperty { unrecaptured, .. } => {
                    (0..=100000000000).contains(&unrecaptured)
                }
                _ => true,
            }
    }
    pub(crate) fn supported(&self) -> bool {
        self.valid() && !self.special
    }
    // Line 26 and the unrecaptured section 1250 part of it, whole dollars.
    pub(crate) fn recognized(&self) -> (i64, i64) {
        let form = payment_year(PriorSaleCents {
            gross_profit: self.gross_profit,
            contract_price: self.contract_price,
            earlier_payments: self.earlier_payments,
            payments: self.payments,
        });
        let unrecaptured = match self.kind {
            PropertyCharacter::BusinessProperty { unrecaptured, .. } => {
                phase::cap(dollars(unrecaptured), [form.gain])
            }
            _ => 0,
        };
        (form.gain, unrecaptured)
    }
}
