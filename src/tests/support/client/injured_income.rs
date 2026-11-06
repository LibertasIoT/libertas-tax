pub(in crate::tests) mod additional_medicare_contributors {
    use crate::tests::support::assert::injured_income::additional_medicare_contributors::money;
    use crate::tests::support::builders::additional_medicare::Source;
    use crate::tests::support::client::injured_income_source_roundtrip;
    use crate::tests::*;
    pub(in crate::tests) fn persisted(d: FederalDraft, source: Source) -> FederalDraft {
        let d = known_fact_partnership_draft(d);
        let before = money(&d, source);
        let original = d.income.clone();
        let draft = injured_income_source_roundtrip(d, source.topic());
        assert_eq!(draft.income, original);
        assert_eq!(money(&draft, source).return_lines, before.return_lines);
        draft
    }
}

pub(in crate::tests) mod estate_estimated_payment_sources {
    use crate::tests::support::assert::injured_income::estate_estimated_payment_sources::financial;
    use crate::tests::*;
    pub(in crate::tests) fn persist(d: FederalDraft, dated: bool) -> FederalDraft {
        use crate::tests::cross::interview::accepted;
        let mut session = Interview::new(TEST_USER, d);
        crate::tests::support::client::admit_native_fixture_facts(&mut session);
        let request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        accepted(&send(&mut session, roundtrip_native_request(request)));
        financial(&session.draft, dated);
        let stored = native_draft_records(&session.draft);
        let draft = draft_from_native_records(session.draft.tax_year, &stored);
        assert_eq!(native_draft_records(&draft), stored);
        assert_eq!(draft, session.draft);
        let mut reopened = restart(draft);
        let request =
            crate::tests::support::client::income_pages::open_document_center(&mut reopened);
        let before = reopened.draft.clone();
        accepted(&send(&mut reopened, request));
        assert_eq!(reopened.draft, before);
        financial(&reopened.draft, dated);
        reopened.draft
    }
}

pub(in crate::tests) mod home_employee_niit_sources {
    use crate::tests::support::assert::payroll::injured_home_employee_money as money;
    use crate::tests::support::builders::home_employee::Source;
    use crate::tests::support::client::injured_income_source_roundtrip;
    use crate::tests::*;
    pub(in crate::tests) fn persisted(
        d: FederalDraft,
        source: Source,
        owner: Owner,
    ) -> FederalDraft {
        let before = money(&d, source, owner);
        let entry = d
            .income
            .iter()
            .find(|e| {
                matches!(
                    e.income,
                    FederalIncome::HomeSale { .. } | FederalIncome::EmployeeWages { .. }
                )
            })
            .unwrap()
            .clone();
        let draft = injured_income_source_roundtrip(d, source.topic());
        assert_eq!(draft.income.iter().find(|e| e.id == entry.id), Some(&entry));
        assert_eq!(money(&draft, source, owner).return_lines, before.return_lines);
        draft
    }
}

pub(in crate::tests) mod ltc_cent_sources {
    use crate::tests::cross::interview::{accepted, open_topic};
    use crate::tests::support::assert::injured_income::ltc_cent_sources::ordinary;
    use crate::tests::*;
    pub(in crate::tests) fn saved(draft: FederalDraft) -> (FederalDraft, FederalResult) {
        let mut session = Interview::new(TEST_USER, draft);
        super::super::income::admit_native_fixture_facts(&mut session);
        for _ in 0..2 {
            let page = super::super::income_pages::open_document_center(&mut session);
            accepted(&send(&mut session, page));
            let page = open_topic(&mut session, TaxSection::OtherIncome);
            let P::SaveIncomes { documents, .. } = &page else { unreachable!() };
            if !documents.documents.is_empty() {
                accepted(&send(&mut session, page));
            }
            let result = supported(&session.draft);
            ordinary(&result);
            session = restart(session.draft);
            assert_eq!(estimate(&session.draft), result);
        }
        let result = supported(&session.draft);
        let mut plain = session.draft.clone();
        plain.injured_spouse = None;
        let native = supported(&plain);
        assert_eq!((native.tax, native.refund, native.balance), (Some(0), Some(10_000), Some(0)));
        assert_eq!(value(&native, "agi"), 30_001);
        (session.draft, result)
    }
}

pub(in crate::tests) mod medicare_withholding_sources {
    use crate::tests::support::assert::injured_income::medicare_withholding_sources::money;
    use crate::tests::support::builders::medicare_withholding::Source;
    use crate::tests::support::client::injured_income_source_roundtrip;
    use crate::tests::*;
    pub(in crate::tests) fn persist(d: FederalDraft, source: Source) -> FederalDraft {
        let before = money(&d, source);
        let original = d.income.clone();
        let draft = injured_income_source_roundtrip(d, TaxSection::Wages);
        assert_eq!(draft.income, original);
        assert_eq!(money(&draft, source).return_lines, before.return_lines);
        draft
    }
}

pub(in crate::tests) mod native_payroll_other_tax_sources {
    use crate::tests::support::assert::injured_income::native_payroll_other_tax_sources::money;
    use crate::tests::support::builders::payroll_taxes::Source;
    use crate::tests::support::client::injured_income_source_roundtrip;
    use crate::tests::*;
    pub(in crate::tests) fn persist(d: FederalDraft, source: Source, owner: Owner) -> FederalDraft {
        let before = money(&d, source, owner);
        let original = d.income.clone();
        let draft = injured_income_source_roundtrip(d, source.topic());
        assert_eq!(draft.income, original);
        assert_eq!(money(&draft, source, owner).return_lines, before.return_lines);
        draft
    }
}

pub(in crate::tests) mod niit_relief_completeness {
    use crate::tests::cross::interview::{accepted, open_topic};
    use crate::tests::support::builders::injured_income::niit_relief_completeness::source;
    use crate::tests::*;
    pub(in crate::tests) fn accepted_case(
        case: usize,
        owner: Owner,
        injured: Owner,
    ) -> (FederalDraft, i64) {
        let mut claim = allocation();
        claim.injured = injured;
        let mut draft = injured_return((200_000, 100_000), (50_000, 0), 0, claim);
        add(&mut draft, income(savings_income(5000, 0)));
        let mut interest = income(savings_income(15_000, 0));
        interest.owner = crate::DocumentOwner::Spouses;
        add(&mut draft, interest);
        let (kind, gross, topic) = source(case);
        let mut entry = income(kind);
        // These controls have no independently entered document name. A native
        // editor clears the test helper's generic synthetic label on such records.
        entry.label.clear();
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut draft, entry);
        let before = draft.clone();
        let mut session = Interview::new(TEST_USER, draft);
        super::super::income::admit_native_fixture_facts(&mut session);
        let request = super::super::income_pages::open_document_center(&mut session);
        let bytes = request.to_avro();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        accepted(&send(&mut session, request));
        let record_page = open_topic(&mut session, topic);
        let P::SaveIncomes { documents, .. } = &record_page else { unreachable!() };
        if !documents.documents.is_empty() {
            accepted(&send(&mut session, record_page));
        }
        assert_eq!(session.draft.income, before.income);
        let bytes = session.draft.to_avro();
        let persisted = FederalDraft::from_avro(&bytes).unwrap();
        assert_eq!(persisted.to_avro(), bytes);
        let mut restarted = restart(persisted);
        let request = super::super::income_pages::open_document_center(&mut restarted);
        accepted(&send(&mut restarted, request));
        let record_page = open_topic(&mut restarted, topic);
        let P::SaveIncomes { documents, .. } = &record_page else { unreachable!() };
        if !documents.documents.is_empty() {
            accepted(&send(&mut restarted, record_page));
        }
        (restarted.draft, gross)
    }
}

pub(in crate::tests) mod niit_supporting_sources {
    use crate::tests::support::assert::injured_income::niit_supporting_sources::money;
    use crate::tests::support::builders::injured_income::niit_supporting_sources::Supporting;
    use crate::tests::support::client::injured_income_source_roundtrip;
    use crate::tests::*;
    pub(in crate::tests) fn persisted(d: FederalDraft, kind: Supporting) -> FederalDraft {
        let before = money(&d, kind);
        let source = d
            .income
            .iter()
            .find(|e| {
                matches!(
                    e.income,
                    FederalIncome::DisasterMitigation { .. }
                        | FederalIncome::TrumpAbleRollover { .. }
                        | FederalIncome::PropertyAcquisition { .. }
                )
            })
            .unwrap()
            .clone();
        let draft = injured_income_source_roundtrip(d, kind.topic());
        assert_eq!(draft.income.iter().find(|e| e.id == source.id), Some(&source));
        assert_eq!(money(&draft, kind).return_lines, before.return_lines);
        draft
    }
}
