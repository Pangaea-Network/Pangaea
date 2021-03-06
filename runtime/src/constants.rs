
pub mod time {
    /// This determines the average expected block time that we are targeting.
    /// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
    /// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
    /// up by `pallet_aura` to implement `fn slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub use primitives::BlockNumber;
    pub const MILLISECS_PER_BLOCK: u64 = 6000;

    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;


}

pub mod currency {
    use primitives::Balance;
    pub const GAIA: Balance = 1_000_000_000_000;
    pub const DOLLARS: Balance = GAIA / 100;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLICENTS: Balance = CENTS/1_000;
    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
    }
}