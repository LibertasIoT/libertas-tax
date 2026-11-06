use crate::common::Owner;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

// Pub. 527 Worksheet 5-1 reads this return invocation's AGI and SALT MAGI
// (`agi`) and its deduction route. Line 2c figures the dwelling's casualty
// Form6251 Related Adjustments retains regular AGI-based limits for AMT;
// AMT property basis and losses still settle independently. The allowance is
// on Form 4684 Section A with 10% of the AGI figured without that dwelling's
// rental income and expenses (Pub. 527, Worksheet 5-1 line 2c instructions).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HomeUseContext {
    pub agi: [i64; 2],
    // Some(true) itemizes and Some(false) takes the standard deduction in
    // this invocation; None is an automatic choice still being compared.
    pub itemizes: Option<bool>,
    pub(crate) dwelling_agi_cents: alloc::collections::BTreeMap<i64, i64>,
}

// Settled cash-source metadata for the native Form1040 withholding categories.
// Rows retain actual selected-return source Cents; targets retain the existing
// separately rounded annual Dollars. This packet performs no tax allocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeWithholdingSource {
    pub(crate) id: i64,
    pub(crate) owner: Owner,
    pub(crate) cents: [i64; 3],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeWithholdingSources {
    pub(crate) rows: alloc::vec::Vec<NativeWithholdingSource>,
    pub(crate) joint_dollars: [i64; 3],
    pub(crate) complete: bool,
    // An inherited record is credited once; its linked statement identities
    // remain available for validating the actual source relationship.
    pub(crate) inherited_payment_ids: alloc::collections::BTreeMap<i64, alloc::vec::Vec<i64>>,
}

// Deduction-specific Save requiredness uses shared person attribution.
// The native financial source owners retain these final actual operands.
// Final native operands retained for Save requiredness. None is an unavailable
// native component, not a known zero. This packet never reruns an expense cap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExpenseRoleDollars {
    pub(crate) itemizing: bool,
    pub(crate) itemized_dollars: i64,
    pub(crate) property_dollars: i64,
    pub(crate) interest_dollars: Option<i64>,
    pub(crate) connected_dollars: Option<i64>,
    pub(crate) salt_dollars: i64,
    pub(crate) income_tax_category_dollars: i64,
    pub(crate) foreign_dollars: i64,
    pub(crate) current_paid_dollars: Option<i64>,
    pub(crate) retained_carry_dollars: Option<i64>,
}

/// Injured Amount Page
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredAmountPage {
    /// Accepted component
    #[libertas_enum_source("$.value.components")]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_size(min = 1)]
    pub component: String,
    /// Shared item
    #[libertas_formatted_text]
    #[libertas_read_only]
    #[libertas_output_only]
    pub label: Vec<u8>,
    /// Amount on the joint return
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub joint: i64,
    /// Minimum injured-spouse part
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub minimum: i64,
    /// Maximum injured-spouse part
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub maximum: i64,
    /// Injured spouse's part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub injured: i64,
}

/// Injured Printed Row
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredPrintedRow {
    /// Row
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Joint return
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub joint: i64,
    /// Injured spouse
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub injured: i64,
    /// Other spouse
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub other: i64,
}

/// Injured Amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredAmount {
    /// Accepted component key
    #[libertas_size(min = 1)]
    pub component: String,
    /// Injured spouse's allocated amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub injured: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AllocationPlan {
    pub(crate) required: Vec<InjuredAmountPage>,
    // Bounded real answers and separately approved automatic initial values.
    pub(crate) accepted: Vec<InjuredAmount>,
    pub(crate) known: Vec<InjuredPrintedRow>,
    pub(crate) known_child: [i64; 2],
    pub(crate) child_caps: [i64; 2],
}

/// Federal income tax withheld
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Withheld {
    /// From your W-2s
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub w2: i64,
    /// From your 1099s
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub forms_1099: i64,
    /// From other forms
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub other: i64,
    /// Total withheld
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub total: i64,
}

/// Farmland tax installment election
/// Payment timing only: the full estimated federal tax remains unchanged. The first installment is included in the current amount due; the remaining balance is deferred, not forgiven. File Form 1062, a separate Schedule A and covenant for each elected sale. The App does not file or pay. Later installments are due on the unextended return deadlines for the next three years; default or death can accelerate the unpaid balance. Whole-dollar rounding may require a final installment adjustment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmTaxInstallments {
    /// Tax attributable to all elected sales
    #[libertas_money("USD", 2)]
    pub applicable: i64,
    /// First installment due by the unextended return deadline
    #[libertas_money("USD", 2)]
    pub first: i64,
    /// Remaining liability deferred to later years
    #[libertas_money("USD", 2)]
    pub remaining: i64,
    /// Net current federal tax position before other payments and refundable credits
    /// Annual tax less the deferred balance. A negative amount contributes to the estimated overpayment; it is not negative annual tax or forgiveness of future installments.
    #[libertas_money("USD", 2)]
    pub current: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeElectiveGroup {
    pub(crate) component: alloc::string::String,
    pub(crate) label: &'static str,
    pub(crate) amount: i64,
    pub(crate) elected: i64,
    pub(crate) allowed: i64,
    pub(crate) used: i64,
    pub(crate) elective: i64,
    pub(crate) allowed_owners: Option<[i64; 2]>,
    pub(crate) elected_owners: Option<[i64; 2]>,
}

// Native Form3800 amounts are already reported whole Dollars. These fields
// retain that precision and the same invocation's settled allowance/use.
// Current credits also retain ownership with no elective-payment election;
// prior non-elective carries keep their existing separate history context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeElectiveSources {
    pub(crate) joint: i64,
    pub(crate) used_joint: i64,
    pub(crate) groups: Vec<NativeElectiveGroup>,
}
