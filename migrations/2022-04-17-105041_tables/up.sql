create table questions
(
    id            integer not null
        constraint questions_pk
            primary key autoincrement,
    language      text    not null,
    question      text    not null,
    answer        text    not null,
    question_type text    not null
);