use std::collections::HashMap;
use chrono::offset::Utc;
use maud::{html, DOCTYPE, Markup, PreEscaped};
use r2d2_postgres::postgres::error::SqlState;
use r2d2_postgres::postgres::row::Row;
use warp::{any, body, reply, Filter, Reply, Rejection};
use warp::http::{Response, StatusCode};
use warp::reject::custom;
use warp::reply::with_header;
use warp::path::{end, path};

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
				link rel="icon" type="image/png" href="/static/favicon.png";
			}
			body {
				nav {
					a.banner href="/" {
						img src="/static/logo.svg";
						span { b { "ACM" } " Cyber" }
					}
					ul {
						li { a href="/events" { "Events" } }
						li { a href="/challenges" { "Challenges" } }
						li { a href="/scoreboard" { "Scoreboard" } }
						@if count > 0 {
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
	let titles = ["Fall GM", "File Analysis", "Packet Capture", "Memory Forensics", "DEFCON TALK", "Bruin Quest", "CTF After Dark"];
	let short_titles = ["fallgm", "file", "packet", "mems", "defcon", "bquest", "ctf"];
	let descriptions = [
		r#"
		<h1>Fall GM</h1>
		<h2>Week 1</h2>
		<p>Dip your toes <small>(or fully submerge, we won’t judge)</small> into the world of Cybersecurity by attending our <strong>Fall GM!</strong> </p> 
		<p> We’ll go over what ACM Cyber is all about--and why you should totally be a part of our amazing, <span role="img" aria-label="sparkle">✨</span>beginner-friendly<span role="img" aria-label="sparkle">✨</span> cybersecurity club.</p> 
		<p> And we have some fun challenges prepared that might make your brain cry but in a good way, we promise.  </p> 
		<p> They’re also <strong>Avatar: The Last Airbender</strong> themed, because we love you and want you to have fun.</p>
		"#,
		r#"
		<h1>File Analysis</h1>
		<h2>Cyber Academy Week 2</h2>
		<p>You use them all the time, they live on your computer, and they have names like .doc, .pdf, .jpg...</p> 
		<p> Believe or not, files hold a multitude of information in addition to the stuff you see when you simply click on it. </p> 
		<p>Whether you </p>
		<p><span>&nbsp;&nbsp;&nbsp;&nbsp;</span> - think you know everything files have to offer, </p>
		<p><span>&nbsp;&nbsp;&nbsp;&nbsp;</span> - or you’ve never opened a file in your entire life, </p>
		<p><span>&nbsp;&nbsp;&nbsp;&nbsp;</span> - <small>or you just want to check out ACM Cyber because you think we’re cool (and we are, we’re very cool)</small>, </p>
		<p>come to our <strong>first</strong> workshop of the year (!) and learn all there is to know about metadata, file carving, and all the fun little things your files might be trying to tell you.</p>
		"#,
		r#"
		<h1>Packets</h1>
		<h2>Cyber Academy Week 3</h2>
		<p>What makes a Wi-Fi network unsecure? </p> 
		<p> How can hackers steal your data traversing a network? </p> 
		<p> Learn how a simple Google search propagates through the Internet, and what tools <strong>anyone</strong> can use to analyze network traffic.  </p> 
		<p> Join us for a live demonstration as we use <strong>Wireshark</strong> to capture login information sent over HTTP (Hypertext Transfer Protocol)!
		</p>
		<h3>Make sure to install <a href="https://www.wireshark.org/download.html">Wireshark</a> beforehand!</h3>
		"#,
		r#"
		<h1>Memory Analysis</h1>
		<h2>Cyber Academy Week 5</h2>
		<p>What secrets can we find from your memory? </p> 
		<p> No, we’re not interested in whatever you’re hiding in your brain; we’re talking about the memory on your computer! </p> 
		<p> Whether it be the content of a <strong>text file, your Chrome history, or top secret password</strong> to your computer and social media accounts, if it’s been in the memory, we’ll find it!  </p> 
		<p> Want to know how? Attend our memory forensics workshop to learn all about analyzing RAM (Random-Access Memory) dumps and how to pick out the <strong>juicy secrets</strong> hiding in the bytes.<p>
		<h3>Check out our pre-workshop guide <a href="https://docs.google.com/document/d/1m41GtudLR5CPmSANWCeVlg2vEXKIYMxWo0YMFdPi85I/edit">here</a></h3>
		"#,
		r#"
		<h1>"What Makes a Hacker?"</h1>
		<h2>By DEFCON Special Guest Speaker Sanjana Sarda</h2>
		<h2>Week 6</h2>
		<p>Talk to the one and only, chaotic neutral, former president of ACM Cyber and speaker at DEFCON's IoT Village 2020, Sanjana Sarda!  <p>
		<h3>Officer Testimonials:</h3>
		<p>Have you ever wondered what it's like to talk to the face of God? - Jeffrey</p>
		<p> Queen of all things Cyber, our illustrious former empress - Aarin</p>
		<p> Sanjana is a tree - Ellie </p>
		<p> leet haxxor tree - William </p>
		<p><a href="https://youtu.be/0G2g-tLMK70">Link</a> to her talk at IoT Village from last summer :O<p>
		"#,
		r#"
		<h1>Bruin Quest</h1>
		<h2>An ACM Cyber x Hack x ICPC Collab!</h2>
		<h2>Week 7</h2>
		<p>Whether you cannot, or decide not to, leave your house and see what the UCLA campus has to offer, that is okay, because you can now live vicariously through the <strong>brain puzzles</strong> and <strong>creative prompts</strong> we’ve made purely for your enjoyment. </p> 
		<p> Explore a <strong>virtual UCLA campus</strong> and complete fun challenges to compete for points, prizes, and the glory of being <strong>Bruin Quest Champion</strong>, and who doesn’t love bragging rights for being the smartest, creative-est, and most Bruin-est person this campus has to offer?  </p> 
		<p> Brought to you by the combined efforts of <strong>Hack, ICPC, and Cyber</strong>, because three metaphorical club brains are much better than one.  </p> 
		<p> It’s basically a party at this point! (And you should definitely bring your friends.)</p>
		"#,
		r#"
		<h1>CTF After Dark</h1>
		<h2>Week 8-9</h2>
		<p>This is the moment you’ve been training for.</p> 
		<p>Get ready for some high octane, mind-boggling tests of hacking skill! Collaborate with your friends in our beginner friendly <strong>Capture The Flag</strong> (CTF) competition.</p> 
		<p>Challenges will be targeted at the skills that you’ve learned from our Fall workshops. Top teams get bragging rights as well as <strong>prizes</strong>. </p>
		<p>And don’t worry if you ever feel stuck or lonely. We’ll be in our discord channel, ready to answer any of your burning questions.
		</p>
		"#,
	];
	Ok(page("Events", html! {
		script src="https://ajax.googleapis.com/ajax/libs/jquery/3.5.1/jquery.min.js" {}
		h1 { "Cyber's Seasonal Almanac" }
		h2 { "Fall 2020 Edition" }
		div class="workshop-deet" id="deet" {
			@for (i, x) in titles.iter().enumerate() {
				@let title: String = x.to_string();
				@let desc = descriptions[i];
				div class="workshop-description" id=(title) {
					(PreEscaped(desc))
				}
			}
		}
		section class="challenges" {
			ul {
				li {
					input class="workshop" id="fallgm" name="ws" type="radio" value="Fall GM" {}
					label class="workshop-s" id="workshop-0" for="fallgm" { // id gives for, name gives group
						span {"Fall GM"}
					}
				}
				@for (i, x) in titles.iter().enumerate() {
					@let title: String = x.to_string();
					@let short: String = short_titles[i].to_string();
					@let index = i;
					@if index != 0 {
						@let group = match index {
							1 | 4 => "workshop-left workshop-left-1",
							2 | 5 => "workshop-left workshop-left-2",
							3 | 6 => "workshop-left workshop-left-3",
							_ => "workshop-left",
						};
						li {
							input class="workshop" id=(short) name="ws" type="radio" value=(title) {
								label for=(short) class="workshop-s" id="workshop-top" { 
									span {(title)}
								 }
								 label for=(short) class=(group) { 
									 span {(title)}
								 }
							}
						}
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
	if ctf.try_get("start").map(|start| now < start).unwrap_or(false) {
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
		style { "dialog{display:none;}dialog:target{display:block;}" }
		h1 { "Challenges" }
		section class="challenges" {
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
	if ctf.try_get("start").map(|start| now < start).unwrap_or(false) {
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
							th class="challenge" { span { (title) } }
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

fn make_profile(team: Option<Row>, error: Option<&str>) -> Markup {
	html! {
		h1 { "Profile" }
		section class="profile" {
			@if let Some(error) = error { p class="error" { (error) } }
			@match team {
				Some(team) => {
					@let name: String = team.get("name");
					@let email: String = team.get("email");
					form method="POST" {
						label {
							"Team Name: "
							input type="text" disabled="disabled" value=(name);
						}
						label {
							"Email: "
							input type="email" name="email" value=(email);
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
	let team = match client.query("SELECT name, email FROM scrap.team
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
				input type="email" name="email" placeholder="Email";
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
	if ctf.try_get("start").map(|start| now < start).unwrap_or(false) || 
		ctf.try_get("stop").map(|stop| now > stop).unwrap_or(false) {
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
	let team = match client.query("SELECT name, email FROM scrap.team
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
	let email = profile_form!(form.get("email"), "Email is required.", false);
	let password = profile_form!(form.get("password"), "", true);
	let current_password = profile_form!(form.get("current_password"), "Current password is required.", false);
	match client.execute("UPDATE scrap.team
		SET email=$2, hash=CASE WHEN ($3 != '') THEN crypt($3, gen_salt('bf')) ELSE hash END
		WHERE id=lookup($1)
		AND hash=crypt($4, hash)",
		&[&session, &email, &password, &current_password]) {
		Ok(n) if n > 0 => (),
		Ok(_) => return Ok(Response::builder()
			.status(StatusCode::UNAUTHORIZED)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Profile", make_profile(team, Some("Incorrect password.")), client, session)?)),
		Err(ref e) if e.code() == Some(&SqlState::UNIQUE_VIOLATION) => return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Profile", make_profile(team, Some("Email conflict.")), client, session)?)),
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
	let email = register_form!(form.get("email"), "Email is required.");
	let password = register_form!(form.get("password"), "Password is required.");
	if name.len() > 64 || !name.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
		return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Registration", make_register(Some("Invalid team name length or characters.")), client, session)?))
	}
	match client.execute("INSERT INTO scrap.team
		(name, email, hash) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
		&[name, email, password]) {
		Ok(_) => (),
		Err(ref e) if e.code() == Some(&SqlState::UNIQUE_VIOLATION) => return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.header("content-security-policy", "script-src 'none'")
			.body(make_body("Registration", make_register(Some("Team name or email conflict.")), client, session)?)),
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
		.header("set-cookie", format!("session={}; HttpOnly; SameSite=Lax; Max-Age=31536000", cookie))
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

pub fn run(port: u16, pool: ClientPool) {
	let client = any().map(move || pool.get().unwrap());
	let session = warp::cookie::optional("session")
		.map(|cookie: Option<String>| cookie.unwrap_or(String::new()));
	let invalid = warp::cookie::optional("invalid")
		.map(|cookie: Option<String>| cookie.unwrap_or(String::new()));
	let get = warp::get2().and(client.clone()).and(session.clone());
	let post = warp::post2().and(client.clone()).and(session.clone());
	let routes = get.clone().and(end()).and_then(get_home)
		.or(get.clone().and(path("challenges")).and(end()).and(invalid.clone()).and_then(get_challenges))
		.or(get.clone().and(path("scoreboard")).and(end()).and_then(get_scoreboard))
		.or(get.clone().and(path("profile")).and(end()).and_then(get_profile))
		.or(get.clone().and(path("register")).and(end()).and_then(get_register))
		.or(get.clone().and(path("login")).and(end()).and_then(get_login))
		.or(get.clone().and(path("events")).and(end()).and_then(get_almanac))
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
		.or(get.clone().and(path("logout")).and(end()).and_then(logout))
		.recover(error);
	warp::serve(routes).run(([127, 0, 0, 1], port));
}
