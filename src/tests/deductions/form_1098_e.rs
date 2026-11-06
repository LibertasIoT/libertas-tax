use super::*;

// Box 2 decides whether the filer's additional interest joins box 1; the lender
// and borrower blocks reach no federal figure.
#[test]
fn form_1098_e_qualified_interest_follows_box_2() {
    let form = Form1098E { interest: 100, box_2: true, ..Default::default() };
    let mut source = StudentLoanSource::printed(7, student_loan_paper(form));
    source.additional_interest = 25;
    assert_eq!(source.qualified_interest(), 125);
    assert!(source.valid());
    // An answer kept from an earlier correction counts only while box 2 is checked.
    source.document.box_2 = false;
    assert_eq!(source.qualified_interest(), 100);
    source.document.box_1 = -1;
    assert!(!source.valid());
}
