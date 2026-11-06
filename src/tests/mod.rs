use crate::federal_rules::{estimate, preferential_tax, social_taxable, valid_birth};
use crate::forms::form_2441::care_rate;
use crate::forms::schedule_eic::eic_amount;
use crate::interview::{Interview, Reply};
use crate::k1_loss::{K1LossClass, LossAmounts};
use crate::k1_page::K1Partnership as K1PartnershipPaper;
use crate::k1_page::K1Scorp as K1ScorpPaper;
use crate::tests::federal::schedules_1_3::schedule_1_line;
use crate::*;
use TaxInterviewProtocol as P;
use alloc::{vec, vec::Vec};

mod activity;
mod attribution;
mod business;
#[path = "core/mod.rs"]
mod core_tests;
mod cost_recovery;
mod credits;
pub(crate) mod cross;
mod deductions;
mod family;
mod farm;
mod federal;
mod foreign;
mod health;
#[path = "support/heap.rs"]
mod heap;
mod home;
mod income;
mod investment;
mod passthrough;
mod payments;
mod retirement;
pub(crate) mod support;

// Compatibility for existing topic tests; new helpers belong in support.
pub(crate) use support::builders::{Chosen, every_refusal_item, make_professional};
pub(crate) use support::client::TEST_USER;
pub(in crate::tests) use support::{assert::*, builders::*, client::*};

// Tests by area; existing imports remain available through the support facade.
include!("attribution/unit13_root_registration.rs");
include!("home/unit42_root_registration.rs");
include!("health/unit46_root_registration.rs");
include!("federal/unit47/root_registration.rs");

include!("passthrough/unit27_root_registration.rs");

include!("cost_recovery/unit29_root_registration.rs");

#[path = "cost_recovery/unit17/mod.rs"]
mod unit17;
#[path = "activity/unit19/mod.rs"]
mod unit19;
#[path = "federal/unit51/mod.rs"]
mod unit51;

#[path = "passthrough/k1res/mod.rs"]
mod k1res;

#[path = "passthrough/k1res_section179.rs"]
mod k1res_section179;
