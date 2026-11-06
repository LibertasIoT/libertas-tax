// Form 2555 retained earnings, election, qualifying-period and housing source facts.
// https://www.irs.gov/instructions/i2555
// https://www.irs.gov/publications/p54
// https://www.irs.gov/pub/irs-drop/n-26-25.pdf
use crate::common::Owner;
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Prior-year housing deduction record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HousingCarryover {
    /// 2025 Form 2555 line 46 housing deduction before income limit
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// 2025 Form 2555 line 48 amount deducted
    /// Exclude any carryover used from 2024. Only the unused 2025 amount can carry into 2026.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deducted: i64,
}

/// Earlier revocation of a foreign exclusion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ExclusionRevocation {
    /// No revocation effective for2021 through2025
    None,
    /// Revoked for a year from2021 through2025
    Revoked {
        /// First year the revocation applied to
        // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
        #[libertas_number(min = 2021, max = 2025)]
        year: i32,
        /// Which election
        elections: RevokedElections,
        /// The IRS consented to a new election for 2026
        consent: bool,
    },
}
/// Revoked election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum RevokedElections {
    /// Foreign earned income exclusion
    EarnedIncome,
    /// Housing exclusion
    Housing,
    /// Both
    Both,
}
impl ExclusionRevocation {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Revoked { year, .. } => (2021..=2025).contains(year),
        }
    }
    pub(crate) fn barred(&self) -> [bool; 2] {
        match self {
            Self::None => [false; 2],
            Self::Revoked { year, elections, consent } => {
                let barred = !consent && (2021..=2025).contains(year);
                [
                    barred && *elections != RevokedElections::Housing,
                    barred && *elections != RevokedElections::EarnedIncome,
                ]
            }
        }
    }
}

/// Form 2555 record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignEarnedClaim {
    /// Person filing this Form 2555
    pub owner: Owner,
    /// Qualifying test and period
    pub qualification: ForeignQualification,
    /// Foreign tax home throughout the period
    pub tax_home: bool,
    /// Earlier revocation and IRS consent, independently for each election
    pub revocation: ExclusionRevocation,
    /// Foreign earned income exclusion elected
    pub exclusion: bool,
    /// Direct prior-service pay, excluding all awards
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub prior_year_pay: i64,
    /// Optional prior-service exclusion and retained 2025 return facts
    /// [NullText]
    /// Do not claim an exclusion for pay earned in 2025
    /// [SetText]
    /// The exclusion for pay earned in 2025 uses these return records
    #[libertas_default($null)]
    pub prior_year_history: Option<PriorYearHistory>,
    /// Awards received in 2026, already included in the employee pay
    pub multi_year_pay: Vec<MultiYearPay>,
    /// Housing exclusion or deduction
    pub housing: ForeignHousing,
}

/// 2025 return facts for the prior-service exclusion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PriorYearHistory {
    /// Time of 2025 in your qualifying period (2025 Form 2555 line 31)
    // Calendar structure: qualifying days in 2025 cannot exceed its 365 days.
    #[libertas_number(min = 1, max = 365)]
    #[libertas_physical_unit("day")]
    pub prior_year_days: i32,
    /// Foreign earned income excluded on your 2025 Form 2555 line 42
    #[libertas_money("USD", 2)]
    // 2025 section 911 maximum foreign earned income exclusion ($130,000).
    #[libertas_number(min = 0, max = 13000000)]
    pub prior_year_excluded: i64,
}
impl PriorYearHistory {
    pub(crate) fn valid(&self) -> bool {
        (1..=365).contains(&self.prior_year_days)
            && (0..=13000000).contains(&self.prior_year_excluded)
    }
}

/// Bonus or stock award
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MultiYearPay {
    /// What it is
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub description: String,
    /// Amount received in 2026, already included in your employee pay
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Service time it was earned over, in all
    /// The full vesting or bonus service period. A partial calendar month counts only if it includes at least 15 days.
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("month")]
    pub months_total: i32,
    /// Time in 2026 with a foreign tax home and qualifying foreign services
    // Calendar structure: a service year contains at most twelve months.
    #[libertas_number(min = 0, max = 12)]
    #[libertas_physical_unit("month")]
    pub months_in_return_year: i32,
    /// Time in 2025 with a foreign tax home and qualifying foreign services
    /// Any exclusion for these months uses the retained 2025 records above. Portions earned before 2025 are received too late to exclude in 2026.
    // Calendar structure: a service year contains at most twelve months.
    #[libertas_number(min = 0, max = 12)]
    #[libertas_physical_unit("month")]
    pub months_in_prior_year: i32,
}
impl MultiYearPay {
    pub(crate) fn valid(&self) -> bool {
        (0..=100000000000).contains(&self.amount)
            // Positive native service months feed the existing Exact ratio;
            // subtracting either bounded service-year count stays within i32.
            && self.months_total >= 1
            && (0..=12).contains(&self.months_in_return_year)
            && (0..=12).contains(&self.months_in_prior_year)
            && self.months_in_return_year + self.months_in_prior_year <= self.months_total
    }
}

impl ForeignEarnedClaim {
    pub(crate) fn pay_valid(&self) -> bool {
        (0..=100000000000).contains(&self.prior_year_pay)
            && self.prior_year_history.as_ref().is_none_or(PriorYearHistory::valid)
            // Same employee_pay projection exclusion as the source-page check.
            && self.multi_year_pay.len() <= 10
            && self.multi_year_pay.iter().all(MultiYearPay::valid)
            && (!matches!(self.qualification, ForeignQualification::PriorYearOnly)
                || (self.exclusion
                    && self.prior_year_history.is_some()
                    && self.housing == ForeignHousing::None
                    && self.multi_year_pay.iter().all(|a| a.months_in_return_year == 0)))
    }

    // Keep exact rational service-year portions through the existing form-line
    // boundary. Direct prior pay and awards are disjoint accepted source facts.
    pub(crate) fn employee_pay(&self, gross: i64) -> (i64, i64) {
        use crate::kernel::ratio::exact::{dollars, ratio, whole};
        let direct = self.prior_year_pay;
        let mut current = whole(gross - direct);
        let mut prior = whole(direct);
        for award in &self.multi_year_pay {
            current -= whole(award.amount)
                * ratio(
                    i64::from(award.months_total - award.months_in_return_year),
                    i64::from(award.months_total),
                )
                .expect("validated service months");
            prior += whole(award.amount)
                * ratio(i64::from(award.months_in_prior_year), i64::from(award.months_total))
                    .expect("validated service months");
        }
        let exclusion = self.prior_year_history.as_ref().map_or(0, |p| {
            let limit = fraction(
                crate::year_constants::FOREIGN_EARNED_INCOME_EXCLUSION_2025_DOLLARS,
                i64::from(p.prior_year_days),
                365,
            );
            let room = positive(limit - crate::kernel::ratio::dollars(p.prior_year_excluded));
            phase::cap(dollars(&prior).expect("bounded prior pay"), [room])
        });
        (dollars(&current).expect("bounded current pay"), exclusion)
    }
}

/// Foreign housing claim
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignHousing {
    /// No housing claim
    None,
    /// Housing expenses and election
    Claim {
        /// Dated locations where housing expenses were incurred during 2026
        /// Include each stay once with inclusive dates. Dates must be inside your qualifying period and cannot overlap. Repeated stays in a location are combined before applying its limit. For a second-only adverse household, enter the second household's location here.
        // Calendar structure: dated entries are ordered and nonoverlapping within 2026 (365 days).
        #[libertas_size(min = 1, max = 365)]
        stays: Vec<ForeignHousingStay>,
        /// Eligible housing expenses during the qualifying period
        /// Reasonable rent, utilities other than telephone, property insurance, occupancy taxes, nonrefundable lease fees, furniture rental and residential parking. Exclude purchases, improvements, purchased furniture, meals, domestic labor, mortgage interest, deductible taxes and tax-free lodging. Enter only this claimant's expense share.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses: i64,
        /// Nontaxable housing allowances for this period
        /// U.S. Government or similar tax-free allowances intended to compensate you or your spouse for housing during this claim's qualifying period. Enter the allowance paid, not a calculated reduction. Do not include taxable employer pay or add this allowance to foreign wages. The App reduces the housing amount after the expense limit and base amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        nontaxable_allowances: i64,
        /// Prior-year self-employed housing deduction record
        /// No prior-year deduction is claimed without these records. Current-year deduction is used first; any unused 2025 remainder expires after 2026.
        /// [NullText]
        /// Do not claim a prior-year foreign housing deduction
        /// [SetText]
        /// The unused 2025 foreign housing deduction record is
        #[libertas_default($null)]
        prior: Option<HousingCarryover>,
        /// Household treatment when both spouses claim housing
        #[libertas_default(OnlyClaim)]
        household: ForeignHousingHousehold,
    },
}
impl ForeignHousing {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Claim { expenses, nontaxable_allowances, prior, stays, household, .. } => {
                (0..=100000000000).contains(expenses)
                    && (0..=100000000000).contains(nontaxable_allowances)
                    && prior.as_ref().is_none_or(|v| {
                        (0..=100000000000).contains(&v.amount)
                            && (0..=v.amount).contains(&v.deducted)
                    })
                    && match household {
                        ForeignHousingHousehold::Shared { total_expenses } => {
                            (0..=100000000000).contains(total_expenses)
                        }
                        ForeignHousingHousehold::SeparateReturnShared {
                            total_expenses,
                            spouse_expenses,
                            ..
                        } => [total_expenses, spouse_expenses]
                            .into_iter()
                            .all(|v| (0..=100000000000).contains(v)),
                        _ => true,
                    }
                    && !stays.is_empty()
                    && stays.len() <= 365
                    && stays.iter().all(|s| {
                        crate::dates::source_valid_date(s.starts_on)
                            && crate::dates::source_valid_date(s.ends_on)
                            && (20260101..=20261231).contains(&s.starts_on)
                            && (20260101..=20261231).contains(&s.ends_on)
                    })
            }
        }
    }
}

/// Spouse housing treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignHousingHousehold {
    /// Second foreign household because of adverse living conditions
    Adverse {
        /// Eligible spouse or dependent household, with no duplicate claim
        /// One additional foreign household is maintained for your spouse or dependents because conditions at your tax home are dangerous, unhealthful or otherwise adverse. Minor dependents are in your legal or joint custody. Include its eligible costs in the housing expenses above; exclude tax-free employer lodging. No other person claims these costs or separately claims housing for this spouse. Only general-limit locations are currently supported for this combined-household calculation. No leaves this housing claim out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
    /// Adverse-condition second household in the same listed high-cost locality
    AdverseSameLocation {
        /// Listed locality of the spouse or dependent household
        /// The second abode must be away from your tax home because of dangerous, unhealthful or otherwise adverse conditions, but both abodes must be in the same listed high-cost locality for every qualifying stay. Include both households' eligible costs once in Housing expenses above. One location limit and one base housing amount apply, not a separate limit for each abode. A second household in another locality needs further allocation review.
        second_location: ForeignHousingLocation,
        /// Complete adverse-condition and expense facts without a duplicate claim
        /// Confirm that one additional foreign household is maintained for your spouse or dependents; minor dependents are in your legal or joint custody. No other person claims these costs or separately claims housing for this spouse. Tax-free employer lodging is excluded from the combined expenses. No leaves this housing claim out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
    /// Only one spouse claims this household's housing
    /// Use the claimant's qualifying period and include only expenses eligible during that period. The other spouse does not claim these expenses.
    OnlyClaim,
    /// Both spouses separately claim expenses of the same household
    Shared {
        /// Total eligible household expenses before allocation
        /// Enter the same total on both spouse entries and enter each spouse's chosen share as that person's housing expenses. Shares cannot exceed this total together; each share must relate to its claimant's qualifying period. Each claimant uses their own full base amount. This is a user allocation, not an automatic choice of the lowest tax.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        total_expenses: i64,
    },
    /// Shared household with spouse filing a separate return
    SeparateReturnShared {
        /// Total eligible shared household expenses before allocation
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        total_expenses: i64,
        /// Expenses allocated to the spouse's separate housing claim
        /// From the agreed expense allocation, not the spouse's computed exclusion. Your expenses plus this amount cannot exceed the household total. Each spouse uses their own qualifying period and full base amount.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse_expenses: i64,
        /// Reconciled allocation with no expenses claimed twice
        /// Your spouse's expense records show it. No leaves this housing claim out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
    /// Spouses maintain separate households for separate tax homes
    Separate {
        /// Tax homes and each household are outside reasonable commuting distance of the other tax home
        /// Each spouse claims only their own household expenses. Both commuting-distance conditions must hold for both claims to be allowed. No leaves this housing claim out of the estimate with a notice; that situation is not supported. When the spouses' answers differ, both separate claims are left out.
        qualified: bool,
    },
    /// Adverse-condition second household has all eligible housing costs
    AdverseSecondOnly {
        /// Listed locality of your primary abode and foreign tax home
        /// The primary abode is in a different listed high-cost locality and has no housing expenses included in this claim. For example, employer lodging excluded under section 119 is not an eligible primary-abode expense.
        primary_location: ForeignHousingLocation,
        /// Listed locality where all included housing expenses were incurred
        /// Every housing stay and all entered expenses belong to this second household. The one limit is based on this locality; no simultaneous primary-locality limit is added.
        second_location: ForeignHousingLocation,
        /// Complete adverse-condition, expense and no-duplicate-claim facts
        /// Confirm that one additional foreign household is maintained for your spouse or dependents because conditions at your tax home are dangerous, unhealthful or otherwise adverse; minor dependents are in your legal or joint custody. You included no primary-household costs, tax-free lodging, or costs claimed by another person. A case with eligible costs in both different localities needs separate review. No leaves this housing claim out of the estimate with a notice; that situation is not supported.
        confirmed: bool,
    },
}

impl ForeignHousing {
    // A claim whose eligibility or household facts are not confirmed is left
    // out of the estimate.
    pub(crate) fn supported(&self) -> bool {
        use ForeignHousingHousehold::*;
        match self {
            Self::None => true,
            Self::Claim { household, .. } => match household {
                Adverse { confirmed }
                | AdverseSameLocation { confirmed, .. }
                | SeparateReturnShared { confirmed, .. }
                | AdverseSecondOnly { confirmed, .. } => *confirmed,
                Separate { qualified } => *qualified,
                OnlyClaim | Shared { .. } => true,
            },
        }
    }
}

/// One stay at a foreign housing location
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignHousingStay {
    /// Housing location
    pub location: ForeignHousingLocation,
    /// First qualifying day at this location in 2026
    #[libertas_date_only]
    pub starts_on: u32,
    /// Last qualifying day at this location in 2026, inclusive
    #[libertas_date_only]
    pub ends_on: u32,
}

/// Foreign housing location under IRS Notice 2026-25
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ForeignHousingLocation {
    /// Other location subject to the general housing limit
    General,
    /// Angola: Luanda
    AngolaLuanda,
    /// Argentina: Buenos Aires
    ArgentinaBuenosAires,
    /// Aruba: All cities
    ArubaAllCities,
    /// Australia: Melbourne
    AustraliaMelbourne,
    /// Australia: Sydney
    AustraliaSydney,
    /// Australia: Wollongong
    AustraliaWollongong,
    /// Bahamas, The: Nassau
    BahamasTheNassau,
    /// Bahrain: Bahrain
    BahrainBahrain,
    /// Belgium: Brussels
    BelgiumBrussels,
    /// Bermuda: Bermuda
    BermudaBermuda,
    /// Brazil: Sao Paulo
    BrazilSaoPaulo,
    /// Canada: Calgary
    CanadaCalgary,
    /// Canada: Montreal
    CanadaMontreal,
    /// Canada: Ottawa
    CanadaOttawa,
    /// Canada: Toronto
    CanadaToronto,
    /// Canada: Vancouver
    CanadaVancouver,
    /// Canada: Victoria
    CanadaVictoria,
    /// Cayman Islands: Grand Cayman
    CaymanIslandsGrandCayman,
    /// China: Beijing
    ChinaBeijing,
    /// China: Hong Kong
    ChinaHongKong,
    /// China: Shanghai
    ChinaShanghai,
    /// Colombia: Bogota
    ColombiaBogota,
    /// Colombia: All cities other than Bogota
    ColombiaAllCitiesOtherThanBogota,
    /// Curacao: Curacao
    CuracaoCuracao,
    /// Democratic Republic of the Congo: Kinshasa
    DemocraticRepublicOfTheCongoKinshasa,
    /// Denmark: Copenhagen
    DenmarkCopenhagen,
    /// Dominican Republic: Santo Domingo
    DominicanRepublicSantoDomingo,
    /// Estonia: Tallinn
    EstoniaTallinn,
    /// France: Garches, Paris, Sevres, Suresnes, and Versailles
    FranceGarches,
    /// France: Lyon
    FranceLyon,
    /// Germany: Berlin
    GermanyBerlin,
    /// Germany: Boeblingen, Ludwigsburg, Nellingen, and Stuttgart
    GermanyBoeblingen,
    /// Germany: Bonn and Wahn
    GermanyBonn,
    /// Germany: Cologne
    GermanyCologne,
    /// Germany: Garmisch-Partenkirchen and Oberammergau
    GermanyGarmischPartenkirchen,
    /// Germany: Gelnhausen and Hanau
    GermanyGelnhausen,
    /// Germany: Ingolstadt and Munich
    GermanyIngolstadt,
    /// Germany: Kaiserslautern, Landkreis, Pirmasens, Sembach, and Zweibrucken
    GermanyKaiserslautern,
    /// Germany: Mainz and Wiesbaden
    GermanyMainz,
    /// Guatemala: Guatemala City
    GuatemalaGuatemalaCity,
    /// Guinea: Conakry
    GuineaConakry,
    /// Holy See, The: Holy See, The
    HolySeeTheHolySee,
    /// India: Mumbai
    IndiaMumbai,
    /// India: New Delhi
    IndiaNewDelhi,
    /// Ireland: Dublin
    IrelandDublin,
    /// Israel: Beer Sheva
    IsraelBeerSheva,
    /// Israel: Jerusalem and West Bank
    IsraelJerusalem,
    /// Israel: Tel Aviv
    IsraelTelAviv,
    /// Italy: Genoa
    ItalyGenoa,
    /// Italy: La Spezia
    ItalyLaSpezia,
    /// Italy: Milan
    ItalyMilan,
    /// Italy: Naples
    ItalyNaples,
    /// Italy: Rome
    ItalyRome,
    /// Italy: Vicenza
    ItalyVicenza,
    /// Jamaica: Kingston
    JamaicaKingston,
    /// Japan: Gifu, Komaki, and Nagoya
    JapanGifu,
    /// Japan: Okinawa Prefecture
    JapanOkinawaPrefecture,
    /// Japan: Osaka-Kobe
    JapanOsakaKobe,
    /// Japan: Tokyo City
    JapanTokyoCity,
    /// Kazakhstan: Almaty
    KazakhstanAlmaty,
    /// Kuwait: Kuwait City
    KuwaitKuwaitCity,
    /// Kuwait: All cities other than Kuwait City
    KuwaitAllCitiesOtherThanKuwaitCity,
    /// Luxembourg: Luxembourg
    LuxembourgLuxembourg,
    /// Malaysia: Kuala Lumpur
    MalaysiaKualaLumpur,
    /// Malta: Malta
    MaltaMalta,
    /// Mexico: Mexico City
    MexicoMexicoCity,
    /// Netherlands: Amsterdam and Schiphol
    NetherlandsAmsterdam,
    /// Netherlands: The Hague
    NetherlandsTheHague,
    /// Oman: Muscat
    OmanMuscat,
    /// Poland: Krakow
    PolandKrakow,
    /// Poland: Warsaw
    PolandWarsaw,
    /// Portugal: Alverca and Lisbon
    PortugalAlverca,
    /// Qatar: Doha
    QatarDoha,
    /// Romania: Bucharest
    RomaniaBucharest,
    /// Russia: Moscow
    RussiaMoscow,
    /// Russia: Saint Petersburg
    RussiaSaintPetersburg,
    /// Saudi Arabia: Riyadh
    SaudiArabiaRiyadh,
    /// Singapore: Singapore
    SingaporeSingapore,
    /// Slovenia: Ljubljana
    SloveniaLjubljana,
    /// South Korea: Camp Colbern and Camp Mercer
    SouthKoreaCampColbern,
    /// South Korea: K-16, Kimpo Airfield, Seoul, and Suwon
    SouthKoreaK16,
    /// Spain: Barcelona
    SpainBarcelona,
    /// Spain: Madrid
    SpainMadrid,
    /// Switzerland: Bern
    SwitzerlandBern,
    /// Switzerland: Geneva
    SwitzerlandGeneva,
    /// Switzerland: Zurich
    SwitzerlandZurich,
    /// Taiwan: Taipei
    TaiwanTaipei,
    /// Tanzania: Dar Es Salaam
    TanzaniaDarEsSalaam,
    /// Thailand: Bangkok
    ThailandBangkok,
    /// Trinidad and Tobago: Port of Spain
    TrinidadAndTobagoPortOfSpain,
    /// Ukraine: Kiev
    UkraineKiev,
    /// United Arab Emirates: Abu Dhabi
    UnitedArabEmiratesAbuDhabi,
    /// United Arab Emirates: Dubai
    UnitedArabEmiratesDubai,
    /// United Kingdom: Basingstoke
    UnitedKingdomBasingstoke,
    /// United Kingdom: Bath
    UnitedKingdomBath,
    /// United Kingdom: Bracknell, High Wycombe, and Reading
    UnitedKingdomBracknell,
    /// United Kingdom: Caversham
    UnitedKingdomCaversham,
    /// United Kingdom: Cheltenham
    UnitedKingdomCheltenham,
    /// United Kingdom: Croughton
    UnitedKingdomCroughton,
    /// United Kingdom: Farnborough
    UnitedKingdomFarnborough,
    /// United Kingdom: Gibraltar
    UnitedKingdomGibraltar,
    /// United Kingdom: Harrogate and Menwith Hill
    UnitedKingdomHarrogate,
    /// United Kingdom: London
    UnitedKingdomLondon,
    /// United Kingdom: Loudwater
    UnitedKingdomLoudwater,
    /// United Kingdom: Surrey
    UnitedKingdomSurrey,
    /// United Kingdom: All cities* other than Aldermaston, Belfast, Birmingham, Bristol, Brough, Cambridge, Chelmsford, Chicksands, Dunstable, Edinburgh, Edzell, Fairford, Felixstowe, Ft. Halstead, Glenrothes, Gloucestershire, Greenham Common, Hythe, Kemble, Liverpool, Nottingham, Oxfordshire, Plymouth, Portsmouth, Rochester, Waterbeach, Welford, West Byfleet, and Wiltshire, which are subject to the generally applicable limitation *Excluding those cities previously listed
    UnitedKingdomAllCitiesOtherThanAldermaston,
    /// Venezuela: Caracas
    VenezuelaCaracas,
    /// Vietnam: Hanoi
    VietnamHanoi,
    /// Vietnam: Ho Chi Minh City
    VietnamHoChiMinhCity,
}
impl ForeignHousingLocation {
    pub(crate) fn limits(self) -> (i64, i64) {
        match self {
            Self::General => (
                crate::year_constants::FOREIGN_HOUSING_LIMIT_DOLLARS,
                crate::year_constants::FOREIGN_HOUSING_LIMIT_DAILY_CENTS,
            ),
            Self::AngolaLuanda => (84000, 23014),
            Self::ArgentinaBuenosAires => (56500, 15479),
            Self::ArubaAllCities => (46200, 12658),
            Self::AustraliaMelbourne => (40600, 11123),
            Self::AustraliaSydney => (65600, 17973),
            Self::AustraliaWollongong => (46000, 12306),
            Self::BahamasTheNassau => (49700, 13616),
            Self::BahrainBahrain => (48300, 13233),
            Self::BelgiumBrussels => (42900, 11753),
            Self::BermudaBermuda => (90000, 24658),
            Self::BrazilSaoPaulo => (56600, 15507),
            Self::CanadaCalgary => (45100, 12356),
            Self::CanadaMontreal => (52000, 14247),
            Self::CanadaOttawa => (50800, 13918),
            Self::CanadaToronto => (62700, 17178),
            Self::CanadaVancouver => (73400, 20110),
            Self::CanadaVictoria => (45800, 12548),
            Self::CaymanIslandsGrandCayman => (64193, 17587),
            Self::ChinaBeijing => (69000, 18904),
            Self::ChinaHongKong => (114300, 31315),
            Self::ChinaShanghai => (57001, 15617),
            Self::ColombiaBogota => (58700, 16082),
            Self::ColombiaAllCitiesOtherThanBogota => (49400, 13534),
            Self::CuracaoCuracao => (45800, 12548),
            Self::DemocraticRepublicOfTheCongoKinshasa => (42000, 11507),
            Self::DenmarkCopenhagen => (43704, 11974),
            Self::DominicanRepublicSantoDomingo => (45500, 12466),
            Self::EstoniaTallinn => (46600, 12767),
            Self::FranceGarches => (73600, 20164),
            Self::FranceLyon => (40700, 11151),
            Self::GermanyBerlin => (44100, 12082),
            Self::GermanyBoeblingen => (46100, 12630),
            Self::GermanyBonn => (42000, 11507),
            Self::GermanyCologne => (56200, 15397),
            Self::GermanyGarmischPartenkirchen => (40700, 11151),
            Self::GermanyGelnhausen => (45500, 12466),
            Self::GermanyIngolstadt => (51500, 14110),
            Self::GermanyKaiserslautern => (48100, 13178),
            Self::GermanyMainz => (50400, 13808),
            Self::GuatemalaGuatemalaCity => (42000, 11507),
            Self::GuineaConakry => (51300, 14055),
            Self::HolySeeTheHolySee => (49000, 13425),
            Self::IndiaMumbai => (67920, 18608),
            Self::IndiaNewDelhi => (56124, 15376),
            Self::IrelandDublin => (42600, 11671),
            Self::IsraelBeerSheva => (61800, 16932),
            Self::IsraelJerusalem => (49000, 13425),
            Self::IsraelTelAviv => (50800, 13918),
            Self::ItalyGenoa => (41800, 11452),
            Self::ItalyLaSpezia => (40400, 11068),
            Self::ItalyMilan => (73200, 20055),
            Self::ItalyNaples => (50300, 13781),
            Self::ItalyRome => (49000, 13425),
            Self::ItalyVicenza => (40900, 11205),
            Self::JamaicaKingston => (41200, 11288),
            Self::JapanGifu => (74300, 20356),
            Self::JapanOkinawaPrefecture => (41300, 11315),
            Self::JapanOsakaKobe => (90664, 24839),
            Self::JapanTokyoCity => (67300, 18438),
            Self::KazakhstanAlmaty => (48000, 13151),
            Self::KuwaitKuwaitCity => (64400, 17644),
            Self::KuwaitAllCitiesOtherThanKuwaitCity => (57700, 15808),
            Self::LuxembourgLuxembourg => (57900, 15863),
            Self::MalaysiaKualaLumpur => (46200, 12658),
            Self::MaltaMalta => (55100, 15096),
            Self::MexicoMexicoCity => (47900, 13123),
            Self::NetherlandsAmsterdam => (52900, 14493),
            Self::NetherlandsTheHague => (58400, 16000),
            Self::OmanMuscat => (41300, 11315),
            Self::PolandKrakow => (54900, 15041),
            Self::PolandWarsaw => (62300, 17068),
            Self::PortugalAlverca => (44800, 12274),
            Self::QatarDoha => (45888, 12572),
            Self::RomaniaBucharest => (41200, 11288),
            Self::RussiaMoscow => (108000, 29589),
            Self::RussiaSaintPetersburg => (60000, 16438),
            Self::SaudiArabiaRiyadh => (40000, 10959),
            Self::SingaporeSingapore => (86700, 23753),
            Self::SloveniaLjubljana => (51400, 14082),
            Self::SouthKoreaCampColbern => (54200, 14849),
            Self::SouthKoreaK16 => (44300, 12137),
            Self::SpainBarcelona => (40600, 11123),
            Self::SpainMadrid => (59700, 16356),
            Self::SwitzerlandBern => (82200, 22466),
            Self::SwitzerlandGeneva => (116900, 32027),
            Self::SwitzerlandZurich => (67218, 18416),
            Self::TaiwanTaipei => (46188, 12654),
            Self::TanzaniaDarEsSalaam => (44000, 12055),
            Self::ThailandBangkok => (59000, 16164),
            Self::TrinidadAndTobagoPortOfSpain => (54500, 14932),
            Self::UkraineKiev => (72000, 19726),
            Self::UnitedArabEmiratesAbuDhabi => (49687, 13613),
            Self::UnitedArabEmiratesDubai => (57174, 15664),
            Self::UnitedKingdomBasingstoke => (41099, 11260),
            Self::UnitedKingdomBath => (41000, 11233),
            Self::UnitedKingdomBracknell => (62100, 17014),
            Self::UnitedKingdomCaversham => (73800, 20219),
            Self::UnitedKingdomCheltenham => (54300, 14877),
            Self::UnitedKingdomCroughton => (45000, 12329),
            Self::UnitedKingdomFarnborough => (54700, 14986),
            Self::UnitedKingdomGibraltar => (44616, 12224),
            Self::UnitedKingdomHarrogate => (46600, 12767),
            Self::UnitedKingdomLondon => (68600, 18795),
            Self::UnitedKingdomLoudwater => (57400, 15726),
            Self::UnitedKingdomSurrey => (48402, 13261),
            Self::UnitedKingdomAllCitiesOtherThanAldermaston => (44200, 12110),
            Self::VenezuelaCaracas => (57000, 15616),
            Self::VietnamHanoi => (46800, 12822),
            Self::VietnamHoChiMinhCity => (42000, 11507),
        }
    }
}

/// Qualifying foreign period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignQualification {
    /// Only pay for qualifying 2025 services received in 2026
    /// You have no qualifying 2026 foreign period. Enter the retained 2025 pay and return records below. No current-year foreign income or housing is excluded.
    PriorYearOnly,
    /// Bona fide resident of a foreign country
    /// For an uninterrupted period that includes a whole calendar year.
    Residence {
        /// Date your bona fide residence began (line 10)
        #[libertas_date_only]
        began: u32,
        /// Date it ended
        /// Leave empty if it continues.
        /// [NullText]
        /// Foreign residence continues
        /// [SetText]
        /// Foreign residence ended on
        #[libertas_date_only]
        ended: Option<u32>,
        /// Basis for your bona fide residence
        basis: ResidenceBasis,
        /// You told the foreign authorities you were not a resident and they did not tax you as one
        /// Yes means this residence test is not met; use the physical presence test if it fits.
        nonresident_statement: bool,
    },
    /// Physically present abroad 330 full days in 12 months
    Presence {
        /// First day of the 12-month period (line 16)
        /// Choose 12 consecutive months that include part of 2026 and in which you were abroad at least 330 full days; the period ends the day before the same date a year later.
        #[libertas_date_only]
        starts_on: u32,
        /// Full 24-hour periods physically in foreign countries within that period
        /// Count from your travel record (line 18). A full day starts at midnight; exclude days in the United States or its territories and travel over international waters. The test needs at least 330.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 366)]
        #[libertas_physical_unit("day")]
        foreign_days: i32,
    },
}

/// Basis of bona fide foreign residence
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ResidenceBasis {
    /// U.S. citizen
    Citizen,
    /// Resident alien who is a citizen or national of a country with a U.S. income tax treaty
    TreatyNational,
}
