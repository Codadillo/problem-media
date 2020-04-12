CREATE TABLE users (
    id Serial PRIMARY KEY,
    name VarChar NOT NULL,
    pass VarChar NOT NULL
);

CREATE TABLE problems (
    id Serial PRIMARY KEY,
    owner_id Int4,
    p_type VarChar NOT NULL,
    topic VarChar NOT NULL,
    tags VarChar[] NOT NULL,
    data VarChar NOT NULL
);
