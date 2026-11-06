use super::*;
use crate::tests::support::client::income_pages::open_documents;
use alloc::string::ToString;

fn one_month(premium: i64, benchmark: i64, advance: i64) -> MarketplaceDocumentView {
    let mut form = all_year("Actual January source", premium, benchmark, advance);
    form.value.set_amounts(PolicyAmounts::ByMonth {
        months: vec![PolicyMonth { month: 1, premium, benchmark: Some(benchmark), advance }],
    });
    form.coverage.retain(|month| month.month == 1);
    form
}
fn single() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3_130_000, 0);
    name_accepted_income_documents(&mut d);
    d
}
fn reopen(s: &mut Interview, value: MarketplaceFactsPage) -> MarketplaceFactsPage {
    let expected = value.clone();
    let mut saved = save_marketplace_and_reopen(s, value);
    let topic = open_credits_topic(s, TaxSection::Marketplace);
    saved.forms = marketplace_page_of(&topic).forms.clone();
    // Only newly created accepted identities may change on Save.
    let mut normalized = saved.clone();
    for (row, expected) in normalized.forms.iter_mut().zip(&expected.forms) {
        row.id = expected.id;
    }
    for (row, expected) in normalized.qsehra.iter_mut().zip(&expected.qsehra) {
        row.id = expected.id;
    }
    for (row, expected) in normalized.other_families.iter_mut().zip(&expected.other_families) {
        row.id = expected.id;
    }
    for (row, expected) in normalized.covered.iter_mut().zip(&expected.covered) {
        row.id = expected.id;
    }
    assert_eq!(normalized, expected);
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    saved
}
fn area(people: Vec<i64>, amount: i64) -> MarketplaceBenchmarkMonthPage {
    MarketplaceBenchmarkMonthPage {
        month: 1,
        allocated: vec![],
        groups: vec![MarketplaceBenchmarkGroupPage {
            state: TaxState::CA,
            rating_area: "Actual eligible-family area".into(),
            people,
            source: MarketplaceBenchmarkSourcePage::Marketplace { amount },
        }],
    }
}
fn marketplace_request(s: &mut Interview, mut value: MarketplaceFactsPage) -> P {
    let topic_forms = core::mem::take(&mut value.forms);
    let forms: Vec<_> = topic_forms
        .iter()
        .map(|form| crate::Form1095ADocument { id: form.id, value: form.value.clone() })
        .collect();
    let mut source_page = open_documents(s);
    let P::SaveDocuments { documents, .. } = &mut source_page else { unreachable!() };
    if documents.common.marketplace.documents != forms {
        let topic = open_credits_topic(s, TaxSection::Marketplace);
        let mut accepted_facts = marketplace_page_of(&topic).clone();
        accepted_facts.forms.clear();
        assert_eq!(
            value, accepted_facts,
            "a source refusal must not also submit edited topic facts"
        );
        documents.common.marketplace.documents = forms;
        source_page
    } else {
        value.forms = topic_forms;
        save_marketplace(s, value)
    }
}
fn invalid(s: &mut Interview, value: MarketplaceFactsPage) {
    let before = s.draft.clone();
    let request = marketplace_request(s, value);
    error(send(s, request));
    assert_eq!(s.draft, before);
}

fn capture_marketplace(name: &str, session: &Interview, value: &MarketplaceFactsPage) {
    extern crate std;
    if let Ok(path) = std::env::var(name) {
        // Capture a genuinely delivered editor: its hidden picker catalogs
        // belong to the server and are never synthesized by the client probe.
        let mut reopened = restart(session.draft.clone());
        let facts = value.clone();
        let forms = facts
            .forms
            .iter()
            .map(|form| crate::Form1095ADocument { id: form.id, value: form.value.clone() })
            .collect();
        let source = save_marketplace_sources(&mut reopened, forms);
        accepted(&send(&mut reopened, source));
        let page = save_marketplace(&mut reopened, facts);
        std::fs::write(path, page.to_avro()).unwrap();
    }
}

#[test]
fn unit46_g8962_2_qsehra_actual_sources_save_reopen_and_affordability_controls() {
    // 2026 $31,300 gives $259.79/month affordability; $600-$200=$400
    // is unaffordable. January PTC $900-$172=$728, less actual$500=$228.
    let (mut s, _) = at_the_marketplace_page(single());
    let mut value = marketplace_fixture_page(vec![one_month(100_000, 90_000, 30_000)]);
    value.qsehra.push(MarketplaceQsehraPage {
        id: 0,
        employee: -1,
        source: MarketplaceQsehraSourcePage::Notice {
            employer: "Actual employer".into(),
            annual_permitted: 50_000,
        },
        months: vec![MarketplaceQsehraMonthPage {
            month: 1,
            self_only: 20_000,
            actual: 50_000,
            self_benchmark: 60_000,
            covered: vec![-1],
        }],
        determinations: vec![],
    });
    let mut saved = reopen(&mut s, value);
    assert!(saved.qsehra[0].id > 0);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 228);
    let mut wrong = saved.clone();
    wrong.qsehra[0].months[0].actual += 1;
    invalid(&mut s, wrong); // The notice's actual annual source must reconcile.
    saved.qsehra[0].months[0].self_only = 40_000;
    reopen(&mut s, saved);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 0);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_repayment"), 300);
}

#[test]
fn unit46_g8962_2_dependent_notice_identity_saves_reopens_without_inferred_family_context() {
    let mut d = single();
    add_child(&mut d);
    let employee = d.dependents[0].id;
    let (mut s, _) = at_the_marketplace_page(d);
    // The employee need not be personally enrolled for the claiming taxpayer
    // to have a coverage month. The notice is still that employee's source,
    // not a filer-owned W-2 and not proof of parent/sibling relationships.
    let mut value = marketplace_fixture_page(vec![one_month(100_000, 90_000, 30_000)]);
    value.dependent_qsehra_employees = vec![employee];
    value.qsehra.push(MarketplaceQsehraPage {
        id: 0,
        employee,
        source: MarketplaceQsehraSourcePage::Notice {
            employer: "Dependent employee's actual employer".into(),
            annual_permitted: 50_000,
        },
        months: vec![MarketplaceQsehraMonthPage {
            month: 1,
            self_only: 20_000,
            actual: 50_000,
            self_benchmark: 60_000,
            covered: vec![employee],
        }],
        determinations: vec![],
    });
    let saved = reopen(&mut s, value);
    assert!(saved.qsehra[0].id > 0);
    assert_eq!(saved.qsehra[0].employee, employee);
    capture_marketplace("TAX_UNIT46_G2_DEPENDENT", &s, &saved);
    let result = estimate(&s.draft);
    assert_ne!(result.state, ResultState::EstimatedForSupportedScope);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| { issue.display.key == "PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED" })
    );
    // The accepted actual notice identity survives restart while its missing
    // employee income/relationship context remains a precise source boundary.
    let before = s.draft.clone();
    for (mut bad, key) in [
        (saved.clone(), "PTC_QSEHRA_BENEFIT_MISMATCH"),
        (saved.clone(), "PTC_DEPENDENT_QSEHRA_NOTICE_REQUIRED"),
    ] {
        match key {
            "PTC_QSEHRA_BENEFIT_MISMATCH" => bad.qsehra[0].months[0].actual += 1,
            "PTC_DEPENDENT_QSEHRA_NOTICE_REQUIRED" => {
                bad.qsehra[0].source = MarketplaceQsehraSourcePage::W2 { document: 1 };
            }
            _ => unreachable!(),
        }
        rejected_with(
            {
                let request = marketplace_request(&mut s, bad);
                send(&mut s, request)
            },
            key,
        );
        assert_eq!(s.draft, before);
    }
    // Plan-covered metadata is not the employee's tax-family relationship
    // set and does not require personal Marketplace enrollment.
    let mut metadata = saved.clone();
    metadata.qsehra[0].months[0].covered = vec![-1];
    let metadata = reopen(&mut s, metadata);
    assert_eq!(metadata.qsehra[0].employee, employee);
    let mut forged = saved;
    forged.qsehra[0].employee = employee + 100;
    let before = s.draft.clone();
    let request = marketplace_request(&mut s, forged);
    error(current_source_refusal(
        "health-context-18",
        &mut s,
        request,
        "MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A",
        Some("[value/qsehra/0/employee]"),
        SourceScope::Health,
    ));
    assert_eq!(s.draft, before);
}

#[test]
fn unit46_g8962_3_lawful_family_cost_saves_reopens_and_never_guesses_missing_cost() {
    let mut d = single();
    add_child(&mut d);
    let child = d.dependents[0].id;
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![one_month(100_000, 120_000, 60_000)]);
    value.forms[0].coverage[0].people.push(child);
    value.eligibility.push(MarketplaceEligibilityMonthPage {
        month: 1,
        person: child,
        status: MarketplacePersonEligibility::NotLawfullyPresent,
    });
    value.forms[0].lawful_premiums.push(MarketplaceLawfulPremiumMonthPage {
        month: 1,
        people: vec![-1],
        source: MarketplaceLawfulPremiumSource::Marketplace { amount: 80_000 },
        paid: None,
    });
    value.benchmarks.push(area(vec![-1], 90_000));
    let mut saved = reopen(&mut s, value);
    // The actual two-person tax family remains; annual contribution$1,252,
    // monthly$104. Eligible credit min($800,$900-$104)=$796.
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 796);
    saved.forms[0].lawful_premiums.clear();
    reopen(&mut s, saved);
    unsupported(&s.draft, "Marketplace source");
}

fn external_people(s: &mut Interview, labels: &[&str]) -> MarketplaceFactsPage {
    let value = MarketplaceFactsPage {
        other_families: labels
            .iter()
            .map(|label| MarketplaceFamilyPage { id: 0, label: (*label).into() })
            .collect(),
        ..MarketplaceFactsPage::default()
    };
    let mut value = reopen(s, value);
    value.covered = value
        .other_families
        .iter()
        .map(|family| MarketplaceCoveredPersonPage {
            id: 0,
            identity: MarketplacePersonIdentityPage::OtherFamily {
                family: family.id,
                label: family.label.clone(),
            },
        })
        .collect();
    reopen(s, value)
}
fn agreed(
    family: MarketplaceFamilyRefPage,
    numerator: u32,
    denominator: u32,
) -> MarketplaceFamilySharePage {
    MarketplaceFamilySharePage {
        family,
        agreement: MarketplaceAgreement::Agreed {
            share: MarketplaceFraction { numerator, denominator },
        },
    }
}

#[test]
fn unit46_g8962_4_three_family_allocation_saves_reopens_and_rejects_overallocation() {
    // Pub974 example1 isolated into one source period: $8,000/$9,000/
    // $4,500 allocated25/40/35; this return owns$2,000/$2,250/$1,125.
    let (mut s, _) = at_the_marketplace_page(single());
    let mut value = external_people(&mut s, &["Sharon", "Erik"]);
    let ids: Vec<_> = value.covered.iter().map(|person| person.id).collect();
    let mut policy = one_month(800_000, 900_000, 450_000);
    policy.coverage[0].people.extend(&ids);
    policy.allocation.push(MarketplaceAllocationPeriodPage {
        from: 1,
        to: 1,
        kind: MarketplaceAllocationKindPage::OtherShared {
            families: vec![
                agreed(MarketplaceFamilyRefPage::ThisReturn, 25, 100),
                agreed(
                    MarketplaceFamilyRefPage::Other { family: value.other_families[0].id },
                    40,
                    100,
                ),
                agreed(
                    MarketplaceFamilyRefPage::Other { family: value.other_families[1].id },
                    35,
                    100,
                ),
            ],
        },
    });
    value.forms.push(policy);
    value.eligibility = [-1]
        .into_iter()
        .chain(ids)
        .map(|person| MarketplaceEligibilityMonthPage {
            month: 1,
            person,
            status: MarketplacePersonEligibility::Eligible,
        })
        .collect();
    let saved = reopen(&mut s, value);
    capture_marketplace("TAX_UNIT46_G4_VALID", &s, &saved);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 2000);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_advance"), 1125);
    let mut wrong = saved.clone();
    let MarketplaceAllocationKindPage::OtherShared { families } =
        &mut wrong.forms[0].allocation[0].kind
    else {
        unreachable!()
    };
    families[0].agreement = MarketplaceAgreement::Agreed {
        share: MarketplaceFraction { numerator: 26, denominator: 100 },
    };
    capture_marketplace("TAX_UNIT46_G4_OVER", &s, &wrong);
    invalid(&mut s, wrong);
    let mut too_small = saved.clone();
    let MarketplaceAllocationKindPage::OtherShared { families } =
        &mut too_small.forms[0].allocation[0].kind
    else {
        unreachable!()
    };
    families[0].agreement = MarketplaceAgreement::Agreed {
        share: MarketplaceFraction { numerator: 24, denominator: 100 },
    };
    capture_marketplace("TAX_UNIT46_G4_UNDER", &s, &too_small);
    invalid(&mut s, too_small); // Explicit enrolling family cannot contradict its residual.
    let mut omitted = saved.clone();
    let MarketplaceAllocationKindPage::OtherShared { families } =
        &mut omitted.forms[0].allocation[0].kind
    else {
        unreachable!()
    };
    families.remove(0);
    reopen(&mut s, omitted);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_advance"), 1125);
    let mut defaults = saved.clone();
    // Pub974 example2's actual enroller is Erik, not Bill (this return).
    defaults.forms[0].enrolling =
        MarketplaceFamilyRefPage::Other { family: defaults.other_families[1].id };
    let MarketplaceAllocationKindPage::OtherShared { families } =
        &mut defaults.forms[0].allocation[0].kind
    else {
        unreachable!()
    };
    for row in families {
        row.agreement = MarketplaceAgreement::NoAgreement;
    }
    let mut defaults = reopen(&mut s, defaults);
    let all_default = supported(&s.draft);
    assert_eq!(value_of(&all_default, "premium_credit_advance"), 1485);
    {
        extern crate std;
        let facts = stored_marketplace(&s);
        let policy = &facts.forms[0];
        let counts = [
            (MarketplaceFamilyRef::ThisReturn, 1),
            (MarketplaceFamilyRef::Other { family: facts.other_families[0].id }, 1),
            (MarketplaceFamilyRef::Other { family: facts.other_families[1].id }, 1),
        ];
        let rows = crate::marketplace_allocation::allocate(
            &policy.allocation[0].kind,
            &policy.enrolling,
            &counts,
            |_| None,
        )
        .unwrap();
        for (family, percent, premium, benchmark, advance) in [
            (MarketplaceFamilyRef::ThisReturn, 33, 264_000, 297_000, 148_500),
            (
                MarketplaceFamilyRef::Other { family: facts.other_families[0].id },
                33,
                264_000,
                297_000,
                148_500,
            ),
            (policy.enrolling.clone(), 34, 272_000, 306_000, 153_000),
        ] {
            let row = rows.iter().find(|row| row.family == family).unwrap();
            assert_eq!(row.premium, crate::kernel::ratio::exact::ratio(percent, 100).unwrap());
            assert_eq!(
                crate::kernel::ratio::exact::whole(800_000) * &row.premium,
                crate::kernel::ratio::exact::whole(premium)
            );
            assert_eq!(
                crate::kernel::ratio::exact::whole(450_000) * &row.advance,
                crate::kernel::ratio::exact::whole(advance)
            );
            assert!(matches!(&row.benchmark,
                    crate::marketplace_allocation::AllocatedBenchmark::SourceShare(share)
                    if crate::kernel::ratio::exact::whole(900_000) * share == crate::kernel::ratio::exact::whole(benchmark)));
        }
        if let Ok(path) = std::env::var("TAX_UNIT46_DEFAULT_DIAGNOSTIC") {
            let observed: Vec<_> = rows.iter().map(|row| serde_json::json!({
                "family": alloc::format!("{:?}", row.family), "fraction": row.premium.to_string(),
                "premium_source_cents": (crate::kernel::ratio::exact::whole(800_000) * &row.premium).to_string(),
                "benchmark_source_cents": (crate::kernel::ratio::exact::whole(900_000) * &row.premium).to_string(),
                "advance_source_cents": (crate::kernel::ratio::exact::whole(450_000) * &row.advance).to_string(),
            })).collect();
            std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
                "scope": "Published fixture C and actual owner match; general floor policy is dispatcher inference cbda263", "actual": observed,
                "published": {"Bill":"33/100","Sharon":"33/100","Erik":"34/100"},
                "app_current_return_advance": value_of(&all_default,"premium_credit_advance"),
                "primary": "https://www.irs.gov/publications/p974",
                "section": "Other Taxpayers Allocating Policy Amounts With Two or More Other Taxpayers, Example 2",
            })).unwrap()).unwrap();
        }
    }
    let MarketplaceAllocationKindPage::OtherShared { families } =
        &mut defaults.forms[0].allocation[0].kind
    else {
        unreachable!()
    };
    families[0].agreement = MarketplaceAgreement::Agreed {
        share: MarketplaceFraction { numerator: 99, denominator: 100 },
    };
    families[1].agreement = MarketplaceAgreement::Agreed {
        share: MarketplaceFraction { numerator: 99, denominator: 100 },
    };
    // NoAgreement in one actual family supersedes all partial agreements;
    // their 198% sum is not a selected agreed partition.
    let mut partial = reopen(&mut s, defaults);
    capture_marketplace("TAX_UNIT46_G4_PARTIAL", &s, &partial);
    assert_eq!(
        value_of(&supported(&s.draft), "premium_credit_advance"),
        value_of(&all_default, "premium_credit_advance")
    );
    let MarketplaceAllocationKindPage::OtherShared { families } =
        &mut partial.forms[0].allocation[0].kind
    else {
        unreachable!()
    };
    families[2].family = families[1].family.clone();
    invalid(&mut s, partial); // Complete source family graph still rejects duplicates.
}

#[test]
fn unit46_g8962_5_payment_exceptions_save_reopen_at_actual_coverage_and_appeal_boundaries() {
    let (mut s, _) = at_the_marketplace_page(single());
    let mut value = marketplace_fixture_page(vec![one_month(100_000, 90_000, 60_000)]);
    value.forms[0].payments.push(MarketplacePaymentMonthPage {
        month: 1,
        paid: 90_000,
        refunded: 0,
        unpaid: 10_000,
        tax_free_premiums: 0,
        tax_free_coverage: None,
        coverage: MarketplacePaymentCoverage::PaidByDeadline,
        appeal: None,
    });
    let mut value = reopen(&mut s, value);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 0);
    for (coverage, credit) in [
        (MarketplacePaymentCoverage::FirstGraceMonth, 728),
        (MarketplacePaymentCoverage::ThresholdMet { issuer_provided: true }, 728),
        (MarketplacePaymentCoverage::ThresholdMet { issuer_provided: false }, 0),
        (
            MarketplacePaymentCoverage::EmergencyOrder {
                issuer_provided: true,
                order_date: 20260101,
            },
            728,
        ),
        (
            MarketplacePaymentCoverage::EmergencyOrder {
                issuer_provided: false,
                order_date: 20260101,
            },
            0,
        ),
    ] {
        value.forms[0].payments[0].coverage = coverage;
        value = reopen(&mut s, value);
        assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), credit);
    }
    value.forms[0].payments[0].paid = 100_000;
    value.forms[0].payments[0].unpaid = 0;
    value.forms[0].payments[0].coverage = MarketplacePaymentCoverage::PaidByDeadline;
    value.forms[0].payments[0].appeal = Some(MarketplaceAppealPage {
        determined: 20260101,
        enrolled: 20260101,
        paid_on: Some(20260501),
        required_deadline: 20260501,
    });
    value = reopen(&mut s, value);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 728);
    value.forms[0].payments[0].appeal.as_mut().unwrap().paid_on = Some(20260502);
    value = reopen(&mut s, value);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 0);
    let mut wrong = value.clone();
    wrong.forms[0].payments[0].paid += 1;
    invalid(&mut s, wrong);
    value.forms[0].payments[0].appeal = None;
    value.forms[0].payments[0].coverage =
        MarketplacePaymentCoverage::NoQualifyingCoverage { issuer_returned_advance: 60_000 };
    reopen(&mut s, value);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_advance"), 0);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_repayment"), 0);
}

#[test]
fn unit46_g8962_6_mixed_mec_eligible_benchmark_saves_reopens_and_rejects_wrong_family() {
    let mut d = single();
    add_child(&mut d);
    let child = d.dependents[0].id;
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![one_month(100_000, 120_000, 60_000)]);
    value.forms[0].coverage[0].people.push(child);
    value.eligibility.push(MarketplaceEligibilityMonthPage {
        month: 1,
        person: child,
        status: MarketplacePersonEligibility::OtherMec { kind: MarketplaceMecKind::Medicare },
    });
    value.benchmarks.push(area(vec![-1], 90_000));
    let mut value = reopen(&mut s, value);
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 796);
    value.benchmarks[0].groups[0].people.push(child);
    reopen(&mut s, value);
    unsupported(&s.draft, "Marketplace source");
}

#[test]
fn unit46_g8962_7_allocated_and_unallocated_benchmark_sources_save_reopen_without_duplication() {
    let (mut s, _) = at_the_marketplace_page(single());
    let mut value = external_people(&mut s, &["Other actual family"]);
    let other = value.covered[0].id;
    let family = value.other_families[0].id;
    let mut shared = one_month(100_000, 60_000, 60_000);
    shared.coverage[0].people.push(other);
    shared.allocation.push(MarketplaceAllocationPeriodPage {
        from: 1,
        to: 1,
        kind: MarketplaceAllocationKindPage::OtherShared {
            families: vec![
                agreed(MarketplaceFamilyRefPage::ThisReturn, 1, 2),
                agreed(MarketplaceFamilyRefPage::Other { family }, 1, 2),
            ],
        },
    });
    value.forms = vec![shared, one_month(70_000, 60_000, 0), one_month(70_000, 60_000, 0)];
    value.eligibility = [-1, other]
        .into_iter()
        .map(|person| MarketplaceEligibilityMonthPage {
            month: 1,
            person,
            status: MarketplacePersonEligibility::Eligible,
        })
        .collect();
    value.benchmarks = vec![area(vec![-1], 60_000)];
    let mut value = reopen(&mut s, value);
    // Two unallocated statements repeat one actual$600 family benchmark.
    // Allocated$300+$600-$172 contribution=$728, not$1,328.
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 728);
    assert_eq!(value_of(&supported(&s.draft), "premium_credit_advance"), 300);
    value.benchmarks.clear();
    reopen(&mut s, value);
    unsupported(&s.draft, "Marketplace source");
}

#[test]
fn unit46_g8962_2_dependent_employee_income_and_person_specific_affordability_lifecycle() {
    extern crate std;
    let mut d = single();
    d.income[0] = wage(1, Owner::Taxpayer, 1_930_000, 0);
    add_child(&mut d);
    add_child(&mut d);
    let employee = d.dependents[0].id;
    let sibling = d.dependents[1].id;
    let (mut s, _) = at_the_marketplace_page(d);
    let mut value = marketplace_fixture_page(vec![one_month(100_000, 90_000, 30_000)]);
    value.forms[0].coverage[0].people.extend([employee, sibling]);
    for person in [employee, sibling] {
        value.eligibility.push(MarketplaceEligibilityMonthPage {
            month: 1,
            person,
            status: MarketplacePersonEligibility::Eligible,
        });
    }
    value.dependents.push(DependentReturn {
        person: employee,
        agi: 1_200_000,
        exempt_interest: 0,
        foreign_exclusion: 0,
        social: 0,
        taxable_social: 0,
    });
    value.dependent_qsehra_employees = vec![employee];
    value.qsehra.push(MarketplaceQsehraPage {
        id: 0,
        employee,
        source: MarketplaceQsehraSourcePage::Notice {
            employer: "Dependent employee actual employer".into(),
            annual_permitted: 10_000,
        },
        months: vec![MarketplaceQsehraMonthPage {
            month: 1,
            self_only: 10_000,
            actual: 10_000,
            self_benchmark: 20_000,
            covered: vec![employee],
        }],
        determinations: vec![],
    });
    value.employee_qsehra_facts.push(MarketplaceQsehraEmployeeFactsPage {
        employee,
        income: Some(MarketplaceQsehraIncomeSource::RequiredReturn),
        household: Some(MarketplaceQsehraEmployeeHouseholdPage::Known {
            spouse: None,
            dependents: vec![],
        }),
    });
    // Native-client probes begin after the separately accepted source editor.
    if std::env::var("TAX_UNIT46_G2_CALC_INPUT").is_ok()
        || std::env::var("TAX_UNIT46_G2_SEED").is_ok()
    {
        let forms = value
            .forms
            .iter()
            .map(|form| crate::Form1095ADocument { id: form.id, value: form.value.clone() })
            .collect();
        let source = save_marketplace_sources(&mut s, forms);
        accepted(&send(&mut s, source));
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_G2_CALC_INPUT") {
        let before = s.draft.clone();
        let request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(request, P::EstimateMarketplace { .. }));
        let Reply::Response(response) = send(&mut s, request) else {
            panic!("calculation response");
        };
        assert!(matches!(response, P::RunningEstimate { .. }));
        assert_eq!(s.draft, before);
        std::fs::write(std::env::var("TAX_UNIT46_G2_CALC_OUTPUT").unwrap(), response.to_avro())
            .unwrap();
        return;
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_G2_SEED") {
        let seed = open_credits_topic(&mut s, TaxSection::Marketplace);
        std::fs::write(path, seed.to_avro()).unwrap();
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_G2_ANSWERS") {
        // Preserve genuinely delivered hidden catalogs; fresh visible rows still have id0.
        let mut answers = value.clone();
        if std::env::var("TAX_UNIT46_G2_CASE").as_deref() == Ok("affordable") {
            answers.qsehra[0].months[0].self_benchmark = 19_960;
            answers.benchmarks.push(area(vec![-1, sibling], 70_000));
        }
        capture_marketplace("TAX_UNIT46_G2_ANSWERS", &s, &answers);
        assert!(std::path::Path::new(&path).exists());
    }
    if let Ok(path) = std::env::var("TAX_UNIT46_G2_PAYLOAD") {
        let forms = value
            .forms
            .iter()
            .map(|form| crate::Form1095ADocument { id: form.id, value: form.value.clone() })
            .collect();
        let source = save_marketplace_sources(&mut s, forms);
        accepted(&send(&mut s, source));
        let request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        credits_hub(send_past_less_common(&mut s, request));
        let reopened = open_credits_topic(&mut s, TaxSection::Marketplace);
        let saved = marketplace_page_of(&reopened);
        assert_eq!(saved.forms[0].coverage[0].people, vec![-1, employee, sibling]);
        match std::env::var("TAX_UNIT46_G2_CASE").as_deref() {
            Ok("affordable") => {
                assert_eq!(saved.qsehra[0].months[0].self_benchmark, 19_960);
                assert_eq!(saved.benchmarks[0].groups[0].people, vec![-1, sibling]);
                assert_eq!(
                    saved.benchmarks[0].groups[0].source,
                    MarketplaceBenchmarkSourcePage::Marketplace { amount: 70_000 }
                );
                assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 545);
            }
            Ok("unaffordable") | Err(_) => {
                assert_eq!(saved.qsehra[0].months[0].self_benchmark, 20_000);
                assert!(saved.benchmarks.is_empty());
                assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 745);
            }
            other => panic!("unknown G2 client fixture case {other:?}"),
        }
        assert_eq!(
            marketplace_page_of(&reopened).employee_qsehra_facts,
            value.employee_qsehra_facts
        );
        assert_eq!(restart(s.draft.clone()).draft, s.draft);
        if let Ok(path) = std::env::var("TAX_UNIT46_G2_REOPEN") {
            std::fs::write(path, reopened.to_avro()).unwrap();
        }
        return;
    }
    let mut value = reopen(&mut s, value);
    // Independent cents oracle: claimant household$31,300, family3/FPL$26,650,
    // 117% ->2.10%, annual$657, monthly$55. Employee own$12,000 gives
    // $99.60 affordability threshold. Required$100 is unaffordable, although
    // substituting claimant income would wrongly call it affordable.
    assert_eq!(value_of(&supported(&s.draft), "marketplace_household_income"), 31_300);
    assert_eq!(
        crate::marketplace_qsehra::employee_income(stored_marketplace(&s), employee),
        Some(crate::kernel::ratio::exact::whole(1_200_000))
    );
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 745);
    capture_marketplace("TAX_UNIT46_G2_UNAFFORDABLE", &s, &value);
    let before = s.draft.clone();
    let mut bad = value.clone();
    bad.employee_qsehra_facts[0].household = None;
    rejected_with(
        {
            let request = marketplace_request(&mut s, bad);
            send(&mut s, request)
        },
        "PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED",
    );
    assert_eq!(s.draft, before);
    let mut bad = value.clone();
    bad.employee_qsehra_facts[0].income = None;
    invalid(&mut s, bad);
    let mut bad = value.clone();
    bad.employee_qsehra_facts[0].income = Some(MarketplaceQsehraIncomeSource::Documentary {
        components: MarketplaceQsehraIncome {
            agi: 1_200_001,
            exempt_interest: 0,
            foreign_exclusion: 0,
            social: 0,
            taxable_social: 0,
        },
    });
    rejected_with(
        {
            let request = marketplace_request(&mut s, bad);
            send(&mut s, request)
        },
        "PTC_QSEHRA_RETURN_CONFLICT",
    );
    assert_eq!(s.draft, before);
    let mut unavailable = value.clone();
    unavailable.employee_qsehra_facts[0].household =
        Some(MarketplaceQsehraEmployeeHouseholdPage::OtherPeople);
    let unavailable = reopen(&mut s, unavailable);
    assert!(
        estimate(&s.draft)
            .issues
            .iter()
            .any(|i| i.display.key == "PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED")
    );
    capture_marketplace("TAX_UNIT46_G2_OTHER_PEOPLE", &s, &unavailable);
    value = reopen(&mut s, value);
    value.qsehra[0].months[0].self_benchmark = 19_960;
    value.benchmarks.push(area(vec![-1, sibling], 70_000));
    value = reopen(&mut s, value);
    // Exact equality is affordable; only employee excluded. Neither parent nor
    // sibling is employee spouse/dependent. Actual remaining-family benchmark
    // $700-$55-$100=$545; C4(B) reduction survives affordable employee coverage.
    assert_eq!(value_of(&supported(&s.draft), "premium_tax_credit"), 545);
    capture_marketplace("TAX_UNIT46_G2_AFFORDABLE", &s, &value);
    let mut bad = value.clone();
    bad.employee_qsehra_facts[0].household = Some(MarketplaceQsehraEmployeeHouseholdPage::Known {
        spouse: Some(employee),
        dependents: vec![],
    });
    invalid(&mut s, bad);
    value.benchmarks[0].groups[0].people.push(employee);
    reopen(&mut s, value);
    unsupported(&s.draft, "Marketplace source");
}

#[test]
fn unit47_marketplace_native_policy_cohort_survives_save_reopen_and_restart() {
    let (mut session, _) = at_the_marketplace_page(single());
    let mut page = marketplace_fixture_page(
        (0..31)
            .map(|index| {
                let mut form = one_month(1_000, 100_000, 0);
                form.value.issuer = alloc::format!("Synthetic insurer {index}");
                form.value.policy_number = alloc::format!("Synthetic policy {index}");
                form
            })
            .collect(),
    );
    page.benchmarks = vec![area(vec![-1], 100_000)];
    let page = reopen(&mut session, page);
    assert_eq!(page.forms.len(), 31);
    assert!(page.forms.iter().all(|form| form.id > 0));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(value(&supported(&saved), "premium_tax_credit"), 310);
    let mut cold = restart(saved.clone());
    let opened = open_credits_topic(&mut cold, TaxSection::Marketplace);
    assert_eq!(marketplace_page_of(&opened).forms, page.forms);
    let mut bad = page;
    bad.forms[30].coverage[0].people.push(999_999);
    invalid(&mut cold, bad);
    assert_eq!(cold.draft, saved);
}
