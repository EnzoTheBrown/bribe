// @generated automatically by Diesel CLI.

diesel::table! {
    EVENTS (id) {
        id -> Nullable<Integer>,
        date -> Date,
        description -> Text,
        person_id -> Integer,
        title -> Nullable<Text>,
        category -> Nullable<Text>,
    }
}

diesel::table! {
    MESSAGES (id) {
        id -> Nullable<Integer>,
        event_id -> Integer,
        source -> Text,
        content -> Text,
    }
}

diesel::table! {
    USERS (id) {
        id -> Nullable<Integer>,
        full_name -> Text,
        birth_date -> Date,
        email -> Text,
        hashed_password -> Text,
        lang -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    EVENTS,
    MESSAGES,
    USERS,
);
