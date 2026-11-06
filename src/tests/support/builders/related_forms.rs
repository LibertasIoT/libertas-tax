use crate::related_forms::{Activity, CenterDocument, DocumentKind};
use crate::tests::support::builders::native_income::document;
use crate::tests::support::client::income_pages::save_documents;
use crate::{RelatedFormSummary, RelatedForms};

#[derive(Default)]
pub(in crate::tests) struct RelatedBusiness(pub RelatedForms);

impl Activity for RelatedBusiness {
    const KINDS: &'static [DocumentKind] = crate::related_forms::BUSINESS_KINDS;
    fn related(&self) -> &RelatedForms {
        &self.0
    }
    fn related_mut(&mut self) -> &mut RelatedForms {
        &mut self.0
    }
}

pub(in crate::tests) fn related_document(id: i64, kind: DocumentKind) -> CenterDocument {
    CenterDocument {
        id,
        form_id: id,
        part: crate::common::test_label("Synthetic part"),
        eligible: alloc::vec![8, 9],
        kind,
        summary: RelatedFormSummary {
            label: crate::common::test_label("Synthetic payer form"),
            payer: "Synthetic payer".into(),
            amount: Some(12_345),
        },
    }
}

pub(in crate::tests) fn related_component_editor()
-> (crate::interview::Interview, crate::TaxInterviewProtocol) {
    component_editor(crate::related_forms::ActivityKind::Farm)
}

pub(in crate::tests) fn related_activity_editor(
    kind: crate::related_forms::ActivityKind,
) -> (crate::interview::Interview, i64) {
    use crate::related_forms::ActivityKind;
    use crate::tests::*;
    let (mut session, request) = component_editor(kind);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    let activity = session
        .draft
        .income
        .iter()
        .find(|entry| match (&entry.income, kind) {
            (FederalIncome::Business { data }, ActivityKind::Business) => {
                matches!(data.activity, BusinessActivity::Ordinary { .. })
            }
            (FederalIncome::Business { data }, ActivityKind::Farm) => {
                matches!(data.activity, BusinessActivity::Farming { .. })
            }
            (FederalIncome::FarmRental { .. }, ActivityKind::FarmRental) => true,
            (FederalIncome::Rental { .. }, ActivityKind::Rental)
            | (FederalIncome::HobbySales { .. }, ActivityKind::Hobby)
            | (FederalIncome::Royalty { .. }, ActivityKind::Royalty)
            | (
                FederalIncome::NonbusinessProperty {
                    kind: NonbusinessPropertyKind::PersonalRental { .. },
                    ..
                },
                ActivityKind::Equipment,
            ) => true,
            _ => false,
        })
        .expect("accepted activity of the requested kind")
        .id;
    (session, activity)
}

// The actual Farm editor represents zero receipts by an empty list. Build the
// shared fixture in that canonical representation: a farm that can receive the
// farm papers of a fixture (a single farm receives them by default).
pub(in crate::tests) fn receiving_farm() -> crate::FederalIncome {
    use crate::tests::*;
    let mut farm = raised_farm(0, 0);
    let FederalIncome::Business { data } = &mut farm else { unreachable!() };
    let BusinessActivity::Farming { data: facts } = &mut data.activity else { unreachable!() };
    facts.accounting = FarmAccounting::Cash { receipts: vec![] };
    farm
}

fn component_editor(
    kind: crate::related_forms::ActivityKind,
) -> (crate::interview::Interview, crate::TaxInterviewProtocol) {
    use crate::tests::*;
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(cash_business(0)));
    add(
        &mut draft,
        income(if kind == crate::related_forms::ActivityKind::FarmRental {
            FederalIncome::FarmRental { data: farm_rental_data(0, 0) }
        } else {
            receiving_farm()
        }),
    );
    let additional = match kind {
        crate::related_forms::ActivityKind::Rental => {
            Some(FederalIncome::Rental { data: rental(0, 0) })
        }
        crate::related_forms::ActivityKind::Hobby => Some(hobby_sales(0, 0, 0)),
        crate::related_forms::ActivityKind::Royalty => Some(royalty(0, 0)),
        crate::related_forms::ActivityKind::Equipment => Some(personal_rent(0, 0)),
        _ => None,
    };
    if let Some(activity) = additional {
        add(&mut draft, income(activity));
    }
    let mut session = Interview::new(TEST_USER, draft);
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let rows = vec![
        document(
            0,
            IncomeSource::SelfEmployment {
                value: SelfEmploymentDocument::Form1099K(k_document(Form1099K {
                    payer: Default::default(),
                    recipient: Default::default(),
                    cash_tips: 0,
                    tip_occupation_codes: Default::default(),
                    gross: 40_000,
                    withholding: 0,
                    parts: vec![
                        KPart { id: 0, amount: 15_000, belongs: None, refunded: 0 },
                        KPart { id: 0, amount: 25_000, belongs: None, refunded: 0 },
                    ],
                    state_details: Default::default(),
                })),
            },
        ),
        document(
            0,
            IncomeSource::OtherIncome {
                value: OtherIncomeDocument::Form1099Misc(misc_document(Form1099Misc {
                    payer: crate::PrintedParty::from_name("Fable Land Tenant".into()),
                    recipient: Default::default(),
                    cash_tips: 0,
                    tip_occupation_codes: Default::default(),
                    overtime: 0,
                    component_ids: Default::default(),
                    crop_insurance: 0,
                    rents: 12_000,
                    royalties: 23_000,
                    other: 0,
                    other_use: None,
                    withholding: 0,
                    fishing: 0,
                    medical: 0,
                    substitute: 0,
                    attorney: 0,
                    client_funds: 0,
                    fish_sales: 0,
                    deferred: 0,
                    state_details: Default::default(),
                })),
            },
        ),
        document(
            0,
            IncomeSource::GovernmentPayment {
                refund_state: None,
                refund_locality: Default::default(),
                value: government_document(Form1099G {
                    taxable_grants: 34_000,
                    agriculture_payment: 45_000,
                    ..Default::default()
                }),
            },
        ),
    ];
    let request = save_documents(&mut session, rows);
    (session, request)
}

pub(in crate::tests) fn related_receipt_draft() -> crate::FederalDraft {
    use crate::tests::*;
    let (mut session, mut request) = related_component_editor();
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let OtherIncomeDocument::Form1099Misc(form) =
        &mut documents.other.other_income.documents[0].document
    else {
        unreachable!()
    };
    form.crop_insurance = Some(50_000);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        request,
    ));
    add(&mut session.draft, income(cooperative_routed_income(80_000, 0)));
    session.draft
}

pub(in crate::tests) fn other_related_return() -> crate::FederalDraft {
    use crate::tests::*;
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(hobby_sales(1_000, 0, 0)));
    add(&mut draft, income(hobby_sales(2_000, 0, 0)));
    add(&mut draft, income(personal_rent(1_500, 0)));
    add(&mut draft, income(royalty(2_000, 0)));
    let form = crate::FederalForm1099Misc {
        cash_tips: 0,
        tip_occupation_codes: Default::default(),
        overtime: 0,
        component_ids: Default::default(),
        crop_insurance: 0,
        rents: 100_000,
        royalties: 300_000,
        other: 0,
        other_use: None,
        withholding: 0,
        fishing: 0,
        medical: 0,
        substitute: 0,
        attorney: 0,
        client_funds: 0,
        fish_sales: 0,
        deferred: 0,
    };
    add(&mut draft, income(FederalIncome::Misc1099 { data: misc_source(form) }));
    add(
        &mut draft,
        income(FederalIncome::K1099 {
            data: k_source(FederalForm1099K {
                cash_tips: 0,
                tip_occupation_codes: Default::default(),
                gross: 40_000,
                withholding: 0,
                parts: vec![FederalKPart { id: 0, amount: 40_000, belongs: None, refunded: 5_000 }],
            }),
        }),
    );
    draft
}

pub(in crate::tests) fn related_keys(keys: &[i64]) -> crate::RelatedForms {
    crate::RelatedForms {
        candidates: alloc::vec::Vec::new(),
        forms: keys.iter().map(|&key| crate::RelatedFormView { key, summary: None }).collect(),
    }
}

pub(in crate::tests) fn rental_mortgage_context_fixture(
    explicit: bool,
) -> (crate::FederalDraft, i64) {
    use crate::tests::*;
    let mut draft = complete(FederalStatus::Single);
    let mut first = rental(0, 0);
    first.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 60,
        overlap_days: 0,
        direct_expenses: 0,
    };
    first.expenses[0].category = RentalExpenseCategory::MortgageInterest;
    first.expenses[0].amount = 200_000;
    link_home_fixture(&mut draft, &mut first);
    if !explicit {
        first.related.clear();
    }
    let mut second = first.clone();
    second.related.clear();
    add(&mut draft, income(FederalIncome::Rental { data: first }));
    let first_id = draft.income.last().unwrap().id;
    add(&mut draft, income(FederalIncome::Rental { data: second }));
    (draft, first_id)
}

pub(in crate::tests) fn remove_related_component_request(
    session: &mut crate::interview::Interview,
    key: i64,
) -> (crate::TaxInterviewProtocol, i64) {
    use crate::tests::*;
    let source = crate::related_forms::catalog(&session.draft)
        .into_iter()
        .find(|source| source.id == key)
        .expect("accepted component");
    let mut request = crate::tests::support::client::income_pages::open_document_center(session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    match source.kind {
        DocumentKind::Form1099KPart => {
            let row = documents
                .common
                .self_employment
                .documents
                .iter_mut()
                .find(|row| row.id == source.form_id)
                .unwrap();
            let FederalIncome::K1099 { data } = &session
                .draft
                .income
                .iter()
                .find(|entry| entry.id == source.form_id)
                .unwrap()
                .income
            else {
                unreachable!()
            };
            let mut updated = data.clone();
            let part = updated.parts.iter().position(|part| part.id == key).unwrap();
            updated.form.gross =
                Some(updated.form.gross.unwrap_or(0) - updated.parts.remove(part).amount);
            row.document = SelfEmploymentDocument::Form1099K(k_document(updated));
        }
        DocumentKind::Form1099MiscRents | DocumentKind::Form1099MiscRoyalties => {
            let row = documents
                .other
                .other_income
                .documents
                .iter_mut()
                .find(|row| row.id == source.form_id)
                .unwrap();
            let OtherIncomeDocument::Form1099Misc(form) = &mut row.document else { unreachable!() };
            if source.kind == DocumentKind::Form1099MiscRents {
                form.rents = Some(0);
            } else {
                form.royalties = Some(0);
            }
        }
        DocumentKind::Form1099GGrant | DocumentKind::Form1099GAgriculture => {
            let row = documents
                .common
                .government_payments
                .documents
                .iter_mut()
                .find(|row| row.id == source.form_id)
                .unwrap();
            let form = &mut row.document;
            if source.kind == DocumentKind::Form1099GGrant {
                form.taxable_grants = Some(0);
            } else {
                form.agriculture_payments = Some(0);
            }
        }
        _ => panic!("fixture component must leave another amount on its parent form"),
    }
    (request, source.form_id)
}
