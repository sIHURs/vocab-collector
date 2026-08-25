#![forbid(unsafe_code)]

//! Product entities, rules, repository contracts, and application-facing view models.

mod models;
mod normalization;
mod repositories;
mod review;
mod views;

pub use models::*;
pub use normalization::*;
pub use repositories::*;
pub use review::*;
pub use views::*;
