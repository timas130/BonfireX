create schema translation;

create table translation.resources (
    id bigint not null generated always as identity primary key,
    path text not null unique,
    lang_id text not null,
    source text not null,
    modified_at timestamptz not null default now()
);
