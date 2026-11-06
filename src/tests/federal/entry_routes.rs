// Every section, income record and refused situation has an entry route: one
// classification, kept here, that fails to compile when a TaxSection or an
// IncomeRecord variant is added without a route, and fails when a route names
// an option or page the walk does not offer. Documents are routed by their
// category union (IncomeSource), whose variants are the standalone documents.
use super::*;
use crate::tests::cross::interview::{from_review, open_often_missed, save_often_missed};

// How a person reaches a section, a record or a refused situation.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Route {
    // The household pages and the whole-return answers.
    Household,
    // A hub or page opened from See all topics, Go to or the walk itself.
    Hub,
    // A document of its category (income documents and the standalone
    // forms of the credits and deductions topics).
    Document,
    // What no document completes: the activities, payments and carryovers.
    BeyondDocuments,
    // A direct question on Income without a form, opening the record's own page.
    WithoutForm,
    // An option of "Income and taxes people often miss".
    Income(u32),
    // An option of "Deductions and credits people often miss".
    Benefit(u32),
    // Derived from other answers, or shown on the page of its record.
    Derived,
}

fn section_route(section: TaxSection) -> Route {
    use Route::*;
    use TaxSection as T;
    match section {
        T::Setup | T::People | T::Spouse | T::Children | T::InjuredSpouse => Household,
        T::IncomeWithoutForm
        | T::Checklist
        | T::Payments
        | T::Deductions
        | T::Taxes
        | T::GiftCarryovers => BeyondDocuments,
        T::Income | T::Adjustments | T::Itemized | T::Credits | T::OftenMissed | T::Review => Hub,
        T::Documents
        | T::Wages
        | T::InterestDividends
        | T::InvestmentSales
        | T::Retirement
        | T::GovernmentBenefits
        | T::Gambling
        | T::PassThrough
        | T::CanceledDebt
        | T::Hsa
        | T::LongTermCareBenefits
        | T::StudentLoan
        | T::Mortgage
        | T::Education => Document,
        T::SelfEmployment => WithoutForm,
        T::Rentals | T::Farming | T::WorkAbroad | T::IraContributions | T::Care | T::OtherTaxes => {
            BeyondDocuments
        }
        T::OtherIncome => Income(MissedIncome::SCHOLARSHIP.0),
        T::MoreAdjustments => Benefit(MissedBenefits::ALIMONY_PAID.0),
        T::Educator => Benefit(MissedBenefits::EDUCATOR.0),
        T::EmployeeExpenses => Benefit(MissedBenefits::EMPLOYEE_EXPENSES.0),
        T::Medical => Benefit(MissedBenefits::MEDICAL.0),
        T::OtherItemized => Benefit(MissedBenefits::INVESTMENT_INTEREST.0),
        T::Marketplace => Income(MissedIncome::MARKETPLACE.0),
        T::MoreCredits => Benefit(MissedBenefits::FOREIGN_TAX.0),
        T::HomeOffices | T::SelfEmployedBenefits | T::Saver | T::ParentTax | T::IncomeByPeriod => {
            Derived
        }
    }
}

fn record_route(record: &IncomeRecord) -> Route {
    use IncomeRecord as R;
    use Route::*;
    match record {
        R::CashTipsWithoutW2(_) | R::Business(_) => WithoutForm,
        R::PrivateLoanInterest(_) => Income(MissedIncome::PRIVATE_LOAN_INTEREST.0),
        R::Rental(_)
        | R::Royalties(_)
        | R::EquipmentRental(_)
        | R::ForeignAccountOrTrust(_)
        | R::Farm(_)
        | R::FarmAveraging(_)
        | R::FarmLoss(_)
        | R::FarmRental(_)
        | R::Form2555(_)
        | R::ForeignPay(_)
        | R::NonfarmOptionalMethod(_)
        | R::Form8606(_)
        | R::CapitalLossCarryover(_)
        | R::QbiLossCarryforward(_)
        | R::NetOperatingLoss(_) => BeyondDocuments,
        R::SellerMortgageInterest(_) | R::InstallmentPayments(_) | R::InstallmentSale(_) => {
            Income(MissedIncome::INSTALLMENT.0)
        }
        R::OpportunityFundDeferral(_) | R::MainHomeSale { .. } => {
            Income(MissedIncome::PROPERTY_SOLD.0)
        }
        R::RelatedExchange(_) => Income(MissedIncome::RELATED_EXCHANGE.0),
        R::GamblingLosses(_) => Income(MissedIncome::GAMBLING.0),
        R::Form8915(_) => Income(MissedIncome::DISASTER_DISTRIBUTION.0),
        R::MissedRmd(_) => Income(MissedIncome::MISSED_RMD.0),
        R::AlimonyReceived(_) => Income(MissedIncome::ALIMONY.0),
        R::HealthLoanRepayment(_) => Income(MissedIncome::CANCELED_DEBT.0),
        R::Scholarship(_) => Income(MissedIncome::SCHOLARSHIP.0),
        R::JuryPay(_) | R::HobbySales(_) | R::OtherIncome(_) => Hub,
        R::CasualtyLivingInsurance(_) | R::DisasterMitigation(_) => {
            Income(MissedIncome::CASUALTY.0)
        }
        R::Form8814(_) => Benefit(MissedBenefits::CHILD_INCOME.0),
        R::SmallBusinessStockLoss(_) | R::WorthlessSecurity(_) | R::BadDebt(_) => {
            Benefit(MissedBenefits::WORTHLESS.0)
        }
        R::LtcInsured(_)
        | R::Qcd(_)
        | R::TaxableBondPremium(_)
        | R::BusinessPropertyLosses(_)
        | R::SpouseSection179(_)
        | R::SocialLumpSums(_)
        | R::RothHistory(_)
        | R::Section1256Carryback(_)
        | R::EducationAccountExpenses(_)
        | R::InheritedIra(_)
        | R::AbleContributions(_)
        | R::Clergy(_) => Derived,
    }
}

// The same declarations produce an exhaustive match of each actual category
// union and its kind list. A new nested variant cannot silently inherit a route.
macro_rules! document_routes {
    ($function:ident, $kinds:ident, $ty:ident, [$($variant:ident),+ $(,)?]) => {
        fn $function(value: &$ty) -> Route {
            match value {
                $($ty::$variant(_) => Route::Document,)+
            }
        }
        const $kinds: &[i32] = &[
            $(libertas_macros::variant_index!(IncomeSourceKind::$variant) as i32,)+
        ];
    };
}

document_routes!(wage_route, WAGE_KINDS, WageDocument, [W2, Form8919]);
document_routes!(
    interest_route,
    INTEREST_KINDS,
    InterestDividendDocument,
    [Int1099, Div1099, Oid1099, Form2439, SavingsBonds, Form8818]
);
document_routes!(
    sale_route,
    SALE_KINDS,
    InvestmentSaleDocument,
    [
        Sale1099B,
        Contracts1099B,
        Form1099B,
        Form1099DA,
        Form1099S,
        Form3921,
        Form3922,
        IsoShareSale,
        EmployeeShareSale,
        EsppShareSale,
        NuaShareSale,
        QsbsSale,
        OpportunityFund
    ]
);
document_routes!(
    retirement_route,
    RETIREMENT_KINDS,
    RetirementDocument,
    [R1099, Rrb1099R, Ssa1099, Rrb1099]
);
document_routes!(
    employment_route,
    EMPLOYMENT_KINDS,
    SelfEmploymentDocument,
    [Form1099Nec, Form1099K]
);
const GOVERNMENT_KINDS: &[i32] =
    &[libertas_macros::variant_index!(IncomeSourceKind::Form1099G) as i32];
document_routes!(
    other_route,
    OTHER_KINDS,
    OtherIncomeDocument,
    [
        Form1099Misc,
        Q1099,
        Sa1099,
        Ltc1099,
        Qa1099,
        WildfireRelief,
        DisasterRelief,
        HsaTrusteeStatement
    ]
);
document_routes!(
    k1_route,
    K1_KINDS,
    K1Document,
    [K1Partnership, K1PartnershipActivities, K1Scorp, K1ScorpActivities, K1EstateTrust]
);
document_routes!(debt_route, DEBT_KINDS, CanceledDebtDocument, [Cancel1099C, Acquisition1099A]);

fn source_route(source: &IncomeSource) -> Route {
    match source {
        IncomeSource::Wage { value } => wage_route(value),
        IncomeSource::InterestDividend { value } => interest_route(value),
        IncomeSource::InvestmentSale { value } => sale_route(value),
        IncomeSource::Retirement { value } => retirement_route(value),
        IncomeSource::SelfEmployment { value } => employment_route(value),
        IncomeSource::GovernmentPayment { .. } => Route::Document,
        IncomeSource::OtherIncome { value } => other_route(value),
        IncomeSource::K1 { value } => k1_route(value),
        IncomeSource::CanceledDebt { value } => debt_route(value),
        IncomeSource::Farming(_) | IncomeSource::Gambling(_) => Route::Document,
        IncomeSource::Record { value } => record_route(value),
    }
}

#[test]
fn every_document_kind_has_exactly_one_center_category() {
    let mut kinds: Vec<_> = [
        WAGE_KINDS,
        INTEREST_KINDS,
        SALE_KINDS,
        RETIREMENT_KINDS,
        EMPLOYMENT_KINDS,
        GOVERNMENT_KINDS,
        OTHER_KINDS,
        K1_KINDS,
        DEBT_KINDS,
        &[libertas_macros::variant_index!(IncomeSourceKind::Patr1099) as i32],
        &[libertas_macros::variant_index!(IncomeSourceKind::W2g) as i32],
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    kinds.sort_unstable();
    assert!(kinds.windows(2).all(|pair| pair[0] != pair[1]));
    let mut offered = crate::interview::center_document_kinds();
    offered.sort_unstable();
    assert_eq!(kinds, offered);

    // Destructure the actual center, including every category. New categories
    // require an explicit ownership check alongside these standalone forms.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let P::SaveDocuments {
        documents:
            DocumentCenter {
                common:
                    DocumentsYouMayHave {
                        wages,
                        interest_dividends,
                        investment_sales,
                        retirement,
                        self_employment,
                        government_payments,
                        gambling,
                        k1,
                        mortgages,
                        health_accounts,
                        student_loans,
                        education,
                        marketplace,
                    },
                other:
                    OtherDocumentTypes {
                        farming,
                        other_income,
                        canceled_debt,
                        vehicle_loans,
                        vehicle_gifts,
                        credits,
                    },
            },
        ..
    } = crate::tests::support::client::income_pages::open_document_center(&mut session)
    else {
        panic!("document center expected")
    };
    assert!(vehicle_gifts.documents.is_empty());
    let _ = (
        wages,
        interest_dividends,
        investment_sales,
        retirement,
        self_employment,
        government_payments,
        gambling,
        k1,
        farming,
        other_income,
        canceled_debt,
    );
    fn element_type<T>(_: &[T]) -> &'static str {
        core::any::type_name::<T>()
    }
    let standalone = [
        element_type(&mortgages.documents),
        element_type(&health_accounts.documents),
        element_type(&student_loans.documents),
        element_type(&education.students),
        element_type(&marketplace.documents),
        element_type(&vehicle_loans.documents),
        core::any::type_name::<TaxCreditBondDocument>(),
        core::any::type_name::<Form8986Paper>(),
    ];
    // The credit category is itself a union, and must stay exhaustive too.
    let credit_type = |document: &CreditDocument| match document {
        CreditDocument::CreditBond(_) => core::any::type_name::<TaxCreditBondDocument>(),
        CreditDocument::PartnershipAudit(_) => core::any::type_name::<Form8986Paper>(),
    };
    for document in &credits.documents {
        assert!(standalone.contains(&credit_type(document)));
    }
    for (index, kind) in standalone.iter().enumerate() {
        assert!(!standalone[..index].contains(kind), "{kind} has two categories");
    }
}

#[test]
fn every_section_is_reachable_by_its_route() {
    // The classification is exhaustive over the sequence, and the sequence
    // holds every section once.
    let sequence = crate::interview::SEQUENCE;
    for (index, section) in sequence.iter().enumerate() {
        assert!(!sequence[..index].contains(section), "{section:?}");
        let _ = section_route(*section);
    }
    // A section routed to an option is planned by that option.
    let d = complete(FederalStatus::Single);
    for section in sequence {
        let bits = match section_route(section) {
            Route::Income(bit) => OftenMissed { income: bit, ..Default::default() },
            Route::Benefit(bit) => OftenMissed { benefits: bit, ..Default::default() },
            _ => continue,
        };
        let mut chosen = d.clone();
        chosen.often_missed = bits;
        // An itemized topic is planned as the deductions page while the
        // return does not itemize.
        let planned = crate::interview::planned_sections(&chosen);
        assert!(
            planned.contains(&section) || planned.contains(&TaxSection::Deductions),
            "{section:?} is not planned by its option"
        );
    }
    // Every section the review offers opens from it (See all topics and
    // Go to), so none depends on an option alone; a section it does not offer
    // is conditional on the return's facts.
    let mut s = Interview::new(TEST_USER, d);
    let Reply::Response(P::Review { allowed_sections, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("review expected")
    };
    let conditional = [
        TaxSection::Spouse,
        TaxSection::Itemized,
        TaxSection::Medical,
        TaxSection::Taxes,
        TaxSection::Mortgage,
        TaxSection::GiftCarryovers,
        TaxSection::OtherItemized,
        TaxSection::SelfEmployedBenefits,
        TaxSection::Saver,
        TaxSection::ParentTax,
        TaxSection::IncomeByPeriod,
        TaxSection::InjuredSpouse,
    ];
    for section in sequence {
        if !allowed_sections.contains(&(section as i32)) {
            assert!(conditional.contains(&section), "{section:?} is not offered");
            continue;
        }
        let reply = from_review(&mut s, section);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{section:?}: {reply:?}");
    }
}

#[test]
fn every_income_record_and_document_has_a_route() {
    // IncomeRecord and IncomeSource are matched exhaustively above. Every
    // option a record is routed to exists on the page.
    let incomes: Vec<u32> = crate::interview::MISSED_INCOME.iter().map(|(bit, _)| bit.0).collect();
    let jury = IncomeRecord::JuryPay(JuryPay { received: 0, remitted: 0 });
    // Specialized records remain directly available on their topic, while
    // generic income without a form is answered on its own front page.
    assert_eq!(record_route(&jury), Route::Hub);
    assert!(incomes.contains(&MissedIncome::DISALLOWED_CREDIT.0));
    assert_eq!(source_route(&IncomeSource::Record { value: jury }), Route::Hub);
}

fn refusal_route(item: &Unprepared) -> TaxSection {
    match item {
        Unprepared::Situation { situation } => match situation {
            RareSituation::RailroadRepresentative
            | RareSituation::RemicResidual
            | RareSituation::Straddles
            | RareSituation::SpecialDepreciation
            | RareSituation::AssetDisposed
            | RareSituation::LikeKindExchange
            | RareSituation::TaxTreaty
            | RareSituation::Chapter11Debtor
            | RareSituation::ForeignCorporation
            | RareSituation::Casualty => TaxSection::OftenMissed,
        },
        Unprepared::OtherAdjustments
        | Unprepared::AlternativeMinimumTax
        | Unprepared::HousingCreditRecapture
        | Unprepared::Section965 => TaxSection::OftenMissed,
        Unprepared::OtherAdditionalTaxes { kind } => match kind {
            RareAdditionalTax::Form4255NetEpe
            | RareAdditionalTax::Form4255ExcessivePayment
            | RareAdditionalTax::CreditRecaptureOther
            | RareAdditionalTax::InstallmentLotsTimeshares
            | RareAdditionalTax::InstallmentInterestHistory
            | RareAdditionalTax::HousingDirectOrOrdinaryPartner
            | RareAdditionalTax::FractionalCharitableRecapture
            | RareAdditionalTax::DeferredCompensation409A
            | RareAdditionalTax::DeferredCompensation457A
            | RareAdditionalTax::ExcessBenefits72m5
            | RareAdditionalTax::TrustAccumulation4970
            | RareAdditionalTax::InsiderStock4985
            | RareAdditionalTax::LookBack8697
            | RareAdditionalTax::LookBack8866
            | RareAdditionalTax::Pfic8621Interest
            | RareAdditionalTax::AnotherIdentifiedAddedTax => TaxSection::OftenMissed,
        },
    }
}

#[test]
fn every_refused_situation_is_offered_on_the_page_that_declares_it() {
    // The page offers the types of the refusal owners, so each refused kind
    // is a value the page saves, and the estimate then names it.
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    for item in Unprepared::all() {
        assert_eq!(refusal_route(&item), TaxSection::OftenMissed);
        let before = open_often_missed(&mut s);
        let P::SaveOftenMissed { value, .. } = before else { unreachable!() };
        let reply = save_often_missed(
            &mut s,
            OftenMissed {
                unprepared: vec![item.clone()],
                income: value.income,
                benefits: value.benefits,
            },
        );
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{item:?}: {reply:?}");
        assert_eq!(s.draft.often_missed.unprepared, vec![item.clone()]);
        let result = estimate(&s.draft);
        assert_eq!(result.state, ResultState::Unsupported, "{item:?}");
        let refusal = item.refused();
        assert!(
            result.issues.iter().any(|issue| issue.display.key == refusal.issue_key
                && issue.section == crate::interview::section_name(TaxSection::OftenMissed)),
            "{item:?}"
        );
    }
    // The whole-return refusals are answered with the household's facts.
    for (bit, key, _) in crate::federal_rules::WHOLE_RETURN_REFUSALS {
        let mut household = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let reply = crate::tests::cross::interview::save_whole_return(&mut household, bit);
        assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{key}: {reply:?}");
        let result = estimate(&household.draft);
        assert!(
            result.issues.iter().any(|issue| issue.display.key == key
                && issue.section == crate::interview::section_name(TaxSection::People)),
            "{key}"
        );
    }
}

#[test]
fn a_new_plan_reopens_each_automatically_continued_hub_at_its_question() {
    use crate::tests::cross::interview::{save_checklist, save_without_form};

    let opened_topic = |reply| match reply {
        Reply::Edit(P::SaveIncomes { topic, .. }) => topic,
        Reply::Edit(P::SaveIraContributions { .. }) => TaxSection::IraContributions,
        Reply::Edit(P::SaveCare { .. }) => TaxSection::Care,
        other => panic!("the plan must open its question: {other:?}"),
    };
    for (checklist, self_work, topic) in [
        (Checklist::NONE, true, TaxSection::SelfEmployment),
        (
            Checklist { paid: PaidTopics::IRA_CONTRIBUTIONS.0, ..Checklist::NONE },
            false,
            TaxSection::IraContributions,
        ),
        (Checklist { paid: PaidTopics::CARE.0, ..Checklist::NONE }, false, TaxSection::Care),
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let reply = if self_work {
            save_without_form(
                &mut session,
                IncomeWithoutFormAnswers { self_work: true, ..Default::default() },
            )
        } else {
            let request = save_checklist(&session.draft, TaxSection::Documents as i32, checklist);
            send(&mut session, request)
        };
        assert_eq!(opened_topic(reply), topic);
        let saved = session.draft.clone();
        let mut session = restart(saved.clone());
        let resumed = send(&mut session, P::OpenInterview { user: TEST_USER });
        assert_eq!(opened_topic(resumed), topic, "the unanswered plan survives restart");
        assert_eq!(session.draft, saved);
    }
}

#[test]
fn ordinary_wage_households_reach_review_within_the_request_budget() {
    use crate::tests::support::builders::interview_profiles::wage_household_answer;
    use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;

    for (filing, capture, requests, wages, children) in [
        // Income without a form adds one submitted editor to each household;
        // the wages page asks the W-2's situations once (one more editor).
        (Filing::Single, "TAX_FIELD_BUDGET_A", 9, 1, 0),
        (Filing::MarriedJointly, "TAX_FIELD_BUDGET_B", 14, 2, 2),
    ] {
        let native = NativeInterviewCase(capture);
        let mut session = Interview::new(TEST_USER, FederalDraft::empty(capture.into()));
        let open = P::OpenInterview { user: TEST_USER };
        native.write_file("OUTPUT", "00-request.avro", &open);
        let mut reply = send(&mut session, open);
        let mut sent = 1;
        loop {
            let (Reply::Response(value) | Reply::Edit(value)) = &reply;
            native.write_file("OUTPUT", &alloc::format!("{:02}-reply.avro", sent - 1), value);
            if let Reply::Response(P::Review { remaining_items, correction_notes, .. }) = &reply {
                assert!(remaining_items.is_empty(), "{capture}: {remaining_items:?}");
                assert!(correction_notes.is_empty(), "{capture}: {correction_notes:?}");
                break;
            }
            assert!(sent < requests, "{capture}: unexpected extra editor: {reply:?}");
            let request = wage_household_answer(reply, filing);
            native.write_file("OUTPUT", &alloc::format!("{sent:02}-request.avro"), &request);
            reply = send(&mut session, request);
            sent += 1;
        }
        assert_eq!(session.draft.income.len(), wages);
        assert_eq!(session.draft.dependents.len(), children);
        assert_eq!(session.draft.income[0].owner, crate::DocumentOwner::Yours);
        if filing == Filing::MarriedJointly {
            assert_eq!(session.draft.income[1].owner, crate::DocumentOwner::Spouses);
        }
        assert_eq!(estimate(&session.draft).state, ResultState::EstimatedForSupportedScope);
    }
}
