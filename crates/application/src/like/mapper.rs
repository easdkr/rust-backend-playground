pub fn domain_from_record(
    record: infrastructure::persistence::seaorm::like::Model,
) -> crate::like::entity::Like {
    crate::like::entity::Like {
        id: record.id,
        post_id: record.post_id,
        user_id: record.user_id,
    }
}
