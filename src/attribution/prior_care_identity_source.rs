use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// Separate-return claimants of the prior-year care recipients
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredPriorCareClaimants {
    /// The actual qualifying people and their claimants
    People {
        /// People whose 2025 care was first paid in 2026
        #[libertas_size(min = 1)]
        people: Vec<InjuredPriorCarePerson>,
    },
}

impl InjuredPriorCareClaimants {
    pub(crate) fn people(&self) -> &[InjuredPriorCarePerson] {
        match self {
            Self::People { people } => people,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let people = self.people();
        (1..=100).contains(&people.len()) && people.iter().enumerate().all(|(i,p)| {
            p.person.valid() && (1..=100000000000).contains(&p.expense) && !people[..i].iter().any(|v| matches!((&v.person, &p.person), (InjuredPriorCareIdentity::OnReturn { person: a }, InjuredPriorCareIdentity::OnReturn { person: b }) if a == b))
        })
    }
}

/// One actual prior-year qualifying person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InjuredPriorCarePerson {
    /// The qualifying person
    pub person: InjuredPriorCareIdentity,
    /// Spouse who would claim this person on separate 2025 returns
    pub claimant: crate::Owner,
    /// Actual 2025 qualifying expenses for this person first paid in 2026
    /// From the care and payment records. Include only the expenses in this retained worksheet's paid-in-2026 total.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub expense: i64,
}

/// Identity of a prior-year qualifying person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InjuredPriorCareIdentity {
    /// A person already on this return
    OnReturn {
        /// Person
        person: i64,
    },
    /// A person not on this year's return
    Other {
        /// Name from the prior-year care records
        #[libertas_size(min = 1)]
        name: String,
    },
}

impl InjuredPriorCareIdentity {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::OnReturn { person } => *person != 0,
            Self::Other { name } => !name.trim().is_empty(),
        }
    }
}
