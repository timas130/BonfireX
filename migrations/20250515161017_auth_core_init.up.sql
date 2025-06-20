create schema auth_core;

create table auth_core.users (
    id bigint not null generated always as identity primary key,
    email text null,
    permission_level int not null default 0,
    banned bool not null default false,
    active bool not null default false,
    email_verification_sent_at timestamptz null,
    email_verification_code text null unique,
    password text null,
    created_at timestamptz not null default now()
);

create unique index on auth_core.users (email) where email is not null;

create table auth_core.user_contexts (
    id bigint not null generated always as identity primary key,
    ip inet not null,
    user_agent text not null
);

create unique index on auth_core.user_contexts (ip, user_agent);

create table auth_core.login_attempts (
    id bigint not null generated always as identity primary key,
    user_id bigint not null references auth_core.users on delete cascade,
    user_context_id bigint not null references auth_core.user_contexts on delete restrict,
    status int not null,
    created_at timestamptz not null default now()
);

create table auth_core.sessions (
    id bigint not null generated always as identity primary key,
    user_id bigint not null references auth_core.users on delete cascade,
    login_attempt_id bigint null references auth_core.login_attempts on delete set null,
    last_user_context_id bigint not null references auth_core.user_contexts on delete restrict,
    access_token text not null,
    expires_at timestamptz not null,
    created_at timestamptz not null default now()
);

create unique index on auth_core.sessions (access_token);
