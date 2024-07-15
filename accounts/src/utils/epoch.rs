use anchor_lang::solana_program::{clock::{Clock, Epoch}, sysvar::Sysvar};

pub trait EpochUtils {
    fn current_epoch() -> Epoch {
        Clock::get().unwrap().unix_timestamp as Epoch
    }
}

impl EpochUtils for Epoch {}