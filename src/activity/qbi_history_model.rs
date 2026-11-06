/// Qualified business income losses carried into 2026
/// Prior-return carryforwards; both zero when there are none. The amounts belong to the people filing this return and include all applicable QBI carryforward history. Combine separate prior returns only when their owners file this return; a former-spouse allocation, missing history or newly released suspended losses needs its own worksheet.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
pub struct QbiHistory {
    /// Business loss carried from 2025 Form 8995 line 16 or Form 8995-A Schedule C line 6
    /// Enter its magnitude as a positive amount. Include only these filers' carryforwards, even if the business closed. Do not enter an NOL, capital loss or still-suspended passive/at-risk loss here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub business: i64,
    /// REIT/PTP loss carried from 2025 Form 8995 line 17 or Form 8995-A line 29
    /// Enter its magnitude as a positive amount. This separate loss category offsets qualified REIT/PTP income only, not ordinary business QBI.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub reit_ptp: i64,
    /// Ownership of retained QBI losses
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The retained loss ownership has not been established
    /// [SetText]
    /// The retained losses belong to the spouses as follows
    #[libertas_default($null)]
    pub injured_owners: Option<crate::InjuredQbiLossOwners>,
}
impl QbiHistory {
    /// No prior QBI loss.
    pub(crate) const NONE: Self = Self { business: 0, reit_ptp: 0, injured_owners: None };
    pub(crate) fn amounts(&self) -> [i64; 2] {
        [self.business, self.reit_ptp]
    }
    /// Whether any prior QBI loss is entered.
    pub(crate) fn any(&self) -> bool {
        self.amounts() != [0, 0]
    }
    pub(crate) fn valid(&self) -> bool {
        self.amounts().iter().all(|v| (0..=100000000000).contains(v))
            && self.injured_owners.as_ref().is_none_or(|v| v.valid(self.amounts()))
    }
}
