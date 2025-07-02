// @generated automatically by Diesel CLI.

diesel::table! {
    axiom_verge2_states (raw_state) {
        raw_state -> Blob,
        progress -> Unsigned<Integer>,
        elapsed -> Unsigned<Integer>,
        time_since_visit -> Unsigned<Integer>,
        estimated_remaining -> Unsigned<Integer>,
        processed -> Bool,
        queued -> Bool,
        won -> Bool,
        hist -> Nullable<Blob>,
        prev -> Nullable<Blob>,
    }
}
