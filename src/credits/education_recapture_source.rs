// Filed education-credit history and intrinsic source consistency checks.
use crate::education_source::EducationMethod;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const MAX_CENTS: i64 = 100_000_000_000;

/// Filed 2025 education-credit history affected by assistance received in 2026
/// Use this ordinary single-credit path only when one student had the sole 2025 education credit and reducing it cannot change another 2025 credit or tax limit. Other histories need a full prior-return refigure.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PriorYearEducationCreditRecapture {
    /// Credit claimed on the filed 2025 return
    pub method: EducationMethod,
    /// Adjusted qualified expenses entered for that student on the filed 2025 Form 8863
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub filed_qualified_expenses: i64,
    /// 2026 refund or tax-free assistance allocable to those 2025 expenses
    /// Include only amounts received after the 2025 return was filed. Exclude amounts elected to reduce qualified 2026 expenses for the same academic period instead.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub after_filing_reduction: i64,
    /// 2025 modified AGI used on Form 8863 line 3 or 14
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub filed_education_magi: i64,
    /// Filed 2025 return was married filing jointly
    pub filed_joint: bool,
    /// Total 2025 education credit actually claimed, Form 8863 lines 8 plus 19
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub filed_credit_used: i64,
    /// Single-credit history and full prior tax benefit confirmed
    /// Check the filed 2025 Form 1040 and Form 8863 (students, credit used and filing status). Yes when this was the only 2025 education-credit student, the filed credit was fully usable, the filing unit (including the spouse on a joint return) is unchanged, and its refigure does not release or change any other 2025 credit or prior tax computation. No needs the complete prior return refigured separately before the recapture can be calculated.
    pub ordinary_prior_return: bool,
}

impl PriorYearEducationCreditRecapture {
    pub(crate) fn valid(&self) -> bool {
        [
            self.filed_qualified_expenses,
            self.after_filing_reduction,
            self.filed_education_magi,
            self.filed_credit_used,
        ]
        .iter()
        .all(|amount| (0..=MAX_CENTS).contains(amount))
            && self.after_filing_reduction <= self.filed_qualified_expenses
    }
}
