use crate::tests::*;

// Fixture adapters: tests state native cents; accepted entries keep the paper and its answers.

/// A Form 1099-G refund entry with its topic answer.
pub(in crate::tests) fn refund_source(
    amount: i64,
    withholding: i64,
    history: TaxRefundHistory,
) -> GovernmentPaymentSource {
    tax_refund_source(FederalTaxRefund { amount, withholding, history })
}

pub(in crate::tests) fn refund_income(
    amount: i64,
    withholding: i64,
    history: TaxRefundHistory,
) -> FederalIncome {
    FederalIncome::GovernmentPayment { data: refund_source(amount, withholding, history) }
}

/// A W-2 stated natively beside the topic answers its saved entry carries.
#[derive(Clone, Debug)]
pub(in crate::tests) struct AnsweredW2 {
    pub w2: W2,
    pub answers: W2Answers,
}

impl core::ops::Deref for AnsweredW2 {
    type Target = W2;
    fn deref(&self) -> &W2 {
        &self.w2
    }
}

impl core::ops::DerefMut for AnsweredW2 {
    fn deref_mut(&mut self) -> &mut W2 {
        &mut self.w2
    }
}

impl WagePaperFixture for AnsweredW2 {
    fn wage_source(self) -> W2Source {
        let mut source = self.w2.wage_source();
        source.answers = self.answers;
        source
    }
}

/// A native W-2 with no topic answers yet.
pub(in crate::tests) fn w2_source(w2: W2) -> AnsweredW2 {
    AnsweredW2 { w2, answers: Default::default() }
}

/// The topic answers of the first accepted W-2 in a draft.
pub(in crate::tests) fn stored_w2_answers(draft: &FederalDraft, index: usize) -> W2Answers {
    let FederalIncome::Wage { data: WageSource::W2(source) } = &draft.income[index].income else {
        panic!("an accepted W-2")
    };
    source.answers.clone()
}

/// The stored main-home sale that reads as this calculation record: the dates, residence days and
/// reason are stated as the answers the record's exclusion comes from.
pub(in crate::tests) fn main_home_source(sale: HomeSale) -> MainHomeSale {
    let HomeSale {
        price,
        expenses,
        basis,
        term: _,
        use_history,
        separate,
        payment,
        exchange,
        exclusion,
        ordinary,
        facts,
    } = sale;
    let mut closed = facts.as_ref().map_or(20261231, |facts| facts.closed);
    let mut more = HomeSaleMore::Nothing;
    let mut reason = HomeSaleReason::TwoYears;
    let mut service = None;
    let lived = |care: &HomeCare, days: u32| {
        if care.days == 0 && care.owned_residence_days == 0 {
            HomeLived::Days { days }
        } else {
            HomeLived::CareFacility {
                home_days: days,
                facility_days: care.days,
                owned_residence_days: care.owned_residence_days,
            }
        }
    };
    let added_care = |care: &DutyHomeCare| {
        (care.earlier_days != 0 || care.earlier_owned_residence_days != 0).then_some(
            crate::home_sale::HomeServiceCare {
                facility_days: care.earlier_days,
                owned_residence_days: care.earlier_owned_residence_days,
            },
        )
    };
    let recent_lived = |history: &DutyHomeUse| {
        lived(
            &HomeCare {
                days: history.care.recent_days,
                owned_residence_days: history.care.recent_owned_residence_days,
            },
            history.recent_lived,
        )
    };
    let earlier = |closed: u32, prior: bool, since: u32| {
        if prior || since < 730 {
            // A prior exclusion is a sale less than 730 days before this closing.
            let since = if prior { since.min(729) } else { since };
            EarlierHomeSale::Yes { sold: crate::dates::add_days(closed, -(since as i32)) }
        } else {
            EarlierHomeSale::No
        }
    };
    let (owned, you_lived, you_earlier, spouse_lived) = match exclusion {
        HomeExclusion::Full { taxpayer, spouse } => (
            taxpayer.owned,
            lived(&taxpayer.care, taxpayer.lived),
            earlier(closed, taxpayer.prior_exclusion, 730),
            spouse.map(|spouse| {
                (lived(&spouse.care, spouse.lived), earlier(closed, spouse.prior_exclusion, 730))
            }),
        ),
        HomeExclusion::Reduced { reason: event, taxpayer, spouse } => {
            reason = HomeSaleReason::Event { reason: event };
            (
                taxpayer.owned,
                lived(&taxpayer.care, taxpayer.lived),
                earlier(closed, false, taxpayer.since_prior),
                spouse.map(|spouse| {
                    (lived(&spouse.care, spouse.lived), earlier(closed, false, spouse.since_prior))
                }),
            )
        }
        HomeExclusion::Widowed {
            sale_date,
            death_date,
            history,
            late_prior,
            joint_tests_at_death,
        } => {
            closed = sale_date;
            more = HomeSaleMore::SurvivingSpouse {
                died: death_date,
                both_qualified: joint_tests_at_death,
                spouse_earlier_sale: late_prior,
                late_acquired: None,
                late_lived: 0,
            };
            (
                history.owned,
                lived(&history.care, history.lived),
                earlier(closed, history.prior_exclusion, 730),
                None,
            )
        }
        HomeExclusion::ServiceSuspension { election, taxpayer, spouse } => {
            service = Some(HomeService {
                election,
                added_lived: taxpayer.history.earlier_lived,
                added_care: added_care(&taxpayer.history.care),
                spouse_added_lived: spouse.as_ref().map(|spouse| spouse.history.earlier_lived),
                spouse_added_care: spouse
                    .as_ref()
                    .and_then(|spouse| added_care(&spouse.history.care)),
            });
            (
                taxpayer.history.recent_owned + taxpayer.history.earlier_owned,
                recent_lived(&taxpayer.history),
                earlier(closed, taxpayer.prior_exclusion, 730),
                spouse.map(|spouse| {
                    (recent_lived(&spouse.history), earlier(closed, spouse.prior_exclusion, 730))
                }),
            )
        }
        HomeExclusion::ReducedService { election, reason: event, taxpayer, spouse } => {
            reason = HomeSaleReason::Event { reason: event };
            service = Some(HomeService {
                election,
                added_lived: taxpayer.history.earlier_lived,
                added_care: added_care(&taxpayer.history.care),
                spouse_added_lived: spouse.as_ref().map(|spouse| spouse.history.earlier_lived),
                spouse_added_care: spouse
                    .as_ref()
                    .and_then(|spouse| added_care(&spouse.history.care)),
            });
            (
                taxpayer.history.recent_owned + taxpayer.history.earlier_owned,
                recent_lived(&taxpayer.history),
                earlier(closed, false, taxpayer.since_prior),
                spouse.map(|spouse| {
                    (recent_lived(&spouse.history), earlier(closed, false, spouse.since_prior))
                }),
            )
        }
        HomeExclusion::Decline => {
            more = HomeSaleMore::Decline;
            (1000, HomeLived::Days { days: 1000 }, EarlierHomeSale::No, None)
        }
    };
    // The calculation record keeps these answers in its facts.
    if let Some(facts) = &facts {
        if facts.decline {
            more = HomeSaleMore::Decline;
        }
        if let Some(situation) = facts.special {
            more = HomeSaleMore::Refused { situation };
        }
        if let HomeSaleMore::SurvivingSpouse { late_acquired, late_lived, .. } = &mut more {
            *late_acquired = facts.late_acquired;
            *late_lived = facts.late_lived;
        }
    }
    if !ordinary {
        more = HomeSaleMore::Refused { situation: HomeSaleSpecial::Other };
    }
    // A stated rental or business use period names the days owned the sale's dates derive from.
    let owned_days = match &use_history {
        HomeSaleUse::Adjusted { owned_days, .. } => *owned_days,
        HomeSaleUse::Personal => owned,
    };
    let acquired = facts.as_ref().map_or_else(
        || crate::dates::add_days(closed, -(owned_days as i32)),
        |facts| facts.acquired,
    );
    let (depreciation, nonqualified_days) = match use_history {
        HomeSaleUse::Personal => (0, 0),
        HomeSaleUse::Adjusted { depreciation, nonqualified_days, .. } => {
            (depreciation, nonqualified_days)
        }
    };
    let (property_received, property_value, foreign_person, firpta_withheld) = facts
        .as_ref()
        .map_or((false, SourceCents::NotReported, false, SourceCents::NotReported), |facts| {
            (
                facts.property_received,
                facts.property_value.clone(),
                facts.foreign_person,
                facts.firpta_withheld.clone(),
            )
        });
    let reported_value = match &property_value {
        SourceCents::Reported { amount } => *amount,
        _ => 0,
    };
    let form = Form1099S {
        filer: Default::default(),
        transferor: Default::default(),
        address: "12 Synthetic Lane".into(),
        closed,
        proceeds: price - reported_value,
        property_received,
        foreign_person,
    };
    let answers = HomeSaleAnswers {
        property_value,
        firpta_withheld,
        form_1099s: facts.as_ref().is_none_or(|facts| facts.form_1099s),
        acquired,
        received: facts.map_or(HomeReceived::Bought, |facts| facts.received),
        expenses,
        payment,
        exchange,
        basis,
        lived: you_lived,
        earlier_sale: you_earlier,
        spouse: match spouse_lived {
            Some((lived, earlier_sale)) => HomeSpouse::Joint { lived, earlier_sale },
            None => HomeSpouse::NotJoint,
        },
        reason,
        service,
        use_history: HomeRentalUse { depreciation, nonqualified_days, separate },
        more,
    };
    if answers.form_1099s {
        MainHomeSale {
            form: crate::HomeSaleSource::Paper(form_1099s_paper(form)),
            answers: Some(crate::RealEstateSaleUse::MainHome(answers)),
        }
    } else {
        MainHomeSale::records(form, answers)
    }
}

/// A native view of an accepted source whose edits are written back through its adapter.
pub(in crate::tests) struct NativeEdit<'a, S, N: Clone> {
    target: &'a mut S,
    native: N,
    write: fn(N) -> S,
}

impl<S, N: Clone> core::ops::Deref for NativeEdit<'_, S, N> {
    type Target = N;
    fn deref(&self) -> &N {
        &self.native
    }
}

impl<S, N: Clone> core::ops::DerefMut for NativeEdit<'_, S, N> {
    fn deref_mut(&mut self) -> &mut N {
        &mut self.native
    }
}

impl<S, N: Clone> Drop for NativeEdit<'_, S, N> {
    fn drop(&mut self) {
        *self.target = (self.write)(self.native.clone());
    }
}

pub(in crate::tests) fn edit_home(
    source: &mut MainHomeSale,
) -> NativeEdit<'_, MainHomeSale, HomeSale> {
    NativeEdit { native: HomeSale::from(&*source), target: source, write: main_home_source }
}

/// The printed Form 1099-S that reads as this closing record.
pub(in crate::tests) fn form_1099s_paper(form: Form1099S) -> Form1099SPaper {
    let Form1099S {
        filer,
        transferor,
        address,
        closed,
        proceeds,
        property_received,
        foreign_person,
    } = form;
    Form1099SPaper {
        calendar_year: 2026,
        filer,
        transferor,
        box_1_date_of_closing: closed,
        box_2a_total_gross_proceeds: proceeds,
        box_3_address_or_legal_description: address,
        box_6_property_or_services: property_received,
        box_7_foreign_person: foreign_person,
        ..Default::default()
    }
}

pub(in crate::tests) fn edit_iso(
    source: &mut IsoExerciseSource,
) -> NativeEdit<'_, IsoExerciseSource, IsoExercise> {
    NativeEdit {
        native: IsoExercise::try_from(&*source).expect("answered ISO fixture"),
        target: source,
        write: iso_exercise_source,
    }
}

pub(in crate::tests) fn edit_debt(
    source: &mut PersonalDebtSource,
) -> NativeEdit<'_, PersonalDebtSource, PersonalDebtCancellation> {
    NativeEdit {
        native: PersonalDebtCancellation::from(&*source),
        target: source,
        write: debt_source,
    }
}

pub(in crate::tests) fn edit_acquisition(
    source: &mut PropertyAcquisitionSource,
) -> NativeEdit<'_, PropertyAcquisitionSource, PropertyAcquisition> {
    NativeEdit {
        native: PropertyAcquisition::from(&*source),
        target: source,
        write: acquisition_source,
    }
}

pub(in crate::tests) fn mortgage_loan_sources(loans: Vec<MortgageLoan>) -> Vec<Mortgage1098Source> {
    loans.into_iter().map(mortgage_loan_source).collect()
}
