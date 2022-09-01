#![cfg_attr(
    not(test),
    deny(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unwrap_used,
    )
)] // allow in tests
#![deny(clippy::unseparated_literal_suffix)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod clearing_house;
pub mod vamm;
