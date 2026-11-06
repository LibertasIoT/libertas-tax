use crate::tests::*;

pub(in crate::tests) fn home_use(owned: u32, lived: u32) -> HomeUse {
    HomeUse { care: HomeCare::NONE, owned, lived, prior_exclusion: false }
}
pub(in crate::tests) fn home_sale(joint: bool) -> HomeSale {
    HomeSale {
        use_history: HomeSaleUse::Personal,
        separate: None,
        payment: SalePayment::InFull,
        exchange: None,
        price: 80000000,
        expenses: 5000000,
        basis: 20000000,
        term: GainTerm::Long,
        exclusion: HomeExclusion::Full {
            taxpayer: home_use(1000, 1000),
            spouse: joint.then(|| home_use(0, 1000)),
        },
        ordinary: true,
        facts: None,
    }
}
pub(in crate::tests) fn duty_home_use(recent_owned: u32, earlier_lived: u32) -> DutyHomeUse {
    DutyHomeUse {
        recent_owned,
        recent_lived: 0,
        earlier_owned: 0,
        earlier_lived,
        care: DutyHomeCare::NONE,
    }
}
pub(in crate::tests) fn official_duty_sale(joint: bool) -> HomeSale {
    let mut home = home_sale(joint);
    home.exclusion = HomeExclusion::ServiceSuspension {
        election: HomeServiceElection {
            service: HomeDutyKind::Uniformed {
                duty: QualifiedHomeDuty { extended_orders: true, remote_or_quarters: true },
            },
            one_property: true,
            suspended_days: 2192,
        },
        taxpayer: FullDutyHomeUse { history: duty_home_use(1827, 800), prior_exclusion: false },
        spouse: joint
            .then(|| FullDutyHomeUse { history: duty_home_use(0, 800), prior_exclusion: false }),
    };
    home
}

pub(in crate::tests) fn reduced_home_periods(days: u32) -> ReducedHomeUse {
    ReducedHomeUse { care: HomeCare::NONE, owned: days, lived: days, since_prior: 730 }
}

pub(in crate::tests) fn home_care(days: u32, owned_residence_days: u32) -> HomeCare {
    HomeCare { days, owned_residence_days }
}
// Synthetic settlement and residence records: $800,000 price less $50,000
// selling expenses and $200,000 basis leaves $550,000 before the exclusion.
pub(in crate::tests) fn home_sale_source(joint: bool) -> crate::MainHomeSale {
    crate::MainHomeSale {
        form: crate::HomeSaleSource::Paper(home_sale_paper(crate::Form1099S {
            filer: Default::default(),
            transferor: Default::default(),
            address: "12 Synthetic Lane".into(),
            closed: 20261231,
            proceeds: 80_000_000,
            property_received: false,
            foreign_person: false,
        })),
        answers: Some(crate::RealEstateSaleUse::MainHome(crate::HomeSaleAnswers {
            property_value: crate::SourceCents::NotReported,
            firpta_withheld: crate::SourceCents::NotReported,
            form_1099s: true,
            acquired: 20240405,
            received: crate::HomeReceived::Bought,
            expenses: 5_000_000,
            payment: crate::SalePayment::InFull,
            exchange: None,
            basis: 20_000_000,
            lived: crate::HomeLived::Days { days: 1000 },
            earlier_sale: crate::EarlierHomeSale::No,
            spouse: if joint {
                crate::HomeSpouse::Joint {
                    lived: crate::HomeLived::Days { days: 1000 },
                    earlier_sale: crate::EarlierHomeSale::No,
                }
            } else {
                crate::HomeSpouse::NotJoint
            },
            reason: crate::HomeSaleReason::TwoYears,
            service: None,
            use_history: crate::HomeRentalUse {
                depreciation: 0,
                nonqualified_days: 0,
                separate: None,
            },
            more: crate::HomeSaleMore::Nothing,
        })),
    }
}

// Ownership began before the added official-duty period; the 800 days of
// residence fit within that ownership, with no residence in the recent five years.
pub(in crate::tests) fn official_duty_sale_source(joint: bool) -> crate::MainHomeSale {
    let mut source = home_sale_source(joint);
    let answers = source.facts_mut();
    answers.acquired = 20160101;
    answers.lived = crate::HomeLived::Days { days: 0 };
    if let crate::HomeSpouse::Joint { lived, .. } = &mut answers.spouse {
        *lived = crate::HomeLived::Days { days: 0 };
    }
    answers.service = Some(crate::HomeService {
        election: crate::HomeServiceElection {
            service: crate::HomeDutyKind::Uniformed {
                duty: crate::QualifiedHomeDuty { extended_orders: true, remote_or_quarters: true },
            },
            one_property: true,
            suspended_days: 2192,
        },
        added_lived: 800,
        added_care: None,
        spouse_added_lived: joint.then_some(800),
        spouse_added_care: None,
    });
    source
}

// The same stored sale closed on this date; the home facts keep their days.
pub(in crate::tests) fn sold_on(mut source: crate::MainHomeSale, date: u32) -> crate::MainHomeSale {
    match &mut source.form {
        crate::HomeSaleSource::Paper(form) => form.box_1_date_of_closing = date,
        crate::HomeSaleSource::Records(sale) => sale.closed = date,
    }
    source
}
