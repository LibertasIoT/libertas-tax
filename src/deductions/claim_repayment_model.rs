use crate::diagnostics::RepaymentFilers;
use crate::kernel::ratio::dollars;
use crate::{FederalStatus, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Earlier-year income repayment comparison
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum RepaymentClaim {
    /// No
    None,
    /// Yes, compare the deduction and the credit (claim of right)
    Claim {
        /// Method
        /// Automatic compares complete supported returns. An explicit selection preserves your choice; the other method may produce a lower tax.
        #[libertas_default(Automatic)]
        method: RepaymentMethod,
        /// Original-year return records
        /// One record per original filing unit and year, including intervening years affected by changed capital losses or minimum-tax credits. Records may be entered in any order. The repayment amounts come from Income; do not enter a calculated credit.
        #[libertas_size(min = 1)]
        years: Vec<RepaymentTaxYear>,
        /// Claim-of-right eligibility and complete records confirmed
        /// Benefits were included in income under an apparent unrestricted right, and the repayment obligation was established after that year. All repayments are documented and entered once. No fraud, voluntary gift, unentered income repayment, net Social Security repayment, or unresolved joint-return ownership. Prior returns are complete calendar-year resident returns with only the ordinary sources listed, fixed standard deductions, no dependency claims, qualifying children, other deductions, credits apart from the entered earned-income credit, minimum-tax credit and calculated elderly credit, adjustments, preferential income beyond the entered dividend statements or other Chapter 1 special taxes beyond calculated AMT and the entered early-distribution tax. NIIT and payroll taxes are outside this historical credit computation. The 2025 enhanced senior deduction is calculated separately. Social Security uses the entered regular-method records. No later pre-2026 Social Security lump-sum election depends on these original-year amounts. Ordinary retirement distributions use the separate original-year source records; enter any permitted nontaxable Schedule R pensions in the elderly-credit facts. Railroad repayments must match original fully taxable components for the same recipient; contributory repayments involving tax-free recovery need further allocation. No unentered retirement basis, early-distribution tax, contribution deduction, excess contribution or required-distribution shortfall. Specified private-activity-bond interest is entered separately within total tax-exempt interest; no other AMT preference, adjustment, foreign credit or unentered minimum-tax-credit balance. Enter retained prior-year minimum-tax-credit sources separately when used on the original return. Supply consecutive continuation records for affected capital losses and minimum-tax credits. Other dependent later-year tax calculations require their own source calculation. Listed wages are ordinary earned wages, without combat-pay elections, inmate pay, deferred-compensation exclusions or self-employment. All previously determined tax and credits, including amendments or IRS changes, are reflected in the source records. Income-sensitive items other than supported regular-method Social Security, childless EIC, age-based Schedule R credit and enhanced senior deduction require further calculation. Check the original returns, repayment obligation and payment records to answer. No gives the repayment no tax benefit, with a notice.
        confirmed: bool,
    },
}

/// Repayment tax treatment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RepaymentMethod {
    /// Compare both methods
    Automatic,
    /// Claim the itemized deduction
    Deduction,
    /// Claim the credit
    Credit,
}

/// Original-year childless earned-income credit facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RepaymentEic {
    // This is missing original-year source data, not an unanswered eligibility gate.
    // A filed credit alone cannot recover home/dependency facts when the original
    // supporting records are unavailable. historical_eic keeps the filed credit
    // unchanged with WARN_REPAYMENT_EIC_ASSUMED_UNCHANGED; it never invents facts.
    /// Not entered
    /// The credit is then assumed unchanged by excluding the repaid benefits, with a note where it could change.
    NotSupplied,
    /// Original-year facts and filed credit
    Facts {
        /// Main home in the United States more than half the year
        /// For a joint return, both filers must meet this rule in the original year. Include qualifying extended military duty abroad under the EIC rules. No means no earned-income credit for that year.
        home: bool,
        /// Neither filer is another person's dependent or qualifying child
        /// Apply the original year's rules both before and after excluding the repaid benefits. Include applicable exceptions for a person who did not need to file and filed only to recover withholding or estimated payments. Whether anyone claimed you on their original-year return shows it. No means no earned-income credit for that year.
        independent: bool,
        /// Earned-income credit was permitted for that year
        /// Yes means no applicable two-year or ten-year ban and any required Form 8862 reinstatement was satisfied. No means the credit was disallowed for that year. Check the original return, any Form 8862 and IRS disallowance or reinstatement notices to answer.
        allowed: bool,
        /// Original earned-income credit, Form 1040 line 27
        /// The App recomputes this amount from the source facts and compares it with the filed amount, including later IRS or amended-return changes. Do not enter a repayment credit here.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        filed_credit: i64,
    },
}

/// Timely valid SSNs on the original return
/// Select the people on this original return who had an employment-valid SSN issued by its due date, including extensions. This one source fact supplies both EIC and the enhanced senior deduction; current-year SSN answers are not reused.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RepaymentSsns {
    // A filed return does not prove the SSNs' employment validity or issuance
    // dates. Unavailable historical SSA records cannot be inferred from today's
    // SSNs. historical_eic/senior use WARN_REPAYMENT_SSNS_ASSUMED_NONE and omit
    // the increase/deduction rather than supplying a negative source fact.
    /// Not entered
    /// The filed earned-income credit is assumed unchanged, and the enhanced senior deduction adds nothing to the comparison, with a note where they could change.
    NotSupplied,
    /// Neither filer
    Neither,
    /// You only
    Taxpayer,
    /// Current spouse only
    Spouse,
    /// Both filers
    Both,
}

impl RepaymentSsns {
    pub(crate) fn mask(self) -> Option<u8> {
        match self {
            Self::NotSupplied => None,
            Self::Neither => Some(0),
            Self::Taxpayer => Some(1),
            Self::Spouse => Some(2),
            Self::Both => Some(3),
        }
    }
}

/// Original-year elderly-credit pension records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RepaymentElderly {
    // Missing historical pension statements cannot be reconstructed from the
    // filed Schedule R credit: several pension/income combinations yield it.
    // historical_elderly omits that optional credit with
    // WARN_REPAYMENT_ELDERLY_ASSUMED_NONE when the source records are unavailable.
    /// Not entered
    /// No elderly credit is assumed in either calculation, with a note where it could apply. Otherwise provide the original year's complete nontaxable pension records, even when the list is empty.
    NotSupplied,
    /// Complete original-year pension records
    Facts {
        /// Nontaxable Schedule R pensions
        /// Enter each qualifying pension once for the original filing unit, including veterans' pensions. Social Security belongs in its separate original-year record. Taxable retirement distributions belong in their separate records. Do not include a pension payment’s tax-free return of investment here.
        exempt_pensions: Vec<crate::ExemptPension>,
    },
}

/// Original Social Security or equivalent railroad benefits
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RepaymentSocial {
    /// No benefits on the original return
    None,
    /// Regular-method benefit records
    Benefits {
        /// Original combined SSA-1099 and RRB-1099 box 5 net benefits
        /// Include all filers on this original return. Do not subtract Medicare premiums or workers' compensation offsets. Do not include children's benefits or non-equivalent railroad pension benefits. Negative original-year net benefits require their own repayment calculation.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Original taxable benefits, Form 1040 line 6b
        /// Reconciliation only: the App recomputes taxable benefits before and after excluding the repaid income. Do not freeze this amount in the revised return.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        filed_taxable: i64,
        /// Complete regular-method records confirmed
        /// No earlier-year lump-sum election, treaty treatment, non-equivalent railroad benefits, foreign or benefit exclusions, or other unentered adjustment. All applicable box 5 amounts belong to this original filing unit. Do not enter later-year receipts here. The original SSA-1099 and the return's benefits worksheet show it. No figures only the repayment deduction, not the credit, with a notice; that situation is not supported.
        ordinary: bool,
    },
}

/// Original-year distribution timing and additional tax
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RepaymentDistribution {
    /// Received at age 59½ or later
    Normal,
    /// Received before age 59½
    Early {
        /// Recipient on the original return
        owner: Owner,
        /// SIMPLE IRA within two years of first participation
        /// Select only for a SIMPLE IRA distribution subject to the 25% rate. Other eligible distributions use 10%.
        simple: bool,
        /// Apply the medical-expense exception
        /// The App applies the original return's eligible unreimbursed expenses above 7.5% of recomputed AGI once. Select No when no exception applies. Other exceptions need their source-fact calculations.
        medical: bool,
    },
}

/// Original-year early-distribution tax records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RepaymentEarly {
    /// No early distributions
    None,
    /// Complete original-year Form 5329 Part I records
    Facts {
        /// Eligible unreimbursed medical expenses paid in the original year
        /// Include expenses allowable under Schedule A even though this original return uses the standard deduction. Exclude reimbursements, HSA/FSA-funded amounts, subsidies and amounts otherwise excluded or deducted above the line. Enter each expense once for this original filing unit, not once per distribution. Enter zero when no medical exception is claimed.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        medical_expenses: i64,
        /// Original additional tax on early distributions
        /// Form 5329 Part I line 4, including both separate spouse forms and any directly reported 10% tax. Exclude other Form 5329 taxes. The App reproduces this source amount before refiguring it; it is not an entered repayment credit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        filed_tax: i64,
        /// Complete early-distribution records and exception facts confirmed
        /// Each distribution belongs to its selected original filer, was received before age 59½, and is subject to 10% or the selected SIMPLE 25% rate. The only exception claimed or available is the selected medical-expense exception. No inherited, Roth, commercial-annuity, governmental 457(b), separation-from-service, disability, education, first-home, insurance, disaster or other unentered exception or recapture. Expenses are eligible for this filing unit both before and after the repayment exclusion. No other retirement tax or deduction changes. The original Form 5329 shows it. No figures only the repayment deduction, not the credit, with a notice; that situation is not supported.
        confirmed: bool,
    },
}

/// Original-year ordinary retirement distribution
/// Enter each source once for the original filing unit. These records do not replace current-year Income entries. Distributions are not earned income for EIC; pension investment recovered tax-free is not a nontaxable Schedule R pension.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RepaymentRetirement {
    /// Original-year railroad pension components
    Railroad {
        /// Original recipient on this return
        owner: Owner,
        /// RRB-1099-R box 7 gross pension
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        gross: i64,
        /// Box 4 contributory amount included in gross
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        contributory: i64,
        /// Original-year Simplified Method source worksheet
        /// Absent when no recoverable employee investment remains or for a spouse/divorced-spouse annuity, which cannot recover the employee's cost. All dates, months and prior recovery refer to this original return's year. Recovery applies only to box 4, never boxes 5 or 6.
        /// [NullText]
        /// No employee investment remains recoverable for this annuity
        /// [SetText]
        /// The original-year Simplified Method worksheet is
        basis: Option<crate::PensionBasis>,
        /// Ordinary age-60-or-later railroad benefits with complete records
        /// Resident pension treatment, not disability wages, foreign/treaty income, unresolved shared-recipient cost, General Rule recovery or unresolved prior-year repayment. Same-year repayments are already netted in the RRB payment boxes. Reconcile the original filing unit's source records before calculating a claim-of-right credit. The original RRB-1099-R shows it. No figures only the repayment deduction, not the credit, with a notice; that situation is not supported.
        normal: bool,
    },
    /// Fully taxable traditional IRA distribution
    TraditionalIra {
        /// Form 1099-R box 1 gross distribution
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        gross: i64,
        /// Ordinary fully taxable distribution confirmed
        /// Received by a filer on this original return under the selected distribution treatment. No after-tax basis in any traditional, SEP or SIMPLE IRA, Roth or inherited account, rollover, conversion, qualified charitable distribution, special repayment, corrective distribution or other exclusion. Required distributions were satisfied; no excess-contribution tax or income-sensitive retirement adjustment apart from the entered early-distribution medical exception. Use the gross amount before withholding. The original 1099-R and Form 8606 history show it. No figures only the repayment deduction, not the credit, with a notice; that situation is not supported.
        normal: bool,
        /// Distribution timing
        #[libertas_default(Normal)]
        treatment: RepaymentDistribution,
    },
    /// Employer-plan pension with payer-determined taxable amount
    Pension {
        /// Form 1099-R box 1 gross distribution
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        gross: i64,
        /// Form 1099-R box 2a taxable amount
        /// Copy the payer-determined amount, including the full gross amount when fully taxable. Do not substitute a self-calculated amount or use an undetermined box 2b. The difference must be only a tax-free return of after-tax investment.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        taxable: i64,
        /// Ordinary employer-plan pension records confirmed
        /// Received by a filer on this original return under the selected distribution treatment. The payer correctly determined taxability. No IRA, Roth, commercial annuity, railroad pension, disability wages, rollover, conversion, special lump-sum election, employer-stock treatment, public-safety insurance exclusion, distribution repayment or unresolved basis. Required distributions were satisfied; no excess-contribution tax or income-sensitive retirement adjustment apart from the entered early-distribution medical exception. The original 1099-R shows it. No figures only the repayment deduction, not the credit, with a notice; that situation is not supported.
        normal: bool,
        /// Distribution timing
        #[libertas_default(Normal)]
        treatment: RepaymentDistribution,
    },
}

/// Original-year dividend and fund-distribution records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RepaymentDividends {
    /// No dividends or capital-gain distributions
    None,
    /// Original Forms 1099-DIV
    Statements {
        /// Total ordinary dividends, box 1a
        /// Includes qualified dividends and short-term gains distributed as ordinary dividends. Combine statements belonging to this original filing unit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        ordinary: i64,
        /// Qualified dividends within box 1a, box 1b
        /// Use only dividends eligible for preferential treatment after the original year's holding-period and related-payment rules. Not added to AGI a second time.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        qualified: i64,
        /// Total capital-gain distributions, box 2a
        /// Includes the separately identified box 2b and 2d components below. This is a payer-reported distribution, not an entered net gain from asset sales.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        gains: i64,
        /// Unrecaptured section 1250 gain, box 2b
        #[libertas_default(0)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        unrecaptured: i64,
        /// Collectibles gain, box 2d
        #[libertas_default(0)]
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        collectibles: i64,
        /// Complete source statements and qualification confirmed
        /// Amounts belong to this original filing unit. Qualified dividends satisfy applicable holding-period and related-payment rules. Sales and capital-loss carryovers belong in the separate original-year capital records. No nominee ownership, Form 2439, section 1202 exclusion, opportunity-fund deferral, section 199A dividends, investment-interest election or other unentered adjustment. Special gain components are distinct parts of box 2a. Exempt-interest dividends belong in tax-exempt interest, with their AMT portion identified there. The original 1099-DIVs show it. No figures only the repayment deduction, not the credit, with a notice; that situation is not supported.
        confirmed: bool,
    },
}
impl RepaymentDividends {
    pub(crate) fn amounts(&self) -> [i64; 5] {
        match *self {
            Self::None => [0; 5],
            Self::Statements { ordinary, qualified, gains, unrecaptured, collectibles, .. } => {
                [ordinary, qualified, gains, unrecaptured, collectibles]
            }
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let [ordinary, qualified, gains, unrecaptured, collectibles] = self.amounts();
        self.amounts().iter().all(|v| (0..=100000000000).contains(v))
            && qualified <= ordinary
            && unrecaptured + collectibles <= gains
    }
}

/// Original AMT Schedule D incoming capital losses
/// Copy the AMT worksheet's actual lines6 and14. These are retained source balances, not a repayment adjustment calculated by the user.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RepaymentAmtLosses {
    /// AMT short-term loss entering the original year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub short: i64,
    /// AMT long-term loss entering the original year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub long: i64,
}

/// Exclusion-only capital basis
/// Retained basis and wash-sale adjustment computed with permanent AMT exclusions only, without deferral adjustments. These are source facts for the same asset, proceeds and holding period.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RepaymentExclusionBasis {
    /// Basis using permanent exclusions only
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Disallowed exclusion-only wash-sale loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wash_sale: i64,
}

/// Historical capital sale
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RepaymentSale {
    /// Sale source facts
    pub sale: crate::FederalSale,
    /// Different exclusion-only basis facts
    /// Within a confirmed exclusion-only worksheet, absence means regular basis and wash-sale facts apply. Otherwise these records remain incomplete. Proceeds and holding period are never entered twice.
    /// [NullText]
    /// Regular basis and wash-sale facts also apply to the exclusion-only worksheet
    /// [SetText]
    /// Different exclusion-only basis and wash-sale facts are
    #[libertas_default($null)]
    pub exclusion: Option<RepaymentExclusionBasis>,
}

/// Exclusion-only incoming capital losses
/// Retained original-year balances computed with permanent exclusions only. Supplying this worksheet confirms every sale's exclusion-only basis is complete. An explicit zero differs from missing records. No unresolved minimum-tax-credit NOL or other source adjustment applies.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RepaymentExclusionLosses {
    /// Incoming exclusion-only short-term loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub short: i64,
    /// Incoming exclusion-only long-term loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub long: i64,
}

/// Original-year capital sales and incoming loss balances
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RepaymentCapital {
    /// No sales or incoming capital losses
    None,
    /// Source sales and capital-loss records
    Records {
        /// Capital-asset sales in the original year
        /// Reuse the source proceeds, basis, holding period, asset use and loss adjustments for each sale. Historical withholding is recorded only for source completeness; it is not a current payment or repayment credit.
        /// ----
        /// Original-year sale
        sales: Vec<RepaymentSale>,
        /// Incoming short-term loss carryover
        /// Original Schedule D line 6, as a positive amount. Enter the actual prior-year worksheet balance, not a calculated repayment adjustment.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        short_loss: i64,
        /// Incoming long-term loss carryover
        /// Original Schedule D line 14, as a positive amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        long_loss: i64,
        /// Different incoming AMT capital losses
        /// Leave absent only when retained records establish the same regular and AMT incoming losses. Enter explicit zero AMT balances when only regular losses remain. Each sale carries its own separate AMT basis and wash-sale adjustment.
        /// [NullText]
        /// Incoming AMT capital losses equal the regular losses
        /// [SetText]
        /// Different incoming AMT capital losses are
        #[libertas_default($null)]
        amt: Option<RepaymentAmtLosses>,
        /// Complete exclusion-only capital worksheet
        /// Supply when independent loss utilization must be recomputed for minimum-tax credit. This replaces fixed capital-exclusion adjustments; do not supply both methods.
        /// [NullText]
        /// Use fixed capital-exclusion adjustments instead of a separate worksheet
        /// [SetText]
        /// The complete exclusion-only capital worksheet is
        #[libertas_default($null)]
        exclusion: Option<RepaymentExclusionLosses>,
        /// Complete ordinary capital records confirmed
        /// Sales and incoming balances belong to this original filing unit, with resolved regular and AMT basis, gain/loss character and incoming balances. Basis, proceeds, wash-sale adjustments and holding periods are resolved. No unentered transaction, basis adjustment, minimum-tax-credit dependency, business property, short-held fund-loss recharacterization, foreign treatment, special election or unentered later-year dependency. The App follows changed capital losses through supplied consecutive returns of this same filing unit. The App reconciles actual 2025 worksheet balances with current capital history, compares the complete 2026 Chapter1 tax, and reports refigured future capital losses only when the credit method is selected. The original Schedule D and Form 8949 show it. No figures only the repayment deduction, not the credit, with a notice; that situation is not supported.
        confirmed: bool,
    },
}

/// Original return for repayment comparison
/// This historical worksheet currently covers 2023–2025 supported returns, including dividend distributions, ordinary retirement distributions, regular-method Social Security, childless earned-income credit, age-based elderly credit and the 2025 enhanced senior deduction. Other income-sensitive items require their own calculation. Original filed tax is a reconciliation check, not an entered credit. Original-year amounts include all sources for that filing unit, not just the repaid benefits.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct RepaymentTaxYear {
    /// Original tax year
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2023, max = 2025)]
    pub year: u32,
    /// People on that return
    pub filers: RepaymentFilers,
    /// Original filing status
    pub status: FederalStatus,
    /// Original total taxable wages
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
    /// Original taxable interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// Original dividend and capital-gain distribution statements
    #[libertas_default(None)]
    pub dividends: RepaymentDividends,
    /// Original capital sales and loss balances
    #[libertas_default(None)]
    pub capital: RepaymentCapital,
    /// Original tax-exempt interest
    /// Include Form 1040 line 2a. This affects Social Security provisional income and EIC investment income, but is not itself added to AGI. Include specified private-activity-bond interest in this total and identify its AMT portion separately below.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt_interest: i64,
    /// Specified private-activity-bond interest subject to AMT
    /// Included in total tax-exempt interest above, not added to regular AGI. Use the original statements' specified private-activity-bond interest and dividends subject to AMT, net of source-reported adjustments. Exclude qualifying 2009–2010 bond issues and other amounts not subject to AMT; no unentered bond-premium or basis adjustment.
    #[libertas_default(0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub private_activity_interest: i64,
    /// Original Social Security or equivalent railroad benefits
    #[libertas_default(None)]
    pub social: RepaymentSocial,
    /// Original-year retirement distributions
    /// Include ordinary IRA and employer-plan records for all filers on this original return. Leave empty only when none. Other retirement treatments require their own historical calculation.
    /// ----
    /// Retirement source
    pub retirement: Vec<RepaymentRetirement>,
    /// Original early-distribution tax records
    #[libertas_default(None)]
    pub early: RepaymentEarly,
    // The filed taxable benefits do not uniquely establish historical living
    // arrangements. If the underlying residence records are unavailable,
    // historical_social/elderly use WARN_REPAYMENT_LIVING_APART_ASSUMED and the
    // existing smaller-benefit comparison; null also covers a non-separate year.
    /// Lived apart from spouse for the entire original year
    /// Needed only where a separate original return's Social Security or elderly-credit calculation depends on it. With the unknown answer there, the App assumes the arrangement that matches the filed taxable Social Security and gives the smaller credit, with a note. Do not use current-year living arrangements.
    /// [NullText]
    /// Original-year living arrangements are unknown or not needed
    /// [SetText]
    /// Lived apart for the entire original year
    pub separate_lived_apart: Option<bool>,
    /// Original taxable governmental unemployment
    /// Amount included in Schedule 1 line 7, after any original-year repayment netting. The App excludes only the portion repaid in 2026 from the matching Income entries.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub benefits: i64,
    /// Original standard deduction from Form 1040 line 12
    /// Use the actual fixed standard deduction, including the original age and blindness additions. No dependent-filer limitation, itemizing or other deduction is included here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub standard: i64,
    /// Original Form 1040 income tax, line 22
    /// The App must reproduce this from the source income, deductions, applicable credits and original filing table. This line includes AMT, but excludes Schedule 2 Part II taxes such as NIIT and early-distribution tax. Enter the latter separately when applicable. Withholding, estimated payments and refunds are not income tax.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income_tax: i64,
    /// Permanent capital adjustments for minimum-tax credit
    /// Copy the capital-related permanent exclusion components from the original Form 8801 source records, excluding the standard deduction and private-activity interest already entered. Include signed amounts and their source descriptions. An empty supplied list confirms no permanent capital adjustment; absence leaves classification unresolved when regular and AMT capital sources differ. These fixed amounts may be reused when incoming capital balances are unchanged. The App also follows loss differences proven to arise solely from its known permanent deductions. Other changed loss histories need independent exclusion-only worksheets. Do not enter a computed tax or credit.
    /// [NullText]
    /// Permanent capital-adjustment records are unavailable
    /// [SetText]
    /// Permanent capital adjustments are
    pub capital_exclusions: Option<Vec<crate::MinimumTaxExclusion>>,
    /// Minimum-tax-credit source history
    /// Retained records from the year immediately before this original return. Leave absent when no credit was available or when the preceding return in this comparison already supplies its complete sources. The App calculates both contexts and propagates supported unused balances through consecutive returns.
    /// [NullText]
    /// No additional minimum-tax-credit history is needed
    /// [SetText]
    /// Earlier minimum-tax-credit source history is
    #[libertas_default($null)]
    pub minimum_tax: Option<crate::MinimumTaxHistory>,
    /// Original-year earned-income credit
    #[libertas_default(NotSupplied)]
    pub eic: RepaymentEic,
    /// Original-year timely SSNs
    #[libertas_default(NotSupplied)]
    pub ssns: RepaymentSsns,
    /// Original-year elderly-credit records
    #[libertas_default(NotSupplied)]
    pub elderly: RepaymentElderly,
}
impl RepaymentClaim {
    pub(crate) fn method(&self) -> Option<RepaymentMethod> {
        match self {
            Self::None => None,
            Self::Claim { method, .. } => Some(*method),
        }
    }
    pub(crate) fn select(&mut self, value: RepaymentMethod) {
        if let Self::Claim { method, .. } = self {
            *method = value;
        }
    }
}

impl RepaymentRetirement {
    pub(crate) fn amounts(&self) -> (i64, i64, bool) {
        match *self {
            Self::Railroad { gross, contributory, ref basis, normal, .. } => (
                gross,
                gross - basis.as_ref().map_or(0, |b| b.excluded(contributory)),
                normal && basis.as_ref().is_none_or(|b| b.eligible),
            ),
            Self::TraditionalIra { gross, normal, .. } => (gross, gross, normal),
            Self::Pension { gross, taxable, normal, .. } => (gross, taxable, normal),
        }
    }
}

impl RepaymentSale {
    pub(crate) fn exclusion_sale(&self) -> crate::FederalSale {
        let mut sale = self.sale.clone();
        sale.amt = self.exclusion.as_ref().map_or(crate::SaleAmt::SameAsRegular, |v| {
            crate::SaleAmt::Different {
                record: crate::CapitalSaleAmt {
                    term: None,
                    basis: v.basis,
                    wash_sale: v.wash_sale,
                },
            }
        });
        sale
    }
}

impl RepaymentCapital {
    fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Records { sales, short_loss, long_loss, amt, exclusion, .. } => {
                exclusion.as_ref().is_none_or(|v| {
                    [v.short, v.long].iter().all(|n| (0..=100000000000).contains(n))
                }) && amt.as_ref().is_none_or(|v| {
                    [v.short, v.long].iter().all(|n| (0..=100000000000).contains(n))
                })
                    // Temporary arithmetic exclusion: amounts accumulates short, long
                    // and collectible gains in i64 before capital-owner projection.
                    && sales.len() <= 100
                    && sales.iter().all(|sale| {
                        crate::federal_model::validate_sale(&sale.sale).is_ok()
                            && crate::federal_model::validate_sale(&sale.exclusion_sale()).is_ok()
                            && (sale.exclusion.is_none() || exclusion.is_some())
                    })
                    && [short_loss, long_loss].iter().all(|v| (0..=100000000000).contains(*v))
            }
        }
    }
    pub(crate) fn incoming(&self, alternative: bool) -> [i64; 2] {
        match self {
            Self::None => [0; 2],
            Self::Records { short_loss, long_loss, amt, .. } => {
                if alternative && let Some(v) = amt {
                    [dollars(v.short), dollars(v.long)]
                } else {
                    [dollars(*short_loss), dollars(*long_loss)]
                }
            }
        }
    }
}

impl RepaymentTaxYear {
    // Every original-year record the credit comparison needs is in its
    // supported ordinary form; otherwise only the deduction is figured.
    pub(crate) fn records_supported(&self) -> bool {
        !matches!(self.social, RepaymentSocial::Benefits { ordinary: false, .. })
            && !matches!(self.early, RepaymentEarly::Facts { confirmed: false, .. })
            && !matches!(self.dividends, RepaymentDividends::Statements { confirmed: false, .. })
            && !matches!(self.capital, RepaymentCapital::Records { confirmed: false, .. })
            && self.retirement.iter().all(|p| p.amounts().2)
    }
}

impl RepaymentClaim {
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let key =
            "MESSAGE_CHECK_THE_ORIGINAL_YEAR_RECORDS_OF_YOUR_REPAYMENT_COMPARISON_ONE_RECORD_PER";
        let Self::Claim { years, .. } = self else {
            return None;
        };
        // Tax structure: three supported historical years, at most two distinct
        // filing units per year; calculate rejects duplicate person/year records.
        if years.is_empty() || years.len() > 6 {
            return Some(ValidationIssue::new(key).at_field("years"));
        }
        for (index, y) in years.iter().enumerate() {
            if !(2023..=2025).contains(&y.year)
                || ![y.wages, y.interest, y.exempt_interest, y.benefits, y.standard, y.income_tax]
                    .iter()
                    .all(|value| (0..=100000000000).contains(value))
            {
                return Some(ValidationIssue::new(key).at_list_item("years", index));
            }
            if let Some(mut issue) =
                y.minimum_tax.as_ref().and_then(|value| value.issue_for_year(y.year - 1))
            {
                issue.key = key;
                return Some(issue.in_field("minimum_tax").in_list("years", index));
            }
            if !(y
                .capital_exclusions
                .as_ref()
                // Temporary receiver exclusion: at most three constructed source
                // rows join these in MinimumTaxHistory's current 100-row admission.
                .is_none_or(|v| v.len() <= 97 && v.iter().all(crate::MinimumTaxExclusion::valid))
                && y.capital.valid()
                && y.dividends.valid()
                && (0..=y.exempt_interest).contains(&y.private_activity_interest)
                // Temporary arithmetic exclusion: calculate accumulates native
                // gross/taxable columns before rounding and historical tax owners.
                && y.retirement.len() <= 100
                && y.retirement.iter().all(|p| {
                    let (gross, taxable, _) = p.amounts();
                    (0..=100000000000).contains(&gross)
                        && (0..=gross).contains(&taxable)
                        && match p {
                            RepaymentRetirement::Railroad { contributory, basis, .. } => {
                                (0..=gross).contains(contributory)
                                    && basis.as_ref().is_none_or(|b| b.valid_for_year(y.year))
                            }
                            _ => true,
                        }
                })
                && match &y.early {
                    RepaymentEarly::None => true,
                    RepaymentEarly::Facts { medical_expenses, filed_tax, .. } => {
                        [medical_expenses, filed_tax]
                            .iter()
                            .all(|v| (0..=100000000000).contains(*v))
                    }
                }
                && match &y.social {
                    RepaymentSocial::None => true,
                    RepaymentSocial::Benefits { amount, filed_taxable, .. } => {
                        (0..=100000000000).contains(amount) && (0..=*amount).contains(filed_taxable)
                    }
                }
                && match &y.elderly {
                    RepaymentElderly::NotSupplied => true,
                    RepaymentElderly::Facts { exempt_pensions } => {
                        // Temporary arithmetic exclusion: elderly recomputation
                        // sums exempt pension cents in i64 before dollars().
                        exempt_pensions.len() <= 100
                            && exempt_pensions.iter().all(|p| {
                                !p.label.trim().is_empty() && (0..=100000000000).contains(&p.amount)
                            })
                    }
                }
                && match &y.eic {
                    RepaymentEic::NotSupplied => true,
                    RepaymentEic::Facts { filed_credit, .. } => {
                        (0..=100000000000).contains(filed_credit)
                    }
                })
            {
                return Some(ValidationIssue::new(key).at_list_item("years", index));
            }
        }
        None
    }
}
