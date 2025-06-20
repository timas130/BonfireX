create schema notification;

create table notification.preferences (
    user_id bigint not null primary key,
    lang_id text not null,
    created_at timestamptz not null default now()
);

create table notification.notifications (
    id bigint not null generated always as identity primary key,
    user_id bigint not null,
    definition_id text not null,
    data jsonb not null,
    params bytea not null,
    read_at timestamptz null,
    created_at timestamptz not null default now()
);
