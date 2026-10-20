// Coverage: Form 6252 — Supported (T3-6252, 2026-09-27; audited Missing 2026-09-24).
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
//   gain (Form 8995 line 1); a 2026 installment sale of investment property keeps
//   RareSituation::InstallmentSale. Earlier-year business gains do not enter the section 179
//   business-income limit. STORAGE_FORMAT 71. Tests (hand-computed): src/tests/investment/business.rs
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
use crate::kernel::installment::{
    PriorSaleCents, Schedule, YearOfSaleDollars, payment_year, year_of_sale,
};
use crate::kernel::lines::push_line;
use crate::kernel::phase;
use crate::kernel::ratio::dollars;
use crate::{FederalDraft, FederalResult, FormLine, Owner, TaxForm};
use alloc::{format, string::String, vec::Vec};
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
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        assumed_debt: i64,
        /// Principal received in 2026
        /// The down payment and the principal of every 2026 payment on the buyer's note, without interest. Include sale money paid at closing to your lender or for your selling costs.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 1)]
    pub gross_profit: i64,
    /// Contract price
    /// Line 18 of the Form 6252 for the year of the sale.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub contract_price: i64,
    /// Payments received before 2026
    /// Lines 22 and 23 of your 2025 Form 6252 added together, without interest.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub earlier_payments: i64,
    /// Principal received in 2026
    /// Without interest; report the interest as interest income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 1)]
    pub gross_profit: i64,
    /// Contract price
    /// Line 18 of the Form 6252 for the year of the sale.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub contract_price: i64,
    /// Payments received before 2026
    /// Lines 22 and 23 of your 2025 Form 6252 added together, without interest.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub earlier_payments: i64,
    /// Principal received in 2026
    /// Without interest; report the interest as interest income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        /// Unrecaptured section 1250 gain not yet reported
        /// For a building or other depreciated real property: the smaller of its depreciation and its gain, less the gain you reported from this sale before 2026. Zero for land or equipment.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
            SoldProperty::BusinessProperty { used, unrecaptured } => match used {
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

/// Installment sale from an earlier year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EarlierInstallment {
    /// Seller
    pub owner: Owner,
    /// Sale
    pub sale: InstallmentPayments,
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
        if *interest == NoteInterest::Low && price > 300_000 {
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

// Section 453A(b)(2): nondealer notes from 2026 sales above $150,000
// (farm property excepted) outstanding at the end of the year.
pub(crate) fn section_453a(draft: &FederalDraft) -> i64 {
    let mut total = 0;
    let mid_quarter = crate::depreciation::mid_quarter(draft);
    for entry in &crate::community::property_entries(draft) {
        match &entry.income {
            crate::FederalIncome::Business { .. } | crate::FederalIncome::FarmRental { .. } => {
                let assets = match &entry.income {
                    crate::FederalIncome::Business { data } => &data.assets,
                    crate::FederalIncome::FarmRental { data } => &data.assets,
                    _ => continue,
                };
                for asset in assets {
                    if let Some(v) = asset.sale_for(mid_quarter, false)
                        && let Some(form) = v.installment
                        && form.price > 150_000
                    {
                        total += form.outstanding();
                    }
                }
            }
            crate::FederalIncome::Rental { data } => {
                if let Ok(Some(v)) = crate::asset_sale::rental_sale(data, false)
                    && let Some(form) = v.installment
                    && form.price > 150_000
                {
                    total += form.outstanding();
                }
            }
            _ => {}
        }
    }
    total
}

const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/f6252--dft.pdf";

fn row(
    r: &mut FederalResult,
    line: &str,
    id: &str,
    amount: i64,
    input: &str,
    explanation: crate::Message,
) {
    push_line(
        r,
        Some(FormLine::new(TaxForm::Form6252, line)),
        id,
        amount * 100,
        &[input],
        SOURCE,
        explanation,
    );
}

// One year-of-sale Form 6252's rows, and line 26's destination.
pub(crate) fn report_sale(
    r: &mut FederalResult,
    prefix: &str,
    input: &str,
    form: &Schedule,
    unrecaptured: i64,
) {
    let line13 = form.cost - form.depreciation + form.expenses + form.recapture;
    for (line, suffix, amount, explanation) in [
        ("5", "price", form.price, msg!("EXPLAIN_INSTALLMENT_N")),
        ("6", "assumed_debt", form.debt, msg!("EXPLAIN_INSTALLMENT_N_2")),
        ("7", "net_price", form.price - form.debt, msg!("EXPLAIN_INSTALLMENT_N_3")),
        ("8", "cost", form.cost, msg!("EXPLAIN_INSTALLMENT_N_4")),
        ("9", "depreciation", form.depreciation, msg!("EXPLAIN_INSTALLMENT_N_5")),
        ("10", "adjusted_basis", form.cost - form.depreciation, msg!("EXPLAIN_INSTALLMENT_N_6")),
        ("11", "expenses", form.expenses, msg!("EXPLAIN_INSTALLMENT_N_7")),
        ("12", "recapture", form.recapture, msg!("EXPLAIN_INSTALLMENT_N_8")),
        ("13", "total", line13, msg!("EXPLAIN_INSTALLMENT_N_9")),
        ("16", "gross_profit", form.gross_profit, msg!("EXPLAIN_INSTALLMENT_N_10")),
        ("17", "excess_debt", form.excess_debt, msg!("EXPLAIN_INSTALLMENT_N_11")),
        ("18", "contract_price", form.contract, msg!("EXPLAIN_INSTALLMENT_N_12")),
        ("20", "deemed_payment", form.excess_debt, msg!("EXPLAIN_INSTALLMENT_N_13")),
        ("21", "payments", form.payments, msg!("EXPLAIN_INSTALLMENT_N_14")),
        (
            "22",
            "total_payments",
            form.excess_debt + form.payments,
            msg!("EXPLAIN_INSTALLMENT_N_15"),
        ),
    ] {
        row(r, line, &format!("{prefix}_{suffix}"), amount, input, explanation);
    }
    report_income(r, prefix, input, form, unrecaptured, true);
}

fn report_income(
    r: &mut FederalResult,
    prefix: &str,
    input: &str,
    form: &Schedule,
    unrecaptured: i64,
    section1231: bool,
) {
    row(r, "24", &format!("{prefix}_income"), form.income, input, msg!("EXPLAIN_N_INCOME"));
    row(r, "25", &format!("{prefix}_ordinary"), form.ordinary, input, msg!("EXPLAIN_N_ORDINARY"));
    row(
        r,
        "26",
        &format!("{prefix}_gain"),
        form.gain,
        input,
        if section1231 { msg!("EXPLAIN_N_GAIN") } else { msg!("EXPLAIN_N_GAIN_2") },
    );
    if section1231 && unrecaptured > 0 {
        push_line(
            r,
            None,
            &format!("{prefix}_unrecaptured_1250"),
            unrecaptured * 100,
            &[input],
            "https://www.irs.gov/instructions/i1040sd",
            msg!("EXPLAIN_N_UNRECAPTURED_N_2"),
        );
    }
}

impl InstallmentPayments {
    pub(crate) fn valid(&self) -> bool {
        !self.property.is_empty()
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

fn earlier(draft: &FederalDraft) -> impl Iterator<Item = (usize, &InstallmentPayments)> {
    draft
        .screening
        .iter()
        .flat_map(|s| s.installment_payments.iter())
        .map(|v| &v.sale)
        .enumerate()
        .filter(|(_, v)| v.supported())
}

// Earlier-year sales' Schedule D gains in cents, [short-term, long-term].
// Same recognized amounts with their retained filer, before aggregation.
pub(crate) fn earlier_capital_sources(draft: &FederalDraft) -> ([i64; 2], [[i64; 2]; 2]) {
    let mut total = [0, 0];
    let mut owners = [[0; 2]; 2];
    for record in
        draft.screening.iter().flat_map(|s| &s.installment_payments).filter(|v| v.sale.supported())
    {
        let sale = &record.sale;
        let (gain, _) = sale.recognized();
        let term = match sale.kind {
            PropertyCharacter::ShortTermCapital => 0,
            PropertyCharacter::LongTermCapital => 1,
            PropertyCharacter::BusinessProperty { .. } => continue,
        };
        total[term] += gain * 100;
        owners[usize::from(record.owner == Owner::Spouse)][term] += gain * 100;
    }
    (total, owners)
}

// The synthetic source id of an earlier-year sale; never an income entry's.
fn source_id(index: usize) -> i64 {
    -1 - index as i64
}

// Resolve the native source identity against its retained seller. This is
// metadata only: do not run Form6252 again to establish an owner's amount.
pub(crate) fn earlier_source_owner(draft: &FederalDraft, id: i64) -> Option<Owner> {
    draft
        .screening
        .as_ref()?
        .installment_payments
        .iter()
        .enumerate()
        .find(|(index, record)| {
            source_id(*index) == id
                && record.sale.supported()
                && matches!(record.sale.kind, PropertyCharacter::BusinessProperty { .. })
        })
        .map(|(_, record)| record.owner)
}

// Earlier-year business and rental property sales as section 1231 sources
// (Form 4797 line 4), with whether each is a business you materially
// participated in (not net investment income, not passive).
pub(crate) fn earlier_sources(
    draft: &FederalDraft,
) -> Vec<(i64, crate::farm_land::LandGain, bool)> {
    earlier(draft)
        .filter_map(|(index, sale)| {
            let PropertyCharacter::BusinessProperty { used, .. } = &sale.kind else {
                return None;
            };
            let (gain, unrecaptured) = sale.recognized();
            Some((
                source_id(index),
                crate::farm_land::LandGain {
                    gain,
                    percent: 0,
                    recapture: 0,
                    ordinary: 0,
                    section1231: gain,
                    unrecaptured,
                },
                *used == PropertyUse::Business,
            ))
        })
        .collect()
}

// The part of an earlier-year installment gain a section 1231 lookback makes
// ordinary (cents-free whole dollars): qualified business income of the
// business that sold the property.
fn qbi_part(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    index: usize,
) -> Result<i64, crate::section1231::SourceError> {
    let Some(character) = character else {
        return Ok(0);
    };
    let Some((_, sale)) = earlier(draft).find(|(i, _)| *i == index) else {
        return Ok(0);
    };
    let form = payment_year(PriorSaleCents {
        gross_profit: sale.gross_profit,
        contract_price: sale.contract_price,
        earlier_payments: sale.earlier_payments,
        payments: sale.payments,
    });
    if form.gain <= 0 || character.net <= 0 || character.ordinary == 0 {
        return Ok(0);
    }
    Ok(
        match crate::section1231::activity_character(
            draft,
            pool,
            character,
            source_id(index),
            false,
        )? {
            crate::section1231::PropertyCharacter::Capital => 0,
            crate::section1231::PropertyCharacter::Ordinary => form.gain,
            crate::section1231::PropertyCharacter::Recaptured(amount) => amount,
        },
    )
}

// The qualified business income of earlier-year installment sales by the
// business named for each, in whole dollars: `Some(id)` joins that business,
// `None` is a business no longer on the return (a stand-alone activity with
// no W-2 wages and no qualified property).
pub(crate) type QbiShares = Vec<(Option<i64>, i64, Owner)>;
pub(crate) fn qbi_shares(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
) -> Result<QbiShares, crate::section1231::SourceError> {
    let mut shares = Vec::new();
    for (index, sale) in earlier(draft) {
        let (
            PropertyCharacter::BusinessProperty { used: PropertyUse::Business, .. },
            InstallmentQbi::Qualified { business },
        ) = (&sale.kind, &sale.qbi)
        else {
            continue;
        };
        let part = qbi_part(draft, pool, character, index)?;
        if part == 0 {
            continue;
        }
        // A business the return no longer holds is the same declared state.
        let held = business.filter(|id| {
            draft
                .income
                .iter()
                .any(|e| e.id == *id && matches!(&e.income, crate::FederalIncome::Business { .. }))
        });
        shares.push((
            held,
            part,
            draft.screening.as_ref().unwrap().installment_payments[index].owner,
        ));
    }
    Ok(shares)
}

// The earlier-year sales' rows and refusals, and the return-level section
// 453A limit on 2026 sales. `pool` is the regular-tax passive pool;
// `character` the regular section 1231 result.
pub(crate) fn report(
    draft: &FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    r: &mut FederalResult,
) {
    let outstanding = section_453a(draft);
    if outstanding > 5_000_000 {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Review,
            "Installment sales",
            "ISSUE_INSTALLMENT_NOTES_FROM_RETURN_YEAR_SALES_ABOVE_150_000_TOTAL_MORE_THAN_5_0_5149A91B",
            "Installment notes from 2026 sales above $150,000 total more than $5,000,000 at the end of 2026, so interest is owed on the deferred tax (section 453A, Form 6252 and Publication 537), which is not calculated.",
        );
    }
    let Some(screening) = draft.screening.as_ref() else {
        return;
    };
    for (index, value) in screening.installment_payments.iter().enumerate() {
        let sale = &value.sale;
        #[cfg(test)]
        let section = format!("Installment payments: {}", sale.property);
        if !sale.valid() {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                &section,
                "ISSUE_ENTER_A_SALE_DATE_BEFORE_RETURN_YEAR_A_GROSS_PROFIT_NO_LARGER_THAN_THE_CON_7BB8E34C",
                "Enter a sale date before 2026, a gross profit no larger than the contract price, and payments that do not exceed the contract price (Form 6252 lines 16, 18, 21 and 23).",
            );
            continue;
        }
        if sale.special {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                &section,
                "ISSUE_THIS_EARLIER_YEAR_INSTALLMENT_SALE_IS_NOT_CALCULATED_A_RELATED_PART_7D8BAB22",
                "This earlier-year installment sale is not calculated: a related-party sale, a dealer sale, a contingent price, a pledged or disposed note, remaining recapture, collectibles, small business stock, unstated interest, a different AMT gain or suspended passive losses need their own worksheets (Form 6252 lines 3, 4, 25 and Part III; Publication 537).",
            );
            continue;
        }
        let form = payment_year(PriorSaleCents {
            gross_profit: sale.gross_profit,
            contract_price: sale.contract_price,
            earlier_payments: sale.earlier_payments,
            payments: sale.payments,
        });
        let (_, unrecaptured) = sale.recognized();
        let prefix = format!("installment_{}", index + 1);
        let input = "screening.installment_payments";
        row(
            r,
            "21",
            &format!("{prefix}_payments"),
            form.payments,
            input,
            msg!("EXPLAIN_N_PAYMENTS"),
        );
        row(
            r,
            "22",
            &format!("{prefix}_total_payments"),
            form.payments,
            input,
            msg!("EXPLAIN_N_TOTAL_PAYMENTS"),
        );
        row(
            r,
            "23",
            &format!("{prefix}_earlier_payments"),
            form.earlier_payments,
            input,
            msg!("EXPLAIN_N_EARLIER_PAYMENTS"),
        );
        let business = matches!(sale.kind, PropertyCharacter::BusinessProperty { .. });
        report_income(r, &prefix, input, &form, unrecaptured, business);
        // Form 4797 line 4: an earlier sale of business or rental property
        // brings its Form 6252 line 26 gain into the section 1231 netting.
        if business {
            push_line(
                r,
                Some(FormLine::new(TaxForm::Form4797, "4")),
                &format!("{prefix}_section1231"),
                form.gain * 100,
                &[input],
                "https://www.irs.gov/instructions/i4797",
                msg!("EXPLAIN_INSTALLMENT_N_SECTION1231"),
            );
        }
        match &sale.kind {
            PropertyCharacter::BusinessProperty { used: PropertyUse::Rental, .. }
                if form.gain > 0 && pool.passive_loss > 0 =>
            {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                    &section,
                    "ISSUE_INSTALLMENT_GAIN_FROM_AN_EARLIER_SALE_OF_RENTAL_OR_PASSIVE_ACTIVITY_13F232BB",
                    "Installment gain from an earlier sale of rental or passive-activity property is passive income that allows part of this return's unallowed passive losses (Form 8582 lines 1 through 4), which is not calculated.",
                )
            }
            // The part a section 1231 lookback makes ordinary is qualified
            // business income of the business that sold the property
            // (Reg. 1.199A-3(b)(2)(ii)(A)); the figure is the activity the
            // person named, if any.
            PropertyCharacter::BusinessProperty { used: PropertyUse::Business, .. }
                if matches!(sale.qbi, InstallmentQbi::Qualified { .. })
                    && form.gain > 0
                    && character.is_some_and(|v| v.net > 0 && v.ordinary > 0) =>
            {
                match qbi_part(draft, pool, character, index) {
                    Ok(part) => row(
                        r,
                        "4",
                        &format!("{prefix}_qualified_income"),
                        part,
                        input,
                        msg!("EXPLAIN_INSTALLMENT_N_QUALIFIED_INCOME"),
                    ),
                    Err(error) => error.report_at(
                        r,
                        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
                        #[cfg(test)]
                        &section,
                    ),
                }
            }
            _ => {}
        }
    }
}

// Community-property projection; accepted source facts are retained.

crate::community::divide_enum!(SalePayment {
    records: [Installment { amounts: [assumed_debt, payments], facts: [interest, special] }],
    units: [InFull],
    tuples: [],
});
