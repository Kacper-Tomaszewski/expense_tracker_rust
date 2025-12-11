use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: u32,
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MonthlyLimitData {
    pub general: f64,
    pub categories: HashMap<String, f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub transactions: Vec<Transaction>,
    pub limits: HashMap<String, MonthlyLimitData>,
    pub theme: String,
    pub language: String,
    pub currency: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            transactions: vec![],
            limits: HashMap::new(),
            theme: "light".to_string(),
            language: "pl".to_string(),
            currency: "PLN".to_string(),
        }
    }
}