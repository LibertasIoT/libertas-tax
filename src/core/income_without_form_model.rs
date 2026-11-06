use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Income without a form
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct IncomeWithoutFormAnswers {
    /// Did you do any work for yourself — side jobs, gig or app work, cash jobs?
    /// Enter the receipts and any expenses on Self-employment.
    pub self_work: bool,
    /// Did you get tips you didn't report to your employer?
    /// Enter those tips and their payroll details on Unreported tips.
    pub tips: bool,
    /// Any other income no form shows?
    /// Include only income you haven't entered elsewhere on this return.
    /// [NullText]
    /// No
    /// [SetText]
    /// Yes — other income was
    pub other: Option<WithoutFormOther>,
}

/// Other income without a form
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WithoutFormOther {
    /// Amount received in 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Type of income
    /// Briefly describe the income; this description accompanies the amount on Schedule 1 line 8z.
    #[libertas_size(min = 1)]
    pub description: String,
}
