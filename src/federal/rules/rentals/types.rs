use super::*;

// One property's Schedule E amounts after dividing expenses by days of use
// and, for a dwelling unit used as a home, after the Worksheet 5-1 limit.
// Amounts are cents.
pub(crate) struct HomeCasualtyAdmission {
    pub building: usize,
    pub cents: i64,
    pub dollars: i64,
}

pub(crate) struct RentalUse {
    // One computed source-cent amount shared by the deduction and basis owner.
    pub(crate) energy_deduction: i64,
    pub(crate) expense_rows: Vec<crate::extended_model::RentalExpenseAmount>,
    pub current_operating_for_risk: Option<Exact>,
    // Already business-only Form4797 PartIV income, before activity limits.
    pub asset_recapture_cents: i64,
    pub prior_at_risk_cents: i64,
    // Line 21 before passive limits; never below zero for a home.
    pub net: i64,
    // Depreciation deducted this year (rental portion, after the home limit).
    pub depreciation: i64,
    pub home_third_tier: Option<crate::casualty::home::ThirdTierResult>,
    pub current_casualty_losses: Vec<HomeCasualtyAdmission>,
    pub otherwise_casualty_cents: i64,
    // Personal use exceeded the greater of 14 days or 10% of fair rental days.
    pub home: bool,
    // A home rented fewer than 15 days: no income and no rental expenses.
    pub excluded: bool,
    // Worksheet 5-1 lines 7a and 7b.
    pub carry: [i64; 2],
    // Personal portion of mortgage interest and taxes entered as rental expenses.
    pub personal_fixed: Exact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1PropertyResult {
    pub id: i64,
    pub net_cents: i64,
    pub section179_sales_cents: alloc::vec::Vec<i64>,
    pub suspended_cents: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub nonpassive_cents: i64,
    // Reported form projections only.
    pub net: i64,
    pub suspended: i64,
    // Retain the amount excluded from the passive pool, including prior loss
    // absorbed by the same activity. Later NIIT depends on this distinction.
    pub nonpassive: i64,
    pub rentals: alloc::vec::Vec<K1RentalPropertyResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1RentalPropertyResult {
    pub real_estate: bool,
    pub net_cents: i64,
    pub suspended_cents: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub nonpassive_cents: i64,
    // Reported form projections only.
    pub net: i64,
    pub suspended: i64,
    pub nonpassive: i64,
    pub exclude_niit: bool,
    pub niit_disposition_loss: i64,
    // Index of this activity's one QBI entry, including its wages and UBIA.
    pub qbi_index: Option<usize>,
}

impl K1PropertyResult {
    pub(crate) fn reported_net(&self, sales: &[i64]) -> Option<i64> {
        let raw_sales = sales.iter().copied().try_fold(0i64, i64::checked_add)?;
        let reported_sales =
            sales.iter().map(|amount| dollars(*amount)).try_fold(0i64, i64::checked_add)?;
        reported_sales.checked_add(dollars(self.net_cents.checked_sub(raw_sales)?))
    }

    pub(crate) fn business(&self) -> Self {
        let net_cents = self.net_cents - self.rentals.iter().map(|r| r.net_cents).sum::<i64>();
        let suspended_cents = crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(
            self.suspended_cents.get(),
            self.rentals.iter().map(|r| r.suspended_cents.get()).sum(),
        )
        .expect("rental character holds are part of the parent hold");
        let nonpassive_cents =
            self.nonpassive_cents - self.rentals.iter().map(|r| r.nonpassive_cents).sum::<i64>();
        Self {
            id: self.id,
            net_cents,
            suspended_cents,
            nonpassive_cents,
            net: self.net - self.rentals.iter().map(|r| r.net).sum::<i64>(),
            section179_sales_cents: self.section179_sales_cents.clone(),
            suspended: dollars(suspended_cents.get()),
            nonpassive: dollars(nonpassive_cents),
            rentals: alloc::vec::Vec::new(),
        }
    }
}

impl RentalResult {
    pub(crate) fn niit_owners(&self, draft: &FederalDraft) -> Option<[i64; 2]> {
        if !self.inputs_complete
            || self.income_by_id.values().sum::<i64>() != self.income
            || self.nii_adjustment_by_id.values().sum::<i64>()
                != self.scorp_niit_adjustment
                    + self.partner_niit_adjustment
                    + self.rental_niit_adjustment
                    + self.estate_trust_niit_adjustment
            || self.nii_adjustment_by_id.keys().any(|id| !self.income_by_id.contains_key(id))
        {
            return None;
        }
        let mut owners = [0; 2];
        for (id, income) in &self.income_by_id {
            let owner = draft.income.iter().find(|entry| entry.id == *id)?.owner;
            // Both operands are the existing owner's annual source dollars.
            // No raw-cost scaling or inferred release/basis split is involved.
            owners[usize::from(owner == crate::DocumentOwner::Spouses)] +=
                income + self.nii_adjustment_by_id.get(id)?;
        }
        Some(owners)
    }

    pub(crate) fn add_property(
        &mut self,
        id: i64,
        net: i64,
        suspended: i64,
        nonpassive: i64,
        rental: Option<K1RentalPropertyResult>,
    ) {
        let index = match self.k1_section1231.iter().position(|v| v.id == id) {
            Some(index) => index,
            None => {
                self.k1_section1231.push(K1PropertyResult {
                    id,
                    net_cents: 0,
                    section179_sales_cents: alloc::vec::Vec::new(),
                    suspended_cents: crate::kernel::carry::Amount::default(),
                    nonpassive_cents: 0,
                    net: 0,
                    suspended: 0,
                    nonpassive: 0,
                    rentals: alloc::vec::Vec::new(),
                });
                self.k1_section1231.len() - 1
            }
        };
        let item = &mut self.k1_section1231[index];
        item.net_cents += net;
        item.suspended_cents = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
            item.suspended_cents.get() + suspended,
        )
        .expect("nonnegative property source hold");
        item.nonpassive_cents += nonpassive;
        item.net = dollars(item.net_cents);
        item.suspended = dollars(item.suspended_cents.get());
        item.nonpassive = dollars(item.nonpassive_cents);
        if let Some(rental) = rental {
            item.rentals.push(rental);
        }
    }
}

// Preserve partner ownership through the common pool; subsequent SE, QBI,
// EBL and NIIT calculations must not reinterpret these as rental or S-corp net.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartnerPassiveResult {
    pub id: i64,
    pub net: i64,
    pub nonpassive: i64,
    pub suspended: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub prior_released: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1Amount {
    pub id: i64,
    pub net: i64,
    // Actual same-invocation source endpoint; net retains summed reported rows.
    pub source_net_cents: Option<i64>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct K1Cuts {
    pub ordinary: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub short: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    pub long: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    // All three together.
    pub passive: crate::kernel::carry::Amount<crate::kernel::carry::Cents>,
    // Ordinary Form 4797 loss held back, by business activity, actual source cents.
    // The source rows and QBI must use the same per-activity allocation.
    pub ordinary_by_id:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
    // The same Part IX cuts retained by source, short/long actual source cents.
    pub capital_by_id:
        alloc::vec::Vec<(i64, [crate::kernel::carry::Amount<crate::kernel::carry::Cents>; 2])>,
    // Activity-item losses withheld from the return, by the actual source.
    pub income_by_id:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
}

// Pub974 initial-income projection needs the loss BEFORE aggregation with an
// activity's own profit. A current1000/prior5000 activity has net−4000, but
// may have allowed prior loss1000 even when its final net is zero.
// All these rows are real-estate components; the supported K-1 owner admits
// only one box2 activity per accepted parent (box1/3 are distinct classes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HealthRentalActivity {
    pub source: i64,
    pub current: i64,
    pub prior: i64,
    pub allowed_loss: i64,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct HealthPassiveProjection {
    pub deferred: bool,
    pub rentals: alloc::vec::Vec<HealthRentalActivity>,
    pub mixed_losses: bool,
    pub nonpassive_losses: bool,
    pub released_outside_pool: bool,
}

impl HealthPassiveProjection {
    pub(crate) fn initial_addback(&self) -> Result<i64, ()> {
        if !self.deferred {
            return Ok(0);
        }
        // Pub974 pp51/56 defers8582 PartsII/III and excludes real-estate
        // rental losses. The inspected primary sources do not establish a
        // general reallocation of competing other losses or disposed/former
        // passive releases. Do not claim either counterfactual as verified.
        if self.mixed_losses || self.nonpassive_losses || self.released_outside_pool {
            return Err(());
        }
        debug_assert!(self.rentals.iter().all(|v| v.source > 0
            && v.allowed_loss >= 0
            && v.allowed_loss <= positive(-v.current) + v.prior));
        Ok(self.rentals.iter().map(|v| v.allowed_loss).sum())
    }
}

// Actual personal-property inputs in cents, before PAL allocation or filing
// rounding. These are source inputs, not a claim about an allowed loss or a
// current/prior loss-release split. Regular history remains separate from AMT.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PersonalRentalInputs {
    pub owner: crate::Owner,
    pub receipts_cents: i64,
    pub regular_expenses_cents: i64,
    pub expenses_cents: i64,
    pub regular_prior_cents: i64,
    pub prior_cents: i64,
    pub investment_recharacterization: bool,
}

// Runtime source operands from the actual rental-use and at-risk invocation.
// Before/after amounts include the native owner's applicable prior deductions
// and home-use limits; they are not reconstructed current-only profit. The
// final ScheduleE amount remains annual Dollars after the existing PAL owner,
// not a precision feed made by multiplying that amount by100.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RentalOperatingSource {
    pub source: i64,
    pub owner: crate::Owner,
    pub alternative: bool,
    pub before_at_risk_cents: i64,
    pub after_at_risk_cents: i64,
    pub prior_at_risk_cents: i64,
    pub prior_passive_cents: i64,
    pub prior_home_carry_cents: [i64; 2],
    pub home_limited: bool,
    pub excluded: bool,
    pub property_disposition: bool,
    pub filed_ordinary_dollars: Option<i64>,
}

// Native source identity, separate from an annual form row or a list index.
// An accepted K-1 parent can have ordinary, box2 and box3 pool entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostPalComponent {
    PersonalProperty,
    DirectRental,
    DirectBusiness,
    FarmRental,
    PartnershipBusiness,
    PartnershipGroup { group: i64 },
    ScorpBusiness,
    ScorpGroup { group: i64 },
    PartnershipRealEstate,
    PartnershipOtherRental,
    ScorpRealEstate,
    ScorpOtherRental,
    EstateActivity { activity: i64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostPalCharacter {
    ScheduleE,
    ScheduleC,
    PersonalExpense,
    OrdinaryProperty,
    PropertyDisposition,
    Section1231,
    ShortCapital,
    LongCapital,
    DebtCancellation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostPalCharacterAmounts {
    pub character: PostPalCharacter,
    pub net_cents: i64,
    pub suspended_cents: i64,
    pub allowed_net_cents: i64,
}

// Form 6251 keeps an activity's preferences inside the same limitation
// calculation. These flags come from its selected native owner stages.
// https://www.irs.gov/instructions/i6251
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1AmtActivitySource {
    // Includes a formerly passive activity's actual retained/released loss.
    pub(crate) passive: bool,
    pub(crate) loss_limited: bool,
    pub(crate) depletion_cents: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K1AmtActivityAmounts {
    pub(crate) source: K1AmtActivitySource,
    // The actual Schedule E endpoint, including separate nonpassive amounts,
    // after character allocation, section 179 and partner-level care expenses.
    pub(crate) schedule_e_cents: i64,
    pub(crate) schedule_e_dollars: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostPalSourceAmounts {
    pub income: i64,
    pub owner: crate::Owner,
    pub component: PostPalComponent,
    pub(crate) k1_amt: Option<K1AmtActivityAmounts>,
    // False only for positive personal income recharacterized by its native
    // owner. Its known zero hold cannot offset another passive loss.
    pub common_pool: bool,
    // The common contribution, not the separate nonpassive/PTP or own-income
    // release. Those results remain with their native owners. Personal
    // and K-1 operands are actual cents; other adapters retain their annual boundary.
    pub net_cents: i64,
    // The incoming Form8582 prior column, not an inferred current/prior release.
    pub prior_cents: i64,
    pub suspended_cents: i64,
    pub allowed_net_cents: i64,
    pub characters: alloc::vec::Vec<PostPalCharacterAmounts>,
}

// One complete same-invocation result. Personal and K-1 inputs retain actual
// source cents; other owners retain their explicitly reported annual adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostPalSources {
    pub alternative: bool,
    pub total_loss_cents: i64,
    pub suspended_cents: i64,
    pub allowed_loss_cents: i64,
    pub special_allowance_cents: i64,
    pub sources: alloc::vec::Vec<PostPalSourceAmounts>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct RentalResult {
    pub(crate) source_projection: Option<
        alloc::sync::Arc<
            Result<crate::community::SourceProjection, crate::activity_risk::SourceIssue>,
        >,
    >,
    pub(crate) macrs: Option<alloc::sync::Arc<crate::macrs_source::MacrsElectionView>>,
    // Required source failures travel with this invocation, including cached
    // owner snapshots; consumers withhold dependent amounts rather than using
    // an empty pool as a known-zero result.
    pub source_issue: Option<crate::activity_risk::SourceIssue>,
    pub pool_issue: Option<crate::kernel::loss_limits::LossLimitError>,
    // One actual return179 allocation for this source invocation. A supplied
    // selected-system result is never reconstructed from PAL or raw K-1 boxes.
    // None retains the legacy lazy owner call until that source is read.
    pub section179_allocations: Option<alloc::collections::BTreeMap<i64, i64>>,
    // Gross allowed net-loss components entering Form461, before unrelated
    // business profits offset them; dollars, owned by their actual source.
    pub business_losses_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub qbi_source_ids: alloc::vec::Vec<i64>,
    pub inputs_complete: bool,
    pub income: i64,
    // Allowed prior home-use casualty loss, separately from Schedule E; signed cents.
    pub prior_home_casualty_by_owner: [i64; 2],
    pub settled_home_casualty_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub prior_home_casualty_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub home_use: HomeUseContext,
    // Actual Schedule E source dollars after activity limits, before Form461.
    // Personal-property rental expenses remain their Schedule1 owner's work.
    pub income_by_id: alloc::collections::BTreeMap<i64, i64>,
    // The actual Form8960 line4b changes at each native activity invocation,
    // in already settled annual dollars, including known zero for each source.
    pub nii_adjustment_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub personal_inputs: alloc::collections::BTreeMap<i64, PersonalRentalInputs>,
    // One source per actual successful rental-use invocation in this tax view.
    // Missing source or final amount remains absent, never an inferred zero.
    pub operating_sources: alloc::collections::BTreeMap<i64, RentalOperatingSource>,
    // Same selected regular/AMT invocation, including source-specific unavailable facts.
    pub(crate) foreign_sources:
        alloc::collections::BTreeMap<i64, crate::rental_foreign::RentalForeignSource>,
    // Actual Form4835 property sources after the same source's Form6198.
    // Native Form4797 dollars remain the recognized property boundary.
    pub farm_rental_property: alloc::collections::BTreeMap<i64, crate::farm_land::LandGain>,
    pub(crate) direct_business_property:
        alloc::collections::BTreeMap<i64, Vec<crate::business_risk::PropertySource>>,
    pub farm_rental_property_rows:
        alloc::collections::BTreeMap<i64, crate::asset_sale::FarmRentalPropertyRows>,
    pub operating_prior_allowed: alloc::collections::BTreeMap<i64, Option<i64>>,
    // None if a required pool source is incomplete. A missing row is never a
    // zero suspension; only a complete actual source can carry that result.
    pub post_pal: Option<PostPalSources>,
    // Admitted positive nonpassive rental source cents beyond the filing
    // projection. Passive loss pools and prior-loss releases still own their
    // annual-dollar result; do not reconstruct their residuals here.
    pub health_work_income_residual_cents: i64,
    // The net passive income of publicly traded partnerships, inside `income`: Form 4952
    // line 4a investment income (2026 draft i4952 line 4a; section 469(k)).
    pub ptp_income: i64,
    pub qbi: i64,
    pub passive_loss: i64,
    pub scorp_niit_adjustment: i64,
    pub partner_niit_adjustment: i64,
    // Form 8960 line 4b: a real-estate professional's safe-harbor rentals
    // (Reg. 1.1411-4(g)(7)) reversed out of the Schedule E net, regular tax.
    pub rental_niit_adjustment: i64,
    // Schedule K-1 (Form 1041) box 6 net after box 9 deductions (passive and
    // nonpassive), its nonpassive part, and the Form 8960 line 4b reversal of
    // that nonpassive part. Boxes 7 and 8 stay in `income` as passive rent.
    pub estate_activities: alloc::vec::Vec<crate::estate_trust::EstateActivityResult>,
    // Schedule E Part III columns (c), (d), (e), (f), retained from the
    // actual allowed activity rows, whole dollars.
    pub estate_columns: [i64; 4],
    pub estate_trust_business: i64,
    pub estate_trust_nonpassive: i64,
    pub estate_trust_niit_adjustment: i64,
    pub scorp_business: i64,
    // Actual final ordinary business amounts after basis/risk/PAL and the
    // existing179/care deductions, before461. This is a source retention
    // of ordinary_pass_through_business, not income_by_id's mixed classes.
    pub ordinary_business_by_id: alloc::collections::BTreeMap<i64, i64>,
    pub partners: alloc::vec::Vec<PartnerPassiveResult>,
    // Each partnership and S corporation K-1's amount in Schedule E Part II
    // (box 1, guaranteed payments and the box 2 and 3 activities, less its
    // section 179) and the passive loss still suspended, by K-1 identity.
    pub k1_amounts: alloc::vec::Vec<K1Amount>,
    // What each K-1's basis and at-risk limits allowed and held back, by
    // class, in the regular tax system.
    pub k1_ledgers: alloc::vec::Vec<(i64, crate::k1_loss::LossLedger)>,
    pub partner_se: alloc::vec::Vec<(i64, crate::partnership::PartnerSeResult)>,
    // Form8958 provenance for the actual partner on the other return. These
    // sources never enter this filer's SE, health or retirement deductions.
    pub counterpart_partner_se: alloc::vec::Vec<(i64, crate::partnership::PartnerSeResult)>,
    pub scorp_nonpassive: i64,
    // K-1 box 2 activities a real estate professional materially
    // participated in: nonpassive, so not passive investment income.
    pub pass_through_rental_nonpassive: i64,
    pub personal_expense_adjustment: i64,
    pub activities: alloc::vec::Vec<crate::business_rules::QbiActivity>,
    // Source identity for negative allowed QBI; no second QBI calculation.
    pub qbi_loss_sources: alloc::vec::Vec<i64>,
    pub scorp_suspensions:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
    pub k1_section1231: alloc::vec::Vec<K1PropertyResult>,
    pub scorp_qbi_ordinary_suspensions:
        alloc::vec::Vec<(i64, crate::kernel::carry::Amount<crate::kernel::carry::Cents>)>,
    // What the passive rules hold back of the activity items (code
    // R, code S), whole dollars: the loss that does not reach Form 4797 or
    // Schedule D this year.
    pub k1_cuts: K1Cuts,
    // Kept rentals: Form 8582 disallowance of partial dispositions, by
    // income identity and Form 4797 Part II / Part I, whole dollars.
    pub rental_partial_cuts: alloc::vec::Vec<(i64, [i64; 2])>,
    // Schedule E line 26 before royalties: each rental property's income or
    // allowed loss (lines 21 and 22), whole dollars.
    pub part_one: i64,
    // Schedule E line 40: each Form 4835 farm rental's income or allowed
    // loss (line 32 or 34c), whole dollars.
    pub farm_rental: i64,
    // Form 8582's combined passive result before the loss limits: every
    // passive activity's net less its prior-year unallowed losses, plus
    // personal-property rental profit. Form 8582-CR line 6 is zero when this
    // is zero or less.
    pub net_passive_income: i64,
    // Form 8582 line 9: the losses of rental real estate with active
    // participation that the special allowance let you deduct. Form 8582-CR
    // line 35 takes it from the $25,000 ($12,500) allowance that remains.
    pub special_allowance: i64,
    // Form 8582 line 6 modified adjusted gross income, not below zero: Form
    // 8582-CR line 22 for the rehabilitation credit's special allowance.
    pub modified_agi: i64,
    pub health_ptc: HealthPassiveProjection,
}

impl RentalResult {
    // Source-specific final operating cents from this exact common invocation.
    // An absent/incomplete packet remains unavailable, including an expected
    // source omitted by a failed producer. No annual-dollar readback is used.
    pub(crate) fn direct_business_allowances(
        &self,
        alternative: bool,
    ) -> Option<alloc::collections::BTreeMap<i64, i64>> {
        let packet = self.post_pal.as_ref()?;
        if packet.alternative != alternative
            || !self.inputs_complete
            || self.source_issue.is_some()
            || self.pool_issue.is_some()
        {
            return None;
        }
        Some(
            packet
                .sources
                .iter()
                .filter(|source| {
                    source.component == PostPalComponent::DirectBusiness && source.common_pool
                })
                .filter_map(|source| {
                    source
                        .characters
                        .iter()
                        .find(|part| part.character == PostPalCharacter::ScheduleC)
                        .map(|part| (source.income, part.allowed_net_cents))
                })
                .collect(),
        )
    }

    // i8995: only the section 179 a passive activity's Form 8582 allows this
    // year reduces QBI; the held part waits with its carryover.
    pub(crate) fn section179_qbi_cents(&self, d: &FederalDraft, id: i64) -> i64 {
        let held = if crate::section179::passive_pass_through(d, id) {
            self.k1_ledgers.iter().find(|(k, _)| *k == id).map_or(0, |(_, ledger)| {
                ledger.passive_suspended[crate::k1_loss::K1LossClass::Section179].get()
            })
        } else {
            0
        };
        self.section179_allowed_cents(d, id) - held
    }

    pub(crate) fn section179_allowed_cents(&self, d: &FederalDraft, id: i64) -> i64 {
        self.section179_allocations.as_ref().map_or_else(
            || crate::section179::pass_through_allowed(d, id),
            |sources| sources.get(&id).copied().unwrap_or(0),
        )
    }

    // Final allowed personal-property cost from the same system's one PAL
    // invocation. Neither a missing source nor an incomplete invocation is zero.
    pub(crate) fn personal_expenses_cents(&self) -> Option<alloc::collections::BTreeMap<i64, i64>> {
        let packet = self.post_pal.as_ref()?;
        self.personal_inputs
            .iter()
            .map(|(&id, input)| {
                let source = packet.sources.iter().find(|source| {
                    source.income == id && source.component == PostPalComponent::PersonalProperty
                })?;
                Some((
                    id,
                    crate::kernel::carry::native_balance(
                        input.expenses_cents + input.prior_cents,
                        source.suspended_cents,
                    ),
                ))
            })
            .collect()
    }

    // Pub. 596 Worksheet 1 lines 11–13: exclude nonpassive amounts and
    // passive amounts already included in earned income. SE and participation
    // are independent classifications; the exclusions must not overlap.
    pub(crate) fn eic_passive_income(&self) -> i64 {
        self.income
            - self.scorp_nonpassive
            - self.pass_through_rental_nonpassive
            - self.estate_trust_nonpassive
            - self.partners.iter().map(|v| v.nonpassive).sum::<i64>()
            - self.partner_se.iter().map(|(_, v)| v.passive_earnings).sum::<i64>()
    }

    // Ordinary trade/business amounts after basis, risk and passive limits.
    // Do not use QBI eligibility or SE classification to decide Form 461 scope.
    pub(crate) fn ordinary_pass_through_business(&self) -> i64 {
        self.scorp_business
            + self.estate_trust_business
            + self.partners.iter().map(|v| v.net).sum::<i64>()
    }
}

// Source adapters attach presentation/tax facts before entering the passive
// pool. The allocator never recovers an activity's type by looking up its ID.
// The selected system's known eligible business income is used by loss
// limitations. Section199A's property projection belongs only to regular tax.
pub(crate) struct UniformBusinessIncome {
    pub(crate) income: i64,
    pub(crate) regular_qbi: Option<crate::business_rules::QbiActivity>,
}

pub(crate) struct PassiveEntry<'a> {
    pub(crate) id: i64,
    pub(crate) net: i64,
    // Actual native owner-stage amounts. Annual fields are report projections.
    pub(crate) source_cents: Option<PassiveSourceCents>,
    pub(crate) amt_source: Option<K1AmtActivitySource>,
    pub(crate) rental_allowance: bool,
    // The part of the activity's prior unallowed loss that the special
    // allowance does not admit: it arose in a year without active
    // participation (section 469(i)(1); Form 8582 Part V, Worksheet 3).
    pub(crate) ineligible_prior: i64,
    // The prior unallowed loss netted into `net` (Form 8582 column (c)); the
    // activity's current result is `net + prior`.
    pub(crate) prior: i64,
    pub(crate) personal: bool,
    pub(crate) direct_business: bool,
    pub(crate) scorp: Option<&'a crate::ScorpK1>,
    pub(crate) partner: bool,
    pub(crate) source_group: Option<i64>,
    pub(crate) characters: Option<crate::scorp::ActivityCharacters>,
    // The activity items (code E, R and S) in the activity's net,
    // and whether the net is in the pool (a passive activity) or they are
    // nonpassive.
    pub(crate) items: Option<(crate::k1_items::ActivityItems, bool)>,
    pub(crate) nonpassive: i64,
    pub(crate) prior_released: i64,
    // Only when every allowed dollar has the same QBI treatment. Mixed K-1
    // statements and historical vintages need their own post-limit allocation.
    pub(crate) uniform_business: Option<UniformBusinessIncome>,
    // Form 4797 gain of a rental sold in full this year (section 469(g)):
    // part of the activity's passive net, reported on Form 4797, not E.
    pub(crate) disposition_gain: i64,
    // A kept rental whose partial building dispositions share this pool entry.
    pub(crate) rental_partial: bool,
    // Schedule K-1 (Form 1041) boxes 6–8 passive income: Schedule E line 33
    // column (d), never Part I, never a rental allowance or suspended loss.
    pub(crate) estate_trust: bool,
    pub(crate) estate_activity: Option<(i64, &'a crate::EstateActivity)>,
    // A Form 4835 farm rental: its allowed amount is Schedule E line 40,
    // never Part I.
    pub(crate) farm_rental: bool,
    // A K-1 box 2 or box 3 rental activity: Schedule E
    // line 28 of its K-1, never Part I.
    pub(crate) pass_through: Option<PassThroughRental>,
    pub(crate) released: bool,
    // A publicly traded partnership's passive net (section 469(k)): outside
    // the common pool. A gain is income, a loss stays suspended with its PTP.
    pub(crate) ptp: i64,
}

// Same-invocation forwarding only; the normal loss owners and passive pool
// compute these amounts. No reported-dollar reconstruction enters the pool.
pub(crate) struct PassiveSourceCents {
    // Separately stated Schedule E items, in source order: guaranteed services,
    // guaranteed capital, prior fuel recovery, and the PTP's own section179.
    // These retain their distinct reported rows/column while stages keep cents.
    pub(crate) separate_nonpassive: [i64; 4],
    pub(crate) net: i64,
    pub(crate) prior: i64,
    pub(crate) ineligible_prior: i64,
    pub(crate) nonpassive: i64,
    pub(crate) prior_released: i64,
    pub(crate) disposition_gain: i64,
    pub(crate) ptp: i64,
}

#[derive(Clone)]
pub(crate) struct PassThroughRental {
    // "partnership" or "scorp", the K-1 kind's calculation-id prefix.
    pub(crate) kind: &'static str,
    pub(crate) real_estate: bool,
    pub(crate) material: bool,
    pub(crate) qbi: Option<crate::K1RentalQbi>,
    pub(crate) exclude_niit: bool,
    pub(crate) niit_disposition_loss: i64,
}

impl PassiveEntry<'_> {
    pub(crate) fn component(&self) -> PostPalComponent {
        if self.direct_business {
            PostPalComponent::DirectBusiness
        } else if self.personal {
            PostPalComponent::PersonalProperty
        } else if let Some((_, activity)) = self.estate_activity {
            PostPalComponent::EstateActivity { activity: activity.id }
        } else if let Some(rental) = &self.pass_through {
            match (rental.kind, rental.real_estate) {
                ("partnership", true) => PostPalComponent::PartnershipRealEstate,
                ("partnership", false) => PostPalComponent::PartnershipOtherRental,
                ("scorp", true) => PostPalComponent::ScorpRealEstate,
                ("scorp", false) => PostPalComponent::ScorpOtherRental,
                _ => unreachable!("native K1 rental source kind"),
            }
        } else if let Some(group) = self.source_group {
            if self.partner {
                PostPalComponent::PartnershipGroup { group }
            } else {
                PostPalComponent::ScorpGroup { group }
            }
        } else if self.partner {
            PostPalComponent::PartnershipBusiness
        } else if self.scorp.is_some() {
            PostPalComponent::ScorpBusiness
        } else if self.farm_rental {
            PostPalComponent::FarmRental
        } else {
            PostPalComponent::DirectRental
        }
    }

    pub(crate) fn real_estate(&self) -> bool {
        self.pass_through.as_ref().map_or(
            !self.personal
                && !self.direct_business
                && self.scorp.is_none()
                && !self.partner
                && self.source_group.is_none()
                && !self.estate_trust,
            |rental| rental.real_estate,
        )
    }

    // One K-1 rental activity, its loss limits already applied.
    pub(crate) fn pass_through(
        id: i64,
        kind: &'static str,
        activity: crate::k1_rental::RentalActivity,
        niit_safe_harbor: bool,
    ) -> Self {
        PassiveEntry {
            amt_source: None,
            source_cents: Some(PassiveSourceCents {
                separate_nonpassive: [0; 4],
                net: activity.passive,
                prior: activity.prior,
                ineligible_prior: activity.ineligible_prior,
                nonpassive: activity.nonpassive,
                prior_released: activity.prior_released,
                disposition_gain: activity.property_gain,
                ptp: 0,
            }),
            id,
            net: dollars(activity.passive),
            rental_allowance: activity.allowance,
            ineligible_prior: dollars(activity.ineligible_prior),
            prior: dollars(activity.prior),
            personal: false,
            direct_business: false,
            scorp: None,
            partner: false,
            source_group: None,
            characters: activity.characters,
            items: None,
            nonpassive: dollars(activity.nonpassive),
            prior_released: dollars(activity.prior_released),
            uniform_business: None,
            disposition_gain: dollars(activity.property_gain),
            rental_partial: false,
            estate_trust: false,
            estate_activity: None,
            farm_rental: false,
            ptp: 0,
            released: activity.released,
            pass_through: Some(PassThroughRental {
                kind,
                real_estate: activity.real_estate,
                material: activity.material,
                qbi: activity.qbi,
                exclude_niit: niit_safe_harbor && activity.material,
                niit_disposition_loss: activity.niit_disposition_loss[1],
            }),
        }
    }
}

// At-risk loss character stays attached to its source form. Amounts are cents.
#[derive(Clone, Debug)]
pub(crate) struct RentalRisk {
    pub ordinary: i64,
    pub sale: i64,
    pub recapture: Result<i64, crate::activity_risk::ProjectionIssue>,
}

impl RentalRisk {
    pub(crate) fn ordinary_income(&self) -> Result<i64, crate::activity_risk::ProjectionIssue> {
        crate::kernel::ratio::checked_sum([self.ordinary, self.recapture.clone()?]).ok_or(
            crate::activity_risk::ProjectionIssue::Arithmetic(
                crate::kernel::loss_limits::LossLimitError::Overflow,
            ),
        )
    }
    pub(crate) fn from_activity(activity: &crate::activity_risk::ActivityProjection) -> Self {
        Self {
            ordinary: activity.components.allowed[0],
            sale: activity.components.allowed[1],
            recapture: activity.recapture.clone(),
        }
    }
}

pub(crate) use crate::return_carriers::HomeUseContext;

impl HomeUseContext {
    pub(crate) fn route(d: &FederalDraft, force_itemized: Option<bool>) -> Option<bool> {
        force_itemized
            .or_else(|| (d.deduction_choice() == crate::DeductionChoice::Itemized).then_some(true))
    }
    pub(crate) fn dwellings(d: &FederalDraft) -> impl Iterator<Item = i64> + '_ {
        d.income.iter().filter_map(|entry| match &entry.income {
            FederalIncome::Rental { data }
                if !matches!(data.personal_use, RentalPersonalUse::No) =>
            {
                Some(entry.id)
            }
            _ => None,
        })
    }
    // A first or hypothetical calculation starts every dwelling from the
    // supplied AGI; the home-use fixed point replaces it with the observed one.
    pub(crate) fn provisional(d: &FederalDraft, agi: [i64; 2], itemizes: Option<bool>) -> Self {
        let dwelling_agi_cents =
            Self::dwellings(d).filter_map(|id| Some((id, agi[0].checked_mul(100)?))).collect();
        Self { agi, itemizes, dwelling_agi_cents }
    }
    // The invocation's final AGI less each dwelling's amounts included in it.
    // A dwelling whose amount is absent stays unavailable.
    pub(crate) fn observed(
        d: &FederalDraft,
        agi: [i64; 2],
        itemizes: Option<bool>,
        rental: &RentalResult,
    ) -> Self {
        let dwelling_agi_cents = Self::dwellings(d)
            .filter_map(|id| {
                let excluded = rental.income_by_id.get(&id)?.checked_mul(100)?;
                let casualty = *rental.settled_home_casualty_by_id.get(&id)?;
                Some((id, agi[0].checked_mul(100)?.checked_sub(excluded)?.checked_sub(casualty)?))
            })
            .collect();
        Self { agi, itemizes, dwelling_agi_cents }
    }
    // Form 4684 line 17's AGI for this home-use dwelling, in cents, read by
    // the dwelling's Worksheet 5-1 line 2c casualty.
    pub(crate) fn dwelling_agi_cents(&self, rental: i64) -> Option<i64> {
        self.dwelling_agi_cents.get(&rental).copied()
    }
}

// Section 469(c)(7)(A): with the election, a member sale is not a
// disposition of the entire elected activity. Read the actual direct and
// K-1 owners; other rentals and retained home years are not invented members.
#[derive(Clone)]
pub(crate) struct ProfessionalSource {
    pub(crate) id: i64,
    pub(crate) parts: crate::scorp::ActivityCharacters,
    pub(crate) income_release: [i64; 2],
}

#[derive(Clone, Default)]
pub(crate) struct ProfessionalGroups {
    pub(crate) sources: alloc::vec::Vec<ProfessionalSource>,
    pub(crate) magi: i64,
}

impl ProfessionalGroups {
    pub(crate) fn source(&self, id: i64) -> Option<&ProfessionalSource> {
        self.sources.iter().find(|source| source.id == id)
    }

    pub(crate) fn apply(
        &self,
        id: i64,
        activities: &mut [crate::k1_rental::RentalActivity],
        active: bool,
    ) {
        if let Some(source) = self.source(id) {
            for activity in activities.iter_mut().filter(|activity| activity.real_estate) {
                activity.grouped_former_passive(source.parts, source.income_release, active);
            }
        }
    }
}

// Test-only record of the home-use context each regular-tax dwelling
// invocation receives, so tests compare it with the estimate's final AGI.
#[cfg(test)]
extern crate std;
#[cfg(test)]
std::thread_local! {
    pub(crate) static HOME_USE_SEEN: core::cell::RefCell<Vec<(i64, HomeUseContext)>> =
        const { core::cell::RefCell::new(Vec::new()) };
}
