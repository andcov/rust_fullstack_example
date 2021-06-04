table! {
    posts (id) {
        id -> Integer,
        userid -> Integer,
        body -> Text,
        postingtime -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        country -> Varchar,
        age -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(posts, users,);
