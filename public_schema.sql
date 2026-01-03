--
-- PostgreSQL database dump
--

\restrict FVk1tzeG6MbcL1lTMxS00d8vCebCCOT2HVbwQsakx7f6wdaa4qU5hwkQSxsy7PK

-- Dumped from database version 18.1
-- Dumped by pg_dump version 18.1

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: public; Type: SCHEMA; Schema: -; Owner: pg_database_owner
--

CREATE SCHEMA public;


ALTER SCHEMA public OWNER TO pg_database_owner;

--
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: pg_database_owner
--

COMMENT ON SCHEMA public IS 'standard public schema';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: session_notifications; Type: TABLE; Schema: public; Owner: user123
--

CREATE TABLE public.session_notifications (
    internal_id bigint NOT NULL,
    session_id text NOT NULL,
    notification_type text NOT NULL,
    message text NOT NULL
);


ALTER TABLE public.session_notifications OWNER TO user123;

--
-- Name: session_notifications_internal_id_seq; Type: SEQUENCE; Schema: public; Owner: user123
--

CREATE SEQUENCE public.session_notifications_internal_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.session_notifications_internal_id_seq OWNER TO user123;

--
-- Name: session_notifications_internal_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: user123
--

ALTER SEQUENCE public.session_notifications_internal_id_seq OWNED BY public.session_notifications.internal_id;


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: user123
--

CREATE TABLE public.sessions (
    id text,
    user_id text,
    logged_in boolean
);


ALTER TABLE public.sessions OWNER TO user123;

--
-- Name: users; Type: TABLE; Schema: public; Owner: user123
--

CREATE TABLE public.users (
    internal_id bigint NOT NULL,
    public_inapp_id text CONSTRAINT users_app_id_not_null NOT NULL,
    username text NOT NULL,
    password_hash text NOT NULL,
    password_salt text NOT NULL,
    bio text,
    role_id bigint DEFAULT 1 NOT NULL
);


ALTER TABLE public.users OWNER TO user123;

--
-- Name: users_internal_id_seq; Type: SEQUENCE; Schema: public; Owner: user123
--

CREATE SEQUENCE public.users_internal_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_internal_id_seq OWNER TO user123;

--
-- Name: users_internal_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: user123
--

ALTER SEQUENCE public.users_internal_id_seq OWNED BY public.users.internal_id;


--
-- Name: session_notifications internal_id; Type: DEFAULT; Schema: public; Owner: user123
--

ALTER TABLE ONLY public.session_notifications ALTER COLUMN internal_id SET DEFAULT nextval('public.session_notifications_internal_id_seq'::regclass);


--
-- Name: users internal_id; Type: DEFAULT; Schema: public; Owner: user123
--

ALTER TABLE ONLY public.users ALTER COLUMN internal_id SET DEFAULT nextval('public.users_internal_id_seq'::regclass);


--
-- Name: session_notifications session_notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: user123
--

ALTER TABLE ONLY public.session_notifications
    ADD CONSTRAINT session_notifications_pkey PRIMARY KEY (internal_id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: user123
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (internal_id);


--
-- PostgreSQL database dump complete
--

\unrestrict FVk1tzeG6MbcL1lTMxS00d8vCebCCOT2HVbwQsakx7f6wdaa4qU5hwkQSxsy7PK

