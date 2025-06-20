create schema auth_password_recovery;

create table auth_password_recovery.password_reset_requests (
    id bigint not null generated always as identity primary key,
    user_id bigint not null,
    token text not null,
    created_at timestamptz not null default now(),
    used_at timestamptz null
);

create unique index on auth_password_recovery.password_reset_requests (token);
