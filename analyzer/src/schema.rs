// @generated automatically by Diesel CLI.

diesel::table! {
    db_states (raw_state) {
        raw_state -> Blob,
        progress -> Unsigned<Integer>,
        elapsed -> Unsigned<Integer>,
        time_since_visit -> Unsigned<Integer>,
        estimated_remaining -> Unsigned<Integer>,
        step_time -> Unsigned<Integer>,
        processed -> Bool,
        queued -> Bool,
        won -> Bool,
        hist -> Nullable<Tinyblob>,
        prev -> Nullable<Blob>,
    }
}
