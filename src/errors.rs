use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShsError {
    #[error("invalid threshold: need 1 <= t <= n , got t={threshold} , n={num_shares}")]
    InvalidThreshold{
        threshold:usize,
        num_shares:usize,
    },
    #[error("insufficient shares: need {threshold} got {provided}")]
    InsufficientShares {
        threshold: usize,
        provided: usize,
    },
    #[error("invalid share format: {reason}")]
    InvalidShareFormat {
        reason: String,
    },
    #[error("reconstruction failed: {reason}")]
    ReconstructionFailed{
        reason:String,
    },
    #[error("secret must not be empty")]
    EmptySecret,
}