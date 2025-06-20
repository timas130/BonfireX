create schema auth_oauth;

create table auth_oauth.flows (
    id bigint not null generated always as identity primary key,
    issuer text not null,
    state text not null,
    nonce text not null,
    pkce_verifier text not null,
    created_at timestamptz not null default now()
);

create table auth_oauth.auth_sources (
    id bigint not null generated always as identity primary key,
    user_id bigint not null,
    issuer text not null,
    issuer_user_id text not null,
    created_at timestamptz not null default now()
);
