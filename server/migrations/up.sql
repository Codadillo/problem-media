CREATE TABLE users (
    id Serial PRIMARY KEY,
    name VarChar NOT NULL,
    pass VarChar NOT NULL,
    recommended_ids Int4[] NOT NULL DEFAULT ARRAY[]::Int4[]
);

CREATE TABLE problems (
    id Serial PRIMARY KEY,
    owner_id Int4 NOT NULL,
    p_type VarChar NOT NULL,
    topic VarChar NOT NULL,
    tags VarChar[] NOT NULL,
    prompt VarChar NOT NULL,
    data VarChar NOT NULL,
    recommendations Int4 NOT NULL DEFAULT 0,
    explanation VarChar NOT NULL
);
