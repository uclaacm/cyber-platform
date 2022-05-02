# Cyber Platform
Website and CTF platform for ACM Cyber written in Rust! Visit the currently deployed platform at [https://acmcyber.com/](https://acmcyber.com/).

## Building the Platform

```
cargo +nightly build --release
```

## Deploying the Platform

In order to deploy the Cyber Platform, you will need to set up a couple different dependencies.

**Dependencies:**
- Rust Binary
- Challenges Repository
- Postgres SQL Server

### Rust Binary
First, clone this repository. The instructions to build and prepare the Rust binary are above (see "Building the Platform"). Once you have built the binary and moved it to the appropriate relative location, now its time to construct the other dependencies.

### Challenges Repository
The cyber platform links to a challenge repository formatted like this [https://github.com/uclaacm/cyber-academy-f20](https://github.com/uclaacm/cyber-academy-f20). Follow the formatting of adding challenges and events in the instructions in the repository and update the ```ctf.toml``` file to have correct dates coresponding to when you want the platform to accept flags for challenges. Make sure this repository is then cloned to the same folder as the Rust Binary and cyber-platform repository.

### Postgres SQL Server
The cyber platform uses a Postgres SQL Server to store data for users.

### Finally Deploying
Once you have set up all of the above dependencies, run the following command (while changing appropriate spots for your specific dependencies) and you should have the platform deployed!

```
./scrap --repo ctf-after-dark-w21/ --static cyber-platform/static/ --port 8000 --uri postgres://postgres:password@localhost:5432/postgres
```

# Contact
If you have any questions about the cyber platform, please reach out to ACM Cyber (uclacyber@gmail.com).
