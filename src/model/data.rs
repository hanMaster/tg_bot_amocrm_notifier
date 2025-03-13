use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Record {
    pub _links: Links,
    pub _embedded: Embedded,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Links {
    pub next: Option<Link>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Link {
    pub href: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Embedded {
    pub leads: Vec<Lead>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Lead {
    pub id: u64,
    pub name: String,
    pub created_at: i64,
    pub custom_fields_values: Vec<CustomField>,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct CustomField {
    pub field_id: u64,
    pub field_name: String,
    pub values: Vec<Val>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Val {
    pub value: FlexibleType,
    pub enum_id: Option<u64>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum FlexibleType {
    Str(String),
    Int(i64),
}

// Profitbase
#[derive(Deserialize, Debug, Clone)]
pub struct ProfitRecord {
    pub status: String,
    pub data: Vec<ProfitData>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ProfitData {
    pub number: String,
    #[serde(rename = "houseName")]
    pub house_name: String,
    #[serde(rename = "projectName")]
    pub project_name: String,
}
