// @generated automatically by Diesel CLI.

diesel::table! {
    event (id) {
        id -> Nullable<Integer>,
        date -> Date,
        description -> Text,
        person_id -> Integer,
    }
}

diesel::table! {
    message (id) {
        id -> Nullable<Integer>,
        event_id -> Integer,
        source -> Text,
        content -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Nullable<Integer>,
        full_name -> Text,
        birth_date -> Date,
        email -> Text,
        hashed_password -> Text,
        lang -> Text,
    }
}

diesel::joinable!(message -> event (event_id));

diesel::allow_tables_to_appear_in_same_query!(event, message, user,);
