diesel::table! {
    users (id) {
        id -> Int4,
        name -> Text,
        email -> Text,
        creation_date -> Timestamp,
        last_login_date -> Timestamp,
    }
}
