create schema auth_oauth_provider;

create table auth_oauth_provider.clients (
    id bigint not null generated always as identity primary key,
    owner_id bigint not null,
    client_id text not null unique,
    client_secret text not null,
    redirect_uris text[] not null,
    display_name text not null,
    privacy_url text null,
    tos_url text null,
    official boolean not null default false,
    allowed_scopes text[] not null default '{"openid", "profile", "email"}',
    enforce_code_challenge boolean not null default false,
    created_at timestamptz not null default now()
);

create table auth_oauth_provider.grants (
    id bigint not null generated always as identity primary key,
    client_id bigint not null references auth_oauth_provider.clients (id) on delete cascade,
    user_id bigint not null,
    scopes text[] not null,
    created_at timestamptz not null default now()
);

create unique index on auth_oauth_provider.grants (user_id, client_id);

create table auth_oauth_provider.flows (
    id bigint not null generated always as identity primary key,
    client_id bigint not null references auth_oauth_provider.clients (id) on delete cascade,
    grant_id bigint null references auth_oauth_provider.grants (id) on delete cascade,
    user_id bigint not null,
    redirect_uri text not null,
    scopes text[] not null,
    state text null,
    nonce text null,
    code_challenge text null,
    code_challenge_method text null,
    code text null,
    access_token text null,
    refresh_token text null,
    created_at timestamptz not null default now(),
    authorized_at timestamptz null,
    access_token_expires_at timestamptz null,
    refresh_token_expires_at timestamptz null
);

create or replace function auth_oauth_provider.merge_arrays(a1 anyarray, a2 anyarray) returns anyarray as
$$
select array_agg(x order by x) from (select distinct unnest($1 || $2) as x) s;
$$ language sql strict;

