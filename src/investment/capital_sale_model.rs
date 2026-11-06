use crate::validation::ValidationIssue;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Holding period
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum GainTerm {
    /// Held one year or less
    Short,
    /// Held more than one year
    Long,
}

/// What you sold
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SaleUse {
    /// Investment
    /// Stock, bonds, fund shares, digital assets, or a call or put you bought. For an option you bought, its cost is the basis; one that expired has proceeds of zero.
    Investment {
        /// Disallowed wash-sale loss (Form 1099-B box 1g, Form 1099-DA box 1i)
        /// Also add it to the cost of the replacement shares when you enter their sale.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        wash_sale: i64,
    },
    /// Personal-use property
    /// A gain is taxable; a personal loss cannot offset another gain or income. Use a separate entry for each item. A home, property formerly used in business and other special cases require their own worksheets.
    Personal,
    /// An option you wrote
    /// A call or put you sold to open and then bought back or let expire in 2026. Proceeds are the premium you received and basis is what you paid to buy it back (zero if it expired); the 1099-B may instead show the net in box 1d, a loss as negative, with zero basis. Its gain or loss is short-term however long it was open. An option exercised against you has no entry of its own: a call's premium is added to the stock sale's proceeds, a put's premium reduces the cost of the shares you bought.
    WrittenOption {
        /// Disallowed wash-sale loss (Form 1099-B box 1g)
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        wash_sale: i64,
    },
}

/// Capital-asset sale
/// Enter each personal item separately, even if one Form 1099-K covers several items.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalSale {
    /// Holding period
    pub term: GainTerm,
    /// Collectible capital asset
    /// Includes investment/personal art, rugs, antiques, metals, gems, stamps, coins and alcoholic beverages. Not self-created work whose sale is ordinary income. Long-term gains have a maximum 28% rate; short-term gains use ordinary rates.
    pub collectible: bool,
    /// Net sale proceeds after selling fees
    /// Negative only for an option you wrote that you bought back for more than its premium, as the 1099-B shows it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub proceeds: i64,
    /// Adjusted tax basis, including acquisition commissions
    /// Do not include selling fees already subtracted from proceeds. For ordinary nonstatutory-option shares or vested stock awards, use the established adjusted basis from complete compensation and acquisition records, including compensation already taxed as wages. Broker-reported basis may omit that compensation. This field does not calculate unresolved compensation or special elections. Choose “Basis cannot be established; assume zero basis” when it cannot be established: the estimate then counts the whole proceeds as gain.
    /// [NullText]
    /// Basis cannot be established; assume zero basis
    /// [SetText]
    /// My established adjusted tax basis is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: Option<i64>,
    /// AMT sale facts
    pub amt: SaleAmt,
    /// Federal income tax withheld for this sale
    /// Allocate the payer statement's withholding once across its entries; do not repeat the statement total for each item.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// Asset use and any investment loss adjustment
    pub use_kind: SaleUse,
    /// A treatment this page does not calculate
    /// An ordinary sale of nonstatutory-option shares or vested stock awards may use this worksheet when compensation has already been correctly reported, adjusted basis and holding period are established, and no deferral, restricted-property election or other special treatment remains. Do not enter the wages again as sale proceeds or add them to an already adjusted basis a second time. ISO sales use their dedicated worksheet. Form 1099-B and the broker's supplemental statement show it; an unknown basis is the first choice. Any choice but None of these leaves this sale out of the estimate with a notice; those treatments are not supported.
    pub special: SaleSpecial,
    /// Form 8949 column (f)/(g) adjustment code
    /// At most one code applies to this lot. A sale needing more than one code is not supported by this worksheet.
    pub adjustment: SaleAdjustment,
    /// Form 1099-B or 1099-DA basis reporting
    /// Whether the broker's Form 1099-B or 1099-DA reported this basis to the IRS, reported it without notifying the IRS, or whether no form was received at all. Together with the holding period and the digital asset answer, this selects the Form 8949 box (A-L) and Schedule D line this lot is totaled on.
    pub basis_reporting: BasisReporting,
    /// A digital asset reported on Form 1099-DA, or sold with no Form 1099-DA or 1099-B
    /// Selects Form 8949 box G to L instead of A to F.
    #[libertas_default(false)]
    pub digital_asset: bool,
    /// Accrued market discount (Form 1099-B box 1f, Form 1099-DA box 1h)
    /// The gain up to this amount is interest (Form 8949 code D).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// This entry totals several sales (Form 8949 code M)
    #[libertas_default(false)]
    pub several_sales: bool,
}

/// Form 8949 column (f)/(g) adjustment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SaleAdjustment {
    /// No adjustment code applies
    None,
    /// Code B: the payer statement's reported basis was incorrect
    /// Enter the actual adjusted basis above in the sale's basis field. This carries the payer statement's incorrect basis so column (e)/(g) can show the correction.
    BasisCorrection {
        /// Basis the payer statement reported
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        reported_basis: i64,
    },
    /// Code T: the payer statement's holding period was incorrect
    /// This sale's holding period above is the corrected term.
    HoldingPeriodCorrection,
    /// Code E: selling expenses not reflected on the payer statement
    SellingExpenses {
        /// Selling expenses not already subtracted from proceeds
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Code N: you received this form as a nominee for the actual owner
    /// The sale is shown and its gain or loss backed out, so it adds nothing to your income. File a Form 1099-B naming the actual owner, who reports the sale.
    Nominee,
    /// Code E: an option premium the form does not reflect
    /// For shares sold through an exercised option when the form's proceeds leave the premium out (box 6 shows gross proceeds): the premium you received for a call you wrote is added; what you paid for a put you bought and exercised is subtracted. A call you bought or a put you wrote changes the shares' basis instead (code B, or the basis above).
    OptionPremium {
        /// Premium: positive if you received it, negative if you paid it
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        amount: i64,
    },
}

/// Form 1099-B or 1099-DA basis reporting
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BasisReporting {
    /// Reported to the IRS (Form 8949 box A/D, or G/J for a digital asset)
    ReportedToIrs,
    /// Not reported to the IRS (Form 8949 box B/E, or H/K for a digital asset)
    /// A Form 1099-B or 1099-DA was received, but its basis was not reported to the IRS.
    NotReported,
    /// No Form 1099-B or 1099-DA was received (Form 8949 box C/F, or I/L for a digital asset)
    NoForm1099B,
}

/// AMT sale facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum SaleAmt {
    /// Same as regular tax
    /// The regular basis and wash-sale adjustment also apply for AMT. Your AMT records (the Form 6251 basis worksheets kept with your returns) show any difference, such as shares from an incentive stock option or property with different AMT depreciation.
    SameAsRegular,
    /// Different for AMT
    /// Supply the adjusted amounts from your AMT records. This does not calculate unresolved stock-compensation income, depreciation recapture or other special treatment.
    Different {
        /// AMT sale facts
        record: CapitalSaleAmt,
    },
}

impl SaleAmt {
    pub(crate) fn different(&self) -> Option<&CapitalSaleAmt> {
        match self {
            Self::SameAsRegular => None,
            Self::Different { record } => Some(record),
        }
    }
}

/// AMT capital-asset sale facts
/// Use the same sold asset and proceeds as the regular sale. AMT basis and, when established by separate records, holding period may differ.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CapitalSaleAmt {
    /// Different AMT capital-gain holding period, if applicable
    /// Choose “Use this sale's regular-tax holding period” when the regular holding period also applies. Use only an established AMT holding period, for example where an AMT election changes the holding start. Unresolved stock compensation still requires its source worksheet.
    /// [NullText]
    /// Use this sale's regular-tax holding period
    /// [SetText]
    /// The different AMT holding period is
    #[libertas_default($null)]
    pub term: Option<GainTerm>,
    /// Adjusted AMT basis of the sold asset
    /// Include prior AMT basis adjustments and acquisition costs. Exclude selling costs already removed from proceeds.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Disallowed AMT wash-sale loss
    /// Refigure from AMT loss and replacement-share records; enter zero for personal-use property. Do not copy a regular-tax wash loss when the AMT loss differs.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wash_sale: i64,
}

/// A treatment of a sale this page does not calculate
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
pub enum SaleSpecial {
    /// None of these
    #[default]
    NoneOfThese,
    /// Stock compensation not yet in your basis
    /// Wages or stock awards whose compensation isn't in a W-2 or isn't resolved. Ordinary employee shares have their own entries on this page.
    StockCompensation,
    /// Business property
    /// Property you used in a business or rental, or formerly used there.
    BusinessProperty,
    /// Your home
    Home,
    /// Another treatment this page does not calculate
    /// A contingent payment debt instrument (box 2 Ordinary), a rollover of gain (code R), the DC Zone or qualified community asset exclusion (code X), a trader's mark-to-market election, a carried-interest gain, or a loss your broker says the proceeds can't support.
    Another,
}
impl SaleSpecial {
    // Answers and unknowns: the notice of the treatment that leaves the sale
    // out of the estimate; None for no treatment.
    pub(crate) fn notice(self) -> Option<&'static str> {
        match self {
            Self::NoneOfThese => None,
            Self::StockCompensation => Some("WARN_SALE_STOCK_COMPENSATION_LEFT_OUT"),
            Self::BusinessProperty => Some("WARN_SALE_BUSINESS_PROPERTY_LEFT_OUT"),
            Self::Home => Some("WARN_SALE_HOME_LEFT_OUT"),
            Self::Another => Some("WARN_SALE_OTHER_LEFT_OUT"),
        }
    }
}

// Printed broker documents and manual entries use the same relationship
// checks after their source and personal facts form the native sale view.
pub(crate) fn native_sale_issue(v: &FederalSale) -> Option<ValidationIssue> {
    let wash = match v.use_kind {
        SaleUse::Investment { wash_sale } | SaleUse::WrittenOption { wash_sale } => wash_sale,
        SaleUse::Personal => 0,
    };
    let written = matches!(v.use_kind, SaleUse::WrittenOption { .. });
    let investment = matches!(v.use_kind, SaleUse::Investment { .. });
    if wash != 0 && i128::from(wash) > i128::from(v.basis.unwrap_or(0)) - i128::from(v.proceeds) {
        Some(
            ValidationIssue::new("SALE_WASH_SALE_ABOVE_LOSS")
                .at_field("wash_sale")
                .in_field("use_kind"),
        )
    } else if v.proceeds < 0 && !written {
        Some(
            ValidationIssue::new("SALE_NEGATIVE_PROCEEDS_WRITTEN_OPTION_ONLY").at_field("proceeds"),
        )
    } else if written && v.term != GainTerm::Short {
        Some(ValidationIssue::new("SALE_WRITTEN_OPTION_SHORT_TERM").at_field("term"))
    } else if matches!(v.adjustment, crate::SaleAdjustment::OptionPremium { .. }) && !investment {
        // The adjustment is the person's answer; the printed sale alone never has one.
        Some(ValidationIssue::new("SALE_OPTION_PREMIUM_ON_STOCK").at_field("adjustment"))
    } else if v.market_discount != 0 && v.basis.is_none() {
        Some(ValidationIssue::new("SALE_MARKET_DISCOUNT_NEEDS_BASIS").at_field("basis"))
    } else if v.market_discount != 0 && !investment {
        Some(ValidationIssue::new("SALE_MARKET_DISCOUNT_ON_A_BOND").at_field("market_discount"))
    } else {
        None
    }
}
