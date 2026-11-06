use crate::kernel::ratio::exact::{self, Exact};

pub(crate) use crate::return_carriers::{NativeWithholdingSource, NativeWithholdingSources};

impl NativeWithholdingSources {
    pub(crate) fn projected(&self, d: &crate::FederalDraft, first: usize) -> Option<[[i64; 2]; 3]> {
        let source = self;
        if !source.complete || first > 1 || source.joint_dollars.iter().any(|v| *v < 0) {
            return None;
        }
        let mut cents: [[Exact; 2]; 3] =
            core::array::from_fn(|_| core::array::from_fn(|_| exact::whole(0)));
        let mut native_cents: [Exact; 3] = core::array::from_fn(|_| exact::whole(0));
        let mut seen = alloc::collections::BTreeSet::new();
        for row in &source.rows {
            if row.id <= 0 || !seen.insert(row.id) || row.cents.iter().any(|v| *v < 0) {
                return None;
            }
            let mut matches = d.income.iter().filter(|e| e.id == row.id);
            let entry = matches.next()?;
            if matches.next().is_some() || entry.owner != crate::DocumentOwner::from(row.owner) {
                return None;
            }
            let (weights, denominator) = entry.accepted_assignment().owner_weights()?;
            for (category, value) in row.cents.iter().enumerate() {
                let value = exact::whole(*value);
                native_cents[category] += &value;
                for (owner, weight) in weights.iter().enumerate() {
                    cents[category][owner] += &value * exact::ratio(*weight, denominator)?;
                }
            }
        }
        let mut owners = [[0; 2]; 3];
        for (category, (cents, native_cents)) in cents.into_iter().zip(native_cents).enumerate() {
            owners[category] = exact::project_rounded_pair(
                source.joint_dollars[category],
                &native_cents,
                cents,
                first,
            )?;
            if owners[category].iter().any(|v| *v < 0) {
                return None;
            }
        }
        Some(owners)
    }

    pub(crate) fn collecting() -> Self {
        Self { complete: true, ..Self::default() }
    }

    pub(crate) fn retain(&mut self, id: i64, owner: crate::Owner, cents: [i64; 3]) {
        if cents == [0; 3] {
            return;
        }
        self.rows.push(NativeWithholdingSource { id, owner, cents });
    }

    pub(crate) fn settle(&mut self, native_cents: [i64; 3], joint_dollars: [i64; 3]) {
        self.joint_dollars = joint_dollars;
        // Coalesce once at the completed boundary, avoiding a linear lookup
        // for every collected document. Identity order has no money policy.
        self.rows.sort_by_key(|row| row.id);
        let mut complete = self.complete;
        self.rows.dedup_by(|row, previous| {
            if row.id != previous.id {
                return false;
            }
            if row.owner != previous.owner {
                complete = false;
            }
            for (stored, amount) in previous.cents.iter_mut().zip(row.cents) {
                if let Some(sum) = stored.checked_add(amount) {
                    *stored = sum;
                } else {
                    complete = false;
                }
            }
            true
        });
        self.complete = complete;
        let sums = self.rows.iter().try_fold([0i64; 3], |mut sums, row| {
            for (sum, amount) in sums.iter_mut().zip(row.cents) {
                *sum = sum.checked_add(amount)?;
            }
            Some(sums)
        });
        self.complete &= sums == Some(native_cents)
            && native_cents.map(crate::kernel::ratio::dollars) == joint_dollars;
        for ids in self.inherited_payment_ids.values_mut() {
            ids.sort_unstable();
        }
    }
}

// Same-invocation income contributions retained by the ordinary native
// collectors. These categories identify existing annual rounding boundaries;
// they do not calculate taxable income or allocate a source between owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum IncomeCategory {
    HouseholdWages,
    MedicaidWaiverWages,
    StrikeBenefits,
    TaxableRefunds,
    AlaskaDividends,
    Prizes,
    NotForProfit,
    OlympicPrizes,
    MedicaidWaiverExclusion,
    InmatePay,
    DeferredPension,
    DigitalAssets,
    Other8z,
    EmployeeTips,
    EmployeeWages,
    UncoveredEmployeeWages,
    DisabilityPension,
    IsoCompensation,
    Alimony,
    Gambling,
    PersonalDebt,
    Hobby,
    Scholarship,
    Grants,
    FamilyLeave,
    CasualtyLivingInsurance,
    TaxedRelief,
    HealthLoanAssistance,
    TrumpDeath,
    Royalties,
    ScorpLoanGain,
    K1OrdinaryGain,
    ClergyHousing,
    MovingExcess,
    DeferralExcess,
    Interest,
    OrdinaryDividends,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeIncomeComponent {
    // The actual accepted income record; unrelated calls do not create one.
    pub(crate) id: i64,
    pub(crate) owner: usize,
    pub(crate) category: IncomeCategory,
    // Actual native cents, including signed exclusions. None is an unresolved
    // native result and never an inferred zero or a document preview.
    pub(crate) source_cents: Option<Exact>,
    // Only the amount present in the legacy Rest source ledger. A native
    // contribution absent from that ledger has Some(0), never a fake source.
    pub(crate) rest_cents: Option<Exact>,
}

// Actual supported Schedule C/F source classes from the same invocation.
// Targets are native annual Dollars; net operands retain genuine Cents.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NativeBusinessIncome {
    pub(crate) joint_dollars: [i64; 2],
    pub(crate) w2_cents: [[i64; 2]; 2],
    pub(crate) other_cents: [[i64; 2]; 2],
    // Actual native Form2555 per-C allocation, summed by owner: already
    // annual Dollars, included in the existing foreign.excluded operand.
    pub(crate) foreign_exclusion_dollars: [i64; 2],
}
impl NativeBusinessIncome {
    pub(crate) fn reconciles(&self, targets: [i64; 2], owners: [i64; 2]) -> bool {
        let source_owners: [i64; 2] = [0, 1].map(|owner| {
            (0..2).map(|form| self.w2_cents[form][owner] + self.other_cents[form][owner]).sum()
        });
        let forms = [0, 1].map(|form| {
            self.w2_cents[form].iter().sum::<i64>() + self.other_cents[form].iter().sum::<i64>()
        });
        self.joint_dollars == targets
            && source_owners == owners
            && crate::kernel::ratio::project_rounded_pair(targets.iter().sum(), forms, 0)
                == Some(targets)
            && self.w2_cents[1] == [0; 2]
    }
}

// Final same-invocation Form8839 amounts. The native owner already rounded
// and settled these annual Dollars; no source precision is reconstructed.
pub(crate) struct AdoptionIncomeDollars {
    pub(crate) owners: [i64; 2],
    pub(crate) w2_owners: [i64; 2],
    pub(crate) deemed_excess: i64,
    pub(crate) source_class_complete: bool,
}
impl From<&crate::adoption::Benefits> for AdoptionIncomeDollars {
    fn from(value: &crate::adoption::Benefits) -> Self {
        Self {
            owners: value.wages,
            w2_owners: value.w2_wages,
            deemed_excess: value.deemed_excess,
            source_class_complete: value.source_class_complete,
        }
    }
}

// Same-invocation Form461 operands. Annual source Dollars and genuine
// unrounded owner Cents have distinct variants; neither is a new worksheet.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Native461IncomeGroup {
    Cents { joint_dollars: i64, owners: [i64; 2] },
    Dollars { joint_dollars: i64, owners: [i64; 2] },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Native461SourceIssue {
    IncompleteSources,
    CapitalOwnerUnavailable,
}
#[derive(Debug)]
pub(crate) struct Native461Sources {
    pub(crate) joint_addback_dollars: i64,
    pub(crate) eligible_joint_dollars: i64,
    pub(crate) groups: Result<alloc::vec::Vec<Native461IncomeGroup>, Native461SourceIssue>,
}
impl Default for Native461Sources {
    fn default() -> Self {
        Self {
            joint_addback_dollars: 0,
            eligible_joint_dollars: 0,
            groups: Ok(alloc::vec::Vec::new()),
        }
    }
}
impl Native461Sources {
    pub(crate) fn owner_net(&self, first: usize) -> Result<[i64; 2], Native461SourceIssue> {
        use Native461SourceIssue::IncompleteSources;
        if first > 1 {
            return Err(IncompleteSources);
        }
        let mut joint = 0i64;
        let mut net = [0i64; 2];
        for group in self.groups.as_ref().map_err(|issue| *issue)? {
            let (target, owners) = match *group {
                Native461IncomeGroup::Cents { joint_dollars, owners } => (
                    joint_dollars,
                    crate::kernel::ratio::project_rounded_pair(joint_dollars, owners, first)
                        .ok_or(IncompleteSources)?,
                ),
                Native461IncomeGroup::Dollars { joint_dollars, owners } => {
                    if owners[0].checked_add(owners[1]) != Some(joint_dollars) {
                        return Err(IncompleteSources);
                    }
                    (joint_dollars, owners)
                }
            };
            joint = joint.checked_add(target).ok_or(IncompleteSources)?;
            for (total, amount) in net.iter_mut().zip(owners) {
                *total = total.checked_add(amount).ok_or(IncompleteSources)?;
            }
        }
        if joint != self.eligible_joint_dollars || net[0].checked_add(net[1]) != Some(joint) {
            return Err(IncompleteSources);
        }
        Ok(net)
    }
    // Dispatcher07a1fe4d: the addback is bounded by each owner's NET
    // eligible loss, not sums of separately losing activities or expenses.
    pub(crate) fn bounds(&self, first: usize) -> Result<[i64; 2], Native461SourceIssue> {
        use Native461SourceIssue::IncompleteSources;
        let joint = self.joint_addback_dollars;
        if joint < 0 || first > 1 {
            return Err(IncompleteSources);
        }
        if joint == 0 {
            return Ok([0; 2]);
        }
        let net = self.owner_net(first)?;
        let losses = [
            crate::kernel::phase::checked_loss(net[0]).ok_or(IncompleteSources)?,
            crate::kernel::phase::checked_loss(net[1]).ok_or(IncompleteSources)?,
        ];
        let owner = if first == 0 { crate::Owner::Taxpayer } else { crate::Owner::Spouse };
        crate::kernel::attribution::native_allowed_bounds_for(joint, losses, owner)
            .map(|(minimum, maximum)| [minimum, maximum])
            .ok_or(IncompleteSources)
    }
}

// Each admitted group retains the ordinary calculator's annual amount.
// Components still awaiting a complete native source feed stay in the
// existing unresolved source path; they are never filled from its residual.
// The existing Form8959 computation already settled these three tax pools.
// Payroll/RRTA operands are genuine Cents; ScheduleSE net is native Dollars.
pub(crate) struct NativeAdditionalMedicare {
    // Known part pools remain readable when another actual source is unavailable.
    pub(crate) unavailable_parts: [bool; 3],
    pub(crate) tax_dollars: [i64; 3],
    pub(crate) payroll_categories: [(i64, [i64; 2]); 3],
    pub(crate) railroad: (i64, [i64; 2]),
    pub(crate) se_joint_dollars: i64,
    pub(crate) se_dollars: Option<[i64; 2]>,
}
impl NativeAdditionalMedicare {
    pub(crate) fn capacities(&self, joint: i64) -> Option<[i64; 2]> {
        if self.unavailable_parts.iter().any(|value| *value) {
            return None;
        }
        let current = |(native, cents): (i64, [i64; 2])| {
            if native < 0
                || cents.iter().any(|amount| *amount < 0)
                || crate::kernel::ratio::dollars(cents[0].checked_add(cents[1])?) != native
            {
                return None;
            }
            // A category rounding to native zero does not create a positive
            // contributor to a different category's tax.
            Some(cents.map(|amount| native != 0 && amount != 0))
        };
        if self.tax_dollars.iter().any(|amount| *amount < 0)
            || self.tax_dollars.iter().try_fold(0i64, |sum, amount| sum.checked_add(*amount))?
                != joint
        {
            return None;
        }
        let mut exclusive = [0i64; 2];
        let mut shared = 0i64;
        for (part, tax) in self.tax_dollars.into_iter().enumerate() {
            if tax == 0 {
                continue;
            }
            let owners = match part {
                0 => {
                    let mut owners = [false; 2];
                    for category in self.payroll_categories {
                        let contributors = current(category)?;
                        for (owner, contributor) in owners.iter_mut().zip(contributors) {
                            *owner |= contributor;
                        }
                    }
                    owners
                }
                1 => current(self.railroad)?,
                _ => {
                    let net = self.se_dollars?;
                    if net.iter().any(|amount| *amount < 0)
                        || net[0].checked_add(net[1])? != self.se_joint_dollars
                    {
                        return None;
                    }
                    net.map(|amount| amount != 0)
                }
            };
            match owners {
                [true, false] => exclusive[0] = exclusive[0].checked_add(tax)?,
                [false, true] => exclusive[1] = exclusive[1].checked_add(tax)?,
                [true, true] => shared = shared.checked_add(tax)?,
                [false, false] => return None,
            }
        }
        Some([exclusive[0].checked_add(shared)?, exclusive[1].checked_add(shared)?])
    }
}

// Distinct native tax boundaries: Form4137 is already Dollars by person;
// uncollected A/B/M/N and golden-parachute taxes each round their source sum.
#[derive(Default)]
pub(crate) struct NativeOtherPayrollTaxes {
    pub(crate) tips: (i64, [i64; 2]), // native Dollars, owner Dollars
    pub(crate) uncollected: (i64, [i64; 2]), // native Dollars, source Cents
    pub(crate) golden: (i64, [i64; 2]), // native Dollars, source Cents
}
impl NativeOtherPayrollTaxes {
    pub(crate) fn owners(&self, first: usize) -> Option<[i64; 2]> {
        if first >= 2 || self.tips.0 != self.tips.1[0].checked_add(self.tips.1[1])? {
            return None;
        }
        let uncollected = crate::kernel::ratio::project_rounded_pair(
            self.uncollected.0,
            self.uncollected.1,
            first,
        )?;
        let golden =
            crate::kernel::ratio::project_rounded_pair(self.golden.0, self.golden.1, first)?;
        Some([
            self.tips.1[0].checked_add(uncollected[0])?.checked_add(golden[0])?,
            self.tips.1[1].checked_add(uncollected[1])?.checked_add(golden[1])?,
        ])
    }
    pub(crate) fn joint(&self) -> i64 {
        self.tips.0 + self.uncollected.0 + self.golden.0
    }
}

// Same Form8959 invocation. Native annual credit categories remain distinct;
// ordinary raw source operands retain possible signed owner cancellation.
// None at the enclosing field is unavailable, never an inferred zero credit.
pub(crate) struct NativeMedicareWithholding {
    pub(crate) railroad_unavailable: bool,
    pub(crate) rrta_dollars: i64,
    pub(crate) rrta_withheld_cents: [i64; 2],
    pub(crate) ordinary_dollars: i64,
    // Actual native positive aggregate; signed operands remain below.
    pub(crate) ordinary_net_cents: i64,
    pub(crate) ordinary_withheld_cents: [i64; 2],
    pub(crate) ordinary_wages_cents: [i64; 2],
    pub(crate) ordinary_regular_cents: i64,
}

// One native fuel category. Source Cents survive until its final annual
// boundary; an absent packet is unavailable, not an empty native collection.
pub(crate) struct NativeFuelCreditSources {
    pub(crate) joint_cents: i64,
    pub(crate) business_cents: [i64; 2],
    pub(crate) home_cents: i64,
}

// Same invocation's native allowed foreign credit. A known physical source
// can settle ownership directly; mixed limited use retains actual eligible
// source Cents, not a gross-tax allocation ratio. Historical/K-3 provenance
// without a native packet stays unknown.
pub(crate) enum NativeForeignCreditCapacity {
    PayeeCents([i64; 2]),
    // Ceilings after each actual ordinary Form1116 category limit.
    CategoryDollars([i64; 2]),
}
pub(crate) struct NativeForeignCreditSources {
    pub(crate) joint: i64,
    pub(crate) owners: Option<[i64; 2]>,
    pub(crate) capacity: Option<NativeForeignCreditCapacity>,
}
impl NativeForeignCreditSources {
    pub(crate) fn unavailable(joint: i64) -> Self {
        Self { joint, owners: (joint == 0).then_some([0; 2]), capacity: None }
    }
}

#[derive(Default)]
pub(crate) struct NativeIncomeCategories {
    // Original native annual categories, before any printed source-class
    // split. Financial ownership retains these exact boundaries and operands.
    pub(crate) whole_groups: alloc::vec::Vec<(i64, [Exact; 2])>,
    pub(crate) groups: alloc::vec::Vec<(i64, [Exact; 2])>, // native dollars, actual cents
    pub(crate) w2_groups: alloc::vec::Vec<(i64, [Exact; 2])>,
    // A mixed source category with no conserving class projection remains
    // unassigned. Its native amount still leaves Rest exactly once.
    pub(crate) unclassified_dollars: i64,
    pub(crate) source_class_unavailable: bool,
    // Actual Form2106 excess outside W2 box1/codeL. Synthetic empty packets
    // retain known zero, independently of an unresolved actual mixed source.
    pub(crate) line_1a_other_cents: [i64; 2],
    pub(crate) line_1a_class_unavailable: bool,
    pub(crate) removed_cents: [Exact; 2],
    // Same-invocation gross native sources; these are NOT additional groups.
    // Existing 2b/3b projection applies its native targets/exclusion once.
    pub(crate) investment_sources: [Option<[Exact; 2]>; 2],
    // Final native annual targets whose source owners are resolved later from
    // the retained LTC and personal-property packets. These are not previews.
    pub(crate) ltc_dollars: i64,
    // The same native invocation exhausts its actual care-benefit Cents subtotal.
    pub(crate) ltc_sources_complete: bool,
    pub(crate) fuel_credit_sources: Option<alloc::boxed::Box<NativeFuelCreditSources>>,
    pub(crate) saver_credit_owners: Option<[i64; 2]>,
    pub(crate) foreign_credit_sources: Option<alloc::boxed::Box<NativeForeignCreditSources>>,
    // Same invocation's actual Form1041-T beneficiary allocations, in Cents.
    pub(crate) estate_estimated_cents_by_owner: Option<[crate::kernel::ratio::exact::Exact; 2]>,
    pub(crate) employee_wage_tax_dollars: Option<[i64; 2]>,
    pub(crate) other_payroll_taxes: Option<alloc::boxed::Box<NativeOtherPayrollTaxes>>,
    pub(crate) medicare_withholding: Option<alloc::boxed::Box<NativeMedicareWithholding>>,
    pub(crate) additional_medicare: Option<alloc::boxed::Box<NativeAdditionalMedicare>>,
    // Actual final native itemized NIIT deduction, with explicit unknown
    // owners. None is reserved for older synthetic source inventories.
    pub(crate) niit_itemized:
        Option<alloc::boxed::Box<super::niit_deductions::NiitItemizedDeductions>>,
    // Same-invocation P/J/next with complete current-source zero evidence.
    // A missing packet is unknown, including in synthetic owner contexts.
    pub(crate) niit_retained: Option<alloc::boxed::Box<super::niit_retained::RetainedNiitUse>>,
    // Actual confirmed NUA exclusion cents by person, present only when
    // every other current worksheet contribution is independently zero.
    pub(crate) niit_current_nua_cents: Option<[i64; 2]>,
    // Same final native Hobby-goods category: annual target and genuine
    // captured owner Cents. The combined Schedule1 hobby/not-for-profit row
    // is not this Form8960 source. None preserves unavailable attribution.
    pub(crate) niit_hobby: Option<(i64, [Exact; 2])>,
    // Same final passive C source items that the native SE owner excluded,
    // except statutory wage receipts. Whole-dollar target and source cents.
    pub(crate) niit_passive_business: Option<(i64, [i64; 2])>,
    pub(crate) personal_rent_dollars: i64,
    // Settled Form2441 owner amounts, already native annual Dollars.
    // Provenance classifies those amounts; it never changes financial owners.
    pub(crate) care_taxable_dollars: [i64; 2],
    pub(crate) care_taxable_from_w2: [bool; 2],
    // Absent only for synthetic contexts whose source inventory excludes
    // adoption. Actual native calls supply known zero or a complete packet.
    pub(crate) adoption: Option<alloc::boxed::Box<AdoptionIncomeDollars>>,
    // None is missing when an actual statutory W2 route is present. Empty
    // synthetic/nonstatutory packets keep the original financial path.
    pub(crate) statutory_business: Option<NativeBusinessIncome>,
    pub(crate) excess_business_loss: Native461Sources,
}

impl NativeIncomeCategories {
    // Source classes have a fixed residual order within the actual native
    // annual category: W-2 rounds first, Other receives the exact remainder.
    // A pure class retains the original native target without re-rounding it.
    pub(crate) fn source_class_totals(
        joint: i64,
        w2: [Exact; 2],
        other: [Exact; 2],
    ) -> Option<[i64; 2]> {
        if w2.iter().all(|value| value == &exact::whole(0)) {
            Some([0, joint])
        } else if other.iter().all(|value| value == &exact::whole(0)) {
            Some([joint, 0])
        } else {
            let wages = exact::dollars(&w2.into_iter().sum())?;
            Some([wages, joint.checked_sub(wages)?])
        }
    }

    fn recorded_w2(
        draft: &crate::FederalDraft,
        components: &[NativeIncomeComponent],
        category: IncomeCategory,
    ) -> Option<[Exact; 2]> {
        // These native categories can contain actual W2 source components.
        // Extra unreported employee tips remain Other even inside a Wage entry.
        if !matches!(
            category,
            IncomeCategory::InmatePay
                | IncomeCategory::MedicaidWaiverWages
                | IncomeCategory::MedicaidWaiverExclusion
        ) {
            return Some(core::array::from_fn(|_| exact::whole(0)));
        }
        components.iter().filter(|part| part.category == category).try_fold(
            core::array::from_fn(|_| exact::whole(0)),
            |mut owners, part| {
                let entry = draft.income.iter().find(|entry| entry.id == part.id)?;
                if matches!(entry.income, crate::FederalIncome::Wage { .. }) {
                    if part.owner != usize::from(entry.owner == crate::DocumentOwner::Spouses) {
                        return None;
                    }
                    owners[part.owner] += part.source_cents.as_ref()?;
                }
                Some(owners)
            },
        )
    }

    fn recorded(
        draft: &crate::FederalDraft,
        components: &[NativeIncomeComponent],
        category: IncomeCategory,
    ) -> Option<([Exact; 2], [Exact; 2])> {
        components.iter().filter(|part| part.category == category).try_fold(
            (core::array::from_fn(|_| exact::whole(0)), core::array::from_fn(|_| exact::whole(0))),
            |(mut sources, mut removed), part| {
                let entry = draft.income.iter().find(|entry| entry.id == part.id)?;
                if part.owner != usize::from(entry.owner == crate::DocumentOwner::Spouses) {
                    return None;
                }
                let (weights, denominator) = entry.accepted_assignment().owner_weights()?;
                let source_cents = part.source_cents.as_ref()?;
                for (owner, weight) in weights.iter().enumerate() {
                    sources[owner] += source_cents * exact::ratio(*weight, denominator)?;
                }
                // Rest still carries the original source under its native
                // carrier owner; remove that operand once before reattribution.
                removed[part.owner] += part.rest_cents.as_ref()?;
                Some((sources, removed))
            },
        )
    }

    // Combine only known native entry contributions with actual late owner
    // operands. Admission below still checks the complete native subtotal.
    pub(crate) fn source_owners(
        draft: &crate::FederalDraft,
        components: &[NativeIncomeComponent],
        category: IncomeCategory,
        late: [i64; 2],
    ) -> Option<[Exact; 2]> {
        Self::recorded(draft, components, category)
            .map(|(sources, _)| [0, 1].map(|i| &sources[i] + exact::whole(late[i])))
    }

    pub(crate) fn collect(
        draft: &crate::FederalDraft,
        components: &[NativeIncomeComponent],
        boundaries: &[(IncomeCategory, Exact, i64, Option<[Exact; 2]>)],
    ) -> Self {
        let mut result = Self::default();
        for (category, native_cents, joint_dollars, native_owners) in boundaries {
            let category = *category;
            let joint_dollars = *joint_dollars;
            let Some((recorded, removed)) = Self::recorded(draft, components, category) else {
                continue;
            };
            // Owner-indexed operands are supplied only by the same native
            // calculation. They include non-entry operands (e.g. W2 codeII),
            // without inventing an entry or removing them twice from Rest.
            let owners = native_owners.clone().unwrap_or(recorded);
            if &owners.iter().cloned().sum::<Exact>() != native_cents {
                continue;
            }
            result.whole_groups.push((joint_dollars, owners.clone()));
            // Whole-category financial ownership and removal are already known.
            // A printed-class projection failure must not put the source back in Rest.
            for (total, cents) in result.removed_cents.iter_mut().zip(removed) {
                *total += cents;
            }
            let w2 = match category {
                // These are same-invocation owner arrays whose native producer
                // reads only actual W2 sources. They are not per-entry deltas.
                IncomeCategory::DeferredPension
                | IncomeCategory::MovingExcess
                | IncomeCategory::DeferralExcess => Some(owners.clone()),
                _ => Self::recorded_w2(draft, components, category),
            };
            let classes = w2.and_then(|w2| {
                let other = [0, 1].map(|i| &owners[i] - &w2[i]);
                Self::source_class_totals(joint_dollars, w2.clone(), other.clone())
                    .map(|totals| (w2, other, totals))
            });
            let Some((w2, other, [w2_joint, other_joint])) = classes else {
                result.source_class_unavailable = true;
                result.unclassified_dollars += joint_dollars;
                continue;
            };
            if w2.iter().all(|value| value == &exact::whole(0)) {
                // No source-class split: retain the original native boundary,
                // including any already-applied late annual-dollar charges.
                result.groups.push((joint_dollars, owners));
            } else if other.iter().all(|value| value == &exact::whole(0)) {
                result.w2_groups.push((joint_dollars, owners));
            } else {
                result.w2_groups.push((w2_joint, w2));
                result.groups.push((other_joint, other));
            }
        }
        result
    }
}
