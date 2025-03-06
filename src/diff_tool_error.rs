
#[derive(Debug)]
pub enum DiffToolError {
    KbDBConnectingFailed(String),
    KbDbConnectionIsNothing(String),
    KbDbExecutionError(String),
    KbDbQueryError(String),
    KbDbPreparationError(String),
    KbDbGetRowError(String),
    KbDbRowDataPaseError(String),
    KbDbTransactionError(String),
}

