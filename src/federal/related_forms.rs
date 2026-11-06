use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Calculation sources with recipient resolution from the accepted return.
#[derive(Clone, Copy)]
pub(crate) struct RelatedFormContext<'a> {
    pub draft: &'a crate::FederalDraft,
    pub claims: &'a BTreeMap<i64, i64>,
    macrs: Option<&'a alloc::sync::Arc<crate::macrs_source::MacrsElectionView>>,
    participation:
        Option<&'a alloc::sync::Arc<crate::direct_participation::DirectParticipationView>>,
}

impl<'a> RelatedFormContext<'a> {
    pub fn original(draft: &'a crate::FederalDraft, claims: &'a BTreeMap<i64, i64>) -> Self {
        Self { draft, claims, participation: None, macrs: None }
    }
    pub fn bound(
        draft: &'a crate::FederalDraft,
        claims: &'a BTreeMap<i64, i64>,
        participation: &'a alloc::sync::Arc<crate::direct_participation::DirectParticipationView>,
    ) -> Self {
        Self { draft, claims, participation: Some(participation), macrs: None }
    }
    pub fn with_macrs(
        mut self,
        view: &'a alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
    ) -> Self {
        self.macrs = Some(view);
        self
    }
    pub fn macrs(self) -> alloc::sync::Arc<crate::macrs_source::MacrsElectionView> {
        self.macrs.cloned().unwrap_or_else(|| {
            alloc::sync::Arc::new(crate::macrs_source::MacrsElectionView::new(self.draft))
        })
    }
    pub fn participation(
        self,
    ) -> alloc::sync::Arc<crate::direct_participation::DirectParticipationView> {
        match self.participation {
            Some(view) => view.clone(),
            None => self.draft.direct_participation_view(),
        }
    }
}

/// The document or independently allocated part an activity can claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentKind {
    Form1099Nec,
    Form1099KPart,
    Form1099MiscRents,
    Form1099MiscRoyalties,
    Form1099MiscOther,
    Form1099MiscBusiness,
    Form1099MiscCropInsurance,
    Patr1099,
    Form1099GGrant,
    Form1099GAgriculture,
    Mortgage1098,
}

/// An accepted source's display projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CenterDocument {
    pub id: i64,
    pub form_id: i64,
    /// The component's own name without its document: "Box 1", "Payment part: $10".
    pub part: Vec<u8>,
    pub eligible: Vec<i64>,
    pub kind: DocumentKind,
    pub summary: RelatedFormSummary,
}

/// Form summary
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
pub struct RelatedFormSummary {
    /// Form
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Payer or lender
    #[libertas_read_only]
    pub payer: String,
    /// Reported amount
    /// [NullText]
    /// Not available yet
    #[libertas_read_only]
    #[libertas_money("USD", 2)]
    pub amount: Option<i64>,
}

/// Form you can relate
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
pub struct RelatedFormChoice {
    /// Accepted source identity
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    pub id: i64,
    /// Form summary
    #[libertas_read_only]
    pub summary: RelatedFormSummary,
}

/// Related form
/// View the form here; edit it in Documents.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
#[libertas_link("ViewRelatedForm", stacked = true)]
#[libertas_calculation_root("ProjectRelatedForm")]
pub struct RelatedFormView {
    /// Form
    #[libertas_enum_source("^^^.candidates")]
    #[libertas_unique(1)]
    #[libertas_calculation_request("ProjectRelatedForm")]
    pub key: i64,
    /// Summary
    /// [NullText]
    /// Not loaded yet
    #[libertas_read_only]
    #[libertas_output_only]
    pub summary: Option<RelatedFormSummary>,
}

/// Related forms
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport,
)]
pub struct RelatedForms {
    /// Forms you can relate
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub candidates: Vec<RelatedFormChoice>,
    /// Forms related to this activity
    /// Add or remove a relation here. Edit the form in Documents.
    #[libertas_unordered]
    /// ----
    /// Form
    pub forms: Vec<RelatedFormView>,
}

/// Activity related forms
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
pub struct ActivityRelatedForms {
    /// Activity
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub activity: Vec<u8>,
    /// Related forms
    #[libertas_read_only]
    pub forms: Vec<ReviewedRelatedForm>,
}

/// Related form
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
#[libertas_link("ViewReviewedRelatedForm", stacked = true)]
pub struct ReviewedRelatedForm {
    /// Accepted source identity
    #[libertas_hidden]
    #[libertas_read_only]
    pub key: i64,
    /// Form summary
    #[libertas_read_only]
    pub summary: RelatedFormSummary,
}

/// Applicability belongs to the activity kind.
pub trait Activity {
    const KINDS: &'static [DocumentKind];
    fn related(&self) -> &RelatedForms;
    fn related_mut(&mut self) -> &mut RelatedForms;
}

impl RelatedForms {
    pub fn page<A: Activity>(
        stored: &[i64],
        center: &[CenterDocument],
        claimed_elsewhere: &BTreeSet<i64>,
    ) -> Self {
        let candidates: Vec<_> = center
            .iter()
            .filter(|d| A::KINDS.contains(&d.kind) && !claimed_elsewhere.contains(&d.id))
            .map(|d| RelatedFormChoice { id: d.id, summary: d.summary.clone() })
            .collect();
        let forms = stored
            .iter()
            .map(|&key| RelatedFormView { key, summary: Self::project(&candidates, key) })
            .collect();
        Self { candidates, forms }
    }

    pub fn project(candidates: &[RelatedFormChoice], key: i64) -> Option<RelatedFormSummary> {
        candidates.iter().find(|d| d.id == key).map(|d| d.summary.clone())
    }

    /// The first rejected member locates the submitted key in the activity editor.
    /// Display projections never authorize a submitted relation.
    pub fn validate<A: Activity>(
        keys: &[i64],
        center: &[CenterDocument],
        claimed_elsewhere: &BTreeSet<i64>,
    ) -> Result<(), usize> {
        let mut seen = BTreeSet::new();
        for (index, key) in keys.iter().enumerate() {
            if !seen.insert(*key)
                || claimed_elsewhere.contains(key)
                || !center.iter().any(|d| d.id == *key && A::KINDS.contains(&d.kind))
            {
                return Err(index);
            }
        }
        Ok(())
    }

    /// Removing a relation never modifies its document.
    pub fn prune<A: Activity>(stored: &mut Vec<i64>, center: &[CenterDocument]) -> bool {
        let before = stored.len();
        stored.retain(|key| center.iter().any(|d| d.id == *key && A::KINDS.contains(&d.kind)));
        stored.len() != before
    }

    /// Resolve the complete eligible activity cohort once, before filtering its consumers.
    pub fn effective(
        activities: &[ActivityClaim],
        center: &[CenterDocument],
    ) -> BTreeMap<i64, i64> {
        center
            .iter()
            .filter_map(|d| {
                let eligible: Vec<_> = activities
                    .iter()
                    .filter(|a| a.kinds.contains(&d.kind) && d.eligible.contains(&a.id))
                    .collect();
                let mut explicit = eligible.iter().filter(|a| a.related.contains(&d.id));
                let claimant = match (explicit.next(), explicit.next()) {
                    (Some(a), None) => Some(a.id),
                    (None, None) if eligible.len() == 1 => Some(eligible[0].id),
                    _ => None,
                };
                claimant.map(|id| (d.id, id))
            })
            .collect()
    }
}

/// One activity's explicit claims and declared applicability.
#[derive(Clone, Debug)]
pub struct ActivityClaim {
    pub id: i64,
    pub related: Vec<i64>,
    pub kinds: &'static [DocumentKind],
}

pub const BUSINESS_KINDS: &[DocumentKind] = &[
    DocumentKind::Form1099Nec,
    DocumentKind::Form1099KPart,
    DocumentKind::Form1099MiscRents,
    DocumentKind::Form1099MiscRoyalties,
    DocumentKind::Form1099MiscOther,
    DocumentKind::Form1099MiscBusiness,
    DocumentKind::Patr1099,
    DocumentKind::Form1099GGrant,
];
pub const RENTAL_KINDS: &[DocumentKind] =
    &[DocumentKind::Form1099MiscRents, DocumentKind::Form1099KPart, DocumentKind::Mortgage1098];
pub const FARM_KINDS: &[DocumentKind] = &[
    DocumentKind::Form1099GGrant,
    DocumentKind::Patr1099,
    DocumentKind::Form1099GAgriculture,
    DocumentKind::Form1099MiscCropInsurance,
];
pub const FARM_RENTAL_KINDS: &[DocumentKind] = FARM_KINDS;
pub const HOBBY_KINDS: &[DocumentKind] =
    &[DocumentKind::Form1099KPart, DocumentKind::Form1099MiscOther];
pub const EQUIPMENT_KINDS: &[DocumentKind] = &[DocumentKind::Form1099MiscRents];
pub const ROYALTY_KINDS: &[DocumentKind] = &[DocumentKind::Form1099MiscRoyalties];

/// The native activity class used when a new row has not yet been accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityKind {
    Business,
    Rental,
    Farm,
    FarmRental,
    Hobby,
    Equipment,
    Royalty,
}

/// Current editor metadata needed for source eligibility; no provisional tax record is fabricated.
pub struct ProvisionalActivity {
    pub id: i64,
    pub owner: crate::Owner,
    pub kind: ActivityKind,
    pub cooperative_eligible: bool,
}

pub fn catalog_for_provisional(
    draft: &crate::FederalDraft,
    activity: &ProvisionalActivity,
) -> Vec<CenterDocument> {
    crate::interview::related_forms::catalog_with_provisional(draft, Some(activity))
        .into_iter()
        .filter(|d| d.eligible.contains(&activity.id))
        .collect()
}

/// Project accepted source components, preserving their physical-form identity.
pub fn catalog(draft: &crate::FederalDraft) -> Vec<CenterDocument> {
    crate::interview::related_forms::catalog(draft)
}

pub fn catalog_for(draft: &crate::FederalDraft, activity: i64) -> Vec<CenterDocument> {
    catalog(draft).into_iter().filter(|d| d.eligible.contains(&activity)).collect()
}

pub fn draft_claims(draft: &crate::FederalDraft) -> Vec<ActivityClaim> {
    crate::interview::related_forms::draft_claims(draft)
}

pub fn claimed_elsewhere(draft: &crate::FederalDraft, activity: i64) -> BTreeSet<i64> {
    draft_claims(draft).into_iter().filter(|a| a.id != activity).flat_map(|a| a.related).collect()
}

pub fn effective_claimants(draft: &crate::FederalDraft) -> BTreeMap<i64, i64> {
    RelatedForms::effective(&draft_claims(draft), &catalog(draft))
}

// A component whose amount needs a receiving activity that nothing links yet.
pub(crate) struct Unplaced<'a> {
    pub component: CenterDocument,
    pub entry: &'a crate::FederalIncomeEntry,
    pub sections: Vec<crate::TaxSection>,
    pub key: &'static str,
}

impl Unplaced<'_> {
    // The one topic page that owns the activity this paper's amount belongs to,
    // when that page can host it. Papers that fit several kinds of activity
    // are owned by the page of their natural kind.
    pub(crate) fn owner(&self) -> Option<crate::TaxSection> {
        let owner = match self.component.kind {
            DocumentKind::Form1099MiscRents | DocumentKind::Form1099MiscRoyalties => {
                crate::TaxSection::Rentals
            }
            DocumentKind::Form1099GAgriculture
            | DocumentKind::Form1099MiscCropInsurance
            | DocumentKind::Patr1099
            | DocumentKind::Form1099GGrant => crate::TaxSection::Farming,
            DocumentKind::Form1099Nec | DocumentKind::Form1099MiscBusiness => {
                crate::TaxSection::SelfEmployment
            }
            _ => return None,
        };
        self.sections.contains(&owner).then_some(owner)
    }
}

// Source owners decide whether a component needs an activity. The catalog alone
// decides which activities are eligible; the supplied accepted map owns defaults.
pub(crate) fn unplaced<'a>(
    draft: &'a crate::FederalDraft,
    claims: &BTreeMap<i64, i64>,
) -> Vec<Unplaced<'a>> {
    let mut found = Vec::new();
    let center = catalog(draft);
    let activities = draft_claims(draft);
    let mut required = BTreeSet::new();
    for entry in &draft.income {
        required.extend(
            crate::information_returns::flows(&entry.income)
                .into_iter()
                .filter_map(|flow| crate::information_returns::component_key(entry, flow.slot)),
        );
        match &entry.income {
            crate::FederalIncome::Misc1099 { data }
                if crate::Form1099Misc::from(data).crop_insurance != 0 =>
            {
                required.insert(data.component_ids.crop_insurance);
            }
            crate::FederalIncome::GovernmentPayment { data }
                if crate::FederalUnemployment::from(data).agriculture_payment != 0 =>
            {
                required.insert(data.component_ids.agriculture);
            }
            _ => (),
        }
    }
    required.extend(
        crate::cooperative::sources_with_claims(draft, claims)
            .into_iter()
            .filter(crate::cooperative::recipient_required)
            .map(
                |source| {
                    if source.allocation.id == 0 { source.parent } else { source.allocation.id }
                },
            ),
    );
    let activity_kinds = [
        (crate::TaxSection::SelfEmployment, BUSINESS_KINDS),
        (crate::TaxSection::Rentals, RENTAL_KINDS),
        (crate::TaxSection::Farming, FARM_KINDS),
        (crate::TaxSection::Farming, FARM_RENTAL_KINDS),
        (crate::TaxSection::OtherIncome, HOBBY_KINDS),
        (crate::TaxSection::OtherIncome, EQUIPMENT_KINDS),
        (crate::TaxSection::Rentals, ROYALTY_KINDS),
    ];
    for component in center
        .iter()
        .filter(|component| required.contains(&component.id) && !claims.contains_key(&component.id))
    {
        let Some(entry) = draft.income.iter().find(|entry| entry.id == component.form_id) else {
            continue;
        };
        let mut sections = Vec::new();
        for (section, kinds) in activity_kinds {
            if kinds.contains(&component.kind)
                && (component.eligible.is_empty()
                    || activities.iter().any(|activity| {
                        activity.kinds == kinds && component.eligible.contains(&activity.id)
                    }))
                && !sections.contains(&section)
            {
                sections.push(section);
            }
        }
        let explicitly_claimed =
            activities.iter().any(|activity| activity.related.contains(&component.id));
        let key = if explicitly_claimed || !component.eligible.is_empty() {
            "ISSUE_RELATED_FORM_ACTIVITY_NOT_CHOSEN"
        } else {
            "ISSUE_RELATED_FORM_ACTIVITY_MISSING"
        };
        found.push(Unplaced { component: component.clone(), entry, sections, key });
    }
    found
}

pub(crate) fn destination_issues_with_claims(
    draft: &crate::FederalDraft,
    claims: &BTreeMap<i64, i64>,
    result: &mut crate::FederalResult,
) -> bool {
    let mut emitted = false;
    for Unplaced { component, entry, sections, key } in unplaced(draft, claims) {
        // Source-completeness checks keep their refusal result. Replace only their
        // recipient diagnostic for this same source with the component correction.
        result.issues.retain(|issue| {
            issue.context.identity() != crate::federal_model::NoteIdentity::Income(entry.id)
                || !matches!(issue.display.key.as_str(),
                    "ISSUE_COOP_ACTIVITY_NEEDED"
                    | "ISSUE_FORM_1099_G_BOX_7_AGRICULTURE_PAYMENTS_NEED_AN_EXISTING_FARM_ACTIVI_99130ACC"
                    | "ISSUE_FORM_1099_G_BOX_7_AGRICULTURE_PAYMENTS_NEED_A_CHOSEN_FARM")
        });
        let context = crate::interview::income_issue_context(entry).alternatives(&sections);
        crate::federal_rules::issue_message(
            result,
            &crate::interview::resolved_issue_context(draft, &context),
            #[cfg(test)]
            "Related form",
            #[cfg(test)]
            "This source component needs its receiving activity. Add it on that activity's Related forms.",
            crate::Message::new(key, alloc::vec![crate::MessageArg::Encoded(component.part)]),
        );
        emitted = true;
    }
    emitted
}

/// Accepted related form
#[derive(Clone, Debug, PartialEq, LibertasAvroDecode, LibertasAvroEncode, LibertasExport)]
#[libertas_read_only]
#[allow(clippy::large_enum_variant)]
pub enum RelatedForm {
    /// 1099-NEC
    Form1099Nec {
        payer: crate::PrintedParty,
        recipient: crate::PrintedParty,
        document: crate::FederalForm1099Nec,
        state_details: crate::StateIncomeFields,
    },
    /// 1099-K
    Form1099K {
        payer: crate::PrintedParty,
        recipient: crate::PrintedParty,
        document: crate::FederalForm1099K,
        state_details: crate::StateWithholdingFields,
    },
    /// 1099-MISC
    Form1099Misc {
        payer: crate::PrintedParty,
        recipient: crate::PrintedParty,
        document: crate::FederalForm1099Misc,
        state_details: crate::StateIncomeFields,
    },
    /// 1099-PATR
    Patr1099(crate::FederalCooperative),
    /// 1099-G
    Form1099G {
        payer: crate::PrintedParty,
        recipient: crate::PrintedParty,
        document: crate::FederalUnemployment,
        state_details: crate::StateWithholdingFields,
    },
    /// 1098 mortgage interest
    Mortgage1098(crate::MortgageLoan),
}

impl Activity for crate::Rental {
    const KINDS: &'static [DocumentKind] = crate::related_forms::RENTAL_KINDS;

    fn related(&self) -> &RelatedForms {
        &self.related
    }
    fn related_mut(&mut self) -> &mut RelatedForms {
        &mut self.related
    }
}

// Save, estimate and preview validate the same activity-owned component keys.
pub(crate) fn activity_issue(
    draft: &crate::FederalDraft,
    entry: &crate::FederalIncomeEntry,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    use crate::{
        Business, BusinessActivity, EquipmentRent, FederalIncome, NonbusinessPropertyKind, Rental,
        RoyaltyProperty,
    };

    let (keys, variant) = match &entry.income {
        FederalIncome::Business { data }
            if matches!(data.activity, BusinessActivity::Ordinary { .. }) =>
        {
            (&data.related, "Business")
        }
        FederalIncome::Business { data } => match &data.activity {
            BusinessActivity::Farming { data } => (&data.related, "Farm"),
            _ => return None,
        },
        FederalIncome::FarmRental { data } => (&data.farm.related, "FarmRental"),
        FederalIncome::Rental { data } => (&data.related, "Rental"),
        FederalIncome::HobbySales { data } => (&data.related, "HobbySales"),
        FederalIncome::Royalty { data } => (&data.related, "Royalties"),
        FederalIncome::NonbusinessProperty {
            kind: NonbusinessPropertyKind::PersonalRental { related, .. },
            ..
        } => (related, "EquipmentRental"),
        _ => return None,
    };
    let center = crate::related_forms::catalog_for(draft, entry.id);
    let claimed = crate::related_forms::claimed_elsewhere(draft, entry.id);
    let validated = match &entry.income {
        FederalIncome::Business { data } => match data.activity {
            BusinessActivity::Farming { .. } => {
                RelatedForms::validate::<crate::Farm>(keys, &center, &claimed)
            }
            _ => RelatedForms::validate::<Business>(keys, &center, &claimed),
        },
        FederalIncome::FarmRental { .. } => {
            RelatedForms::validate::<crate::FarmRental>(keys, &center, &claimed)
        }
        FederalIncome::Rental { .. } => RelatedForms::validate::<Rental>(keys, &center, &claimed),
        FederalIncome::HobbySales { .. } => {
            RelatedForms::validate::<crate::HobbySales>(keys, &center, &claimed)
        }
        FederalIncome::Royalty { .. } => {
            RelatedForms::validate::<RoyaltyProperty>(keys, &center, &claimed)
        }
        FederalIncome::NonbusinessProperty { .. } => {
            RelatedForms::validate::<EquipmentRent>(keys, &center, &claimed)
        }
        _ => return None,
    };
    validated.err().map(|member| {
        ValidationIssue::new("PROBLEM_RELATED_FORM_UNAVAILABLE")
            .at_field("key")
            .in_list("forms", member)
            .in_field("related")
            .in_field(variant)
    })
}

// Validate accepted relations before any calculation filters its source cohort.
pub(crate) fn source_issue(
    draft: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    draft
        .income
        .iter()
        .find_map(|entry| activity_issue(draft, entry).map(|issue| (entry.id, issue)))
}
