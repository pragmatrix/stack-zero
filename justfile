up:
	cd migration && cargo run -- up

generate-jwt-secret:
	openssl rand -base64 32
