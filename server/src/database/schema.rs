table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        pass -> Varchar,
    }
}

table! {
    problems (id) {
        id  -> Int4,
        owner_id -> Int4,
        p_type -> Varchar,
        topic -> Varchar,
        tags -> Array<Varchar>,
        data -> Varchar,
    }
}
