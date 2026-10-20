use super::*;
use crate::income_records::{IncomeRecordError as Error, assemble, project};

#[test]
fn native_identity_order_pruning_and_empty_income_are_lossless() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(personal_sale(-12345, 67890, GainTerm::Long)));
    draft.income[0].id = i64::MAX - 2;
    draft.income[1].id = 42;
    draft.next_id = i64::MAX - 1;
    draft.tax_year = 2031;
    let mut records = project(TEST_USER, 9, &draft).unwrap();
    records.reverse();
    let mut target = draft.clone();
    target.income.clear();
    assemble(TEST_USER, 9, &mut target, 2, records).unwrap();
    assert_eq!(target, draft);
    draft.income.remove(0);
    assemble(TEST_USER, 9, &mut target, 1, project(TEST_USER, 9, &draft).unwrap()).unwrap();
    assert_eq!(target, draft, "a pruned native identity is not retained");
    draft.income.clear();
    assemble(TEST_USER, 9, &mut target, 0, project(TEST_USER, 9, &draft).unwrap()).unwrap();
    assert_eq!(target, draft);
}

#[test]
fn malformed_income_scopes_leave_the_entire_target_unchanged() {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(personal_sale(12345, 67890, GainTerm::Short)));
    let valid = project(TEST_USER, 0, &draft).unwrap();
    for case in 0..15 {
        let mut records = valid.clone();
        let expected = match case {
            0 => {
                records.pop();
                Error::CountMismatch
            }
            1 => {
                records.push(records[0].clone());
                Error::CountMismatch
            }
            2 => {
                records[1] = records[0].clone();
                Error::DuplicateRecord
            }
            3..=7 => {
                let TaxDataKey::ReturnRecord { user, tax_year, copy, kind, id } = &mut records[0].0
                else {
                    unreachable!()
                };
                match case {
                    3 => {
                        *user += 1;
                        Error::WrongNamespace
                    }
                    4 => {
                        *tax_year += 1;
                        Error::WrongNamespace
                    }
                    5 => {
                        *copy += 1;
                        Error::WrongNamespace
                    }
                    6 => {
                        *kind = 1;
                        Error::UnexpectedRecord
                    }
                    _ => {
                        *id = 0;
                        Error::InvalidIdentity
                    }
                }
            }
            8 => {
                records[0].0 = TaxDataKey::Draft { user: TEST_USER };
                Error::UnexpectedRecord
            }
            9 => {
                records[0].1 = TaxAppData::StorageFormat { format: STORAGE_FORMAT };
                Error::UnexpectedRecord
            }
            _ => {
                let TaxAppData::IncomeDocument { position, entry } = &mut records[0].1 else {
                    unreachable!()
                };
                match case {
                    10 => {
                        entry.id += 100;
                        Error::InvalidIdentity
                    }
                    11 => {
                        *position = -1;
                        Error::InvalidOrder
                    }
                    12 => {
                        *position = 1;
                        Error::InvalidOrder
                    }
                    13 => {
                        *position = 2;
                        Error::InvalidOrder
                    }
                    _ => {
                        *position = i64::MAX;
                        Error::InvalidOrder
                    }
                }
            }
        };
        let mut target = draft.clone();
        assert_eq!(assemble(TEST_USER, 0, &mut target, 2, records), Err(expected), "case {case}");
        assert_eq!(target, draft, "case {case}");
    }
    for (user, copy) in [(0, 0), (-1, 0), (TEST_USER, -1)] {
        assert_eq!(project(user, copy, &draft), Err(Error::InvalidIdentity));
        let mut target = draft.clone();
        assert_eq!(
            assemble(user, copy, &mut target, 2, valid.clone()),
            Err(Error::InvalidIdentity)
        );
        assert_eq!(target, draft);
    }
    draft.income[1].id = draft.income[0].id;
    assert_eq!(project(TEST_USER, 0, &draft), Err(Error::InvalidIdentity));
    draft.income[1].id = 0;
    assert_eq!(project(TEST_USER, 0, &draft), Err(Error::InvalidIdentity));
}
