use crate::tests::*;

pub(in crate::tests) fn home_use(owned: u32, lived: u32) -> HomeUse {
    HomeUse { care: HomeCare::NONE, owned, lived, prior_exclusion: false }
}
pub(in crate::tests) fn home_sale(joint: bool) -> HomeSale {
    HomeSale {
        use_history: HomeSaleUse::Personal,
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
