--
-- PostgreSQL database dump
--


-- Dumped from database version 18.3 (Ubuntu 18.3-1.pgdg24.04+1)
-- Dumped by pg_dump version 18.3 (Ubuntu 18.3-1.pgdg24.04+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
-- sqlx does not work with the select below so I will comment it out
-- SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;


-- Do a manual CREATE SCHEMA public 
-- if needed

--
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON SCHEMA public IS 'standard public schema';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: access_attributes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.access_attributes (
    primary_key bigint NOT NULL,
    name text NOT NULL,
    subject text NOT NULL,
    subject_id text NOT NULL
);


--
-- Name: access_attributes_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.access_attributes_primary_key_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: access_attributes_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.access_attributes_primary_key_seq OWNED BY public.access_attributes.primary_key;


--
-- Name: csrf_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.csrf_tokens (
    primary_key bigint NOT NULL,
    id text NOT NULL,
    session_id text NOT NULL,
    designated_purpose text NOT NULL,
    one_time_use boolean DEFAULT false NOT NULL
);


--
-- Name: csrf_tokens_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.csrf_tokens_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: csrf_tokens_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.csrf_tokens_primary_key_seq OWNED BY public.csrf_tokens.primary_key;


--
-- Name: session_custom_data; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session_custom_data (
    primary_key bigint CONSTRAINT custom_session_data_primary_key_not_null NOT NULL,
    name text CONSTRAINT custom_session_data_name_not_null NOT NULL,
    value text CONSTRAINT custom_session_data_value_not_null NOT NULL,
    session_id text CONSTRAINT custom_session_data_session_id_not_null NOT NULL
);


--
-- Name: custom_session_data_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.custom_session_data_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: custom_session_data_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.custom_session_data_primary_key_seq OWNED BY public.session_custom_data.primary_key;


--
-- Name: custom_site_variables; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.custom_site_variables (
    primary_key bigint CONSTRAINT site_pages_variables_primary_key_not_null NOT NULL,
    name text CONSTRAINT site_pages_variables_name_not_null NOT NULL,
    value text CONSTRAINT site_pages_variables_value_not_null NOT NULL
);


--
-- Name: forum_thread_message_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_thread_message_versions (
    primary_key bigint NOT NULL,
    id text NOT NULL,
    user_id text NOT NULL,
    content text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    message_id text NOT NULL
);


--
-- Name: forum_thread_messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_thread_messages (
    primary_key bigint NOT NULL,
    id text NOT NULL,
    thread_id text NOT NULL,
    user_id text NOT NULL
);


--
-- Name: forum_thread_messages_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_thread_messages_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_thread_messages_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_thread_messages_primary_key_seq OWNED BY public.forum_thread_messages.primary_key;


--
-- Name: forum_threads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forum_threads (
    primary_key bigint NOT NULL,
    id text NOT NULL,
    name text NOT NULL,
    forum_id text NOT NULL
);


--
-- Name: forum_threads_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forum_threads_primary_key_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forum_threads_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forum_threads_primary_key_seq OWNED BY public.forum_threads.primary_key;


--
-- Name: forums; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.forums (
    primary_key bigint NOT NULL,
    id text NOT NULL,
    name text NOT NULL,
    parent_id text NOT NULL
);


--
-- Name: forums_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.forums_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: forums_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.forums_primary_key_seq OWNED BY public.forums.primary_key;


--
-- Name: session_notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session_notifications (
    primary_key bigint NOT NULL,
    session_id text NOT NULL,
    notification_type text NOT NULL,
    message text NOT NULL
);


--
-- Name: session_notifications_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.session_notifications_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: session_notifications_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.session_notifications_primary_key_seq OWNED BY public.session_notifications.primary_key;


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sessions (
    primary_key bigint NOT NULL,
    id text NOT NULL,
    user_id text NOT NULL,
    logged_in boolean NOT NULL,
    previous_page_load_time_ms bigint DEFAULT 0 NOT NULL,
    previous_get_page text DEFAULT '/'::text NOT NULL
);


--
-- Name: sessions_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sessions_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sessions_primary_key_seq1; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sessions_primary_key_seq1
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sessions_primary_key_seq1; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.sessions_primary_key_seq1 OWNED BY public.sessions.primary_key;


--
-- Name: site_pages_variables_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.site_pages_variables_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: site_pages_variables_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.site_pages_variables_primary_key_seq OWNED BY public.custom_site_variables.primary_key;


--
-- Name: thread_message_versions_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.thread_message_versions_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: thread_message_versions_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.thread_message_versions_primary_key_seq OWNED BY public.forum_thread_message_versions.primary_key;


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    primary_key bigint NOT NULL,
    id text NOT NULL,
    username text NOT NULL,
    password_hash text NOT NULL,
    bio text NOT NULL
);


--
-- Name: users_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.users_primary_key_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.users_primary_key_seq OWNED BY public.users.primary_key;


--
-- Name: webhook_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.webhook_tokens (
    primary_key integer NOT NULL,
    token text NOT NULL,
    designated_purpose text NOT NULL
);


--
-- Name: webhook_tokens_primary_key_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.webhook_tokens_primary_key_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: webhook_tokens_primary_key_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.webhook_tokens_primary_key_seq OWNED BY public.webhook_tokens.primary_key;


--
-- Name: access_attributes primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_attributes ALTER COLUMN primary_key SET DEFAULT nextval('public.access_attributes_primary_key_seq'::regclass);


--
-- Name: csrf_tokens primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.csrf_tokens ALTER COLUMN primary_key SET DEFAULT nextval('public.csrf_tokens_primary_key_seq'::regclass);


--
-- Name: custom_site_variables primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_site_variables ALTER COLUMN primary_key SET DEFAULT nextval('public.site_pages_variables_primary_key_seq'::regclass);


--
-- Name: forum_thread_message_versions primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_thread_message_versions ALTER COLUMN primary_key SET DEFAULT nextval('public.thread_message_versions_primary_key_seq'::regclass);


--
-- Name: forum_thread_messages primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_thread_messages ALTER COLUMN primary_key SET DEFAULT nextval('public.forum_thread_messages_primary_key_seq'::regclass);


--
-- Name: forum_threads primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_threads ALTER COLUMN primary_key SET DEFAULT nextval('public.forum_threads_primary_key_seq'::regclass);


--
-- Name: forums primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forums ALTER COLUMN primary_key SET DEFAULT nextval('public.forums_primary_key_seq'::regclass);


--
-- Name: session_custom_data primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_custom_data ALTER COLUMN primary_key SET DEFAULT nextval('public.custom_session_data_primary_key_seq'::regclass);


--
-- Name: session_notifications primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_notifications ALTER COLUMN primary_key SET DEFAULT nextval('public.session_notifications_primary_key_seq'::regclass);


--
-- Name: sessions primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions ALTER COLUMN primary_key SET DEFAULT nextval('public.sessions_primary_key_seq1'::regclass);


--
-- Name: users primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users ALTER COLUMN primary_key SET DEFAULT nextval('public.users_primary_key_seq'::regclass);


--
-- Name: webhook_tokens primary_key; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webhook_tokens ALTER COLUMN primary_key SET DEFAULT nextval('public.webhook_tokens_primary_key_seq'::regclass);


--
-- Name: access_attributes access_attributes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_attributes
    ADD CONSTRAINT access_attributes_pkey PRIMARY KEY (primary_key);


--
-- Name: csrf_tokens csrf_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.csrf_tokens
    ADD CONSTRAINT csrf_tokens_pkey PRIMARY KEY (primary_key);


--
-- Name: session_custom_data custom_session_data_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_custom_data
    ADD CONSTRAINT custom_session_data_pkey PRIMARY KEY (primary_key);


--
-- Name: custom_site_variables custom_site_variables_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_site_variables
    ADD CONSTRAINT custom_site_variables_name_key UNIQUE (name);


--
-- Name: forum_thread_messages forum_thread_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_thread_messages
    ADD CONSTRAINT forum_thread_messages_pkey PRIMARY KEY (primary_key);


--
-- Name: forum_threads forum_threads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_threads
    ADD CONSTRAINT forum_threads_pkey PRIMARY KEY (primary_key);


--
-- Name: forums forums_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forums
    ADD CONSTRAINT forums_pkey PRIMARY KEY (primary_key);


--
-- Name: session_custom_data session_name_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_custom_data
    ADD CONSTRAINT session_name_unique UNIQUE (session_id, name);


--
-- Name: session_notifications session_notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_notifications
    ADD CONSTRAINT session_notifications_pkey PRIMARY KEY (primary_key);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (primary_key);


--
-- Name: custom_site_variables site_pages_variables_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.custom_site_variables
    ADD CONSTRAINT site_pages_variables_pkey PRIMARY KEY (primary_key);


--
-- Name: forum_thread_message_versions thread_message_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.forum_thread_message_versions
    ADD CONSTRAINT thread_message_versions_pkey PRIMARY KEY (primary_key);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (primary_key);


--
-- Name: webhook_tokens webhook_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webhook_tokens
    ADD CONSTRAINT webhook_tokens_pkey PRIMARY KEY (primary_key);


--
-- PostgreSQL database dump complete
--

