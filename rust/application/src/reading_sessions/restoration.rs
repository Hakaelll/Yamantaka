use yamantaka_domain::{
    Failure, Result,
    reading_sessions::{ReaderSessionContext, restoration::ReaderSessionRestoration},
};

/// Decode only known fields. The governed history page retains the original
/// object, including extension fields; decoding does not rewrite stored data.
pub fn decode_session_context(json: Option<&str>) -> Result<Option<ReaderSessionRestoration>> {
    json.map(|value| {
        let context: ReaderSessionContext = serde_json::from_str(value).map_err(|_| {
            Failure::Invalid(
                "Stored reading context requires recovery; original data is preserved".into(),
            )
        })?;
        context.try_into()
    })
    .transpose()
}
