use crate::tests::*;

// Redesign 3: a stored situation refuses the return by name (every needle
// appears in one issue, under the page it is chosen on, M3-P15) with its
// milestone, and the draft without it keeps its own result.
// One chosen item the estimate refuses, on the page enum that owns it.
#[derive(Clone, Debug)]
pub(crate) enum Chosen {
    Situation(RareSituation),
    Adjustment(MoreAdjustment),
    Credit(MoreCredit),
    Tax(OtherTax),
}
impl From<RareSituation> for Chosen {
    fn from(v: RareSituation) -> Self {
        Self::Situation(v)
    }
}
impl From<MoreAdjustment> for Chosen {
    fn from(v: MoreAdjustment) -> Self {
        Self::Adjustment(v)
    }
}
impl From<MoreCredit> for Chosen {
    fn from(v: MoreCredit) -> Self {
        Self::Credit(v)
    }
}
impl From<OtherTax> for Chosen {
    fn from(v: OtherTax) -> Self {
        Self::Tax(v)
    }
}
impl Chosen {
    pub(crate) fn refused(&self) -> crate::federal_rules::Refusal {
        match self {
            Self::Situation(v) => Some(v.refused()),
            Self::Adjustment(v) => v.refused(),
            Self::Credit(v) => v.refused(),
            Self::Tax(v) => v.refused(),
        }
        .expect("a refusal kind")
    }
    // Adds the item to its page's stored list.
    pub(crate) fn choose(&self, d: &mut FederalDraft) {
        match self {
            Self::Situation(v) => d.situations.push(*v),
            Self::Adjustment(v) => d.adjustment_refusals.push(v.clone()),
            Self::Credit(v) => d.credit_refusals.push(v.refusal().expect("a refusal kind")),
            Self::Tax(v) => d.tax_refusals.push(v.refusal().expect("refusal item")),
        }
    }
}

// Every refused item: the situations, then the payload-less items of the
// page enums in declaration order.
pub(crate) fn every_refusal_item() -> Vec<Chosen> {
    RareSituation::ALL
        .iter()
        .map(|s| Chosen::from(*s))
        .chain([MoreAdjustment::OtherAdjustments].map(Chosen::from))
        .chain(
            [
                OtherTax::AlternativeMinimumTax,
                OtherTax::HousingCreditRecapture,
                OtherTax::Section965,
            ]
            .map(Chosen::from),
        )
        .chain(
            RareAdditionalTax::ALL
                .map(|kind| Chosen::from(OtherTax::OtherAdditionalTaxes { kind })),
        )
        .collect()
}

pub(in crate::tests) fn survivor_with_adult_son(gross_income: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Surviving);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::WidowedPriorYear { could_file_joint: true };
    add_child(&mut d);
    let son = &mut d.dependents[0];
    son.birth_date = 20020301;
    son.situation = DependentSituation::Supported {
        your_support_over_half: son.your_support_over_half(),
        gross_income,
    };
    d
}
