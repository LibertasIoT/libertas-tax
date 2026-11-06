// Coverage: Section 1202 qualified small business stock (B7a, 2026-10-09). QsbIssuerSale (one issuing
//   corporation's 2026 sales of original-issue stock held more than five years, issued August 11,
//   1993 through July 4, 2025: 50, 75 or 100 percent exclusion by issue date, 60 percent with the
//   empowerment-zone increase, the per-issuer limit of the greater of $10,000,000 (MFS $5,000,000)
//   less prior eligible gain and ten times the issuer stock basis sold this year, the included part
//   of a 50, 60 or 75 percent exclusion as 28 percent rate gain, the 7 percent AMT preference of
//   Form 6251 line 2h for stock acquired before September 28, 2010; IRC 1202, 1(h)(4) and (7),
//   57(a)(7); Instructions for Schedule D) gains its page, QsbsSale, on Investment sales, with the
//   sale rows, box 4 and the Form 8949 box imported and the issuer facts entered. Stock issued
//   after July 4, 2025 falls under the P.L. 119-21 section 70431 three-, four- and five-year tiers,
//   whose shortest holding period cannot be met by a 2026 sale (earliest July 5, 2028), so such a
//   lot is refused by QSBS_LOT_OUTSIDE_SECTION_1202_DATES; the limits and dates are in
//   year_constants.rs. The taxable gain is reported on Form 8949 with code Q in the box the page
//   names (D, E or F) and totals on Schedule D line 8b, 9 or 10; the excluded part is column (g)
//   negative. No qualification answer of Yes, or an incomplete history, taxes the whole gain
//   (WARN_QSBS_ASSUMED_NOT_QUALIFIED, WARN_QSBS_EXCLUSION_LEFT_OUT); special treatment leaves the
//   sale out with WARN_QSBS_SALE_LEFT_OUT. Forms: 8949 (code Q), Schedule D, 6251 line 2h, 28%
//   Rate Gain Worksheet line 2. Tests: src/tests/investment/investment_kinds.rs qsbs_*.
//   Client lot-date validation uses typed calendar dates and add_months for the
//   strict five-year hold, matching the unchanged native supported-date gate.
//   Tests: investment_kinds validation_audit_f11_* (fresh/null/reopen/rejected state).
//   Native source outputs retain admitted per-entry/owner28% cents and actual
//   cohort/zone taxable rows; issuer totals and uniform-cap rounding are unchanged.
//   Existing refused mixed caps/history remain unavailable, not guessed owners.
//   Evidence: tests/unit13_qsbs_source.rs (mixed dates/owners/zone/caps/cents/Save).
use crate::kernel::phase;
use crate::year_constants as yc;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const MAX: i64 = 100_000_000_000;

/// Qualified small-business stock lot
/// A directly purchased original-issue lot sold for cash in 2026. Enter separate lots for different issue dates or original bases. Gifts, inheritances, exchanged shares, pass-through shares, installment sales and section 1045 rollovers require their separate rules.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct QsbStockLot {
    /// Original-issue acquisition date
    /// This path covers shares acquired August 11, 1993 through July 4, 2025 and held more than five years. The acquisition date determines the 50%, 75% or 100% exclusion. In a 2026 direct sale, shares issued after July 4, 2025 cannot yet satisfy their minimum holding period.
    #[libertas_date_only]
    pub issued: u32,
    /// 2026 cash sale date
    #[libertas_date_only]
    pub sold: u32,
    /// Original issue price and capitalized acquisition costs for this sold lot
    /// Do not add later capital contributions or another basis increase. This same unchanged amount is both adjusted sale basis and the original-issue basis used for the ten-times-basis limit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub basis: i64,
    /// Net cash proceeds after selling costs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Gain attributable to time through December 31, 2018, for the 60% empowerment-zone exclusion
    /// Complete only when this lot was acquired December 22, 2000 through February 17, 2009 and the issuer qualified as an empowerment-zone business for the required period. Establish value at December 31, 2018 (or an earlier cessation date) and limit this amount to actual 2026 realized gain. Enter gain, not proceeds or a second stock basis.
    pub zone_gain_through_2018: Option<crate::QsbsZoneValuation>,
}

impl QsbStockLot {
    // The dates a 2026 sale of original-issue stock must meet: issued from
    // August 11, 1993 through July 4, 2025, sold in 2026 and more than five
    // years after the issue.
    fn dates_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if !crate::dates::source_valid_date(self.issued)
            || !(yc::SECTION_1202_FIRST_ISSUE_DATE..=yc::SECTION_1202_LAST_ORIGINAL_TIER_ISSUE_DATE)
                .contains(&self.issued)
        {
            return Some(
                ValidationIssue::new("QSBS_LOT_OUTSIDE_SECTION_1202_DATES").at_field("issued"),
            );
        }
        if !crate::dates::source_valid_date(self.sold)
            || self.sold / 10000 != 2026
            || self.sold <= self.issued + yc::SECTION_1202_HOLDING_YEARS_AS_DATE
        {
            return Some(
                ValidationIssue::new("QSBS_LOT_OUTSIDE_SECTION_1202_DATES").at_field("sold"),
            );
        }
        None
    }
    pub(crate) fn dates_valid(&self) -> bool {
        self.dates_issue().is_none()
    }

    // The first page relationship this lot fails, by its key (the client
    // rules above and their server twins): a loss or break-even sale of
    // this stock is a 1099-B sale, not a section 1202 gain, and a zone-gain
    // figure must fit the lot's gain and the zone dates.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.dates_issue() {
            return Some(issue);
        }
        if self.proceeds <= self.basis {
            return Some(
                crate::validation::ValidationIssue::new("QSBS_LOT_PROCEEDS_NOT_ABOVE_BASIS")
                    .at_field("proceeds"),
            );
        }
        let zone = self.zone_gain_through_2018.and_then(crate::QsbsZoneValuation::gain)?;
        if !(0..=self.proceeds - self.basis).contains(&zone) {
            return Some(
                crate::validation::ValidationIssue::new("QSBS_LOT_ZONE_GAIN_OUTSIDE_THE_LOT_GAIN")
                    .at_field("zone_gain_through_2018"),
            );
        }
        if !(yc::SECTION_1202_ZONE_FIRST_DATE..=yc::SECTION_1202_FIFTY_PERCENT_LAST_DATE)
            .contains(&self.issued)
        {
            return Some(
                crate::validation::ValidationIssue::new("QSBS_LOT_ZONE_GAIN_OUTSIDE_ZONE_DATES")
                    .at_field("zone_gain_through_2018"),
            );
        }
        None
    }

    pub(crate) fn valid(&self) -> bool {
        self.dates_valid()
            && (1..=MAX).contains(&self.basis)
            && (0..=MAX).contains(&self.proceeds)
            && self.proceeds > self.basis
            && self.zone_gain_through_2018.and_then(crate::QsbsZoneValuation::gain).is_none_or(
                |zone| {
                    (yc::SECTION_1202_ZONE_FIRST_DATE..=yc::SECTION_1202_FIFTY_PERCENT_LAST_DATE)
                        .contains(&self.issued)
                        && (0..=self.proceeds - self.basis).contains(&zone)
                },
            )
    }
}

/// Direct sale of qualified small-business stock — section 1202
/// Enter all 2026 sales for one issuing corporation, using the same issuer name and prior-gain history if multiple source records or spouses are entered. The exclusion limit applies once to all sales from that issuer on this return, not once per lot or document.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbIssuerSale {
    /// Exact legal name of the issuing corporation
    /// Use the same name on every entry for this issuer. Do not use a broker name, brand, predecessor name or ticker in place of the legal issuer identity.
    #[libertas_size(min = 1)]
    pub issuer: String,
    /// Eligible gain already taken into account under section 1202 for this issuer before 2026
    /// Reconcile every prior return of the current taxpayer and, for a joint return, both spouses. Allocate gain taken into account on an earlier joint return equally to its spouses for later years. Enter zero only after checking the complete history. Do not enter prior sale proceeds or prior taxable gain here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_eligible_gain: Option<i64>,
    /// Bought other than for cash at original issue, or a gift, inheritance, exchange, pass-through ownership, installment sale, rollover or another special treatment
    /// Yes leaves this sale out of the estimate with a notice; that treatment is not supported.
    pub special: Option<bool>,
    /// Qualified domestic C corporation and gross-assets test confirmed for every issue date
    /// The issuer was a domestic C corporation. Aggregate assets of the issuer, its predecessors and its controlled group did not exceed $50 million immediately before and after each original issue; use the statutory contribution valuation rules. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub issuer_qualified: Option<bool>,
    /// Active qualified business confirmed throughout substantially all of each holding period
    /// At least 80% of asset value was used in a qualified active business. Exclude disallowed professional services, finance, investment, farming, mining and hospitality businesses; account for subsidiaries, working capital and permitted exceptions from issuer records. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub active_business: Option<bool>,
    /// No disqualifying issuer redemption or other section 1202 exception
    /// Check the applicable shareholder and significant redemption windows around every issuance; confirm no special rule or election changes qualification, gain recognition or tax character. No applies no section 1202 exclusion, so the whole gain is taxed.
    pub no_disqualification: Option<bool>,
    /// Extra 60% empowerment-zone exclusion applies to one or more sold lots
    /// For shares acquired December 22, 2000 through February 17, 2009, confirm No only after checking the issuer's empowerment-zone status and gain through 2018. Choose Yes when the issuer qualified and the source valuation determines the eligible gain for each affected lot. Check the issuer's retained zone certification and valuation; an unavailable historical certification or valuation leaves the increase unclaimed with a notice. Leave absent when all lots are outside this issue-date interval.
    // When an applicable issuer's historical zone certification or valuation is unavailable,
    // purchase/sale amounts cannot reconstruct it; WARN_QSBS_ZONE_ASSUMED_NO omits the increase.
    // Outside the statutory issue-date interval the absent answer remains not applicable.
    pub zone_60_exclusion: Option<crate::QsbsZoneCertification>,
    /// Complete issuer history and all current-year lots confirmed
    /// The prior-gain figure and every 2026 disposition of this issuer's qualifying shares on the filing unit are included once. A joint return needs its spouses' separate and joint prior-return history. Your prior returns' section 1202 exclusions for this issuer show it. No applies no section 1202 exclusion, so the whole gain is taxed, with a notice.
    pub complete_history: Option<bool>,
    /// 2026 sold lots
    pub lots: Vec<QsbStockLot>,
    /// Form 8949 box of the sales
    /// Box D when the broker reported the basis to the IRS, E when it did not, F when no Form 1099-B was received.
    pub reporting: crate::BasisReporting,
    /// Federal income tax withheld from these sales
    /// Allocate payer-statement withholding once; do not repeat the same Form 1099 amount on another entry.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
}

impl QsbIssuerSale {
    pub(crate) fn issuer_key(&self) -> String {
        self.issuer.trim().to_ascii_lowercase()
    }
    pub(crate) fn valid(&self) -> bool {
        !self.issuer.trim().is_empty()
            && self.prior_eligible_gain.is_none_or(|gain| (0..=MAX).contains(&gain))
            && (0..=MAX).contains(&self.withholding)
            && !self.lots.is_empty()
            && self.lots.iter().all(QsbStockLot::valid)
    }
}

// A fund's section 1202 gain through a regulated investment company (IRC 1202(g)(4)(B); Form
// 1099-DIV box 2c, Form 2439 box 1c), in cents: the fund's gain is one issuer's, so the per-issuer
// limit (the greater of $10,000,000, $5,000,000 if married filing separately, and ten times the
// fund's basis, unknown here and so not used) caps what is eligible. The exclusion follows the
// date the fund bought the stock; no answer other than None gives no exclusion.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FundGain {
    pub excluded: i64,
    // The 28 percent rate gain left after a 50 or 75 percent exclusion.
    pub rate_28: i64,
    pub amt_preference: i64,
}

pub(crate) fn fund_gain(
    gain: i64,
    exclusion: Option<crate::FundQsbsExclusion>,
    separate: bool,
) -> FundGain {
    let percent = match exclusion {
        Some(crate::FundQsbsExclusion::Fifty) => 50,
        Some(crate::FundQsbsExclusion::SeventyFive) => 75,
        Some(crate::FundQsbsExclusion::Hundred) => 100,
        Some(crate::FundQsbsExclusion::None) | None => 0,
    };
    let limit = if separate {
        yc::SECTION_1202_ISSUER_LIMIT_SEPARATE_CENTS
    } else {
        yc::SECTION_1202_ISSUER_LIMIT_CENTS
    };
    let eligible = phase::cap(gain, [limit]);
    let excluded = crate::kernel::ratio::fraction(eligible, percent, 100);
    let older = percent == 50 || percent == 75;
    FundGain {
        excluded,
        rate_28: if older { eligible - excluded } else { 0 },
        amt_preference: if older {
            crate::kernel::ratio::fraction(excluded, yc::SECTION_1202_AMT_PREFERENCE_PERCENT, 100)
        } else {
            0
        },
    }
}

/// Stock sale as printed on the source statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QsbsPaperLot {
    /// Lot identifier
    /// The lot, certificate or tax-lot number printed for this sale.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub lot_identifier: String,
    /// Number of shares
    /// [NullText]
    /// The number of shares is not printed
    /// [SetText]
    /// The printed number of shares is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub shares: Option<u64>,
    /// Original-issue acquisition date
    /// This path covers shares acquired August 11, 1993 through July 4, 2025 and held more than five years. The acquisition date determines the 50%, 75% or 100% exclusion. In a 2026 direct sale, shares issued after July 4, 2025 cannot yet satisfy their minimum holding period.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub issued: u32,
    /// 2026 cash sale date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub sold: u32,
    /// Original issue price and capitalized acquisition costs for this sold lot
    /// Do not add later capital contributions or another basis increase. This same unchanged amount is both adjusted sale basis and the original-issue basis used for the ten-times-basis limit.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub basis: i64,
    /// Net cash proceeds after selling costs
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
}

impl QsbsPaperLot {
    pub(crate) fn record(&self) -> QsbStockLot {
        QsbStockLot {
            issued: self.issued,
            sold: self.sold,
            basis: self.basis,
            proceeds: self.proceeds,
            zone_gain_through_2018: None,
        }
    }
}
impl From<&QsbStockLot> for QsbsPaperLot {
    fn from(lot: &QsbStockLot) -> Self {
        Self {
            lot_identifier: String::new(),
            shares: None,
            issued: lot.issued,
            sold: lot.sold,
            basis: lot.basis,
            proceeds: lot.proceeds,
        }
    }
}

/// Qualified small business stock sold (section 1202)
/// Printed sales of one corporation’s shares from a Form 1099-B or paying agent’s or closing statement. Enter its sale rows and withholding here; qualification, issuer history and historical valuations belong to Investment sales.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into source_issuer, source_recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the 2026 sales of one corporation's stock from a Form 1099-B or a paying agent's or closing statement exactly. Check the tax year against the supplied year; report a mismatch. Read the printed calendar year as an integer such as 2026, or zero when blank. Return the issuing corporation's legal name as printed as issuer. For each lots row also return the printed lot identifier as text (empty when none) and the number of shares sold (null when not printed). Return one lots row for each sale of that corporation's shares: box 1b date acquired as issued, box 1c date sold as sold, box 1d proceeds as proceeds and box 1e cost or other basis as basis; report a blank or illegible date or amount as an error, and never combine rows. If the document shows sales of more than one corporation's stock, report an error saying each corporation needs its own entry. Return ReportedToIrs when a Form 1099-B shows basis reported to the IRS (box 12 checked, or Form 8949 box D indicated), NotReported when it shows basis not reported (box 12 unchecked, or box E indicated), and NoForm1099B when the document is not a Form 1099-B. Read box 4 federal income tax withheld into withholding, returning zero when it is clearly blank. Do not read wash sales or empowerment-zone values, and do not decide whether the stock qualifies or the gain is excluded; the filer answers those. Copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.source_issuer.federal_id,@.source_recipient.federal_id,@.calendar_year,@.issuer,@.lots,@.withholding",
    form_date_order = "MonthDayYear"
)]
pub struct QsbsSale {
    /// Calendar year
    #[libertas_llm_doc_field]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// Source document issuer as printed
    #[libertas_llm_doc_field]
    pub source_issuer: crate::PrintedParty,
    /// Source document recipient as printed
    #[libertas_llm_doc_field]
    pub source_recipient: crate::PrintedParty,
    /// Issuing corporation
    /// Its exact legal name, the same on every entry for it; not a broker, brand or ticker.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_size(min = 1)]
    pub issuer: String,
    /// 2026 sales
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    /// ----
    /// Sale
    pub lots: Vec<QsbsPaperLot>,
    /// Basis reported to the IRS
    #[libertas_llm_doc_field]
    pub reporting: crate::BasisReporting,
    /// Federal income tax withheld
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
}

impl QsbsSale {
    // The server-checked relationship the lot dates must meet (stock issued
    // from August 11, 1993 through July 4, 2025, sold in 2026 more than five
    // years later), by its key.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.lots
            .iter()
            .enumerate()
            .find_map(|(i, lot)| lot.record().issue().map(|issue| issue.in_list("lots", i)))
    }
    // The stored issuer sale this page writes.
    pub(crate) fn record(&self) -> QsbIssuerSale {
        QsbIssuerSale {
            issuer: self.issuer.clone(),
            prior_eligible_gain: None,
            special: None,
            issuer_qualified: None,
            active_business: None,
            no_disqualification: None,
            zone_60_exclusion: None,
            complete_history: None,
            lots: self.lots.iter().map(QsbsPaperLot::record).collect(),
            reporting: self.reporting,
            withholding: self.withholding,
        }
    }
    // The reverse of record.
    pub(crate) fn from_record(v: &QsbIssuerSale) -> Self {
        Self {
            calendar_year: 0,
            source_issuer: Default::default(),
            source_recipient: Default::default(),
            issuer: v.issuer.clone(),
            lots: v.lots.iter().map(QsbsPaperLot::from).collect(),
            reporting: v.reporting,
            withholding: v.withholding,
        }
    }
}
