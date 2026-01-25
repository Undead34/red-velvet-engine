use serde::Serialize;

pub type CountryCode = String;
pub type BankRef = String;
pub type EntityType = String;
pub type KycLevel = String;
pub type Currency = String;

pub type Amount = f64; // por ahora
pub type Score = i16; // impacto al riesgo (+/-)

#[derive(Clone, Serialize, Copy, Debug, Default, PartialEq, Eq)]
pub enum Flag {
    #[default]
    Unknown,
    No,
    Yes,
}

impl Flag {
    pub fn is_yes(self) -> bool {
        matches!(self, Flag::Yes)
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Money {
    pub value: f64,
    pub ccy: Currency,
}
