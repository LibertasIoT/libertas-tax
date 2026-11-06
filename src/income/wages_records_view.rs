use crate::{
    CashTipsWithoutW2, ClergyRecord, CommunityShare, IncomeRecord, IncomeRecordItem,
    InvestmentCapitalCap, Owner, RefundManagement,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Other wage records
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WagesRecords {
    /// Records for this topic
    #[libertas_unordered]
    /// ----
    /// Record
    pub documents: Vec<WagesRecordItem>,
}

/// Wage record
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WagesRecordItem {
    /// Calculated carryover bounds for this edited source document
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    #[libertas_output_only]
    pub investment_caps: Option<InvestmentCapitalCap>,
    /// Accepted record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Whose record
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Record
    #[libertas_ask_on_add]
    pub document: WagesRecord,
    /// Community or separate
    /// Community: pay earned while married and domiciled in a community-property state, and income from community property, including property you can't show is separate. Separate: income from property owned before the marriage, received as a gift or inheritance or bought with separate funds, pay earned while domiciled elsewhere, or property made separate by a written agreement. Partly: an item that is both, such as a pension earned partly during the marriage. Your state's law and any agreement between you decide this (Pub. 555).
    #[libertas_constrained_by("^<WagesRecordItem>.allowed_community")]
    pub community: CommunityShare,
    /// Amount on the return
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    #[libertas_output_only]
    pub preview: Option<i64>,
    /// Management of the funds from which this withholding was paid
    /// For Texas community-property refund allocation, use your ownership and any valid management agreement. This does not change who earned the wages or whose form reports the withholding.
    /// [NullText]
    /// No management of the withholding funds is recorded
    /// [SetText]
    /// The withholding funds were managed as
    #[libertas_default($null)]
    #[libertas_constrained_by("$.allowed_refund_management")]
    pub refund_management: Option<RefundManagement>,
    /// Texas source-management answer is needed
    /// The service derives this from the settled native payment source. Null
    /// means that source is not yet available; it is not a negative answer.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub refund_management_needed: Option<bool>,
    /// Allowed community-property answers for this row
    /// A row the person adds starts with the page's answers until its
    /// document's own answers arrive.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.allowed_community")]
    #[libertas_output_only]
    pub allowed_community: Vec<i32>,
}

/// Wage record
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum WagesRecord {
    /// Cash tips not reported on a W-2
    /// One record per employer. Do not repeat tips already entered on that employer's W-2 tip records.
    CashTipsWithoutW2(CashTipsWithoutW2),
    /// Minister, religious worker or Form4029 exemption
    /// One per person. Mark ministerial W2s and business fees separately. Earnings under a vow of poverty belong to the order; do not enter them. A retired minister's pension housing designation is not excluded here; its otherwise taxable pension amount remains taxable. Enter foreign church pay without a W2 as a ministerial business.
    Clergy(ClergyRecord),
}

impl WagesRecordItem {
    pub(crate) fn from_income(row: IncomeRecordItem) -> Option<Self> {
        let IncomeRecordItem {
            investment_caps,
            id,
            owner,
            document,
            community,
            preview,
            refund_management,
            refund_management_needed,
            allowed_community,
        } = row;
        let document = match document {
            IncomeRecord::CashTipsWithoutW2(value) => WagesRecord::CashTipsWithoutW2(value),
            IncomeRecord::Clergy(value) => WagesRecord::Clergy(value),
            _ => return None,
        };
        Some(Self {
            investment_caps,
            id,
            owner,
            document,
            community,
            preview,
            refund_management,
            refund_management_needed,
            allowed_community,
        })
    }

    pub(crate) fn into_income(self) -> IncomeRecordItem {
        let Self {
            investment_caps,
            id,
            owner,
            document,
            community,
            preview,
            refund_management,
            refund_management_needed,
            allowed_community,
        } = self;
        IncomeRecordItem {
            investment_caps,
            id,
            owner,
            document: match document {
                WagesRecord::CashTipsWithoutW2(value) => IncomeRecord::CashTipsWithoutW2(value),
                WagesRecord::Clergy(value) => IncomeRecord::Clergy(value),
            },
            community,
            preview,
            refund_management,
            refund_management_needed,
            allowed_community,
        }
    }
}
