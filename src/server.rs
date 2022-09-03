
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use maud::{html, DOCTYPE, Markup, PreEscaped};
use r2d2_postgres::postgres::error::SqlState;
use r2d2_postgres::postgres::row::Row;
use warp::{any, body, reply, Filter, Reply, Rejection};
use warp::http::{Response, StatusCode};
use warp::reject::custom;
use warp::reply::with_header;
use warp::path::{end, path};
use regex::Regex;

use crate::database::{Client, ClientPool};

macro_rules! result {
	($expr:expr) => {
		match $expr {
			Ok(value) => value,
			Err(e) => return Err(custom(e)),
		}
	}
}

macro_rules! form {
	($field:expr, $title:expr, $error:expr, $page:ident, $client:ident, $session:ident) => {
		match $field {
			Some(value) if value.len() > 0 => value,
			_ => return Ok(Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(make_body($title, $page(Some($error)), $client, $session)?))
		}
	}	
}

fn make_body(page: &str, content: Markup, mut client: Client, session: String) -> Result<String, Rejection> {
	let count: i64 = result!(client.query("SELECT COUNT(*) as count FROM scrap.session
		WHERE cookie=$1",
		&[&session]))[0].get("count");
	let title: String = result!(client.query("SELECT title FROM scrap.ctf", &[]))[0].get(0);
	Ok(html! {
		(DOCTYPE)
		html {
			head {
				meta charset="utf-8";
				meta name="viewport" content="width=device-width, initial-scale=1";
				title { @if page.len() > 0 { (page) " | " } (title) }
				link rel="stylesheet" href="/static/style.css";
				link rel="icon" type="image/png" href="/static/logo.svg";
			}
			body {
				nav {
					a.banner href="/" {
						img src="/static/wordmark.svg" alt="ACM Cyber";
						// span { img src="/static/wordmark.svg"; }
					}
					ul {
						li { a href="/pbr" {"PBR"} }
						li { a href="/events" { "Events" } }
						li { a href="/challenges" { "Challenges" } }
						li { a href="/scoreboard" { "Scoreboard" } }
						li { a href="/internship" { "Internship" } }
						@if count > 0 {
							// li { a href="/rewards" { "Rewards" } }
							li { a href="/profile" { "Profile" } }
							li { a href="/logout" { "Logout" } }
						} @else {
							li { a href="/login" { "Login" } }
							li { a href="/register" { "Register" } }
						}
					}
				}
				main { (content) }
			}
		}
	}.into_string())
}

fn make_reply(body: String) -> impl Reply {
	reply::with_header(reply::html(body), "content-security-policy", "script-src 'self' https://ajax.googleapis.com/ajax/libs/jquery/3.5.1/jquery.min.js")
}

fn page(title: &str, content: Markup, client: Client, session: String) -> Result<impl Reply, Rejection> {
	Ok(make_reply(make_body(title, content, client, session)?))
}

fn get_home(mut client: Client, session: String) -> Result<impl Reply, Rejection> {
	let home: String = result!(client.query("SELECT home FROM scrap.ctf", &[]))[0].get("home");
	Ok(page("", html! {
		(PreEscaped(home))
	}, client, session)?)
}

fn get_almanac(mut client: Client, session: String) -> Result<impl Reply, Rejection> {
	let events = result!(client.query("SELECT
		id, title, short, date, description, link, slides, 
		CASE WHEN id = 1 THEN 1 ELSE 0 END AS is_first,
		CASE WHEN id % 2 = 0 THEN 1 ELSE 0 END AS is_even
		FROM scrap.event
		ORDER BY is_first DESC, is_even DESC, id ASC",
		&[]));
	Ok(page("Events", html! {
		script src="https://ajax.googleapis.com/ajax/libs/jquery/3.5.1/jquery.min.js" {}
		h1 { "Spring 2022 Events" }
		section class="events tiles" {
			
			ul {
				@if let Some((first_event, rest_events)) = &events.split_first() {
					@let first_short: String = first_event.get("short");
					@let first_slug: String = first_short.replace(" ", "-").to_lowercase();
					@let first_id: String = format!("{}-deet", first_slug);
					li {
						input class="workshop" id=(first_slug) name="ws" type="radio" value=(first_id) {}
						label class="workshop-0" for=(first_slug) { // id gives for, name gives group
							span {(first_short)}
							img src= {"/static/events/" (first_slug) ".svg"} alt=(first_short) {}
						}
					}
					@for event in rest_events.iter() {
						@let short: String = event.get("short");
						@let slug: String = short.replace(" ", "-").to_lowercase();
						@let id: String = format!("{}-deet", slug);
						li {
							input class="workshop" id=(slug) name="ws" type="radio" value=(id) {}
							label for=(slug) class="workshop-left" { 
								span {(short)}
								img src= {"/static/events/" (slug) ".svg"} alt=(short) {}
							}
						}
					}
				}
				@for _ in 0..3 {
					li {}
				}
			}
			div class="workshop-deet" id="deet" {
				@for event in &events {
					@let title: String = event.get("title");
					@let short: String = event.get("short");
					@let id: String = format!("{}-deet", short.replace(" ", "-").to_lowercase());
					@let description: String = event.get("description");
					@let date: String = event.get("date");
					@let link: String = event.get("link");
					@let slides: String = event.get("slides");
					div class="workshop-description" id=(id) {
						h1 { (title) }
						h3 { (date) }
						@if link == "" {
							h3 { "Facebook Event Page: Coming Soon!"}
						} @else {
							h3 { a href={ (link) } {"Facebook Event Page"} }
						}
						@if slides == "" {
							h3 { "Slides: Coming Soon!"}
						} @else {
							h3 { a href={ (slides) } {"Slides"} }
						}
						p { (PreEscaped(description)) }
					}
				}
			}
		}
		script src="/static/almanac.js" {}
	}, client, session)?)
}

fn get_challenges(mut client: Client, session: String, invalid: String) -> Result<impl Reply, Rejection> {
	let now = Utc::now();
	let ctf = &result!(client.query("SELECT start, stop FROM scrap.ctf", &[]))[0];
	if ctf.try_get("start").map(|start: DateTime<Utc>| now < start).unwrap_or(false) {
		return Ok(with_header(page("Challenges", html! {
			h1 { "Challenges" }
			p { "Challenges are not available." }
		}, client, session)?, "set-cookie", "invalid=; HttpOnly; SameSite=Lax; Max-Age=-1"));
	}
	let challenges = result!(client.query("SELECT
		slug, title, author, description, tags, challenge.solves, challenge.value,
		team.id IS NOT NULL AS authenticated,
		solved(team.solves, challenge.id) AS solved
		FROM scrap.challenge challenge
		LEFT JOIN scrap.team team ON team.id=lookup($1)
		WHERE enabled=true
		ORDER BY value ASC, slug ASC",
		&[&session]));
	Ok(with_header(page("Challenges", html! {
		h1 { "Challenges" }
		section class="challenges tiles" {
			ul {
				@for challenge in &challenges {
					@let slug: String = challenge.get("slug");
					@let title: String = challenge.get("title");
					@let author: String = challenge.get("author");
					@let description: String = challenge.get("description");
					@let tags: Vec<String> = challenge.get("tags");
					@let solves: i32 = challenge.get("solves");
					@let value: i32 = challenge.get("value");
					@let authenticated: bool = challenge.get("authenticated");
					@let solved: bool = challenge.get("solved");
					li {
						a solved?[solved] href={ "#" (slug) } {
							div {
								h1.value { (value) }
								p.tags {
									@for tag in &tags {
										span { (tag) }
									}
								}
							}
						}
						div class="modal-container" id=(slug) {
							dialog open="open" id=(slug) {
								h1 { (title) }
								p.value data=(value) { (value) }
								p.solves data=(solves) { (solves) }
								p.description { (PreEscaped(description)) }
								p.author { (author) }
								p.tags {
									@for tag in &tags {
										span { (tag) }
									}
								}
								@if authenticated && !solved {
									form method="POST" {
										input type="hidden" name="slug" value=(slug);
										input type="text" name="flag" placeholder=(
											if slug == invalid {
												"incorrect flag"
											} else {
												"flag{...}"
											});
										button type="submit" { "Submit" }
									}
								}
								a class="close" href="#!" { "Close" }
							}
							a class="modal-bg" href="#!" {}
						}
					}
				}
				@for _ in 0..3 {
					li {}
				}
			}
		}
	}, client, session)?, "set-cookie", "invalid=; HttpOnly; SameSite=Lax; Max-Age=-1"))
}

fn get_scoreboard(mut client: Client, session: String) -> Result<impl Reply, Rejection> {
	let now = Utc::now();
	let ctf = &result!(client.query("SELECT start, stop FROM scrap.ctf", &[]))[0];
	if ctf.try_get("start").map(|start: DateTime<Utc>| now < start).unwrap_or(false) {
		return Ok(page("Scoreboard", html! {
			h1 { "Scoreboard" }
			p { "Scoreboard is not available." }
		}, client, session)?);
	}
	let teams = result!(client.query("SELECT name, score, solves, ROW_NUMBER()
		OVER (ORDER BY score DESC, submit ASC) AS place FROM scrap.team ORDER BY score DESC, submit ASC", &[]));
	let challenges = result!(client.query("SELECT id, title FROM scrap.challenge
		WHERE enabled=true
		ORDER BY slug ASC", &[]));
	Ok(page("Scoreboard", html! {
		h1 { "Scoreboard" }
		section class="scoreboard" {
			table {
				thead {
					tr {
						th class="place" { "#" }
						th class="team" { "Team" }
						th class="score" { "Score" }
						@for challenge in &challenges {
							@let title: String = challenge.get("title");
							th class="challenge" { div { (title) } }
						}
					}
				}
				tbody {
					@for team in teams {
						@let name: String = team.get("name");
						@let solves: i64 = team.get("solves");
						@let score: i32 = team.get("score");
						@let place: i64 = team.get("place");
						tr {
							td class="place" { (place) }
							td class="team" { (name) }
							td class="score" { (score) }
							@for challenge in &challenges {
								@let id: i32 = challenge.get("id");
								@let mask: i64 = 1 << (id - 1);
								td class="challenge" solved?[mask & solves > 0];
							}
						}
					}
				}
			}
		}
	}, client, session)?)
}

fn get_internship(client: Client, session: String) -> Result<impl Reply, Rejection> {
	Ok(page("Internship", html! {
		h1 { "All About the Cyber Internship" }
		h2 { "Being a Cyber Intern"}
		p { "You might be wondering....what's it take to be a cyber intern? Do I have to be the greatest hacker of all time?" }
		p { "The answer is a resounding no. We want people who are excited to learn and teach about cyber to join us!"}
		p {" As a cyber intern you'll work alongside officers to conceptualize and build workshops. You will get to concoct your own challenges for both workshops and the CTF After Dark events. But don't worry! You will have a set of mentors (and the whole team) to help you along the way. "}
		p {" After a quarter of serving as a Cyber intern, you will *ascend* to officer."}
		br{}
		h2 { "Applying"}
		p {"If you are on the ACM general mailing list, you'll receive an email when all intern program applications (not just Cyber) are open. Use that link and answer the Cyber specific questions."}
		p {"From there you'll receive updates about next steps, likely an interview with a current officer. It is not a rigorous process and overall we want you to be 100% yourself. You don't need to be the world's up-and-coming white hat hacker to fit with the Cyber team :-)"}
		br{}
		h2 { "Do we get paid?"}
		p {"No"}
	}, client, session)?)
}

fn get_pbr(client: Client, session: String) -> Result<impl Reply, Rejection> {
	Ok(page("Psi Beta Rho", html! {
		h1 { "Psi Beta Rho" }
		h2 { "What is Psi Beta Rho?"}
		p { "Psi Beta Rho, also known as PBR, is UCLA's competetive cybersecurity team! We are a group of students who are passionate about learning more about cybersecurity and competing at CTFs. PBR attends a variety of different cybersecurity competitions throughout the year by we primarily compete in capture the flag (CTF) competitions." }
		br{}
		h2 { "What are CTFs?"}
		p {"CTFs or capture the flag competitions are jeapardy style cybersecurity competitions where teams compete to solve a variety of problems from various different categories. The goal of each challenge is to find a particular flag which is some sort of string which can be submitted to earn points. For an example of what CTF challenges look like, be sure to check out the Challenges page on this site."}
		br{}
		h2 { "Do I need to have a ton of cybersecurity experience?"}
		p { "No! Psi Beta Rho is open to all skill levels. Whether you are just launching your cybersecurity career or are already an experienced professional, PBR is for you!" }
		br{}
		h2 { "How can I join?" }
		p { "If you are on the ACM Cyber mailing list, we will make an announcement when applications to join PBR are open. We also run weekly practices that are open to any UCLA student so feel free to stop by to check out the team! For more information or to ask questions, be sure to join the ACM Cyber Discord." }
	}, client, session)?)
}

fn make_profile(team: Option<Row>, error: Option<&str>) -> Markup {
	html! {
		h1 { "Profile" }
		section class="profile" {
			@if let Some(error) = error { p class="error" { (error) } }
			@match team {
				Some(team) => {
					@let name: String = team.get("name");
					@let discord: String = team.get("discord");
					form method="POST" {
						label {
							"Team Name: "
							input type="text" disabled="disabled" value=(name);
						}
						label {
							"Discord: "
							input type="text" name="discord" value=(discord);
						}
						label {
							"Password: "
							input type="password" name="password" placeholder="Optional";
						}
						label {
							"Current Password: "
							input type="password" name="current_password";
						}
						button type="submit" { "Save" }
					}
				},
				None => {
					p class="not-logged-in" { "Log in to view your profile." }
				}
			}
		}
	}
}

fn get_profile(mut client: Client, session: String) -> Result<impl Reply, Rejection> {
	let team = match client.query("SELECT name, discord FROM scrap.team
		WHERE id=lookup($1)",
		&[&session]) {
		Ok(mut teams) => teams.pop(),
		Err(e) => return Err(custom(e)),
	};
	Ok(page("Profile", make_profile(team, None), client, session)?)
}

fn make_register(error: Option<&str>) -> Markup {
	html! {
		h1 { "Register" }
		section class="register" {
			@if let Some(error) = error { p class="error" { (error) } }
			form method="POST" {
				input type="text" name="name" placeholder="Team Name" maxlength="64" pattern="[ -~]+";
				input type="text" name="discord" placeholder="Discord handle (eg. cyber#1234)";
				input type="password" name="password" placeholder="Password";
				button type="submit" { "Register" }
			}
		}
	}
}

fn get_register(client: Client, session: String) -> Result<impl Reply, Rejection> {
	Ok(page("Register", make_register(None), client, session)?)
}

fn make_login(error: Option<&str>) -> Markup {
	html! {
		h1 { "Login" }
		section class="login" {
			@if let Some(error) = error { p class="error" { (error) } }
			form method="POST" {
				input type="text" name="name" placeholder="Team Name";
				input type="password" name="password" placeholder="Password";
				button type="submit" { "Log In" }
			}
		}
	}
}

fn get_login(client: Client, session: String) -> Result<impl Reply, Rejection> {
	Ok(page("Login", make_login(None), client, session)?)
}

fn make_admin(error: Option<&str>) -> Markup {
	html! {
		h1 { "Admin" }
		section {//class="login" {
			@if let Some(error) = error { p class="error" { (error) } }
			form method="POST" {
				input type="text" name="name" placeholder="Team Name";
				input type="text" name="tickets" placeholder="0";
				button type="submit" { "gib" }
			}
		}
	}
}

fn get_admin(mut client: Client, session: String) -> Result<impl Reply, Rejection> {
	let status: bool = result! (client.query("SELECT isAdmin from scrap.team 
	inner join scrap.session on scrap.team.id = scrap.session.team 
	where scrap.session.cookie=$1",
		&[&session]))[0].get("isAdmin");
	if status {
		Ok(page("Admin", make_admin(None), client, session)?)
	}
	else {
		Ok(page("Login", make_login(None), client, session)?)
	}
}

fn error(err: Rejection) -> Result<impl Reply, Rejection> {
	match err.status() {
		StatusCode::METHOD_NOT_ALLOWED => {
			Ok(Response::builder()
				.status(StatusCode::NOT_FOUND)
				.body("404 Page Not Found"))
		},
		_ => {
			Ok(Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body("500 Internal Server Error"))
		}
	}
}

fn submit(mut client: Client, session: String, form: HashMap<String, String>) -> Result<impl Reply, Rejection> {
	let now = Utc::now();
	let ctf = &result!(client.query("SELECT start, stop FROM scrap.ctf", &[]))[0];
	if ctf.try_get("start").map(|start: DateTime<Utc>| now < start).unwrap_or(false) || 
		ctf.try_get("stop").map(|stop: DateTime<Utc>| now > stop).unwrap_or(false) {
		return Ok(Response::builder()
			.header("location", "/challenges")
			.status(StatusCode::SEE_OTHER)
			.body("".to_string()));
	}
	let empty = String::new();
	let slug = form.get("slug").unwrap_or(&empty);
	let flag = form.get("flag").unwrap_or(&empty);
	let mut transaction = result!(client.transaction());
	let rows = result!(transaction.execute("UPDATE scrap.team team
		SET solves=update(team.solves, challenge.id), submit=NOW()
		FROM scrap.challenge challenge
		WHERE team.id=lookup($1)
		AND slug=$2 AND flag=$3
		AND NOT solved(team.solves, challenge.id)",
		&[&session, &slug, &flag])) as i32;
	if rows > 0 {
		result!(transaction.execute("UPDATE scrap.challenge
			SET solves=solves+$2
			WHERE slug=$1",
			&[&slug, &rows]));
		result!(transaction.execute("UPDATE scrap.team team
			SET score=COALESCE((SELECT SUM(challenge.value)
			FROM scrap.challenge challenge
			WHERE solved(team.solves, challenge.id)), 0)",
			&[]));
		result!(transaction.commit());
		return Ok(Response::builder()
			.header("location", "/challenges")
			.status(StatusCode::SEE_OTHER)
			.body("".to_string()));
	} else { 
		return Ok(Response::builder()
			.header("location", "/challenges")
			.header("set-cookie", format!("invalid={}; HttpOnly; SameSite=Lax", slug))
			.status(StatusCode::SEE_OTHER)
			.body("".to_string()));
	}
}

fn edit(mut client: Client, session: String, form: HashMap<String, String>) -> Result<impl Reply, Rejection> {
	let team = match client.query("SELECT name, discord FROM scrap.team
		WHERE id=lookup($1)",
		&[&session]) {
		Ok(mut teams) => teams.pop(),
		Err(e) => return Err(custom(e)),
	};
	macro_rules! profile_form {
		($field:expr, $error:expr, $optional:expr) => {
			match $field {
				Some(value) if value.len() > 0 || $optional => value,
				_ => return Ok(Response::builder()
					.status(StatusCode::BAD_REQUEST)
					.header("content-security-policy", "script-src 'none'")
					.body(make_body("Profile", make_profile(team, Some($error)), client, session)?)),
			}
		}
	}
	let discord = profile_form!(form.get("discord"), "Discord handle is required.", false);
	let password = profile_form!(form.get("password"), "", true);
	let current_password = profile_form!(form.get("current_password"), "Current password is required.", false);
	let re = Regex::new(r"^.{2,32}?#\d{4}$").unwrap();
	if !re.is_match(discord) {
		return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Profile", make_profile(team, Some("Invalid Discord handle.")), client, session)?))
	}
	match client.execute("UPDATE scrap.team
		SET discord=$2, hash=CASE WHEN ($3 != '') THEN crypt($3, gen_salt('bf')) ELSE hash END
		WHERE id=lookup($1)
		AND hash=crypt($4, hash)",
		&[&session, &discord, &password, &current_password]) {
		Ok(n) if n > 0 => (),
		Ok(_) => return Ok(Response::builder()
			.status(StatusCode::UNAUTHORIZED)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Profile", make_profile(team, Some("Incorrect password.")), client, session)?)),
		Err(ref e) if e.code() == Some(&SqlState::UNIQUE_VIOLATION) => return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Profile", make_profile(team, Some("Discord handle conflict.")), client, session)?)),
		Err(e) => return Err(custom(e)),
	}
	Ok(Response::builder()
		.header("location", "/profile")
		.status(StatusCode::SEE_OTHER)
		.body("".to_string()))
}

fn register(mut client: Client, session: String, form: HashMap<String, String>) -> Result<impl Reply, Rejection> {
	macro_rules! register_form {
		($field:expr, $error:expr) => {
			form!($field, "Registration", $error, make_register, client, session)
		}
	}
	let name = register_form!(form.get("name"), "Team name is required.");
	let discord = register_form!(form.get("discord"), "Discord handle is required.");
	let password = register_form!(form.get("password"), "Password is required.");
	if name.len() > 64 || !name.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
		return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Registration", make_register(Some("Invalid team name length or characters.")), client, session)?))
	}
	let re = Regex::new(r"^.{2,32}?#\d{4}$").unwrap();
	if !re.is_match(discord) {
		return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Registration", make_register(Some("Invalid Discord handle.")), client, session)?))
	}
	match client.execute("INSERT INTO scrap.team
		(name, discord, hash) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
		&[name, discord, password]) {
		Ok(_) => (),
		Err(ref e) if e.code() == Some(&SqlState::UNIQUE_VIOLATION) => return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Registration", make_register(Some("Team name or Discord handle conflict.")), client, session)?)),
		Err(e) => return Err(custom(e)),
	}
	Ok(Response::builder()
		.header("location", "/login")
		.status(StatusCode::SEE_OTHER)
		.body("".to_string()))
}

fn login(mut client: Client, session: String, form: HashMap<String, String>) -> Result<impl Reply, Rejection> {
	macro_rules! login_form {
		($field:expr, $error:expr) => {
			form!($field, "Login", $error, make_login, client, session)
		}
	}
	let name = login_form!(form.get("name"), "Team name is required.");
	let password = login_form!(form.get("password"), "Password is required.");
	let id: i32 = match client.query("SELECT id FROM scrap.team
		WHERE name=$1 AND hash=crypt($2, hash)",
		&[name, password]) {
		Ok(teams) => match teams.get(0) {
			Some(team) => team.get("id"),
			None => return Ok(Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.header("content-security-policy", "script-src 'none'")
				.body(make_body("Login", make_login(Some("Invalid team name or password.")), client, session)?)),
		},
		Err(e) => return Err(custom(e)),
	};
	let cookie: String = match client.query("INSERT INTO scrap.session
		(team, cookie) VALUES ($1, gen_random_uuid())
		RETURNING cookie",
		&[&id]) {
		Ok(sessions) => sessions[0].get("cookie"),
		Err(e) => return Err(custom(e)),
	};
	Ok(Response::builder()
		.header("location", "/challenges")
		.header("set-cookie", format!("session2={}; HttpOnly; SameSite=Lax; Max-Age=86400", cookie))
		.status(StatusCode::SEE_OTHER)
		.body("".to_string()))
}

fn logout(mut client: Client, session: String) -> Result<impl Reply, Rejection> {
	match client.execute("DELETE FROM scrap.session
		WHERE cookie=$1",
		&[&session]) {
		Ok(_n) => Ok(Response::builder()
			.header("location", "/")
			.status(StatusCode::SEE_OTHER)
			.body("".to_string())),
		Err(e) => return Err(custom(e)),
	}
}

fn gib_tickets(mut client: Client, session: String, form: HashMap<String, String>) -> Result<impl Reply, Rejection> {
	macro_rules! admin_form {
		($field:expr, $error:expr) => {
			form!($field, "Admin", $error, make_admin, client, session)
		}
	}
	let name = admin_form!(form.get("name"), "Team name is required.");
	let tickets = admin_form!(form.get("tickets"), "Number of Tickets are required.");
	let num_tickets: i32 = tickets.parse::<i32>().unwrap();
	match client.execute("UPDATE scrap.team SET premium_tickets=premium_tickets+$2
		WHERE name=$1",
		&[&name, &num_tickets]) {
			Ok(_n) => Ok(Response::builder()
				.header("location", "/admin")
				.status(StatusCode::SEE_OTHER)
				.body("gibben".to_string())),
			Err(e) => return Err(custom(e)),
		}
}

pub fn run(port: u16, pool: ClientPool) {
	let client = any().map(move || pool.get().unwrap());
	let session = warp::cookie::optional("session2")
		.map(|cookie: Option<String>| cookie.unwrap_or(String::new()));
	let invalid = warp::cookie::optional("invalid")
		.map(|cookie: Option<String>| cookie.unwrap_or(String::new()));
	let get = warp::get2().and(client.clone()).and(session.clone());
	let post = warp::post2().and(client.clone()).and(session.clone());
	let routes = get.clone().and(end()).and_then(get_home)
		.or(get.clone().and(path("challenges")).and(end()).and(invalid.clone()).and_then(get_challenges))
		.or(get.clone().and(path("scoreboard")).and(end()).and_then(get_scoreboard))
		.or(get.clone().and(path("internship")).and(end()).and_then(get_internship))
		.or(get.clone().and(path("pbr")).and(end()).and_then(get_pbr))
		.or(get.clone().and(path("profile")).and(end()).and_then(get_profile))
		.or(get.clone().and(path("register")).and(end()).and_then(get_register))
		.or(get.clone().and(path("login")).and(end()).and_then(get_login))
		.or(get.clone().and(path("events")).and(end()).and_then(get_almanac))
		.or(get.clone().and(path("admin")).and(end()).and_then(get_admin))
		.or(post.clone().and(path("challenges")).and(end())
			.and(body::content_length_limit(4096))
			.and(body::form()).and_then(submit))
		.or(post.clone().and(path("profile")).and(end())
			.and(body::content_length_limit(4096))
			.and(body::form()).and_then(edit))
		.or(post.clone().and(path("register")).and(end())
			.and(body::content_length_limit(4096))
			.and(body::form()).and_then(register))
		.or(post.clone().and(path("login")).and(end())
			.and(body::content_length_limit(4096))
			.and(body::form()).and_then(login))
		.or(post.clone().and(path("admin")).and(end())
			.and(body::content_length_limit(4096))
			.and(body::form()).and_then(gib_tickets))
		.or(get.clone().and(path("logout")).and(end()).and_then(logout))
		.recover(error);
	warp::serve(routes).run(([127, 0, 0, 1], port));
}
