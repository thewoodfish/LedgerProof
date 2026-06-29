mod xls;

use crate::models::transaction::RawTransaction;
use anyhow::Result;

pub fn parse_statement(file_bytes: &[u8]) -> Result<Vec<RawTransaction>> {
    xls::parse(file_bytes)
}
