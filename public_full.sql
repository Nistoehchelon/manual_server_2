--
-- PostgreSQL database dump
--

\restrict yTkRr4aTE9srEcFP1QkkpbaeRpTW23jUjHgur6RfrtSaPA16HmTbfGwdFcddMdU

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
-- Data for Name: session_notifications; Type: TABLE DATA; Schema: public; Owner: user123
--

COPY public.session_notifications (internal_id, session_id, notification_type, message) FROM stdin;
\.


--
-- Data for Name: sessions; Type: TABLE DATA; Schema: public; Owner: user123
--

COPY public.sessions (id, user_id, logged_in) FROM stdin;
6476886c96943		f
64719d212603a		f
64768a921a41e		f
64768abe20aeb		f
6473ac60e76be	6477d4dad63de	t
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: user123
--

COPY public.users (internal_id, public_inapp_id, username, password_hash, password_salt, bio, role_id) FROM stdin;
5	6473baafa6d23	123	d8814f88fbdd345bc0d42438f8fff732a843aece01053a4dda0b4f014d78e3e7	6aacd4d53a717e87c808abf28d3c69d8f30a59ebcfa7f07513c84ad9aa39fb0c5116ae09a30ccfdf10203bcebe75b6ee1293e0cadc0b38fbc1fbb05ccaa277fd	Default bio	1
6	64740300fa286	Lola	e302925606dc8cc7fba10dd9a2fe09bf4016f6194e814c7356573728db5df5aa	9c57fe8980ecab8609265520098a709225734579089784267f4d94609dede52f039b8e496ff4c7df6e7baadee48dd819ce3a533dda09bac1ea5e9433339e2d58	Default bio	1
7	6476848532dd6	1234	859fee5e8a7bf04367aaecd7e72a0032df87a1b91fda96a3e7ed453101e4823d	c7615c62e612e77faea941bce304043fc1f8ebcc3d3e5d89fcf1d12aae6ae8a70e77f2b715508e5a3a58b02b25b7d719837c8b09f27f263d9ea83c96803f7e85	Default bio	1
8	6476849a7ec3a	%3Cb%3Eb%3C%2Fb%3E	d13b23c46f07045dfe629bc4d7b05541708611b594b746c14f995000affceeef	6d4a95660da87f8021d2a8bc921bb91db45d83170275400b47d2c12263fc0347e0d026ab7fb4b2493e8c9d0c37294818111030f615caeb80262da2fff2055dd1	Default bio	1
9	647684aa20b67	2++213+++2313	c4ded9795680cc08f66ffefe61c882d45df9d025c300e889f74c27e3ee885070	da67549dc4e8931b97d5292e827f8764d2132e35f13c4ce2f05dc1af5adcaa0ecb31ff376091516ddc0fd94b4e88a72734e9334e7ccacd3e45085a282ed93329	Default bio	1
10	647684e7abf79	1++2	e84a02b5ae93385a625ece8f2bdb32a92968789248542452e452f5e6757fc093	d6f29e236f8588bea460c117614c1b6c57d7c14e05990e426ad5936c945dc255c0135f21c48d624fe42d562971fd07c49368c2502e2310e7393028527cb07be6	Default bio	1
11	647688e02b5d7	1 2 3 4 5	c237df2f5af23c2c8ccc658d9f4681d5664479315ec2ac5a69c406c4d27f8d6d	976b0c6c6cbfcfaa1d4a4163b78afe26fc58fe03b449103c1c6ec6e0ec3a531e2dca419aa84a61e16868de40cf46547b9a87dfbd105d8457e0d37d5fb640f196	Default bio	1
12	6477d4c6ea325	99   99	6bdaf135bde888b9a91662005e439e83848f3f99ae284d9251c06ea6ccfaaa66	5138a608e591a26c973c3c60a35d0f7eec347d84947765d3c1d0bd42751eb7c55586f49c73a16569927f5ffb2bbfc872e52e5d99952f45aa9f514a6afd4cd671	Default bio	1
13	6477d4dad63de	Lol 😊👌😘💕	a27d20986dc424678d9f2d7287986713494c98737030d3c1c31d988b76e534e3	105320cfbd752e429d5f27baf120869cf94598c5142dd3ec7d01af78f9a828abab0acc1646f29c47e0f9bf07e562afa1ea8d6948a14dc6a9e7e447936ad7cbb3	Default bio	1
\.


--
-- Name: session_notifications_internal_id_seq; Type: SEQUENCE SET; Schema: public; Owner: user123
--

SELECT pg_catalog.setval('public.session_notifications_internal_id_seq', 96, true);


--
-- Name: users_internal_id_seq; Type: SEQUENCE SET; Schema: public; Owner: user123
--

SELECT pg_catalog.setval('public.users_internal_id_seq', 13, true);


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

\unrestrict yTkRr4aTE9srEcFP1QkkpbaeRpTW23jUjHgur6RfrtSaPA16HmTbfGwdFcddMdU

