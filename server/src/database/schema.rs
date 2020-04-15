table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        pass -> Varchar,
        recommended_ids -> Array<Int4>,
    }
}

table! {
    problems (id) {
        id  -> Int4,
        owner_id -> Int4,
        p_type -> Varchar,
        topic -> Varchar,
        tags -> Array<Varchar>,
        prompt -> Varchar,
        data -> Varchar,
        recommendations -> Int4,
    }
}
