use super::*;

// Complete server context for normal WorkflowRequest construction. The reduced
// projection omits every unanswered required amount; it never sends a partial
// Request or treats a range bound as an answer.
/// Injured spouse allocation context
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct InjuredSpouseInitial {
    /// Injured spouse
    pub injured: Owner,
    /// Debt the refund pays
    pub debt: OffsetDebt,
    /// The injured spouse is legally obligated to pay this debt
    pub obligated: bool,
    /// Community-property residence during 2026
    pub community: InjuredCommunity,
    /// Joint estimated payments
    /// Separate payments already belong to their payer. Agree on the joint amount, or use the IRS separate-tax formula.
    pub estimated: EstimatedSplit,
    /// Who is legally obligated on the loans whose interest is deducted
    /// Leave empty when there is no student-loan deduction. A borrower who is only a dependent student does not make a spouse legally obligated.
    /// [NullText]
    /// No student-loan interest deduction to allocate
    /// [SetText]
    /// The legally obligated student-loan borrower is
    pub student_loan_borrower: Option<InjuredLoanBorrower>,
    /// Previously answered shared amounts
    /// Review the injured spouse's part of each amount. The other spouse receives the rest.
    #[libertas_size(min = 0)]
    #[libertas_read_only]
    pub allocations: Vec<InjuredAmountAnswered>,
    /// New shared amounts to allocate
    /// Enter the injured spouse's part of each amount. The other spouse receives the rest.
    #[libertas_size(min = 0)]
    #[libertas_read_only]
    pub new_allocations: Vec<InjuredAmountInitial>,
    /// Dependents whose separate-return claimant is needed
    #[libertas_size(min = 0)]
    #[libertas_read_only]
    pub claimants: Vec<InjuredClaimant>,
    /// Joint estimated payments available to divide
    /// [NullText]
    /// The exact capacity exceeds the displayed amount's native representation
    /// [SetText]
    /// Available to divide
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub joint_estimated: Option<i64>,
    /// Actual student-loan deduction on the joint return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub student_loan_deduction: i64,
    /// Available community-property states
    #[libertas_hidden]
    #[libertas_read_only]
    pub states: Vec<InjuredStateChoice>,
    /// Dependents on this return
    #[libertas_hidden]
    #[libertas_read_only]
    pub people: Vec<PersonChoice>,
    /// Accepted component catalog
    #[libertas_hidden]
    #[libertas_read_only]
    pub components: Vec<InjuredComponentChoice>,
    /// Applicable debt facts
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_debt_facts: Vec<i32>,
    /// Children's credits already attributed to the injured spouse
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub known_child_credit_injured: i64,
    /// Children's credits already attributed to the other spouse
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub known_child_credit_other: i64,
    /// Combined child-credit ceiling for the injured spouse's claimants
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub child_credit_cap_injured: i64,
    /// Combined child-credit ceiling for the other spouse's claimants
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub child_credit_cap_other: i64,
    /// Allocation rows
    #[libertas_read_only]
    pub known: Vec<InjuredPrintedRow>,
    /// Prospective protected part of the joint overpayment
    /// Before later IRS refunds, offsets or outstanding debts. The IRS makes the final computation.
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub estimated_share: Option<i64>,
    /// Allocation explanation
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub explanation: Vec<u8>,
}

// Response construction context cannot reuse the Request row: its label is
// OutputOnly and omitted from a submitted Request, but supplied in this reply.
/// Previously answered shared allocation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct InjuredAmountAnswered {
    /// Component
    pub component: String,
    /// Shared item
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Amount on the joint return
    #[libertas_money("USD", 2)]
    pub joint: i64,
    /// Minimum injured-spouse part
    #[libertas_money("USD", 2)]
    pub minimum: i64,
    /// Maximum injured-spouse part
    #[libertas_money("USD", 2)]
    pub maximum: i64,
    /// Injured spouse's part
    #[libertas_money("USD", 2)]
    pub injured: i64,
}

/// Shared allocation context
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct InjuredAmountInitial {
    /// Component
    pub component: String,
    /// Shared item
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Amount on the joint return
    #[libertas_money("USD", 2)]
    pub joint: i64,
    /// Minimum injured-spouse part
    #[libertas_money("USD", 2)]
    pub minimum: i64,
    /// Maximum injured-spouse part
    #[libertas_money("USD", 2)]
    pub maximum: i64,
}

impl InjuredSpouseAllocation {
    pub(crate) fn construction(
        &self,
        draft: &FederalDraft,
        result: &FederalResult,
    ) -> InjuredSpouseInitial {
        let page = self.page(draft, result);
        let mut allocations = Vec::new();
        let mut new_allocations = Vec::new();
        for row in page.allocations {
            let accepted = result.injured_spouse.as_ref().and_then(|plan| {
                let mut matches =
                    plan.accepted.iter().filter(|answer| answer.component == row.component);
                match (matches.next(), matches.next()) {
                    (Some(answer), None) => Some(answer),
                    _ => None,
                }
            });
            if let Some(answer) = accepted {
                allocations.push(InjuredAmountAnswered {
                    component: row.component,
                    label: row.label,
                    joint: row.joint,
                    minimum: row.minimum,
                    maximum: row.maximum,
                    injured: answer.injured,
                });
            } else {
                new_allocations.push(InjuredAmountInitial {
                    component: row.component,
                    label: row.label,
                    joint: row.joint,
                    minimum: row.minimum,
                    maximum: row.maximum,
                });
            }
        }
        InjuredSpouseInitial {
            injured: page.injured,
            debt: page.debt,
            obligated: page.obligated,
            community: page.community,
            estimated: page.estimated,
            student_loan_borrower: page.student_loan_borrower,
            allocations,
            new_allocations,
            claimants: page.claimants,
            joint_estimated: page.joint_estimated,
            student_loan_deduction: page.student_loan_deduction,
            states: page.states,
            people: page.people,
            components: page.components,
            allowed_debt_facts: page.allowed_debt_facts,
            known_child_credit_injured: page.known_child_credit_injured,
            known_child_credit_other: page.known_child_credit_other,
            child_credit_cap_injured: page.child_credit_cap_injured,
            child_credit_cap_other: page.child_credit_cap_other,
            known: page.known,
            estimated_share: page.estimated_share,
            explanation: page.explanation,
        }
    }
}
