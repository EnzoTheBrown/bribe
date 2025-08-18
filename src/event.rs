use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use strum_macros::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumString, Display)]
pub enum EventCategory {
    BirthEarlyYears,
    FamilyRelationships,
    EducationLearning,
    CareerWork,
    ResidenceRelocation,
    HealthWellbeing,
    FinanceWealth,
    CityCommunity,
    CultureSpirituality,
    TravelAdventure,
    HobbySportLeisure,
    DigitalTech,
    EnvironmentNature,
    LaterLife,
    PassingCommemoration,
    Unknown,
}
